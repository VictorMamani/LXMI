//! Short-lived passive observer for the current user's ZZZ process context.
//! It never starts, signals, opens, or controls observed processes.

use lxmi_core::{SystemInfo, ZENLESS_ZONE_ZERO_APP_ID};
use lxmi_proton::{CompatibilityToolKind, CompatibilityToolStatus, ProtonScanner};
use lxmi_runtime::{
    ConfiguredRuntimeEvidence, ObservationEvidenceState, ObservationTarget, ObservedRuntimeTool,
    ProcessTopologyState, ProcfsObserver, ProcfsScanDiagnostics, RuntimeCandidate,
    RuntimeObservationAppId, RuntimeObservationPath, RuntimeObservationSelection,
    RuntimeObservationSnapshot, RuntimeObservationStatus, WineServerObservation,
};
use lxmi_steam::{
    inspect_steam_compat_tool_config, SteamCompatToolConfigStatus, SteamCompatToolMapping,
    SteamDiscoveryResult, SteamDiscoveryScanner, SteamGameInstallation,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const OBSERVATION_WINDOW: Duration = Duration::from_secs(90);
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const STOP_CHECK_INTERVAL: Duration = Duration::from_millis(100);
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeObservationError {
    pub code: String,
    pub detail: String,
}

struct ObservationSession {
    stop_requested: AtomicBool,
    active: AtomicBool,
    snapshot: Mutex<RuntimeObservationSnapshot>,
    target: ObservationTarget,
    candidates: Vec<RuntimeCandidate>,
    observer: Arc<dyn ObservationScanner>,
}

trait ObservationScanner: Send + Sync {
    fn scan(
        &self,
        target: &ObservationTarget,
        installed_tools: &[RuntimeCandidate],
    ) -> lxmi_runtime::ProcfsScan;
}

impl ObservationScanner for ProcfsObserver {
    fn scan(
        &self,
        target: &ObservationTarget,
        installed_tools: &[RuntimeCandidate],
    ) -> lxmi_runtime::ProcfsScan {
        ProcfsObserver::scan(self, target, installed_tools)
    }
}

#[derive(Clone, Default)]
pub struct RuntimeObservationController {
    session: Arc<Mutex<Option<Arc<ObservationSession>>>>,
}

impl RuntimeObservationController {
    pub fn start_zzz(
        &self,
        user_confirmed_manual_launch: bool,
    ) -> Result<RuntimeObservationSnapshot, RuntimeObservationError> {
        if !user_confirmed_manual_launch {
            return Err(observation_error(
                "consent_required",
                "Confirma que iniciarás Zenless Zone Zero manualmente desde Steam mientras LXMI observa.",
            ));
        }
        if let Some(active) = self.active_session() {
            return Ok(redacted_snapshot(&active));
        }

        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let game = discovery
            .games
            .games
            .iter()
            .find(|game| game.steam_app_id == ZENLESS_ZONE_ZERO_APP_ID)
            .ok_or_else(|| {
                observation_error(
                    "zzz_not_detected",
                    "El escaneo read-only actual no encontró el manifest Steam de Zenless Zone Zero (AppID 4162040).",
                )
            })?;
        let system = SystemInfo::current();
        let tools = ProtonScanner.scan(&discovery.steam.installations);
        let candidates = tools
            .tools
            .iter()
            .filter(|tool| {
                tool.kind == CompatibilityToolKind::Proton
                    && tool.status == CompatibilityToolStatus::Valid
            })
            .map(RuntimeCandidate::from)
            .collect::<Vec<_>>();
        let target = observation_target(game, system.home_directory.clone());
        let configuration = configured_runtime_evidence(&discovery, game.steam_library.as_path());
        let observer = ProcfsObserver::for_current_user().map_err(|error| {
            observation_error(
                "procfs_unavailable",
                &format!(
                    "No se pudo preparar la vista read-only del usuario actual: {}",
                    safe_io_kind(&error)
                ),
            )
        })?;
        let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let started = now_millis();
        let snapshot = initial_snapshot(
            session_id,
            ZENLESS_ZONE_ZERO_APP_ID,
            started,
            configuration,
            &candidates,
            &target,
        );
        let session = Arc::new(ObservationSession {
            stop_requested: AtomicBool::new(false),
            active: AtomicBool::new(true),
            snapshot: Mutex::new(snapshot),
            target,
            candidates,
            observer: Arc::new(observer),
        });
        {
            let mut current = self
                .session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(active) = current
                .as_ref()
                .filter(|current_session| current_session.active.load(Ordering::Acquire))
            {
                return Ok(redacted_snapshot(active));
            }
            *current = Some(Arc::clone(&session));
        }
        tracing::info!(
            app_id = ZENLESS_ZONE_ZERO_APP_ID,
            "Passive runtime observation started"
        );
        thread::Builder::new()
            .name("lxmi-zzz-observer".into())
            .spawn({
                let session = Arc::clone(&session);
                move || observe_session(session)
            })
            .map_err(|error| {
                session.active.store(false, Ordering::Release);
                if let Ok(mut snapshot) = session.snapshot.lock() {
                    snapshot.active = false;
                    snapshot.status = RuntimeObservationStatus::BackendError;
                    snapshot.updated_at_unix_millis = now_millis();
                }
                observation_error(
                    "observer_thread_failed",
                    &format!(
                        "No se pudo iniciar la observación: {}",
                        safe_io_kind(&error)
                    ),
                )
            })?;
        Ok(redacted_snapshot(&session))
    }

    pub fn current(&self) -> Option<RuntimeObservationSnapshot> {
        self.active_or_last_session()
            .map(|session| redacted_snapshot(&session))
    }

    pub fn stop(&self) -> Option<RuntimeObservationSnapshot> {
        let session = self.active_or_last_session()?;
        if session.active.load(Ordering::Acquire) {
            session.stop_requested.store(true, Ordering::Release);
            tracing::info!(
                app_id = ZENLESS_ZONE_ZERO_APP_ID,
                "Passive runtime observation stop requested"
            );
        }
        Some(redacted_snapshot(&session))
    }

    fn active_session(&self) -> Option<Arc<ObservationSession>> {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .filter(|session| session.active.load(Ordering::Acquire))
            .cloned()
    }

    fn active_or_last_session(&self) -> Option<Arc<ObservationSession>> {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

fn observe_session(session: Arc<ObservationSession>) {
    let started = std::time::Instant::now();
    let mut seen_game = false;
    let mut first_scan = true;
    let mut last_logged_scan: Option<(RuntimeObservationStatus, ProcfsScanDiagnostics)> = None;
    loop {
        if session.stop_requested.load(Ordering::Acquire) {
            let status = {
                let snapshot = session
                    .snapshot
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                status_after_user_stop(snapshot.status)
            };
            finish_session(&session, status, started.elapsed());
            break;
        }
        if started.elapsed() >= OBSERVATION_WINDOW {
            finish_session(
                &session,
                RuntimeObservationStatus::TimedOut,
                started.elapsed(),
            );
            break;
        }

        let scan = session.observer.scan(&session.target, &session.candidates);
        let should_mark_seen = scan.matches.len() == 1;
        if should_mark_seen {
            seen_game = true;
        }
        let diagnostic_snapshot = scan.diagnostics.clone();
        update_from_scan(&session, scan, seen_game);
        let (status, diagnostics) = {
            let snapshot = session
                .snapshot
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            (snapshot.status, snapshot.scan_diagnostics.clone())
        };
        if first_scan || last_logged_scan.as_ref() != Some(&(status, diagnostic_snapshot.clone())) {
            if first_scan || diagnostics.accepted_candidates > 0 {
                tracing::info!(
                    app_id = ZENLESS_ZONE_ZERO_APP_ID,
                    status = ?status,
                    same_uid_processes_scanned = diagnostics.same_uid_processes_scanned,
                    candidate_processes_considered = diagnostics.candidate_processes_considered,
                    candidate_metadata_unavailable = diagnostics.candidate_metadata_unavailable,
                    candidate_environments_unavailable = diagnostics.candidate_environments_unavailable,
                    candidate_environments_permission_denied = diagnostics.candidate_environments_permission_denied,
                    accepted_contexts_environment_unavailable = diagnostics.accepted_contexts_environment_unavailable,
                    accepted_contexts_environment_permission_denied = diagnostics.accepted_contexts_environment_permission_denied,
                    rejected_executable_mismatch = diagnostics.rejected_executable_mismatch,
                    rejected_insufficient_identity_evidence = diagnostics.rejected_insufficient_identity_evidence,
                    accepted_candidates = diagnostics.accepted_candidates,
                    "Passive runtime scan outcome"
                );
            } else {
                tracing::debug!(
                    app_id = ZENLESS_ZONE_ZERO_APP_ID,
                    status = ?status,
                    same_uid_processes_scanned = diagnostics.same_uid_processes_scanned,
                    candidate_processes_considered = diagnostics.candidate_processes_considered,
                    candidate_metadata_unavailable = diagnostics.candidate_metadata_unavailable,
                    candidate_environments_unavailable = diagnostics.candidate_environments_unavailable,
                    candidate_environments_permission_denied = diagnostics.candidate_environments_permission_denied,
                    accepted_contexts_environment_unavailable = diagnostics.accepted_contexts_environment_unavailable,
                    accepted_contexts_environment_permission_denied = diagnostics.accepted_contexts_environment_permission_denied,
                    rejected_executable_mismatch = diagnostics.rejected_executable_mismatch,
                    rejected_insufficient_identity_evidence = diagnostics.rejected_insufficient_identity_evidence,
                    accepted_candidates = diagnostics.accepted_candidates,
                    "Passive runtime scan outcome"
                );
            }
            last_logged_scan = Some((status, diagnostic_snapshot));
        }
        first_scan = false;
        if seen_game && status == RuntimeObservationStatus::ProcessExited {
            finish_session(
                &session,
                RuntimeObservationStatus::ProcessExited,
                started.elapsed(),
            );
            break;
        }

        let mut slept = Duration::ZERO;
        while slept < POLL_INTERVAL {
            if session.stop_requested.load(Ordering::Acquire) {
                break;
            }
            let interval = STOP_CHECK_INTERVAL.min(POLL_INTERVAL - slept);
            thread::sleep(interval);
            slept += interval;
        }
    }
}

fn status_after_user_stop(current: RuntimeObservationStatus) -> RuntimeObservationStatus {
    match current {
        RuntimeObservationStatus::WaitingForGame => RuntimeObservationStatus::ProcessNotFound,
        RuntimeObservationStatus::Observed => RuntimeObservationStatus::Completed,
        current => current,
    }
}

fn update_from_scan(session: &ObservationSession, scan: lxmi_runtime::ProcfsScan, seen_game: bool) {
    let mut snapshot = session
        .snapshot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    snapshot.updated_at_unix_millis = now_millis();
    snapshot.elapsed_millis = snapshot
        .updated_at_unix_millis
        .saturating_sub(snapshot.started_at_unix_millis);
    snapshot.active = session.active.load(Ordering::Acquire);
    snapshot.environment_evidence = scan.environment_evidence;
    snapshot.scan_diagnostics = scan.diagnostics;
    snapshot.warnings = scan.warnings;
    match scan.matches.len() {
        0 if seen_game => snapshot.status = RuntimeObservationStatus::ProcessExited,
        0 if snapshot.status == RuntimeObservationStatus::AmbiguousProcess => {}
        0 => snapshot.status = RuntimeObservationStatus::WaitingForGame,
        1 => {
            let found = scan.matches.into_iter().next().expect("one match");
            snapshot.status = RuntimeObservationStatus::Observed;
            snapshot.process = Some(found.process);
            snapshot.ambiguous_process_count = 0;
            snapshot.app_id_evidence = found.app_id;
            snapshot.observed_runtime = found.runtime;
            snapshot.compatdata = found.compatdata;
            snapshot.prefix = found.prefix;
            snapshot.process_topology = found.topology;
            snapshot.process_topology_state = found.topology_state;
            snapshot.wineserver = found.wineserver;
            snapshot.environment_evidence = found.environment_evidence;
        }
        count => {
            snapshot.status = RuntimeObservationStatus::AmbiguousProcess;
            snapshot.process = None;
            snapshot.ambiguous_process_count = count;
            snapshot.app_id_evidence = RuntimeObservationAppId {
                state: ObservationEvidenceState::Ambiguous,
                observed_values: Vec::new(),
                sources: vec!["más de un proceso coincide con la identidad del juego".into()],
            };
            snapshot.observed_runtime = RuntimeObservationSelection {
                state: ObservationEvidenceState::Ambiguous,
                tool: None,
                evidence_source: Some("no se seleccionó arbitrariamente un proceso".into()),
            };
            snapshot.compatdata.state = ObservationEvidenceState::Ambiguous;
            snapshot.compatdata.observed_path = None;
            snapshot.prefix.state = ObservationEvidenceState::Ambiguous;
            snapshot.prefix.observed_path = None;
            snapshot.process_topology.clear();
            snapshot.process_topology_state = ProcessTopologyState::Ambiguous;
            snapshot.wineserver = WineServerObservation {
                state: ObservationEvidenceState::Ambiguous,
                related_process_count: 0,
                evidence: "Hay varios procesos candidatos; no se eligió un contexto.".into(),
            };
        }
    }
}

fn finish_session(
    session: &ObservationSession,
    status: RuntimeObservationStatus,
    elapsed: Duration,
) {
    session.active.store(false, Ordering::Release);
    let mut snapshot = session
        .snapshot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    snapshot.status = status;
    snapshot.active = false;
    snapshot.elapsed_millis = elapsed.as_millis().min(u64::MAX as u128) as u64;
    snapshot.updated_at_unix_millis = now_millis();
    tracing::info!(
        app_id = snapshot.app_id,
        status = ?snapshot.status,
        accepted_candidates = snapshot.scan_diagnostics.accepted_candidates,
        "Passive runtime observation completed"
    );
}

fn initial_snapshot(
    session_id: u64,
    app_id: u32,
    timestamp: u64,
    configured_runtime: ConfiguredRuntimeEvidence,
    candidates: &[RuntimeCandidate],
    target: &ObservationTarget,
) -> RuntimeObservationSnapshot {
    let available_proton_tools = candidates
        .iter()
        .map(|tool| ObservedRuntimeTool {
            internal_id: tool.internal_id.clone(),
            display_name: tool.display_name.clone(),
            version: tool.version.clone(),
            path: tool.path.to_string_lossy().into_owned(),
        })
        .collect();
    RuntimeObservationSnapshot {
        session_id,
        status: RuntimeObservationStatus::WaitingForGame,
        active: true,
        app_id,
        started_at_unix_millis: timestamp,
        updated_at_unix_millis: timestamp,
        elapsed_millis: 0,
        scan_diagnostics: ProcfsScanDiagnostics::default(),
        process: None,
        ambiguous_process_count: 0,
        app_id_evidence: RuntimeObservationAppId {
            state: ObservationEvidenceState::Unavailable,
            observed_values: Vec::new(),
            sources: Vec::new(),
        },
        configured_runtime,
        available_proton_tools,
        observed_runtime: RuntimeObservationSelection {
            state: ObservationEvidenceState::Unknown,
            tool: None,
            evidence_source: None,
        },
        compatdata: RuntimeObservationPath {
            state: ObservationEvidenceState::NotObserved,
            observed_path: None,
            expected_path: target
                .expected_compatdata_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            evidence_source: None,
        },
        prefix: RuntimeObservationPath {
            state: ObservationEvidenceState::Unknown,
            observed_path: None,
            expected_path: target
                .expected_compatdata_path
                .as_ref()
                .map(|path| path.join("pfx").to_string_lossy().into_owned()),
            evidence_source: None,
        },
        process_topology: Vec::new(),
        process_topology_state: ProcessTopologyState::NotObserved,
        wineserver: WineServerObservation {
            state: ObservationEvidenceState::NotObserved,
            related_process_count: 0,
            evidence: "La observación aún espera el proceso del juego.".into(),
        },
        environment_evidence: ObservationEvidenceState::Unknown,
        warnings: Vec::new(),
    }
}

fn observation_target(game: &SteamGameInstallation, home: Option<PathBuf>) -> ObservationTarget {
    ObservationTarget {
        app_id: game.steam_app_id,
        expected_executable_names: game
            .installation
            .game
            .expected_executable_names
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        game_install_path: Some(game.installation.install_path.clone()),
        expected_executable_path: game.installation.executable_path.clone(),
        expected_compatdata_path: Some(game.compatdata.compatdata_path.clone()),
        home_directory: home.or_else(|| SystemInfo::current().home_directory),
    }
}

fn configured_runtime_evidence(
    discovery: &SteamDiscoveryResult,
    library_root: &Path,
) -> ConfiguredRuntimeEvidence {
    let matching_roots = discovery
        .steam
        .installations
        .iter()
        .filter(|installation| {
            installation
                .libraries
                .iter()
                .any(|library| library.path == library_root)
        })
        .collect::<Vec<_>>();
    let inspection = (matching_roots.len() == 1).then(|| {
        inspect_steam_compat_tool_config(&matching_roots[0].root_path, ZENLESS_ZONE_ZERO_APP_ID)
    });
    let per_game = inspection
        .as_ref()
        .and_then(|result| result.mappings.as_ref())
        .and_then(|mappings| mappings.per_app.as_ref());
    let global = inspection
        .as_ref()
        .and_then(|result| result.mappings.as_ref())
        .and_then(|mappings| mappings.global_default.as_ref());
    let config_path = inspection
        .as_ref()
        .map(|result| result.config_path.to_string_lossy().into_owned());
    ConfiguredRuntimeEvidence {
        per_game_state: mapping_state(inspection.as_ref().map(|result| result.status), per_game),
        per_game_internal_id: per_game.and_then(mapping_internal_name),
        global_state: mapping_state(inspection.as_ref().map(|result| result.status), global),
        global_internal_id: global.and_then(mapping_internal_name),
        source: config_path,
    }
}

fn mapping_internal_name(mapping: &SteamCompatToolMapping) -> Option<String> {
    mapping
        .internal_name
        .as_deref()
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(str::to_owned)
}

fn mapping_state(
    status: Option<SteamCompatToolConfigStatus>,
    mapping: Option<&SteamCompatToolMapping>,
) -> String {
    if mapping.is_some() {
        "configured".into()
    } else if status == Some(SteamCompatToolConfigStatus::Read) {
        "no_mapping_in_inspected_source".into()
    } else {
        "unknown".into()
    }
}

fn redacted_snapshot(session: &ObservationSession) -> RuntimeObservationSnapshot {
    let mut snapshot = session
        .snapshot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    snapshot.redact_home_paths(session.target.home_directory.as_deref());
    snapshot
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn safe_io_kind(error: &std::io::Error) -> &'static str {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => "permiso denegado",
        std::io::ErrorKind::NotFound => "entrada ausente",
        _ => "error de lectura",
    }
}

fn observation_error(code: &str, detail: &str) -> RuntimeObservationError {
    RuntimeObservationError {
        code: code.into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyScanner;

    impl ObservationScanner for EmptyScanner {
        fn scan(
            &self,
            _target: &ObservationTarget,
            _installed_tools: &[RuntimeCandidate],
        ) -> lxmi_runtime::ProcfsScan {
            lxmi_runtime::ProcfsScan {
                matches: Vec::new(),
                warnings: Vec::new(),
                environment_evidence: ObservationEvidenceState::NotObserved,
                diagnostics: ProcfsScanDiagnostics::default(),
            }
        }
    }

    #[test]
    fn configured_runtime_and_available_candidates_do_not_fill_observed_runtime() {
        let candidate = RuntimeCandidate {
            internal_id: Some("proton-experimental".into()),
            display_name: "Proton Experimental".into(),
            path: PathBuf::from("/steam/Proton Experimental"),
            source: lxmi_proton::CompatibilityToolSource::SteamLibrary,
            version: None,
        };
        let target = ObservationTarget {
            app_id: ZENLESS_ZONE_ZERO_APP_ID,
            expected_executable_names: vec!["ZenlessZoneZero.exe".into()],
            game_install_path: Some(PathBuf::from("/games/zzz")),
            expected_executable_path: None,
            expected_compatdata_path: Some(PathBuf::from("/steam/compatdata/4162040")),
            home_directory: None,
        };
        let snapshot = initial_snapshot(
            1,
            ZENLESS_ZONE_ZERO_APP_ID,
            10,
            ConfiguredRuntimeEvidence {
                per_game_state: "no_mapping_in_inspected_source".into(),
                per_game_internal_id: None,
                global_state: "unknown".into(),
                global_internal_id: None,
                source: None,
            },
            &[candidate],
            &target,
        );
        assert_eq!(snapshot.available_proton_tools.len(), 1);
        assert_eq!(
            snapshot.observed_runtime.state,
            ObservationEvidenceState::Unknown
        );
        assert_eq!(snapshot.observed_runtime.tool, None);
        assert_eq!(
            snapshot.configured_runtime.per_game_state,
            "no_mapping_in_inspected_source"
        );
    }

    #[test]
    fn home_paths_are_redacted_before_crossing_the_ui_boundary() {
        let target = ObservationTarget {
            app_id: ZENLESS_ZONE_ZERO_APP_ID,
            expected_executable_names: vec![],
            game_install_path: None,
            expected_executable_path: None,
            expected_compatdata_path: Some(PathBuf::from("/home/tester/.steam/compatdata/4162040")),
            home_directory: Some(PathBuf::from("/home/tester")),
        };
        let mut snapshot = initial_snapshot(
            1,
            ZENLESS_ZONE_ZERO_APP_ID,
            10,
            ConfiguredRuntimeEvidence {
                per_game_state: "unknown".into(),
                per_game_internal_id: None,
                global_state: "unknown".into(),
                global_internal_id: None,
                source: Some("/home/tester/.steam/config/config.vdf".into()),
            },
            &[],
            &target,
        );
        snapshot.redact_home_paths(target.home_directory.as_deref());
        assert_eq!(
            snapshot.compatdata.expected_path.as_deref(),
            Some("~/.steam/compatdata/4162040")
        );
        assert_eq!(
            snapshot.configured_runtime.source.as_deref(),
            Some("~/.steam/config/config.vdf")
        );
    }

    fn test_target() -> ObservationTarget {
        ObservationTarget {
            app_id: ZENLESS_ZONE_ZERO_APP_ID,
            expected_executable_names: vec!["ZenlessZoneZero.exe".into()],
            game_install_path: Some(PathBuf::from("/fixture/zzz")),
            expected_executable_path: Some(PathBuf::from("/fixture/zzz/ZenlessZoneZero.exe")),
            expected_compatdata_path: Some(PathBuf::from("/fixture/compatdata/4162040")),
            home_directory: None,
        }
    }

    fn empty_snapshot(target: &ObservationTarget) -> RuntimeObservationSnapshot {
        initial_snapshot(
            1,
            ZENLESS_ZONE_ZERO_APP_ID,
            100,
            ConfiguredRuntimeEvidence {
                per_game_state: "unknown".into(),
                per_game_internal_id: None,
                global_state: "unknown".into(),
                global_internal_id: None,
                source: None,
            },
            &[],
            target,
        )
    }

    fn observed_process(pid: u32) -> lxmi_runtime::ObservedRuntimeProcess {
        lxmi_runtime::ObservedRuntimeProcess {
            process: lxmi_runtime::ObservedGameProcess {
                pid,
                parent_pid: Some(10),
                executable_name: "ZenlessZoneZero.exe".into(),
                identity_evidence: vec!["expected_executable_name".into()],
            },
            app_id: RuntimeObservationAppId {
                state: ObservationEvidenceState::Verified,
                observed_values: vec![ZENLESS_ZONE_ZERO_APP_ID],
                sources: vec!["STEAM_COMPAT_APP_ID".into()],
            },
            runtime: RuntimeObservationSelection {
                state: ObservationEvidenceState::Unknown,
                tool: None,
                evidence_source: None,
            },
            compatdata: RuntimeObservationPath {
                state: ObservationEvidenceState::ObservedAndMatches,
                observed_path: Some("/fixture/compatdata/4162040".into()),
                expected_path: Some("/fixture/compatdata/4162040".into()),
                evidence_source: Some("STEAM_COMPAT_DATA_PATH".into()),
            },
            prefix: RuntimeObservationPath {
                state: ObservationEvidenceState::DerivedFromObservedCompatdata,
                observed_path: Some("/fixture/compatdata/4162040/pfx".into()),
                expected_path: None,
                evidence_source: Some("STEAM_COMPAT_DATA_PATH".into()),
            },
            topology: Vec::new(),
            topology_state: ProcessTopologyState::PartialTopology,
            wineserver: WineServerObservation {
                state: ObservationEvidenceState::NotObserved,
                related_process_count: 0,
                evidence: "none".into(),
            },
            environment_evidence: ObservationEvidenceState::Observed,
        }
    }

    #[test]
    fn process_disappearance_keeps_observed_evidence_without_control_actions() {
        let target = test_target();
        let session = ObservationSession {
            stop_requested: AtomicBool::new(false),
            active: AtomicBool::new(true),
            snapshot: Mutex::new(empty_snapshot(&target)),
            target,
            candidates: Vec::new(),
            observer: Arc::new(EmptyScanner),
        };
        update_from_scan(
            &session,
            lxmi_runtime::ProcfsScan {
                matches: vec![observed_process(42)],
                warnings: Vec::new(),
                environment_evidence: ObservationEvidenceState::Observed,
                diagnostics: ProcfsScanDiagnostics {
                    accepted_candidates: 1,
                    ..ProcfsScanDiagnostics::default()
                },
            },
            true,
        );
        assert_eq!(
            session.snapshot.lock().unwrap().status,
            RuntimeObservationStatus::Observed
        );

        update_from_scan(
            &session,
            lxmi_runtime::ProcfsScan {
                matches: Vec::new(),
                warnings: Vec::new(),
                environment_evidence: ObservationEvidenceState::NotObserved,
                diagnostics: ProcfsScanDiagnostics::default(),
            },
            true,
        );
        let snapshot = session.snapshot.lock().unwrap();
        assert_eq!(snapshot.status, RuntimeObservationStatus::ProcessExited);
        assert_eq!(snapshot.process.as_ref().unwrap().pid, 42);
        assert_eq!(
            snapshot.app_id_evidence.observed_values,
            vec![ZENLESS_ZONE_ZERO_APP_ID]
        );
    }

    #[test]
    fn ambiguous_scan_never_selects_one_process() {
        let target = test_target();
        let session = ObservationSession {
            stop_requested: AtomicBool::new(false),
            active: AtomicBool::new(true),
            snapshot: Mutex::new(empty_snapshot(&target)),
            target,
            candidates: Vec::new(),
            observer: Arc::new(EmptyScanner),
        };
        update_from_scan(
            &session,
            lxmi_runtime::ProcfsScan {
                matches: vec![observed_process(41), observed_process(42)],
                warnings: Vec::new(),
                environment_evidence: ObservationEvidenceState::Observed,
                diagnostics: ProcfsScanDiagnostics {
                    accepted_candidates: 2,
                    ..ProcfsScanDiagnostics::default()
                },
            },
            true,
        );
        let snapshot = session.snapshot.lock().unwrap();
        assert_eq!(snapshot.status, RuntimeObservationStatus::AmbiguousProcess);
        assert_eq!(snapshot.process, None);
        assert_eq!(snapshot.ambiguous_process_count, 2);
        assert_eq!(
            snapshot.observed_runtime.state,
            ObservationEvidenceState::Ambiguous
        );
    }

    #[test]
    fn user_stop_reports_not_found_or_completed_without_clearing_snapshot() {
        assert_eq!(
            status_after_user_stop(RuntimeObservationStatus::WaitingForGame),
            RuntimeObservationStatus::ProcessNotFound
        );
        assert_eq!(
            status_after_user_stop(RuntimeObservationStatus::Observed),
            RuntimeObservationStatus::Completed
        );
        assert_eq!(
            status_after_user_stop(RuntimeObservationStatus::AmbiguousProcess),
            RuntimeObservationStatus::AmbiguousProcess
        );

        let target = test_target();
        let session = ObservationSession {
            stop_requested: AtomicBool::new(false),
            active: AtomicBool::new(true),
            snapshot: Mutex::new(empty_snapshot(&target)),
            target,
            candidates: Vec::new(),
            observer: Arc::new(EmptyScanner),
        };
        update_from_scan(
            &session,
            lxmi_runtime::ProcfsScan {
                matches: vec![observed_process(42)],
                warnings: Vec::new(),
                environment_evidence: ObservationEvidenceState::Observed,
                diagnostics: ProcfsScanDiagnostics {
                    accepted_candidates: 1,
                    ..ProcfsScanDiagnostics::default()
                },
            },
            true,
        );
        finish_session(
            &session,
            RuntimeObservationStatus::Completed,
            Duration::from_secs(2),
        );
        let snapshot = session.snapshot.lock().unwrap();
        assert!(!snapshot.active);
        assert_eq!(snapshot.status, RuntimeObservationStatus::Completed);
        assert_eq!(snapshot.process.as_ref().unwrap().pid, 42);
        assert_eq!(snapshot.scan_diagnostics.accepted_candidates, 1);
    }

    #[test]
    fn observation_requires_explicit_user_confirmation_before_discovery() {
        let controller = RuntimeObservationController::default();
        let error = controller
            .start_zzz(false)
            .expect_err("consent is required");
        assert_eq!(error.code, "consent_required");
        assert!(controller.current().is_none());
    }

    #[test]
    #[ignore = "opt-in host smoke test; never starts Steam or ZZZ, user may start ZZZ manually first"]
    fn host_procfs_observation_smoke_is_read_only() {
        let discovery = SteamDiscoveryScanner::from_environment().scan();
        let game = discovery
            .games
            .games
            .iter()
            .find(|game| game.steam_app_id == ZENLESS_ZONE_ZERO_APP_ID)
            .expect("local Steam manifest for ZZZ");
        let observer = ProcfsObserver::for_current_user().expect("current-user procfs view");
        let target = observation_target(game, SystemInfo::current().home_directory);
        let scan = observer.scan(&target, &[]);
        assert!(scan.warnings.len() <= 16);
    }
}
