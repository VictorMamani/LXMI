//! Fixed fixture context boundary. A Windows PID is never accepted from IPC.
//! Session image, Windows creation time and owned-child report gate Inject without a shared pipe.
use super::*;
use crate::{context_lab as contexts, loader_lab as loaders};
use lxmi_xxmi::{inspect_prefix_dosdevices, map_linux_path};
use std::sync::Arc;

const RUNNER: &str = "lxmi-context-inject-runner.exe";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextInjectorResponse {
    pub protocol: u16,
    pub pid: u32,
    pub target_pid: u32,
    pub nonce: String,
    pub path_readable: bool,
    pub identity_bound: bool,
    pub target_identity: CrossPrefixTargetIdentity,
    pub inject_called: bool,
    pub inject_code: i32,
    pub windows_error: u32,
    pub wine_prefix: String,
    pub compatdata: String,
    #[serde(default)]
    pub named_event_visible: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextTargetResponse {
    pub protocol: u16,
    pub pid: u32,
    pub nonce: String,
    pub path_readable: bool,
    pub handshake_sent: bool,
    pub process_image: String,
    pub creation_time: String,
    pub ready: bool,
    pub wine_prefix: String,
    pub compatdata: String,
    #[serde(default)]
    pub event_received: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetIdentityResult {
    Verified,
    PidNotVisible,
    PidCollision,
    ImageMismatch,
    CreationTimeMismatch,
    AccessDenied,
    ProcessExited,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossPrefixTargetIdentity {
    pub expected_windows_pid: u32,
    pub observed_windows_pid: u32,
    pub expected_image: String,
    pub observed_image: String,
    // Decimal FILETIME strings preserve all 64 bits across JSON/JavaScript.
    pub expected_creation_time: String,
    pub observed_creation_time: String,
    pub nonce: String,
    pub result: TargetIdentityResult,
}
pub struct DirectInjectPreflightGate;
impl DirectInjectPreflightGate {
    pub fn allows(identity: &CrossPrefixTargetIdentity, nonce: &str) -> bool {
        identity.result == TargetIdentityResult::Verified
            && identity.nonce == nonce
            && identity.expected_windows_pid != 0
            && identity.expected_windows_pid == identity.observed_windows_pid
            && !identity.expected_image.is_empty()
            && identity
                .expected_image
                .eq_ignore_ascii_case(&identity.observed_image)
            && identity
                .expected_creation_time
                .parse::<u64>()
                .ok()
                .is_some_and(|v| v > 0)
            && identity.expected_creation_time == identity.observed_creation_time
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TargetSelfIdentity {
    protocol: u16,
    nonce: String,
    pid: u32,
    process_image: String,
    creation_time: String,
    ready: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectContextOutcome {
    InjectedSuccessfully,
    BlockedNotAttempted,
    LoaderFailure,
    MarkerNotVerified,
    IdentityProbeVerified,
    IdentityProbeNotVerified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderContextRequirementState {
    RequiredForTestedTopology,
    NotRequiredForTestedTopology,
    NotEstablished,
    Conflicting,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderContextRequirement {
    pub property: String,
    pub state: LoaderContextRequirementState,
    pub evidence: String,
}
pub fn loader_context_requirements() -> Vec<LoaderContextRequirement> {
    ["same_proton_tool", "same_prefix", "same_wineserver", "same_windows_session", "shared_filesystem_mapping"]
        .into_iter().map(|property| LoaderContextRequirement { property: property.into(), state: LoaderContextRequirementState::NotEstablished,
            evidence: "Fixture results do not establish a universal or ZZZ requirement; compare independently reviewed sessions".into() }).collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectContextResult {
    pub mode: ContextExperimentMode,
    pub nonce: String,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub outcome: DirectContextOutcome,
    pub target: ContextTargetResponse,
    pub injector: ContextInjectorResponse,
    pub context_target: WindowsRuntimeContext,
    pub context_injector: WindowsRuntimeContext,
    pub identity: ContextIdentity,
    pub marker_verified: bool,
    pub marker: Option<LoaderDllMarker>,
    pub process_target: BridgeProcessSummary,
    pub target_launcher_identity: Option<OwnedLinuxProcessIdentity>,
    pub process_injector: BridgeProcessSummary,
    pub source_hashes: BTreeMap<String, String>,
    pub upstream: Option<LoaderLabProvenance>,
    pub reentry_evidence: Option<crate::context_reentry::ReentryEvidence>,
    pub requirements: Vec<LoaderContextRequirement>,
    pub capabilities: Vec<LoaderCapability>,
    pub game_join_plan: GameContextJoinPlan,
    pub warnings: Vec<String>,
}
fn error(code: BridgeErrorCode, detail: impl Into<String>) -> BridgeError {
    BridgeError::new(code, detail)
}
fn validate_responses(
    nonce: &str,
    target: &ContextTargetResponse,
    runner: &ContextInjectorResponse,
) -> Result<(), BridgeError> {
    if target.protocol != 2
        || runner.protocol != 2
        || target.nonce != nonce
        || runner.nonce != nonce
        || target.pid == 0
        || runner.pid == 0
        || (runner.identity_bound
            && (runner.target_pid != target.pid || runner.target_pid == runner.pid))
        || !target.ready
        || runner.target_identity.expected_windows_pid != target.pid
        || !runner
            .target_identity
            .expected_image
            .eq_ignore_ascii_case(&target.process_image)
        || runner.target_identity.expected_creation_time != target.creation_time
        || runner.target_identity.nonce != nonce
        || runner.identity_bound
            != DirectInjectPreflightGate::allows(&runner.target_identity, nonce)
        || (runner.inject_called && !runner.identity_bound)
    {
        return Err(error(
            BridgeErrorCode::ProtocolMismatch,
            "Owned target/image/creation-time/nonce/protocol mismatch",
        ));
    }
    Ok(())
}
fn outcome(r: &ContextInjectorResponse, marker: bool) -> DirectContextOutcome {
    if !r.identity_bound {
        DirectContextOutcome::BlockedNotAttempted
    } else if !r.inject_called || r.inject_code != 0 {
        DirectContextOutcome::LoaderFailure
    } else if !marker {
        DirectContextOutcome::MarkerNotVerified
    } else {
        DirectContextOutcome::InjectedSuccessfully
    }
}
fn parse<T: serde::de::DeserializeOwned>(out: &ProcessOutput) -> Result<T, BridgeError> {
    if !out.status.success() {
        return Err(error(
            BridgeErrorCode::NonZeroExit,
            "Context fixture exited nonzero",
        )
        .with_stderr(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    if out.stdout_truncated || out.stderr_truncated {
        return Err(error(
            BridgeErrorCode::OutputLimitExceeded,
            "Context output truncated",
        ));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| error(BridgeErrorCode::MalformedResponse, e.to_string()))
}
fn peer(pid: u32, nonce: &str, readable: bool, prefix: &str, compat: &str) -> ContextPeerResponse {
    ContextPeerResponse {
        protocol: 1,
        peer: "context-inject-fixture".into(),
        pid,
        nonce: nonce.into(),
        path_readable: readable,
        probe_succeeded: false,
        windows_error: 0,
        wine_prefix: prefix.into(),
        compatdata: compat.into(),
        wineserver_binary: String::new(),
    }
}
/// Native service must pass a freshly reverified managed official component.
/// This API has no game path, process name, DLL or PID arguments from the frontend.
pub fn run_direct_context_lab(
    config: &ContextLabConfig,
    sources: &LoaderLabSources,
) -> Result<DirectContextResult, BridgeError> {
    run_with(
        config,
        sources,
        Arc::new(NativeProcessExecutor),
        |_| vec![], // No global process enumeration in this identity preflight.
    )
}
fn run_with(
    config: &ContextLabConfig,
    sources: &LoaderLabSources,
    executor: Arc<dyn BridgeProcessExecutor + Send + Sync>,
    observe_servers: fn(&str) -> Vec<u32>,
) -> Result<DirectContextResult, BridgeError> {
    run_core(
        config,
        &sources.application_root,
        &sources.protected_roots,
        Some(sources),
        executor,
        observe_servers,
    )
}
pub(crate) fn run_reentry_core(
    config: &ContextLabConfig,
    application_root: &Path,
    protected_roots: &[PathBuf],
) -> Result<DirectContextResult, BridgeError> {
    run_core(
        config,
        application_root,
        protected_roots,
        None,
        Arc::new(NativeProcessExecutor),
        |_| vec![],
    )
}
fn run_core(
    config: &ContextLabConfig,
    application_root: &Path,
    protected_roots: &[PathBuf],
    sources: Option<&LoaderLabSources>,
    executor: Arc<dyn BridgeProcessExecutor + Send + Sync>,
    observe_servers: fn(&str) -> Vec<u32>,
) -> Result<DirectContextResult, BridgeError> {
    let reentry = sources.is_none();
    let runner_name = if reentry {
        "lxmi-context-reentry-helper.exe"
    } else {
        RUNNER
    };
    if !config.side_effects_acknowledged {
        return Err(error(
            BridgeErrorCode::UnsafePrefixContext,
            "Acknowledge isolated fixture prefix writes",
        ));
    }
    let managed = contexts::checked_root(&config.managed_root)?;
    use std::os::unix::fs::MetadataExt;
    reject_symlink_components(Path::new("/"), &config.managed_root)?;
    if fs::metadata(&managed).map(|m| m.uid()).ok() != Some(unsafe { libc::getuid() })
        || managed.components().any(|c| {
            c.as_os_str()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("zenless")
        })
    {
        return Err(error(
            BridgeErrorCode::UnsafePath,
            "Fixture root must be same-UID storage outside the game",
        ));
    }

    loaders::reject_protected_overlap(&managed, protected_roots)?;
    if let Some(sources) = sources {
        loaders::validate_provenance(&sources.provenance)?;
    }
    validate_proton_script(&config.explicit_runtime.proton_script)?;
    let app = application_root
        .canonicalize()
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let build = app.join("target/x86_64-pc-windows-gnu/release/lxmi-loader-lab");
    let loader = if let Some(sources) = sources {
        let path =
            loaders::validate_package_loader_source(&managed, &sources.upstream_loader_path)?;
        if loaders::safe_artifact_hash(&managed, &path)? != sources.expected_upstream_loader_sha256
        {
            return Err(error(
                BridgeErrorCode::LoaderProvenanceInvalid,
                "Official loader hash mismatch",
            ));
        }
        Some(path)
    } else {
        None
    };
    let nonce = random_nonce()?;
    let session = managed
        .join(if reentry {
            "tests/context-reentry-v1"
        } else {
            "tests/direct-inject-boundary"
        })
        .join(&nonce);
    ensure_controlled_directory(&managed, &session)?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&session, fs::Permissions::from_mode(0o700))
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let root = session.join("loader-v1");
    for sub in [
        "target", "runner", "runtime", "results", "home", "config", "cache", "data", "compat-a",
        "compat-b",
    ] {
        ensure_controlled_directory(&managed, &root.join(sub))?;
    }
    let mut entries = vec![
        (
            build.join(LOADER_LAB_TARGET),
            root.join("target").join(LOADER_LAB_TARGET),
            app.as_path(),
        ),
        (
            build.join(runner_name),
            root.join("runner").join(runner_name),
            app.as_path(),
        ),
    ];
    if let Some(loader) = loader {
        entries.extend([
            (
                build.join(LOADER_LAB_TEST_DLL),
                root.join("runtime").join(LOADER_LAB_TEST_DLL),
                app.as_path(),
            ),
            (
                loader,
                root.join("runtime").join(UPSTREAM_LOADER_DLL),
                managed.as_path(),
            ),
        ]);
    }
    let mut source_hashes = BTreeMap::new();
    for (source, destination, source_root) in entries {
        let digest = loaders::safe_artifact_hash(source_root, &source)?;
        let bytes = read_bounded_file(&source, HELPER_MAX_BYTES)
            .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .and_then(|mut f| f.write_all(&bytes))
            .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
        if sha256_file(&destination, HELPER_MAX_BYTES).ok().as_ref() != Some(&digest) {
            return Err(error(
                BridgeErrorCode::HelperHashMismatch,
                "Staged fixture differs",
            ));
        }
        source_hashes.insert(
            destination
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            digest,
        );
    }
    // Metadata remains separate from runtime bytes; source packages are unchanged.
    let metadata = serde_json::to_vec(&(&source_hashes, &sources.map(|s| &s.provenance)))
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    fs::write(session.join("fixture-manifest.json"), metadata)
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let compat_a = root.join("compat-a");
    let compat_b = if config.mode == ContextExperimentMode::SamePrefix {
        compat_a.clone()
    } else {
        root.join("compat-b")
    };
    let runner = root.join("runner").join(runner_name);
    for compat in [&compat_a, &compat_b] {
        let call = contexts::invocation(config, &root, compat, &runner, vec!["bootstrap".into()]);
        let out = contexts::execute(executor.as_ref(), &call)?;
        let value: serde_json::Value = parse(&out)?;
        if value.get("bootstrap") != Some(&serde_json::Value::Bool(true)) {
            return Err(error(
                BridgeErrorCode::ProtocolMismatch,
                "Bootstrap response invalid",
            ));
        }
    }
    let prefix_before = if reentry {
        Some(crate::context_reentry::prefix_snapshot(
            &compat_a.join("pfx"),
        ))
    } else {
        None
    };
    let marker = root.join("results/filesystem-marker");
    fs::write(&marker, &nonce).map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let maps = [&compat_a, &compat_b]
        .map(|compat| map_linux_path(&marker, &inspect_prefix_dosdevices(&compat.join("pfx"))));
    if maps.iter().any(|m| m.windows_path.is_none()) {
        return Err(error(
            BridgeErrorCode::PathNotVisible,
            "Fixture marker mapping unavailable",
        ));
    }
    let target = root.join("target").join(LOADER_LAB_TARGET);
    let mut call_a = contexts::invocation(
        config,
        &root,
        &compat_a,
        &target,
        vec![
            if reentry {
                "context-reentry".into()
            } else {
                "context-boundary".into()
            },
            nonce.clone().into(),
        ],
    );
    call_a
        .environment
        .insert("LXMI_LOADER_TEST_NONCE".into(), nonce.clone().into());
    let call_b = contexts::invocation(
        config,
        &root,
        &compat_b,
        &runner,
        vec![nonce.clone().into()],
    );
    let ownership = Arc::new(std::sync::Mutex::new(None));
    call_a.ownership = Some(ownership.clone());
    let executor_a = executor.clone();
    let child_call = call_a.clone();
    let worker = thread::spawn(move || contexts::execute(executor_a.as_ref(), &child_call));
    let ready = root.join("results/target-ready");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready.exists() && !worker.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(100));
    }
    let valid_ready = read_bounded_file(&ready, 32)
        .ok()
        .is_some_and(|b| b == nonce.as_bytes());
    let expected_image =
        map_linux_path(&target, &inspect_prefix_dosdevices(&compat_a.join("pfx"))).windows_path;
    let self_identity = read_bounded_file(&root.join("results/target-identity.json"), 65536)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<TargetSelfIdentity>(&bytes).ok());
    let owned_child = ownership.lock().expect("owned identity").clone();
    let candidates_a = observe_servers(&compat_a.join("pfx").display().to_string());
    let reentry_evidence = if reentry {
        // A has published ready; B has not been invoked. Deliberate late join.
        if !valid_ready || worker.is_finished() {
            None
        } else {
            thread::sleep(Duration::from_millis(300));
            Some(crate::context_reentry::collect_evidence(
                &compat_a.join("pfx"),
                &compat_b.join("pfx"),
                prefix_before.unwrap_or_default(),
            ))
        }
    } else {
        None
    };
    let output_b = (|| {
        let identity = self_identity.as_ref().ok_or_else(|| {
            error(
                BridgeErrorCode::MalformedResponse,
                "Target self identity unavailable",
            )
        })?;
        let image = expected_image.as_deref().ok_or_else(|| {
            error(
                BridgeErrorCode::PathNotVisible,
                "Target mapping unavailable",
            )
        })?;
        let time = identity.creation_time.parse::<u64>().map_err(|_| {
            error(
                BridgeErrorCode::ProtocolMismatch,
                "Invalid target creation time",
            )
        })?;
        if !valid_ready
            || identity.protocol != 2
            || identity.nonce != nonce
            || !identity.ready
            || identity.pid == 0
            || time == 0
            || !identity.process_image.eq_ignore_ascii_case(image)
            || worker.is_finished()
            || !owned_child
                .as_ref()
                .is_some_and(|child| child.pid > 0 && child.start_time_ticks.is_some())
        {
            return Err(error(
                BridgeErrorCode::ProtocolMismatch,
                "Session/path/creation/owned-child correlation failed before PID query",
            ));
        }
        let utf16: Vec<u16> = image.encode_utf16().collect();
        if utf16.len() >= 4096 || utf16.contains(&0) {
            return Err(error(
                BridgeErrorCode::UnsafePath,
                "Invalid Windows fixture path",
            ));
        }
        let mut request = Vec::new();
        request.extend_from_slice(&2u32.to_le_bytes());
        request.extend_from_slice(&identity.pid.to_le_bytes());
        request.extend_from_slice(&time.to_le_bytes());
        request.extend_from_slice(nonce.as_bytes());
        request.extend_from_slice(&(utf16.len() as u32).to_le_bytes());
        for value in utf16 {
            request.extend_from_slice(&value.to_le_bytes());
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("results/identity-request.bin"))
            .and_then(|mut file| file.write_all(&request))
            .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
        contexts::execute(executor.as_ref(), &call_b)
    })();
    let candidates_b = observe_servers(&compat_b.join("pfx").display().to_string());

    // DLL marker creation is asynchronous. Keep the bound fixture alive until
    // its worker has published evidence, but never wait indefinitely.
    if output_b
        .as_ref()
        .ok()
        .and_then(|out| parse::<ContextInjectorResponse>(out).ok())
        .is_some_and(|r| r.inject_called && r.inject_code == 0)
    {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !root.join("results/loader-result.json").exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(25));
        }
    }
    // Only the nonce-scoped target consumes this private stop marker. No server signals.
    let stop_result = fs::write(root.join("results/stop"), &nonce);
    let output_a = worker
        .join()
        .map_err(|_| error(BridgeErrorCode::ProcessLaunchFailed, "Target task panicked"))??;
    stop_result.map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let output_b = output_b?;
    let reentry_evidence = reentry_evidence.map(|mut e| {
        e.prefix_after = crate::context_reentry::prefix_snapshot(&compat_a.join("pfx"));
        e.endpoint_a_after_exit = crate::context_reentry::endpoint(&compat_a.join("pfx"));
        e
    });
    let target: ContextTargetResponse = parse(&output_a)?;
    let injector: ContextInjectorResponse = parse(&output_b)?;
    validate_responses(&nonce, &target, &injector)?;
    let ready_identity = self_identity
        .ok_or_else(|| error(BridgeErrorCode::ProtocolMismatch, "Missing self identity"))?;
    if ready_identity.pid != target.pid
        || ready_identity.creation_time != target.creation_time
        || ready_identity.process_image != target.process_image
    {
        return Err(error(
            BridgeErrorCode::ProtocolMismatch,
            "Target final response differs from ready identity",
        ));
    }
    let marker_path = root.join("results/loader-result.json");
    let marker = match read_bounded_file(&marker_path, 16384) {
        Ok(bytes) => Some(
            serde_json::from_slice::<LoaderDllMarker>(&bytes)
                .map_err(|e| error(BridgeErrorCode::MalformedResponse, e.to_string()))?,
        ),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(error(BridgeErrorCode::Io, e.to_string())),
    };
    let target_map = map_linux_path(
        &root.join("target").join(LOADER_LAB_TARGET),
        &inspect_prefix_dosdevices(&compat_a.join("pfx")),
    );
    let dll_map = map_linux_path(
        &root.join("runtime").join(LOADER_LAB_TEST_DLL),
        &inspect_prefix_dosdevices(&compat_a.join("pfx")),
    );
    let marker_verified = marker.as_ref().is_some_and(|m| {
        m.protocol == 1
            && m.loaded
            && m.nonce == nonce
            && m.process == LOADER_LAB_TARGET
            && Some(m.process_path_windows.to_ascii_lowercase())
                == target_map.windows_path.map(|p| p.to_ascii_lowercase())
            && Some(m.dll_path_windows.to_ascii_lowercase())
                == dll_map.windows_path.map(|p| p.to_ascii_lowercase())
    });
    let context_target = contexts::context(
        config,
        &peer(
            target.pid,
            &nonce,
            target.path_readable,
            &target.wine_prefix,
            &target.compatdata,
        ),
        maps[0].clone(),
        candidates_a,
        &compat_a,
    );
    let context_injector = contexts::context(
        config,
        &peer(
            injector.pid,
            &nonce,
            injector.path_readable,
            &injector.wine_prefix,
            &injector.compatdata,
        ),
        maps[1].clone(),
        candidates_b,
        &compat_b,
    );
    let identity = compare_runtime_contexts(&context_target, &context_injector);
    let identity_bound = injector.identity_bound;
    let mut game_join_plan = GameContextJoinPlan::default();
    game_join_plan.loader_context_evidence.push(format!(
        "Fixture {:?}: {:?}; not a ZZZ test; non-Verified process identity blocks Inject",
        config.mode,
        outcome(&injector, marker_verified)
    ));
    game_join_plan.blockers.push("A helper outside the game Windows context has no demonstrated safe target binding; game-prefix join remains unauthorized".into());
    Ok(DirectContextResult { mode:config.mode,nonce,explicit_runtime:config.explicit_runtime.clone(),outcome:if reentry { if identity_bound { DirectContextOutcome::IdentityProbeVerified } else { DirectContextOutcome::IdentityProbeNotVerified } } else { outcome(&injector,marker_verified) },target,injector,
        context_target,context_injector,identity,marker_verified,marker,process_target:contexts::summary(&call_a,output_a),target_launcher_identity:owned_child,process_injector:contexts::summary(&call_b,output_b),source_hashes,upstream:sources.map(|s| s.provenance.clone()),reentry_evidence,requirements:loader_context_requirements(), capabilities: vec![LoaderCapability {
            name: if reentry { "ContextReentryIdentityProbe" } else if config.mode == ContextExperimentMode::SamePrefix { "DirectInjectFixtureSamePrefix" } else { "DirectInjectFixtureCrossPrefix" }.into(),
            state: if marker_verified || (reentry && identity_bound) { LoaderCapabilityState::Verified } else if !identity_bound { LoaderCapabilityState::Blocked } else { LoaderCapabilityState::Failed },
            evidence: "Only this nonce-scoped fixture; Exact fixture image/time/session identity gates the upstream call; no game compatibility established".into(),
        }],
        game_join_plan,warnings:vec!["BlockedNotAttempted is an identity preflight result, NOT an observed upstream Inject failure".into(),"No game compatibility established; Hook blocked; strategy NoDecision".into(),"Wine servers are not signalled; they may exit naturally".into()] })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn identity() -> CrossPrefixTargetIdentity {
        CrossPrefixTargetIdentity {
            expected_windows_pid: 220,
            observed_windows_pid: 220,
            expected_image: "Z:\\fixture\\target.exe".into(),
            observed_image: "Z:\\fixture\\target.exe".into(),
            expected_creation_time: "123456789".into(),
            observed_creation_time: "123456789".into(),
            nonce: "a".repeat(32),
            result: TargetIdentityResult::Verified,
        }
    }
    fn runner() -> ContextInjectorResponse {
        ContextInjectorResponse {
            protocol: 2,
            pid: 200,
            target_pid: 220,
            nonce: "a".repeat(32),
            path_readable: true,
            identity_bound: true,
            named_event_visible: None,
            target_identity: identity(),
            inject_called: true,
            inject_code: 0,
            windows_error: 0,
            wine_prefix: String::new(),
            compatdata: String::new(),
        }
    }
    fn target() -> ContextTargetResponse {
        ContextTargetResponse {
            protocol: 2,
            pid: 220,
            nonce: "a".repeat(32),
            path_readable: true,
            event_received: None,
            handshake_sent: false,
            process_image: "Z:\\fixture\\target.exe".into(),
            creation_time: "123456789".into(),
            ready: true,
            wine_prefix: String::new(),
            compatdata: String::new(),
        }
    }
    #[test]
    fn only_bound_owned_pid_can_be_used() {
        let mut r = runner();
        r.identity_bound = false;
        assert!(validate_responses(&r.nonce, &target(), &r).is_err());
    }
    #[test]
    fn wrong_nonce_rejected() {
        let mut r = runner();
        r.nonce = "b".repeat(32);
        assert!(validate_responses(&target().nonce, &target(), &r).is_err());
    }
    #[test]
    fn wrong_server_pid_rejected() {
        let mut r = runner();
        r.target_pid = 222;
        assert!(validate_responses(&r.nonce, &target(), &r).is_err());
    }
    #[test]
    fn filesystem_does_not_prove_target_visibility() {
        let mut r = runner();
        r.identity_bound = false;
        r.inject_called = false;
        assert_eq!(
            outcome(&r, false),
            DirectContextOutcome::BlockedNotAttempted
        );
    }
    #[test]
    fn old_marker_does_not_override_loader_failure() {
        let mut r = runner();
        r.inject_code = 110;
        assert_eq!(outcome(&r, true), DirectContextOutcome::LoaderFailure);
    }
    #[test]
    fn zero_return_needs_verified_marker() {
        assert_eq!(
            outcome(&runner(), false),
            DirectContextOutcome::MarkerNotVerified
        );
        assert_eq!(
            outcome(&runner(), true),
            DirectContextOutcome::InjectedSuccessfully
        );
    }
    #[test]
    fn lab_result_never_establishes_game_requirements() {
        assert!(loader_context_requirements()
            .iter()
            .all(|r| r.state == LoaderContextRequirementState::NotEstablished));
        assert!(!GameContextJoinPlan::default().game_experiment_authorized);
    }
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
        }
    }
    fn fixture_config(
        mode: ContextExperimentMode,
    ) -> (Fixture, ContextLabConfig, LoaderLabSources) {
        use std::os::unix::fs::PermissionsExt;
        let managed = env::temp_dir()
            .join(format!("lxmi-boundary-test-{}", random_nonce().unwrap()))
            .join("lxmi");
        let app = managed.join("build");
        let build = app.join("target/x86_64-pc-windows-gnu/release/lxmi-loader-lab");
        fs::create_dir_all(&build).unwrap();
        for name in [
            RUNNER,
            LOADER_LAB_TARGET,
            LOADER_LAB_TEST_DLL,
            "lxmi-context-reentry-helper.exe",
        ] {
            fs::write(build.join(name), b"MZ fixture bytes, never executed").unwrap();
        }
        let loader = managed.join("packages/xxmi/fixture/payload/3dmloader.dll");
        fs::create_dir_all(loader.parent().unwrap()).unwrap();
        fs::write(&loader, b"MZ synthetic upstream fixture").unwrap();
        let proton = app.join("proton");
        fs::write(&proton, b"fake, not executed").unwrap();
        fs::set_permissions(&proton, fs::Permissions::from_mode(0o700)).unwrap();
        let config = ContextLabConfig {
            managed_root: managed.clone(),
            explicit_runtime: ExplicitBridgeRuntime {
                display_name: "FIXTURE".into(),
                version: None,
                proton_script: proton,
            },
            steam_client_install_path: app.clone(),
            mode,
            side_effects_acknowledged: true,
        };
        let sources = LoaderLabSources {
            application_root: app,
            upstream_loader_path: loader.clone(),
            expected_upstream_loader_sha256: sha256_file(&loader, HELPER_MAX_BYTES).unwrap(),
            provenance: LoaderLabProvenance {
                package_id: "a".repeat(64),
                repository: "SpectrumQT/XXMI-Libs-Package".into(),
                release_id: 1,
                tag: "FIXTURE".into(),
                commit: "abcdef0".into(),
                signature_verified: true,
                component_signatures_verified: true,
            },
            protected_roots: vec![],
        };
        (Fixture(managed), config, sources)
    }
    #[derive(Default)]
    struct Fake(std::sync::Mutex<Option<PathBuf>>);
    impl BridgeProcessExecutor for Fake {
        fn execute(&self, call: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
            use std::{ffi::OsStr, os::unix::process::ExitStatusExt};
            assert!(!call.terminate_group_on_timeout);
            assert_eq!(call.args[0], "runinprefix");
            let root = &call.working_directory;
            let compat = PathBuf::from(&call.environment[OsStr::new("STEAM_COMPAT_DATA_PATH")]);
            let bytes = if call.args[2] == "bootstrap" {
                let dos = compat.join("pfx/dosdevices");
                fs::create_dir_all(&dos).unwrap();
                fs::create_dir_all(compat.join("pfx/drive_c")).unwrap();
                if !dos.join("z:").exists() {
                    std::os::unix::fs::symlink("/", dos.join("z:")).unwrap();
                    std::os::unix::fs::symlink("../drive_c", dos.join("c:")).unwrap();
                }
                b"{\"protocol\":1,\"bootstrap\":true}".to_vec()
            } else if call.args[2] == "context-boundary" || call.args[2] == "context-reentry" {
                let reentry = call.args[2] == "context-reentry";
                let nonce = call.args[3].to_string_lossy().to_string();
                *self.0.lock().unwrap() = Some(compat.clone());
                if let Some(observer) = &call.ownership {
                    *observer.lock().unwrap() = Some(OwnedLinuxProcessIdentity {
                        pid: 42,
                        start_time_ticks: Some(100),
                    });
                }
                let image = format!(
                    "Z:{}",
                    root.join("target")
                        .join(LOADER_LAB_TARGET)
                        .display()
                        .to_string()
                        .replace('/', "\\")
                );
                fs::write(
                    root.join("results/target-identity.json"),
                    serde_json::to_vec(&TargetSelfIdentity {
                        protocol: 2,
                        nonce: nonce.clone(),
                        pid: 220,
                        process_image: image.clone(),
                        creation_time: "123456789".into(),
                        ready: true,
                    })
                    .unwrap(),
                )
                .unwrap();
                fs::write(root.join("results/target-ready"), &nonce).unwrap();
                let end = Instant::now() + Duration::from_secs(5);
                while !root.join("results/stop").exists() && Instant::now() < end {
                    thread::sleep(Duration::from_millis(5));
                }
                serde_json::to_vec(&ContextTargetResponse {
                    protocol: 2,
                    pid: 220,
                    nonce,
                    path_readable: true,
                    event_received: if reentry {
                        Some(root.join("results/event-signalled").exists())
                    } else {
                        None
                    },
                    handshake_sent: false,
                    process_image: image,
                    creation_time: "123456789".into(),
                    ready: true,
                    wine_prefix: compat.join("pfx").display().to_string(),
                    compatdata: compat.display().to_string(),
                })
                .unwrap()
            } else {
                let nonce = call.args[2].to_string_lossy().to_string();
                let same = self.0.lock().unwrap().as_ref() == Some(&compat);
                let reentry = call.args[1]
                    .to_string_lossy()
                    .ends_with("lxmi-context-reentry-helper.exe");
                assert!(
                    root.join("results/target-ready").exists(),
                    "helper cannot start before target ready"
                );
                if reentry && same {
                    fs::write(root.join("results/event-signalled"), &nonce).unwrap();
                }
                if same && !reentry {
                    let windows = |path: PathBuf| {
                        format!("Z:{}", path.display().to_string().replace('/', "\\"))
                    };
                    let marker = LoaderDllMarker {
                        protocol: 1,
                        loaded: true,
                        nonce: nonce.clone(),
                        process: LOADER_LAB_TARGET.into(),
                        process_path_windows: windows(root.join("target").join(LOADER_LAB_TARGET)),
                        dll_path_windows: windows(root.join("runtime").join(LOADER_LAB_TEST_DLL)),
                    };
                    fs::write(
                        root.join("results/loader-result.json"),
                        serde_json::to_vec(&marker).unwrap(),
                    )
                    .unwrap();
                }
                // Different server tables may use the same numeric Windows PID.
                serde_json::to_vec(&ContextInjectorResponse {
                    protocol: 2,
                    pid: if same { 236 } else { 220 },
                    target_pid: 220,
                    target_identity: CrossPrefixTargetIdentity {
                        expected_windows_pid: 220,
                        observed_windows_pid: 220,
                        expected_image: format!(
                            "Z:{}",
                            root.join("target")
                                .join(LOADER_LAB_TARGET)
                                .display()
                                .to_string()
                                .replace('/', "\\")
                        ),
                        observed_image: format!(
                            "Z:{}",
                            root.join(if same { "target" } else { "runner" })
                                .join(if same { LOADER_LAB_TARGET } else { RUNNER })
                                .display()
                                .to_string()
                                .replace('/', "\\")
                        ),
                        expected_creation_time: "123456789".into(),
                        observed_creation_time: if same { "123456789" } else { "987654321" }.into(),
                        nonce: nonce.clone(),
                        result: if same {
                            TargetIdentityResult::Verified
                        } else {
                            TargetIdentityResult::PidCollision
                        },
                    },
                    nonce,
                    path_readable: true,
                    named_event_visible: if reentry { Some(same) } else { None },
                    identity_bound: same,
                    inject_called: same && !reentry,
                    inject_code: if same { 0 } else { -1 },
                    windows_error: 0,
                    wine_prefix: compat.join("pfx").display().to_string(),
                    compatdata: compat.display().to_string(),
                })
                .unwrap()
            };
            Ok(ProcessOutput {
                status: ExitStatus::from_raw(0),
                elapsed_millis: 1,
                stdout: bytes,
                stderr: vec![],
                stdout_truncated: false,
                stderr_truncated: false,
            })
        }
    }
    #[test]
    fn fake_same_context_load_preserves_sources() {
        let (_fixture, config, sources) = fixture_config(ContextExperimentMode::SamePrefix);
        let before = sha256_file(&sources.upstream_loader_path, HELPER_MAX_BYTES).unwrap();
        let result = run_with(&config, &sources, Arc::new(Fake::default()), |_| vec![42]).unwrap();
        assert_eq!(result.outcome, DirectContextOutcome::InjectedSuccessfully);
        assert!(result.marker_verified);
        assert_eq!(result.identity.prefix, ContextRelation::Same);
        assert_eq!(result.identity.wineserver, ContextRelation::Unknown);
        assert_eq!(
            sha256_file(&sources.upstream_loader_path, HELPER_MAX_BYTES).unwrap(),
            before
        );
    }
    #[test]
    fn fake_cross_context_collision_never_calls_inject() {
        let (_fixture, config, sources) = fixture_config(ContextExperimentMode::DifferentPrefixes);
        let result = run_with(&config, &sources, Arc::new(Fake::default()), |_| vec![]).unwrap();
        assert_eq!(result.target.pid, result.injector.pid);
        assert!(!result.injector.inject_called);
        assert!(!result.marker_verified);
        assert_eq!(result.outcome, DirectContextOutcome::BlockedNotAttempted);
        assert_eq!(result.identity.prefix, ContextRelation::Different);
        assert_eq!(result.identity.marker_path_mapping, ContextRelation::Same);
        assert_eq!(result.capabilities[0].state, LoaderCapabilityState::Blocked);
    }
    #[test]
    fn rejects_game_prefix_and_zzz_storage_before_executor() {
        let (_fixture, mut config, sources) = fixture_config(ContextExperimentMode::SamePrefix);
        let managed = config.managed_root.clone();
        for sub in [
            "steamapps/compatdata/4162040/lxmi",
            "Zenless Zone Zero/lxmi",
        ] {
            let path = managed.join(sub);
            fs::create_dir_all(&path).unwrap();
            config.managed_root = path;
            assert!(
                run_with(&config, &sources, Arc::new(Fake::default()), |_| panic!(
                    "must not inspect procfs"
                ))
                .is_err()
            );
        }
    }
    #[test]
    fn consent_rejection_has_no_execution() {
        let (_fixture, mut config, sources) = fixture_config(ContextExperimentMode::SamePrefix);
        config.side_effects_acknowledged = false;
        assert!(
            run_with(&config, &sources, Arc::new(Fake::default()), |_| panic!(
                "must not inspect procfs"
            ))
            .is_err()
        );
    }
    #[test]
    fn correct_pid_image_time_is_allowed() {
        assert!(DirectInjectPreflightGate::allows(
            &identity(),
            &"a".repeat(32)
        ));
    }
    #[test]
    fn same_pid_wrong_image_and_session_path_are_rejected() {
        for path in ["Z:\\other\\target.exe", "Z:\\fixture\\runner.exe"] {
            let mut i = identity();
            i.observed_image = path.into();
            assert!(!DirectInjectPreflightGate::allows(&i, &i.nonce));
        }
    }
    #[test]
    fn same_pid_wrong_creation_time_is_rejected() {
        let mut i = identity();
        i.observed_creation_time = "987654321".into();
        assert!(!DirectInjectPreflightGate::allows(&i, &i.nonce));
    }
    #[test]
    fn correct_pid_with_wrong_nonce_is_rejected() {
        assert!(!DirectInjectPreflightGate::allows(
            &identity(),
            &"b".repeat(32)
        ));
    }
    #[test]
    fn every_non_verified_state_blocks_inject() {
        for state in [
            TargetIdentityResult::PidNotVisible,
            TargetIdentityResult::PidCollision,
            TargetIdentityResult::ImageMismatch,
            TargetIdentityResult::CreationTimeMismatch,
            TargetIdentityResult::AccessDenied,
            TargetIdentityResult::ProcessExited,
            TargetIdentityResult::Unknown,
        ] {
            let mut i = identity();
            i.result = state;
            assert!(!DirectInjectPreflightGate::allows(&i, &i.nonce));
        }
    }
    #[test]
    fn windows_filetime_requires_nonzero_exact_decimal() {
        for time in ["0", "not-time", "18446744073709551616"] {
            let mut i = identity();
            i.expected_creation_time = time.into();
            i.observed_creation_time = time.into();
            assert!(!DirectInjectPreflightGate::allows(&i, &i.nonce));
        }
    }
    #[test]
    fn late_reentry_fake_uses_independent_invocation_without_loader() {
        for mode in [
            ContextExperimentMode::SamePrefix,
            ContextExperimentMode::DifferentPrefixes,
        ] {
            let (_fixture, config, sources) = fixture_config(mode);
            let result = run_core(
                &config,
                &sources.application_root,
                &sources.protected_roots,
                None,
                Arc::new(Fake::default()),
                |_| vec![],
            )
            .unwrap();
            assert!(!result.injector.inject_called);
            assert!(result
                .source_hashes
                .keys()
                .all(|name| !name.ends_with(".dll")));
            assert!(
                result
                    .reentry_evidence
                    .unwrap()
                    .target_ready_before_second_invocation
            );
            assert_eq!(
                result.injector.named_event_visible,
                Some(mode == ContextExperimentMode::SamePrefix)
            );
            assert_ne!(
                result.process_target.arguments,
                result.process_injector.arguments
            );
        }
    }
    #[test]
    fn reentry_rejects_game_prefix_and_installation_before_execution() {
        for path in ["steamapps/compatdata/4162040", "Zenless Zone Zero"] {
            let (_fixture, mut config, sources) = fixture_config(ContextExperimentMode::SamePrefix);
            config.managed_root = config.managed_root.join(path);
            fs::create_dir_all(&config.managed_root).unwrap();
            assert!(run_core(
                &config,
                &sources.application_root,
                &[],
                None,
                Arc::new(Fake::default()),
                |_| panic!("No process scan")
            )
            .is_err());
        }
    }
    #[test]
    fn exited_target_prevents_second_helper_invocation() {
        struct EarlyExit {
            fake: Fake,
            helper_called: std::sync::atomic::AtomicBool,
        }
        impl BridgeProcessExecutor for EarlyExit {
            fn execute(&self, call: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
                use std::os::unix::process::ExitStatusExt;
                if call.args[2] == "context-reentry" {
                    return Ok(ProcessOutput {
                        status: ExitStatus::from_raw(0),
                        elapsed_millis: 1,
                        stdout: b"{}".to_vec(),
                        stderr: vec![],
                        stdout_truncated: false,
                        stderr_truncated: false,
                    });
                }
                if call.args[2] != "bootstrap" {
                    self.helper_called
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                }
                self.fake.execute(call)
            }
        }
        let (_fixture, config, sources) = fixture_config(ContextExperimentMode::SamePrefix);
        let fake = Arc::new(EarlyExit {
            fake: Fake::default(),
            helper_called: std::sync::atomic::AtomicBool::new(false),
        });
        assert!(run_core(
            &config,
            &sources.application_root,
            &[],
            None,
            fake.clone(),
            |_| vec![]
        )
        .is_err());
        assert!(!fake.helper_called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
