//! Controlled, one-shot Windows helper bridge through an explicitly selected Proton tool.
//!
//! This crate never launches a game. Its only accepted executable is an LXMI-managed helper,
//! and its only accepted prefix is an LXMI-owned isolated test context.

mod loader_lab;

pub use loader_lab::*;

use lxmi_bridge_protocol::{
    BridgeRequest, BridgeResponse, ENVIRONMENT_MARKER, ENVIRONMENT_MARKER_VALUE, HELPER_VERSION,
    MARKER_FILE, PROTOCOL_VERSION, REQUEST_LIMIT_BYTES, RESPONSE_LIMIT_BYTES,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::process::CommandExt,
    path::{Component, Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};
use tracing::{info, warn};

pub const BRIDGE_HELPER_FILE: &str = "lxmi-bridge-helper.exe";
pub const BRIDGE_HELPER_MANIFEST_FILE: &str = "lxmi-bridge-helper.json";
pub const HELPER_MAX_BYTES: u64 = 64 * 1024 * 1024;
pub const HELPER_MANIFEST_MAX_BYTES: u64 = 16 * 1024;
pub const BRIDGE_TEST_TIMEOUT: Duration = Duration::from_secs(20);
pub const BRIDGE_OUTPUT_LIMIT_BYTES: usize = 1024 * 1024;
const PREFIX_MARKER_FILE: &str = "lxmi-bridge-prefix.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixMode {
    IsolatedTemporaryPrefix,
    ExistingGameCompatdataReadOnlyContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitBridgeRuntime {
    pub display_name: String,
    pub version: Option<String>,
    pub proton_script: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeHelperManifest {
    pub protocol_version: u16,
    pub helper_version: String,
    pub build_target: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum HelperState {
    Missing,
    Available {
        expected_sha256: String,
        actual_sha256: String,
        integrity_matches: bool,
        helper_version: String,
        protocol_version: u16,
        build_target: String,
    },
    Invalid {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeOptions {
    pub helper: HelperState,
    pub managed_runtime_available: bool,
    pub managed_runtime_path: Option<PathBuf>,
    pub compatdata_path: PathBuf,
    pub prefix_path: PathBuf,
    pub prefix_mode: PrefixMode,
    pub warning: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandshakeState {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathVisibility {
    MappedAndReadable,
    MappedButUnreadable,
    Unmapped,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeProcessSummary {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub exit_code: i32,
    pub elapsed_millis: u128,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeTestResult {
    /// The bridge-only runtime; this is deliberately not GameRuntimePlan.selection.
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub prefix_mode: PrefixMode,
    pub compatdata_path: PathBuf,
    pub helper_path: PathBuf,
    pub expected_helper_sha256: String,
    pub actual_helper_sha256: String,
    pub helper_version: String,
    pub helper_protocol_version: u16,
    pub handshake: HandshakeState,
    pub runtime_path_linux: PathBuf,
    pub runtime_path_windows: String,
    pub path_visibility: PathVisibility,
    pub runtime_manifest_sha256: String,
    pub runtime_manifest_hash_matches: bool,
    pub environment_marker_matches: bool,
    pub process: BridgeProcessSummary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeErrorCode {
    RuntimeNotFound,
    ProtonExecutableMissing,
    HelperMissing,
    HelperManifestInvalid,
    HelperHashMismatch,
    ProtocolMismatch,
    NonceMismatch,
    Timeout,
    NonZeroExit,
    MalformedResponse,
    PathNotVisible,
    UnsafePath,
    UnsafePrefixContext,
    PermissionDenied,
    ProcessLaunchFailed,
    OutputLimitExceeded,
    LoaderArtifactsMissing,
    LoaderProvenanceInvalid,
    LoaderTestFailed,
    UserAcknowledgementRequired,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeError {
    pub code: BridgeErrorCode,
    pub detail: String,
    pub stderr: Option<String>,
}

impl BridgeError {
    fn new(code: BridgeErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
            stderr: None,
        }
    }

    fn with_stderr(mut self, stderr: String) -> Self {
        if !stderr.trim().is_empty() {
            self.stderr = Some(stderr);
        }
        self
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl BridgeErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeNotFound => "runtime_not_found",
            Self::ProtonExecutableMissing => "proton_executable_missing",
            Self::HelperMissing => "helper_missing",
            Self::HelperManifestInvalid => "helper_manifest_invalid",
            Self::HelperHashMismatch => "helper_hash_mismatch",
            Self::ProtocolMismatch => "protocol_mismatch",
            Self::NonceMismatch => "nonce_mismatch",
            Self::Timeout => "timeout",
            Self::NonZeroExit => "non_zero_exit",
            Self::MalformedResponse => "malformed_response",
            Self::PathNotVisible => "path_not_visible",
            Self::UnsafePath => "unsafe_path",
            Self::UnsafePrefixContext => "unsafe_prefix_context",
            Self::PermissionDenied => "permission_denied",
            Self::ProcessLaunchFailed => "process_launch_failed",
            Self::OutputLimitExceeded => "output_limit_exceeded",
            Self::LoaderArtifactsMissing => "loader_artifacts_missing",
            Self::LoaderProvenanceInvalid => "loader_provenance_invalid",
            Self::LoaderTestFailed => "loader_test_failed",
            Self::UserAcknowledgementRequired => "user_acknowledgement_required",
            Self::Io => "io",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeTestConfig {
    pub managed_root: PathBuf,
    pub runtime_root: PathBuf,
    pub explicit_runtime: ExplicitBridgeRuntime,
    pub steam_client_install_path: PathBuf,
    pub prefix_mode: PrefixMode,
}

#[derive(Debug, Clone)]
struct ProcessInvocation {
    executable: PathBuf,
    args: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
    working_directory: PathBuf,
    input: Vec<u8>,
    timeout: Duration,
    output_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessOutput {
    status: ExitStatus,
    elapsed_millis: u128,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

#[derive(Debug)]
enum ProcessFailure {
    TimedOut(ProcessOutput),
    Launch(io::Error),
    OutputRead(io::Error),
}

trait BridgeProcessExecutor {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure>;
}

#[derive(Debug, Default)]
struct NativeProcessExecutor;

impl BridgeProcessExecutor for NativeProcessExecutor {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
        let mut command = Command::new(&invocation.executable);
        command
            .env_clear()
            .args(&invocation.args)
            .envs(&invocation.environment)
            .current_dir(&invocation.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);

        let started = Instant::now();
        let mut child = command.spawn().map_err(ProcessFailure::Launch)?;
        let stdout = child.stdout.take().expect("stdout pipe configured");
        let stderr = child.stderr.take().expect("stderr pipe configured");
        let output_limit = invocation.output_limit;
        let stdout_reader = thread::spawn(move || read_capped(stdout, output_limit));
        let stderr_reader = thread::spawn(move || read_capped(stderr, output_limit));

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(error) = stdin.write_all(&invocation.input) {
                warn!(error = %error, "Bridge helper closed stdin before request completed");
            }
        }

        let deadline = started + invocation.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(20));
                }
                Ok(None) => {
                    let status = terminate_owned_process_group(&mut child)
                        .map_err(ProcessFailure::OutputRead)?;
                    let output = collect_readers(
                        status,
                        started.elapsed().as_millis(),
                        stdout_reader,
                        stderr_reader,
                    )
                    .map_err(ProcessFailure::OutputRead)?;
                    return Err(ProcessFailure::TimedOut(output));
                }
                Err(error) => {
                    let _ = terminate_owned_process_group(&mut child);
                    return Err(ProcessFailure::Launch(error));
                }
            }
        };
        let output = collect_readers(
            status,
            started.elapsed().as_millis(),
            stdout_reader,
            stderr_reader,
        )
        .map_err(ProcessFailure::OutputRead)?;
        Ok(output)
    }
}

fn terminate_owned_process_group(child: &mut Child) -> io::Result<ExitStatus> {
    let process_group = -(child.id() as i32);
    // The child was placed in its own process group; Proton and its Wine child inherit it.
    unsafe {
        libc::kill(process_group, libc::SIGTERM);
    }
    let grace_deadline = Instant::now() + Duration::from_millis(300);
    while Instant::now() < grace_deadline {
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    unsafe {
        libc::kill(process_group, libc::SIGKILL);
    }
    let _ = child.kill();
    child.wait()
}

fn collect_readers(
    status: ExitStatus,
    elapsed_millis: u128,
    stdout_reader: thread::JoinHandle<io::Result<CappedOutput>>,
    stderr_reader: thread::JoinHandle<io::Result<CappedOutput>>,
) -> io::Result<ProcessOutput> {
    let stdout = stdout_reader
        .join()
        .map_err(|_| io::Error::other("stdout reader panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| io::Error::other("stderr reader panicked"))??;
    Ok(ProcessOutput {
        status,
        elapsed_millis,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct CappedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_capped(mut reader: impl Read, limit: usize) -> io::Result<CappedOutput> {
    let mut bytes = Vec::with_capacity(limit.min(16 * 1024));
    let mut buffer = [0u8; 8192];
    let mut truncated = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let available = limit.saturating_sub(bytes.len());
        let keep = count.min(available);
        bytes.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }
    Ok(CappedOutput { bytes, truncated })
}

/// Reports bridge availability and expected/actual helper hashes without running Proton.
pub fn inspect_bridge_options(managed_root: &Path, runtime_root: Option<&Path>) -> BridgeOptions {
    let root = match managed_root.canonicalize() {
        Ok(root) => root,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return BridgeOptions {
                helper: HelperState::Missing,
                managed_runtime_available: false,
                managed_runtime_path: None,
                compatdata_path: managed_root.join("test-prefixes/bridge-v1/compatdata"),
                prefix_path: managed_root.join("test-prefixes/bridge-v1/compatdata/pfx"),
                prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
                warning: prefix_warning(),
            };
        }
        Err(error) => {
            return BridgeOptions {
                helper: HelperState::Invalid {
                    detail: format!("managed storage unavailable: {error}"),
                },
                managed_runtime_available: false,
                managed_runtime_path: None,
                compatdata_path: managed_root.join("test-prefixes/bridge-v1/compatdata"),
                prefix_path: managed_root.join("test-prefixes/bridge-v1/compatdata/pfx"),
                prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
                warning: prefix_warning(),
            };
        }
    };
    let helper_path = root.join("helpers").join(BRIDGE_HELPER_FILE);
    let helper_manifest_path = root.join("helpers").join(BRIDGE_HELPER_MANIFEST_FILE);
    let helper = inspect_helper(&root, &helper_path, &helper_manifest_path);
    let managed_runtime_path =
        runtime_root.and_then(|path| safe_existing_directory(&root, path).ok());
    BridgeOptions {
        helper,
        managed_runtime_available: managed_runtime_path.is_some(),
        managed_runtime_path,
        compatdata_path: root.join("test-prefixes/bridge-v1/compatdata"),
        prefix_path: root.join("test-prefixes/bridge-v1/compatdata/pfx"),
        prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
        warning: prefix_warning(),
    }
}

fn inspect_helper(root: &Path, helper_path: &Path, manifest_path: &Path) -> HelperState {
    let helper_meta = match fs::symlink_metadata(helper_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return HelperState::Missing,
        Err(error) => {
            return HelperState::Invalid {
                detail: format!("helper could not be inspected: {error}"),
            }
        }
    };
    if helper_meta.file_type().is_symlink()
        || !helper_meta.is_file()
        || helper_meta.len() > HELPER_MAX_BYTES
    {
        return HelperState::Invalid {
            detail: "helper must be a regular file no larger than 64 MiB".into(),
        };
    }
    if safe_existing_file(root, helper_path).is_err() {
        return HelperState::Invalid {
            detail: "helper path is outside managed storage or contains a symlink".into(),
        };
    }
    let manifest_bytes = match read_bounded_file(manifest_path, HELPER_MANIFEST_MAX_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            return HelperState::Invalid {
                detail: format!("helper manifest is unavailable or invalid: {error}"),
            }
        }
    };
    let manifest: BridgeHelperManifest = match serde_json::from_slice(&manifest_bytes) {
        Ok(manifest) => manifest,
        Err(error) => {
            return HelperState::Invalid {
                detail: format!("helper manifest is not valid JSON: {error}"),
            }
        }
    };
    if manifest.protocol_version != PROTOCOL_VERSION || manifest.helper_version != HELPER_VERSION {
        return HelperState::Invalid {
            detail: "helper manifest protocol/version does not match this LXMI build".into(),
        };
    }
    let actual = match sha256_file(helper_path, HELPER_MAX_BYTES) {
        Ok(hash) => hash,
        Err(error) => {
            return HelperState::Invalid {
                detail: format!("helper could not be hashed: {error}"),
            }
        }
    };
    HelperState::Available {
        integrity_matches: actual.eq_ignore_ascii_case(&manifest.sha256),
        expected_sha256: manifest.sha256,
        actual_sha256: actual,
        helper_version: manifest.helper_version,
        protocol_version: manifest.protocol_version,
        build_target: manifest.build_target,
    }
}

/// The only directory LXMI creates before Proton starts is its own empty compatdata root.
/// Proton itself creates `pfx/`; LXMI never fabricates a Wine prefix layout.
pub fn prepare_isolated_test_context(managed_root: &Path) -> Result<PathBuf, BridgeError> {
    let root = managed_root.canonicalize().map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::RuntimeNotFound,
            format!("managed root unavailable: {error}"),
        )
    })?;
    if !root.is_dir() {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "managed root is not a directory",
        ));
    }
    let context_root = root.join("test-prefixes/bridge-v1");
    ensure_controlled_directory(&root, &context_root)?;
    validate_or_create_prefix_marker(&context_root)?;
    let compatdata = context_root.join("compatdata");
    ensure_controlled_directory(&root, &compatdata)?;
    let prefix = compatdata.join("pfx");
    if let Ok(metadata) = fs::symlink_metadata(&prefix) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                "existing test-prefix pfx path is not a regular directory",
            ));
        }
    }

    for directory in ["home", "config", "cache", "data"] {
        ensure_controlled_directory(&root, &context_root.join(directory))?;
    }
    Ok(compatdata)
}

fn validate_or_create_prefix_marker(context_root: &Path) -> Result<(), BridgeError> {
    let marker = context_root.join(PREFIX_MARKER_FILE);
    match fs::symlink_metadata(&marker) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePath,
                    "LXMI test-prefix marker must be a regular file",
                ));
            }
            let bytes = read_bounded_file(&marker, 4096)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            let expected = b"{\"owner\":\"LXMI\",\"kind\":\"bridge-test-prefix\",\"schema\":1}\n";
            if bytes != expected {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePrefixContext,
                    "test-prefix metadata does not match the LXMI-owned bridge context",
                ));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&marker)
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            file.write_all(b"{\"owner\":\"LXMI\",\"kind\":\"bridge-test-prefix\",\"schema\":1}\n")
                .and_then(|()| file.sync_all())
                .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
        }
        Err(error) => {
            return Err(BridgeError::new(BridgeErrorCode::Io, error.to_string()));
        }
    }
    Ok(())
}

fn ensure_controlled_directory(root: &Path, path: &Path) -> Result<(), BridgeError> {
    if !path.starts_with(root) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "refusing to create a directory outside LXMI managed storage",
        ));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|error| BridgeError::new(BridgeErrorCode::UnsafePath, error.to_string()))?;
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => cursor.push(part),
            _ => {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePath,
                    "managed directory path must contain only normal components",
                ));
            }
        }
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePath,
                    format!("unsafe non-directory component: {}", cursor.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&cursor)
                    .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
            }
            Err(error) => return Err(BridgeError::new(BridgeErrorCode::Io, error.to_string())),
        }
    }
    Ok(())
}

pub fn run_bridge_test(config: &BridgeTestConfig) -> Result<BridgeTestResult, BridgeError> {
    run_bridge_test_with_executor(config, &NativeProcessExecutor)
}

fn run_bridge_test_with_executor(
    config: &BridgeTestConfig,
    executor: &dyn BridgeProcessExecutor,
) -> Result<BridgeTestResult, BridgeError> {
    info!("Runtime bridge test started");
    if config.prefix_mode != PrefixMode::IsolatedTemporaryPrefix {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "bridge execution is restricted to the LXMI-owned isolated test prefix; game compatdata is never accepted",
        ));
    }
    let managed_root = config.managed_root.canonicalize().map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::RuntimeNotFound,
            format!("managed root unavailable: {error}"),
        )
    })?;
    let runtime_root = safe_existing_directory(&managed_root, &config.runtime_root)?;
    let helper_path = managed_root.join("helpers").join(BRIDGE_HELPER_FILE);
    let helper_manifest_path = managed_root
        .join("helpers")
        .join(BRIDGE_HELPER_MANIFEST_FILE);
    let helper_path = safe_existing_file(&managed_root, &helper_path).map_err(|error| {
        if error.code == BridgeErrorCode::RuntimeNotFound {
            BridgeError::new(BridgeErrorCode::HelperMissing, error.detail)
        } else {
            error
        }
    })?;
    let helper_manifest_path =
        safe_existing_file(&managed_root, &helper_manifest_path).map_err(|_| {
            BridgeError::new(
                BridgeErrorCode::HelperManifestInvalid,
                "managed helper manifest is missing or unsafe",
            )
        })?;
    let helper_manifest = read_helper_manifest(&helper_manifest_path)?;
    if helper_manifest.protocol_version != PROTOCOL_VERSION
        || helper_manifest.helper_version != HELPER_VERSION
        || !helper_manifest
            .build_target
            .starts_with("x86_64-pc-windows-")
    {
        return Err(BridgeError::new(
            BridgeErrorCode::HelperManifestInvalid,
            "helper manifest protocol, version, or build target is unsupported",
        ));
    }
    let actual_helper_sha256 = sha256_file(&helper_path, HELPER_MAX_BYTES)
        .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    if !actual_helper_sha256.eq_ignore_ascii_case(&helper_manifest.sha256) {
        return Err(BridgeError::new(
            BridgeErrorCode::HelperHashMismatch,
            "helper SHA-256 does not match its LXMI build manifest",
        ));
    }

    let proton_script = validate_proton_script(&config.explicit_runtime.proton_script)?;
    let steam_root = config
        .steam_client_install_path
        .canonicalize()
        .map_err(|error| {
            BridgeError::new(
                BridgeErrorCode::RuntimeNotFound,
                format!("Steam root unavailable: {error}"),
            )
        })?;
    if !steam_root.is_dir() {
        return Err(BridgeError::new(
            BridgeErrorCode::RuntimeNotFound,
            "Steam client install path is not a directory",
        ));
    }

    let compatdata_path = prepare_isolated_test_context(&managed_root)?;
    let runtime_relative_path = runtime_root
        .strip_prefix(&managed_root)
        .map_err(|_| BridgeError::new(BridgeErrorCode::UnsafePath, "runtime escaped managed root"))?
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if !safe_relative_path(&runtime_relative_path) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "runtime path could not be represented safely for the helper protocol",
        ));
    }
    let marker_path = runtime_root.join(MARKER_FILE);
    let marker_metadata = fs::symlink_metadata(&marker_path)
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if marker_metadata.file_type().is_symlink()
        || !marker_metadata.is_file()
        || marker_metadata.len() > RESPONSE_LIMIT_BYTES as u64
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "managed runtime marker must be a bounded regular file",
        ));
    }
    let expected_runtime_manifest_sha256 =
        sha256_file(&marker_path, RESPONSE_LIMIT_BYTES as u64)
            .map_err(|error| BridgeError::new(BridgeErrorCode::Io, error.to_string()))?;
    let nonce = random_nonce()?;
    let request = BridgeRequest {
        protocol_version: PROTOCOL_VERSION,
        operation: lxmi_bridge_protocol::BridgeOperation::InspectManagedRuntime,
        runtime_relative_path,
        nonce: nonce.clone(),
    };
    let input = serde_json::to_vec(&request)
        .map_err(|error| BridgeError::new(BridgeErrorCode::MalformedResponse, error.to_string()))?;
    if input.len() > REQUEST_LIMIT_BYTES {
        return Err(BridgeError::new(
            BridgeErrorCode::OutputLimitExceeded,
            "serialized bridge request exceeds 64 KiB",
        ));
    }

    let context_root = compatdata_path.parent().ok_or_else(|| {
        BridgeError::new(BridgeErrorCode::UnsafePath, "test context has no parent")
    })?;
    let mut environment = BTreeMap::new();
    if let Some(path) = env::var_os("PATH") {
        environment.insert(OsString::from("PATH"), path);
    }
    if let Some(lang) = env::var_os("LANG") {
        environment.insert(OsString::from("LANG"), lang);
    }
    environment.insert(
        OsString::from("HOME"),
        context_root.join("home").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_CONFIG_HOME"),
        context_root.join("config").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_CACHE_HOME"),
        context_root.join("cache").into_os_string(),
    );
    environment.insert(
        OsString::from("XDG_DATA_HOME"),
        context_root.join("data").into_os_string(),
    );
    environment.insert(
        OsString::from("STEAM_COMPAT_CLIENT_INSTALL_PATH"),
        steam_root.as_os_str().to_owned(),
    );
    environment.insert(
        OsString::from("STEAM_COMPAT_DATA_PATH"),
        compatdata_path.as_os_str().to_owned(),
    );
    environment.insert(
        OsString::from(ENVIRONMENT_MARKER),
        OsString::from(ENVIRONMENT_MARKER_VALUE),
    );
    environment.insert(OsString::from("WINEDEBUG"), OsString::from("-all"));

    let invocation = ProcessInvocation {
        executable: proton_script.clone(),
        args: vec![
            OsString::from("runinprefix"),
            helper_path.as_os_str().to_owned(),
        ],
        environment,
        working_directory: context_root.to_path_buf(),
        input,
        timeout: BRIDGE_TEST_TIMEOUT,
        output_limit: BRIDGE_OUTPUT_LIMIT_BYTES,
    };
    info!(runtime = %config.explicit_runtime.display_name, "Explicit bridge test runtime selected");
    info!(helper = %helper_path.display(), "Windows bridge helper launched");
    let output = executor
        .execute(&invocation)
        .map_err(|failure| match failure {
            ProcessFailure::TimedOut(output) => BridgeError::new(
                BridgeErrorCode::Timeout,
                format!(
                    "bridge helper exceeded {} seconds",
                    BRIDGE_TEST_TIMEOUT.as_secs()
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
            format!(
                "bridge process output exceeded the {} byte capture limit",
                BRIDGE_OUTPUT_LIMIT_BYTES
            ),
        )
        .with_stderr(stderr));
    }
    let response: BridgeResponse =
        serde_json::from_slice(stdout.trim().as_bytes()).map_err(|error| {
            BridgeError::new(
                BridgeErrorCode::MalformedResponse,
                format!("helper stdout did not contain one valid JSON response: {error}"),
            )
            .with_stderr(stderr.clone())
        })?;
    let exit_code = output.status.code().unwrap_or(-1);
    let summary = BridgeProcessSummary {
        executable: proton_script,
        arguments: vec!["runinprefix".into(), helper_path.display().to_string()],
        exit_code,
        elapsed_millis: output.elapsed_millis,
        stdout_truncated: output.stdout_truncated,
        stderr_truncated: output.stderr_truncated,
        stdout,
        stderr: stderr.clone(),
    };
    if response.protocol_version != PROTOCOL_VERSION {
        return Err(BridgeError::new(
            BridgeErrorCode::ProtocolMismatch,
            format!(
                "helper protocol {} does not match LXMI protocol {PROTOCOL_VERSION}",
                response.protocol_version
            ),
        )
        .with_stderr(stderr));
    }
    if response.nonce.as_deref() != Some(nonce.as_str()) {
        return Err(BridgeError::new(
            BridgeErrorCode::NonceMismatch,
            "helper response nonce did not match this invocation",
        )
        .with_stderr(stderr));
    }
    if exit_code != 0 {
        return Err(BridgeError::new(
            BridgeErrorCode::NonZeroExit,
            format!("bridge helper exited with code {exit_code}"),
        )
        .with_stderr(stderr));
    }
    if !response.success {
        return Err(BridgeError::new(
            BridgeErrorCode::PathNotVisible,
            response
                .diagnostic
                .as_ref()
                .map(|diagnostic| diagnostic.detail.clone())
                .unwrap_or_else(|| "helper could not read the managed runtime marker".into()),
        )
        .with_stderr(stderr));
    }
    if response.helper_version != helper_manifest.helper_version
        || response.build_target != "x86_64-windows"
        || response.helper_sha256.as_deref() != Some(actual_helper_sha256.as_str())
    {
        return Err(BridgeError::new(
            BridgeErrorCode::HelperHashMismatch,
            "helper self-reported identity does not match the staged helper manifest/hash",
        )
        .with_stderr(stderr));
    }
    if !response.environment_marker_matches {
        return Err(BridgeError::new(
            BridgeErrorCode::MalformedResponse,
            "the non-secret LXMI test environment marker did not propagate",
        )
        .with_stderr(stderr));
    }
    if !response.runtime_visible || !response.runtime_manifest_readable {
        return Err(BridgeError::new(
            BridgeErrorCode::PathNotVisible,
            "helper started but could not read the managed runtime manifest",
        )
        .with_stderr(stderr));
    }
    let runtime_path_windows = response.runtime_path_windows.ok_or_else(|| {
        BridgeError::new(
            BridgeErrorCode::PathNotVisible,
            "helper did not report its Windows-visible runtime path",
        )
    })?;
    let runtime_manifest_sha256 = response.runtime_manifest_sha256.ok_or_else(|| {
        BridgeError::new(
            BridgeErrorCode::MalformedResponse,
            "helper omitted runtime manifest SHA-256",
        )
    })?;
    if !runtime_manifest_sha256.eq_ignore_ascii_case(&expected_runtime_manifest_sha256) {
        return Err(BridgeError::new(
            BridgeErrorCode::PathNotVisible,
            "the helper read a different runtime manifest than the one inspected on Linux",
        )
        .with_stderr(stderr));
    }
    info!("Runtime bridge handshake succeeded");
    info!("Managed runtime path visible to helper");
    info!(exit_code, "Windows bridge helper exited");
    info!("Runtime bridge test completed");

    let mut warnings = vec![
        "Este runtime fue seleccionado solo para la prueba; GameRuntimePlan.selected permanece Unknown.".into(),
        "El nonce correlaciona request/response y no autentica al helper.".into(),
        "Proton crea o actualiza el prefix exclusivo de LXMI. Nunca se utiliza compatdata de ZZZ.".into(),
    ];
    if !stderr.trim().is_empty() {
        warnings.push(
            "Proton/helper escribieron diagnósticos en stderr; están disponibles en el resultado."
                .into(),
        );
    }
    Ok(BridgeTestResult {
        explicit_runtime: config.explicit_runtime.clone(),
        prefix_mode: config.prefix_mode,
        compatdata_path,
        helper_path,
        expected_helper_sha256: helper_manifest.sha256,
        actual_helper_sha256,
        helper_version: response.helper_version,
        helper_protocol_version: response.protocol_version,
        handshake: HandshakeState::Succeeded,
        runtime_path_linux: runtime_root,
        runtime_path_windows,
        path_visibility: PathVisibility::MappedAndReadable,
        runtime_manifest_sha256,
        runtime_manifest_hash_matches: true,
        environment_marker_matches: response.environment_marker_matches,
        process: summary,
        warnings,
    })
}

fn validate_proton_script(path: &Path) -> Result<PathBuf, BridgeError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        BridgeError::new(BridgeErrorCode::ProtonExecutableMissing, error.to_string())
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BridgeError::new(
            BridgeErrorCode::ProtonExecutableMissing,
            "selected Proton entrypoint must be a regular non-symlink file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(BridgeError::new(
                BridgeErrorCode::ProtonExecutableMissing,
                "selected Proton entrypoint is not executable",
            ));
        }
    }
    path.canonicalize().map_err(|error| {
        BridgeError::new(BridgeErrorCode::ProtonExecutableMissing, error.to_string())
    })
}

fn safe_existing_directory(root: &Path, path: &Path) -> Result<PathBuf, BridgeError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "managed runtime must be a real directory, not a symlink",
        ));
    }
    reject_symlink_components(root, path)?;
    let canonical = path
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if !canonical.starts_with(root) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "managed runtime is outside the LXMI data root",
        ));
    }
    Ok(canonical)
}

fn safe_existing_file(root: &Path, path: &Path) -> Result<PathBuf, BridgeError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "expected a regular non-symlink file",
        ));
    }
    reject_symlink_components(root, path)?;
    let canonical = path
        .canonicalize()
        .map_err(|error| BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string()))?;
    if !canonical.starts_with(root) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "file is outside LXMI managed storage",
        ));
    }
    Ok(canonical)
}

fn reject_symlink_components(root: &Path, path: &Path) -> Result<(), BridgeError> {
    if !path.starts_with(root) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "path is outside LXMI managed storage",
        ));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|error| BridgeError::new(BridgeErrorCode::UnsafePath, error.to_string()))?;
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => cursor.push(part),
            _ => {
                return Err(BridgeError::new(
                    BridgeErrorCode::UnsafePath,
                    "managed path must contain only normal components",
                ));
            }
        }
        let metadata = fs::symlink_metadata(&cursor).map_err(|error| {
            BridgeError::new(BridgeErrorCode::RuntimeNotFound, error.to_string())
        })?;
        if metadata.file_type().is_symlink() {
            return Err(BridgeError::new(
                BridgeErrorCode::UnsafePath,
                format!(
                    "symlink is not allowed in managed path: {}",
                    cursor.display()
                ),
            ));
        }
    }
    Ok(())
}

fn read_helper_manifest(path: &Path) -> Result<BridgeHelperManifest, BridgeError> {
    let bytes = read_bounded_file(path, HELPER_MANIFEST_MAX_BYTES).map_err(|error| {
        BridgeError::new(BridgeErrorCode::HelperManifestInvalid, error.to_string())
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        BridgeError::new(BridgeErrorCode::HelperManifestInvalid, error.to_string())
    })
}

fn read_bounded_file(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "not a bounded regular file",
        ));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeded size limit",
        ));
    }
    Ok(bytes)
}

fn sha256_file(path: &Path, limit: u64) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "file exceeded hash size limit",
            ));
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn random_nonce() -> Result<String, BridgeError> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::Io,
            format!("secure nonce generation failed: {error}"),
        )
    })?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') || value.contains(':') || value.starts_with('/') {
        return false;
    }
    Path::new(value)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn prefix_warning() -> String {
    "Bridge test only: Proton will create/update the LXMI-owned isolated prefix. Game compatdata is never used. The bridge runtime is separate from the game's selected Proton.".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lxmi_bridge_protocol::BridgeOperation;
    use std::{
        ffi::OsStr,
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = env::temp_dir().join(format!(
                "lxmi-bridge-domain-{}-{now}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct FakeExecutor {
        response: Option<BridgeResponse>,
        failure: Option<BridgeErrorCode>,
        invocation: std::sync::Mutex<Option<ProcessInvocation>>,
    }

    impl BridgeProcessExecutor for FakeExecutor {
        fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
            *self.invocation.lock().unwrap() = Some(invocation.clone());
            if let Some(code) = self.failure {
                return Err(if code == BridgeErrorCode::Timeout {
                    ProcessFailure::TimedOut(fake_process_output())
                } else {
                    ProcessFailure::Launch(io::Error::other(code.as_str()))
                });
            }
            let request: BridgeRequest = serde_json::from_slice(&invocation.input).unwrap();
            let mut response = self.response.clone().unwrap();
            if response.nonce.is_none() {
                response.nonce = Some(request.nonce);
            }
            let bytes = serde_json::to_vec(&response).unwrap();
            Ok(ProcessOutput {
                status: fake_status(0),
                elapsed_millis: 2,
                stdout: bytes,
                stderr: Vec::new(),
                stdout_truncated: false,
                stderr_truncated: false,
            })
        }
    }

    fn fake_status(code: i32) -> ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(code << 8)
        }
        #[cfg(not(unix))]
        {
            let _ = code;
            panic!("tests currently target Linux")
        }
    }

    fn fake_process_output() -> ProcessOutput {
        ProcessOutput {
            status: fake_status(1),
            elapsed_millis: BRIDGE_TEST_TIMEOUT.as_millis(),
            stdout: Vec::new(),
            stderr: b"timed out".to_vec(),
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn configured_fixture() -> (TestDirectory, BridgeTestConfig, BridgeHelperManifest) {
        let temp = TestDirectory::new();
        let root = temp.0.join("lxmi");
        let runtime = root.join("runtimes/zenless-zone-zero/zzmi/fixture");
        let helpers = root.join("helpers");
        let proton = temp.0.join("Proton/proton");
        let steam = temp.0.join("Steam");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&helpers).unwrap();
        fs::create_dir_all(proton.parent().unwrap()).unwrap();
        fs::create_dir_all(&steam).unwrap();
        fs::write(runtime.join(MARKER_FILE), br#"{"runtime_id":"fixture"}"#).unwrap();
        fs::write(&proton, b"proton fixture").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&proton, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let helper_path = helpers.join(BRIDGE_HELPER_FILE);
        fs::write(&helper_path, b"windows helper fixture bytes").unwrap();
        let hash = sha256_file(&helper_path, HELPER_MAX_BYTES).unwrap();
        let manifest = BridgeHelperManifest {
            protocol_version: PROTOCOL_VERSION,
            helper_version: HELPER_VERSION.to_owned(),
            build_target: "x86_64-pc-windows-gnu".into(),
            sha256: hash,
        };
        fs::write(
            helpers.join(BRIDGE_HELPER_MANIFEST_FILE),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let config = BridgeTestConfig {
            managed_root: root,
            runtime_root: runtime,
            explicit_runtime: ExplicitBridgeRuntime {
                display_name: "Proton test fixture".into(),
                version: Some("fixture-version".into()),
                proton_script: proton,
            },
            steam_client_install_path: steam,
            prefix_mode: PrefixMode::IsolatedTemporaryPrefix,
        };
        (temp, config, manifest)
    }

    fn successful_fake(config: &BridgeTestConfig, manifest: &BridgeHelperManifest) -> FakeExecutor {
        let root = config.managed_root.canonicalize().unwrap();
        let helper_path = root.join("helpers").join(BRIDGE_HELPER_FILE);
        let marker = config.runtime_root.join(MARKER_FILE);
        let marker_hash = sha256_file(&marker, RESPONSE_LIMIT_BYTES as u64).unwrap();
        let helper_hash = sha256_file(&helper_path, HELPER_MAX_BYTES).unwrap();
        FakeExecutor {
            response: Some(BridgeResponse {
                protocol_version: PROTOCOL_VERSION,
                success: true,
                helper_version: manifest.helper_version.clone(),
                build_target: "x86_64-windows".into(),
                helper_sha256: Some(helper_hash),
                runtime_visible: true,
                runtime_manifest_readable: true,
                runtime_path_windows: Some("Z:\\home\\lxmi\\runtimes\\fixture".into()),
                runtime_manifest_sha256: Some(marker_hash),
                environment_marker_matches: true,
                nonce: None,
                diagnostic: None,
            }),
            failure: None,
            invocation: std::sync::Mutex::new(None),
        }
    }

    fn run_fake_success(
        config: &BridgeTestConfig,
        manifest: &BridgeHelperManifest,
    ) -> BridgeTestResult {
        let executor = successful_fake(config, manifest);
        run_bridge_test_with_executor(config, &executor).unwrap()
    }

    #[test]
    fn isolated_test_context_creates_only_lxmi_owned_prefix_root() {
        let temp = TestDirectory::new();
        let root = temp.0.join("lxmi");
        fs::create_dir_all(&root).unwrap();
        let compatdata = prepare_isolated_test_context(&root).unwrap();
        assert!(compatdata.is_dir());
        assert!(!compatdata.join("pfx").exists());
        assert!(compatdata
            .parent()
            .unwrap()
            .join(PREFIX_MARKER_FILE)
            .is_file());
    }

    #[test]
    fn refuses_game_compatdata_context() {
        let (_temp, mut config, _) = configured_fixture();
        config.prefix_mode = PrefixMode::ExistingGameCompatdataReadOnlyContext;
        let fake = FakeExecutor {
            response: None,
            failure: None,
            invocation: std::sync::Mutex::new(None),
        };
        let error = run_bridge_test_with_executor(&config, &fake).unwrap_err();
        assert_eq!(error.code, BridgeErrorCode::UnsafePrefixContext);
        assert!(fake.invocation.lock().unwrap().is_none());
    }

    #[test]
    fn explicit_bridge_runtime_does_not_change_game_runtime_selection() {
        let (_temp, config, manifest) = configured_fixture();
        let result = run_fake_success(&config, &manifest);
        assert_eq!(result.explicit_runtime.display_name, "Proton test fixture");
        assert_eq!(result.handshake, HandshakeState::Succeeded);
        assert_eq!(result.path_visibility, PathVisibility::MappedAndReadable);
        assert_eq!(result.prefix_mode, PrefixMode::IsolatedTemporaryPrefix);
        assert_eq!(result.process.arguments[0], "runinprefix");
        assert_eq!(
            result.process.arguments[1],
            result.helper_path.display().to_string()
        );
    }

    #[test]
    fn only_managed_helper_and_runtime_paths_are_accepted() {
        let (_temp, mut config, _) = configured_fixture();
        config.runtime_root = PathBuf::from("/tmp/outside-runtime");
        let error = run_bridge_test(&config).unwrap_err();
        assert_eq!(error.code, BridgeErrorCode::RuntimeNotFound);
    }

    #[test]
    fn changed_helper_hash_is_rejected_before_process_launch() {
        let (_temp, config, _) = configured_fixture();
        fs::write(
            config.managed_root.join("helpers").join(BRIDGE_HELPER_FILE),
            b"tampered helper",
        )
        .unwrap();
        let fake = FakeExecutor {
            response: None,
            failure: None,
            invocation: std::sync::Mutex::new(None),
        };
        let error = run_bridge_test_with_executor(&config, &fake).unwrap_err();
        assert_eq!(error.code, BridgeErrorCode::HelperHashMismatch);
        assert!(fake.invocation.lock().unwrap().is_none());
    }

    #[test]
    fn timeout_has_a_distinct_typed_error() {
        let (_temp, config, _) = configured_fixture();
        let fake = FakeExecutor {
            response: None,
            failure: Some(BridgeErrorCode::Timeout),
            invocation: std::sync::Mutex::new(None),
        };
        let error = run_bridge_test_with_executor(&config, &fake).unwrap_err();
        assert_eq!(error.code, BridgeErrorCode::Timeout);
        assert_eq!(error.stderr.as_deref(), Some("timed out"));
    }

    #[test]
    fn nonce_mismatch_and_protocol_mismatch_are_rejected() {
        let (_temp, config, manifest) = configured_fixture();
        let mut fake = successful_fake(&config, &manifest);
        let mut response = fake.response.clone().unwrap();
        response.nonce = Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into());
        fake.response = Some(response);
        assert_eq!(
            run_bridge_test_with_executor(&config, &fake)
                .unwrap_err()
                .code,
            BridgeErrorCode::NonceMismatch
        );

        let mut fake = successful_fake(&config, &manifest);
        let mut response = fake.response.clone().unwrap();
        response.protocol_version += 1;
        response.nonce = Some("0123456789abcdef0123456789abcdef".into());
        fake.response = Some(response);
        assert_eq!(
            run_bridge_test_with_executor(&config, &fake)
                .unwrap_err()
                .code,
            BridgeErrorCode::ProtocolMismatch
        );
    }

    #[test]
    fn invocation_uses_direct_proton_argv_and_controlled_environment() {
        let (_temp, config, manifest) = configured_fixture();
        let fake = successful_fake(&config, &manifest);
        let executor = FakeExecutor {
            response: fake.response.clone(),
            failure: None,
            invocation: std::sync::Mutex::new(None),
        };
        let _ = run_bridge_test_with_executor(&config, &executor).unwrap();
        let invocation = executor.invocation.lock().unwrap().clone().unwrap();
        assert_eq!(invocation.executable, config.explicit_runtime.proton_script);
        assert_eq!(invocation.args[0], OsStr::new("runinprefix"));
        assert!(invocation.args[1]
            .to_string_lossy()
            .ends_with(BRIDGE_HELPER_FILE));
        assert_eq!(
            invocation.environment[OsStr::new("STEAM_COMPAT_CLIENT_INSTALL_PATH")],
            config
                .steam_client_install_path
                .canonicalize()
                .unwrap()
                .as_os_str()
        );
        assert!(invocation
            .environment
            .contains_key(OsStr::new("STEAM_COMPAT_DATA_PATH")));
        assert!(invocation
            .environment
            .contains_key(OsStr::new(ENVIRONMENT_MARKER)));
        assert!(!invocation
            .environment
            .contains_key(OsStr::new("WINEDLLOVERRIDES")));
    }

    #[test]
    fn output_reader_caps_storage_but_drains_the_stream() {
        let source = vec![b'x'; 1024];
        let output = read_capped(&source[..], 64).unwrap();
        assert_eq!(output.bytes.len(), 64);
        assert!(output.truncated);
    }

    #[test]
    fn inspect_options_hashes_helper_without_running_proton() {
        let (_temp, config, manifest) = configured_fixture();
        let root = config.managed_root.canonicalize().unwrap();
        let options = inspect_bridge_options(&root, Some(&config.runtime_root));
        match options.helper {
            HelperState::Available {
                integrity_matches,
                expected_sha256,
                actual_sha256,
                ..
            } => {
                assert!(integrity_matches);
                assert_eq!(expected_sha256, manifest.sha256);
                assert_eq!(actual_sha256, manifest.sha256);
            }
            other => panic!("expected helper availability, got {other:?}"),
        }
        assert!(options.managed_runtime_available);
        assert!(options.warning.contains("isolated prefix"));
    }

    #[test]
    fn helper_protocol_request_contains_version_operation_and_random_nonce() {
        let nonce = random_nonce().unwrap();
        let request = BridgeRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: BridgeOperation::InspectManagedRuntime,
            runtime_relative_path: "runtimes/fixture".into(),
            nonce: nonce.clone(),
        };
        assert_eq!(nonce.len(), 32);
        assert!(nonce.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(request.protocol_version, 1);
    }

    #[test]
    fn process_executor_uses_own_process_group() {
        let (_temp, config, manifest) = configured_fixture();
        let fake = successful_fake(&config, &manifest);
        let _ = run_bridge_test_with_executor(&config, &fake).unwrap();
        let invocation = fake.invocation.lock().unwrap().clone().unwrap();
        assert!(invocation.timeout <= Duration::from_secs(30));
        assert_eq!(invocation.output_limit, 1024 * 1024);
        assert!(invocation.input.len() < REQUEST_LIMIT_BYTES);
    }
}
