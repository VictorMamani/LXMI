//! Application orchestration; commands are thin async adapters.
use lxmi_core::SystemInfo;
use lxmi_proton::ProtonScanner;
use lxmi_runtime::{GameInstallationAssessment, RuntimePlanner};
use lxmi_steam::{SteamDiscoveryResult, SteamDiscoveryScanner};
use lxmi_xxmi::{
    inspect_runtime, integration_for_game, plan_installation_for_integration, DownloadCache,
    ErrorCode, GitHubReleaseProvider, InstallationPlan, ManagedStore, OfficialPackageKind,
    PackageManifest, ReleaseProvider, Result, RuntimeDiscovery, UpstreamRelease, XxmiError,
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
fn store_for(discovery: &SteamDiscoveryResult) -> Result<ManagedStore> {
    let store = ManagedStore::from_system(&SystemInfo::current())?;
    let mut protected = Vec::new();
    for root in &discovery.steam.installations {
        protected.push(root.root_path.clone());
        protected.extend(root.libraries.iter().map(|l| l.path.clone()));
    }
    for game in &discovery.games.games {
        protected.push(game.installation.install_path.clone());
        protected.push(game.compatdata.compatdata_path.clone());
    }
    // Refuse writes if Steam discovery failed with an error: cannot establish boundaries.
    if !matches!(
        discovery.steam.status,
        lxmi_core::SteamDetectionStatus::Detected | lxmi_core::SteamDetectionStatus::NotInstalled
    ) {
        return Err(XxmiError::new(ErrorCode::StorageUnavailable,None,"Resuelve el diagnóstico de Steam antes de importar paquetes: límites de almacenamiento no comprobados."));
    }
    store.ensure_outside(&protected)?;
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
            "zenless-zone-zero" => {
                // The local game's executable may be nested. Inspect the manifest root and the
                // observed executable directory, plus their documented importer subdirectory.
                let executable_parent = game
                    .installation
                    .executable_path
                    .as_deref()
                    .and_then(std::path::Path::parent)
                    .map(std::path::Path::to_path_buf);
                let mut candidates = vec![
                    game.installation.install_path.clone(),
                    game.installation.install_path.join("ZZMI"),
                ];
                if let Some(parent) = executable_parent {
                    candidates.push(parent.join("ZZMI"));
                    candidates.push(parent);
                }
                candidates.sort();
                candidates.dedup();
                for path in candidates {
                    observations.push(inspect_runtime(&path));
                }
            }
            _ => {}
        }
    }
    observations.push(inspect_runtime(&store.root().join("runtimes/wwmi")));
    observations.push(inspect_runtime(&store.root().join("runtimes/zzmi")));
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
