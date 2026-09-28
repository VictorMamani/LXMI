//! Builds immutable, LXMI-managed importer roots from verified upstream packages.
//! This module never writes to Steam, the game installation, compatdata or a prefix.
use crate::{
    filesystem::{missing, Directory},
    package::hash_file,
    *,
};
use lxmi_core::{GameDistribution, GameExecutableStatus, GameInstallationStatus};
use lxmi_runtime::{GameInstallationAssessment, GameRuntimePlan, PrefixAssessment};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET_GAME_ID: &str = "zenless-zone-zero";
const IMPORTER_FOLDER: &str = "ZZMI";
const MANAGED_RUNTIME_RELATIVE: &str = "runtimes/zenless-zone-zero/zzmi";
const ZZMI_LOADER_IDENTITY: &str = "XXMI Launcher.exe";
const LIBRARY_DLLS_IN_IMPORTER: &[&str] = &["d3d11.dll", "d3dcompiler_47.dll"];
static NEXT_ASSEMBLY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeComponent {
    Zzmi,
    XxmiLibraries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAssemblyFile {
    pub source_component: RuntimeComponent,
    pub source_package_id: String,
    pub source_relative_path: String,
    /// Relative to the managed importer root (`.../<runtime-id>/ZZMI`).
    pub destination_relative_path: String,
    pub size: u64,
    pub sha256: String,
    pub derived: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConfigPlan {
    pub source_relative_path: String,
    pub source_sha256: String,
    pub derived_sha256: String,
    pub target_process: String,
    pub target_evidence: String,
    /// This is a Windows process-name allowlist in upstream 3Dmigoto config.
    pub loader_process_identity: String,
    pub loader_identity_evidence: String,
    pub loader_identity_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAssemblyPlan {
    pub runtime_id: String,
    pub game_id: String,
    pub integration: IntegrationKind,
    /// Upstream `App.Root`; the relative importer path resolves to `app_root/ZZMI`.
    pub app_root: PathBuf,
    pub importer_root: PathBuf,
    pub import_path_from_app_root: String,
    pub zzmi_package_id: String,
    pub libraries_package_id: String,
    pub zzmi_version: Option<String>,
    pub libraries_version: Option<String>,
    pub target_process: String,
    pub loader_library_source: PathBuf,
    pub loader_library_sha256: String,
    pub config: RuntimeConfigPlan,
    pub files: Vec<RuntimeAssemblyFile>,
    pub directories_to_create: Vec<String>,
    pub evidence: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub schema_version: u32,
    pub runtime_id: String,
    pub game_id: String,
    pub integration: IntegrationKind,
    pub created_unix_seconds: u64,
    pub import_path_from_app_root: String,
    pub zzmi_package_id: String,
    pub libraries_package_id: String,
    pub zzmi_version: Option<String>,
    pub libraries_version: Option<String>,
    pub zzmi_release_tag: Option<String>,
    pub libraries_release_tag: Option<String>,
    pub target_process: String,
    pub loader_process_identity: String,
    /// Relative to `$XDG_DATA_HOME/lxmi`; the DLL remains in the verified Libraries package.
    pub loader_library_store_path: String,
    pub loader_library_sha256: String,
    pub files: Vec<RuntimeAssemblyFile>,
    pub directories: Vec<String>,
    pub configuration: RuntimeConfigPlan,
    pub evidence: Vec<String>,
    pub platform_compatibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedRuntime {
    pub runtime_id: String,
    pub app_root: PathBuf,
    pub importer_root: PathBuf,
    pub manifest_path: PathBuf,
    pub loader_library_path: PathBuf,
    pub zzmi_package_id: String,
    pub libraries_package_id: String,
    pub zzmi_version: Option<String>,
    pub libraries_version: Option<String>,
    pub target_process: String,
    pub loader_process_identity: String,
    pub file_count: usize,
    pub manifest: RuntimeManifest,
    pub newly_created: bool,
    pub installed_into_game: bool,
}

#[derive(Debug)]
struct PlannedBytes {
    file: RuntimeAssemblyFile,
    bytes: Option<Vec<u8>>,
}

/// Produces a reviewable assembly plan; it does not create or modify files.
pub fn plan_zzmi_assembly(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    zzmi: &VerifiedPackage,
    libraries: &VerifiedPackage,
    protected_paths: &[PathBuf],
) -> Result<RuntimeAssemblyPlan> {
    let (plan, _) = build_plan(store, platform, zzmi, libraries, protected_paths)?;
    Ok(plan)
}

/// Materializes an immutable assembly only below the LXMI XDG data root.
pub fn assemble_zzmi_runtime(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    zzmi_id: &str,
    libraries_id: &str,
    protected_paths: &[PathBuf],
) -> Result<ManagedRuntime> {
    tracing::info!(game = platform.game.id, "XXMI runtime assembly started");
    let zzmi = store.verify(zzmi_id)?;
    let libraries = store.verify(libraries_id)?;
    let (plan, pending) = build_plan(store, platform, &zzmi, &libraries, protected_paths)?;
    if let Some(existing) = read_runtime(&plan)? {
        tracing::info!(runtime_id = %existing.runtime_id, "Existing managed runtime verified");
        return Ok(existing);
    }

    let root = store.initialize()?;
    let staging = root.child("staging")?;
    let runtimes = root.private_child("runtimes")?;
    let game_root = runtimes.private_child("zenless-zone-zero")?;
    let integration_root = game_root.private_child("zzmi")?;
    if integration_root.metadata(&plan.runtime_id)?.is_some() {
        return read_runtime(&plan)?.ok_or_else(|| {
            XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(plan.app_root.clone()),
                "Ya existe un runtime con el mismo ID, pero su manifest o contenido no coincide.",
            )
        });
    }

    let stage_name = format!(
        "runtime-assembly-{}-{}",
        std::process::id(),
        NEXT_ASSEMBLY.fetch_add(1, Ordering::Relaxed)
    );
    let stage = staging.new_private_child(&stage_name)?;
    let result = (|| {
        let importer = stage.private_child(IMPORTER_FOLDER)?;
        for pending_file in &pending {
            validate_relative_path(&pending_file.file.destination_relative_path)?;
            importer.ensure_parents(&pending_file.file.destination_relative_path)?;
            let bytes = match &pending_file.bytes {
                Some(bytes) => bytes.clone(),
                None => source_bytes_for_pending(&zzmi, &libraries, pending_file)?,
            };
            if bytes.len() as u64 != pending_file.file.size
                || sha256(&bytes) != pending_file.file.sha256
            {
                return Err(XxmiError::new(
                    ErrorCode::ChecksumMismatch,
                    Some(pending_file.file.source_relative_path.clone().into()),
                    "El paquete fuente cambió después de calcular el plan.",
                ));
            }
            importer.write_new(&pending_file.file.destination_relative_path, &bytes)?;
        }
        importer.private_child("Mods")?;
        importer.sync()?;

        let created_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| XxmiError::new(ErrorCode::Io, None, "Reloj anterior a Unix epoch."))?
            .as_secs();
        let manifest = RuntimeManifest {
            schema_version: 1,
            runtime_id: plan.runtime_id.clone(),
            game_id: plan.game_id.clone(),
            integration: plan.integration,
            created_unix_seconds,
            import_path_from_app_root: plan.import_path_from_app_root.clone(),
            zzmi_package_id: plan.zzmi_package_id.clone(),
            libraries_package_id: plan.libraries_package_id.clone(),
            zzmi_version: plan.zzmi_version.clone(),
            libraries_version: plan.libraries_version.clone(),
            zzmi_release_tag: zzmi.manifest.upstream.as_ref().map(|u| u.tag.clone()),
            libraries_release_tag: libraries.manifest.upstream.as_ref().map(|u| u.tag.clone()),
            target_process: plan.target_process.clone(),
            loader_process_identity: plan.config.loader_process_identity.clone(),
            loader_library_store_path: format!(
                "packages/xxmi/{}/payload/3dmloader.dll",
                libraries.manifest.id
            ),
            loader_library_sha256: plan.loader_library_sha256.clone(),
            files: plan.files.clone(),
            directories: plan.directories_to_create.clone(),
            configuration: plan.config.clone(),
            evidence: plan.evidence.clone(),
            platform_compatibility: "unverified".into(),
        };
        let json = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
        stage.write_new("runtime-manifest.json", &json)?;
        stage.sync()?;
        staging.promote(&stage_name, &integration_root, &plan.runtime_id)?;
        let mut result = read_runtime(&plan)?.ok_or_else(|| {
            XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(plan.app_root.clone()),
                "El runtime promovido no superó su verificación posterior.",
            )
        })?;
        result.newly_created = true;
        tracing::info!(runtime_id = %result.runtime_id, "Managed XXMI runtime promoted");
        Ok(result)
    })();
    if result.is_err() {
        if let Err(error) = staging.remove_stage(&stage_name) {
            if !missing(&error) {
                tracing::warn!(code = ?error.code, "Incomplete runtime staging requires cleanup");
            }
        }
    }
    result
}

/// Returns an existing verified assembly without creating the runtime directory.
pub fn existing_zzmi_runtime(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    zzmi_id: &str,
    libraries_id: &str,
    protected_paths: &[PathBuf],
) -> Result<Option<ManagedRuntime>> {
    let zzmi = store.verify(zzmi_id)?;
    let libraries = store.verify(libraries_id)?;
    let (plan, _) = build_plan(store, platform, &zzmi, &libraries, protected_paths)?;
    read_runtime(&plan)
}

fn build_plan(
    store: &ManagedStore,
    platform: &GameRuntimePlan,
    zzmi: &VerifiedPackage,
    libraries: &VerifiedPackage,
    protected_paths: &[PathBuf],
) -> Result<(RuntimeAssemblyPlan, Vec<PlannedBytes>)> {
    if platform.game.id != TARGET_GAME_ID {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            None,
            "El ensamblador 0.5.3 solo materializa ZZMI para Zenless Zone Zero.",
        ));
    }
    let (game_path, game_library, executable) = match &platform.installation {
        GameInstallationAssessment::Detected {
            install_path,
            steam_library,
            executable_path: Some(executable),
            executable_status: GameExecutableStatus::Found,
            directory_status: GameInstallationStatus::Installed,
            distribution: GameDistribution::Steam,
            ..
        } => (install_path, steam_library, executable),
        _ => return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "Se necesita ZZZ detectado desde Steam, directorio presente y ejecutable reconocido.",
        )),
    };
    let target_process = executable
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::UnsafePath,
                Some(executable.clone()),
                "Nombre de ejecutable no válido.",
            )
        })?
        .to_owned();
    if !platform
        .game
        .expected_executable_names
        .contains(&target_process.as_str())
    {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            Some(executable.clone()),
            "El ejecutable observado no está en el registro de ejecutables de ZZZ.",
        ));
    }
    let executable_parent = executable.parent().ok_or_else(|| {
        XxmiError::new(
            ErrorCode::UnsafePath,
            Some(executable.clone()),
            "Ejecutable sin carpeta padre.",
        )
    })?;
    let executable_directory = Directory::open_absolute(executable_parent)?;
    let executable_metadata = executable_directory
        .metadata(target_process.as_str())?
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::NotFound,
                Some(executable.clone()),
                "El ejecutable detectado ya no existe.",
            )
        })?;
    if !executable_metadata.is_file() {
        return Err(XxmiError::new(
            ErrorCode::UnsafePath,
            Some(executable.clone()),
            "El target no es un archivo regular.",
        ));
    }

    if zzmi.manifest.kind != PackageKind::GameIntegration(IntegrationKind::Zzmi)
        || libraries.manifest.kind != PackageKind::XxmiLibraries
    {
        return Err(XxmiError::new(
            ErrorCode::WrongGameIntegration,
            None,
            "El runtime requiere ZZMI y XXMI Libraries como paquetes separados.",
        ));
    }
    require_official_package(zzmi, OfficialPackageKind::Zzmi, false)?;
    require_official_package(libraries, OfficialPackageKind::XxmiLibraries, true)?;

    let mut protected = protected_paths.to_vec();
    protected.extend([game_path.clone(), game_library.clone()]);
    if let PrefixAssessment::CandidateFound { path } = &platform.prefix {
        protected.push(path.clone());
    }
    if let lxmi_runtime::CompatDataAssessment::Found { path } = &platform.compatdata {
        protected.push(path.clone());
    }
    store.ensure_outside(&protected)?;

    let libraries_root = Directory::open_absolute(&libraries.payload_path)?;
    let loader_entry = libraries
        .manifest
        .files
        .iter()
        .find(|file| file.relative_path == "3dmloader.dll")
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::MissingRequiredFile,
                None,
                "XXMI Libraries no contiene 3dmloader.dll.",
            )
        })?;
    let (loader_size, loader_hash) = hash_file(
        &libraries_root,
        "3dmloader.dll",
        ImportLimits::default().max_file_bytes,
    )?;
    if loader_size != loader_entry.size || loader_hash != loader_entry.sha256 {
        return Err(XxmiError::new(
            ErrorCode::ChecksumMismatch,
            Some(libraries.payload_path.join("3dmloader.dll")),
            "El loader de Libraries no coincide con el inventario verificado.",
        ));
    }

    let zzmi_root = Directory::open_absolute(&zzmi.payload_path)?;
    let config_entry = zzmi
        .manifest
        .files
        .iter()
        .find(|file| file.relative_path == "d3dx.ini")
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::MissingRequiredFile,
                None,
                "ZZMI no contiene d3dx.ini.",
            )
        })?;
    let source_config = zzmi_root.read_file("d3dx.ini", 2 * 1024 * 1024)?;
    if sha256(&source_config) != config_entry.sha256 {
        return Err(XxmiError::new(
            ErrorCode::ChecksumMismatch,
            Some(zzmi.payload_path.join("d3dx.ini")),
            "d3dx.ini cambió después de validar el paquete.",
        ));
    }
    let (derived_config, loader_identity) = derive_d3dx_config(&source_config, &target_process)?;
    if loader_identity != ZZMI_LOADER_IDENTITY {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            Some(zzmi.payload_path.join("d3dx.ini")),
            "La identidad loader del paquete no coincide con la configuración upstream fijada.",
        ));
    }

    let runtime_seed = serde_json::to_vec(&(
        &zzmi.manifest.id,
        &libraries.manifest.id,
        &target_process,
        sha256(&derived_config),
    ))
    .map_err(|e| XxmiError::new(ErrorCode::InvalidMetadata, None, e.to_string()))?;
    let runtime_id = format!("zzmi-{}", sha256(&runtime_seed));
    let app_root = store
        .root()
        .join(MANAGED_RUNTIME_RELATIVE)
        .join(&runtime_id);
    let importer_root = app_root.join(IMPORTER_FOLDER);

    let mut pending = Vec::new();
    let mut targets = HashSet::new();
    for source in &zzmi.manifest.files {
        validate_relative_path(&source.relative_path)?;
        if !targets.insert(source.relative_path.to_lowercase()) {
            return Err(XxmiError::new(
                ErrorCode::UnsafePath,
                Some(source.relative_path.clone().into()),
                "Colisión de archivos ZZMI ignorando mayúsculas.",
            ));
        }
        let bytes = if source.relative_path == "d3dx.ini" {
            Some(derived_config.clone())
        } else {
            None
        };
        let (size, digest) = match &bytes {
            Some(value) => (value.len() as u64, sha256(value)),
            None => (source.size, source.sha256.clone()),
        };
        let file = RuntimeAssemblyFile {
            source_component: RuntimeComponent::Zzmi,
            source_package_id: zzmi.manifest.id.clone(),
            source_relative_path: source.relative_path.clone(),
            destination_relative_path: source.relative_path.clone(),
            size,
            sha256: digest,
            derived: bytes.is_some(),
            reason: if bytes.is_some() {
                "Copia derivada; target se resuelve al nombre del ejecutable observado en la instalación Steam de ZZZ."
            } else {
                "Payload ZZMI verificado; se conserva su ruta relativa original."
            }
            .into(),
        };
        pending.push(PlannedBytes { file, bytes });
    }
    for name in LIBRARY_DLLS_IN_IMPORTER {
        let source = libraries
            .manifest
            .files
            .iter()
            .find(|file| file.relative_path == *name)
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::MissingRequiredFile,
                    None,
                    format!("XXMI Libraries no contiene {name}."),
                )
            })?;
        if !targets.insert(name.to_lowercase()) {
            return Err(XxmiError::new(
                ErrorCode::InvalidPackage,
                Some((*name).into()),
                "Colisión de archivos entre ZZMI y XXMI Libraries.",
            ));
        }
        pending.push(PlannedBytes {
            file: RuntimeAssemblyFile {
                source_component: RuntimeComponent::XxmiLibraries,
                source_package_id: libraries.manifest.id.clone(),
                source_relative_path: source.relative_path.clone(),
                destination_relative_path: source.relative_path.clone(),
                size: source.size,
                sha256: source.sha256.clone(),
                derived: false,
                reason: "XXMI Launcher despliega esta DLL en importer_path; se ensambla únicamente en el root privado de LXMI."
                    .into(),
            },
            bytes: None,
        });
    }
    pending.sort_by(|a, b| {
        a.file
            .destination_relative_path
            .cmp(&b.file.destination_relative_path)
    });
    let files = pending
        .iter()
        .map(|item| item.file.clone())
        .collect::<Vec<_>>();
    let config = RuntimeConfigPlan {
        source_relative_path: "d3dx.ini".into(),
        source_sha256: config_entry.sha256.clone(),
        derived_sha256: sha256(&derived_config),
        target_process: target_process.clone(),
        target_evidence: "Steam manifest → game scanner → archivo ejecutable regular observado; filename coincide con el registry ZZZ upstream."
            .into(),
        loader_process_identity: loader_identity,
        loader_identity_evidence: "ZZMI upstream fija [Loader] loader = XXMI Launcher.exe; 3Dmigoto lo usa como nombre de proceso permitido para cargar su DLL."
            .into(),
        loader_identity_change: None,
    };
    let plan = RuntimeAssemblyPlan {
        runtime_id,
        game_id: TARGET_GAME_ID.into(),
        integration: IntegrationKind::Zzmi,
        app_root,
        importer_root,
        import_path_from_app_root: "ZZMI/".into(),
        zzmi_package_id: zzmi.manifest.id.clone(),
        libraries_package_id: libraries.manifest.id.clone(),
        zzmi_version: zzmi.manifest.version.as_ref().map(|v| v.raw.clone()),
        libraries_version: libraries.manifest.version.as_ref().map(|v| v.raw.clone()),
        target_process,
        loader_library_source: libraries.payload_path.join("3dmloader.dll"),
        loader_library_sha256: loader_hash,
        config,
        files,
        directories_to_create: vec!["Mods".into()],
        evidence: vec![
            "Upstream ZZMI importer_folder is relative to App.Root and equals ZZMI/.".into(),
            "The importer root is placed under LXMI XDG managed storage, outside the game and prefix.".into(),
            "XXMI Libraries deploys d3d11.dll and d3dcompiler_47.dll under importer_path; 3dmloader.dll remains in the separate Libraries package for the injector API.".into(),
            "The runtime package sources remain immutable; only derived copies are assembled.".into(),
        ],
        warnings: vec![
            "This assembly is not a launchable Windows helper or a tested Proton integration.".into(),
            "Steam/ZZZ/Proton launch and DLL loading remain unverified; no game files are changed.".into(),
        ],
    };
    tracing::info!(runtime_id = %plan.runtime_id, files = plan.files.len(), "Managed runtime assembly plan generated");
    Ok((plan, pending))
}

fn source_bytes_for_pending(
    zzmi: &VerifiedPackage,
    libraries: &VerifiedPackage,
    pending: &PlannedBytes,
) -> Result<Vec<u8>> {
    let source_package = match pending.file.source_component {
        RuntimeComponent::Zzmi => zzmi,
        RuntimeComponent::XxmiLibraries => libraries,
    };
    let directory = Directory::open_absolute(&source_package.payload_path)?;
    let bytes = directory.read_file(
        &pending.file.source_relative_path,
        ImportLimits::default().max_file_bytes,
    )?;
    if pending.file.source_relative_path == "d3dx.ini" {
        return Err(XxmiError::new(
            ErrorCode::InvalidMetadata,
            Some("d3dx.ini".into()),
            "La configuración derivada debe provenir del plan, no de una copia directa.",
        ));
    }
    Ok(bytes)
}

fn require_official_package(
    package: &VerifiedPackage,
    kind: OfficialPackageKind,
    require_component_signatures: bool,
) -> Result<()> {
    let provenance = package.manifest.upstream.as_ref().ok_or_else(|| {
        XxmiError::new(
            ErrorCode::MissingSignature,
            None,
            "El paquete administrado no conserva provenance upstream.",
        )
    })?;
    let expected_repository = crate::official_repository(&kind);
    if package.manifest.authenticity != PackageAuthenticity::OfficialReleaseVerified
        || provenance.repository != expected_repository
        || provenance.signature_status != SignatureStatus::Verified
        || provenance.signature_base64.is_none()
        || (require_component_signatures && !provenance.component_signatures_verified)
    {
        return Err(XxmiError::new(
            ErrorCode::InvalidSignature,
            None,
            format!("Se requiere un paquete autenticado desde {expected_repository} con las firmas necesarias verificadas."),
        ));
    }
    Ok(())
}

fn read_runtime(plan: &RuntimeAssemblyPlan) -> Result<Option<ManagedRuntime>> {
    let app_root = match Directory::open_absolute(&plan.app_root) {
        Ok(value) => value,
        Err(error) if missing(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    app_root.assert_private()?;
    let manifest: RuntimeManifest =
        serde_json::from_slice(&app_root.read_file("runtime-manifest.json", 2 * 1024 * 1024)?)
            .map_err(|e| {
                XxmiError::new(
                    ErrorCode::InvalidMetadata,
                    Some(plan.app_root.clone()),
                    e.to_string(),
                )
            })?;
    if manifest.schema_version != 1
        || manifest.runtime_id != plan.runtime_id
        || manifest.game_id != TARGET_GAME_ID
        || manifest.integration != IntegrationKind::Zzmi
        || manifest.zzmi_package_id != plan.zzmi_package_id
        || manifest.libraries_package_id != plan.libraries_package_id
        || manifest.target_process != plan.target_process
        || manifest.files != plan.files
        || manifest.configuration != plan.config
    {
        return Err(XxmiError::new(
            ErrorCode::ChecksumMismatch,
            Some(plan.app_root.clone()),
            "runtime-manifest.json no corresponde al plan actual.",
        ));
    }
    let importer = app_root.child(IMPORTER_FOLDER)?;
    let mods_meta = importer.metadata("Mods")?.ok_or_else(|| {
        XxmiError::new(
            ErrorCode::MissingRequiredFile,
            Some(plan.importer_root.join("Mods")),
            "El runtime no contiene Mods/.",
        )
    })?;
    if !mods_meta.is_dir() {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            Some(plan.importer_root.join("Mods")),
            "Mods debe ser un directorio.",
        ));
    }
    for file in &manifest.files {
        validate_relative_path(&file.destination_relative_path)?;
        let (size, digest) = hash_file(
            &importer,
            &file.destination_relative_path,
            ImportLimits::default().max_file_bytes,
        )?;
        if size != file.size || digest != file.sha256 {
            return Err(XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(plan.importer_root.join(&file.destination_relative_path)),
                "El archivo ensamblado no coincide con runtime-manifest.json.",
            ));
        }
    }
    Ok(Some(ManagedRuntime {
        runtime_id: manifest.runtime_id.clone(),
        app_root: plan.app_root.clone(),
        importer_root: plan.importer_root.clone(),
        manifest_path: plan.app_root.join("runtime-manifest.json"),
        loader_library_path: plan.loader_library_source.clone(),
        zzmi_package_id: manifest.zzmi_package_id.clone(),
        libraries_package_id: manifest.libraries_package_id.clone(),
        zzmi_version: manifest.zzmi_version.clone(),
        libraries_version: manifest.libraries_version.clone(),
        target_process: manifest.target_process.clone(),
        loader_process_identity: manifest.loader_process_identity.clone(),
        file_count: manifest.files.len(),
        manifest,
        newly_created: false,
        installed_into_game: false,
    }))
}

fn derive_d3dx_config(input: &[u8], target_process: &str) -> Result<(Vec<u8>, String)> {
    let text = std::str::from_utf8(input).map_err(|_| {
        XxmiError::new(
            ErrorCode::InvalidMetadata,
            Some("d3dx.ini".into()),
            "d3dx.ini no es UTF-8.",
        )
    })?;
    if target_process.is_empty()
        || target_process.contains(['/', '\\', ':', '\0'])
        || target_process.chars().any(char::is_control)
    {
        return Err(XxmiError::new(
            ErrorCode::UnsafePath,
            Some(target_process.into()),
            "Target process debe ser solo un filename.",
        ));
    }
    let mut in_loader = false;
    let mut loader_sections = 0;
    let mut targets = 0;
    let mut loaders = 0;
    let mut loader_identity = None;
    let mut output = String::with_capacity(text.len() + target_process.len());
    for raw in text.split_inclusive('\n') {
        let ending = if raw.ends_with("\r\n") {
            "\r\n"
        } else if raw.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let line = raw
            .strip_suffix(ending)
            .unwrap_or(raw)
            .strip_suffix('\r')
            .unwrap_or(raw.strip_suffix(ending).unwrap_or(raw));
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_loader = trimmed[1..trimmed.len() - 1]
                .trim()
                .eq_ignore_ascii_case("Loader");
            if in_loader {
                loader_sections += 1;
            }
            output.push_str(line);
            output.push_str(ending);
            continue;
        }
        if in_loader && !trimmed.starts_with([';', '#']) {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim().to_ascii_lowercase().as_str() {
                    "target" => {
                        targets += 1;
                        if value.trim().contains([';', '#']) {
                            return Err(XxmiError::new(
                                ErrorCode::InvalidMetadata,
                                Some("d3dx.ini".into()),
                                "No se puede derivar target con comentario inline ambiguo.",
                            ));
                        }
                        output.push_str(key.trim_end());
                        output.push_str("= ");
                        output.push_str(target_process);
                        output.push_str(ending);
                        continue;
                    }
                    "loader" => {
                        loaders += 1;
                        loader_identity = Some(value.trim().to_owned());
                    }
                    _ => {}
                }
            }
        }
        output.push_str(line);
        output.push_str(ending);
    }
    if loader_sections != 1 || targets != 1 || loaders != 1 {
        return Err(XxmiError::new(
            ErrorCode::InvalidMetadata,
            Some("d3dx.ini".into()),
            "Se requiere exactamente una sección [Loader] con un target y una identidad loader.",
        ));
    }
    let identity = loader_identity
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::InvalidMetadata,
                Some("d3dx.ini".into()),
                "La identidad loader está vacía.",
            )
        })?;
    Ok((output.into_bytes(), identity))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PackageAuthenticity, UpstreamProvenance};
    use std::{fs, os::unix::fs::PermissionsExt, path::Path, sync::atomic::AtomicU64};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Tree(PathBuf);
    impl Tree {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "lxmi-runtime-assembly-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn store(&self) -> ManagedStore {
            ManagedStore::at(self.0.join("lxmi"), ImportLimits::default()).unwrap()
        }
        fn package(&self, libraries: bool) -> VerifiedPackage {
            let path = self.0.join(if libraries { "libs" } else { "zzmi" });
            fs::create_dir(&path).unwrap();
            if libraries {
                for file in ["3dmloader.dll", "d3d11.dll", "d3dcompiler_47.dll"] {
                    fs::write(path.join(file), format!("fixture {file}")).unwrap();
                }
                fs::write(
                    path.join("Manifest.json"),
                    r#"{"version":"fixture","signatures":{"3dmloader.dll":"fixture","d3d11.dll":"fixture","d3dcompiler_47.dll":"fixture"}}"#,
                )
                .unwrap();
            } else {
                for file in ZZMI_REQUIRED_FILES {
                    let target = path.join(file);
                    fs::create_dir_all(target.parent().unwrap()).unwrap();
                    fs::write(target, b"synthetic fixture content").unwrap();
                }
                fs::write(path.join("d3dx.ini"), "[Loader]\ntarget = ZenlessZoneZeroBeta.exe\nloader = XXMI Launcher.exe\n[Include]\ninclude = Core\\ZZMI\\main.ini\n").unwrap();
                fs::write(
                    path.join("Core/ZZMI/main.ini"),
                    "global $version = fixture\n",
                )
                .unwrap();
            }
            let store = self.store();
            let mut verified = store.import_directory(&path).unwrap();
            verified.manifest.authenticity = PackageAuthenticity::OfficialReleaseVerified;
            verified.manifest.upstream = Some(UpstreamProvenance {
                repository: if libraries {
                    "SpectrumQT/XXMI-Libs-Package"
                } else {
                    "leotorrez/ZZMI-Package"
                }
                .into(),
                release_id: 1,
                tag: if libraries { "v1.1.7" } else { "v1.5.0" }.into(),
                commit: "fixture-commit".into(),
                release_url: "https://example.invalid/release".into(),
                asset_name: "fixture.zip".into(),
                asset_url: "https://example.invalid/asset".into(),
                published_at: "fixture".into(),
                metadata_retrieved_at: "fixture".into(),
                downloaded_at: "fixture".into(),
                download_sha256: "fixture".into(),
                expected_asset_sha256: None,
                signature_base64: Some("fixture".into()),
                signature_status: SignatureStatus::Verified,
                companion_asset_name: None,
                companion_asset_sha256: None,
                component_signatures_verified: libraries,
            });
            let manifest_path = store
                .root()
                .join("packages/xxmi")
                .join(&verified.manifest.id)
                .join("lxmi-package.json");
            fs::write(
                manifest_path,
                serde_json::to_vec_pretty(&verified.manifest).unwrap(),
            )
            .unwrap();
            store.verify(&verified.manifest.id).unwrap()
        }
        fn platform(&self) -> GameRuntimePlan {
            let game = self.0.join("game");
            let exe_dir = game.join("games/ZenlessZoneZero Game");
            fs::create_dir_all(&exe_dir).unwrap();
            let exe = exe_dir.join("ZenlessZoneZero.exe");
            fs::write(&exe, b"synthetic game marker").unwrap();
            let prefix = self.0.join("compatdata/pfx");
            fs::create_dir_all(&prefix).unwrap();
            fs::set_permissions(&prefix, fs::Permissions::from_mode(0o700)).unwrap();
            let installation = lxmi_steam::SteamGameInstallation {
                installation: lxmi_core::GameInstallation {
                    game: lxmi_core::SUPPORTED_GAMES[1],
                    install_path: game.clone(),
                    status: GameInstallationStatus::Installed,
                    distribution: GameDistribution::Steam,
                    executable_path: Some(exe),
                    executable_status: GameExecutableStatus::Found,
                },
                steam_app_id: lxmi_core::ZENLESS_ZONE_ZERO_APP_ID,
                manifest_name: "Zenless Zone Zero fixture".into(),
                steam_library: self.0.join("Steam"),
                compatdata: lxmi_core::ProtonCompatData {
                    app_id: lxmi_core::ZENLESS_ZONE_ZERO_APP_ID,
                    compatdata_path: self.0.join("compatdata"),
                    prefix_path: Some(prefix.clone()),
                    status: lxmi_core::ProtonCompatDataStatus::PrefixFound,
                },
            };
            lxmi_runtime::plan_game(
                lxmi_core::SUPPORTED_GAMES[1],
                lxmi_steam::SteamGameDiscoveryStatus::Complete,
                Some(&installation),
                &lxmi_proton::CompatibilityToolDiscoveryResult {
                    status: lxmi_proton::CompatibilityToolDiscoveryStatus::Complete,
                    tools: vec![],
                    issues: vec![],
                },
            )
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn config_derivation_changes_only_target_and_preserves_loader_identity() {
        let source = b"[Loader]\ntarget = old.exe\nloader = XXMI Launcher.exe\n";
        let (derived, loader) = derive_d3dx_config(source, "ZenlessZoneZero.exe").unwrap();
        assert_eq!(loader, ZZMI_LOADER_IDENTITY);
        assert_eq!(
            String::from_utf8(derived).unwrap(),
            "[Loader]\ntarget= ZenlessZoneZero.exe\nloader = XXMI Launcher.exe\n"
        );
        assert_eq!(
            source,
            b"[Loader]\ntarget = old.exe\nloader = XXMI Launcher.exe\n"
        );
    }

    #[test]
    fn assembly_plan_uses_managed_importer_root_and_keeps_loader_in_libraries_package() {
        let tree = Tree::new();
        let store = tree.store();
        let zzmi = tree.package(false);
        let libraries = tree.package(true);
        let game = tree.platform();
        let zzmi_before = fs::read(zzmi.payload_path.join("d3dx.ini")).unwrap();
        let plan = plan_zzmi_assembly(&store, &game, &zzmi, &libraries, &[]).unwrap();
        assert_eq!(plan.import_path_from_app_root, "ZZMI/");
        assert!(plan.importer_root.starts_with(store.root()));
        assert!(!store.root().join("runtimes").exists());
        assert!(!plan.importer_root.starts_with(match &game.installation {
            GameInstallationAssessment::Detected { install_path, .. } => install_path,
            _ => unreachable!(),
        }));
        assert!(!plan.importer_root.starts_with(match &game.prefix {
            PrefixAssessment::CandidateFound { path } => path,
            _ => unreachable!(),
        }));
        assert_eq!(plan.target_process, "ZenlessZoneZero.exe");
        assert_eq!(
            plan.loader_library_source,
            libraries.payload_path.join("3dmloader.dll")
        );
        assert!(plan
            .files
            .iter()
            .any(|f| f.destination_relative_path == "d3d11.dll"));
        assert!(!plan
            .files
            .iter()
            .any(|f| f.destination_relative_path == "3dmloader.dll"));
        assert_eq!(
            fs::read(zzmi.payload_path.join("d3dx.ini")).unwrap(),
            zzmi_before
        );
    }

    #[test]
    fn assembly_materializes_only_under_xdg_and_verifies_hash_inventory() {
        let tree = Tree::new();
        let store = tree.store();
        let zzmi = tree.package(false);
        let libraries = tree.package(true);
        let game = tree.platform();
        let game_root = match &game.installation {
            GameInstallationAssessment::Detected { install_path, .. } => install_path.clone(),
            _ => unreachable!(),
        };
        let game_before = directory_snapshot(&game_root);
        let prefix = match &game.prefix {
            PrefixAssessment::CandidateFound { path } => path.clone(),
            _ => unreachable!(),
        };
        let prefix_before = directory_snapshot(&prefix);
        let assembled = assemble_zzmi_runtime(
            &store,
            &game,
            &zzmi.manifest.id,
            &libraries.manifest.id,
            std::slice::from_ref(&prefix),
        )
        .unwrap();
        assert!(assembled.newly_created);
        assert!(!assembled.installed_into_game);
        assert!(assembled.importer_root.starts_with(store.root()));
        assert_eq!(directory_snapshot(&game_root), game_before);
        assert_eq!(directory_snapshot(&prefix), prefix_before);
        assert!(assembled.importer_root.join("Mods").is_dir());
        assert!(assembled.importer_root.join("d3d11.dll").is_file());
        let derived = fs::read_to_string(assembled.importer_root.join("d3dx.ini")).unwrap();
        assert!(derived.contains("target= ZenlessZoneZero.exe"));
        assert_eq!(fs::read(zzmi.payload_path.join("d3dx.ini")).unwrap(), b"[Loader]\ntarget = ZenlessZoneZeroBeta.exe\nloader = XXMI Launcher.exe\n[Include]\ninclude = Core\\ZZMI\\main.ini\n");
        let topology = crate::inspect_launch_topology(&game, Some(&assembled));
        assert_eq!(topology.selected_proton, "unknown");
        assert_eq!(
            topology.same_prefix_requirement,
            crate::SamePrefixRequirement::Unknown
        );
        assert_eq!(
            topology.readiness,
            crate::TopologyReadiness::PlannedIncomplete
        );
        assert!(!topology.execution_enabled);
        assert!(!topology.external_files_modified);
        let repeated = assemble_zzmi_runtime(
            &store,
            &game,
            &zzmi.manifest.id,
            &libraries.manifest.id,
            &[prefix],
        )
        .unwrap();
        assert!(!repeated.newly_created);
        assert_eq!(assembled.runtime_id, repeated.runtime_id);
    }

    #[test]
    fn assembly_rejects_local_packages_without_upstream_authenticity() {
        let tree = Tree::new();
        let store = tree.store();
        let mut zzmi = tree.package(false);
        let libraries = tree.package(true);
        zzmi.manifest.authenticity = PackageAuthenticity::NotAuthenticated;
        let error = build_plan(&store, &tree.platform(), &zzmi, &libraries, &[]).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidSignature);
    }

    fn directory_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn visit(base: &Path, path: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    out.push((path.strip_prefix(base).unwrap().into(), Vec::new()));
                    visit(base, &path, out);
                } else {
                    out.push((
                        path.strip_prefix(base).unwrap().into(),
                        fs::read(&path).unwrap(),
                    ));
                }
            }
            out.sort_by(|left, right| left.0.cmp(&right.0));
        }
        let mut result = Vec::new();
        visit(root, root, &mut result);
        result
    }
}
