//! Application orchestration; commands are thin async adapters.
use lxmi_core::SystemInfo;
use lxmi_proton::ProtonScanner;
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
use std::path::Path;

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
