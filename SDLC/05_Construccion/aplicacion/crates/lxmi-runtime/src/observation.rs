//! Passive, bounded Linux process observation for a known Steam game.
//!
//! This module only reads `/proc` metadata. It does not expose process control,
//! signals, memory reads, tracing, or arbitrary process inspection.

use crate::RuntimeCandidate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const MAX_STATUS_BYTES: usize = 32 * 1024;
const MAX_STAT_BYTES: usize = 8 * 1024;
const MAX_COMM_BYTES: usize = 256;
const MAX_CMDLINE_BYTES: usize = 64 * 1024;
const MAX_ENVIRON_BYTES: usize = 64 * 1024;
const MAX_ANCESTRY_DEPTH: usize = 24;
const MAX_OBSERVATION_WARNINGS: usize = 16;
const APP_ID_ENV_KEYS: &[&str] = &["STEAM_COMPAT_APP_ID", "SteamAppId", "SteamGameId"];
const COMPATDATA_ENV_KEY: &str = "STEAM_COMPAT_DATA_PATH";
const PREFIX_ENV_KEY: &str = "WINEPREFIX";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeObservationStatus {
    WaitingForGame,
    Observed,
    Completed,
    ProcessNotFound,
    AmbiguousProcess,
    ProcessExited,
    TimedOut,
    BackendError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationEvidenceState {
    Verified,
    Conflicting,
    Observed,
    ObservedAndMatches,
    ObservedDifferent,
    DerivedFromObservedCompatdata,
    NotObserved,
    Unknown,
    Ambiguous,
    Unavailable,
    PermissionDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessTopologyState {
    ObservedAncestry,
    PartialTopology,
    Ambiguous,
    NotObserved,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcfsScanDiagnostics {
    pub same_uid_processes_scanned: usize,
    pub candidate_processes_considered: usize,
    pub candidate_metadata_unavailable: usize,
    pub candidate_environments_read: usize,
    pub candidate_environments_unavailable: usize,
    pub candidate_environments_permission_denied: usize,
    /// Accepted game-process contexts whose ancestry includes a process with
    /// unreadable allowlisted environment metadata.
    pub accepted_contexts_environment_unavailable: usize,
    pub accepted_contexts_environment_permission_denied: usize,
    pub rejected_executable_mismatch: usize,
    pub rejected_insufficient_identity_evidence: usize,
    pub accepted_candidates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedRuntimeTool {
    pub internal_id: Option<String>,
    pub display_name: String,
    pub version: Option<String>,
    /// Home-relative display path. The raw process environment is never returned.
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredRuntimeEvidence {
    pub per_game_state: String,
    pub per_game_internal_id: Option<String>,
    pub global_state: String,
    pub global_internal_id: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservationAppId {
    pub state: ObservationEvidenceState,
    pub observed_values: Vec<u32>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservationPath {
    pub state: ObservationEvidenceState,
    pub observed_path: Option<String>,
    pub expected_path: Option<String>,
    pub evidence_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservationSelection {
    pub state: ObservationEvidenceState,
    pub tool: Option<ObservedRuntimeTool>,
    pub evidence_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedGameProcess {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub executable_name: String,
    pub identity_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedProcessNode {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub process_name: String,
    pub executable_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WineServerObservation {
    pub state: ObservationEvidenceState,
    pub related_process_count: usize,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservationSummary {
    pub status: RuntimeObservationStatus,
    pub active: bool,
    pub process_observed: bool,
    pub ambiguous_process_count: usize,
    pub app_id: RuntimeObservationAppId,
    pub runtime: RuntimeObservationSelection,
    pub compatdata: RuntimeObservationPath,
    pub prefix: RuntimeObservationPath,
    pub wineserver: WineServerObservation,
    pub process_topology_state: ProcessTopologyState,
    pub observed_at_unix_millis: Option<u64>,
}

/// Ephemeral UI snapshot. Callers keep this in memory only and must not persist it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeObservationSnapshot {
    pub session_id: u64,
    pub status: RuntimeObservationStatus,
    pub active: bool,
    pub app_id: u32,
    pub started_at_unix_millis: u64,
    pub updated_at_unix_millis: u64,
    pub elapsed_millis: u64,
    pub scan_diagnostics: ProcfsScanDiagnostics,
    pub process: Option<ObservedGameProcess>,
    pub ambiguous_process_count: usize,
    pub app_id_evidence: RuntimeObservationAppId,
    pub configured_runtime: ConfiguredRuntimeEvidence,
    pub available_proton_tools: Vec<ObservedRuntimeTool>,
    pub observed_runtime: RuntimeObservationSelection,
    pub compatdata: RuntimeObservationPath,
    pub prefix: RuntimeObservationPath,
    pub process_topology: Vec<ObservedProcessNode>,
    pub process_topology_state: ProcessTopologyState,
    pub wineserver: WineServerObservation,
    pub environment_evidence: ObservationEvidenceState,
    pub warnings: Vec<String>,
}

impl RuntimeObservationSnapshot {
    pub fn summary(&self) -> RuntimeObservationSummary {
        RuntimeObservationSummary {
            status: self.status,
            active: self.active,
            process_observed: self.process.is_some(),
            ambiguous_process_count: self.ambiguous_process_count,
            app_id: self.app_id_evidence.clone(),
            runtime: self.observed_runtime.clone(),
            compatdata: self.compatdata.clone(),
            prefix: self.prefix.clone(),
            wineserver: self.wineserver.clone(),
            process_topology_state: self.process_topology_state,
            observed_at_unix_millis: (self.process.is_some() || self.ambiguous_process_count > 0)
                .then_some(self.updated_at_unix_millis),
        }
    }

    /// Redacts the home directory from paths returned to the frontend.
    pub fn redact_home_paths(&mut self, home: Option<&Path>) {
        self.configured_runtime.source = self
            .configured_runtime
            .source
            .as_deref()
            .map(|source| redact_string_path(source, home));
        for tool in &mut self.available_proton_tools {
            tool.path = redact_string_path(&tool.path, home);
        }
        if let Some(tool) = &mut self.observed_runtime.tool {
            tool.path = redact_string_path(&tool.path, home);
        }
        redact_observation_path(&mut self.compatdata, home);
        redact_observation_path(&mut self.prefix, home);
    }
}

fn redact_observation_path(path: &mut RuntimeObservationPath, home: Option<&Path>) {
    path.observed_path = path
        .observed_path
        .as_deref()
        .map(|value| redact_string_path(value, home));
    path.expected_path = path
        .expected_path
        .as_deref()
        .map(|value| redact_string_path(value, home));
}

#[derive(Debug, Clone)]
pub struct ObservationTarget {
    pub app_id: u32,
    pub expected_executable_names: Vec<String>,
    pub game_install_path: Option<PathBuf>,
    pub expected_executable_path: Option<PathBuf>,
    pub expected_compatdata_path: Option<PathBuf>,
    pub home_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcfsScan {
    pub matches: Vec<ObservedRuntimeProcess>,
    pub warnings: Vec<String>,
    pub environment_evidence: ObservationEvidenceState,
    pub diagnostics: ProcfsScanDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRejection {
    ExecutableMismatch,
    InsufficientIdentityEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedRuntimeProcess {
    pub process: ObservedGameProcess,
    pub app_id: RuntimeObservationAppId,
    pub runtime: RuntimeObservationSelection,
    pub compatdata: RuntimeObservationPath,
    pub prefix: RuntimeObservationPath,
    pub topology: Vec<ObservedProcessNode>,
    pub topology_state: ProcessTopologyState,
    pub wineserver: WineServerObservation,
    pub environment_evidence: ObservationEvidenceState,
}

#[derive(Debug, Clone)]
pub struct ProcfsObserver {
    proc_root: PathBuf,
    current_uid: u32,
    #[cfg(test)]
    denied_environment_pids: HashSet<u32>,
}

#[derive(Debug, Clone)]
struct ProcessMeta {
    pid: u32,
    parent_pid: Option<u32>,
    start_time: u64,
    comm: String,
}

#[derive(Debug, Clone)]
struct ProcessDetail {
    meta: ProcessMeta,
    executable: Option<PathBuf>,
    command_line: Vec<String>,
    allowlisted_environment: HashMap<String, String>,
    environment_state: ObservationEvidenceState,
}

impl ProcfsObserver {
    /// Uses the current user's `/proc` view. The application never accepts a procfs
    /// root or UID from frontend input.
    #[cfg(unix)]
    pub fn for_current_user() -> io::Result<Self> {
        use std::os::unix::fs::MetadataExt;

        let current_uid = fs::metadata("/proc/self")?.uid();
        Ok(Self {
            proc_root: PathBuf::from("/proc"),
            current_uid,
            #[cfg(test)]
            denied_environment_pids: HashSet::new(),
        })
    }

    #[cfg(not(unix))]
    pub fn for_current_user() -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Passive runtime observation currently requires Linux procfs",
        ))
    }

    #[cfg(test)]
    fn with_proc_root(proc_root: PathBuf, current_uid: u32) -> Self {
        Self {
            proc_root,
            current_uid,
            denied_environment_pids: HashSet::new(),
        }
    }

    pub fn scan(
        &self,
        target: &ObservationTarget,
        installed_tools: &[RuntimeCandidate],
    ) -> ProcfsScan {
        let mut warnings = Vec::new();
        let processes = match self.read_process_index(&mut warnings) {
            Ok(processes) => processes,
            Err(error) => {
                return ProcfsScan {
                    matches: Vec::new(),
                    warnings: vec![format!(
                        "No se pudo recorrer la vista procfs del usuario actual: {}",
                        safe_io_kind(&error)
                    )],
                    environment_evidence: ObservationEvidenceState::Unavailable,
                    diagnostics: ProcfsScanDiagnostics::default(),
                };
            }
        };
        let mut diagnostics = ProcfsScanDiagnostics {
            same_uid_processes_scanned: processes.len(),
            ..ProcfsScanDiagnostics::default()
        };
        let by_pid = processes
            .iter()
            .cloned()
            .map(|process| (process.pid, process))
            .collect::<HashMap<_, _>>();

        let mut matches = Vec::new();
        let mut environment_readable_candidates = 0usize;
        for meta in &processes {
            if !likely_game_or_wine_process(&meta.comm, target) {
                continue;
            }
            diagnostics.candidate_processes_considered += 1;
            let Some(detail) = self.read_detail(meta) else {
                diagnostics.candidate_metadata_unavailable += 1;
                continue;
            };
            match detail.environment_state {
                ObservationEvidenceState::Observed => {
                    diagnostics.candidate_environments_read += 1;
                    environment_readable_candidates += 1;
                }
                ObservationEvidenceState::PermissionDenied => {
                    diagnostics.candidate_environments_permission_denied += 1;
                }
                ObservationEvidenceState::Unavailable => {
                    diagnostics.candidate_environments_unavailable += 1;
                }
                _ => {}
            }
            match self.assess_candidate(&detail, &by_pid, target, installed_tools) {
                Ok(candidate) => {
                    diagnostics.accepted_candidates += 1;
                    match candidate.environment_evidence {
                        ObservationEvidenceState::PermissionDenied => {
                            diagnostics.accepted_contexts_environment_permission_denied += 1;
                        }
                        ObservationEvidenceState::Unavailable => {
                            diagnostics.accepted_contexts_environment_unavailable += 1;
                        }
                        _ => {}
                    }
                    matches.push(candidate);
                }
                Err(CandidateRejection::ExecutableMismatch) => {
                    diagnostics.rejected_executable_mismatch += 1;
                }
                Err(CandidateRejection::InsufficientIdentityEvidence) => {
                    diagnostics.rejected_insufficient_identity_evidence += 1;
                }
            }
        }

        if diagnostics.candidate_environments_permission_denied > 0 {
            push_warning(
                &mut warnings,
                "Linux denegó el acceso a parte del entorno allowlist de procesos candidatos.",
            );
        } else if diagnostics.candidate_environments_unavailable > 0
            || diagnostics.candidate_metadata_unavailable > 0
        {
            push_warning(
                &mut warnings,
                "Linux no permitió leer parte del entorno allowlist de procesos relacionados.",
            );
        }
        matches.sort_by_key(|item| item.process.pid);
        let environment_evidence = if diagnostics.candidate_environments_permission_denied > 0 {
            ObservationEvidenceState::PermissionDenied
        } else if diagnostics.candidate_environments_unavailable > 0
            || (diagnostics.candidate_metadata_unavailable > 0
                && environment_readable_candidates == 0)
        {
            ObservationEvidenceState::Unavailable
        } else if environment_readable_candidates > 0 {
            ObservationEvidenceState::Observed
        } else {
            ObservationEvidenceState::NotObserved
        };
        ProcfsScan {
            matches,
            warnings,
            environment_evidence,
            diagnostics,
        }
    }

    fn read_process_index(&self, warnings: &mut Vec<String>) -> io::Result<Vec<ProcessMeta>> {
        use std::os::unix::fs::MetadataExt;

        let mut processes = Vec::new();
        for entry in fs::read_dir(&self.proc_root)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    push_warning(
                        warnings,
                        &format!("Se omitió una entrada procfs: {}", safe_io_kind(&error)),
                    );
                    continue;
                }
            };
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.is_dir() && metadata.uid() == self.current_uid => metadata,
                Ok(_) => continue,
                Err(_) => continue,
            };
            let _ = metadata;
            let status = match read_limited(&path.join("status"), MAX_STATUS_BYTES) {
                Ok(status) => status,
                Err(_) => continue,
            };
            if effective_uid(&status) != Some(self.current_uid) {
                continue;
            }
            let comm = match read_limited(&path.join("comm"), MAX_COMM_BYTES) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).trim().to_string(),
                Err(_) => continue,
            };
            let stat = match read_limited(&path.join("stat"), MAX_STAT_BYTES) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let Some((parent_pid, start_time)) = parse_process_stat(&stat) else {
                continue;
            };
            processes.push(ProcessMeta {
                pid,
                parent_pid,
                start_time,
                comm,
            });
        }
        Ok(processes)
    }

    fn read_detail(&self, meta: &ProcessMeta) -> Option<ProcessDetail> {
        use std::os::unix::fs::MetadataExt;

        let process_dir = self.proc_root.join(meta.pid.to_string());
        let directory = fs::symlink_metadata(&process_dir).ok()?;
        if !directory.is_dir() || directory.uid() != self.current_uid {
            return None;
        }
        let status = read_limited(&process_dir.join("status"), MAX_STATUS_BYTES).ok()?;
        if effective_uid(&status) != Some(self.current_uid) {
            return None;
        }
        let before = read_limited(&process_dir.join("stat"), MAX_STAT_BYTES).ok()?;
        let (_, start_time) = parse_process_stat(&before)?;
        if start_time != meta.start_time {
            return None;
        }

        let executable = fs::read_link(process_dir.join("exe")).ok();
        let command_line = read_limited(&process_dir.join("cmdline"), MAX_CMDLINE_BYTES)
            .map(|bytes| split_nul_strings(&bytes))
            .unwrap_or_default();
        let (allowlisted_environment, environment_state) =
            match self.read_environment(meta.pid, &process_dir.join("environ")) {
                Ok(bytes) => (
                    parse_allowlisted_environment(&bytes),
                    ObservationEvidenceState::Observed,
                ),
                Err(error) => (HashMap::new(), environment_read_error_state(&error)),
            };

        let after = read_limited(&process_dir.join("stat"), MAX_STAT_BYTES).ok()?;
        let (_, after_start_time) = parse_process_stat(&after)?;
        if after_start_time != meta.start_time {
            return None;
        }

        Some(ProcessDetail {
            meta: meta.clone(),
            executable,
            command_line,
            allowlisted_environment,
            environment_state,
        })
    }

    fn read_environment(&self, pid: u32, path: &Path) -> io::Result<Vec<u8>> {
        #[cfg(test)]
        if self.denied_environment_pids.contains(&pid) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "permission denied fixture",
            ));
        }
        #[cfg(not(test))]
        let _ = pid;
        read_limited(path, MAX_ENVIRON_BYTES)
    }

    fn assess_candidate(
        &self,
        initial: &ProcessDetail,
        by_pid: &HashMap<u32, ProcessMeta>,
        target: &ObservationTarget,
        installed_tools: &[RuntimeCandidate],
    ) -> Result<ObservedRuntimeProcess, CandidateRejection> {
        let mut chain = vec![initial.clone()];
        let mut parent = initial.meta.parent_pid;
        let mut visited = HashSet::from([initial.meta.pid]);
        for _ in 0..MAX_ANCESTRY_DEPTH {
            let Some(parent_pid) = parent else { break };
            if !visited.insert(parent_pid) {
                break;
            }
            let Some(parent_meta) = by_pid.get(&parent_pid) else {
                break;
            };
            let Some(parent_detail) = self.read_detail(parent_meta) else {
                break;
            };
            parent = parent_detail.meta.parent_pid;
            chain.push(parent_detail);
        }

        let process_name = executable_name(initial);
        if !is_expected_executable(
            &process_name,
            &initial.meta.comm,
            &initial.command_line,
            target,
        ) {
            return Err(CandidateRejection::ExecutableMismatch);
        }
        let expected_path_match = path_matches_installation(
            initial.executable.as_deref(),
            target.expected_executable_path.as_deref(),
            target.game_install_path.as_deref(),
        ) || command_path_matches(&initial.command_line, target);
        let app_ids = collect_app_ids(&chain);
        let compat_paths = collect_environment_paths(&chain, COMPATDATA_ENV_KEY);
        let compat_matches = target
            .expected_compatdata_path
            .as_deref()
            .is_some_and(|expected| {
                compat_paths
                    .iter()
                    .any(|value| paths_equal(value, expected))
            });
        let steam_ancestor = chain.iter().skip(1).any(is_steam_related_process);

        let mut identity_evidence = vec!["expected_executable_name".to_string()];
        if expected_path_match {
            identity_evidence.push("executable_path_matches_detected_installation".into());
        }
        if app_ids.contains(&target.app_id) {
            identity_evidence.push("steam_app_id_environment_matches".into());
        }
        if compat_matches {
            identity_evidence.push("compatdata_environment_matches".into());
        }
        if steam_ancestor {
            identity_evidence.push("steam_related_process_in_ancestry".into());
        }
        if identity_evidence.len() < 2 {
            return Err(CandidateRejection::InsufficientIdentityEvidence);
        }

        let app_id = assess_app_id(target.app_id, &chain);
        let compatdata = assess_compatdata(target, &chain);
        let prefix = assess_prefix(&chain, &compatdata);
        let runtime = assess_runtime(&chain, installed_tools);
        let topology = relevant_topology(&chain, installed_tools, target.home_directory.as_deref());
        let wineserver = self.assess_wineserver(
            &self.processes_from_index(by_pid),
            prefix.observed_path.as_deref(),
        );
        let environment_evidence = if chain.iter().any(|detail| {
            matches!(
                detail.environment_state,
                ObservationEvidenceState::Unavailable | ObservationEvidenceState::PermissionDenied
            )
        }) {
            if chain.iter().any(|detail| {
                detail.environment_state == ObservationEvidenceState::PermissionDenied
            }) {
                ObservationEvidenceState::PermissionDenied
            } else {
                ObservationEvidenceState::Unavailable
            }
        } else {
            ObservationEvidenceState::Observed
        };
        let executable_display =
            matched_expected_executable_name(initial, target).unwrap_or_else(|| {
                if process_name.is_empty() {
                    initial.meta.comm.clone()
                } else {
                    process_name
                }
            });
        let topology_state = if chain.len() > 1 {
            ProcessTopologyState::ObservedAncestry
        } else {
            ProcessTopologyState::PartialTopology
        };
        Ok(ObservedRuntimeProcess {
            process: ObservedGameProcess {
                pid: initial.meta.pid,
                parent_pid: initial.meta.parent_pid,
                executable_name: executable_display,
                identity_evidence,
            },
            app_id,
            runtime,
            compatdata,
            prefix,
            topology,
            topology_state,
            wineserver,
            environment_evidence,
        })
    }

    fn assess_wineserver(
        &self,
        process_index: &[ProcessMeta],
        expected_prefix: Option<&str>,
    ) -> WineServerObservation {
        let mut servers = Vec::new();
        let mut unavailable = false;
        for meta in process_index
            .iter()
            .filter(|meta| is_wineserver_name(&meta.comm))
        {
            let Some(detail) = self.read_detail(meta) else {
                unavailable = true;
                continue;
            };
            match detail.allowlisted_environment.get(PREFIX_ENV_KEY) {
                Some(path)
                    if expected_prefix.is_some_and(|expected| {
                        paths_equal(Path::new(path), Path::new(expected))
                    }) =>
                {
                    servers.push((meta.pid, true));
                }
                Some(_) => servers.push((meta.pid, false)),
                None if detail.environment_state == ObservationEvidenceState::Unavailable => {
                    unavailable = true;
                    servers.push((meta.pid, false));
                }
                None => servers.push((meta.pid, false)),
            }
        }
        let matching = servers.iter().filter(|(_, matches)| *matches).count();
        let state = if matching == 1 {
            ObservationEvidenceState::ObservedAndMatches
        } else if matching > 1 {
            ObservationEvidenceState::Ambiguous
        } else if unavailable && !servers.is_empty() {
            ObservationEvidenceState::Unavailable
        } else if !servers.is_empty() {
            ObservationEvidenceState::Unknown
        } else {
            ObservationEvidenceState::NotObserved
        };
        let evidence = match state {
            ObservationEvidenceState::ObservedAndMatches => {
                "Un proceso wineserver del usuario actual expone el mismo WINEPREFIX observado. Esto no demuestra que el proceso del juego use esa instancia de wineserver.".into()
            }
            ObservationEvidenceState::Ambiguous => {
                "Varios procesos wineserver exponen el mismo WINEPREFIX; no se seleccionó uno.".into()
            }
            ObservationEvidenceState::Unavailable => {
                "Hay procesos wineserver, pero Linux no permitió leer todo el entorno allowlist.".into()
            }
            ObservationEvidenceState::Unknown => {
                "Se observaron procesos wineserver, pero no se pudo relacionarlos con el prefix observado.".into()
            }
            _ => "No se observó un proceso wineserver relacionado en la vista del usuario actual.".into(),
        };
        WineServerObservation {
            state,
            related_process_count: matching,
            evidence,
        }
    }

    fn processes_from_index(&self, by_pid: &HashMap<u32, ProcessMeta>) -> Vec<ProcessMeta> {
        by_pid.values().cloned().collect()
    }
}

fn likely_game_or_wine_process(comm: &str, target: &ObservationTarget) -> bool {
    let normalized_comm = normalize_process_name(comm);
    target.expected_executable_names.iter().any(|name| {
        let normalized = normalize_process_name(name);
        !normalized.is_empty()
            && (normalized_comm == normalized
                || (normalized.starts_with(&normalized_comm) && normalized_comm.len() >= 10))
    }) || normalized_comm.contains("wine")
        || normalized_comm.contains("zenless")
}

fn is_expected_executable(
    executable: &str,
    comm: &str,
    command_line: &[String],
    target: &ObservationTarget,
) -> bool {
    let candidate_names = std::iter::once(executable)
        .chain(std::iter::once(comm))
        .chain(command_line.iter().map(String::as_str));
    target.expected_executable_names.iter().any(|expected| {
        let expected = normalize_process_name(expected);
        candidate_names.clone().any(|candidate| {
            let base = basename_any(candidate);
            let normalized = normalize_process_name(base);
            normalized == expected || (normalized.len() >= 10 && expected.starts_with(&normalized))
        })
    })
}

fn matched_expected_executable_name(
    process: &ProcessDetail,
    target: &ObservationTarget,
) -> Option<String> {
    let candidates = process
        .command_line
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(process.meta.comm.as_str()))
        .chain(process.executable.as_deref().and_then(Path::to_str));
    candidates
        .filter_map(|candidate| {
            let name = basename_any(candidate);
            let normalized = normalize_process_name(name);
            target
                .expected_executable_names
                .iter()
                .any(|expected| {
                    let expected = normalize_process_name(expected);
                    normalized == expected
                        || (normalized.len() >= 10 && expected.starts_with(&normalized))
                })
                .then(|| name.to_string())
        })
        .next()
}

fn path_matches_installation(
    observed_executable: Option<&Path>,
    expected_executable: Option<&Path>,
    install_path: Option<&Path>,
) -> bool {
    let Some(path) = observed_executable else {
        return false;
    };
    if expected_executable.is_some_and(|expected| paths_equal(path, expected)) {
        return true;
    }
    install_path.is_some_and(|root| path.starts_with(root))
}

fn command_path_matches(command_line: &[String], target: &ObservationTarget) -> bool {
    command_line.iter().any(|argument| {
        let normalized = argument.replace('\\', "/");
        let path = Path::new(&normalized);
        target
            .expected_executable_path
            .as_deref()
            .is_some_and(|expected| paths_equal(path, expected))
            || target
                .game_install_path
                .as_deref()
                .is_some_and(|root| path.starts_with(root))
    })
}

fn assess_app_id(expected_app_id: u32, chain: &[ProcessDetail]) -> RuntimeObservationAppId {
    let mut values = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for process in chain {
        for key in APP_ID_ENV_KEYS {
            if let Some(value) = process.allowlisted_environment.get(*key) {
                if let Some(app_id) = parse_app_id(value) {
                    values.insert(app_id);
                    sources.insert(format!("{} (PID {})", key, process.meta.pid));
                }
            }
        }
    }
    let state = if values.is_empty() {
        missing_environment_evidence_state(chain)
    } else if values.iter().all(|value| *value == expected_app_id) {
        ObservationEvidenceState::Verified
    } else {
        ObservationEvidenceState::Conflicting
    };
    RuntimeObservationAppId {
        state,
        observed_values: values.into_iter().collect(),
        sources: sources.into_iter().collect(),
    }
}

fn assess_compatdata(
    target: &ObservationTarget,
    chain: &[ProcessDetail],
) -> RuntimeObservationPath {
    let mut paths = collect_environment_paths(chain, COMPATDATA_ENV_KEY);
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return RuntimeObservationPath {
            state: missing_environment_evidence_state(chain),
            observed_path: None,
            expected_path: target
                .expected_compatdata_path
                .as_deref()
                .map(|path| path.to_string_lossy().into_owned()),
            evidence_source: None,
        };
    }
    if paths.len() > 1 {
        return RuntimeObservationPath {
            state: ObservationEvidenceState::Conflicting,
            observed_path: None,
            expected_path: target
                .expected_compatdata_path
                .as_deref()
                .map(|path| path.to_string_lossy().into_owned()),
            evidence_source: Some(COMPATDATA_ENV_KEY.into()),
        };
    }
    let matches_expected = target
        .expected_compatdata_path
        .as_deref()
        .is_some_and(|expected| paths_equal(&paths[0], expected));
    let state = if matches_expected {
        ObservationEvidenceState::ObservedAndMatches
    } else {
        ObservationEvidenceState::ObservedDifferent
    };
    RuntimeObservationPath {
        state,
        observed_path: Some(paths[0].to_string_lossy().into_owned()),
        expected_path: target
            .expected_compatdata_path
            .as_deref()
            .map(|path| path.to_string_lossy().into_owned()),
        evidence_source: Some(COMPATDATA_ENV_KEY.into()),
    }
}

fn missing_environment_evidence_state(chain: &[ProcessDetail]) -> ObservationEvidenceState {
    if chain
        .iter()
        .any(|process| process.environment_state == ObservationEvidenceState::PermissionDenied)
    {
        ObservationEvidenceState::PermissionDenied
    } else if chain
        .iter()
        .any(|process| process.environment_state == ObservationEvidenceState::Unavailable)
    {
        ObservationEvidenceState::Unavailable
    } else {
        ObservationEvidenceState::NotObserved
    }
}

fn assess_prefix(
    chain: &[ProcessDetail],
    compatdata: &RuntimeObservationPath,
) -> RuntimeObservationPath {
    let mut observed = collect_environment_paths(chain, PREFIX_ENV_KEY);
    observed.sort();
    observed.dedup();
    if observed.len() == 1 {
        return RuntimeObservationPath {
            state: ObservationEvidenceState::Observed,
            observed_path: observed
                .first()
                .map(|path| path.to_string_lossy().into_owned()),
            expected_path: None,
            evidence_source: Some(PREFIX_ENV_KEY.into()),
        };
    }
    if observed.len() > 1 {
        return RuntimeObservationPath {
            state: ObservationEvidenceState::Ambiguous,
            observed_path: None,
            expected_path: None,
            evidence_source: Some(PREFIX_ENV_KEY.into()),
        };
    }
    let Some(compatdata_path) = compatdata.observed_path.as_deref() else {
        return RuntimeObservationPath {
            state: ObservationEvidenceState::Unknown,
            observed_path: None,
            expected_path: None,
            evidence_source: None,
        };
    };
    let path = Path::new(compatdata_path).join("pfx");
    RuntimeObservationPath {
        state: ObservationEvidenceState::DerivedFromObservedCompatdata,
        observed_path: Some(path.to_string_lossy().into_owned()),
        expected_path: None,
        evidence_source: Some(format!(
            "{} + documented Proton compatdata/pfx layout",
            COMPATDATA_ENV_KEY
        )),
    }
}

fn assess_runtime(
    chain: &[ProcessDetail],
    installed_tools: &[RuntimeCandidate],
) -> RuntimeObservationSelection {
    let mut matched = Vec::new();
    for tool in installed_tools {
        let found = chain.iter().any(|process| {
            process
                .executable
                .as_deref()
                .is_some_and(|path| path_matches_tool(path, &tool.path))
                || process
                    .command_line
                    .iter()
                    .any(|argument| path_matches_tool(Path::new(argument), &tool.path))
        });
        if found {
            matched.push(ObservedRuntimeTool {
                internal_id: tool.internal_id.clone(),
                display_name: tool.display_name.clone(),
                version: tool.version.clone(),
                path: tool.path.to_string_lossy().into_owned(),
            });
        }
    }
    matched.sort_by(|left, right| left.path.cmp(&right.path));
    matched.dedup_by(|left, right| left.path == right.path);
    match matched.len() {
        1 => RuntimeObservationSelection {
            state: ObservationEvidenceState::Verified,
            tool: matched.pop(),
            evidence_source: Some(
                "running process executable/argv matched an installed Proton tool root".into(),
            ),
        },
        0 => RuntimeObservationSelection {
            state: ObservationEvidenceState::Unknown,
            tool: None,
            evidence_source: None,
        },
        _ => RuntimeObservationSelection {
            state: ObservationEvidenceState::Ambiguous,
            tool: None,
            evidence_source: Some(
                "multiple installed Proton tool roots matched the observed process chain".into(),
            ),
        },
    }
}

fn relevant_topology(
    chain: &[ProcessDetail],
    installed_tools: &[RuntimeCandidate],
    home: Option<&Path>,
) -> Vec<ObservedProcessNode> {
    chain
        .iter()
        .rev()
        .filter_map(|process| {
            let exe_name = process
                .executable
                .as_deref()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .map(str::to_owned);
            let is_game = process.meta.pid == chain.first()?.meta.pid;
            let matched_proton = installed_tools.iter().any(|tool| {
                process
                    .executable
                    .as_deref()
                    .is_some_and(|path| path_matches_tool(path, &tool.path))
                    || process
                        .command_line
                        .iter()
                        .any(|argument| path_matches_tool(Path::new(argument), &tool.path))
            });
            let role = if is_game {
                "game"
            } else if matched_proton
                || process
                    .command_line
                    .iter()
                    .any(|arg| arg.to_ascii_lowercase().contains("/proton"))
            {
                "proton"
            } else if is_wineserver_name(&process.meta.comm) {
                "wineserver"
            } else if process.meta.comm.to_ascii_lowercase().contains("wine") {
                "wine"
            } else if is_steam_related_process(process) {
                "steam"
            } else if process
                .meta
                .comm
                .to_ascii_lowercase()
                .contains("pressure-vessel")
            {
                "container_runtime"
            } else {
                return None;
            };
            Some(ObservedProcessNode {
                pid: process.meta.pid,
                parent_pid: process.meta.parent_pid,
                process_name: process.meta.comm.clone(),
                executable_name: exe_name.map(|name| redact_string_path(&name, home)),
                role: role.into(),
            })
        })
        .collect()
}

fn is_steam_related_process(process: &ProcessDetail) -> bool {
    let comm = process.meta.comm.to_ascii_lowercase();
    if comm.contains("steam") || comm.contains("pressure-vessel") || comm.contains("reaper") {
        return true;
    }
    process.executable.as_ref().is_some_and(|path| {
        path.to_string_lossy()
            .to_ascii_lowercase()
            .contains("/steam/")
    }) || process.command_line.iter().any(|argument| {
        let lower = argument.to_ascii_lowercase();
        lower.contains("steam") || lower.contains("pressure-vessel")
    })
}

fn is_wineserver_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized == "wineserver"
        || normalized == "wineserver64"
        || normalized.starts_with("wineserver-")
}

fn collect_app_ids(chain: &[ProcessDetail]) -> BTreeSet<u32> {
    chain
        .iter()
        .flat_map(|process| {
            APP_ID_ENV_KEYS.iter().filter_map(move |key| {
                process
                    .allowlisted_environment
                    .get(*key)
                    .and_then(|value| parse_app_id(value))
            })
        })
        .collect()
}

fn collect_environment_paths(chain: &[ProcessDetail], key: &str) -> Vec<PathBuf> {
    chain
        .iter()
        .filter_map(|process| process.allowlisted_environment.get(key))
        .filter(|value| Path::new(value).is_absolute())
        .map(PathBuf::from)
        .collect()
}

fn parse_app_id(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok().filter(|value| *value > 0)
}

fn path_matches_tool(observed: &Path, tool_root: &Path) -> bool {
    let normalized = observed.to_string_lossy().replace('\\', "/");
    let observed = Path::new(&normalized);
    observed == tool_root || observed.starts_with(tool_root)
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    lexical_absolute(left) == lexical_absolute(right)
}

fn lexical_absolute(path: &Path) -> PathBuf {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                output.pop();
            }
            other => output.push(other.as_os_str()),
        }
    }
    output
}

fn display_path(path: &Path, home: Option<&Path>) -> String {
    let mut rendered = path.to_string_lossy().into_owned();
    if let Some(home) = home {
        let home = home.to_string_lossy();
        if rendered == home {
            return "~".into();
        }
        if let Some(suffix) = rendered
            .strip_prefix(home.as_ref())
            .filter(|suffix| suffix.starts_with('/'))
        {
            rendered = format!("~{suffix}");
        }
    }
    rendered
}

fn redact_string_path(value: &str, home: Option<&Path>) -> String {
    display_path(Path::new(value), home)
}

fn executable_name(process: &ProcessDetail) -> String {
    process
        .executable
        .as_deref()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .or_else(|| {
            process
                .command_line
                .iter()
                .map(|argument| basename_any(argument))
                .find(|name| !name.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| process.meta.comm.clone())
}

fn basename_any(value: &str) -> &str {
    value.rsplit(['/', '\\']).next().unwrap_or(value)
}

fn normalize_process_name(value: &str) -> String {
    basename_any(value)
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn split_nul_strings(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect()
}

fn parse_allowlisted_environment(bytes: &[u8]) -> HashMap<String, String> {
    let allowlist = APP_ID_ENV_KEYS
        .iter()
        .copied()
        .chain([COMPATDATA_ENV_KEY, PREFIX_ENV_KEY])
        .collect::<HashSet<_>>();
    split_nul_strings(bytes)
        .into_iter()
        .filter_map(|entry| {
            let (key, value) = entry.split_once('=')?;
            allowlist
                .contains(key)
                .then(|| (key.to_owned(), value.to_owned()))
        })
        .collect()
}

fn effective_uid(status: &[u8]) -> Option<u32> {
    let status = std::str::from_utf8(status).ok()?;
    let uid_line = status.lines().find(|line| line.starts_with("Uid:"))?;
    uid_line
        .split_whitespace()
        // `Uid:` is followed by real, effective, saved-set, and filesystem UIDs.
        .nth(2)
        .and_then(|value| value.parse().ok())
}

/// `/proc/<pid>/stat` has a parenthesized command which can itself contain spaces
/// and parentheses. Parse fields only after the final closing parenthesis.
fn parse_process_stat(bytes: &[u8]) -> Option<(Option<u32>, u64)> {
    let value = std::str::from_utf8(bytes).ok()?.trim();
    let close = value.rfind(')')?;
    let remaining = value
        .get(close + 1..)?
        .split_whitespace()
        .collect::<Vec<_>>();
    let parent_pid = remaining.get(1)?.parse::<u32>().ok();
    let start_time = remaining.get(19)?.parse::<u64>().ok()?;
    Some((parent_pid, start_time))
}

fn read_limited(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(limit.min(4096));
    file.take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "proc metadata exceeded its read limit",
        ));
    }
    Ok(bytes)
}

fn safe_io_kind(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::PermissionDenied => "permiso denegado",
        io::ErrorKind::NotFound => "entrada ausente",
        io::ErrorKind::InvalidData => "metadatos fuera del límite permitido",
        _ => "error de lectura",
    }
}

fn environment_read_error_state(error: &io::Error) -> ObservationEvidenceState {
    if error.kind() == io::ErrorKind::PermissionDenied {
        ObservationEvidenceState::PermissionDenied
    } else {
        ObservationEvidenceState::Unavailable
    }
}

fn push_warning(warnings: &mut Vec<String>, warning: &str) {
    if warnings.len() < MAX_OBSERVATION_WARNINGS && !warnings.iter().any(|item| item == warning) {
        warnings.push(warning.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, MetadataExt};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct ProcFixture {
        root: PathBuf,
    }

    impl ProcFixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let root = std::env::temp_dir().join(format!("lxmi-proc-fixture-{nonce}"));
            fs::create_dir_all(&root).expect("fixture proc root");
            Self { root }
        }

        fn add_process(
            &self,
            pid: u32,
            ppid: u32,
            comm: &str,
            cmdline: &[&str],
            environment: &[(&str, &str)],
            executable: Option<&Path>,
        ) {
            let dir = self.root.join(pid.to_string());
            fs::create_dir_all(&dir).expect("process fixture");
            let uid = fs::metadata("/proc/self").expect("current process").uid();
            fs::write(
                dir.join("status"),
                format!("Name:\t{comm}\nUid:\t{uid}\t{uid}\t{uid}\t{uid}\n"),
            )
            .expect("status");
            fs::write(dir.join("comm"), comm).expect("comm");
            fs::write(dir.join("stat"), fixture_stat(pid, comm, ppid, 777)).expect("stat");
            let cmdline = cmdline
                .iter()
                .flat_map(|value| value.as_bytes().iter().copied().chain([0]))
                .collect::<Vec<_>>();
            fs::write(dir.join("cmdline"), cmdline).expect("cmdline");
            let environment = environment
                .iter()
                .flat_map(|(key, value)| {
                    format!("{key}={value}").into_bytes().into_iter().chain([0])
                })
                .collect::<Vec<_>>();
            fs::write(dir.join("environ"), environment).expect("environ");
            if let Some(executable) = executable {
                symlink(executable, dir.join("exe")).expect("exe symlink");
            }
        }
    }

    impl Drop for ProcFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn fixture_stat(pid: u32, comm: &str, ppid: u32, start_time: u64) -> String {
        let tail = std::iter::once("S".to_string())
            .chain(std::iter::once(ppid.to_string()))
            .chain((0..17).map(|_| "0".to_string()))
            .chain(std::iter::once(start_time.to_string()))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{pid} ({comm}) {tail}")
    }

    fn target() -> ObservationTarget {
        ObservationTarget {
            app_id: 4_162_040,
            expected_executable_names: vec!["ZenlessZoneZero.exe".into()],
            game_install_path: Some(PathBuf::from("/games/zzz")),
            expected_executable_path: Some(PathBuf::from("/games/zzz/ZenlessZoneZero.exe")),
            expected_compatdata_path: Some(PathBuf::from("/steam/steamapps/compatdata/4162040")),
            home_directory: Some(PathBuf::from("/home/tester")),
        }
    }

    fn current_uid() -> u32 {
        fs::metadata("/proc/self").expect("current process").uid()
    }

    #[test]
    fn exact_game_path_and_name_identify_process_without_selecting_installed_tool() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let tool = RuntimeCandidate {
            internal_id: Some("proton-experimental".into()),
            display_name: "Proton Experimental".into(),
            path: PathBuf::from("/steam/steamapps/common/Proton Experimental"),
            source: lxmi_proton::CompatibilityToolSource::SteamLibrary,
            version: Some("fixture".into()),
        };
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[tool]);
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(scan.matches[0].process.pid, 202);
        assert_eq!(
            scan.matches[0].app_id.state,
            ObservationEvidenceState::NotObserved
        );
        assert_eq!(
            scan.matches[0].runtime.state,
            ObservationEvidenceState::Unknown
        );
        assert_eq!(
            scan.matches[0].compatdata.state,
            ObservationEvidenceState::NotObserved
        );
        assert_eq!(
            scan.matches[0].prefix.state,
            ObservationEvidenceState::Unknown
        );
        assert_eq!(scan.diagnostics.accepted_candidates, 1);
    }

    #[test]
    fn already_running_zzz_is_found_on_the_first_scan_when_proc_exe_is_wine() {
        let fixture = ProcFixture::new();
        let proton_root = Path::new("/steam/steamapps/common/Proton Experimental");
        fixture.add_process(
            101,
            1,
            "proton",
            &["/steam/steamapps/common/Proton Experimental/proton", "run"],
            &[
                ("STEAM_COMPAT_APP_ID", "4162040"),
                (
                    "STEAM_COMPAT_DATA_PATH",
                    "/steam/steamapps/compatdata/4162040",
                ),
            ],
            Some(proton_root.join("proton").as_path()),
        );
        fixture.add_process(
            202,
            101,
            "wine64-preloader",
            &[r"Z:\games\ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/steam/Proton/files/bin/wine64-preloader")),
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let runtime = RuntimeCandidate {
            internal_id: Some("proton-experimental".into()),
            display_name: "Proton Experimental".into(),
            path: proton_root.to_path_buf(),
            source: lxmi_proton::CompatibilityToolSource::SteamLibrary,
            version: Some("fixture".into()),
        };

        // This is the first scan after observation starts; it must see a process
        // that was already present rather than waiting for a creation event.
        let scan = observer.scan(&target(), &[runtime]);

        assert_eq!(scan.matches.len(), 1);
        assert_eq!(scan.matches[0].process.pid, 202);
        assert_eq!(
            scan.matches[0].process.executable_name,
            "ZenlessZoneZero.exe"
        );
        assert_eq!(
            scan.matches[0].app_id.state,
            ObservationEvidenceState::Verified
        );
        assert_eq!(
            scan.matches[0].compatdata.state,
            ObservationEvidenceState::ObservedAndMatches
        );
        assert_eq!(
            scan.matches[0].runtime.state,
            ObservationEvidenceState::Verified
        );
        assert_eq!(scan.diagnostics.accepted_candidates, 1);
    }

    #[test]
    fn app_id_compatdata_prefix_and_proton_are_observed_from_process_ancestry() {
        let fixture = ProcFixture::new();
        let proton_path = Path::new("/steam/steamapps/common/Proton Experimental/proton");
        fixture.add_process(
            101,
            1,
            "python3",
            &[proton_path.to_str().unwrap(), "waitforexitandrun"],
            &[
                ("STEAM_COMPAT_APP_ID", "4162040"),
                (
                    "STEAM_COMPAT_DATA_PATH",
                    "/steam/steamapps/compatdata/4162040",
                ),
            ],
            Some(Path::new("/usr/bin/python3")),
        );
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let tool = RuntimeCandidate {
            internal_id: Some("proton-experimental".into()),
            display_name: "Proton Experimental".into(),
            path: PathBuf::from("/steam/steamapps/common/Proton Experimental"),
            source: lxmi_proton::CompatibilityToolSource::SteamLibrary,
            version: Some("fixture-version".into()),
        };
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[tool]);
        assert_eq!(scan.matches.len(), 1);
        let match_ = &scan.matches[0];
        assert_eq!(match_.app_id.state, ObservationEvidenceState::Verified);
        assert_eq!(
            match_.compatdata.state,
            ObservationEvidenceState::ObservedAndMatches
        );
        assert_eq!(
            match_.prefix.state,
            ObservationEvidenceState::DerivedFromObservedCompatdata
        );
        assert_eq!(match_.runtime.state, ObservationEvidenceState::Verified);
        assert_eq!(
            match_.runtime.tool.as_ref().unwrap().display_name,
            "Proton Experimental"
        );
        assert!(match_.topology.iter().any(|node| node.role == "proton"));
    }

    #[test]
    fn conflicting_app_id_and_compatdata_are_preserved() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[
                ("STEAM_COMPAT_APP_ID", "999"),
                ("STEAM_COMPAT_DATA_PATH", "/other/compatdata/999"),
            ],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(
            scan.matches[0].app_id.state,
            ObservationEvidenceState::Conflicting
        );
        assert_eq!(
            scan.matches[0].compatdata.state,
            ObservationEvidenceState::ObservedDifferent
        );
    }

    #[test]
    fn conflicting_compatdata_values_in_process_chain_are_not_collapsed() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            101,
            1,
            "proton",
            &["/steam/Proton Experimental/proton"],
            &[("STEAM_COMPAT_DATA_PATH", "/steam/compatdata/4162040")],
            Some(Path::new("/usr/bin/python3")),
        );
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[("STEAM_COMPAT_DATA_PATH", "/other/compatdata/4162040")],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(
            scan.matches[0].compatdata.state,
            ObservationEvidenceState::Conflicting
        );
        assert_eq!(scan.matches[0].compatdata.observed_path, None);
    }

    #[test]
    fn multiple_game_processes_remain_ambiguous_for_session_layer() {
        let fixture = ProcFixture::new();
        for pid in [202, 303] {
            fixture.add_process(
                pid,
                101,
                "ZenlessZoneZero.exe",
                &["ZenlessZoneZero.exe"],
                &[],
                Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
            );
        }
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert_eq!(scan.matches.len(), 2);
    }

    #[test]
    fn process_owned_by_another_uid_is_ignored() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let observer =
            ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid().saturating_add(1));
        let scan = observer.scan(&target(), &[]);
        assert!(scan.matches.is_empty());
    }

    #[test]
    fn process_with_different_effective_uid_is_ignored() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let uid = current_uid();
        fs::write(
            fixture.root.join("202/status"),
            format!(
                "Name:\tZenlessZoneZero.exe\nUid:\t{uid}\t{}\t{uid}\t{uid}\n",
                uid + 1
            ),
        )
        .expect("write effective uid fixture");
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), uid);
        let scan = observer.scan(&target(), &[]);
        assert!(scan.matches.is_empty());
    }

    #[test]
    fn environment_parser_returns_only_allowlisted_keys() {
        let parsed = parse_allowlisted_environment(
            b"STEAM_COMPAT_APP_ID=4162040\0STEAM_COMPAT_DATA_PATH=/steam/compat\0TOKEN=secret\0WINEPREFIX=/tmp/pfx\0",
        );
        assert_eq!(parsed.len(), 3);
        assert!(!parsed.contains_key("TOKEN"));
        assert_eq!(parsed.get("STEAM_COMPAT_APP_ID").unwrap(), "4162040");
    }

    #[test]
    fn unavailable_environment_is_reported_without_dropping_path_identified_game() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        fs::remove_file(fixture.root.join("202/environ")).expect("remove fixture environ");
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(
            scan.matches[0].environment_evidence,
            ObservationEvidenceState::Unavailable
        );
        assert_eq!(
            scan.matches[0].app_id.state,
            ObservationEvidenceState::Unavailable
        );
        assert_eq!(
            scan.matches[0].compatdata.state,
            ObservationEvidenceState::Unavailable
        );
    }

    #[test]
    fn observed_wineprefix_is_distinguished_from_compatdata_derivation() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[("WINEPREFIX", "/custom/prefix")],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert_eq!(scan.matches.len(), 1);
        assert_eq!(
            scan.matches[0].prefix.state,
            ObservationEvidenceState::Observed
        );
        assert_eq!(
            scan.matches[0].prefix.observed_path.as_deref(),
            Some("/custom/prefix")
        );
    }

    #[test]
    fn expected_process_name_without_independent_identity_evidence_is_rejected() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            None,
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        let scan = observer.scan(&target(), &[]);
        assert!(scan.matches.is_empty());
        assert_eq!(scan.diagnostics.rejected_insufficient_identity_evidence, 1);
    }

    #[test]
    fn environment_permission_denial_remains_distinct_from_unavailable() {
        let denied = io::Error::new(io::ErrorKind::PermissionDenied, "fixture");
        let missing = io::Error::new(io::ErrorKind::NotFound, "fixture");
        assert_eq!(
            environment_read_error_state(&denied),
            ObservationEvidenceState::PermissionDenied
        );
        assert_eq!(
            environment_read_error_state(&missing),
            ObservationEvidenceState::Unavailable
        );
    }

    #[test]
    fn accepted_context_reports_denied_ancestor_separately_from_direct_candidate_scan() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            101,
            1,
            "proton",
            &["/steam/Proton Experimental/proton", "run"],
            &[],
            Some(Path::new("/steam/Proton Experimental/proton")),
        );
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[
                ("STEAM_COMPAT_APP_ID", "4162040"),
                (
                    "STEAM_COMPAT_DATA_PATH",
                    "/steam/steamapps/compatdata/4162040",
                ),
            ],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );

        let mut observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        observer.denied_environment_pids.insert(101);
        let scan = observer.scan(&target(), &[]);

        assert_eq!(scan.matches.len(), 1);
        assert_eq!(
            scan.matches[0].environment_evidence,
            ObservationEvidenceState::PermissionDenied
        );
        assert_eq!(scan.diagnostics.accepted_candidates, 1);
        assert_eq!(scan.diagnostics.candidate_environments_permission_denied, 0);
        assert_eq!(
            scan.diagnostics
                .accepted_contexts_environment_permission_denied,
            1
        );
    }

    #[test]
    fn disappeared_proc_entry_is_no_longer_reported() {
        let fixture = ProcFixture::new();
        fixture.add_process(
            202,
            101,
            "ZenlessZoneZero.exe",
            &["ZenlessZoneZero.exe"],
            &[],
            Some(Path::new("/games/zzz/ZenlessZoneZero.exe")),
        );
        let observer = ProcfsObserver::with_proc_root(fixture.root.clone(), current_uid());
        assert_eq!(observer.scan(&target(), &[]).matches.len(), 1);
        fs::remove_dir_all(fixture.root.join("202")).expect("remove vanished process fixture");
        assert!(observer.scan(&target(), &[]).matches.is_empty());
    }

    #[test]
    fn proc_stat_parser_handles_parentheses_in_process_name() {
        let parsed = parse_process_stat(fixture_stat(22, "wine (helper)", 11, 999).as_bytes());
        assert_eq!(parsed, Some((Some(11), 999)));
    }

    #[test]
    fn observation_summary_keeps_configured_candidates_separate_from_observed_tool() {
        let snapshot = RuntimeObservationSnapshot {
            session_id: 1,
            status: RuntimeObservationStatus::WaitingForGame,
            active: true,
            app_id: 4_162_040,
            started_at_unix_millis: 10,
            updated_at_unix_millis: 10,
            elapsed_millis: 0,
            scan_diagnostics: ProcfsScanDiagnostics::default(),
            process: None,
            ambiguous_process_count: 0,
            app_id_evidence: RuntimeObservationAppId {
                state: ObservationEvidenceState::Unavailable,
                observed_values: vec![],
                sources: vec![],
            },
            configured_runtime: ConfiguredRuntimeEvidence {
                per_game_state: "no_per_game_override".into(),
                per_game_internal_id: None,
                global_state: "unknown".into(),
                global_internal_id: None,
                source: None,
            },
            available_proton_tools: vec![ObservedRuntimeTool {
                internal_id: Some("exp".into()),
                display_name: "Proton Experimental".into(),
                version: None,
                path: "~/.steam/Proton Experimental".into(),
            }],
            observed_runtime: RuntimeObservationSelection {
                state: ObservationEvidenceState::Unknown,
                tool: None,
                evidence_source: None,
            },
            compatdata: RuntimeObservationPath {
                state: ObservationEvidenceState::NotObserved,
                observed_path: None,
                expected_path: None,
                evidence_source: None,
            },
            prefix: RuntimeObservationPath {
                state: ObservationEvidenceState::Unknown,
                observed_path: None,
                expected_path: None,
                evidence_source: None,
            },
            process_topology: vec![],
            process_topology_state: ProcessTopologyState::NotObserved,
            wineserver: WineServerObservation {
                state: ObservationEvidenceState::NotObserved,
                related_process_count: 0,
                evidence: "none".into(),
            },
            environment_evidence: ObservationEvidenceState::NotObserved,
            warnings: vec![],
        };
        let summary = snapshot.summary();
        assert!(summary.active);
        assert_eq!(
            summary.process_topology_state,
            ProcessTopologyState::NotObserved
        );
        assert_eq!(summary.runtime.state, ObservationEvidenceState::Unknown);
        assert_eq!(summary.status, RuntimeObservationStatus::WaitingForGame);

        let wire = serde_json::to_value(&snapshot).expect("snapshot serializes for Tauri");
        assert_eq!(wire["active"], true);
        assert_eq!(wire["status"], "waiting_for_game");
        assert_eq!(wire["process_topology_state"], "not_observed");
        assert!(wire.get("scan_diagnostics").is_some());
    }
}
