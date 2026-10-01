//! Fixed fixture context boundary. A Windows PID is never accepted from IPC.
//! A nonce-scoped pipe and its server PID must bind the target before Inject.
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
    pub inject_called: bool,
    pub inject_code: i32,
    pub windows_error: u32,
    pub wine_prefix: String,
    pub compatdata: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextTargetResponse {
    pub protocol: u16,
    pub pid: u32,
    pub nonce: String,
    pub path_readable: bool,
    pub handshake_sent: bool,
    pub wine_prefix: String,
    pub compatdata: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectContextOutcome {
    InjectedSuccessfully,
    TargetIdentityUnavailable,
    LoaderFailure,
    MarkerNotVerified,
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
    pub process_injector: BridgeProcessSummary,
    pub source_hashes: BTreeMap<String, String>,
    pub upstream: LoaderLabProvenance,
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
    if target.protocol != 1
        || runner.protocol != 1
        || target.nonce != nonce
        || runner.nonce != nonce
        || target.pid == 0
        || runner.pid == 0
        || (runner.identity_bound
            && (!target.handshake_sent
                || runner.target_pid != target.pid
                || runner.target_pid == runner.pid))
        || (runner.inject_called && !runner.identity_bound)
    {
        return Err(error(
            BridgeErrorCode::ProtocolMismatch,
            "Owned target/pipe/nonce/protocol mismatch",
        ));
    }
    Ok(())
}
fn outcome(r: &ContextInjectorResponse, marker: bool) -> DirectContextOutcome {
    if !r.identity_bound {
        DirectContextOutcome::TargetIdentityUnavailable
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
        contexts::server_candidates,
    )
}
fn run_with(
    config: &ContextLabConfig,
    sources: &LoaderLabSources,
    executor: Arc<dyn BridgeProcessExecutor + Send + Sync>,
    observe_servers: fn(&str) -> Vec<u32>,
) -> Result<DirectContextResult, BridgeError> {
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

    loaders::reject_protected_overlap(&managed, &sources.protected_roots)?;
    loaders::validate_provenance(&sources.provenance)?;
    validate_proton_script(&config.explicit_runtime.proton_script)?;
    let app = sources
        .application_root
        .canonicalize()
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let build = app.join("target/x86_64-pc-windows-gnu/release/lxmi-loader-lab");
    let loader = loaders::validate_package_loader_source(&managed, &sources.upstream_loader_path)?;
    let hash = loaders::safe_artifact_hash(&managed, &loader)?;
    if hash != sources.expected_upstream_loader_sha256 {
        return Err(error(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "Official loader hash mismatch",
        ));
    }
    let nonce = random_nonce()?;
    let session = managed.join("tests/direct-context-v1").join(&nonce);
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
    let entries = [
        (
            build.join(LOADER_LAB_TARGET),
            root.join("target").join(LOADER_LAB_TARGET),
            app.as_path(),
        ),
        (
            build.join(RUNNER),
            root.join("runner").join(RUNNER),
            app.as_path(),
        ),
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
    ];
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
    let metadata = serde_json::to_vec(&(&source_hashes, &sources.provenance))
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    fs::write(session.join("fixture-manifest.json"), metadata)
        .map_err(|e| error(BridgeErrorCode::Io, e.to_string()))?;
    let compat_a = root.join("compat-a");
    let compat_b = if config.mode == ContextExperimentMode::SamePrefix {
        compat_a.clone()
    } else {
        root.join("compat-b")
    };
    let runner = root.join("runner").join(RUNNER);
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
        vec!["context-boundary".into(), nonce.clone().into()],
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
    let candidates_a = observe_servers(&compat_a.join("pfx").display().to_string());
    let output_b = if valid_ready {
        contexts::execute(executor.as_ref(), &call_b)
    } else {
        Err(error(
            BridgeErrorCode::Timeout,
            "Owned fixture did not publish fresh ready nonce",
        ))
    };
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
    let target: ContextTargetResponse = parse(&output_a)?;
    let injector: ContextInjectorResponse = parse(&output_b)?;
    validate_responses(&nonce, &target, &injector)?;
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
        "Fixture {:?}: {:?}; not a ZZZ test; cross-prefix unavailable pipe means Inject was not attempted",
        config.mode, outcome(&injector, marker_verified)
    ));
    game_join_plan.blockers.push("A numeric Windows PID from another context cannot authorize injection; no game-prefix join executor exists".into());
    Ok(DirectContextResult { mode:config.mode,nonce,explicit_runtime:config.explicit_runtime.clone(),outcome:outcome(&injector,marker_verified),target,injector,
        context_target,context_injector,identity,marker_verified,marker,process_target:contexts::summary(&call_a,output_a),process_injector:contexts::summary(&call_b,output_b),source_hashes,upstream:sources.provenance.clone(),requirements:loader_context_requirements(), capabilities: vec![LoaderCapability {
            name: if config.mode == ContextExperimentMode::SamePrefix { "DirectInjectFixtureSamePrefix" } else { "DirectInjectFixtureCrossPrefix" }.into(),
            state: if marker_verified { LoaderCapabilityState::Verified } else if !identity_bound { LoaderCapabilityState::Blocked } else { LoaderCapabilityState::Failed },
            evidence: "Only this nonce-scoped fixture; missing pipe prevents the upstream call, not an observed Inject failure".into(),
        }],
        game_join_plan,warnings:vec!["TargetIdentityUnavailable prevents Inject; it is NOT an observed upstream Inject failure".into(),"No game compatibility established; Hook blocked; strategy NoDecision".into(),"Wine servers are not signalled; they may exit naturally".into()] })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn runner() -> ContextInjectorResponse {
        ContextInjectorResponse {
            protocol: 1,
            pid: 200,
            target_pid: 220,
            nonce: "a".repeat(32),
            path_readable: true,
            identity_bound: true,
            inject_called: true,
            inject_code: 0,
            windows_error: 0,
            wine_prefix: String::new(),
            compatdata: String::new(),
        }
    }
    fn target() -> ContextTargetResponse {
        ContextTargetResponse {
            protocol: 1,
            pid: 220,
            nonce: "a".repeat(32),
            path_readable: true,
            handshake_sent: true,
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
            DirectContextOutcome::TargetIdentityUnavailable
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
        for name in [RUNNER, LOADER_LAB_TARGET, LOADER_LAB_TEST_DLL] {
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
            } else if call.args[2] == "context-boundary" {
                let nonce = call.args[3].to_string_lossy().to_string();
                *self.0.lock().unwrap() = Some(compat.clone());
                fs::write(root.join("results/target-ready"), &nonce).unwrap();
                let end = Instant::now() + Duration::from_secs(5);
                while !root.join("results/stop").exists() && Instant::now() < end {
                    thread::sleep(Duration::from_millis(5));
                }
                let same = self.0.lock().unwrap().as_ref() == Some(&compat);
                serde_json::to_vec(&ContextTargetResponse {
                    protocol: 1,
                    pid: 220,
                    nonce,
                    path_readable: true,
                    handshake_sent: same,
                    wine_prefix: compat.join("pfx").display().to_string(),
                    compatdata: compat.display().to_string(),
                })
                .unwrap()
            } else {
                let nonce = call.args[2].to_string_lossy().to_string();
                let same = self.0.lock().unwrap().as_ref() == Some(&compat);
                if same {
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
                    protocol: 1,
                    pid: if same { 236 } else { 220 },
                    target_pid: if same { 220 } else { 0 },
                    nonce,
                    path_readable: true,
                    identity_bound: same,
                    inject_called: same,
                    inject_code: if same { 0 } else { -1 },
                    windows_error: if same { 0 } else { 2 },
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
        assert_eq!(
            result.outcome,
            DirectContextOutcome::TargetIdentityUnavailable
        );
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
}
