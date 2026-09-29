//! Narrow Proton experiment using the `Inject` export from an authenticated upstream library.
//! The only target path is the LXMI-built test child under its private loader-v1 directory.

use super::*;
use lxmi_xxmi::{inspect_prefix_dosdevices, map_linux_path, DosDevicesReport, MappingStatus};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

pub const LOADER_LAB_DIRECTORY: &str = "tests/loader-v1";
pub const LOADER_LAB_RUNNER: &str = "lxmi-loader-test-runner.exe";
pub const LOADER_LAB_TARGET: &str = "lxmi-loader-test-target.exe";
pub const LOADER_LAB_TEST_DLL: &str = "lxmi-loader-test.dll";
pub const UPSTREAM_LOADER_DLL: &str = "3dmloader.dll";
const LOADER_LAB_MANIFEST: &str = "loader-lab-manifest.json";
const ARTIFACT_MAX_BYTES: u64 = 128 * 1024 * 1024;
const MARKER_MAX_BYTES: u64 = 256 * 1024;
const TARGET_LOG_MAX_BYTES: u64 = 64 * 1024;
const LOADER_TEST_TIMEOUT: Duration = Duration::from_secs(30);
const LOADER_OUTPUT_LIMIT: usize = 1024 * 1024;
static NEXT_LOADER_STAGE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderMode {
    ExternalLoader,
    Hook,
    DirectInject,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderExperimentMode {
    Baseline,
    Positive,
    MissingTarget,
    MissingDll,
    WrongNonce,
}

impl LoaderExperimentMode {
    fn argument(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Positive => "positive",
            Self::MissingTarget => "missing-target",
            Self::MissingDll => "missing-dll",
            Self::WrongNonce => "wrong-nonce",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderComponentLicenseState {
    VerifiedReusable,
    VerifiedExecutableOnly,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderCapabilityState {
    Verified,
    Failed,
    NotTested,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderCapability {
    pub name: String,
    pub state: LoaderCapabilityState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderLabProvenance {
    pub package_id: String,
    pub repository: String,
    pub release_id: u64,
    pub tag: String,
    pub commit: String,
    pub signature_verified: bool,
    pub component_signatures_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderLabManifest {
    pub schema_version: u32,
    pub test_id: String,
    pub build_target: String,
    pub component_license: LoaderComponentLicenseState,
    pub component_license_evidence: String,
    pub upstream: LoaderLabProvenance,
    pub upstream_loader_sha256: String,
    pub runner_sha256: String,
    pub target_sha256: String,
    pub test_dll_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderLabStatus {
    pub test_root: PathBuf,
    pub ready: bool,
    pub manifest: Option<LoaderLabManifest>,
    pub missing_or_invalid: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoaderLabSources {
    /// Application workspace root, derived by the native backend, never accepted from IPC.
    pub application_root: PathBuf,
    /// Path from `ManagedStore::verify` for an authenticated official package.
    pub upstream_loader_path: PathBuf,
    /// Digest recorded by the freshly verified package manifest.
    pub expected_upstream_loader_sha256: String,
    pub provenance: LoaderLabProvenance,
    pub protected_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LoaderExperimentConfig {
    pub managed_root: PathBuf,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub steam_client_install_path: PathBuf,
    pub mode: LoaderExperimentMode,
    pub side_effects_acknowledged: bool,
    /// The current bytes/hash from a freshly re-verified ManagedStore package.
    pub verified_loader_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoaderExperimentOutcome {
    Passed,
    ExpectedFailure,
}

/// Reviewable, fixed-path description of one Loader Lab run. All paths are
/// derived by LXMI from its managed test root; callers cannot supply targets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderExperimentPlan {
    pub mode: LoaderExperimentMode,
    pub loader_mode: LoaderMode,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub prefix_path: PathBuf,
    pub target_executable: PathBuf,
    pub test_dll: PathBuf,
    pub upstream_loader: PathBuf,
    pub expected_marker: PathBuf,
    pub timeout_seconds: u64,
}

/// Construction is private and happens only after validating the fixed test
/// artifacts, their managed root, the official loader hash, and Proton path.
struct ValidatedLoaderExperimentPlan {
    report: LoaderExperimentPlan,
    nonce: String,
    proton_script: PathBuf,
    runner_path: PathBuf,
    environment: BTreeMap<OsString, OsString>,
    working_directory: PathBuf,
}

impl ValidatedLoaderExperimentPlan {
    fn invocation(&self) -> ProcessInvocation {
        ProcessInvocation {
            executable: self.proton_script.clone(),
            args: vec![
                OsString::from("runinprefix"),
                self.runner_path.as_os_str().to_owned(),
                OsString::from(self.report.mode.argument()),
                OsString::from(self.nonce.as_str()),
            ],
            environment: self.environment.clone(),
            working_directory: self.working_directory.clone(),
            input: Vec::new(),
            timeout: Duration::from_secs(self.report.timeout_seconds),
            output_limit: LOADER_OUTPUT_LIMIT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderDllMarker {
    pub protocol: u16,
    pub loaded: bool,
    pub nonce: String,
    pub process: String,
    pub process_path_windows: String,
    pub dll_path_windows: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct LoaderRunnerResponse {
    protocol: u16,
    mode: String,
    target_started: bool,
    target_ready: bool,
    upstream_loader_loaded: bool,
    inject_called: bool,
    upstream_inject_code: i32,
    target_exit_code: i32,
    marker_present: bool,
    marker_process_matches: bool,
    nonce_verified: bool,
    marker_nonce: String,
    diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderExperimentResult {
    pub plan: LoaderExperimentPlan,
    pub mode: LoaderExperimentMode,
    pub outcome: LoaderExperimentOutcome,
    pub loader_mode: LoaderMode,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub compatdata_path: PathBuf,
    pub prefix_path: PathBuf,
    pub target_path_linux: PathBuf,
    pub target_path_windows: Option<String>,
    pub test_dll_path_linux: PathBuf,
    pub test_dll_path_windows: Option<String>,
    pub upstream_loader_path_linux: PathBuf,
    pub expected_loader_sha256: String,
    pub actual_loader_sha256: String,
    pub target_started: bool,
    pub target_ready: bool,
    pub loader_started: bool,
    pub upstream_inject_code: Option<i32>,
    pub dll_loaded: bool,
    pub marker_verified: bool,
    pub nonce_verified: bool,
    pub marker: Option<LoaderDllMarker>,
    pub process: BridgeProcessSummary,
    pub target_stdout: String,
    pub target_stderr: String,
    pub dosdevices: DosDevicesReport,
    pub capabilities: Vec<LoaderCapability>,
    pub warnings: Vec<String>,
}

pub fn loader_lab_root(managed_root: &Path) -> PathBuf {
    managed_root.join(LOADER_LAB_DIRECTORY)
}

pub fn stage_loader_lab(
    managed_root: &Path,
    sources: &LoaderLabSources,
) -> Result<LoaderLabStatus, BridgeError> {
    validate_provenance(&sources.provenance)?;
    let root = canonical_managed_root(managed_root)?;
    reject_protected_overlap(&root, &sources.protected_roots)?;
    let application_root = sources.application_root.canonicalize().map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::LoaderArtifactsMissing,
            format!("application build root unavailable: {error}"),
        )
    })?;
    let build_root = application_root.join("target/x86_64-pc-windows-gnu/release/lxmi-loader-lab");
    let runner_source = build_root.join(LOADER_LAB_RUNNER);
    let target_source = build_root.join(LOADER_LAB_TARGET);
    let dll_source = build_root.join(LOADER_LAB_TEST_DLL);
    let loader_source = validate_package_loader_source(&root, &sources.upstream_loader_path)?;

    let runner_sha256 = safe_artifact_hash(&application_root, &runner_source)?;
    let target_sha256 = safe_artifact_hash(&application_root, &target_source)?;
    let test_dll_sha256 = safe_artifact_hash(&application_root, &dll_source)?;
    let upstream_loader_sha256 = safe_artifact_hash(&root, &loader_source)?;
    if !valid_sha256(&sources.expected_upstream_loader_sha256)
        || !upstream_loader_sha256.eq_ignore_ascii_case(&sources.expected_upstream_loader_sha256)
    {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "3dmloader.dll bytes do not match the freshly verified package inventory.",
        ));
    }

    stage_loader_lab_verified(
        &root,
        VerifiedLoaderArtifacts {
            application_root: &application_root,
            runner_source: &runner_source,
            target_source: &target_source,
            dll_source: &dll_source,
            loader_source: &loader_source,
            app_source_root: &application_root,
            loader_source_root: &root,
            provenance: sources.provenance.clone(),
            runner_sha256,
            target_sha256,
            test_dll_sha256,
            upstream_loader_sha256,
        },
        &sources.protected_roots,
    )
}

struct VerifiedLoaderArtifacts<'a> {
    application_root: &'a Path,
    runner_source: &'a Path,
    target_source: &'a Path,
    dll_source: &'a Path,
    loader_source: &'a Path,
    app_source_root: &'a Path,
    loader_source_root: &'a Path,
    provenance: LoaderLabProvenance,
    runner_sha256: String,
    target_sha256: String,
    test_dll_sha256: String,
    upstream_loader_sha256: String,
}

fn stage_loader_lab_verified(
    root: &Path,
    artifacts: VerifiedLoaderArtifacts<'_>,
    protected_roots: &[PathBuf],
) -> Result<LoaderLabStatus, BridgeError> {
    let root = canonical_managed_root(root)?;
    reject_protected_overlap(&root, protected_roots)?;
    let lab_root = root.join(LOADER_LAB_DIRECTORY);
    for relative in ["runner", "target", "runtime", "results"] {
        ensure_controlled_directory(&root, &lab_root.join(relative))?;
    }
    let expected_app_root = artifacts.application_root.canonicalize().map_err(|error| {
        BridgeError::new(BridgeErrorCode::LoaderArtifactsMissing, error.to_string())
    })?;
    for source in [
        artifacts.runner_source,
        artifacts.target_source,
        artifacts.dll_source,
    ] {
        let canonical = source.canonicalize().map_err(|error| {
            BridgeError::new(
                BridgeErrorCode::LoaderArtifactsMissing,
                format!("Windows test tool missing; build the loader lab first: {error}"),
            )
        })?;
        if !canonical.starts_with(&expected_app_root) {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "test tool source escaped the LXMI application workspace",
            ));
        }
    }
    let runner_target = lab_root.join("runner").join(LOADER_LAB_RUNNER);
    let target_target = lab_root.join("target").join(LOADER_LAB_TARGET);
    let dll_target = lab_root.join("runtime").join(LOADER_LAB_TEST_DLL);
    let loader_target = lab_root.join("runtime").join(UPSTREAM_LOADER_DLL);
    atomic_copy_inside(
        &root,
        artifacts.app_source_root,
        artifacts.runner_source,
        &runner_target,
        ARTIFACT_MAX_BYTES,
    )?;
    atomic_copy_inside(
        &root,
        artifacts.app_source_root,
        artifacts.target_source,
        &target_target,
        ARTIFACT_MAX_BYTES,
    )?;
    atomic_copy_inside(
        &root,
        artifacts.app_source_root,
        artifacts.dll_source,
        &dll_target,
        ARTIFACT_MAX_BYTES,
    )?;
    atomic_copy_inside(
        &root,
        artifacts.loader_source_root,
        artifacts.loader_source,
        &loader_target,
        ARTIFACT_MAX_BYTES,
    )?;

    let manifest = LoaderLabManifest {
        schema_version: 1,
        test_id: "loader-v1".into(),
        build_target: "x86_64-pc-windows-gnu".into(),
        component_license: LoaderComponentLicenseState::VerifiedExecutableOnly,
        component_license_evidence: "3dmloader.dll procede del paquete oficial verificado XXMI Libraries; fuente 3Dmigoto publicada bajo GPLv3. LXMI permite ejecución local de esta prueba y no redistribuye el binario upstream.".into(),
        upstream: artifacts.provenance,
        upstream_loader_sha256: artifacts.upstream_loader_sha256,
        runner_sha256: artifacts.runner_sha256,
        target_sha256: artifacts.target_sha256,
        test_dll_sha256: artifacts.test_dll_sha256,
    };
    let manifest_path = lab_root.join(LOADER_LAB_MANIFEST);
    atomic_write_inside(
        &root,
        &manifest_path,
        &serde_json::to_vec_pretty(&manifest)
            .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?,
    )?;
    inspect_loader_lab(&root)
}

pub fn inspect_loader_lab(managed_root: &Path) -> Result<LoaderLabStatus, BridgeError> {
    let root = canonical_managed_root(managed_root)?;
    let lab_root = root.join(LOADER_LAB_DIRECTORY);
    let manifest_path = lab_root.join(LOADER_LAB_MANIFEST);
    let mut missing_or_invalid = Vec::new();
    let manifest = match read_bounded_file(&manifest_path, 64 * 1024) {
        Ok(bytes) => match serde_json::from_slice::<LoaderLabManifest>(&bytes) {
            Ok(manifest) => Some(manifest),
            Err(error) => {
                missing_or_invalid.push(format!("manifest: {error}"));
                None
            }
        },
        Err(error) => {
            missing_or_invalid.push(format!("manifest: {error}"));
            None
        }
    };
    if let Some(manifest) = &manifest {
        let files = [
            (
                "runner",
                lab_root.join("runner").join(LOADER_LAB_RUNNER),
                &manifest.runner_sha256,
            ),
            (
                "target",
                lab_root.join("target").join(LOADER_LAB_TARGET),
                &manifest.target_sha256,
            ),
            (
                "test_dll",
                lab_root.join("runtime").join(LOADER_LAB_TEST_DLL),
                &manifest.test_dll_sha256,
            ),
            (
                "upstream_loader",
                lab_root.join("runtime").join(UPSTREAM_LOADER_DLL),
                &manifest.upstream_loader_sha256,
            ),
        ];
        for (name, path, expected) in files {
            match safe_artifact_hash(&root, &path) {
                Ok(actual) if actual.eq_ignore_ascii_case(expected) => {}
                Ok(_) => missing_or_invalid.push(format!("{name}: SHA-256 mismatch")),
                Err(error) => missing_or_invalid.push(format!("{name}: {}", error.detail)),
            }
        }
        if manifest.schema_version != 1
            || manifest.test_id != "loader-v1"
            || manifest.build_target != "x86_64-pc-windows-gnu"
        {
            missing_or_invalid.push("manifest: unsupported schema, test id or target".into());
        }
        if validate_provenance(&manifest.upstream).is_err() {
            missing_or_invalid.push("manifest: upstream provenance is not trusted".into());
        }
    }
    Ok(LoaderLabStatus {
        test_root: lab_root,
        ready: manifest.is_some() && missing_or_invalid.is_empty(),
        manifest,
        missing_or_invalid,
    })
}

pub fn run_loader_experiment(
    config: &LoaderExperimentConfig,
) -> Result<LoaderExperimentResult, BridgeError> {
    info!(mode = ?config.mode, "Loader experiment started");
    if !config.side_effects_acknowledged {
        return Err(BridgeError::new(
            BridgeErrorCode::UserAcknowledgementRequired,
            "La prueba requiere aceptar que Proton escriba únicamente en el prefix aislado loader-v1 de LXMI.",
        ));
    }
    let root = canonical_managed_root(&config.managed_root)?;
    let status = inspect_loader_lab(&root)?;
    if !status.ready {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderArtifactsMissing,
            format!(
                "Loader Lab no está preparado: {}",
                status.missing_or_invalid.join("; ")
            ),
        ));
    }
    let manifest = status.manifest.ok_or_else(|| {
        BridgeError::new(
            BridgeErrorCode::LoaderArtifactsMissing,
            "Loader Lab manifest disappeared after readiness validation.",
        )
    })?;
    if !manifest
        .upstream_loader_sha256
        .eq_ignore_ascii_case(&config.verified_loader_sha256)
    {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "El 3dmloader.dll staged ya no coincide con el paquete oficial que acaba de verificarse.",
        ));
    }
    let expected_nonce = random_nonce()?;
    let proton_script = validate_proton_script(&config.explicit_runtime.proton_script)?;
    let steam_root = config
        .steam_client_install_path
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if !steam_root.is_dir() {
        return Err(BridgeError::new(
            BridgeErrorCode::RuntimeNotFound,
            "Steam client install path is not a directory",
        ));
    }
    let compatdata_path = prepare_named_test_context(&root, "loader-v1", "loader-test-prefix")?;
    let prefix_path = compatdata_path.join("pfx");
    let lab_root = status.test_root;
    reject_game_target(&root, &lab_root.join("target").join(LOADER_LAB_TARGET))?;
    let runner_path = safe_existing_file(&root, &lab_root.join("runner").join(LOADER_LAB_RUNNER))?;
    let target_path = safe_existing_file(&root, &lab_root.join("target").join(LOADER_LAB_TARGET))?;
    let test_dll_path =
        safe_existing_file(&root, &lab_root.join("runtime").join(LOADER_LAB_TEST_DLL))?;
    let upstream_loader_path =
        safe_existing_file(&root, &lab_root.join("runtime").join(UPSTREAM_LOADER_DLL))?;
    let actual_loader_sha256 = sha256_file(&upstream_loader_path, ARTIFACT_MAX_BYTES)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if !actual_loader_sha256.eq_ignore_ascii_case(&config.verified_loader_sha256) {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "3dmloader.dll no coincide con los bytes del package store revalidado.",
        ));
    }

    let compat_root = compatdata_path.parent().ok_or_else(|| {
        BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "isolated test prefix has no parent",
        )
    })?;
    for directory in ["home", "config", "cache", "data"] {
        ensure_controlled_directory(&root, &compat_root.join(directory))?;
    }
    let mut environment = BTreeMap::new();
    if let Some(path) = env::var_os("PATH") {
        environment.insert(OsString::from("PATH"), path);
    }
    if let Some(lang) = env::var_os("LANG") {
        environment.insert(OsString::from("LANG"), lang);
    }
    environment.insert(
        OsString::from("HOME"),
        compat_root.join("home").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_CONFIG_HOME"),
        compat_root.join("config").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_CACHE_HOME"),
        compat_root.join("cache").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_DATA_HOME"),
        compat_root.join("data").into_os_string(),
    );
    environment.insert(
        OsString::from("STEAM_COMPAT_CLIENT_INSTALL_PATH"),
        steam_root.as_os_str().to_owned(),
    );
    environment.insert(
        OsString::from("STEAM_COMPAT_DATA_PATH"),
        compatdata_path.as_os_str().to_owned(),
    );
    environment.insert(OsString::from("WINEDEBUG"), OsString::from("-all"));

    let report_plan = LoaderExperimentPlan {
        mode: config.mode,
        loader_mode: LoaderMode::DirectInject,
        explicit_runtime: config.explicit_runtime.clone(),
        prefix_path: prefix_path.clone(),
        target_executable: target_path.clone(),
        test_dll: test_dll_path.clone(),
        upstream_loader: upstream_loader_path.clone(),
        expected_marker: lab_root.join("results/loader-result.json"),
        timeout_seconds: LOADER_TEST_TIMEOUT.as_secs(),
    };
    validate_experiment_plan(&root, &lab_root, &report_plan)?;
    let validated_plan = ValidatedLoaderExperimentPlan {
        report: report_plan,
        nonce: expected_nonce.clone(),
        proton_script: proton_script.clone(),
        runner_path,
        environment,
        working_directory: lab_root.join("runner"),
    };
    let invocation = validated_plan.invocation();
    info!(runtime = %config.explicit_runtime.display_name, "Explicit loader-test Proton runtime selected");
    info!(target = %target_path.display(), "Fixed LXMI test target prepared");
    info!(loader = %upstream_loader_path.display(), "Authenticated upstream loader staged for test");
    let output = NativeProcessExecutor
        .execute(&invocation)
        .map_err(|failure| match failure {
            ProcessFailure::TimedOut(output) => BridgeError::new(
                BridgeErrorCode::Timeout,
                format!(
                    "loader experiment exceeded {} seconds",
                    LOADER_TEST_TIMEOUT.as_secs()
                ),
            )
            .with_stderr(String::from_utf8_lossy(&output.stderr).into_owned()),
            ProcessFailure::Launch(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                BridgeError::new(BridgeErrorCode::PermissionDenied, error.to_string())
            }
            ProcessFailure::Launch(error) if error.kind() == io::ErrorKind::NotFound => {
                BridgeError::new(BridgeErrorCode::ProtonExecutableMissing, error.to_string())
            }
            ProcessFailure::Launch(error) | ProcessFailure::OutputRead(error) => {
                BridgeError::new(BridgeErrorCode::ProcessLaunchFailed, error.to_string())
            }
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.stdout_truncated || output.stderr_truncated {
        return Err(BridgeError::new(
            BridgeErrorCode::OutputLimitExceeded,
            "loader runner exceeded the 1 MiB output capture limit",
        )
        .with_stderr(stderr));
    }
    let runner: LoaderRunnerResponse = serde_json::from_str(stdout.trim()).map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::MalformedResponse,
            format!("loader runner did not return one valid JSON result: {error}"),
        )
        .with_stderr(stderr.clone())
    })?;
    if runner.protocol != 1 {
        return Err(BridgeError::new(
            BridgeErrorCode::ProtocolMismatch,
            format!(
                "loader runner protocol {} is not supported",
                runner.protocol
            ),
        ));
    }
    let exit_code = output.status.code().unwrap_or(-1);
    let result_file = validated_plan.report.expected_marker.clone();
    let marker = read_marker(&root, &result_file)?;
    let marker_process_valid = marker
        .as_ref()
        .is_some_and(|value| value.loaded && value.process == LOADER_LAB_TARGET);
    let marker_response_consistent = marker.as_ref().is_some_and(|value| {
        runner.marker_present
            && runner.marker_process_matches
            && runner.marker_nonce == value.nonce
            && value.process == LOADER_LAB_TARGET
    });
    let marker_nonce_valid = marker
        .as_ref()
        .is_some_and(|value| value.nonce == expected_nonce);
    let dosdevices = inspect_prefix_dosdevices(&prefix_path);
    let target_mapping = map_linux_path(&target_path, &dosdevices);
    let dll_mapping = map_linux_path(&test_dll_path, &dosdevices);
    let marker_paths_match = match (
        &marker,
        &target_mapping.windows_path,
        &dll_mapping.windows_path,
    ) {
        (Some(marker), Some(target), Some(dll)) => {
            marker.process_path_windows.eq_ignore_ascii_case(target)
                && marker.dll_path_windows.eq_ignore_ascii_case(dll)
        }
        _ => false,
    };
    let path_mapping_verified = target_mapping.status == MappingStatus::Mapped
        && dll_mapping.status == MappingStatus::Mapped
        && marker_paths_match;
    let target_stdout = read_test_log(&root, &lab_root.join("results/target.stdout.log"))?;
    let target_stderr = read_test_log(&root, &lab_root.join("results/target.stderr.log"))?;
    let process = BridgeProcessSummary {
        executable: proton_script,
        arguments: invocation
            .args
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect(),
        exit_code,
        elapsed_millis: output.elapsed_millis,
        stdout_truncated: output.stdout_truncated,
        stderr_truncated: output.stderr_truncated,
        stdout,
        stderr,
    };
    let expected_failure = expected_failure(
        config.mode,
        &runner,
        &process,
        marker.is_some(),
        marker_nonce_valid,
        marker_response_consistent,
    );
    let positive_ok = config.mode == LoaderExperimentMode::Positive
        && exit_code == 0
        && runner.mode == "direct_inject"
        && runner.target_started
        && runner.target_ready
        && runner.upstream_loader_loaded
        && runner.inject_called
        && runner.upstream_inject_code == 0
        && runner.target_exit_code == 0
        && marker_process_valid
        && marker_response_consistent
        && marker_nonce_valid
        && runner.marker_process_matches
        && runner.nonce_verified
        && path_mapping_verified;
    let baseline_ok = config.mode == LoaderExperimentMode::Baseline
        && exit_code == 0
        && runner.mode == "baseline"
        && runner.target_started
        && runner.target_ready
        && !runner.upstream_loader_loaded
        && !runner.inject_called
        && !runner.marker_present
        && runner.target_exit_code == 0
        && marker.is_none();
    if config.mode == LoaderExperimentMode::Positive && !positive_ok {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderTestFailed,
            format!(
                "positive test failed: target={}, ready={}, loader={}, code={}, marker={}, nonce={}, mapping={}; diagnostic={:?}",
                runner.target_started,
                runner.target_ready,
                runner.upstream_loader_loaded,
                runner.upstream_inject_code,
                marker_process_valid,
                marker_nonce_valid,
                path_mapping_verified,
                runner.diagnostic
            ),
        )
        .with_stderr(process.stderr.clone()));
    }
    if config.mode == LoaderExperimentMode::Baseline && !baseline_ok {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderTestFailed,
            format!("baseline test failed: diagnostic={:?}", runner.diagnostic),
        )
        .with_stderr(process.stderr.clone()));
    }
    if matches!(
        config.mode,
        LoaderExperimentMode::MissingTarget
            | LoaderExperimentMode::MissingDll
            | LoaderExperimentMode::WrongNonce
    ) && !expected_failure
    {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderTestFailed,
            format!(
                "negative test did not return the expected structured rejection: diagnostic={:?}",
                runner.diagnostic
            ),
        )
        .with_stderr(process.stderr.clone()));
    }

    let outcome = if expected_failure {
        LoaderExperimentOutcome::ExpectedFailure
    } else {
        LoaderExperimentOutcome::Passed
    };
    let capabilities = capability_results(config.mode, positive_ok, path_mapping_verified);
    info!(mode = ?config.mode, outcome = ?outcome, "Loader experiment completed");
    Ok(LoaderExperimentResult {
        plan: validated_plan.report.clone(),
        mode: config.mode,
        outcome,
        loader_mode: LoaderMode::DirectInject,
        explicit_runtime: config.explicit_runtime.clone(),
        compatdata_path,
        prefix_path,
        target_path_linux: target_path,
        target_path_windows: target_mapping.windows_path,
        test_dll_path_linux: test_dll_path,
        test_dll_path_windows: dll_mapping.windows_path,
        upstream_loader_path_linux: upstream_loader_path,
        expected_loader_sha256: config.verified_loader_sha256.clone(),
        actual_loader_sha256,
        target_started: runner.target_started,
        target_ready: runner.target_ready,
        loader_started: runner.upstream_loader_loaded,
        upstream_inject_code: runner.inject_called.then_some(runner.upstream_inject_code),
        dll_loaded: marker.is_some(),
        marker_verified: positive_ok,
        nonce_verified: marker_nonce_valid && runner.nonce_verified,
        marker,
        process,
        target_stdout,
        target_stderr,
        dosdevices,
        capabilities,
        warnings: vec![
            "La prueba llama el export Direct Inject upstream solo con el PID del hijo creado en esta ejecución; no busca ni abre otros procesos.".into(),
            "La ruta Hook ZZMI instala un hook global y no forma parte de este test.".into(),
            "Esto no verifica ZZMI, ZZZ, Steam, mods ni compatibilidad del juego bajo Linux/Proton.".into(),
            "Proton puede escribir y mantener únicamente test-prefixes/loader-v1; no se usa compatdata del juego.".into(),
        ],
    })
}

fn validate_experiment_plan(
    managed_root: &Path,
    lab_root: &Path,
    plan: &LoaderExperimentPlan,
) -> Result<(), BridgeError> {
    let expected_target = lab_root.join("target").join(LOADER_LAB_TARGET);
    let expected_dll = lab_root.join("runtime").join(LOADER_LAB_TEST_DLL);
    let expected_upstream_loader = lab_root.join("runtime").join(UPSTREAM_LOADER_DLL);
    let expected_marker = lab_root.join("results/loader-result.json");
    let fixed_paths_match = plan.target_executable == expected_target
        && plan.test_dll == expected_dll
        && plan.upstream_loader == expected_upstream_loader
        && plan.expected_marker == expected_marker
        && plan.target_executable.starts_with(managed_root)
        && plan.test_dll.starts_with(managed_root)
        && plan.upstream_loader.starts_with(managed_root)
        && plan.expected_marker.starts_with(managed_root)
        && plan.loader_mode == LoaderMode::DirectInject
        && plan.timeout_seconds == LOADER_TEST_TIMEOUT.as_secs();
    if !fixed_paths_match {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "loader plan must use LXMI's fixed test target, test DLL, verified upstream loader, marker and bounded timeout",
        ));
    }
    reject_game_target(managed_root, &plan.target_executable)?;
    Ok(())
}

#[cfg(test)]
fn mode_name(mode: LoaderExperimentMode) -> &'static str {
    match mode {
        LoaderExperimentMode::Baseline => "baseline",
        LoaderExperimentMode::Positive => "direct_inject",
        LoaderExperimentMode::MissingTarget => "missing_target",
        LoaderExperimentMode::MissingDll => "missing_dll",
        LoaderExperimentMode::WrongNonce => "wrong_nonce",
    }
}

fn expected_failure(
    mode: LoaderExperimentMode,
    response: &LoaderRunnerResponse,
    process: &BridgeProcessSummary,
    marker_present: bool,
    marker_nonce_valid: bool,
    marker_response_consistent: bool,
) -> bool {
    match mode {
        LoaderExperimentMode::MissingTarget => {
            response.mode == "missing_target"
                && !response.target_started
                && !response.marker_present
                && response.diagnostic.as_deref() == Some("target_not_found")
                && process.exit_code == 10
        }
        LoaderExperimentMode::MissingDll => {
            response.mode == "missing_dll"
                && response.target_started
                && response.target_ready
                && response.upstream_loader_loaded
                && response.inject_called
                && response.upstream_inject_code == 110
                && !marker_present
                && response.diagnostic.as_deref() == Some("missing_test_dll")
                && response.target_exit_code == 0
                && !response.marker_present
                && process.exit_code == 11
        }
        LoaderExperimentMode::WrongNonce => {
            response.mode == "wrong_nonce"
                && response.target_started
                && response.target_ready
                && response.inject_called
                && response.upstream_inject_code == 0
                && marker_present
                && response.marker_present
                && response.marker_process_matches
                && marker_response_consistent
                && !marker_nonce_valid
                && !response.nonce_verified
                && response.diagnostic.as_deref() == Some("nonce_mismatch")
                && response.target_exit_code == 0
                && process.exit_code == 12
        }
        LoaderExperimentMode::Baseline | LoaderExperimentMode::Positive => false,
    }
}

fn capability_results(
    mode: LoaderExperimentMode,
    direct_verified: bool,
    mapping_verified: bool,
) -> Vec<LoaderCapability> {
    vec![
        LoaderCapability {
            name: "Windows helper execution under explicit Proton".into(),
            state: if mode == LoaderExperimentMode::Baseline || direct_verified {
                LoaderCapabilityState::Verified
            } else {
                LoaderCapabilityState::NotTested
            },
            evidence: "LXMI-owned loader test target executed in the isolated loader-v1 prefix.".into(),
        },
        LoaderCapability {
            name: "Upstream 3dmloader Direct Inject export under Proton".into(),
            state: if direct_verified { LoaderCapabilityState::Verified } else { LoaderCapabilityState::NotTested },
            evidence: "Only the child PID returned by CreateProcessW is passed to the upstream Inject export.".into(),
        },
        LoaderCapability {
            name: "Remote test DLL path and load".into(),
            state: if direct_verified && mapping_verified { LoaderCapabilityState::Verified } else { LoaderCapabilityState::NotTested },
            evidence: "The test DLL reports its own Windows path and target identity in a nonce marker.".into(),
        },
        LoaderCapability {
            name: "Upstream Hook mode under Proton".into(),
            state: LoaderCapabilityState::Blocked,
            evidence: "Not run because SetWindowsHookEx installs a desktop-wide hook outside the required single-target boundary.".into(),
        },
        LoaderCapability {
            name: "ZZMI default loader compatibility".into(),
            state: LoaderCapabilityState::NotTested,
            evidence: "Current ZZMI config uses the upstream Hook path; the isolated Direct Inject experiment is not equivalent.".into(),
        },
        LoaderCapability {
            name: "ZZZ / Steam / anti-cheat compatibility".into(),
            state: LoaderCapabilityState::NotTested,
            evidence: "No game, launcher, anti-cheat or Steam process is touched.".into(),
        },
    ]
}

fn validate_provenance(provenance: &LoaderLabProvenance) -> Result<(), BridgeError> {
    if provenance.package_id.len() != 64
        || !provenance
            .package_id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || provenance.repository != "SpectrumQT/XXMI-Libs-Package"
        || provenance.tag.is_empty()
        || provenance.commit.len() < 7
        || !provenance.signature_verified
        || !provenance.component_signatures_verified
    {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "3dmloader.dll must come from the official XXMI Libraries package with verified release and component signature provenance.",
        ));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_managed_root(path: &Path) -> Result<PathBuf, BridgeError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "LXMI storage root must be a real non-symlink directory",
        ));
    }
    path.canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))
}

fn reject_protected_overlap(root: &Path, protected_roots: &[PathBuf]) -> Result<(), BridgeError> {
    for path in protected_roots {
        let protected = match path.canonicalize() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if root.starts_with(&protected) || protected.starts_with(root) {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                format!(
                    "Loader Lab storage overlaps a protected Steam/game/prefix path: {}",
                    protected.display()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_package_loader_source(root: &Path, path: &Path) -> Result<PathBuf, BridgeError> {
    let canonical = safe_existing_file(root, path)?;
    let packages_root = root.join("packages/xxmi").canonicalize().map_err(|error| {
        BridgeError::new(BridgeErrorCode::LoaderProvenanceInvalid, error.to_string())
    })?;
    if !canonical.starts_with(packages_root)
        || canonical.file_name().and_then(|name| name.to_str()) != Some(UPSTREAM_LOADER_DLL)
    {
        return Err(BridgeError::new(
            BridgeErrorCode::LoaderProvenanceInvalid,
            "upstream loader source must be 3dmloader.dll inside the managed XXMI package store",
        ));
    }
    Ok(canonical)
}

fn safe_artifact_hash(root: &Path, path: &Path) -> Result<String, BridgeError> {
    let canonical = safe_existing_file(root, path)?;
    sha256_file(&canonical, ARTIFACT_MAX_BYTES)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))
}

fn atomic_copy_inside(
    destination_root: &Path,
    source_root: &Path,
    source: &Path,
    target: &Path,
    limit: u64,
) -> Result<(), BridgeError> {
    let source = safe_existing_file(source_root, source)?;
    let parent = target.parent().ok_or_else(|| {
        BridgeError::new(BridgeErrorCode::UnsafePath, "copy target has no parent")
    })?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if !canonical_parent.starts_with(destination_root) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "copy destination is outside LXMI storage",
        ));
    }
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "staged artifact target is not a regular file",
            ));
        }
    }
    let bytes = read_bounded_file(&source, limit)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    let temporary = parent.join(format!(
        ".lxmi-stage-{}-{}",
        std::process::id(),
        NEXT_LOADER_STAGE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            let _ = fs::remove_file(&temporary);
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "staged artifact target changed to an unsafe type",
            ));
        }
    }
    fs::rename(&temporary, target)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    Ok(())
}

fn atomic_write_inside(root: &Path, target: &Path, bytes: &[u8]) -> Result<(), BridgeError> {
    let parent = target.parent().ok_or_else(|| {
        BridgeError::new(BridgeErrorCode::UnsafePath, "manifest target has no parent")
    })?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if !canonical_parent.starts_with(root) || bytes.len() > 64 * 1024 {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "manifest write escaped LXMI storage or exceeded its limit",
        ));
    }
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "manifest target is not a regular file",
            ));
        }
    }
    let temporary = parent.join(format!(
        ".lxmi-manifest-{}-{}",
        std::process::id(),
        NEXT_LOADER_STAGE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            let _ = fs::remove_file(&temporary);
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "manifest target changed to an unsafe type",
            ));
        }
        fs::remove_file(target)
            .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    }
    fs::rename(temporary, target)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))
}

fn prepare_named_test_context(
    managed_root: &Path,
    name: &str,
    kind: &str,
) -> Result<PathBuf, BridgeError> {
    if !matches!(name, "bridge-v1" | "loader-v1")
        || !matches!(kind, "bridge-test-prefix" | "loader-test-prefix")
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "unknown LXMI test prefix identity",
        ));
    }
    let root = canonical_managed_root(managed_root)?;
    let context_root = root.join("test-prefixes").join(name);
    ensure_controlled_directory(&root, &context_root)?;
    let marker = context_root.join("lxmi-test-prefix.json");
    let expected = format!("{{\"owner\":\"LXMI\",\"kind\":\"{kind}\",\"schema\":1}}\n");
    match fs::symlink_metadata(&marker) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePath,
                    "LXMI test prefix marker must be a regular file",
                ));
            }
            let bytes = read_bounded_file(&marker, 4096)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            if bytes != expected.as_bytes() {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePrefixContext,
                    "prefix metadata does not match the LXMI-owned test context",
                ));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&marker)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            file.write_all(expected.as_bytes())
                .and_then(|()| file.sync_all())
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
        }
        Err(error) => return Err(BridgeError::new(BridgeErrorCode::Io, error.to_string())),
    }
    let compatdata = context_root.join("compatdata");
    ensure_controlled_directory(&root, &compatdata)?;
    let prefix = compatdata.join("pfx");
    if let Ok(metadata) = fs::symlink_metadata(&prefix) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "test prefix pfx is not a regular directory",
            ));
        }
    }
    Ok(compatdata)
}

fn reject_game_target(root: &Path, target: &Path) -> Result<(), BridgeError> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::UnsafePath, error.to_string()))?;
    let canonical_target = target.canonicalize().map_err(|error| {
        BridgeError::new(BridgeErrorCode::LoaderArtifactsMissing, error.to_string())
    })?;
    let name = canonical_target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let path_string = canonical_target.to_string_lossy().to_ascii_lowercase();
    if !canonical_target.starts_with(&canonical_root)
        || name != LOADER_LAB_TARGET
        || path_string.contains("/steamapps/common/")
        || name.eq_ignore_ascii_case("zenlesszonezero.exe")
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "loader target must be LXMI's fixed test executable under tests/loader-v1/target",
        ));
    }
    Ok(())
}

fn read_marker(root: &Path, path: &Path) -> Result<Option<LoaderDllMarker>, BridgeError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(BridgeError::new(BridgeErrorCode::Io, error.to_string())),
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.len() > MARKER_MAX_BYTES =>
        {
            Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "test DLL marker must be a bounded regular file",
            ))
        }
        Ok(_) => {
            let _ = safe_existing_file(root, path)?;
            let bytes = read_bounded_file(path, MARKER_MAX_BYTES)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            serde_json::from_slice(&bytes).map(Some).map_err(|error| {
                BridgeError::new(
                    BridgeErrorCode::MalformedResponse,
                    format!("test DLL marker is invalid JSON: {error}"),
                )
            })
        }
    }
}

fn read_test_log(root: &Path, path: &Path) -> Result<String, BridgeError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(BridgeError::new(BridgeErrorCode::Io, error.to_string())),
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.len() > TARGET_LOG_MAX_BYTES =>
        {
            Err(BridgeError::new(
                BridgeErrorCode::OutputLimitExceeded,
                "test target log is unsafe or exceeds 64 KiB",
            ))
        }
        Ok(_) => {
            let canonical = safe_existing_file(root, path)?;
            let bytes = read_bounded_file(&canonical, TARGET_LOG_MAX_BYTES)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }
}

#[cfg(test)]
fn same_windows_path(left: &str, right: &str) -> bool {
    left.replace('/', "\\")
        .eq_ignore_ascii_case(&right.replace('/', "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(mode: &str) -> LoaderRunnerResponse {
        LoaderRunnerResponse {
            protocol: 1,
            mode: mode.into(),
            ..Default::default()
        }
    }

    fn summary(exit_code: i32) -> BridgeProcessSummary {
        BridgeProcessSummary {
            executable: PathBuf::from("/test/proton"),
            arguments: vec!["runinprefix".into()],
            exit_code,
            elapsed_millis: 1,
            stdout_truncated: false,
            stderr_truncated: false,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    #[test]
    fn modes_are_closed_and_map_to_fixed_cli_arguments() {
        assert_eq!(LoaderExperimentMode::Positive.argument(), "positive");
        assert_eq!(
            LoaderExperimentMode::MissingTarget.argument(),
            "missing-target"
        );
        assert_eq!(mode_name(LoaderExperimentMode::WrongNonce), "wrong_nonce");
    }

    #[test]
    fn experiment_plan_rejects_targets_outside_the_fixed_lxmi_fixture() {
        let managed_root = Path::new("/tmp/lxmi-managed");
        let lab_root = managed_root.join(LOADER_LAB_DIRECTORY);
        let plan = LoaderExperimentPlan {
            mode: LoaderExperimentMode::Positive,
            loader_mode: LoaderMode::DirectInject,
            explicit_runtime: ExplicitBridgeRuntime {
                display_name: "Explicit test runtime".into(),
                version: None,
                proton_script: PathBuf::from("/test/Proton/proton"),
            },
            prefix_path: managed_root.join("test-prefixes/loader-v1/compatdata/pfx"),
            target_executable: PathBuf::from("/tmp/arbitrary.exe"),
            test_dll: lab_root.join("runtime").join(LOADER_LAB_TEST_DLL),
            upstream_loader: lab_root.join("runtime").join(UPSTREAM_LOADER_DLL),
            expected_marker: lab_root.join("results/loader-result.json"),
            timeout_seconds: LOADER_TEST_TIMEOUT.as_secs(),
        };

        let error = validate_experiment_plan(managed_root, &lab_root, &plan)
            .expect_err("arbitrary target must be rejected before execution");
        assert_eq!(error.code, BridgeErrorCode::UnsafePath);
    }

    #[test]
    fn provenance_requires_official_verified_xxmi_libraries_component() {
        let provenance = LoaderLabProvenance {
            package_id: "a".repeat(64),
            repository: "SpectrumQT/XXMI-Libs-Package".into(),
            release_id: 538653,
            tag: "v1.1.7".into(),
            commit: "6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9".into(),
            signature_verified: true,
            component_signatures_verified: true,
        };
        assert!(validate_provenance(&provenance).is_ok());
        let mut untrusted = provenance;
        untrusted.repository = "random/fork".into();
        assert!(validate_provenance(&untrusted).is_err());
    }

    #[test]
    fn wrong_target_requires_no_process_and_no_fallback() {
        let mut result = response("missing_target");
        result.diagnostic = Some("target_not_found".into());
        assert!(expected_failure(
            LoaderExperimentMode::MissingTarget,
            &result,
            &summary(10),
            false,
            false,
            false
        ));
        let mut fallback = response("direct_inject");
        fallback.target_started = true;
        fallback.target_ready = true;
        fallback.upstream_loader_loaded = true;
        fallback.inject_called = true;
        fallback.upstream_inject_code = 0;
        fallback.target_exit_code = 0;
        fallback.marker_present = true;
        fallback.marker_process_matches = true;
        fallback.nonce_verified = true;
        assert!(!expected_failure(
            LoaderExperimentMode::MissingTarget,
            &fallback,
            &summary(0),
            true,
            true,
            true
        ));
    }

    #[test]
    fn missing_dll_is_structured_and_target_exits_cleanly() {
        let mut result = response("missing_dll");
        result.target_started = true;
        result.target_ready = true;
        result.upstream_loader_loaded = true;
        result.inject_called = true;
        result.upstream_inject_code = 110;
        result.target_exit_code = 0;
        result.diagnostic = Some("missing_test_dll".into());
        assert!(expected_failure(
            LoaderExperimentMode::MissingDll,
            &result,
            &summary(11),
            false,
            false,
            false
        ));
    }

    #[test]
    fn wrong_nonce_marker_is_rejected_even_when_dll_loaded() {
        let mut result = response("wrong_nonce");
        result.target_started = true;
        result.target_ready = true;
        result.upstream_loader_loaded = true;
        result.inject_called = true;
        result.upstream_inject_code = 0;
        result.target_exit_code = 0;
        result.marker_present = true;
        result.marker_process_matches = true;
        result.nonce_verified = false;
        result.marker_nonce = "badbadbadbadbadbadbadbadbadbadba".into();
        result.diagnostic = Some("nonce_mismatch".into());
        assert!(expected_failure(
            LoaderExperimentMode::WrongNonce,
            &result,
            &summary(12),
            true,
            false,
            true
        ));
        let mut forged = result;
        forged.nonce_verified = true;
        assert!(!expected_failure(
            LoaderExperimentMode::WrongNonce,
            &forged,
            &summary(12),
            true,
            false,
            true
        ));
    }

    #[test]
    fn baseline_never_claims_loader_capability() {
        let values = capability_results(LoaderExperimentMode::Baseline, false, false);
        let inject = values
            .iter()
            .find(|value| value.name.contains("Direct Inject"))
            .unwrap();
        assert_eq!(inject.state, LoaderCapabilityState::NotTested);
    }

    #[test]
    fn windows_paths_are_compared_without_case_or_separator_sensitivity() {
        assert!(same_windows_path(
            "z:/home/user/test.dll",
            "Z:\\home\\user\\test.dll"
        ));
    }
}
