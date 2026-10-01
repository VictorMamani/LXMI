//! Non-injecting, fixed-artifact named-event context probe. Reuses the bridge executor.
use super::*;
use lxmi_xxmi::{inspect_prefix_dosdevices, map_linux_path, WindowsPathMapping};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextExperimentMode {
    SamePrefix,
    DifferentPrefixes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextEvidenceState {
    Observed,
    Derived,
    Unknown,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEvidence {
    pub state: ContextEvidenceState,
    pub value: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsRuntimeContext {
    pub proton_tool: ContextEvidence,
    pub compatdata_root: ContextEvidence,
    pub prefix: ContextEvidence,
    pub wineserver_identity: ContextEvidence,
    pub windows_session: ContextEvidence,
    pub drive_mapping: WindowsPathMapping,
    pub drive_mappings: BTreeMap<char, PathBuf>,
    pub drive_mappings_state: ContextEvidenceState,
    pub wineserver_candidates: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRelation {
    Same,
    Different,
    Unknown,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextIdentity {
    pub proton_tool: ContextRelation,
    pub compatdata: ContextRelation,
    pub prefix: ContextRelation,
    pub wineserver: ContextRelation,
    pub drive_mapping: ContextRelation,
    pub marker_path_mapping: ContextRelation,
}
fn relation(a: &ContextEvidence, b: &ContextEvidence) -> ContextRelation {
    if a.state == ContextEvidenceState::Unknown || b.state == ContextEvidenceState::Unknown {
        return ContextRelation::Unknown;
    }
    if a.state == ContextEvidenceState::Conflicting || b.state == ContextEvidenceState::Conflicting
    {
        return ContextRelation::Conflicting;
    }
    match (&a.value, &b.value) {
        (Some(a), Some(b)) if a == b => ContextRelation::Same,
        (Some(_), Some(_)) => ContextRelation::Different,
        _ => ContextRelation::Unknown,
    }
}
pub fn compare_runtime_contexts(
    a: &WindowsRuntimeContext,
    b: &WindowsRuntimeContext,
) -> ContextIdentity {
    ContextIdentity {
        proton_tool: relation(&a.proton_tool, &b.proton_tool),
        compatdata: relation(&a.compatdata_root, &b.compatdata_root),
        prefix: relation(&a.prefix, &b.prefix),
        wineserver: relation(&a.wineserver_identity, &b.wineserver_identity),
        drive_mapping: if a.drive_mappings_state != ContextEvidenceState::Observed
            || b.drive_mappings_state != ContextEvidenceState::Observed
            || a.drive_mappings.is_empty()
            || b.drive_mappings.is_empty()
        {
            ContextRelation::Unknown
        } else if a.drive_mappings == b.drive_mappings {
            ContextRelation::Same
        } else {
            ContextRelation::Different
        },
        marker_path_mapping: match (&a.drive_mapping.windows_path, &b.drive_mapping.windows_path) {
            (Some(a), Some(b)) if a == b => ContextRelation::Same,
            (Some(_), Some(_)) => ContextRelation::Different,
            _ => ContextRelation::Unknown,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRequirementState {
    Required,
    NotEstablished,
    NotRequired,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextJoinRequirement {
    pub property: String,
    pub state: ContextRequirementState,
    pub scope: String,
    pub evidence: String,
}
pub fn context_join_requirements() -> Vec<ContextJoinRequirement> {
    ["same_proton_tool", "same_prefix", "same_wineserver", "same_windows_session", "shared_drive_mapping"].into_iter().map(|property| ContextJoinRequirement {
        property: property.into(), state: ContextRequirementState::NotEstablished, scope: "future game loader".into(),
        evidence: "Named-event fixture observes object visibility only; loader necessity has not been tested".into(),
    }).collect()
}

/// Future game description only. There is deliberately no game executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameContextJoinPlan {
    pub game: String,
    pub same_proton_tool: String,
    pub same_prefix: String,
    pub same_wineserver: String,
    pub shared_drive_mapping: String,
    pub game_experiment_authorized: bool,
    pub blockers: Vec<String>,
}
impl Default for GameContextJoinPlan {
    fn default() -> Self {
        Self {
            game: "zenless-zone-zero".into(),
            same_proton_tool: "NotEstablished".into(),
            same_prefix: "NotEstablished".into(),
            same_wineserver: "NotEstablished".into(),
            shared_drive_mapping: "NotEstablished".into(),
            game_experiment_authorized: false,
            blockers: vec![
                "Fixture object visibility does not establish loader/game compatibility".into(),
                "Joining game compatdata can mutate its prefix; not authorized".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPeerResponse {
    pub protocol: u16,
    pub peer: String,
    pub pid: u32,
    pub nonce: String,
    pub path_readable: bool,
    pub probe_succeeded: bool,
    pub windows_error: u32,
    pub wine_prefix: String,
    pub compatdata: String,
    pub wineserver_binary: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextLabResult {
    pub mode: ContextExperimentMode,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub nonce: String,
    pub peer_a: ContextPeerResponse,
    pub peer_b: ContextPeerResponse,
    pub context_a: WindowsRuntimeContext,
    pub context_b: WindowsRuntimeContext,
    pub identity: ContextIdentity,
    pub probe_shared: bool,
    pub process_a: BridgeProcessSummary,
    pub process_b: BridgeProcessSummary,
    pub peer_hashes: Vec<String>,
    pub game_join_plan: GameContextJoinPlan,
    pub join_requirements: Vec<ContextJoinRequirement>,
    pub warnings: Vec<String>,
}
pub struct ContextLabConfig {
    pub managed_root: PathBuf,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub steam_client_install_path: PathBuf,
    pub mode: ContextExperimentMode,
    pub side_effects_acknowledged: bool,
}
/// Only trusted build outputs are staged; frontend cannot provide this source.
pub fn stage_context_peers(managed_root: &Path, build_root: &Path) -> Result<(), BridgeError> {
    let managed = checked_root(managed_root)?;
    let destination = managed.join("tests/context-join-v1/artifacts");
    ensure_controlled_directory(&managed, &destination)?;
    for peer in ["a", "b"] {
        let name = format!("lxmi-context-peer-{peer}.exe");
        let source = build_root.join(&name);
        let bytes = read_bounded_file(&source, HELPER_MAX_BYTES)
            .map_err(|e| error(BridgeErrorCode::HelperMissing, e.to_string()))?;
        if !bytes.starts_with(b"MZ") {
            return Err(error(
                BridgeErrorCode::HelperMissing,
                "Not a PE build output",
            ));
        }
        let hash = format!("{:x}", Sha256::digest(&bytes));
        for (path, content) in [
            (destination.join(&name), bytes),
            (
                destination.join(&name).with_extension("sha256"),
                hash.into_bytes(),
            ),
        ] {
            if path.exists() {
                if read_bounded_file(&path, HELPER_MAX_BYTES).ok().as_ref() != Some(&content) {
                    return Err(error(
                        BridgeErrorCode::HelperHashMismatch,
                        "Staged artifact differs; review it before replacing",
                    ));
                }
            } else {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .and_then(|mut f| f.write_all(&content))
                    .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
            }
        }
    }
    Ok(())
}
fn error(code: BridgeErrorCode, text: impl Into<String>) -> BridgeError {
    BridgeError::new(code, text)
}
fn checked_root(path: &Path) -> Result<PathBuf, BridgeError> {
    let root = path
        .canonicalize()
        .map_err(|e| error(BridgeErrorCode::UnsafePath, e.to_string()))?;
    if root.components().any(|part| {
        matches!(
            part.as_os_str()
                .to_string_lossy()
                .to_ascii_lowercase()
                .as_str(),
            "steamapps" | "compatdata" | "pfx"
        )
    }) || root.file_name().and_then(|x| x.to_str()) != Some("lxmi")
        || root.to_string_lossy().contains("/Steam/")
        || root.to_string_lossy().contains("/compatdata/")
    {
        return Err(error(
            BridgeErrorCode::UnsafePath,
            "Context Lab requires an LXMI root outside Steam/compatdata",
        ));
    }
    safe_existing_directory(&root, &root)?;
    Ok(root)
}
fn artifact(root: &Path, peer: &str) -> Result<PathBuf, BridgeError> {
    let file = root.join(format!(
        "tests/context-join-v1/artifacts/lxmi-context-peer-{peer}.exe"
    ));
    reject_symlink_components(root, &file)?;
    let bytes = read_bounded_file(&file, HELPER_MAX_BYTES)
        .map_err(|e| error(BridgeErrorCode::HelperMissing, e.to_string()))?;
    if !bytes.starts_with(b"MZ") {
        return Err(error(
            BridgeErrorCode::HelperMissing,
            "Context peer is not a PE artifact",
        ));
    }
    let hash = sha256_file(&file, HELPER_MAX_BYTES)
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let expected = read_bounded_file(&file.with_extension("sha256"), 128)
        .map_err(|e| error(BridgeErrorCode::HelperHashMismatch, e.to_string()))?;
    if hash != String::from_utf8_lossy(&expected).trim() {
        return Err(error(
            BridgeErrorCode::HelperHashMismatch,
            "Context peer hash differs from build inventory",
        ));
    }
    Ok(file)
}
fn invocation(
    config: &ContextLabConfig,
    root: &Path,
    compat: &Path,
    peer: &Path,
    args: Vec<OsString>,
) -> ProcessInvocation {
    let mut environment = BTreeMap::new();
    for name in ["PATH", "LANG"] {
        if let Some(value) = env::var_os(name) {
            environment.insert(name.into(), value);
        }
    }
    for (key, path) in [
        ("HOME", root.join("home")),
        ("XDG_CONFIG_HOME", root.join("config")),
        ("XDG_CACHE_HOME", root.join("cache")),
        ("XDG_DATA_HOME", root.join("data")),
        ("STEAM_COMPAT_DATA_PATH", compat.to_owned()),
        (
            "STEAM_COMPAT_CLIENT_INSTALL_PATH",
            config.steam_client_install_path.clone(),
        ),
    ] {
        environment.insert(key.into(), path.into_os_string());
    }
    environment.insert("WINEDEBUG".into(), "-all".into());
    let mut argv = vec!["runinprefix".into(), peer.as_os_str().to_owned()];
    argv.extend(args);
    ProcessInvocation {
        executable: config.explicit_runtime.proton_script.clone(),
        args: argv,
        environment,
        working_directory: root.to_owned(),
        input: Vec::new(),
        timeout: Duration::from_secs(45),
        output_limit: BRIDGE_OUTPUT_LIMIT_BYTES,
        terminate_group_on_timeout: false,
    }
}
fn execute(
    executor: &dyn BridgeProcessExecutor,
    call: &ProcessInvocation,
) -> Result<ProcessOutput, BridgeError> {
    executor.execute(call).map_err(|e| match e {
        ProcessFailure::TimedOut(out) => {
            error(BridgeErrorCode::Timeout, "Context peer exceeded timeout")
                .with_stderr(String::from_utf8_lossy(&out.stderr).into_owned())
        }
        ProcessFailure::Launch(e) | ProcessFailure::OutputRead(e) => {
            error(BridgeErrorCode::ProcessLaunchFailed, e.to_string())
        }
    })
}
fn parse(
    output: &ProcessOutput,
    nonce: &str,
    peer: &str,
) -> Result<ContextPeerResponse, BridgeError> {
    if !output.status.success() {
        return Err(error(BridgeErrorCode::NonZeroExit, "Context peer failed")
            .with_stderr(String::from_utf8_lossy(&output.stderr).into_owned()));
    }
    if output.stdout_truncated || output.stderr_truncated {
        return Err(error(
            BridgeErrorCode::OutputLimitExceeded,
            "Context output truncated",
        ));
    }
    let response: ContextPeerResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| error(BridgeErrorCode::MalformedResponse, e.to_string()))?;
    if response.protocol != 1
        || response.nonce != nonce
        || response.peer != peer
        || response.pid == 0
    {
        return Err(error(
            BridgeErrorCode::ProtocolMismatch,
            "Context peer identity/protocol/nonce mismatch",
        ));
    }
    Ok(response)
}
fn observed(value: String, source: &str) -> ContextEvidence {
    ContextEvidence {
        state: if value.is_empty() {
            ContextEvidenceState::Unknown
        } else {
            ContextEvidenceState::Observed
        },
        value: if value.is_empty() { None } else { Some(value) },
        source: source.into(),
    }
}
fn server_candidates(prefix: &str) -> Vec<u32> {
    use std::os::unix::fs::MetadataExt;
    let mut pids = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return pids;
    };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        if fs::metadata(&path).map(|m| m.uid()).ok() != Some(unsafe { libc::getuid() }) {
            continue;
        }
        let Ok(comm) = read_bounded_file(&path.join("comm"), 256) else {
            continue;
        };
        if !String::from_utf8_lossy(&comm)
            .trim()
            .starts_with("wineserver")
        {
            continue;
        }
        let Ok(bytes) = read_bounded_file(&path.join("environ"), 65536) else {
            continue;
        };
        if bytes.split(|b| *b == 0).any(|v| {
            v.strip_prefix(b"WINEPREFIX=").is_some_and(|v| {
                String::from_utf8_lossy(v).trim_end_matches('/') == prefix.trim_end_matches('/')
            })
        }) {
            pids.push(pid);
        }
    }
    pids.sort();
    pids
}
fn context(
    config: &ContextLabConfig,
    response: &ContextPeerResponse,
    mapping: WindowsPathMapping,
    candidates: Vec<u32>,
    expected_compatdata: &Path,
) -> WindowsRuntimeContext {
    let expected_prefix = expected_compatdata.join("pfx");
    let report = inspect_prefix_dosdevices(&expected_prefix);
    let mut prefix = observed(response.wine_prefix.clone(), "peer allowlisted WINEPREFIX");
    let mut compatdata = observed(
        response.compatdata.clone(),
        "peer allowlisted STEAM_COMPAT_DATA_PATH",
    );
    if !response.wine_prefix.is_empty()
        && Path::new(response.wine_prefix.trim_end_matches('/')) != expected_prefix
    {
        prefix.state = ContextEvidenceState::Conflicting;
    }
    if !response.compatdata.is_empty() && Path::new(&response.compatdata) != expected_compatdata {
        compatdata.state = ContextEvidenceState::Conflicting;
    }
    let mappings_state = if report.state == lxmi_xxmi::DosDevicesState::Found
        && report.issues.is_empty()
        && report
            .mappings
            .iter()
            .all(|m| m.state == lxmi_xxmi::DosDeviceState::Directory)
    {
        ContextEvidenceState::Observed
    } else {
        ContextEvidenceState::Unknown
    };
    WindowsRuntimeContext {
        proton_tool: ContextEvidence { state: ContextEvidenceState::Derived, value: Some(config.explicit_runtime.proton_script.display().to_string()), source: "Explicit lab invocation; not a game selection".into() },
        compatdata_root: compatdata, prefix,
        wineserver_identity: ContextEvidence { state: ContextEvidenceState::Unknown, value: None, source: "Same-UID wineserver with matching WINEPREFIX is a candidate, not a proven peer endpoint".into() },
        windows_session: ContextEvidence { state: ContextEvidenceState::Unknown, value: None, source: "Named event measures object visibility, not complete session identity".into() },
        drive_mappings_state: mappings_state,
        drive_mappings: report.mappings.into_iter().filter(|m| m.state==lxmi_xxmi::DosDeviceState::Directory).filter_map(|m| m.normalized_target.map(|p|(m.drive_letter,p))).collect(),
        drive_mapping: mapping, wineserver_candidates: candidates,
    }
}
fn summary(call: &ProcessInvocation, output: ProcessOutput) -> BridgeProcessSummary {
    BridgeProcessSummary {
        executable: call.executable.clone(),
        arguments: call
            .args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect(),
        exit_code: output.status.code().unwrap_or(-1),
        elapsed_millis: output.elapsed_millis,
        stdout_truncated: output.stdout_truncated,
        stderr_truncated: output.stderr_truncated,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}
pub fn run_context_lab(config: &ContextLabConfig) -> Result<ContextLabResult, BridgeError> {
    run_context_lab_with(
        config,
        std::sync::Arc::new(NativeProcessExecutor),
        server_candidates,
    )
}
fn run_context_lab_with(
    config: &ContextLabConfig,
    executor: std::sync::Arc<dyn BridgeProcessExecutor + Send + Sync>,
    observe_servers: fn(&str) -> Vec<u32>,
) -> Result<ContextLabResult, BridgeError> {
    if !config.side_effects_acknowledged {
        return Err(error(
            BridgeErrorCode::UnsafePrefixContext,
            "Acknowledge isolated Proton prefix writes",
        ));
    }
    let managed = checked_root(&config.managed_root)?;
    validate_proton_script(&config.explicit_runtime.proton_script)?;
    let a = artifact(&managed, "a")?;
    let b = artifact(&managed, "b")?;
    let nonce = format!("{}{}", random_nonce()?, random_nonce()?);
    let root = managed.join("tests/context-join-v1").join(&nonce);
    ensure_controlled_directory(&managed, &root)?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    for sub in [
        "home", "config", "cache", "data", "results", "compat-a", "compat-b",
    ] {
        ensure_controlled_directory(&managed, &root.join(sub))?;
    }
    let compat_a = root.join("compat-a");
    let compat_b = if config.mode == ContextExperimentMode::SamePrefix {
        compat_a.clone()
    } else {
        root.join("compat-b")
    };
    // Proton/Wine initializes only these fresh private prefixes; never copies a game prefix.
    for compat in [&compat_a, &compat_b] {
        let call = invocation(config, &root, compat, &b, vec!["bootstrap".into()]);
        let out = execute(executor.as_ref(), &call)?;
        if !out.status.success() {
            return Err(
                error(BridgeErrorCode::NonZeroExit, "Prefix bootstrap failed")
                    .with_stderr(String::from_utf8_lossy(&out.stderr).into_owned()),
            );
        }
    }
    let marker = root.join("results/marker");
    let ready = root.join("results/ready");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .and_then(|mut f| f.write_all(nonce.as_bytes()))
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let mut calls = Vec::new();
    let mut maps = Vec::new();
    for (compat, peer) in [(&compat_a, &a), (&compat_b, &b)] {
        let report = inspect_prefix_dosdevices(&compat.join("pfx"));
        let mapping = map_linux_path(&marker, &report);
        let ready_mapping = map_linux_path(&ready, &report);
        let windows = mapping.windows_path.clone().ok_or_else(|| {
            error(
                BridgeErrorCode::PathNotVisible,
                "Marker lacks an unambiguous dosdevices mapping",
            )
        })?;
        let ready_windows = ready_mapping.windows_path.ok_or_else(|| {
            error(
                BridgeErrorCode::PathNotVisible,
                "Ready marker lacks mapping",
            )
        })?;
        calls.push(invocation(
            config,
            &root,
            compat,
            peer,
            vec![nonce.clone().into(), windows.into(), ready_windows.into()],
        ));
        maps.push(mapping);
    }
    let call_a = calls[0].clone();
    let executor_a = executor.clone();
    let worker = thread::spawn(move || execute(executor_a.as_ref(), &call_a));
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready.exists() && !worker.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(100));
    }
    let started = read_bounded_file(&ready, 64)
        .ok()
        .is_some_and(|v| v == nonce.as_bytes());
    let candidates_a = observe_servers(&compat_a.join("pfx").display().to_string());
    let result_b = if started {
        execute(executor.as_ref(), &calls[1])
    } else {
        Err(error(
            BridgeErrorCode::Timeout,
            "Peer A did not publish its nonce-scoped ready marker",
        ))
    };
    let candidates_b = observe_servers(&compat_b.join("pfx").display().to_string());
    // Always join the bounded owned invocation, including error paths.
    let output_a = worker
        .join()
        .map_err(|_| error(BridgeErrorCode::ProcessLaunchFailed, "Peer A task panicked"))??;
    let output_b = result_b?;
    let peer_a = parse(&output_a, &nonce, "a")?;
    let peer_b = parse(&output_b, &nonce, "b")?;
    let context_a = context(config, &peer_a, maps.remove(0), candidates_a, &compat_a);
    let context_b = context(config, &peer_b, maps.remove(0), candidates_b, &compat_b);
    let identity = compare_runtime_contexts(&context_a, &context_b);
    Ok(ContextLabResult { mode: config.mode, explicit_runtime: config.explicit_runtime.clone(), nonce,
        probe_shared: peer_a.probe_succeeded && peer_b.probe_succeeded, peer_a, peer_b, context_a, context_b, identity,
        process_a: summary(&calls[0], output_a), process_b: summary(&calls[1], output_b),
        peer_hashes: vec![sha256_file(&a, HELPER_MAX_BYTES), sha256_file(&b, HELPER_MAX_BYTES)].into_iter().collect::<io::Result<_>>().map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?,
        game_join_plan: GameContextJoinPlan::default(), join_requirements: context_join_requirements(), warnings: vec!["Named-event visibility is not loader compatibility or proof of game wineserver identity".into(), "No wineserver cleanup signal; private context servers may exit naturally".into()] })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_never_becomes_same() {
        let unknown = observed(String::new(), "unavailable");
        assert_eq!(relation(&unknown, &unknown), ContextRelation::Unknown);
    }
    #[test]
    fn independent_prefix_relations() {
        assert_eq!(
            relation(&observed("a".into(), "peer"), &observed("a".into(), "peer")),
            ContextRelation::Same
        );
        assert_eq!(
            relation(&observed("a".into(), "peer"), &observed("b".into(), "peer")),
            ContextRelation::Different
        );
    }
    #[test]
    fn conflicting_evidence_is_preserved() {
        let mut a = observed("a".into(), "peer");
        a.state = ContextEvidenceState::Conflicting;
        assert_eq!(relation(&a, &a), ContextRelation::Conflicting);
    }
    #[test]
    fn game_plan_is_not_authorization() {
        assert!(!GameContextJoinPlan::default().game_experiment_authorized);
        assert_eq!(GameContextJoinPlan::default().same_prefix, "NotEstablished");
    }
    #[test]
    fn rejects_game_root_and_arbitrary_root() {
        assert!(checked_root(Path::new("/tmp")).is_err());
        assert!(checked_root(Path::new("/nonexistent/Steam/compatdata/4162040")).is_err());
    }
    struct FakePeers(std::sync::Mutex<Option<PathBuf>>);
    impl BridgeProcessExecutor for FakePeers {
        fn execute(&self, call: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
            use std::{ffi::OsStr, os::unix::process::ExitStatusExt};
            let compat = PathBuf::from(&call.environment[OsStr::new("STEAM_COMPAT_DATA_PATH")]);
            let bytes = if call.args[2] == "bootstrap" {
                let dos = compat.join("pfx/dosdevices");
                fs::create_dir_all(&dos).unwrap();
                fs::create_dir_all(compat.join("pfx/drive_c")).unwrap();
                if !dos.join("c:").exists() {
                    std::os::unix::fs::symlink("../drive_c", dos.join("c:")).unwrap();
                }
                if !dos.join("z:").exists() {
                    std::os::unix::fs::symlink("/", dos.join("z:")).unwrap();
                }
                b"{\"protocol\":1,\"bootstrap\":true}".to_vec()
            } else {
                let peer = if call.args[1].to_string_lossy().ends_with("peer-a.exe") {
                    "a"
                } else {
                    "b"
                };
                let nonce = call.args[2].to_string_lossy().to_string();
                let probe = if peer == "a" {
                    *self.0.lock().unwrap() = Some(compat.clone());
                    let ready = call.args[4]
                        .to_string_lossy()
                        .trim_start_matches("Z:")
                        .replace('\\', "/");
                    fs::write(ready, &nonce).unwrap();
                    true
                } else {
                    self.0.lock().unwrap().as_ref() == Some(&compat)
                };
                serde_json::to_vec(&ContextPeerResponse {
                    protocol: 1,
                    peer: peer.into(),
                    pid: 42,
                    nonce,
                    path_readable: true,
                    probe_succeeded: probe,
                    windows_error: if probe { 0 } else { 2 },
                    wine_prefix: compat.join("pfx").display().to_string(),
                    compatdata: compat.display().to_string(),
                    wineserver_binary: String::new(),
                })
                .unwrap()
            };
            Ok(ProcessOutput {
                status: ExitStatus::from_raw(0),
                elapsed_millis: 1,
                stdout: bytes,
                stderr: Vec::new(),
                stdout_truncated: false,
                stderr_truncated: false,
            })
        }
    }
    struct FixtureRoot(PathBuf);
    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
        }
    }
    fn fixture() -> FixtureRoot {
        let root = env::temp_dir()
            .join(format!("lxmi-context-fixture-{}", random_nonce().unwrap()))
            .join("lxmi");
        fs::create_dir_all(&root).unwrap();
        let build = root.join("build");
        fs::create_dir(&build).unwrap();
        for peer in ["a", "b"] {
            fs::write(
                build.join(format!("lxmi-context-peer-{peer}.exe")),
                b"MZ synthetic test PE fixture",
            )
            .unwrap();
        }
        stage_context_peers(&root, &build).unwrap();
        let script = root.join("proton");
        fs::write(&script, b"fixture never executed").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(script, fs::Permissions::from_mode(0o700)).unwrap();
        FixtureRoot(root)
    }
    fn config(root: &FixtureRoot, mode: ContextExperimentMode) -> ContextLabConfig {
        ContextLabConfig {
            managed_root: root.0.clone(),
            explicit_runtime: ExplicitBridgeRuntime {
                display_name: "FIXTURE".into(),
                version: None,
                proton_script: root.0.join("proton"),
            },
            steam_client_install_path: root.0.clone(),
            mode,
            side_effects_acknowledged: true,
        }
    }
    #[test]
    fn fake_same_prefix_and_paths_remain_separate_from_server_identity() {
        let root = fixture();
        let hash = sha256_file(&artifact(&root.0, "a").unwrap(), HELPER_MAX_BYTES).unwrap();
        let result = run_context_lab_with(
            &config(&root, ContextExperimentMode::SamePrefix),
            std::sync::Arc::new(FakePeers(Default::default())),
            |_| vec![123],
        )
        .unwrap();
        assert!(result.probe_shared);
        assert_eq!(result.identity.prefix, ContextRelation::Same);
        assert_eq!(result.identity.wineserver, ContextRelation::Unknown);
        assert_eq!(
            result.context_a.prefix.state,
            ContextEvidenceState::Observed
        );
        assert_eq!(
            result.context_a.proton_tool.state,
            ContextEvidenceState::Derived
        );
        assert_eq!(result.peer_hashes[0], hash);
        assert!(result
            .join_requirements
            .iter()
            .all(|r| r.state == ContextRequirementState::NotEstablished));
    }
    #[test]
    fn fake_different_prefixes_share_files_but_not_event() {
        let root = fixture();
        let result = run_context_lab_with(
            &config(&root, ContextExperimentMode::DifferentPrefixes),
            std::sync::Arc::new(FakePeers(Default::default())),
            |_| vec![],
        )
        .unwrap();
        assert!(!result.probe_shared);
        assert_eq!(result.identity.prefix, ContextRelation::Different);
        assert_eq!(result.identity.drive_mapping, ContextRelation::Different);
        assert_eq!(result.identity.marker_path_mapping, ContextRelation::Same);
        assert!(!result.game_join_plan.game_experiment_authorized);
    }
    #[test]
    fn missing_consent_never_calls_executor() {
        let root = fixture();
        let mut config = config(&root, ContextExperimentMode::SamePrefix);
        config.side_effects_acknowledged = false;
        assert_eq!(
            run_context_lab_with(
                &config,
                std::sync::Arc::new(FakePeers(Default::default())),
                |_| panic!("must not inspect host")
            )
            .unwrap_err()
            .code,
            BridgeErrorCode::UnsafePrefixContext
        );
    }
    #[test]
    fn symlink_artifact_rejected() {
        let root = fixture();
        let file = root
            .0
            .join("tests/context-join-v1/artifacts/lxmi-context-peer-a.exe");
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink(root.0.join("build/lxmi-context-peer-a.exe"), &file).unwrap();
        assert!(artifact(&root.0, "a").is_err());
    }
    #[test]
    fn modified_artifact_rejected() {
        let root = fixture();
        fs::write(
            root.0
                .join("tests/context-join-v1/artifacts/lxmi-context-peer-a.exe"),
            b"MZ altered",
        )
        .unwrap();
        assert_eq!(
            artifact(&root.0, "a").unwrap_err().code,
            BridgeErrorCode::HelperHashMismatch
        );
    }
    #[test]
    fn response_rejects_wrong_nonce_protocol_and_malformed_json() {
        use std::os::unix::process::ExitStatusExt;
        let mut output = ProcessOutput {
            status: ExitStatus::from_raw(0),
            elapsed_millis: 0,
            stdout: b"{}".to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        };
        assert_eq!(
            parse(&output, "nonce", "a").unwrap_err().code,
            BridgeErrorCode::MalformedResponse
        );
        let response = ContextPeerResponse {
            protocol: 2,
            peer: "a".into(),
            pid: 1,
            nonce: "other".into(),
            path_readable: true,
            probe_succeeded: true,
            windows_error: 0,
            wine_prefix: String::new(),
            compatdata: String::new(),
            wineserver_binary: String::new(),
        };
        output.stdout = serde_json::to_vec(&response).unwrap();
        assert_eq!(
            parse(&output, "nonce", "a").unwrap_err().code,
            BridgeErrorCode::ProtocolMismatch
        );
        output.stdout_truncated = true;
        assert_eq!(
            parse(&output, "nonce", "a").unwrap_err().code,
            BridgeErrorCode::OutputLimitExceeded
        );
        output.status = ExitStatus::from_raw(3 << 8);
        assert_eq!(
            parse(&output, "nonce", "a").unwrap_err().code,
            BridgeErrorCode::NonZeroExit
        );
    }
    #[test]
    fn context_invocation_never_signals_server_group() {
        let root = fixture();
        let config = config(&root, ContextExperimentMode::SamePrefix);
        let call = invocation(
            &config,
            &root.0,
            &root.0.join("compat"),
            &root.0.join("peer.exe"),
            vec!["bootstrap".into()],
        );
        assert!(!call.terminate_group_on_timeout);
        assert_eq!(call.args[0], "runinprefix");
        assert!(!call
            .environment
            .contains_key(std::ffi::OsStr::new("WINESERVERSOCKET")));
    }
    #[test]
    fn unknown_with_untrusted_value_cannot_match() {
        let evidence = ContextEvidence {
            state: ContextEvidenceState::Unknown,
            value: Some("prefix".into()),
            source: "untrusted fixture".into(),
        };
        assert_eq!(relation(&evidence, &evidence), ContextRelation::Unknown);
    }
}
