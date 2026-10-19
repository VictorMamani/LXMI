//! «Play with mods»: one launch at a time through `lxmi-launch`. Paths, AppID and the
//! runtime are derived here from discovery and the saved selection, never the webview.
use lxmi_core::{GameExecutableStatus, GameInstallationStatus};
use lxmi_launch::{run_launch, LaunchStatus, ModLaunchTarget, SteamModLaunchHost};
use lxmi_steam::{SteamDiscoveryResult, SteamDiscoveryScanner};
use lxmi_xxmi::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[cfg(lxmi_mod_loader)]
const HELPER: Option<&[u8]> = Some(include_bytes!(concat!(env!("OUT_DIR"), "/lxmi-loader.exe")));
#[cfg(not(lxmi_mod_loader))]
const HELPER: Option<&[u8]> = None;

/// Only games whose loading was validated end to end (Hook, Steam + Proton).
const SUPPORTED_GAME: &str = "zenless-zone-zero";
/// Launcher Steam starts for ZZZ, relative to the install directory.
const ZZZ_LAUNCHER: &str = "HYP.exe";

struct Session {
    status: Arc<Mutex<LaunchStatus>>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct ModLaunchController {
    session: Mutex<Option<Session>>,
}

fn error(message: impl Into<String>) -> XxmiError {
    XxmiError::new(ErrorCode::Busy, None, message)
}

/// Reasons «Play with mods» cannot start; empty when it can.
pub(crate) fn launch_blockers(game_id: &str, selection: &ActiveProfileSelection) -> Vec<String> {
    let mut blockers = Vec::new();
    if game_id != SUPPORTED_GAME {
        blockers.push("Mod loading for this game has not been validated with Steam + Proton. Only Zenless Zone Zero is currently supported.".into());
    }
    if HELPER.is_none() {
        blockers.push(
            "This build does not include the Windows loader (MinGW was unavailable during compilation).".into(),
        );
    }
    if selection.current.is_none() {
        blockers.push("Apply a mod profile before playing with mods.".into());
    }
    blockers
}

fn target(discovery: &SteamDiscoveryResult, game_id: &str) -> Result<ModLaunchTarget> {
    let game = discovery
        .games
        .games
        .iter()
        .find(|g| {
            g.installation.game.id == game_id
                && g.installation.status == GameInstallationStatus::Installed
                && g.installation.executable_status == GameExecutableStatus::Found
        })
        .ok_or_else(|| error("No ready Steam installation was found."))?;
    let game_executable = game
        .installation
        .executable_path
        .clone()
        .ok_or_else(|| error("The game executable was not found."))?;
    let steam_root = discovery
        .steam
        .installations
        .iter()
        .find(|i| i.libraries.iter().any(|l| l.path == game.steam_library))
        .or(discovery.steam.installations.first())
        .map(|i| i.root_path.clone())
        .ok_or_else(|| error("Steam was not found."))?;
    Ok(ModLaunchTarget {
        game_id: game_id.into(),
        steam_app_id: game.steam_app_id,
        game_executable,
        launcher_executable: game.installation.install_path.join(ZZZ_LAUNCHER),
        compat_data: game.compatdata.compatdata_path.clone(),
        steam_root,
    })
}

fn selected_runtime(
    discovery: &SteamDiscoveryResult,
    store: &ManagedStore,
    game_id: &str,
) -> Result<(ManagedRuntime, String)> {
    let selection = profile_activation_status(store, game_id)?;
    let current = selection
        .current
        .ok_or_else(|| error("Apply a mod profile before playing with mods."))?;
    let platform = crate::xxmi_service::platform_plan(discovery, game_id)?;
    let base = existing_importer_runtime(
        store,
        &platform,
        &current.integration_package_id,
        &current.libraries_package_id,
        &crate::xxmi_service::protected_paths(discovery),
    )?
    .ok_or_else(|| {
        error(
            "The profile base runtime is missing. Prepare the runtime and apply the profile again.",
        )
    })?;
    Ok((resolve_profile_runtime(store, &base)?, current.profile_name))
}

/// Prepares the launch copy for the saved profile without starting any process.
pub(crate) fn prepare_copy(game_id: &str) -> Result<lxmi_launch::PreparedLaunch> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = crate::xxmi_service::store_for(&discovery)?;
    let (runtime, _) = selected_runtime(&discovery, &store, game_id)?;
    let helper = HELPER.ok_or_else(|| error("Loader not included."))?;
    lxmi_launch::prepare_launch_directory(&store, &runtime, helper).map_err(error)
}

#[tauri::command]
pub async fn start_game_with_mods(
    game_id: String,
    controller: tauri::State<'_, ModLaunchController>,
) -> Result<LaunchStatus> {
    {
        let session = controller.session.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(current) = session.as_ref() {
            if !current
                .status
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .finished
            {
                return Err(error("A launch with mods is already in progress."));
            }
        }
    }
    let id = game_id.clone();
    let (host, profile) = tauri::async_runtime::spawn_blocking(move || -> Result<_> {
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let store = crate::xxmi_service::store_for(&discovery)?;
        let selection = profile_activation_status(&store, &id)?;
        if let Some(blocker) = launch_blockers(&id, &selection).into_iter().next() {
            return Err(error(blocker));
        }
        let (runtime, profile) = selected_runtime(&discovery, &store, &id)?;
        let helper = HELPER
            .ok_or_else(|| error("Loader not included."))?
            .to_vec();
        Ok((
            SteamModLaunchHost::new(target(&discovery, &id)?, store, runtime, helper),
            profile,
        ))
    })
    .await
    .map_err(|_| error("Preparation ended unexpectedly."))??;

    let status = Arc::new(Mutex::new(LaunchStatus::new(&game_id, Some(profile))));
    let cancel = Arc::new(AtomicBool::new(false));
    let snapshot = status.lock().unwrap_or_else(|e| e.into_inner()).clone();
    *controller.session.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session {
        status: status.clone(),
        cancel: cancel.clone(),
    });
    std::thread::spawn(move || {
        let mut host = host;
        run_launch(&mut host, &status, &cancel);
        let s = status.lock().unwrap_or_else(|e| e.into_inner());
        tracing::info!(game = %s.game_id, phase = ?s.phase, mods_loaded = s.mods_loaded, "launch with mods finished");
    });
    Ok(snapshot)
}

#[tauri::command]
pub fn game_with_mods_status(
    controller: tauri::State<'_, ModLaunchController>,
) -> Option<LaunchStatus> {
    let session = controller.session.lock().unwrap_or_else(|e| e.into_inner());
    session
        .as_ref()
        .map(|s| s.status.lock().unwrap_or_else(|e| e.into_inner()).clone())
}

#[tauri::command]
pub fn cancel_game_with_mods(controller: tauri::State<'_, ModLaunchController>) -> bool {
    let session = controller.session.lock().unwrap_or_else(|e| e.into_inner());
    match session.as_ref() {
        Some(s) if !s.status.lock().unwrap_or_else(|e| e.into_inner()).finished => {
            s.cancel.store(true, Ordering::SeqCst);
            true
        }
        _ => false,
    }
}
