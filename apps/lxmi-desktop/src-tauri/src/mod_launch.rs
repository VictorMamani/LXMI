//! «Play with mods»: one launch at a time through `lxmi-launch`. Paths, AppID and the
//! runtime are derived here from discovery and the saved selection, never the webview.
use lxmi_core::{GameExecutableStatus, GameInstallationStatus};
use lxmi_launch::{
    preflight, run_launch, LaunchStatus, LoadMode, ModLaunchTarget, PreflightInput,
    SteamModLaunchHost,
};
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

/// Everything «Play with mods» needs for one game. Fixed here: nothing is taken from the
/// webview or from the saved profile.
struct LaunchSpec {
    game_id: &'static str,
    /// Launcher Steam starts, relative to the install directory; the player presses Play in it.
    launcher: &'static str,
    label: &'static str,
    /// Game process the loader waits for, relative to the install directory. `None` keeps
    /// the executable discovery found (ZZZ).
    game: Option<&'static str>,
    /// Importer folder inside the launch copy, as laid out by the assembler.
    importer_folder: &'static str,
    /// Extra arguments for `steam -applaunch`.
    launch_arguments: &'static [&'static str],
    load_mode: LoadMode,
    /// Shown as experimental in the UI until the path is validated in game.
    experimental: bool,
}

/// ZZZ uses the validated hook path on Steam + Proton. Wuthering Waves uses
/// experimental Direct Inject and has not been validated in gameplay.
const LAUNCH_SPECS: &[LaunchSpec] = &[
    LaunchSpec {
        game_id: "zenless-zone-zero",
        launcher: "HYP.exe",
        label: "HoYoPlay",
        game: None,
        importer_folder: "ZZMI",
        launch_arguments: &[],
        load_mode: LoadMode::Hook,
        experimental: false,
    },
    LaunchSpec {
        game_id: "wuthering-waves",
        launcher: "Wuthering Waves.exe",
        label: "Wuthering Waves",
        game: Some("Client/Binaries/Win64/Client-Win64-Shipping.exe"),
        importer_folder: "WWMI",
        // WWMI requires DX11 (upstream wiki: «else WWMI will crash»).
        launch_arguments: &["-dx11"],
        // Upstream WWMI default since v2.2.0 is Direct Inject; hooking no longer works.
        load_mode: LoadMode::DirectInject,
        experimental: true,
    },
];

fn launch_spec(game_id: &str) -> Option<&'static LaunchSpec> {
    LAUNCH_SPECS.iter().find(|spec| spec.game_id == game_id)
}

/// True when «Play with mods» for this game is not yet validated in gameplay.
pub(crate) fn launch_is_experimental(game_id: &str) -> bool {
    launch_spec(game_id).is_some_and(|spec| spec.experimental)
}

/// Static preflight of the selected runtime against the game installation. Reads files
/// and metadata only; nothing is started. Empty when the launch may proceed.
pub(crate) fn runtime_blockers(
    discovery: &SteamDiscoveryResult,
    game_id: &str,
    runtime: &ManagedRuntime,
) -> Vec<String> {
    match target(discovery, game_id) {
        Ok(target) => preflight(&PreflightInput {
            target: &target,
            importer_root: &runtime.importer_root,
            runtime_target: &runtime.target_process,
        }),
        Err(reason) => vec![reason.to_string()],
    }
}

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
    if launch_spec(game_id).is_none() {
        blockers.push("Mod loading is not available for this game yet.".into());
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
    let spec =
        launch_spec(game_id).ok_or_else(|| error("Mod loading is unavailable for this game."))?;
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
    let game_executable = match spec.game {
        Some(relative) => game.installation.install_path.join(relative),
        None => game
            .installation
            .executable_path
            .clone()
            .ok_or_else(|| error("The game executable was not found."))?,
    };
    if !game_executable.is_file() {
        return Err(error(
            "The game process executable was not found in the Steam installation.",
        ));
    }
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
        launcher_executable: game.installation.install_path.join(spec.launcher),
        compat_data: game.compatdata.compatdata_path.clone(),
        steam_root,
        integration_folder: spec.importer_folder,
        launcher_label: spec.label,
        launch_arguments: spec.launch_arguments,
        load_mode: spec.load_mode,
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
        let blockers = runtime_blockers(&discovery, &id, &runtime);
        if !blockers.is_empty() {
            tracing::warn!(game = %id, blockers = blockers.len(), "launch with mods refused by preflight");
            return Err(error(blockers.join(" ")));
        }
        let launch_target = target(&discovery, &id)?;
        tracing::info!(
            game = %id,
            strategy = ?launch_target.load_mode,
            experimental = launch_is_experimental(&id),
            runtime = %runtime.runtime_id,
            importer = %launch_target.integration_folder,
            importer_version = ?runtime.integration_version,
            importer_target = %runtime.target_process,
            launch_arguments = ?launch_target.launch_arguments,
            "launch with mods: preflight passed"
        );
        let helper = HELPER
            .ok_or_else(|| error("Loader not included."))?
            .to_vec();
        Ok((
            SteamModLaunchHost::new(launch_target, store, runtime, helper),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_supported_game_maps_to_one_importer_folder() {
        let zzz = launch_spec("zenless-zone-zero").expect("ZZZ");
        assert_eq!((zzz.importer_folder, zzz.launcher), ("ZZMI", "HYP.exe"));
        assert_eq!(
            zzz.load_mode,
            LoadMode::Hook,
            "ZZZ keeps the validated hook"
        );
        assert!(!zzz.experimental);
        assert_eq!(zzz.game, None, "ZZZ keeps the executable discovery found");
        assert!(zzz.launch_arguments.is_empty());

        let wuwa = launch_spec("wuthering-waves").expect("Wuthering Waves");
        assert_eq!(wuwa.importer_folder, "WWMI");
        assert_eq!(wuwa.launcher, "Wuthering Waves.exe");
        assert_eq!(
            wuwa.game,
            Some("Client/Binaries/Win64/Client-Win64-Shipping.exe")
        );
        assert_eq!(wuwa.launch_arguments, &["-dx11"]);
        assert_eq!(wuwa.load_mode, LoadMode::DirectInject);
        assert!(
            wuwa.experimental,
            "WWMI stays experimental until validated in game"
        );
        assert!(launch_is_experimental("wuthering-waves"));
        assert!(!launch_is_experimental("zenless-zone-zero"));
    }

    #[test]
    fn unknown_games_have_no_launch_spec_and_a_blocker() {
        assert!(launch_spec("unknown-game").is_none());
        let selection = ActiveProfileSelection {
            schema_version: 1,
            game_id: "unknown-game".into(),
            revision: 0,
            current: None,
            previous: None,
        };
        let blockers = launch_blockers("unknown-game", &selection);
        assert!(blockers.iter().any(|b| b.contains("not available")));
    }
}
