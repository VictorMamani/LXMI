//! Application orchestration; commands are thin async adapters.
use lxmi_core::SystemInfo;
use lxmi_proton::{CompatibilityToolKind, CompatibilityToolStatus, ProtonScanner};
use lxmi_runtime::{GameInstallationAssessment, RuntimePlanner};
use lxmi_steam::{SteamDiscoveryResult, SteamDiscoveryScanner};
use lxmi_xxmi::{
    assemble_zzmi_runtime, existing_zzmi_runtime, inspect_launch_topology, inspect_runtime,
    integration_for_game, plan_installation_for_integration, plan_zzmi_assembly, DownloadCache,
    ErrorCode, GitHubReleaseProvider, InstallationPlan, LaunchTopologyPlan, ManagedRuntime,
    ManagedStore, OfficialPackageKind, PackageManifest, ReleaseProvider, Result,
    RuntimeAssemblyPlan, RuntimeDiscovery, UpstreamRelease, XxmiError,
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
