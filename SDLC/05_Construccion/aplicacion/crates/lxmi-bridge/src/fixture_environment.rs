//! Diagnostic-only Linux environment comparison. Never accepts a game prefix.
use super::*;
use std::os::unix::fs::MetadataExt;
const TRANSPORT_FILE: &str = "fixture-container-transport";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureLinuxEnvironment {
    Direct,
    SteamRuntime4,
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
}

fn fixture_root(path: &Path) -> Result<PathBuf, BridgeError> {
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

fn wrap_invocation(
    invocation: &ProcessInvocation,
    entry: &Path,
    variable: &Path,
) -> ProcessInvocation {
    let root = variable
        .parent()
        .expect("private variable directory has a parent");
    let args = vec![
        OsString::from("--verb=run"),
        OsString::from("--"),
        root.join(TRANSPORT_FILE).into_os_string(),
    ];
    let mut environment = invocation.environment.clone();
    environment.insert(
        "LXMI_FIXTURE_INPUT_SHA256".into(),
        format!("{:x}", Sha256::digest(&invocation.input)).into(),
    );
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

struct RuntimeFixtureExecutor<'a> {
    entry: &'a Path,
    variable: &'a Path,
}
impl BridgeProcessExecutor for RuntimeFixtureExecutor<'_> {
    fn execute(&self, invocation: &ProcessInvocation) -> Result<ProcessOutput, ProcessFailure> {
        let request = self
            .variable
            .parent()
            .expect("private variable directory has a parent")
            .join("bridge-request.json");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&request)
            .map_err(ProcessFailure::Launch)?;
        file.write_all(&invocation.input)
            .and_then(|()| file.sync_all())
            .map_err(ProcessFailure::Launch)?;
        drop(file);
        let result =
            NativeProcessExecutor.execute(&wrap_invocation(invocation, self.entry, self.variable));
        if result.is_ok() {
            fs::remove_file(request).map_err(ProcessFailure::OutputRead)?;
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
        fixture_root(executable.parent().ok_or_else(|| {
            BridgeError::new(BridgeErrorCode::UnsafePath, "transport root missing")
        })?)?;
    let expected_prefix = root.join("test-prefixes/bridge-v1/compatdata");
    if env::var_os("STEAM_COMPAT_DATA_PATH").as_deref() != Some(expected_prefix.as_os_str()) {
        return Err(BridgeError::new(
            BridgeErrorCode::UnsafePrefixContext,
            "transport only accepts its own fixture compatdata",
        ));
    }
    let request = safe_existing_file(&root, &root.join("bridge-request.json"))?;
    let bytes = read_bounded_file(&request, REQUEST_LIMIT_BYTES as u64)
        .map_err(|e| BridgeError::new(BridgeErrorCode::Io, e.to_string()))?;
    validate_fixture_request(
        &bytes,
        env::var("LXMI_FIXTURE_INPUT_SHA256").ok().as_deref(),
    )?;
    let steam = PathBuf::from(
        env::var_os("STEAM_COMPAT_CLIENT_INSTALL_PATH").ok_or_else(|| {
            BridgeError::new(BridgeErrorCode::RuntimeNotFound, "Steam root absent")
        })?,
    );
    let proton =
        validate_proton_script(&steam.join("steamapps/common/Proton - Experimental/proton"))?;
    let helper = safe_existing_file(&root, &root.join("helpers").join(BRIDGE_HELPER_FILE))?;
    // Replace ourselves, preserving the existing executor's timeout/process ownership.
    let error = Command::new(proton)
        .args([std::ffi::OsStr::new("runinprefix"), helper.as_os_str()])
        .stdin(Stdio::from(File::open(request).map_err(|e| {
            BridgeError::new(BridgeErrorCode::Io, e.to_string())
        })?))
        .exec();
    Err(BridgeError::new(
        BridgeErrorCode::ProcessLaunchFailed,
        error.to_string(),
    ))
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
    let steam = config
        .steam_client_install_path
        .canonicalize()
        .map_err(|e| BridgeError::new(BridgeErrorCode::RuntimeNotFound, e.to_string()))?;
    let expected_proton =
        validate_proton_script(&steam.join("steamapps/common/Proton - Experimental/proton"))?;
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
            FixtureLinuxEnvironment::Direct => run_bridge_test(config)?,
            FixtureLinuxEnvironment::SteamRuntime4 => {
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
                let entry = safe_existing_file(&runtime, &runtime.join("_v2-entry-point"))?;
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
    };
    if environment == FixtureLinuxEnvironment::SteamRuntime4 {
        let steam = config
            .steam_client_install_path
            .canonicalize()
            .map_err(|e| BridgeError::new(BridgeErrorCode::RuntimeNotFound, e.to_string()))?;
        let runtime =
            safe_existing_directory(&steam, &steam.join("steamapps/common/SteamLinuxRuntime_4"))?;
        let entry = safe_existing_file(&runtime, &runtime.join("_v2-entry-point"))?;
        let versions = safe_existing_file(&runtime, &runtime.join("VERSIONS.txt"))?;
        evidence.outer_executable = entry.clone();
        evidence.outer_arguments = vec![
            "--verb=run".into(),
            "--".into(),
            root.join(TRANSPORT_FILE).display().to_string(),
        ];
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
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;
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
            input: b"nonce-protocol".to_vec(),
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
