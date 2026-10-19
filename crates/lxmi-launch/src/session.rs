//! Launch state machine. Side effects live behind `LaunchHost` so every transition
//! is testable without Steam, Proton or a game.
use crate::{context::SteamProcessContext, events::HelperEvent};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

const LAUNCHER_WAIT: Duration = Duration::from_secs(180);
/// Helper: 300 s waiting for Play + 30 s for the module + 15 s health window.
const HELPER_WAIT: Duration = Duration::from_secs(420);
const MAX_LOG_LINES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchPhase {
    Preparing,
    OpeningLauncher,
    WaitingForLauncher,
    Arming,
    WaitingForPlay,
    CheckingStability,
    Running,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchStatus {
    pub game_id: String,
    pub profile_name: Option<String>,
    pub phase: LaunchPhase,
    pub message: String,
    pub finished: bool,
    pub mods_loaded: bool,
    pub launch_dir: Option<PathBuf>,
    pub helper_log: Vec<String>,
}

impl LaunchStatus {
    pub fn new(game_id: &str, profile_name: Option<String>) -> Self {
        Self {
            game_id: game_id.into(),
            profile_name,
            phase: LaunchPhase::Preparing,
            message: "Preparing the profile launch copy…".into(),
            finished: false,
            mods_loaded: false,
            launch_dir: None,
            helper_log: Vec::new(),
        }
    }
}

pub enum HelperRead {
    Line(String),
    Timeout,
    Closed,
}

pub trait HelperProcess: Send {
    fn read_line(&mut self, timeout: Duration) -> HelperRead;
    /// Terminates only the helper's own process group (its hook goes with it).
    fn terminate(&mut self);
    fn wait(&mut self) -> Option<i32>;
    /// Why the helper stopped without a result, from its own diagnostics.
    fn exit_detail(&mut self) -> Option<String> {
        None
    }
}

pub trait LaunchHost: Send {
    /// Name of the launcher the player presses Play in, for user-facing messages.
    fn launcher_label(&self) -> String {
        "the game launcher".into()
    }
    fn game_running(&self) -> Result<bool, String>;
    fn prepare(&mut self) -> Result<PathBuf, String>;
    fn find_launcher(&self) -> Result<Option<SteamProcessContext>, String>;
    fn request_steam_launch(&self) -> Result<(), String>;
    fn spawn_helper(
        &mut self,
        launcher: &SteamProcessContext,
    ) -> Result<Box<dyn HelperProcess>, String>;
    fn pause(&self, duration: Duration);
}

struct Shared<'a> {
    status: &'a Arc<Mutex<LaunchStatus>>,
}

impl Shared<'_> {
    fn set(&self, phase: LaunchPhase, message: impl Into<String>) {
        let mut s = self.status.lock().unwrap_or_else(|e| e.into_inner());
        s.phase = phase;
        s.message = message.into();
        s.finished = matches!(
            phase,
            LaunchPhase::Running | LaunchPhase::Failed | LaunchPhase::Cancelled
        );
    }
    fn update(&self, f: impl FnOnce(&mut LaunchStatus)) {
        f(&mut self.status.lock().unwrap_or_else(|e| e.into_inner()));
    }
}

fn failure_message(status: &str, launcher: &str) -> String {
    match status {
        "existing_game_rejected" => {
            "The game was already open before mod loading was armed. Close it and try again.".into()
        }
        "game_not_started" => {
            format!("Play was not pressed in {launcher} within 5 minutes. The loader stopped; nothing was changed.")
        }
        "ambiguous_target" => {
            "More than one game process is running. Close them and try again.".into()
        }
        "game_executable_not_visible" => {
            "Proton cannot see the game executable from the prefix.".into()
        }
        "loader_load_failed" | "hook_exports_missing" => {
            "Could not load 3dmloader.dll from XXMI Libraries.".into()
        }
        other => format!("The loader exited with '{other}'."),
    }
}

/// Documented `Inject` return codes (upstream `dll_injector.py`, v2.2.1).
fn inject_message(code: i64) -> String {
    let cause = match code {
        100 => "the game process was not found",
        110 => "the DLL path is invalid for the game",
        120 => "kernel32 could not be resolved inside the game",
        130 => "LoadLibraryW could not be resolved inside the game",
        200 => "memory could not be allocated in the game",
        300 => "memory could not be written in the game",
        400 => "the remote thread could not be created in the game",
        500 => "the remote thread timed out",
        510 => "waiting for the remote thread failed",
        600 => "injection failed",
        _ => "unknown error",
    };
    format!("WWMI injection did not complete (code {code}: {cause}). No mods are confirmed loaded.")
}

fn exit_message(hex: &str) -> String {
    match hex {
        "0xc0000135" => "The game exited while loading mods (0xc0000135): the importer detected a D3D11 device created before it loaded. Try again and press Play as soon as LXMI prompts you.".into(),
        _ => format!("The game exited within the first 15 seconds after mods loaded ({hex}). Review the profile mods or try 'Play without mods'."),
    }
}

/// Runs one launch to a terminal phase. `cancel` is honoured while waiting.
pub fn run_launch(
    host: &mut dyn LaunchHost,
    status: &Arc<Mutex<LaunchStatus>>,
    cancel: &AtomicBool,
) {
    let shared = Shared { status };
    let fail = |message: String| shared.set(LaunchPhase::Failed, message);
    let launcher_name = host.launcher_label();
    match host.game_running() {
        Ok(false) => {}
        Ok(true) => {
            return fail("The game is already running. Close it to start with mods.".into())
        }
        Err(e) => return fail(e),
    }
    match host.prepare() {
        Ok(dir) => shared.update(|s| s.launch_dir = Some(dir)),
        Err(e) => return fail(e),
    }
    let launcher = match host.find_launcher() {
        Err(e) => return fail(e),
        Ok(Some(found)) => found,
        Ok(None) => {
            shared.set(LaunchPhase::OpeningLauncher, "Opening the game in Steam…");
            if let Err(e) = host.request_steam_launch() {
                return fail(e);
            }
            shared.set(
                LaunchPhase::WaitingForLauncher,
                "Waiting for Steam to open the launcher. Do not press Play yet.",
            );
            let started = Instant::now();
            loop {
                if cancel.load(Ordering::SeqCst) {
                    return shared.set(
                        LaunchPhase::Cancelled,
                        "Cancelled before anything was loaded.",
                    );
                }
                match host.find_launcher() {
                    Ok(Some(found)) => break found,
                    Ok(None) if started.elapsed() < LAUNCHER_WAIT => {
                        host.pause(Duration::from_secs(1))
                    }
                    Ok(None) => {
                        return fail("Steam did not open the launcher within 3 minutes.".into())
                    }
                    Err(e) => return fail(e),
                }
            }
        }
    };
    match host.game_running() {
        Ok(false) => {}
        Ok(true) => {
            return fail(
                "The game opened before mod loading was armed. Close it and try again.".into(),
            )
        }
        Err(e) => return fail(e),
    }
    if cancel.load(Ordering::SeqCst) {
        return shared.set(
            LaunchPhase::Cancelled,
            "Cancelled before anything was loaded.",
        );
    }
    shared.set(
        LaunchPhase::Arming,
        "Preparing mod loading in the Steam context…",
    );
    let mut helper = match host.spawn_helper(&launcher) {
        Ok(helper) => helper,
        Err(e) => return fail(e),
    };
    let started = Instant::now();
    let mut terminal = false;
    while !terminal {
        if cancel.load(Ordering::SeqCst) {
            helper.terminate();
            shared.set(
                LaunchPhase::Cancelled,
                "Cancelled. The mod loader was stopped.",
            );
            break;
        }
        if started.elapsed() > HELPER_WAIT {
            helper.terminate();
            fail("The loader timed out and was stopped.".into());
            break;
        }
        let line = match helper.read_line(Duration::from_millis(500)) {
            HelperRead::Line(line) => line,
            HelperRead::Timeout => continue,
            HelperRead::Closed => {
                let _ = helper.wait();
                fail(match helper.exit_detail() {
                    Some(detail) => {
                        format!("The loader exited without reporting a result: {detail}")
                    }
                    None => "The loader exited without reporting a result.".into(),
                });
                break;
            }
        };
        shared.update(|s| {
            if s.helper_log.len() < MAX_LOG_LINES {
                s.helper_log.push(line.clone());
            }
        });
        let Some(event) = HelperEvent::parse(&line) else {
            continue;
        };
        match event {
            HelperEvent::HookInstalled { code: 0 } | HelperEvent::Unhooked { .. } => {}
            HelperEvent::InjectReturned { code: 0 } => {}
            HelperEvent::InjectReturned { code } => {
                fail(inject_message(code));
                terminal = true;
            }
            HelperEvent::HookInstalled { code } => {
                fail(format!(
                    "Could not install the XXMI hook (HookLibrary {code})."
                ));
                terminal = true;
            }
            HelperEvent::Armed { .. } => shared.set(
                LaunchPhase::WaitingForPlay,
                format!("Ready: press Play in {launcher_name} now (you have 5 minutes)."),
            ),
            HelperEvent::GameObserved {
                mods_module_present: true,
                ..
            } => {
                shared.update(|s| s.mods_loaded = true);
                shared.set(
                    LaunchPhase::CheckingStability,
                    "Mod module loaded. Checking that the game stays open…",
                );
            }
            HelperEvent::GameObserved {
                mods_module_present: false,
                ..
            } => {
                fail("The game started, but mods did not load within 30 seconds. Close it and try again.".into());
                terminal = true;
            }
            HelperEvent::GameAlive { .. } => {
                shared.set(
                    LaunchPhase::Running,
                    "The game is running. The mod module is loaded; LXMI does not verify the effect of each mod.",
                );
                terminal = true;
            }
            HelperEvent::GameExited { exit_code_hex } => {
                shared.update(|s| s.mods_loaded = false);
                fail(exit_message(&exit_code_hex));
                terminal = true;
            }
            HelperEvent::Failure { status, .. } => {
                fail(failure_message(&status, &launcher_name));
                terminal = true;
            }
        }
    }
    helper.wait();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex as StdMutex};

    struct FakeHelper {
        lines: VecDeque<HelperRead>,
        terminated: Arc<StdMutex<bool>>,
    }
    impl HelperProcess for FakeHelper {
        fn read_line(&mut self, _: Duration) -> HelperRead {
            self.lines.pop_front().unwrap_or(HelperRead::Closed)
        }
        fn terminate(&mut self) {
            *self.terminated.lock().unwrap() = true;
        }
        fn wait(&mut self) -> Option<i32> {
            Some(0)
        }
    }

    #[derive(Default)]
    struct FakeHost {
        game_running: Vec<bool>,
        launcher_after: usize,
        lookups: StdMutex<usize>,
        steam_requests: StdMutex<usize>,
        lines: Vec<&'static str>,
        spawned: bool,
        terminated: Arc<StdMutex<bool>>,
    }
    fn context() -> SteamProcessContext {
        SteamProcessContext {
            pid: 1,
            start_ticks: 1,
            compat_data: "/c".into(),
            proton_dir: "/p".into(),
            runtime_dir: "/r".into(),
            steam_client_dir: None,
        }
    }
    impl LaunchHost for FakeHost {
        fn game_running(&self) -> Result<bool, String> {
            Ok(!self.game_running.is_empty() && self.game_running[0])
        }
        fn prepare(&mut self) -> Result<PathBuf, String> {
            Ok("/launch".into())
        }
        fn find_launcher(&self) -> Result<Option<SteamProcessContext>, String> {
            let mut n = self.lookups.lock().unwrap();
            *n += 1;
            Ok((*n > self.launcher_after).then(context))
        }
        fn request_steam_launch(&self) -> Result<(), String> {
            *self.steam_requests.lock().unwrap() += 1;
            Ok(())
        }
        fn spawn_helper(
            &mut self,
            _: &SteamProcessContext,
        ) -> Result<Box<dyn HelperProcess>, String> {
            self.spawned = true;
            Ok(Box::new(FakeHelper {
                lines: self
                    .lines
                    .iter()
                    .map(|l| HelperRead::Line(l.to_string()))
                    .collect(),
                terminated: self.terminated.clone(),
            }))
        }
        fn pause(&self, _: Duration) {}
    }

    fn run(host: &mut FakeHost, cancel: bool) -> LaunchStatus {
        let status = Arc::new(Mutex::new(LaunchStatus::new("zenless-zone-zero", None)));
        run_launch(host, &status, &AtomicBool::new(cancel));
        let s = status.lock().unwrap().clone();
        s
    }

    const SUCCESS: [&str; 6] = [
        r#"{"status":"hook_library_returned","code":0}"#,
        r#"{"status":"armed_hook_waiting_for_new_game","timeout_seconds":300}"#,
        r#"{"status":"unhook_library_returned","code":0}"#,
        r#"{"status":"game_observed","windows_pid":1868,"mods_module_present":1,"process_age_ms":3210}"#,
        "noise from Proton",
        r#"{"status":"game_alive_after_load","observation_ms":15000}"#,
    ];

    #[test]
    fn validated_sequence_ends_running_with_mods() {
        let mut host = FakeHost {
            lines: SUCCESS.to_vec(),
            ..Default::default()
        };
        let s = run(&mut host, false);
        assert_eq!(s.phase, LaunchPhase::Running);
        assert!(s.mods_loaded && s.finished);
        assert_eq!(s.launch_dir, Some("/launch".into()));
        assert_eq!(
            *host.steam_requests.lock().unwrap(),
            0,
            "launcher already open"
        );
        assert_eq!(s.helper_log.len(), 6);
    }

    #[test]
    fn opens_the_launcher_through_steam_when_absent() {
        let mut host = FakeHost {
            launcher_after: 3,
            lines: SUCCESS.to_vec(),
            ..Default::default()
        };
        assert_eq!(run(&mut host, false).phase, LaunchPhase::Running);
        assert_eq!(*host.steam_requests.lock().unwrap(), 1);
    }

    #[test]
    fn refuses_when_the_game_is_already_open() {
        let mut host = FakeHost {
            game_running: vec![true],
            ..Default::default()
        };
        let s = run(&mut host, false);
        assert_eq!(s.phase, LaunchPhase::Failed);
        assert!(!host.spawned && s.launch_dir.is_none());
    }

    #[test]
    fn game_exit_after_load_is_a_failure_with_its_code() {
        let mut lines = SUCCESS[..4].to_vec();
        lines.push(r#"{"status":"game_exited_after_load","exit_code":3221225781,"exit_code_hex":"0xc0000135"}"#);
        let mut host = FakeHost {
            lines,
            ..Default::default()
        };
        let s = run(&mut host, false);
        assert_eq!(s.phase, LaunchPhase::Failed);
        assert!(!s.mods_loaded && s.message.contains("0xc0000135"));
    }

    #[test]
    fn helper_failures_and_silent_exit_are_reported() {
        let mut host = FakeHost {
            lines: vec![
                SUCCESS[0],
                SUCCESS[1],
                SUCCESS[2],
                r#"{"status":"game_not_started","windows_error":0}"#,
            ],
            ..Default::default()
        };
        assert!(run(&mut host, false).message.contains("5 minutes"));
        let mut host = FakeHost {
            lines: vec![r#"{"status":"hook_library_returned","code":300}"#],
            ..Default::default()
        };
        assert!(run(&mut host, false).message.contains("HookLibrary 300"));
        let mut host = FakeHost {
            lines: vec![SUCCESS[0]],
            ..Default::default()
        };
        assert!(run(&mut host, false).message.contains("without reporting"));
    }

    #[test]
    fn direct_inject_failures_name_the_documented_code_and_never_confirm_mods() {
        let armed_inject =
            r#"{"status":"armed_inject_waiting_for_new_game","timeout_seconds":300}"#;
        let mut host = FakeHost {
            lines: vec![armed_inject, r#"{"status":"inject_returned","code":600}"#],
            ..Default::default()
        };
        let s = run(&mut host, false);
        assert_eq!(s.phase, LaunchPhase::Failed);
        assert!(s.message.contains("code 600"));
        assert!(!s.mods_loaded, "a failed injection never confirms mods");

        let mut host = FakeHost {
            lines: vec![
                armed_inject,
                r#"{"status":"inject_returned","code":0}"#,
                r#"{"status":"game_observed","windows_pid":7,"mods_module_present":1,"process_age_ms":900}"#,
                r#"{"status":"game_alive_after_load","observation_ms":15000}"#,
            ],
            ..Default::default()
        };
        let s = run(&mut host, false);
        assert_eq!(s.phase, LaunchPhase::Running);
        assert!(
            s.mods_loaded,
            "module presence is recorded after a successful Inject"
        );
    }

    #[test]
    fn cancellation_before_arming_spawns_nothing() {
        let mut host = FakeHost {
            lines: SUCCESS.to_vec(),
            ..Default::default()
        };
        let s = run(&mut host, true);
        assert_eq!(s.phase, LaunchPhase::Cancelled);
        assert!(!host.spawned, "cancelled before arming");
    }
}
