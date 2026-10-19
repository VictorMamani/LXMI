//! Real side effects: procfs, `steam -applaunch`, and the helper inside the Steam
//! Linux Runtime + Proton that Steam itself chose for the launcher.
use crate::{
    context::{count_processes, find_steam_process, ProcessQuery, SteamProcessContext},
    session::{HelperProcess, HelperRead, LaunchHost},
    workdir::{prepare_launch_directory, HELPER_NAME},
};
use lxmi_xxmi::{ManagedRuntime, ManagedStore};
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{fs::DirBuilderExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicI32, Ordering},
        mpsc::{channel, Receiver, RecvTimeoutError},
        Arc,
    },
    time::Duration,
};

/// Environment passed to Steam Runtime; everything else is cleared.
const DESKTOP_ENV: [&str; 11] = [
    "HOME",
    "USER",
    "LOGNAME",
    "PATH",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XAUTHORITY",
    "XDG_RUNTIME_DIR",
    "DBUS_SESSION_BUS_ADDRESS",
    "LANG",
    "LC_ALL",
];
/// `proton runinprefix` skips setup_prefix, where Proton adds its DXVK/vkd3d-proton
/// native overrides. Without them the module resolves as Wine's builtin d3d11 and
/// HookLibrary fails with 300 (hook isolation fixture, 2026-10-15).
const DLL_OVERRIDES: &str = "d3d11,d3d10core,d3d9,dxgi,d3d12,d3d12core=n";

/// How the helper gets the importer into the game. Chosen per game by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMode {
    /// ZZZ, validated on Steam + Proton: global hook installed before the game exists.
    Hook,
    /// WWMI upstream default since v2.2.0: `Inject` on the bound game PID. Experimental.
    DirectInject,
}

impl LoadMode {
    /// Helper verb; the helper rejects anything else.
    pub fn helper_verb(self) -> &'static str {
        match self {
            Self::Hook => "hook-wait",
            Self::DirectInject => "inject-wait",
        }
    }
}

/// Importer folders the helper accepts. Anything else is refused before Proton starts.
pub const IMPORTER_FOLDERS: &[&str] = &["ZZMI", "WWMI"];

#[derive(Debug, Clone)]
pub struct ModLaunchTarget {
    pub game_id: String,
    pub steam_app_id: u32,
    pub game_executable: PathBuf,
    /// Launcher Steam starts for this AppID (HoYoPlay for ZZZ).
    pub launcher_executable: PathBuf,
    pub compat_data: PathBuf,
    pub steam_root: PathBuf,
    /// Importer folder in the launch copy: `ZZMI` or `WWMI`. Fixed by the caller's table.
    pub integration_folder: &'static str,
    /// Launcher name shown in messages, e.g. «HoYoPlay» or «Wuthering Waves».
    pub launcher_label: &'static str,
    /// Extra arguments for `steam -applaunch`, e.g. `-dx11` for WWMI.
    pub launch_arguments: &'static [&'static str],
    pub load_mode: LoadMode,
}

pub struct SteamModLaunchHost {
    target: ModLaunchTarget,
    store: ManagedStore,
    runtime: ManagedRuntime,
    helper: Vec<u8>,
    uid: u32,
    launch_dir: Option<PathBuf>,
}

impl SteamModLaunchHost {
    pub fn new(
        target: ModLaunchTarget,
        store: ManagedStore,
        runtime: ManagedRuntime,
        helper: Vec<u8>,
    ) -> Self {
        Self {
            target,
            store,
            runtime,
            helper,
            uid: unsafe { libc::getuid() },
            launch_dir: None,
        }
    }
    fn query(&self, executable: &Path) -> ProcessQuery {
        ProcessQuery {
            executable: executable.to_path_buf(),
            compat_data: self.target.compat_data.clone(),
            app_id: self.target.steam_app_id,
        }
    }
}

fn windows_path(path: &Path) -> Result<String, String> {
    let canonical =
        fs::canonicalize(path).map_err(|e| format!("No se encontró {}: {e}", path.display()))?;
    let text = canonical
        .to_str()
        .ok_or("Ruta del juego no representable.")?;
    Ok(format!("Z:{}", text.replace('/', "\\")))
}

impl LaunchHost for SteamModLaunchHost {
    fn game_running(&self) -> Result<bool, String> {
        Ok(count_processes(
            Path::new("/proc"),
            self.uid,
            &self.query(&self.target.game_executable),
        )? > 0)
    }

    fn prepare(&mut self) -> Result<PathBuf, String> {
        let prepared = prepare_launch_directory(&self.store, &self.runtime, &self.helper)?;
        self.launch_dir = Some(prepared.dir.clone());
        Ok(prepared.dir)
    }

    fn find_launcher(&self) -> Result<Option<SteamProcessContext>, String> {
        find_steam_process(
            Path::new("/proc"),
            self.uid,
            &self.query(&self.target.launcher_executable),
        )
    }

    fn launcher_label(&self) -> String {
        self.target.launcher_label.into()
    }

    fn request_steam_launch(&self) -> Result<(), String> {
        // Fixed AppID and arguments from the caller's table, direct argv, no shell or URI.
        let mut child = Command::new("steam")
            .arg("-applaunch")
            .arg(self.target.steam_app_id.to_string())
            .args(self.target.launch_arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("No se pudo enviar la solicitud a Steam: {e}"))?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }

    fn spawn_helper(
        &mut self,
        launcher: &SteamProcessContext,
    ) -> Result<Box<dyn HelperProcess>, String> {
        let dir = self
            .launch_dir
            .clone()
            .ok_or("La copia de arranque no está preparada.")?;
        spawn_helper_in(&dir, &self.target, launcher)
    }

    fn pause(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Starts `LXMI Loader.exe hook-wait` from `dir` inside the launcher's Proton/SLR.
fn spawn_helper_in(
    dir: &Path,
    target: &ModLaunchTarget,
    launcher: &SteamProcessContext,
) -> Result<Box<dyn HelperProcess>, String> {
    let entry = launcher.runtime_dir.join("_v2-entry-point");
    let proton = launcher.proton_dir.join("proton");
    for file in [&entry, &proton] {
        if !fs::symlink_metadata(file).is_ok_and(|m| m.is_file()) {
            return Err(format!("No se encontró {}.", file.display()));
        }
    }
    let variable = dir.join("runtime-variable");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&variable)
        .map_err(|e| format!("No se pudo preparar el runtime: {e}"))?;
    let stderr_path = dir.join("helper.stderr.log");
    let stderr = fs::File::create(&stderr_path).map_err(|e| e.to_string())?;
    let app = target.steam_app_id.to_string();
    let client = launcher
        .steam_client_dir
        .clone()
        .unwrap_or_else(|| target.steam_root.clone());

    let mut env: Vec<(OsString, OsString)> = DESKTOP_ENV
        .iter()
        .filter_map(|key| std::env::var_os(key).map(|v| (OsString::from(key), v)))
        .collect();
    let desktop_env = env.len();
    for (key, value) in [
        ("STEAM_COMPAT_DATA_PATH", launcher.compat_data.as_os_str()),
        ("STEAM_COMPAT_CLIENT_INSTALL_PATH", client.as_os_str()),
        ("SteamAppId", app.as_ref()),
        ("SteamGameId", app.as_ref()),
        ("STEAM_COMPAT_APP_ID", app.as_ref()),
        ("PRESSURE_VESSEL_VARIABLE_DIR", variable.as_os_str()),
        ("WINEDEBUG", "-all".as_ref()),
        ("WINEDLLOVERRIDES", DLL_OVERRIDES.as_ref()),
    ] {
        env.push((key.into(), value.to_os_string()));
    }
    let mut run = helper_run_arguments(&entry, &proton, dir, target)?;

    // Ubuntu's apparmor_restrict_unprivileged_userns denies the container's user
    // namespace to unprofiled processes such as this app (bwrap "setting up uid
    // map: Permission denied"). Steam runs under a profile that allows it, so the
    // command goes through Steam's own launcher service when it answers.
    let launch_client = launcher
        .runtime_dir
        .join("pressure-vessel/bin/steam-runtime-launch-client");
    let alongside = alongside_steam_available(&launch_client);
    let mut command = if alongside {
        let mut command = Command::new(&launch_client);
        command.arg("--alongside-steam").arg("--clear-env");
        let mut directory = OsString::from("--directory=");
        directory.push(dir);
        command.arg(directory);
        for (key, value) in &env {
            let mut arg = OsString::from("--env=");
            arg.push(key);
            arg.push("=");
            arg.push(value);
            command.arg(arg);
        }
        // The service starts the command in a new session: announce its group so
        // cancelling reaches the whole tree, not just the forwarded direct child.
        command.args(["--", "/bin/sh", "-c", GROUP_WRAPPER, "sh"]);
        command.args(&run);
        command.env_clear();
        for (key, value) in &env[..desktop_env] {
            command.env(key, value);
        }
        command
    } else {
        let mut command = Command::new(run.remove(0));
        command.args(&run).current_dir(dir).env_clear();
        for (key, value) in &env {
            command.env(key, value);
        }
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|e| format!("No se pudo iniciar el cargador: {e}"))?;
    let stdout = child.stdout.take().ok_or("Sin salida del cargador.")?;
    let mut journal =
        fs::File::create(dir.join("helper.stdout.jsonl")).map_err(|e| e.to_string())?;
    let group = Arc::new(AtomicI32::new(if alongside {
        0
    } else {
        child.id() as i32
    }));
    let announced = Arc::clone(&group);
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = writeln!(journal, "{line}");
            if let Some(pgid) = announced_group(&line) {
                if alongside && announced.load(Ordering::SeqCst) == 0 {
                    announced.store(pgid, Ordering::SeqCst);
                }
                continue;
            }
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    Ok(Box::new(SteamHelper {
        child,
        lines: rx,
        group,
        stderr_path,
    }))
}

/// Argument vector for `SteamLinuxRuntime -- proton runinprefix <helper> <verb> <game> <folder>`.
/// Every value is fixed by the caller's table except the game path, which is canonicalized.
pub(crate) fn helper_run_arguments(
    entry: &Path,
    proton: &Path,
    dir: &Path,
    target: &ModLaunchTarget,
) -> Result<Vec<OsString>, String> {
    if !IMPORTER_FOLDERS.contains(&target.integration_folder) {
        return Err("Carpeta de importador no permitida.".into());
    }
    Ok(vec![
        entry.into(),
        "--verb=run".into(),
        "--".into(),
        proton.into(),
        "runinprefix".into(),
        dir.join(HELPER_NAME).into(),
        target.load_mode.helper_verb().into(),
        windows_path(&target.game_executable)?.into(),
        target.integration_folder.into(),
    ])
}

const GROUP_WRAPPER: &str = r#"printf '{"status":"launch_group","pgid":%d}\n' "$$"; exec "$@""#;

fn announced_group(line: &str) -> Option<i32> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    if value.get("status")?.as_str()? != "launch_group" {
        return None;
    }
    i32::try_from(value.get("pgid")?.as_u64()?)
        .ok()
        .filter(|pgid| *pgid > 1)
}

/// Asks Steam's launcher service to run `true`; any failure falls back to direct spawn.
fn alongside_steam_available(client: &Path) -> bool {
    if !fs::symlink_metadata(client).is_ok_and(|m| m.is_file()) {
        return false;
    }
    let mut command = Command::new(client);
    command
        .args(["--alongside-steam", "--clear-env", "--", "true"])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for key in DESKTOP_ENV {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    let Ok(mut child) = command.spawn() else {
        return false;
    };
    for _ in 0..100 {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    false
}

/// Last meaningful stderr line, explained when it is a known container failure.
fn stderr_detail(text: &str) -> Option<String> {
    if text.contains("setting up uid map")
        || text.contains("No permissions to create new namespace")
    {
        return Some("el sistema no permitió a Steam Linux Runtime crear su contenedor (AppArmor restringe los user namespaces) y el servicio de lanzamiento de Steam no respondió. Reinicia Steam y vuelve a intentarlo.".into());
    }
    let line = text.lines().rev().map(str::trim).find(|l| !l.is_empty())?;
    Some(line.chars().take(300).collect())
}

struct SteamHelper {
    child: Child,
    lines: Receiver<String>,
    /// Process group of the helper tree; 0 until the wrapper announces it.
    group: Arc<AtomicI32>,
    stderr_path: PathBuf,
}

impl HelperProcess for SteamHelper {
    fn read_line(&mut self, timeout: Duration) -> HelperRead {
        match self.lines.recv_timeout(timeout) {
            Ok(line) => HelperRead::Line(line),
            Err(RecvTimeoutError::Timeout) => HelperRead::Timeout,
            Err(RecvTimeoutError::Disconnected) => HelperRead::Closed,
        }
    }
    fn terminate(&mut self) {
        for _ in 0..20 {
            if self.group.load(Ordering::SeqCst) > 1 || !matches!(self.child.try_wait(), Ok(None)) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        // Own process group only: never the shared wineserver, launcher or game.
        let group = self.group.load(Ordering::SeqCst);
        unsafe {
            if group > 1 {
                libc::kill(-group, libc::SIGTERM);
            }
            libc::kill(self.child.id() as i32, libc::SIGTERM);
        }
    }
    fn wait(&mut self) -> Option<i32> {
        self.child.wait().ok().and_then(|s| s.code())
    }
    fn exit_detail(&mut self) -> Option<String> {
        let bytes = fs::read(&self.stderr_path).ok()?;
        stderr_detail(&String::from_utf8_lossy(&bytes))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn game_path_is_passed_through_the_z_drive() {
        let path = super::windows_path(std::path::Path::new("/")).unwrap();
        assert_eq!(path, "Z:\\");
    }

    #[test]
    fn only_the_wrapper_announcement_sets_the_group() {
        assert_eq!(
            super::announced_group(r#"{"status":"launch_group","pgid":4242}"#),
            Some(4242)
        );
        assert_eq!(
            super::announced_group(r#"{"status":"launch_group","pgid":1}"#),
            None
        );
        assert_eq!(
            super::announced_group(r#"{"status":"hook_library_returned","code":0}"#),
            None
        );
    }

    /// Arms and cancels the real helper in an LXMI-owned fixture prefix (never the
    /// game's): `LXMI_FIXTURE_{LAUNCH_DIR,COMPAT,TARGET,RUNTIME,PROTON,STEAM}`.
    #[test]
    #[ignore = "needs Steam, Proton and an LXMI fixture prefix"]
    fn fixture_helper_arms_and_cancels_through_steam() {
        use crate::{events::HelperEvent, session::HelperRead};
        use std::time::{Duration, Instant};
        let var = |k: &str| std::path::PathBuf::from(std::env::var_os(k).expect(k));
        let dir = var("LXMI_FIXTURE_LAUNCH_DIR");
        let target = super::ModLaunchTarget {
            game_id: "fixture".into(),
            steam_app_id: 0,
            game_executable: var("LXMI_FIXTURE_TARGET"),
            launcher_executable: var("LXMI_FIXTURE_TARGET"),
            compat_data: var("LXMI_FIXTURE_COMPAT"),
            steam_root: var("LXMI_FIXTURE_STEAM"),
            integration_folder: "ZZMI",
            launcher_label: "fixture",
            launch_arguments: &[],
            load_mode: super::LoadMode::Hook,
        };
        let launcher = crate::context::SteamProcessContext {
            pid: 0,
            start_ticks: 0,
            compat_data: var("LXMI_FIXTURE_COMPAT"),
            proton_dir: var("LXMI_FIXTURE_PROTON"),
            runtime_dir: var("LXMI_FIXTURE_RUNTIME"),
            steam_client_dir: None,
        };
        let mut helper = super::spawn_helper_in(&dir, &target, &launcher).unwrap();
        let (mut hooked, mut armed) = (None, false);
        let started = Instant::now();
        while !armed && started.elapsed() < Duration::from_secs(180) {
            match helper.read_line(Duration::from_millis(500)) {
                HelperRead::Line(line) => match HelperEvent::parse(&line) {
                    Some(HelperEvent::HookInstalled { code }) => hooked = Some(code),
                    Some(HelperEvent::Armed { .. }) => armed = true,
                    _ => {}
                },
                HelperRead::Timeout => {}
                HelperRead::Closed => break,
            }
        }
        let detail = helper.exit_detail();
        assert!(armed, "helper did not arm: {detail:?}");
        assert_eq!(hooked, Some(0));
        let marker = dir.join(super::HELPER_NAME);
        let marker = marker.file_name().unwrap().to_str().unwrap().to_string();
        let alive = || {
            std::fs::read_dir("/proc")
                .unwrap()
                .flatten()
                .filter_map(|e| std::fs::read(e.path().join("cmdline")).ok())
                .filter(|c| {
                    let c = String::from_utf8_lossy(c);
                    c.contains(&marker) && c.contains("hook-wait")
                })
                .count()
        };
        assert!(alive() > 0);
        helper.terminate();
        let stopped = Instant::now();
        while alive() > 0 && stopped.elapsed() < Duration::from_secs(20) {
            std::thread::sleep(Duration::from_millis(250));
        }
        assert_eq!(alive(), 0, "helper processes survived cancellation");
        helper.wait();
        eprintln!(
            "fixture: hook=0 armed=true cancelled_in_ms={}",
            stopped.elapsed().as_millis()
        );
    }

    fn target(
        mode: super::LoadMode,
        folder: &'static str,
        game: &std::path::Path,
    ) -> super::ModLaunchTarget {
        super::ModLaunchTarget {
            game_id: "test".into(),
            steam_app_id: 1,
            game_executable: game.to_path_buf(),
            launcher_executable: game.to_path_buf(),
            compat_data: game.to_path_buf(),
            steam_root: game.to_path_buf(),
            integration_folder: folder,
            launcher_label: "test",
            launch_arguments: &[],
            load_mode: mode,
        }
    }

    #[test]
    fn helper_arguments_keep_the_validated_zzz_hook_and_add_the_wwmi_inject_verb() {
        let dir = std::env::temp_dir().join(format!("lxmi-args-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let game = dir.join("Client Win64 game.exe");
        std::fs::write(&game, b"").unwrap();
        let entry = std::path::Path::new("/entry");
        let proton = std::path::Path::new("/proton");

        let zzz = super::helper_run_arguments(
            entry,
            proton,
            &dir,
            &target(super::LoadMode::Hook, "ZZMI", &game),
        )
        .unwrap();
        assert!(zzz.iter().any(|a| a == "hook-wait"));
        assert!(!zzz.iter().any(|a| a == "inject-wait"));
        assert_eq!(zzz.last().unwrap(), "ZZMI");

        let wwmi = super::helper_run_arguments(
            entry,
            proton,
            &dir,
            &target(super::LoadMode::DirectInject, "WWMI", &game),
        )
        .unwrap();
        assert!(wwmi.iter().any(|a| a == "inject-wait"));
        assert!(!wwmi.iter().any(|a| a == "hook-wait"));
        assert_eq!(wwmi.last().unwrap(), "WWMI");
        // Path with spaces stays one argument.
        let exe = wwmi
            .iter()
            .find(|a| a.to_string_lossy().contains("Client Win64"))
            .unwrap();
        assert!(exe.to_string_lossy().ends_with("Client Win64 game.exe"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn helper_arguments_refuse_unknown_importer_folders() {
        let game = std::env::temp_dir();
        let bad = target(super::LoadMode::Hook, "Otra", &game);
        assert!(super::helper_run_arguments(
            std::path::Path::new("/e"),
            std::path::Path::new("/p"),
            &game,
            &bad
        )
        .is_err());
    }

    #[test]
    fn explains_the_apparmor_container_failure() {
        let text = "pressure-vessel-wrap[1]: E: Child process exited with code 1: bwrap: setting up uid map: Permission denied\n";
        assert!(super::stderr_detail(text).unwrap().contains("AppArmor"));
        assert_eq!(
            super::stderr_detail("a\nlast line\n\n").as_deref(),
            Some("last line")
        );
        assert_eq!(super::stderr_detail(" \n"), None);
    }
}
