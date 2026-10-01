//! Application orchestration; commands are thin async adapters.
use lxmi_core::{
    GameDistribution, GameExecutableStatus, GameInstallationStatus, SystemInfo,
    ZENLESS_ZONE_ZERO_APP_ID,
};
use lxmi_proton::{
    CompatibilityTool, CompatibilityToolKind, CompatibilityToolSource, CompatibilityToolStatus,
    ProtonScanner,
};
use lxmi_runtime::{
    CompatDataAssessment, GameInstallationAssessment, GameInstallationCompleteness,
    PrefixAssessment, RuntimeObservationSnapshot, RuntimePlanner, RuntimeSelection,
};
use lxmi_steam::{
    inspect_steam_compat_tool_config, SteamCompatToolConfigStatus, SteamCompatToolMapping,
    SteamDiscoveryResult, SteamDiscoveryScanner,
};
use lxmi_xxmi::{
    assemble_zzmi_runtime, existing_zzmi_runtime, inspect_launch_topology, inspect_runtime,
    integration_for_game, plan_installation_for_integration, plan_zzmi_assembly,
    ConfiguredToolMappingEvidence, ContextFindingState, ContextToolKind, ContextToolOrigin,
    DownloadCache, ErrorCode, GitHubReleaseProvider, HookRequirementState, InstallationPlan,
    InstalledCompatibilityToolEvidence, LaunchTopologyPlan, ManagedRuntime, ManagedStore,
    OfficialPackageKind, PackageAuthenticity, PackageKind, PackageManifest,
    PerGameToolMappingState, ReadinessCapabilityState, ReadinessEvidenceState, ReleaseProvider,
    Result, RuntimeAssemblyPlan, RuntimeContextKnowledge, RuntimeDiscovery, SignatureStatus,
    SteamCompatConfigEvidenceState, SteamProcessContextFinding, SteamProtonProcessContextEvidence,
    UpstreamLoaderStrategy, UpstreamRelease, XxmiError, ZzmiCapability, ZzmiCapabilityAssessment,
    ZzmiLaunchReadiness, ZzmiLaunchReadinessInput, ZzmiLoaderMode,
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
    // Hook mode is intentionally unavailable until its global scope can be
    // proven to stay within the LXMI-owned target. Reject before discovery,
    // managed-storage inspection, Proton resolution, or process work.
    if mode == lxmi_bridge::LoaderExperimentMode::Hook {
        return Err(BridgeServiceError {
            code: "hook_scope_not_isolated".into(),
            detail: "Hook de XXMI bloqueado: upstream configura un hook global y LXMI no puede demostrar que quede limitado al proceso de prueba propio. No se inspeccionó Proton ni se inició ningún proceso.".into(),
            stderr: None,
        });
    }

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

#[cfg(test)]
mod hook_guard_tests {
    use super::*;

    #[test]
    fn hook_is_rejected_before_proton_or_lab_resolution() {
        let error = run_loader_lab_experiment(
            "/lxmi-test/nonexistent-proton",
            lxmi_bridge::LoaderExperimentMode::Hook,
            false,
        )
        .expect_err("Hook must be blocked before resolving the supplied runtime");

        assert_eq!(error.code, "hook_scope_not_isolated");
        assert!(error.detail.contains("No se inspeccionó Proton"));
    }
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

struct ReadinessPackageEvidence {
    package_state: ReadinessEvidenceState,
    authenticity: ReadinessEvidenceState,
    version: Option<String>,
    verified: Option<lxmi_xxmi::VerifiedPackage>,
}

fn inspect_readiness_package(
    store: &ManagedStore,
    package_id: Option<&str>,
    expected_kind: PackageKind,
) -> ReadinessPackageEvidence {
    let Some(package_id) = package_id.filter(|value| !value.is_empty()) else {
        return ReadinessPackageEvidence {
            package_state: ReadinessEvidenceState::Missing,
            authenticity: ReadinessEvidenceState::Missing,
            version: None,
            verified: None,
        };
    };
    match store.verify(package_id) {
        Ok(package) if package.manifest().kind == expected_kind => {
            let authenticity = match package.manifest().authenticity {
                PackageAuthenticity::OfficialReleaseVerified => ReadinessEvidenceState::Verified,
                PackageAuthenticity::NotAuthenticated | PackageAuthenticity::MissingSignature => {
                    ReadinessEvidenceState::Incomplete
                }
                PackageAuthenticity::SignatureInvalid => ReadinessEvidenceState::Invalid,
            };
            ReadinessPackageEvidence {
                package_state: ReadinessEvidenceState::Verified,
                authenticity,
                version: package
                    .manifest()
                    .version
                    .as_ref()
                    .map(|version| version.raw.clone()),
                verified: Some(package),
            }
        }
        Ok(_) => ReadinessPackageEvidence {
            package_state: ReadinessEvidenceState::Invalid,
            authenticity: ReadinessEvidenceState::Invalid,
            version: None,
            verified: None,
        },
        Err(error) => {
            let state = match error.code {
                ErrorCode::NotFound => ReadinessEvidenceState::Missing,
                ErrorCode::PermissionDenied | ErrorCode::StorageUnavailable | ErrorCode::Io => {
                    ReadinessEvidenceState::Unreadable
                }
                _ => ReadinessEvidenceState::Invalid,
            };
            ReadinessPackageEvidence {
                package_state: state,
                authenticity: state,
                version: None,
                verified: None,
            }
        }
    }
}

fn evidence_from_compatdata(state: &CompatDataAssessment) -> ReadinessEvidenceState {
    match state {
        CompatDataAssessment::Found { .. } => ReadinessEvidenceState::Verified,
        CompatDataAssessment::NotFound { .. } => ReadinessEvidenceState::Missing,
        CompatDataAssessment::Invalid { .. } => ReadinessEvidenceState::Invalid,
        CompatDataAssessment::Unreadable { .. } => ReadinessEvidenceState::Unreadable,
        CompatDataAssessment::Unknown { .. } | CompatDataAssessment::NotRequired => {
            ReadinessEvidenceState::Unknown
        }
    }
}

fn compatdata_path(state: &CompatDataAssessment) -> Option<PathBuf> {
    match state {
        CompatDataAssessment::NotRequired => None,
        CompatDataAssessment::Unknown { .. } => None,
        CompatDataAssessment::NotFound { expected_path } => Some(expected_path.clone()),
        CompatDataAssessment::Found { path }
        | CompatDataAssessment::Invalid { path }
        | CompatDataAssessment::Unreadable { path, .. } => Some(path.clone()),
    }
}

fn evidence_from_prefix(state: &PrefixAssessment) -> ReadinessEvidenceState {
    match state {
        PrefixAssessment::CandidateFound { .. } => ReadinessEvidenceState::Verified,
        PrefixAssessment::NotInitialized { .. } => ReadinessEvidenceState::Missing,
        PrefixAssessment::Invalid { .. } => ReadinessEvidenceState::Invalid,
        PrefixAssessment::Unreadable { .. } => ReadinessEvidenceState::Unreadable,
        PrefixAssessment::Unknown { .. } | PrefixAssessment::NotRequired => {
            ReadinessEvidenceState::Unknown
        }
    }
}

fn prefix_path(state: &PrefixAssessment) -> Option<PathBuf> {
    match state {
        PrefixAssessment::NotRequired | PrefixAssessment::Unknown { .. } => None,
        PrefixAssessment::NotInitialized { expected_path } => Some(expected_path.clone()),
        PrefixAssessment::CandidateFound { path }
        | PrefixAssessment::Invalid { path }
        | PrefixAssessment::Unreadable { path, .. } => Some(path.clone()),
    }
}

fn compatibility_tool_evidence(tool: &CompatibilityTool) -> InstalledCompatibilityToolEvidence {
    InstalledCompatibilityToolEvidence {
        internal_id: tool.internal_id.clone(),
        display_name: tool.display_name.clone(),
        kind: match tool.kind {
            CompatibilityToolKind::Proton => ContextToolKind::Proton,
            CompatibilityToolKind::SteamLinuxRuntime => ContextToolKind::SteamLinuxRuntime,
            CompatibilityToolKind::Other => ContextToolKind::Other,
            CompatibilityToolKind::Unknown => ContextToolKind::Unknown,
        },
        origin: match tool.source {
            CompatibilityToolSource::SteamLibrary => ContextToolOrigin::SteamLibrary,
            CompatibilityToolSource::Custom => ContextToolOrigin::Custom,
        },
        version: tool.version.clone(),
        path: tool.path.clone(),
    }
}

fn configured_mapping_evidence(mapping: &SteamCompatToolMapping) -> ConfiguredToolMappingEvidence {
    ConfiguredToolMappingEvidence {
        app_id: mapping.app_id,
        internal_name: mapping.internal_name.clone(),
        config: mapping.config.clone(),
        priority: mapping.priority.clone(),
    }
}

fn config_evidence_state(status: SteamCompatToolConfigStatus) -> SteamCompatConfigEvidenceState {
    match status {
        SteamCompatToolConfigStatus::Read => SteamCompatConfigEvidenceState::Read,
        SteamCompatToolConfigStatus::Missing => SteamCompatConfigEvidenceState::Missing,
        SteamCompatToolConfigStatus::Invalid => SteamCompatConfigEvidenceState::Invalid,
        SteamCompatToolConfigStatus::Unreadable => SteamCompatConfigEvidenceState::Unreadable,
        SteamCompatToolConfigStatus::UnsafePath => SteamCompatConfigEvidenceState::UnsafePath,
        SteamCompatToolConfigStatus::TooLarge => SteamCompatConfigEvidenceState::TooLarge,
    }
}

fn context_finding(
    claim: &str,
    state: ContextFindingState,
    source: String,
    detail: &str,
) -> SteamProcessContextFinding {
    SteamProcessContextFinding {
        claim: claim.into(),
        state,
        source,
        detail: detail.into(),
    }
}

fn steam_process_context_evidence(
    discovery: &SteamDiscoveryResult,
    compatibility_tools: &[CompatibilityTool],
    platform: &lxmi_runtime::GameRuntimePlan,
    app_id: u32,
    observation: Option<&RuntimeObservationSnapshot>,
) -> SteamProtonProcessContextEvidence {
    let compatdata_state = evidence_from_compatdata(&platform.compatdata);
    let compatdata_path = compatdata_path(&platform.compatdata);
    let prefix_state = evidence_from_prefix(&platform.prefix);
    let prefix_path = prefix_path(&platform.prefix);
    let library_root = match &platform.installation {
        GameInstallationAssessment::Detected { steam_library, .. } => Some(steam_library.clone()),
        _ => None,
    };
    let matching_roots = discovery
        .steam
        .installations
        .iter()
        .filter(|installation| {
            library_root.as_ref().is_some_and(|library| {
                installation
                    .libraries
                    .iter()
                    .any(|candidate| candidate.path == *library)
            })
        })
        .collect::<Vec<_>>();
    let steam_root = (matching_roots.len() == 1).then(|| matching_roots[0].root_path.clone());
    let app_manifest_path = library_root.as_ref().map(|library| {
        library
            .join("steamapps")
            .join(format!("appmanifest_{app_id}.acf"))
    });
    let inspection = steam_root
        .as_deref()
        .map(|root| inspect_steam_compat_tool_config(root, app_id));
    let config_path = inspection.as_ref().map(|value| value.config_path.clone());
    let config_state = inspection
        .as_ref()
        .map(|value| config_evidence_state(value.status))
        .unwrap_or(SteamCompatConfigEvidenceState::NotInspected);
    let config_issue = inspection.as_ref().and_then(|value| value.detail.clone());
    let mappings = inspection
        .as_ref()
        .and_then(|value| value.mappings.as_ref());
    let per_game_mapping = mappings
        .and_then(|value| value.per_app.as_ref())
        .map(configured_mapping_evidence);
    let global_default_mapping = mappings
        .and_then(|value| value.global_default.as_ref())
        .map(configured_mapping_evidence);
    let per_game_mapping_state = if per_game_mapping.is_some() {
        PerGameToolMappingState::Configured
    } else if config_state == SteamCompatConfigEvidenceState::Read {
        PerGameToolMappingState::NoPerGameOverride
    } else {
        PerGameToolMappingState::Unknown
    };
    let installed_compatibility_tools = compatibility_tools
        .iter()
        .map(compatibility_tool_evidence)
        .collect::<Vec<_>>();
    let mapped_internal_name = per_game_mapping
        .as_ref()
        .and_then(|mapping| mapping.internal_name.as_deref())
        .filter(|name| !name.is_empty());
    let matched_installed_tools = mapped_internal_name
        .map(|internal_name| {
            compatibility_tools
                .iter()
                .filter(|tool| tool.internal_id.as_deref() == Some(internal_name))
                .map(compatibility_tool_evidence)
                .collect()
        })
        .unwrap_or_default();

    let mut findings = Vec::new();
    let manifest_state = match &platform.installation {
        GameInstallationAssessment::Detected { .. } => ContextFindingState::Detected,
        GameInstallationAssessment::NotFound { .. } => ContextFindingState::Absent,
        GameInstallationAssessment::Unknown { .. } => ContextFindingState::Unknown,
    };
    findings.push(context_finding(
        "Steam app manifest",
        manifest_state,
        app_manifest_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Steam game discovery".into()),
        "La existencia del manifest vincula AppID y biblioteca; no identifica por sí sola el runtime efectivo.",
    ));
    findings.push(context_finding(
        "compatdata directory",
        match compatdata_state {
            ReadinessEvidenceState::Verified => ContextFindingState::Detected,
            ReadinessEvidenceState::Missing => ContextFindingState::Absent,
            _ => ContextFindingState::Unknown,
        },
        compatdata_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Steam compatdata discovery".into()),
        "Filesystem discovery only; presence does not prove which compatibility tool executed the game.",
    ));
    findings.push(context_finding(
        "pfx directory",
        match prefix_state {
            ReadinessEvidenceState::Verified => ContextFindingState::Detected,
            ReadinessEvidenceState::Missing => ContextFindingState::Absent,
            _ => ContextFindingState::Unknown,
        },
        prefix_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Steam prefix discovery".into()),
        "This is a prefix candidate. No running Proton process was inspected.",
    ));
    findings.push(context_finding(
        "Per-game compatibility-tool mapping",
        match per_game_mapping_state {
            PerGameToolMappingState::Configured => ContextFindingState::Configured,
            PerGameToolMappingState::NoPerGameOverride => ContextFindingState::Absent,
            PerGameToolMappingState::Unknown => ContextFindingState::Unknown,
        },
        config_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Steam config/config.vdf".into()),
        match per_game_mapping_state {
            PerGameToolMappingState::Configured => "An AppID mapping was read; it is configuration evidence, not observation of an effective process runtime.",
            PerGameToolMappingState::NoPerGameOverride => "No AppID-specific mapping was present in the readable CompatToolMapping section. The global mapping is reported separately and is not promoted to a game selection.",
            PerGameToolMappingState::Unknown => "The config source could not be read or parsed, or no unique Steam root was associated with the game library.",
        },
    ));
    findings.push(context_finding(
        "Effective Steam/Proton execution context",
        ContextFindingState::Unknown,
        "LXMI no inicia Steam ni ZZZ; el contexto efectivo se obtiene únicamente mediante observación pasiva de una sesión iniciada manualmente.".into(),
        "Los mappings configurados y los candidatos instalados no demuestran por sí solos el runtime de una ejecución concreta.",
    ));
    findings.push(context_finding(
        "Loader prefix and wineserver relationship",
        ContextFindingState::Unknown,
        "No process-context or upstream requirement evidence established".into(),
        "Whether a future loader must share WINEPREFIX or wineserver with ZZZ remains unknown.",
    ));

    if matching_roots.len() > 1 {
        findings.push(context_finding(
            "Steam root association",
            ContextFindingState::Unknown,
            "Steam discovery".into(),
            "More than one discovered Steam root contains the game library; config.vdf was not selected arbitrarily.",
        ));
    }

    let mut context = SteamProtonProcessContextEvidence {
        app_id,
        steam_root,
        library_root,
        app_manifest_path,
        compatdata_state,
        compatdata_path,
        prefix_state,
        prefix_path,
        config_path,
        config_state,
        config_issue,
        per_game_mapping_state,
        per_game_mapping,
        global_default_mapping,
        matched_installed_tools,
        installed_compatibility_tools,
        effective_runtime_selection: RuntimeContextKnowledge::Unknown,
        effective_prefix: RuntimeContextKnowledge::Unknown,
        process_context: RuntimeContextKnowledge::Unknown,
        loader_same_prefix_requirement: RuntimeContextKnowledge::Unknown,
        loader_same_wineserver_requirement: RuntimeContextKnowledge::Unknown,
        wineserver_relationship: RuntimeContextKnowledge::Unknown,
        runtime_observation: None,
        findings,
    };
    if let Some(observation) = observation {
        lxmi_xxmi::incorporate_runtime_observation(&mut context, &observation.summary());
    }
    context
}

pub fn inspect_zzz_launch_readiness(
    zzmi_id: Option<&str>,
    libraries_id: Option<&str>,
    observation: Option<&RuntimeObservationSnapshot>,
) -> std::result::Result<ZzmiLaunchReadiness, BridgeServiceError> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = store_for(&discovery).map_err(BridgeServiceError::from)?;
    let tools = ProtonScanner.scan(&discovery.steam.installations);
    let platform = RuntimePlanner::plan_all(&discovery, &tools)
        .into_iter()
        .find(|plan| plan.game.id == "zenless-zone-zero")
        .ok_or_else(|| BridgeServiceError {
            code: "game_registry_missing".into(),
            detail: "Zenless Zone Zero no aparece en el registro de juegos de LXMI.".into(),
            stderr: None,
        })?;

    let zzmi = inspect_readiness_package(
        &store,
        zzmi_id,
        PackageKind::GameIntegration(lxmi_xxmi::IntegrationKind::Zzmi),
    );
    let libraries = inspect_readiness_package(&store, libraries_id, PackageKind::XxmiLibraries);

    let (
        installation,
        game_content_completeness,
        distribution,
        app_id,
        install_path,
        executable_path,
    ) = match &platform.installation {
        GameInstallationAssessment::Detected {
            steam_app_id,
            install_path,
            distribution,
            executable_path,
            executable_status,
            directory_status,
            completeness,
            ..
        } => {
            let correct_app = *steam_app_id == ZENLESS_ZONE_ZERO_APP_ID;
            let installation = match (correct_app, directory_status) {
                (false, _) => ReadinessEvidenceState::Invalid,
                (true, GameInstallationStatus::Installed) => ReadinessEvidenceState::Verified,
                (true, GameInstallationStatus::DirectoryMissing) => ReadinessEvidenceState::Missing,
                (true, GameInstallationStatus::DirectoryInvalid) => ReadinessEvidenceState::Invalid,
                (true, GameInstallationStatus::PermissionDenied) => {
                    ReadinessEvidenceState::Unreadable
                }
            };
            let completeness = match completeness {
                GameInstallationCompleteness::Unknown => ReadinessEvidenceState::Unknown,
            };
            let distribution_state = match distribution {
                GameDistribution::Steam => ReadinessCapabilityState::Verified,
                GameDistribution::Unknown => ReadinessCapabilityState::Unknown,
                GameDistribution::HoYoPlay | GameDistribution::Manual => {
                    ReadinessCapabilityState::Unsupported
                }
            };
            let executable_path = executable_path.clone().filter(|path| {
                *executable_status == GameExecutableStatus::Found
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| platform.game.expected_executable_names.contains(&name))
            });
            (
                installation,
                completeness,
                distribution_state,
                *steam_app_id,
                Some(install_path.clone()),
                executable_path,
            )
        }
        GameInstallationAssessment::NotFound { .. } => (
            ReadinessEvidenceState::Missing,
            ReadinessEvidenceState::Unknown,
            ReadinessCapabilityState::Unknown,
            ZENLESS_ZONE_ZERO_APP_ID,
            None,
            None,
        ),
        GameInstallationAssessment::Unknown { .. } => (
            ReadinessEvidenceState::Unknown,
            ReadinessEvidenceState::Unknown,
            ReadinessCapabilityState::Unknown,
            ZENLESS_ZONE_ZERO_APP_ID,
            None,
            None,
        ),
    };

    let executable_state = match &platform.installation {
        GameInstallationAssessment::Detected {
            executable_status, ..
        } => match executable_status {
            GameExecutableStatus::Found if executable_path.is_some() => {
                ReadinessEvidenceState::Verified
            }
            GameExecutableStatus::Found | GameExecutableStatus::NotFound => {
                ReadinessEvidenceState::Missing
            }
            GameExecutableStatus::SearchIncomplete | GameExecutableStatus::NotScanned => {
                ReadinessEvidenceState::Unknown
            }
        },
        GameInstallationAssessment::NotFound { .. } => ReadinessEvidenceState::Missing,
        GameInstallationAssessment::Unknown { .. } => ReadinessEvidenceState::Unknown,
    };

    let compatdata_state = evidence_from_compatdata(&platform.compatdata);
    let prefix_state = evidence_from_prefix(&platform.prefix);
    let (selected_game_runtime, game_runtime_selection) = match &platform.selection {
        RuntimeSelection::Selected { candidate } => (
            Some(format!(
                "{}{}",
                candidate.display_name,
                candidate
                    .version
                    .as_ref()
                    .map(|version| format!(" · {version}"))
                    .unwrap_or_default()
            )),
            ReadinessCapabilityState::Verified,
        ),
        RuntimeSelection::Unknown { .. } => (None, ReadinessCapabilityState::Unknown),
        RuntimeSelection::NotRequired => (None, ReadinessCapabilityState::Unsupported),
    };

    let mut managed_runtime = ReadinessEvidenceState::Missing;
    let mut managed_runtime_root = None;
    if let (Some(zzmi), Some(libraries)) = (&zzmi.verified, &libraries.verified) {
        if zzmi.manifest().authenticity == PackageAuthenticity::OfficialReleaseVerified
            && libraries.manifest().authenticity == PackageAuthenticity::OfficialReleaseVerified
        {
            match existing_zzmi_runtime(
                &store,
                &platform,
                &zzmi.manifest().id,
                &libraries.manifest().id,
                &protected_paths(&discovery),
            ) {
                Ok(Some(runtime)) => {
                    managed_runtime = ReadinessEvidenceState::Verified;
                    managed_runtime_root = Some(runtime.app_root);
                }
                Ok(None) => managed_runtime = ReadinessEvidenceState::Missing,
                Err(error) if error.code == ErrorCode::PermissionDenied => {
                    managed_runtime = ReadinessEvidenceState::Unreadable
                }
                Err(_) => managed_runtime = ReadinessEvidenceState::Invalid,
            }
        } else {
            managed_runtime = ReadinessEvidenceState::Incomplete;
        }
    }

    let capabilities = vec![
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::ExplicitLabProtonRuntime,
            state: ReadinessCapabilityState::Verified,
            evidence: "LXMI 0.7 registró un host test con Proton Experimental explícitamente elegido para el laboratorio. Este dato no selecciona Proton para ZZZ.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::DirectInjectInIsolatedLab,
            state: ReadinessCapabilityState::Verified,
            evidence: "Host test LXMI 0.7: Direct Inject cargó el DLL de prueba solo en el target LXMI fijo dentro del prefix aislado.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::HookInIsolatedLab,
            state: ReadinessCapabilityState::Blocked,
            evidence: "No ejecutado: el upstream instala un hook global y no se puede demostrar el límite de un único target que exige el Loader Lab.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::RemoteDllViaDirectInject,
            state: ReadinessCapabilityState::Verified,
            evidence: "El test DLL remoto informó su ruta Windows y marker/nonce válidos en el test target LXMI.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::RemoteDllViaHook,
            state: ReadinessCapabilityState::Blocked,
            evidence: "No probado porque el modo Hook no satisface el aislamiento por proceso del experimento.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::ManagedRuntimePathViaProton,
            state: ReadinessCapabilityState::Verified,
            evidence: "El bridge 0.6 leyó el manifiesto administrado mediante una ruta Windows en un contexto aislado; no representa el prefix de ZZZ.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::LoaderIdentitySemantics,
            state: ReadinessCapabilityState::UpstreamDocumented,
            evidence: "La configuración upstream de ZZMI describe loader= como la identidad ejecutable permitida para carga desde una ubicación externa; no se ha probado una identidad LXMI distinta.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::UpstreamDirectInjectCustomLaunch,
            state: ReadinessCapabilityState::UpstreamDocumented,
            evidence: "XXMI Launcher v2.2.1: Custom Launch con modo Inject selecciona use_hook=false y entra al flujo genérico Direct Inject. El lanzamiento estándar sigue heredando el default del paquete.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::UpstreamGenericDirectInject,
            state: ReadinessCapabilityState::UpstreamDocumented,
            evidence: "La ruta Inject está implementada en el MigotoInjector genérico y usa el export Inject de 3dmloader.dll; esto no verifica ZZMI sobre ZZZ/Linux/Proton.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::LoaderComponentLocalUse,
            state: ReadinessCapabilityState::Verified,
            evidence: "3dmloader.dll fue autenticado desde XXMI Libraries v1.1.7 y revisado para ejecución local en laboratorio. No se autoriza redistribución.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::DirectInjectProcessLifecycleInProton,
            state: ReadinessCapabilityState::Unknown,
            evidence: "El test controlado crea su propio target; no prueba inicio/detección/espera del juego iniciado por Steam bajo Proton.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::ZzzTargetProcessMapping,
            state: if installation == ReadinessEvidenceState::Verified
                && executable_state == ReadinessEvidenceState::Verified
            {
                ReadinessCapabilityState::Verified
            } else {
                ReadinessCapabilityState::Unknown
            },
            evidence: "La integración ZZZ fija ZenlessZoneZero.exe como target y el discovery local confirma el ejecutable; la resolución desde el proceso Windows de Steam/Proton no está probada.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::NativeSteamLaunchUpstream,
            state: ReadinessCapabilityState::UpstreamDocumented,
            evidence: "La documentación XXMI fija documenta Native Steam Launch para ZZZ en Windows. No prueba su traducción a Steam Linux/Proton.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::DirectSteamLaunchZzz,
            state: ReadinessCapabilityState::Unsupported,
            evidence: "La documentación upstream indica que Direct Steam Launch para ZZZ sigue pendiente.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::ZzzSteamUnderLinuxProton,
            state: ReadinessCapabilityState::Unverified,
            evidence: "No se ha ejecutado ZZMI/ZZZ mediante Steam Linux y Proton.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::HookVisibilityInSteamContext,
            state: ReadinessCapabilityState::Unknown,
            evidence: "No se sabe si el Hook instalado en un proceso Proton controlado será visible desde el contexto del juego iniciado por Steam.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::SamePrefixRequirement,
            state: ReadinessCapabilityState::Unknown,
            evidence: "No hay evidencia suficiente para afirmar si loader y juego deben compartir prefix/contexto.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::GameSpecificLoaderCompatibility,
            state: ReadinessCapabilityState::NotTested,
            evidence: "El laboratorio usó únicamente lxmi-loader-test-target.exe; ZZZ no se lanzó ni recibió una DLL.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::ZzzProcessIdentificationInProton,
            state: ReadinessCapabilityState::Unknown,
            evidence: "Upstream obtiene el nombre del proceso desde el ejecutable configurado y lo busca; esa detección en Steam/Proton para ZZZ no se ha probado.".into(),
        },
        ZzmiCapabilityAssessment {
            capability: ZzmiCapability::AntiCheatBoundary,
            state: ReadinessCapabilityState::Unknown,
            evidence: "No se investigó ni interactuó con anti-cheat; el comportamiento del flujo ZZZ permanece fuera de verificación.".into(),
        },
    ];

    let compatdata_path = compatdata_path(&platform.compatdata);
    let prefix = prefix_path(&platform.prefix);
    let steam_process_context =
        steam_process_context_evidence(&discovery, &tools.tools, &platform, app_id, observation);
    Ok(lxmi_xxmi::assess_zzmi_launch_readiness(
        ZzmiLaunchReadinessInput {
            game_id: platform.game.id.into(),
            game_name: platform.game.name.into(),
            app_id,
            distribution,
            installation,
            game_content_completeness,
            executable: executable_state,
            compatdata: compatdata_state,
            prefix: prefix_state,
            zzmi_package: zzmi.package_state,
            zzmi_authenticity: zzmi.authenticity,
            xxmi_libraries_package: libraries.package_state,
            xxmi_libraries_authenticity: libraries.authenticity,
            managed_runtime,
            game_runtime_selection,
            selected_game_runtime,
            available_proton_tools: platform
                .available_runtime_candidates
                .iter()
                .map(|tool| {
                    format!(
                        "{}{}",
                        tool.display_name,
                        tool.version
                            .as_ref()
                            .map(|version| format!(" · {version}"))
                            .unwrap_or_default()
                    )
                })
                .collect(),
            explicit_lab_runtime: Some("Proton Experimental · laboratorio LXMI 0.7".into()),
            game_install_path: install_path,
            game_executable_path: executable_path,
            compatdata_path,
            prefix_path: prefix,
            steam_process_context: Some(steam_process_context),
            managed_runtime_root,
            zzmi_version: zzmi.version,
            xxmi_libraries_version: libraries.version,
            upstream_default_loader_mode: ZzmiLoaderMode::Hook,
            upstream_loader_strategy: UpstreamLoaderStrategy::EitherDependingOnLaunchMode,
            hook_requirement: HookRequirementState::NotEstablished,
            capabilities,
        },
    ))
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
