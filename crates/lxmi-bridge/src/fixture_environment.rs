//! Diagnostic-only Linux environment comparison. Never accepts a game prefix.
use super::*;
use std::io::{Seek, SeekFrom};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
pub(crate) const TRANSPORT_FILE: &str = "fixture-container-transport";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureLinuxEnvironment {
    Direct,
    SteamRuntime4,
    SteamRuntimeRun,
}

#[derive(Debug, Serialize)]
pub struct FixtureEnvironmentStep {
    pub environment: FixtureLinuxEnvironment,
    /// The outer argv, separate from the bridge's inner Proton argv.
    pub outer_executable: PathBuf,
    pub outer_arguments: Vec<String>,
    pub entry_point_sha256: Option<String>,
    pub runtime_versions_sha256: Option<String>,
    pub runtime_versions: Option<String>,
    pub private_variable_directory: Option<PathBuf>,
    pub bridge: BridgeTestResult,
    pub game_entry_authorized: bool,
    pub container_metadata: Option<serde_json::Value>,
}

pub(crate) fn fixture_root(path: &Path) -> Result<PathBuf, BridgeError> {
    let root = path
        .canonicalize()
        .map_err(|e| BridgeError::new(BridgeErrorCode::UnsafePath, e.to_string()))?;
    let valid_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("lxmi-environment-fixture-"))
        .is_some_and(|s| {
            s.len() == 32
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        });
    let m = fs::symlink_metadata(path)
        .map_err(|e| BridgeError::new(BridgeErrorCode::UnsafePath, e.to_string()))?;
    if root != path
        || root.parent() != Some(Path::new("/tmp"))
        || !valid_name
        || !m.is_dir()
        || m.file_type().is_symlink()
        || m.uid() != unsafe { libc::getuid() }
        || m.mode() & 0o077 != 0
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "environment diagnostic requires a private, nonce-specific /tmp LXMI fixture root",
        ));
    }
    Ok(root)
}

pub(crate) fn wrap_invocation(
    invocation: &ProcessInvocation,
    entry: &Path,
    variable: &Path,
) -> ProcessInvocation {
    let root = variable
        .parent()
        .expect("private variable directory has a parent");
    let mut args = vec![
        OsString::from("--verb=run"),
        OsString::from("--"),
        root.join(TRANSPORT_FILE).into_os_string(),
    ];
    if entry.file_name() == Some(std::ffi::OsStr::new("run")) {
        args.remove(0);
    }
    let mut environment = invocation.environment.clone();
    environment.insert(
        "LXMI_FIXTURE_INPUT_SHA256".into(),
        format!("{:x}", Sha256::digest(&invocation.input)).into(),
    );
    let parsed: BridgeRequest = serde_json::from_slice(&invocation.input)
        .expect("executor validates request before wrapping");
    environment.insert("LXMI_FIXTURE_REQUEST_NONCE".into(), parsed.nonce.into());
    // Installed runtime's `run` script explicitly supports this override. No writes in its var/.
    environment.insert(
        "PRESSURE_VESSEL_VARIABLE_DIR".into(),
        variable.as_os_str().to_owned(),
    );
    ProcessInvocation {
        executable: entry.to_path_buf(),
        args,
        environment,
        working_directory: invocation.working_directory.clone(),
        // pressure-vessel replaces stdin with /dev/null. The fixed transport reopens our request.
        input: Vec::new(),
        timeout: Duration::from_secs(60),
        output_limit: invocation.output_limit,
        terminate_group_on_timeout: invocation.terminate_group_on_timeout,
        ownership: invocation.ownership.clone(),
    }
}

pub(crate) fn controlled_fixture_invocation(original: &ProcessInvocation) -> ProcessInvocation {
    let mut invocation = original.clone();
    invocation.environment.retain(|key, _| {
        matches!(
            key.to_str(),
            Some(
                "PATH"
                    | "LANG"
                    | "HOME"
                    | "XDG_CONFIG_HOME"
                    | "XDG_CACHE_HOME"
                    | "XDG_DATA_HOME"
                    | "STEAM_COMPAT_CLIENT_INSTALL_PATH"
                    | "STEAM_COMPAT_DATA_PATH"
                    | "LXMI_BRIDGE_TEST_MARKER"
                    | "WINEDEBUG"
                    | "LXMI_QUALIFIED_CONTEXT_SHA256"
            )
        )
    });
    invocation
        .environment
        .insert("PATH".into(), "/usr/bin:/bin".into());
    invocation
        .environment
        .insert("LANG".into(), "C.UTF-8".into());
    invocation
        .environment
        .insert("PYTHONDONTWRITEBYTECODE".into(), "1".into());
    invocation
}
struct DirectFixtureExecutor;
impl BridgeProcessExecutor for DirectFixtureExecutor {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
        NativeProcessExecutor.execute(&controlled_fixture_invocation(invocation))
    }
}

pub(crate) struct RuntimeFixtureExecutor<'a> {
    pub(crate) entry: &'a Path,
    pub(crate) variable: &'a Path,
}
impl BridgeProcessExecutor for RuntimeFixtureExecutor<'_> {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
        let controlled = controlled_fixture_invocation(invocation);
        let invocation = &controlled;
        let parsed: BridgeRequest = serde_json::from_slice(&invocation.input)
            .map_err(|e| ProcessFailure::Launch(io::Error::other(e)))?;
        validate_fixture_request(
            &invocation.input,
            Some(&format!("{:x}", Sha256::digest(&invocation.input))),
        )
        .map_err(|e| ProcessFailure::Launch(io::Error::other(e.to_string())))?;
        let request = self
            .variable
            .parent()
            .expect("private directory")
            .join(format!("bridge-request-{}.json", parsed.nonce));
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&request)
            .map_err(ProcessFailure::Launch)?;
        // RAII cleanup includes write errors, timeout and failed container startup.
        let _cleanup = RequestCleanup(request);
        file.write_all(&invocation.input)
            .and_then(|()| file.sync_all())
            .map_err(ProcessFailure::Launch)?;
        drop(file);
        let result =
            NativeProcessExecutor.execute(&wrap_invocation(invocation, self.entry, self.variable));
        match fs::remove_file(&_cleanup.0) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {} // Transport consumed it.
            Err(e) => return Err(ProcessFailure::OutputRead(e)),
        }
        if result.is_err() {
            let _ = fs::remove_file(
                self.variable
                    .parent()
                    .expect("private directory")
                    .join(format!("container-runtime-{}.json", parsed.nonce)),
            );
        }
        result
    }
}

/// Only called by the diagnostic example copied under the private fixture root.
/// Fixed Proton + fixed LXMI helper; no argv/PID/path input and no shell.
pub fn enter_fixture_container_transport() -> Result<(), BridgeError> {
    let executable =
        env::current_exe().map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    if executable.file_name() != Some(std::ffi::OsStr::new(TRANSPORT_FILE)) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "transport must be the session-owned diagnostic copy",
        ));
    }
    let root =
        qualified_executor::private_recipe_root(executable.parent().ok_or_else(|| {
            BridgeError::new(BridgeErrorCode::UnsafePath, "transport root missing")
        })?)?;
    let expected_prefix = root.join("test-prefixes/bridge-v1/compatdata");
    let approved_prefix = qualified_executor::transport_context(&root)?.unwrap_or(expected_prefix);
    if env::var_os("STEAM_COMPAT_DATA_PATH").as_deref() != Some(approved_prefix.as_os_str()) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "transport only accepts its own fixture compatdata",
        ));
    }
    let nonce = env::var("LXMI_FIXTURE_REQUEST_NONCE").unwrap_or_default();
    let (request, file) = open_fixture_request(
        &root,
        &nonce,
        env::var("LXMI_FIXTURE_INPUT_SHA256").ok().as_deref(),
    )?;
    capture_owned_container_metadata(&root, &nonce)?;
    // Consume exactly once. The validated descriptor, rather than a reopened path, becomes stdin.
    fs::remove_file(request).map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let proton = fixture_proton(&root)?;
    let helper = safe_existing_file(&root, &root.join("helpers").join(BRIDGE_HELPER_FILE))?;
    // Replace ourselves, preserving the existing executor's timeout/process ownership.
    let error = Command::new(proton)
        .args([std::ffi::OsStr::new("runinprefix"), helper.as_os_str()])
        .stdin(Stdio::from(file))
        .exec();
    Err(BridgeError::new(
        BridgeErrorCode::ProcessLaunchFailed,
        error.to_string(),
    ))
}

/// Only the already-validated fixture transport inspects its own namespace.
/// Persist selected documented environment and mounts, never the complete environment/mount table.
fn capture_owned_container_metadata(root: &Path, nonce: &str) -> Result<(), BridgeError> {
    let mountinfo = read_bounded_file(Path::new("/proc/self/mountinfo"), 2 * 1024 * 1024)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let selected_mounts: Vec<_> = String::from_utf8_lossy(&mountinfo).lines().filter_map(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        let point = *fields.get(4)?;
        if !matches!(point,"/usr" | "/run/host/usr" | "/run/host/fonts" | "/run/host/local-fonts" | "/run/host/font-cache" | "/usr/lib/pressure-vessel/overrides") { return None; }
        let separator=fields.iter().position(|f| *f=="-")?;
        Some(serde_json::json!({"mountpoint":point,"filesystem":fields.get(separator+1),"options":fields.get(5)}))
    }).collect();
    let environment: BTreeMap<_, _> = [
        "container",
        "STEAM_RUNTIME",
        "LD_LIBRARY_PATH",
        "FONTCONFIG_PATH",
        "FONTCONFIG_FILE",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
    ]
    .into_iter()
    .filter_map(|key| env::var(key).ok().map(|value| (key, value)))
    .collect();
    let font_paths: BTreeMap<_, _> = [
        "/run/host/fonts",
        "/run/host/local-fonts",
        "/usr/share/fonts",
    ]
    .into_iter()
    .map(|path| {
        (
            path,
            fs::metadata(path)
                .ok()
                .map(|m| serde_json::json!({"device":m.dev(),"inode":m.ino(),"mode":m.mode()})),
        )
    })
    .collect();
    let bytes=serde_json::to_vec(&serde_json::json!({"schema":1,"request_nonce":nonce,"selected_environment":environment,"selected_mounts":selected_mounts,"font_paths":font_paths,"coverage":"Owned fixture transport only, selected mounts and documented variables; not a complete mount/environment dump"}))
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io,e.to_string()))?;
    if bytes.len() > 16384 {
        return Err(BridgeError::new(
            BridgeErrorCode::OutputLimitExceeded,
            "container evidence too large",
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root.join(format!("container-runtime-{nonce}.json")))
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))
}

struct RequestCleanup(PathBuf);
impl Drop for RequestCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        // Container evidence survives successful execution for receipt collection;
        // private request never survives any outcome.
    }
}

pub(crate) fn open_fixture_request(
    root: &Path,
    nonce: &str,
    hash: Option<&str>,
) -> Result<(PathBuf, File), BridgeError> {
    if nonce.len() != 32
        || !nonce
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(BridgeError::new(
            BridgeErrorCode::NonceMismatch,
            "invalid request session",
        ));
    }
    let path = root.join(format!("bridge-request-{nonce}.json"));
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(|e| BridgeError::new(BridgeErrorCode::UnsafePath, e.to_string()))?;
    let m = file
        .metadata()
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let age = m.modified().ok().and_then(|t| t.elapsed().ok());
    if !m.is_file()
        || m.uid() != unsafe { libc::getuid() }
        || m.mode() & 0o777 != 0o600
        || m.nlink() != 1
        || m.len() > REQUEST_LIMIT_BYTES as u64
        || !age.is_some_and(|d| d <= Duration::from_secs(120))
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "unsafe, stale or oversized private request",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(REQUEST_LIMIT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    validate_fixture_request(&bytes, hash)?;
    let parsed: BridgeRequest = serde_json::from_slice(&bytes)
        .map_err(|e| BridgeError::new(BridgeErrorCode::MalformedResponse, e.to_string()))?;
    let current = fs::symlink_metadata(&path)
        .map_err(|e| BridgeError::new(BridgeErrorCode::UnsafePath, e.to_string()))?;
    if parsed.nonce != nonce
        || (m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec())
            != (
                current.dev(),
                current.ino(),
                current.len(),
                current.mtime(),
                current.mtime_nsec(),
            )
    {
        return Err(BridgeError::new(
            BridgeErrorCode::NonceMismatch,
            "request session/file changed",
        ));
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    Ok((path, file))
}

fn validate_fixture_request(bytes: &[u8], expected_hash: Option<&str>) -> Result<(), BridgeError> {
    if bytes.len() > REQUEST_LIMIT_BYTES
        || expected_hash != Some(format!("{:x}", Sha256::digest(bytes)).as_str())
    {
        return Err(BridgeError::new(
            BridgeErrorCode::NonceMismatch,
            "private transport input integrity mismatch",
        ));
    }
    let parsed: BridgeRequest = serde_json::from_slice(bytes)
        .map_err(|e| BridgeError::new(BridgeErrorCode::MalformedResponse, e.to_string()))?;
    if parsed.protocol_version != PROTOCOL_VERSION
        || parsed.runtime_relative_path != "marker"
        || parsed.nonce.len() != 32
        || !parsed.nonce.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(BridgeError::new(
            BridgeErrorCode::MalformedResponse,
            "transport only accepts the fixture marker request",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureProtonMirror {
    pub source_tool: PathBuf,
    pub mirror_root: PathBuf,
    pub verified_tree_manifest_sha256: String,
    pub source_unchanged_during_copy: bool,
}
/// Copy the fixed installed tool using the same descriptor-anchored recovery engine.
/// Proton locks and Python caches must never use the installed source during lab execution.
pub fn stage_fixture_proton_mirror(
    root: &Path,
    steam: &Path,
) -> Result<FixtureProtonMirror, BridgeError> {
    let root = qualified_executor::private_recipe_root(root)?;
    let source =
        validate_proton_script(&steam.join("steamapps/common/Proton - Experimental/proton"))?;
    let source = source.parent().expect("tool parent");
    let mirror = root.join("proton-tool");
    let digest = prefix_recovery::clone_private_tree(source, &mirror)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let info = FixtureProtonMirror {
        source_tool: source.to_path_buf(),
        mirror_root: mirror,
        verified_tree_manifest_sha256: digest,
        source_unchanged_during_copy: true,
    };
    attach_fixture_proton_mirror(&root, &info)?;
    Ok(info)
}
pub fn attach_fixture_proton_mirror(
    root: &Path,
    info: &FixtureProtonMirror,
) -> Result<(), BridgeError> {
    let root = qualified_executor::private_recipe_root(root)?;
    validate_mirror(info)?;
    let bytes = serde_json::to_vec(info)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root.join("proton-mirror.json"))
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))
}
fn validate_mirror(info: &FixtureProtonMirror) -> Result<PathBuf, BridgeError> {
    let owner = qualified_executor::private_recipe_root(
        info.mirror_root
            .parent()
            .ok_or_else(|| BridgeError::new(BridgeErrorCode::UnsafePath, "mirror parent"))?,
    )?;
    if info.mirror_root != owner.join("proton-tool")
        || !info.source_unchanged_during_copy
        || info.verified_tree_manifest_sha256.len() != 64
        || !info
            .verified_tree_manifest_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "private verified Proton mirror required",
        ));
    }
    let tool = safe_existing_directory(&owner, &info.mirror_root)?;
    let script = validate_proton_script(&tool.join("proton"))?;
    let hash = sha256_file(&script, 4 * 1024 * 1024)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    let version = read_bounded_file(&safe_existing_file(&tool, &tool.join("version"))?, 4096)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    if hash != "077db31dda75c570c1a91014f9a814b785307bc91e5dfb1480b984dc34166a54"
        || String::from_utf8_lossy(&version).trim()
            != "1790244902 experimental-11.0-20260924-x86_64"
    {
        return Err(BridgeError::new(
            BridgeErrorCode::HelperHashMismatch,
            "private tool pins changed; no substitution",
        ));
    }
    Ok(script)
}
pub(crate) fn fixture_proton(root: &Path) -> Result<PathBuf, BridgeError> {
    let bytes = read_bounded_file(
        &safe_existing_file(root, &root.join("proton-mirror.json"))?,
        16384,
    )
    .map_err(|e| {
        BridgeError::new(
            BridgeErrorCode::UnsafePath,
            format!("installed tool execution is prohibited; mirror receipt required: {e}"),
        )
    })?;
    let info: FixtureProtonMirror = serde_json::from_slice(&bytes)
        .map_err(|e| BridgeError::new(BridgeErrorCode::MalformedResponse, e.to_string()))?;
    validate_mirror(&info)
}

/// Read-only review of three hives in two already-owned fixtures; no execution or game path.
pub fn review_environment_fixture_sections(
    source_root: &Path,
    destination_root: &Path,
) -> Result<BTreeMap<String, Vec<FixtureRegistrySectionMutation>>, BridgeError> {
    let source = fixture_root(source_root)?;
    let destination = fixture_root(destination_root)?;
    ["system.reg", "user.reg", "userdef.reg"]
        .into_iter()
        .map(|name| {
            let relative = Path::new("test-prefixes/bridge-v1/compatdata/pfx").join(name);
            let before = read_bounded_file(
                &safe_existing_file(&source, &source.join(&relative))?,
                8 * 1024 * 1024,
            )
            .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
            let after = read_bounded_file(
                &safe_existing_file(&destination, &destination.join(&relative))?,
                8 * 1024 * 1024,
            )
            .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
            let sections = diagnose_fixture_registry_sections(&before, &after).map_err(|e| {
                BridgeError::new(BridgeErrorCode::MalformedResponse, format!("{e:?}"))
            })?;
            Ok((name.into(), sections))
        })
        .collect()
}

/// Complete verified clone of a private fixture prefix. No source/destination paths from UI.
pub fn clone_environment_fixture_prefix(
    source_root: &Path,
    destination_root: &Path,
) -> Result<String, BridgeError> {
    let source = fixture_root(source_root)?;
    let destination = fixture_root(destination_root)?;
    if source == destination {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePath,
            "independent clone required",
        ));
    }
    let source = safe_existing_directory(
        &source,
        &source.join("test-prefixes/bridge-v1/compatdata/pfx"),
    )?;
    let context = prepare_isolated_test_context(&destination)?;
    let prefix = context.join("pfx");
    prefix_recovery::clone_private_tree(&source, &prefix)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))
}

/// Fixed helper, fixed runinprefix operation, isolated prefix only. No frontend command.
/// The lab root must be newly prepared, private and named with a fresh 128-bit nonce.
pub fn run_fixture_environment_step(
    config: &BridgeTestConfig,
    environment: FixtureLinuxEnvironment,
) -> Result<FixtureEnvironmentStep, BridgeError> {
    let root = fixture_root(&config.managed_root)?;
    if config.prefix_mode != PrefixMode::IsolatedTemporaryPrefix {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "game prefix is prohibited in environment comparison",
        ));
    }
    let expected_proton = fixture_proton(&root)?;
    if validate_proton_script(&config.explicit_runtime.proton_script)? != expected_proton {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "fixture tool declaration does not match the fixed transport tool",
        ));
    }
    let mut evidence = FixtureEnvironmentStep {
        environment,
        outer_executable: config.explicit_runtime.proton_script.clone(),
        outer_arguments: vec![
            "runinprefix".into(),
            root.join("helpers")
                .join(BRIDGE_HELPER_FILE)
                .display()
                .to_string(),
        ],
        entry_point_sha256: None,
        runtime_versions_sha256: None,
        runtime_versions: None,
        private_variable_directory: None,
        // Filled only after the existing bridge verifies helper/nonce/hash/exit.
        bridge: match environment {
            FixtureLinuxEnvironment::Direct => {
                run_bridge_test_with_executor(config, &DirectFixtureExecutor)?
            }
            FixtureLinuxEnvironment::SteamRuntime4 | FixtureLinuxEnvironment::SteamRuntimeRun => {
                let steam = config
                    .steam_client_install_path
                    .canonicalize()
                    .map_err(|e| {
                        BridgeError::new(BridgeErrorCode::RuntimeNotFound, e.to_string())
                    })?;
                let runtime = safe_existing_directory(
                    &steam,
                    &steam.join("steamapps/common/SteamLinuxRuntime_4"),
                )?;
                let entry_name = if environment == FixtureLinuxEnvironment::SteamRuntimeRun {
                    "run"
                } else {
                    "_v2-entry-point"
                };
                let entry = safe_existing_file(&runtime, &runtime.join(entry_name))?;
                let transport = safe_existing_file(&root, &root.join(TRANSPORT_FILE))?;
                let expected = read_bounded_file(
                    &safe_existing_file(&root, &root.join("fixture-container-transport.sha256"))?,
                    128,
                )
                .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
                if expected
                    != sha256_file(&transport, 256 * 1024 * 1024)
                        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?
                        .as_bytes()
                {
                    return Err(BridgeError::new(
                        BridgeErrorCode::HelperHashMismatch,
                        "fixture transport hash mismatch",
                    ));
                }
                let variable = root.join("runtime-variable");
                ensure_controlled_directory(&root, &variable)?;
                run_bridge_test_with_executor(
                    config,
                    &RuntimeFixtureExecutor {
                        entry: &entry,
                        variable: &variable,
                    },
                )?
            }
        },
        game_entry_authorized: false,
        container_metadata: None,
    };
    if environment != FixtureLinuxEnvironment::Direct {
        let steam = config
            .steam_client_install_path
            .canonicalize()
            .map_err(|e| BridgeError::new(BridgeErrorCode::RuntimeNotFound, e.to_string()))?;
        let runtime =
            safe_existing_directory(&steam, &steam.join("steamapps/common/SteamLinuxRuntime_4"))?;
        let entry_name = if environment == FixtureLinuxEnvironment::SteamRuntimeRun {
            "run"
        } else {
            "_v2-entry-point"
        };
        let entry = safe_existing_file(&runtime, &runtime.join(entry_name))?;
        let versions = safe_existing_file(&runtime, &runtime.join("VERSIONS.txt"))?;
        evidence.outer_executable = entry.clone();
        evidence.outer_arguments = vec![
            "--verb=run".into(),
            "--".into(),
            root.join(TRANSPORT_FILE).display().to_string(),
        ];
        if environment == FixtureLinuxEnvironment::SteamRuntimeRun {
            evidence.outer_arguments.remove(0);
        }
        evidence.entry_point_sha256 = Some(
            sha256_file(&entry, 128 * 1024)
                .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?,
        );
        evidence.runtime_versions_sha256 = Some(
            sha256_file(&versions, 16384)
                .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?,
        );
        evidence.runtime_versions = Some(
            String::from_utf8(
                read_bounded_file(&versions, 16384)
                    .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?,
            )
            .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?,
        );
        evidence.private_variable_directory = Some(root.join("runtime-variable"));
        let response: BridgeResponse = serde_json::from_str(&evidence.bridge.process.stdout)
            .map_err(|e| BridgeError::new(BridgeErrorCode::MalformedResponse, e.to_string()))?;
        let nonce = response.nonce.ok_or_else(|| {
            BridgeError::new(BridgeErrorCode::NonceMismatch, "response nonce missing")
        })?;
        let path =
            safe_existing_file(&root, &root.join(format!("container-runtime-{nonce}.json")))?;
        let bytes = read_bounded_file(&path, 16384)
            .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| BridgeError::new(BridgeErrorCode::MalformedResponse, e.to_string()))?;
        if value["request_nonce"] != nonce {
            return Err(BridgeError::new(
                BridgeErrorCode::NonceMismatch,
                "container evidence session mismatch",
            ));
        }
        fs::remove_file(path).map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
        evidence.container_metadata = Some(value);
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_proton_without_private_mirror_receipt_cannot_execute() {
        use std::os::unix::fs::DirBuilderExt;
        let root = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        assert!(fixture_proton(&root).is_err());
        assert!(validate_mirror(&FixtureProtonMirror {
            source_tool: "/Steam/Proton".into(),
            mirror_root: "/Steam/Proton".into(),
            verified_tree_manifest_sha256: "a".repeat(64),
            source_unchanged_during_copy: true
        })
        .is_err());
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn independent_initial_prefix_clone_is_complete_and_preserves_symlinks() {
        use std::os::unix::fs::DirBuilderExt;
        let root = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        let destination = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        for path in [&root, &destination] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        let prefix = prepare_isolated_test_context(&root).unwrap().join("pfx");
        fs::create_dir(&prefix).unwrap();
        fs::create_dir(prefix.join("dosdevices")).unwrap();
        fs::write(prefix.join("system.reg"), b"WINE REGISTRY Version 2\n").unwrap();
        std::os::unix::fs::symlink("/", prefix.join("dosdevices/z:")).unwrap();
        let digest = clone_environment_fixture_prefix(&root, &destination).unwrap();
        assert_eq!(digest.len(), 64);
        let copied = destination.join("test-prefixes/bridge-v1/compatdata/pfx");
        assert_eq!(
            fs::read(copied.join("system.reg")).unwrap(),
            fs::read(prefix.join("system.reg")).unwrap()
        );
        assert_eq!(
            fs::read_link(copied.join("dosdevices/z:")).unwrap(),
            Path::new("/")
        );
        assert!(clone_environment_fixture_prefix(&root, &destination).is_err());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(destination).unwrap();
    }
    #[test]
    fn request_file_requires_private_mode_nonce_hash_age_and_no_symlink() {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let root = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let nonce = random_nonce().unwrap();
        let request = BridgeRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: lxmi_bridge_protocol::BridgeOperation::InspectManagedRuntime,
            nonce: nonce.clone(),
            runtime_relative_path: "marker".into(),
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let path = root.join(format!("bridge-request-{nonce}.json"));
        fs::write(&path, &bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(open_fixture_request(&root, &nonce, Some(&hash)).is_ok());
        assert!(open_fixture_request(&root, &random_nonce().unwrap(), Some(&hash)).is_err());
        assert!(open_fixture_request(&root, &nonce, Some("bad_hash")).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(open_fixture_request(&root, &nonce, Some(&hash)).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let stale = std::time::SystemTime::now() - Duration::from_secs(121);
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(stale))
            .unwrap();
        assert!(open_fixture_request(&root, &nonce, Some(&hash)).is_err());
        fs::remove_file(&path).unwrap();
        let target = root.join("other");
        fs::write(&target, &bytes).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(open_fixture_request(&root, &nonce, Some(&hash)).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    fn cleanup_also_removes_partial_failed_request_and_environment_is_allowlisted() {
        let root =
            env::temp_dir().join(format!("lxmi-request-cleanup-{}", random_nonce().unwrap()));
        fs::create_dir(&root).unwrap();
        let path = root.join("request");
        fs::write(&path, b"partial").unwrap();
        {
            let _guard = RequestCleanup(path.clone());
        }
        assert!(!path.exists());
        fs::remove_dir(root).unwrap();
        let original = ProcessInvocation {
            executable: "/proton".into(),
            args: vec![],
            environment: BTreeMap::from([
                ("SESSION_SECRET".into(), "never copy".into()),
                ("PATH".into(), "/untrusted/path".into()),
                ("LANG".into(), "arbitrary".into()),
                ("STEAM_COMPAT_DATA_PATH".into(), "/fixture".into()),
            ]),
            working_directory: "/fixture".into(),
            input: vec![],
            timeout: BRIDGE_TEST_TIMEOUT,
            output_limit: 1024,
            terminate_group_on_timeout: true,
            ownership: None,
        };
        let result = controlled_fixture_invocation(&original);
        assert!(!result
            .environment
            .contains_key(std::ffi::OsStr::new("SESSION_SECRET")));
        assert_eq!(
            result.environment[std::ffi::OsStr::new("PATH")],
            "/usr/bin:/bin"
        );
        assert_eq!(result.environment[std::ffi::OsStr::new("LANG")], "C.UTF-8");
    }
    #[test]
    fn clone_rejects_real_prefix_and_arbitrary_destination() {
        assert!(clone_environment_fixture_prefix(
            Path::new("/Steam/compatdata/4162040"),
            Path::new("/tmp")
        )
        .is_err());
    }
    #[test]
    fn wrapper_preserves_protocol_and_controlled_environment() {
        let original = ProcessInvocation {
            executable: "/explicit/proton".into(),
            args: vec!["runinprefix".into(), "/fixture/helper.exe".into()],
            environment: BTreeMap::from([(
                "STEAM_COMPAT_DATA_PATH".into(),
                "/fixture/compatdata".into(),
            )]),
            working_directory: "/fixture".into(),
            input: serde_json::to_vec(&BridgeRequest {
                protocol_version: PROTOCOL_VERSION,
                operation: lxmi_bridge_protocol::BridgeOperation::InspectManagedRuntime,
                nonce: "0123456789abcdef0123456789abcdef".into(),
                runtime_relative_path: "marker".into(),
            })
            .unwrap(),
            timeout: BRIDGE_TEST_TIMEOUT,
            output_limit: 1024,
            terminate_group_on_timeout: true,
            ownership: None,
        };
        let wrapped = wrap_invocation(
            &original,
            Path::new("/runtime/_v2-entry-point"),
            Path::new("/fixture/variable"),
        );
        assert_eq!(
            wrapped.args,
            ["--verb=run", "--", "/fixture/fixture-container-transport"].map(OsString::from)
        );
        assert!(wrapped.input.is_empty());
        assert_eq!(
            wrapped.environment[std::ffi::OsStr::new("LXMI_FIXTURE_INPUT_SHA256")],
            OsString::from(format!("{:x}", Sha256::digest(&original.input)))
        );
        assert_eq!(
            wrapped.environment[std::ffi::OsStr::new("STEAM_COMPAT_DATA_PATH")],
            "/fixture/compatdata"
        );
        assert_eq!(
            wrapped.environment[std::ffi::OsStr::new("PRESSURE_VESSEL_VARIABLE_DIR")],
            "/fixture/variable"
        );
        assert!(wrapped.timeout <= Duration::from_secs(60));
        assert!(!wrapped
            .environment
            .contains_key(std::ffi::OsStr::new("SteamAppId")));
    }
    #[test]
    fn real_prefix_and_arbitrary_roots_are_rejected() {
        for path in [
            "/tmp",
            "/home/university/.local/share/Steam/steamapps/compatdata/4162040/pfx",
            "/home/university/.local/share/lxmi",
        ] {
            assert!(fixture_root(Path::new(path)).is_err());
        }
    }
    #[test]
    fn private_root_is_required_and_symlinks_are_rejected() {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let root = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        assert_eq!(fixture_root(&root).unwrap(), root);
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(fixture_root(&root).is_err());
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let link = PathBuf::from(format!(
            "/tmp/lxmi-environment-fixture-{}",
            random_nonce().unwrap()
        ));
        std::os::unix::fs::symlink(&root, &link).unwrap();
        assert!(fixture_root(&link).is_err());
        fs::remove_file(link).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn transport_request_requires_hash_protocol_nonce_and_fixed_marker() {
        let mut request = BridgeRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: lxmi_bridge_protocol::BridgeOperation::InspectManagedRuntime,
            nonce: "0123456789abcdef0123456789abcdef".into(),
            runtime_relative_path: "marker".into(),
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(
            validate_fixture_request(&bytes, Some(&format!("{:x}", Sha256::digest(&bytes))))
                .is_ok()
        );
        assert!(validate_fixture_request(&bytes, None).is_err());
        assert!(validate_fixture_request(&bytes, Some("wrong_hash")).is_err());
        for path in [
            "../outside",
            "/etc",
            "steamapps/compatdata/4162040",
            "other",
        ] {
            request.runtime_relative_path = path.into();
            let bytes = serde_json::to_vec(&request).unwrap();
            assert!(validate_fixture_request(
                &bytes,
                Some(&format!("{:x}", Sha256::digest(&bytes)))
            )
            .is_err());
        }
        request.runtime_relative_path = "marker".into();
        request.nonce = "bad".into();
        let bytes = serde_json::to_vec(&request).unwrap();
        assert!(
            validate_fixture_request(&bytes, Some(&format!("{:x}", Sha256::digest(&bytes))))
                .is_err()
        );
    }
}
