//! «Play with mods»: one launch at a time through `lxmi-launch`. Paths, AppID and the
//! runtime are derived here from discovery and the saved selection, never the webview.
use lxmi_core::SteamInstallation;
use lxmi_core::{GameExecutableStatus, GameInstallationStatus};
use lxmi_launch::{
    preflight, run_launch, LaunchStatus, LoadMode, ModLaunchTarget, PreflightInput,
    SteamModLaunchHost, SteamProcessContext,
};
use lxmi_proton::{parse_tool_manifest_vdf, CompatibilityToolKind, ProtonScanner};
use lxmi_steam::{
    inspect_steam_compat_tool_config, parse_app_manifest, SteamDiscoveryResult,
    SteamDiscoveryScanner,
};
use lxmi_xxmi::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
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
    /// Shown as experimental while gameplay validation covers only a limited setup.
    experimental: bool,
}

/// ZZZ uses the validated hook path on Steam + Proton. Wuthering Waves uses
/// experimental Direct Inject, tested with dwproton 11.0-14 and DXVK Sarek.
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

/// Steam's own Proton names that local files can map to an AppID without `appinfo.vdf`.
const BUILTIN_TOOLS: &[(&str, u32)] = &[("proton_experimental", 1493710)];

/// `steamapps/common/<installdir>` of an installed Steam app in any library.
fn installed_app_dir(libraries: &[PathBuf], app_id: u32) -> Option<PathBuf> {
    libraries.iter().find_map(|library| {
        let steamapps = library.join("steamapps");
        let text = fs::read_to_string(steamapps.join(format!("appmanifest_{app_id}.acf"))).ok()?;
        let manifest = parse_app_manifest(&text).ok()?;
        (manifest.app_id == app_id).then(|| steamapps.join("common").join(manifest.install_dir))
    })
}

fn is_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}

/// Proton and Steam Linux Runtime Steam will use for this game, read from Steam's own
/// configuration before anything starts. Only an explicit per-game choice is trusted:
/// without it Steam may pick a tool from data LXMI cannot read, and a loader started in a
/// different Proton could not join the game's wineserver.
fn planned_steam_context(
    installations: &[SteamInstallation],
    target: &ModLaunchTarget,
) -> std::result::Result<SteamProcessContext, String> {
    let name = inspect_steam_compat_tool_config(&target.steam_root, target.steam_app_id)
        .mappings
        .and_then(|m| m.per_app)
        .and_then(|m| m.internal_name)
        .filter(|n| !n.trim().is_empty())
        .ok_or("Choose a compatibility tool for this game in Steam (Properties → Compatibility → Force a specific Steam Play compatibility tool) so LXMI knows which Proton Steam will use.")?;
    let libraries: Vec<PathBuf> = installations
        .iter()
        .flat_map(|i| i.libraries.iter().map(|l| l.path.clone()))
        .collect();
    let custom = ProtonScanner
        .scan(installations)
        .tools
        .into_iter()
        .find(|tool| {
            tool.internal_id.as_deref() == Some(name.as_str())
                && tool.kind == CompatibilityToolKind::Proton
        });
    let proton_dir = custom
        .map(|tool| tool.path)
        .or_else(|| {
            BUILTIN_TOOLS
                .iter()
                .find(|(builtin, _)| *builtin == name)
                .and_then(|(_, app)| installed_app_dir(&libraries, *app))
        })
        .and_then(|dir| fs::canonicalize(dir).ok())
        .filter(|dir| is_file(&dir.join("proton")))
        .ok_or_else(|| {
            format!("LXMI cannot locate the compatibility tool «{name}» selected in Steam.")
        })?;
    let runtime_app = fs::read_to_string(proton_dir.join("toolmanifest.vdf"))
        .ok()
        .and_then(|text| parse_tool_manifest_vdf(&text).ok())
        .and_then(|manifest| manifest.require_tool_appid)
        .and_then(|app| app.trim().parse::<u32>().ok())
        .ok_or_else(|| format!("«{name}» does not declare the Steam Linux Runtime it needs."))?;
    let runtime_dir = installed_app_dir(&libraries, runtime_app)
        .filter(|dir| is_file(&dir.join("_v2-entry-point")))
        .ok_or_else(|| format!("The Steam Linux Runtime required by «{name}» (AppID {runtime_app}) is not installed. Open any game with it once from Steam."))?;
    Ok(SteamProcessContext {
        pid: 0,
        start_ticks: 0,
        compat_data: fs::canonicalize(&target.compat_data)
            .unwrap_or_else(|_| target.compat_data.clone()),
        proton_dir,
        runtime_dir,
        steam_client_dir: Some(target.steam_root.clone()),
    })
}

/// True when «Play with mods» for this game has only limited gameplay validation.
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
        Ok(target) => {
            let mut blockers = preflight(&PreflightInput {
                target: &target,
                importer_root: &runtime.importer_root,
                runtime_target: &runtime.target_process,
            });
            if target.load_mode == LoadMode::DirectInject {
                if let Err(reason) = planned_steam_context(&discovery.steam.installations, &target)
                {
                    blockers.push(reason);
                }
            }
            blockers
        }
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
        // Direct Inject has no launcher to wait in: stage the loader with the tools Steam
        // will use and release it into the game's wineserver once Steam started the game.
        let planned = match launch_target.load_mode {
            LoadMode::DirectInject => Some(
                planned_steam_context(&discovery.steam.installations, &launch_target)
                    .map_err(error)?,
            ),
            LoadMode::Hook => None,
        };
        if let Some(context) = &planned {
            tracing::info!(
                game = %id,
                proton = %context.proton_dir.display(),
                runtime = %context.runtime_dir.display(),
                "launch with mods: loader staged before Steam starts the game"
            );
        }
        let helper = HELPER
            .ok_or_else(|| error("Loader not included."))?
            .to_vec();
        let host = SteamModLaunchHost::new(launch_target, store, runtime, helper);
        Ok((
            match planned {
                Some(context) => host.with_planned_context(context),
                None => host,
            },
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
            "WWMI stays experimental while validation is limited to one setup"
        );
        assert!(launch_is_experimental("wuthering-waves"));
        assert!(!launch_is_experimental("zenless-zone-zero"));
    }

    struct SteamTree(PathBuf);
    impl Drop for SteamTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Steam root with a per-game mapping, a custom Proton and the runtime it requires.
    fn steam_tree(mapping: Option<&str>) -> (SteamTree, Vec<SteamInstallation>, ModLaunchTarget) {
        let root = std::env::temp_dir().join(format!(
            "lxmi-planned-{}-{}",
            std::process::id(),
            mapping.unwrap_or("none")
        ));
        let _ = fs::remove_dir_all(&root);
        let write = |path: &Path, text: &str| {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        let config = mapping.map_or(String::new(), |name| {
            format!(r#""3513350" {{ "name" "{name}" "config" "" "priority" "250" }}"#)
        });
        write(
            &root.join("config/config.vdf"),
            &format!(
                r#""InstallConfigStore" {{ "Software" {{ "Valve" {{ "Steam" {{ "CompatToolMapping" {{ {config} }} }} }} }} }}"#
            ),
        );
        let tool = root.join("compatibilitytools.d/custom-proton");
        write(
            &tool.join("compatibilitytool.vdf"),
            r#""compatibilitytools" { "compat_tools" { "custom-proton" { "install_path" "." "display_name" "Custom Proton" "from_oslist" "windows" "to_oslist" "linux" } } }"#,
        );
        write(
            &tool.join("toolmanifest.vdf"),
            r#""manifest" { "version" "2" "commandline" "/proton %verb%" "require_tool_appid" "4183110" "compatmanager_layer_name" "proton" }"#,
        );
        write(&tool.join("proton"), "");
        write(
            &root.join("steamapps/appmanifest_4183110.acf"),
            r#""AppState" { "appid" "4183110" "name" "Steam Linux Runtime 4.0" "installdir" "SteamLinuxRuntime_4" }"#,
        );
        write(
            &root.join("steamapps/common/SteamLinuxRuntime_4/_v2-entry-point"),
            "",
        );
        let compat = root.join("steamapps/compatdata/3513350");
        fs::create_dir_all(compat.join("pfx")).unwrap();
        let installations = vec![SteamInstallation {
            root_path: root.clone(),
            libraries: vec![lxmi_core::SteamLibrary {
                path: root.clone(),
                is_default: true,
            }],
        }];
        let target = ModLaunchTarget {
            game_id: "wuthering-waves".into(),
            steam_app_id: 3513350,
            game_executable: root.join("game.exe"),
            launcher_executable: root.join("launcher.exe"),
            compat_data: compat,
            steam_root: root.clone(),
            integration_folder: "WWMI",
            launcher_label: "Wuthering Waves",
            launch_arguments: &["-dx11"],
            load_mode: LoadMode::DirectInject,
        };
        (SteamTree(root), installations, target)
    }

    #[test]
    fn plans_the_per_game_proton_and_the_runtime_it_requires() {
        let (tree, installations, target) = steam_tree(Some("custom-proton"));
        let context = planned_steam_context(&installations, &target).expect("planned");
        let root = fs::canonicalize(&tree.0).unwrap();
        assert_eq!(
            fs::canonicalize(&context.proton_dir).unwrap(),
            root.join("compatibilitytools.d/custom-proton")
        );
        assert_eq!(
            context.runtime_dir,
            tree.0.join("steamapps/common/SteamLinuxRuntime_4")
        );
        assert_eq!(
            context.compat_data,
            root.join("steamapps/compatdata/3513350")
        );
    }

    #[test]
    fn refuses_to_guess_without_a_per_game_choice_or_with_an_unknown_tool() {
        let (_tree, installations, target) = steam_tree(None);
        let reason = planned_steam_context(&installations, &target).unwrap_err();
        assert!(reason.contains("Properties"), "{reason}");

        let (_tree, installations, target) = steam_tree(Some("proton_9"));
        let reason = planned_steam_context(&installations, &target).unwrap_err();
        assert!(reason.contains("proton_9"), "{reason}");
    }

    /// Resolves against this machine's Steam: `cargo test -p lxmi-desktop -- --ignored planned`.
    #[test]
    #[ignore = "reads the real Steam installation"]
    fn planned_context_on_this_machine() {
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let target = target(&discovery, "wuthering-waves").expect("Wuthering Waves target");
        let context = planned_steam_context(&discovery.steam.installations, &target);
        eprintln!("{context:#?}");
        assert!(context.is_ok());
    }

    /// Stages the real WWMI loader container through Steam and cancels it without release:
    /// no Wine process starts and the game prefix is untouched. Needs Steam running and an
    /// applied WWMI profile: `cargo test -p lxmi-desktop -- --ignored staged_loader`.
    #[test]
    #[ignore = "starts a Steam Linux Runtime container"]
    fn staged_loader_container_reports_and_cancels_without_wine() {
        use lxmi_launch::{HelperEvent, HelperRead, LaunchHost};
        use std::time::{Duration, Instant};
        let id = "wuthering-waves";
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let store = crate::xxmi_service::store_for(&discovery).expect("store");
        let (runtime, _) = selected_runtime(&discovery, &store, id).expect("applied profile");
        let launch_target = target(&discovery, id).expect("target");
        let planned = planned_steam_context(&discovery.steam.installations, &launch_target)
            .expect("planned context");
        let mut host = SteamModLaunchHost::new(
            launch_target,
            store,
            runtime,
            HELPER.expect("loader").to_vec(),
        )
        .with_planned_context(planned.clone());
        let dir = host.prepare().expect("launch copy");
        let started = Instant::now();
        let mut helper = host.spawn_helper(&planned).expect("spawn");
        let mut staged = false;
        while !staged && started.elapsed() < Duration::from_secs(60) {
            match helper.read_line(Duration::from_millis(250)) {
                HelperRead::Line(line) => {
                    eprintln!("helper: {line}");
                    staged = HelperEvent::parse(&line) == Some(HelperEvent::Staged);
                }
                HelperRead::Timeout => {}
                HelperRead::Closed => break,
            }
        }
        let staged_after = started.elapsed();
        let detail = helper.exit_detail();
        helper.terminate();
        helper.wait();
        let marker = dir.file_name().unwrap().to_string_lossy().into_owned();
        let survivors = fs::read_dir("/proc")
            .unwrap()
            .flatten()
            .filter_map(|e| fs::read(e.path().join("cmdline")).ok())
            .filter(|c| String::from_utf8_lossy(c).contains(&marker))
            .count();
        eprintln!(
            "staged={staged} after_ms={} survivors={survivors}",
            staged_after.as_millis()
        );
        assert!(staged, "no loader_staged: {detail:?}");
        assert!(!dir.join("loader-release").exists(), "never released");
        assert_eq!(survivors, 0, "staged processes survived cancellation");
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
