//! Application orchestration; commands are thin async adapters.
use lxmi_core::SystemInfo;
use lxmi_proton::{CompatibilityToolKind, CompatibilityToolStatus, ProtonScanner};
use lxmi_runtime::{GameInstallationAssessment, RuntimePlanner};
use lxmi_steam::{SteamDiscoveryResult, SteamDiscoveryScanner};
use lxmi_xxmi::{
    assemble_zzmi_runtime, existing_zzmi_runtime, inspect_launch_topology, inspect_runtime,
    integration_for_game, plan_installation_for_integration, plan_zzmi_assembly, DownloadCache,
    ErrorCode, GitHubReleaseProvider, InstallationPlan, LaunchTopologyPlan, ManagedRuntime,
    ManagedStore, OfficialPackageKind, PackageAuthenticity, PackageKind, PackageManifest,
    ReleaseProvider, Result, RuntimeAssemblyPlan, RuntimeDiscovery, SignatureStatus,
    UpstreamRelease, XxmiError,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct BridgeRuntimeOption {
    pub display_name: String,
    pub version: Option<String>,
    pub source: String,
    pub proton_script: String,
}

#[derive(Serialize)]
pub struct RuntimeBridgePanel {
    pub options: lxmi_bridge::BridgeOptions,
    pub runtimes: Vec<BridgeRuntimeOption>,
    pub game_runtime_selection: &'static str,
}

#[derive(Serialize)]
pub struct LoaderLabComponent {
    pub package_id: String,
    pub version: Option<String>,
    pub release_tag: String,
    pub commit: String,
    pub loader_sha256: String,
    pub release_signature: String,
    pub component_signatures_verified: bool,
}

#[derive(Serialize)]
pub struct LoaderLabPanel {
    pub status: lxmi_bridge::LoaderLabStatus,
    pub component: Option<LoaderLabComponent>,
    pub component_issue: Option<String>,
    pub runtimes: Vec<BridgeRuntimeOption>,
}

#[derive(Debug, Serialize)]
pub struct BridgeServiceError {
    pub code: String,
    pub detail: String,
    pub stderr: Option<String>,
}

impl From<XxmiError> for BridgeServiceError {
    fn from(error: XxmiError) -> Self {
        Self {
            code: format!("{:?}", error.code).to_ascii_lowercase(),
            detail: error.detail,
            stderr: None,
        }
    }
}

impl From<lxmi_bridge::BridgeError> for BridgeServiceError {
    fn from(error: lxmi_bridge::BridgeError) -> Self {
        Self {
            code: format!("{:?}", error.code).to_ascii_lowercase(),
            detail: error.detail,
            stderr: error.stderr,
        }
    }
}

struct ResolvedBridgeRuntime {
    option: BridgeRuntimeOption,
    proton_script: PathBuf,
    steam_root: PathBuf,
}

fn resolved_bridge_runtimes(discovery: &SteamDiscoveryResult) -> Vec<ResolvedBridgeRuntime> {
    let tools = ProtonScanner.scan(&discovery.steam.installations);
    let mut result = Vec::new();
    for tool in tools.tools.iter().filter(|tool| {
        tool.kind == CompatibilityToolKind::Proton && tool.status == CompatibilityToolStatus::Valid
    }) {
        let tool_path = match tool.path.canonicalize() {
            Ok(path) => path,
            Err(_) => continue,
        };
        let proton_path = tool_path.join("proton");
        let metadata = match fs::symlink_metadata(&proton_path) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => metadata,
            _ => continue,
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }
        }
        #[cfg(not(unix))]
        let _ = metadata;
        let proton_script = match proton_path.canonicalize() {
            Ok(path) if path.starts_with(&tool_path) => path,
            _ => continue,
        };

        let mut roots = Vec::new();
        for installation in &discovery.steam.installations {
            let belongs_to_installation = installation
                .libraries
                .iter()
                .any(|library| tool_path.starts_with(library.path.join("steamapps/common")))
                || tool_path.starts_with(installation.root_path.join("compatibilitytools.d"));
            if belongs_to_installation {
                if let Ok(root) = installation.root_path.canonicalize() {
                    if !roots.contains(&root) {
                        roots.push(root);
                    }
                }
            }
        }
        if roots.len() != 1 {
            continue;
        }
        result.push(ResolvedBridgeRuntime {
            option: BridgeRuntimeOption {
                display_name: tool.display_name.clone(),
                version: tool.version.clone(),
                source: format!("{:?}", tool.source),
                proton_script: proton_script.display().to_string(),
            },
            proton_script,
            steam_root: roots.remove(0),
        });
    }
    result.sort_by(|left, right| {
        left.option
            .display_name
            .cmp(&right.option.display_name)
            .then_with(|| left.option.proton_script.cmp(&right.option.proton_script))
    });
    result
}

pub fn inspect_runtime_bridge(
    zzmi_id: Option<&str>,
    libraries_id: Option<&str>,
) -> std::result::Result<RuntimeBridgePanel, BridgeServiceError> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let managed_runtime = match (zzmi_id, libraries_id) {
        (Some(zzmi_id), Some(libraries_id)) if !zzmi_id.is_empty() && !libraries_id.is_empty() => {
            let platform =
                platform_plan(&discovery, "zenless-zone-zero").map_err(BridgeServiceError::from)?;
            existing_zzmi_runtime(
                &store,
                &platform,
                zzmi_id,
                libraries_id,
                &protected_paths(&discovery),
            )
            .map_err(BridgeServiceError::from)?
        }
        _ => None,
    };
    let options = lxmi_bridge::inspect_bridge_options(
        store.root(),
        managed_runtime
            .as_ref()
            .map(|runtime| runtime.app_root.as_path()),
    );
    Ok(RuntimeBridgePanel {
        options,
        runtimes: resolved_bridge_runtimes(&discovery)
            .into_iter()
            .map(|runtime| runtime.option)
            .collect(),
        game_runtime_selection: "unknown",
    })
}

pub fn run_runtime_bridge_test(
    zzmi_id: &str,
    libraries_id: &str,
    proton_script: &str,
) -> std::result::Result<lxmi_bridge::BridgeTestResult, BridgeServiceError> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let selected_path =
        Path::new(proton_script)
            .canonicalize()
            .map_err(|error| BridgeServiceError {
                code: "runtime_not_found".into(),
                detail: format!("El runtime elegido dejó de estar disponible: {error}"),
                stderr: None,
            })?;
    let selected_runtime = resolved_bridge_runtimes(&discovery)
        .into_iter()
        .find(|candidate| candidate.proton_script == selected_path)
        .ok_or_else(|| BridgeServiceError {
            code: "runtime_not_allowed".into(),
            detail: "El runtime no es un candidato Proton válido del discovery actual.".into(),
            stderr: None,
        })?;
    let platform =
        platform_plan(&discovery, "zenless-zone-zero").map_err(BridgeServiceError::from)?;
    let runtime = existing_zzmi_runtime(
        &store,
        &platform,
        zzmi_id,
        libraries_id,
        &protected_paths(&discovery),
    )
    .map_err(BridgeServiceError::from)?
    .ok_or_else(|| BridgeServiceError {
        code: "managed_runtime_missing".into(),
        detail: "Ensambla primero un runtime ZZMI administrado y verificado.".into(),
        stderr: None,
    })?;
    let options = lxmi_bridge::inspect_bridge_options(store.root(), Some(&runtime.app_root));
    match options.helper {
        lxmi_bridge::HelperState::Available {
            integrity_matches: true,
            ..
        } => {}
        lxmi_bridge::HelperState::Available { .. } => {
            return Err(BridgeServiceError {
                code: "helper_hash_mismatch".into(),
                detail: "La integridad del helper no coincide con su manifest LXMI.".into(),
                stderr: None,
            });
        }
        lxmi_bridge::HelperState::Missing => {
            return Err(BridgeServiceError {
                code: "helper_missing".into(),
                detail: "Construye y coloca el helper Windows LXMI en el almacenamiento administrado antes de iniciar la prueba.".into(),
                stderr: None,
            });
        }
        lxmi_bridge::HelperState::Invalid { detail } => {
            return Err(BridgeServiceError {
                code: "helper_invalid".into(),
                detail,
                stderr: None,
            });
        }
    }
    lxmi_bridge::run_bridge_test(&lxmi_bridge::BridgeTestConfig {
        managed_root: store.root().to_owned(),
        runtime_root: runtime.app_root,
        explicit_runtime: lxmi_bridge::ExplicitBridgeRuntime {
            display_name: selected_runtime.option.display_name,
            version: selected_runtime.option.version,
            proton_script: selected_runtime.proton_script,
        },
        steam_client_install_path: selected_runtime.steam_root,
        prefix_mode: lxmi_bridge::PrefixMode::IsolatedTemporaryPrefix,
    })
    .map_err(BridgeServiceError::from)
}

pub fn inspect_loader_lab() -> std::result::Result<LoaderLabPanel, BridgeServiceError> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let component = verified_loader_package(&store, None)?;
    let (component, component_issue) = match component {
        Some((_, verified, _, _)) => (
            Some(LoaderLabComponent {
                package_id: verified.package_id,
                version: verified.version,
                release_tag: verified.tag,
                commit: verified.commit,
                loader_sha256: verified.loader_sha256,
                release_signature: verified.release_signature,
                component_signatures_verified: verified.component_signatures_verified,
            }),
            None,
        ),
        None => (None, Some("No hay un paquete oficial autenticado de XXMI Libraries con 3dmloader.dll disponible.".into())),
    };
    let status = inspect_loader_lab_read_only(store.root())?;
    Ok(LoaderLabPanel {
        status,
        component,
        component_issue,
        runtimes: resolved_bridge_runtimes(&discovery)
            .into_iter()
            .map(|runtime| runtime.option)
            .collect(),
    })
}

pub fn prepare_loader_lab() -> std::result::Result<lxmi_bridge::LoaderLabStatus, BridgeServiceError>
{
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let (package, component, loader_path, loader_sha256) = verified_loader_package(&store, None)?
        .ok_or_else(|| BridgeServiceError {
            code: "upstream_loader_unavailable".into(),
            detail: "Importa primero XXMI Libraries desde la release oficial verificada; LXMI no descarga ni acepta otro origen para este experimento.".into(),
            stderr: None,
        })?;
    let application_root = application_workspace_root().map_err(|error| BridgeServiceError {
        code: "loader_artifacts_missing".into(),
        detail: format!("No se pudo resolver la raíz local de construcción: {error}"),
        stderr: None,
    })?;
    let upstream = package
        .manifest()
        .upstream
        .as_ref()
        .ok_or_else(|| BridgeServiceError {
            code: "loader_provenance_invalid".into(),
            detail: "Falta upstream provenance en el paquete XXMI Libraries.".into(),
            stderr: None,
        })?;
    let provenance = component.into_provenance(upstream);
    lxmi_bridge::stage_loader_lab(
        store.root(),
        &lxmi_bridge::LoaderLabSources {
            application_root,
            upstream_loader_path: loader_path,
            expected_upstream_loader_sha256: loader_sha256,
            provenance,
            protected_roots: protected_paths(&discovery),
        },
    )
    .map_err(BridgeServiceError::from)
}

pub fn run_loader_lab_experiment(
    proton_script: &str,
    mode: lxmi_bridge::LoaderExperimentMode,
    side_effects_acknowledged: bool,
) -> std::result::Result<lxmi_bridge::LoaderExperimentResult, BridgeServiceError> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let lab_status = inspect_loader_lab_read_only(store.root())?;
    let lab_manifest = lab_status.manifest.ok_or_else(|| BridgeServiceError {
        code: "loader_artifacts_missing".into(),
        detail: "Prepara Loader Lab antes de ejecutar un experimento.".into(),
        stderr: None,
    })?;
    if !lab_status.ready {
        return Err(BridgeServiceError {
            code: "loader_artifacts_invalid".into(),
            detail: lab_status.missing_or_invalid.join("; "),
            stderr: None,
        });
    }
    let (package, component, _, loader_sha256) = verified_loader_package(
        &store,
        Some(&lab_manifest.upstream.package_id),
    )?
    .ok_or_else(|| BridgeServiceError {
        code: "loader_provenance_invalid".into(),
        detail:
            "El paquete XXMI Libraries usado al preparar el laboratorio ya no está autenticado."
                .into(),
        stderr: None,
    })?;
    let upstream = package
        .manifest()
        .upstream
        .as_ref()
        .ok_or_else(|| BridgeServiceError {
            code: "loader_provenance_invalid".into(),
            detail: "Falta upstream provenance en el paquete XXMI Libraries.".into(),
            stderr: None,
        })?;
    let current_provenance = component.into_provenance(upstream);
    if current_provenance != lab_manifest.upstream
        || !loader_sha256.eq_ignore_ascii_case(&lab_manifest.upstream_loader_sha256)
    {
        return Err(BridgeServiceError {
            code: "loader_provenance_invalid".into(),
            detail: "El origen o el hash de 3dmloader.dll no coincide con el paquete autenticado usado en el staging.".into(),
            stderr: None,
        });
    }
    let selected_path =
        Path::new(proton_script)
            .canonicalize()
            .map_err(|error| BridgeServiceError {
                code: "runtime_not_found".into(),
                detail: format!("El Proton de prueba seleccionado no está disponible: {error}"),
                stderr: None,
            })?;
    let selected_runtime = resolved_bridge_runtimes(&discovery)
        .into_iter()
        .find(|candidate| candidate.proton_script == selected_path)
        .ok_or_else(|| BridgeServiceError {
            code: "runtime_not_allowed".into(),
            detail: "El Proton debe ser un candidato válido del discovery actual; no se aceptan ejecutables arbitrarios.".into(),
            stderr: None,
        })?;
    lxmi_bridge::run_loader_experiment(&lxmi_bridge::LoaderExperimentConfig {
        managed_root: store.root().to_owned(),
        explicit_runtime: lxmi_bridge::ExplicitBridgeRuntime {
            display_name: selected_runtime.option.display_name,
            version: selected_runtime.option.version,
            proton_script: selected_runtime.proton_script,
        },
        steam_client_install_path: selected_runtime.steam_root,
        mode,
        side_effects_acknowledged,
        verified_loader_sha256: loader_sha256,
    })
    .map_err(BridgeServiceError::from)
}

#[derive(Clone)]
struct VerifiedLoaderComponent {
    package_id: String,
    version: Option<String>,
    tag: String,
    commit: String,
    loader_sha256: String,
    release_signature: String,
    component_signatures_verified: bool,
}

impl VerifiedLoaderComponent {
    fn into_provenance(
        self,
        provenance: &lxmi_xxmi::UpstreamProvenance,
    ) -> lxmi_bridge::LoaderLabProvenance {
        lxmi_bridge::LoaderLabProvenance {
            package_id: self.package_id,
            repository: provenance.repository.clone(),
            release_id: provenance.release_id,
            tag: self.tag,
            commit: self.commit,
            signature_verified: self.release_signature == "verified",
            component_signatures_verified: self.component_signatures_verified,
        }
    }
}

fn verified_loader_package(
    store: &ManagedStore,
    required_id: Option<&str>,
) -> std::result::Result<
    Option<(
        lxmi_xxmi::VerifiedPackage,
        VerifiedLoaderComponent,
        PathBuf,
        String,
    )>,
    BridgeServiceError,
> {
    let manifests = store.list().map_err(BridgeServiceError::from)?;
    let selected = manifests
        .into_iter()
        .filter(|manifest| required_id.is_none_or(|id| manifest.id == id))
        .filter(|manifest| {
            manifest.kind == PackageKind::XxmiLibraries
                && manifest.authenticity == PackageAuthenticity::OfficialReleaseVerified
        })
        .filter_map(|manifest| {
            let upstream = manifest.upstream.as_ref()?;
            (upstream.repository == "SpectrumQT/XXMI-Libs-Package"
                && upstream.signature_status == SignatureStatus::Verified
                && upstream.component_signatures_verified
                && manifest.files.iter().any(|file| {
                    file.relative_path == "3dmloader.dll" && valid_sha256_text(&file.sha256)
                }))
            .then_some((manifest.imported_unix_seconds, manifest.id))
        })
        .max_by(|left, right| left.cmp(right));
    let Some((_, id)) = selected else {
        return Ok(None);
    };
    let package = store.verify(&id).map_err(BridgeServiceError::from)?;
    let manifest = package.manifest();
    let upstream = manifest
        .upstream
        .as_ref()
        .ok_or_else(|| BridgeServiceError {
            code: "loader_provenance_invalid".into(),
            detail: "Falta upstream provenance en el paquete XXMI Libraries.".into(),
            stderr: None,
        })?;
    let file = manifest
        .files
        .iter()
        .find(|file| file.relative_path == "3dmloader.dll")
        .ok_or_else(|| BridgeServiceError {
            code: "loader_component_missing".into(),
            detail: "El inventario autenticado no contiene 3dmloader.dll.".into(),
            stderr: None,
        })?;
    let path = package.payload_path().join("3dmloader.dll");
    let component = VerifiedLoaderComponent {
        package_id: manifest.id.clone(),
        version: manifest.version.as_ref().map(|version| version.raw.clone()),
        tag: upstream.tag.clone(),
        commit: upstream.commit.clone(),
        loader_sha256: file.sha256.clone(),
        release_signature: format!("{:?}", upstream.signature_status).to_ascii_lowercase(),
        component_signatures_verified: upstream.component_signatures_verified,
    };
    let loader_sha256 = file.sha256.clone();
    Ok(Some((package, component, path, loader_sha256)))
}

fn valid_sha256_text(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn inspect_loader_lab_read_only(
    root: &Path,
) -> std::result::Result<lxmi_bridge::LoaderLabStatus, BridgeServiceError> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            lxmi_bridge::inspect_loader_lab(root).map_err(BridgeServiceError::from)
        }
        Ok(_) => Err(BridgeServiceError {
            code: "unsafe_path".into(),
            detail: "El storage LXMI debe ser un directorio real, no un symlink.".into(),
            stderr: None,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(lxmi_bridge::LoaderLabStatus {
                test_root: lxmi_bridge::loader_lab_root(root),
                ready: false,
                manifest: None,
                missing_or_invalid: vec!["No existe un staging preparado.".into()],
            })
        }
        Err(error) => Err(BridgeServiceError {
            code: "io".into(),
            detail: error.to_string(),
            stderr: None,
        }),
    }
}

fn application_workspace_root() -> std::io::Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
}

#[derive(Serialize)]
pub struct IntegrationStatus {
    pub storage_root: String,
    pub packages: Vec<PackageManifest>,
    pub observations: Vec<RuntimeDiscovery>,
    pub issues: Vec<XxmiError>,
}
fn protected_paths(discovery: &SteamDiscoveryResult) -> Vec<std::path::PathBuf> {
    let mut protected = Vec::new();
    for root in &discovery.steam.installations {
        protected.push(root.root_path.clone());
        protected.extend(root.libraries.iter().map(|l| l.path.clone()));
    }
    for game in &discovery.games.games {
        protected.push(game.installation.install_path.clone());
        protected.push(game.compatdata.compatdata_path.clone());
        protected.push(game.compatdata.compatdata_path.join("pfx"));
    }
    protected
}

fn store_for(discovery: &SteamDiscoveryResult) -> Result<ManagedStore> {
    let store = ManagedStore::from_system(&SystemInfo::current())?;
    // Refuse writes if Steam discovery failed with an error: cannot establish boundaries.
    if !matches!(
        discovery.steam.status,
        lxmi_core::SteamDetectionStatus::Detected | lxmi_core::SteamDetectionStatus::NotInstalled
    ) {
        return Err(XxmiError::new(ErrorCode::StorageUnavailable,None,"Resuelve el diagnóstico de Steam antes de importar paquetes: límites de almacenamiento no comprobados."));
    }
    store.ensure_outside(&protected_paths(discovery))?;
    Ok(store)
}
pub fn status() -> Result<IntegrationStatus> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery)?;
    let mut observations = Vec::new();
    for game in &discovery.games.games {
        match game.installation.game.id {
            "wuthering-waves" => {
                // Finite candidate paths only; do not recursively search external game content.
                for path in [
                    game.installation.install_path.clone(),
                    game.installation.install_path.join("WWMI"),
                    game.installation.install_path.join("Client/Binaries/Win64"),
                ] {
                    observations.push(inspect_runtime(&path));
                }
            }
            // ZZZ is deliberately not inspected for an in-game importer. LXMI's
            // assembled importer is managed below XDG data storage instead.
            "zenless-zone-zero" => {}
            _ => {}
        }
    }
    observations.push(inspect_runtime(&store.root().join("runtimes/wwmi")));
    let (packages, issues) = match store.list() {
        Ok(p) => (p, vec![]),
        Err(e) => (vec![], vec![e]),
    };
    Ok(IntegrationStatus {
        storage_root: store.root().display().to_string(),
        packages,
        observations,
        issues,
    })
}

fn platform_plan(
    discovery: &SteamDiscoveryResult,
    game_id: &str,
) -> Result<lxmi_runtime::GameRuntimePlan> {
    let tools = ProtonScanner.scan(&discovery.steam.installations);
    RuntimePlanner::plan_all(discovery, &tools)
        .into_iter()
        .find(|plan| plan.game.id == game_id)
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::UnsupportedIntegration,
                None,
                "El juego no aparece en el discovery actual de Steam.",
            )
        })
}

pub fn prepare_zzmi_runtime(zzmi_id: &str, libraries_id: &str) -> Result<ManagedRuntime> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery)?;
    let platform = platform_plan(&discovery, "zenless-zone-zero")?;
    assemble_zzmi_runtime(
        &store,
        &platform,
        zzmi_id,
        libraries_id,
        &protected_paths(&discovery),
    )
}

pub fn review_zzmi_assembly(zzmi_id: &str, libraries_id: &str) -> Result<RuntimeAssemblyPlan> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery)?;
    let platform = platform_plan(&discovery, "zenless-zone-zero")?;
    let zzmi = store.verify(zzmi_id)?;
    let libraries = store.verify(libraries_id)?;
    plan_zzmi_assembly(
        &store,
        &platform,
        &zzmi,
        &libraries,
        &protected_paths(&discovery),
    )
}

pub fn inspect_zzmi_launch_topology(
    zzmi_id: &str,
    libraries_id: &str,
) -> Result<LaunchTopologyPlan> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery)?;
    let platform = platform_plan(&discovery, "zenless-zone-zero")?;
    let runtime = existing_zzmi_runtime(
        &store,
        &platform,
        zzmi_id,
        libraries_id,
        &protected_paths(&discovery),
    )?;
    Ok(inspect_launch_topology(&platform, runtime.as_ref()))
}
pub fn import_directory(path: &Path) -> Result<PackageManifest> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    Ok(store_for(&discovery)?
        .import_directory(path)?
        .manifest()
        .clone())
}

pub fn check_official_releases() -> Result<Vec<UpstreamRelease>> {
    let provider = GitHubReleaseProvider::new()?;
    Ok(vec![
        provider.latest_release(OfficialPackageKind::Zzmi)?,
        provider.latest_release(OfficialPackageKind::XxmiLibraries)?,
    ])
}

pub fn download_official_package(kind: OfficialPackageKind, tag: &str) -> Result<PackageManifest> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery)?;
    let provider = GitHubReleaseProvider::new()?;
    let cache = DownloadCache::from_environment()?;
    lxmi_xxmi::download_and_import_official(&store, &cache, &provider, kind, tag)
}
pub fn plan(
    game_id: &str,
    package_id: &str,
    libraries_id: Option<&str>,
) -> Result<InstallationPlan> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let tools = ProtonScanner.scan(&discovery.steam.installations);
    let plans = RuntimePlanner::plan_all(&discovery, &tools);
    let platform = plans.iter().find(|p| p.game.id == game_id).ok_or_else(|| {
        XxmiError::new(
            ErrorCode::UnsupportedIntegration,
            None,
            "Juego no registrado.",
        )
    })?;
    if matches!(
        platform.installation,
        GameInstallationAssessment::Unknown { .. }
    ) {
        return Err(XxmiError::new(
            ErrorCode::InvalidPackage,
            None,
            "Instalación aún desconocida; revisa Steam.",
        ));
    }
    let integration = integration_for_game(game_id).ok_or_else(|| {
        XxmiError::new(
            ErrorCode::UnsupportedIntegration,
            None,
            "No existe una integración registrada para este juego.",
        )
    })?;
    plan_installation_for_integration(
        &store_for(&discovery)?,
        platform,
        integration,
        package_id,
        libraries_id,
    )
}

#[cfg(test)]
mod loader_lab_host_tests {
    use super::*;
    use std::time::SystemTime;

    #[derive(Debug, PartialEq, Eq)]
    struct PathSnapshot {
        path: PathBuf,
        exists: bool,
        is_file: bool,
        is_directory: bool,
        length: u64,
        modified: Option<SystemTime>,
    }

    fn snapshot_zzz_paths() -> Vec<PathSnapshot> {
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let mut paths = Vec::new();
        for game in &discovery.games.games {
            if game.installation.game.id != "zenless-zone-zero" {
                continue;
            }
            if let Some(executable) = &game.installation.executable_path {
                paths.push(executable.clone());
            }
            paths.push(game.compatdata.compatdata_path.clone());
            paths.push(game.compatdata.compatdata_path.join("pfx"));
        }
        paths.sort();
        paths.dedup();
        paths
            .into_iter()
            .map(|path| {
                let metadata = fs::symlink_metadata(&path).ok();
                PathSnapshot {
                    path,
                    exists: metadata.is_some(),
                    is_file: metadata.as_ref().is_some_and(|value| value.is_file()),
                    is_directory: metadata.as_ref().is_some_and(|value| value.is_dir()),
                    length: metadata.as_ref().map_or(0, fs::Metadata::len),
                    modified: metadata.and_then(|value| value.modified().ok()),
                }
            })
            .collect()
    }

    #[test]
    #[ignore = "host test: launches the selected Proton and writes only LXMI loader-v1 test storage/prefix"]
    fn upstream_direct_inject_loader_works_only_against_lxmi_test_target() {
        let before = snapshot_zzz_paths();
        let prepared =
            prepare_loader_lab().expect("official package and compiled LXMI tools should stage");
        assert!(
            prepared.ready,
            "staging should validate: {:?}",
            prepared.missing_or_invalid
        );
        let panel = inspect_loader_lab().expect("Loader Lab inspection should succeed");
        let runtime = panel
            .runtimes
            .iter()
            .find(|candidate| {
                candidate
                    .display_name
                    .to_ascii_lowercase()
                    .contains("experimental")
            })
            .or_else(|| panel.runtimes.first())
            .expect("an explicitly selected installed Proton candidate is required");
        eprintln!(
            "Explicit bridge-test runtime: {} {} ({})",
            runtime.display_name,
            runtime.version.as_deref().unwrap_or("unknown version"),
            runtime.proton_script
        );

        for (mode, expected) in [
            (
                lxmi_bridge::LoaderExperimentMode::Baseline,
                lxmi_bridge::LoaderExperimentOutcome::Passed,
            ),
            (
                lxmi_bridge::LoaderExperimentMode::Positive,
                lxmi_bridge::LoaderExperimentOutcome::Passed,
            ),
            (
                lxmi_bridge::LoaderExperimentMode::MissingTarget,
                lxmi_bridge::LoaderExperimentOutcome::ExpectedFailure,
            ),
            (
                lxmi_bridge::LoaderExperimentMode::MissingDll,
                lxmi_bridge::LoaderExperimentOutcome::ExpectedFailure,
            ),
            (
                lxmi_bridge::LoaderExperimentMode::WrongNonce,
                lxmi_bridge::LoaderExperimentOutcome::ExpectedFailure,
            ),
        ] {
            let result = run_loader_lab_experiment(&runtime.proton_script, mode, true)
                .unwrap_or_else(|error| panic!("{mode:?} failed structurally: {}", error.detail));
            assert_eq!(
                result.outcome, expected,
                "unexpected {mode:?} result: {result:?}"
            );
            if mode == lxmi_bridge::LoaderExperimentMode::Positive {
                assert!(result.target_started && result.target_ready);
                assert!(result.dll_loaded && result.marker_verified && result.nonce_verified);
                assert_eq!(result.loader_mode, lxmi_bridge::LoaderMode::DirectInject);
                assert!(result.target_path_windows.is_some());
                assert!(result.test_dll_path_windows.is_some());
            }
            if mode == lxmi_bridge::LoaderExperimentMode::WrongNonce {
                assert!(result.dll_loaded);
                assert!(!result.marker_verified && !result.nonce_verified);
            }
            eprintln!(
                "{mode:?}: {:?}, exit {}",
                result.outcome, result.process.exit_code
            );
        }

        let after = snapshot_zzz_paths();
        assert_eq!(
            before, after,
            "watched ZZZ executable/compatdata metadata changed during isolated test"
        );
        eprintln!("ZZZ metadata unchanged; test does not launch or enumerate game processes.");
    }
}
