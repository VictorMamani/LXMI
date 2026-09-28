use lxmi_bridge_protocol::{
    BridgeDiagnosticCode, BridgeOperation, BridgeRequest, BridgeResponse, HELPER_VERSION,
    MARKER_FILE, PROTOCOL_VERSION, REQUEST_LIMIT_BYTES,
};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};

const HELPER_LIMIT_BYTES: u64 = 64 * 1024 * 1024;
const MARKER_LIMIT_BYTES: u64 = 1024 * 1024;

fn main() {
    if env::args_os().nth(1).is_some_and(|arg| arg == "--version") {
        println!("lxmi-bridge-helper {HELPER_VERSION} protocol {PROTOCOL_VERSION}");
        return;
    }

    let response = match read_request() {
        Ok(request) => handle_request(&request, &env::current_exe()),
        Err(error) => BridgeResponse::failure(
            None,
            BridgeDiagnosticCode::HelperInspectionFailed,
            error.to_string(),
        ),
    };
    let mut stdout = io::stdout().lock();
    if serde_json::to_writer(&mut stdout, &response).is_ok() {
        let _ = stdout.write_all(b"\n");
        let _ = stdout.flush();
    }
    if !response.success {
        std::process::exit(2);
    }
}

fn read_request() -> io::Result<BridgeRequest> {
    let mut stdin = io::stdin().take(REQUEST_LIMIT_BYTES as u64 + 1);
    let mut bytes = Vec::new();
    stdin.read_to_end(&mut bytes)?;
    parse_request(&bytes)
}

fn parse_request(bytes: &[u8]) -> io::Result<BridgeRequest> {
    if bytes.len() > REQUEST_LIMIT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bridge request exceeded the 64 KiB limit",
        ));
    }
    serde_json::from_slice(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn handle_request(request: &BridgeRequest, executable: &io::Result<PathBuf>) -> BridgeResponse {
    let nonce = Some(request.nonce.clone());
    if request.protocol_version != PROTOCOL_VERSION {
        return BridgeResponse::failure(
            nonce,
            BridgeDiagnosticCode::UnsupportedProtocol,
            format!("unsupported protocol version {}", request.protocol_version),
        );
    }
    if !matches!(request.operation, BridgeOperation::InspectManagedRuntime) {
        return BridgeResponse::failure(
            nonce,
            BridgeDiagnosticCode::HelperInspectionFailed,
            "unsupported bridge operation",
        );
    }
    if !valid_nonce(&request.nonce) {
        return BridgeResponse::failure(
            nonce,
            BridgeDiagnosticCode::InvalidNonce,
            "nonce must contain 32 hexadecimal characters",
        );
    }
    if !safe_relative_path(&request.runtime_relative_path) {
        return BridgeResponse::failure(
            nonce,
            BridgeDiagnosticCode::UnsafeRelativePath,
            "runtime path must be a normalized relative path beneath LXMI storage",
        );
    }

    match inspect_runtime(request, executable) {
        Ok(response) => response,
        Err(error) => BridgeResponse::failure(nonce, error.code, error.detail),
    }
}

#[derive(Debug)]
struct InspectionFailure {
    code: BridgeDiagnosticCode,
    detail: String,
}

fn inspect_runtime(
    request: &BridgeRequest,
    executable: &io::Result<PathBuf>,
) -> Result<BridgeResponse, InspectionFailure> {
    let executable = executable.as_ref().map_err(|error| InspectionFailure {
        code: BridgeDiagnosticCode::HelperInspectionFailed,
        detail: format!("could not locate this helper executable: {error}"),
    })?;
    let helper_dir = executable.parent().ok_or_else(|| InspectionFailure {
        code: BridgeDiagnosticCode::HelperInspectionFailed,
        detail: "helper executable has no parent directory".into(),
    })?;
    let managed_root = helper_dir.parent().ok_or_else(|| InspectionFailure {
        code: BridgeDiagnosticCode::HelperInspectionFailed,
        detail: "helper is not located under the managed helpers directory".into(),
    })?;
    let managed_root = managed_root
        .canonicalize()
        .map_err(|error| InspectionFailure {
            code: BridgeDiagnosticCode::HelperInspectionFailed,
            detail: format!("managed root is unavailable: {error}"),
        })?;
    let runtime_path = managed_root.join(&request.runtime_relative_path);
    reject_symlink_components(&managed_root, &runtime_path).map_err(|error| InspectionFailure {
        code: BridgeDiagnosticCode::RuntimeNotVisible,
        detail: error.to_string(),
    })?;
    let canonical_runtime = runtime_path
        .canonicalize()
        .map_err(|error| InspectionFailure {
            code: BridgeDiagnosticCode::RuntimeNotVisible,
            detail: format!("runtime path is not visible from this Wine/Proton process: {error}"),
        })?;
    if !canonical_runtime.starts_with(&managed_root) || !canonical_runtime.is_dir() {
        return Err(InspectionFailure {
            code: BridgeDiagnosticCode::UnsafeRelativePath,
            detail: "resolved runtime path escaped LXMI managed storage or is not a directory"
                .into(),
        });
    }

    let marker = canonical_runtime.join(MARKER_FILE);
    let metadata = fs::symlink_metadata(&marker).map_err(|error| InspectionFailure {
        code: BridgeDiagnosticCode::ManifestUnreadable,
        detail: format!("runtime manifest is unavailable: {error}"),
    })?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MARKER_LIMIT_BYTES
    {
        return Err(InspectionFailure {
            code: BridgeDiagnosticCode::ManifestUnreadable,
            detail: "runtime manifest must be a regular file no larger than 1 MiB".into(),
        });
    }
    let manifest_bytes =
        read_bounded_file(&marker, MARKER_LIMIT_BYTES).map_err(|error| InspectionFailure {
            code: BridgeDiagnosticCode::ManifestUnreadable,
            detail: error.to_string(),
        })?;
    if !serde_json::from_slice::<serde_json::Value>(&manifest_bytes).is_ok() {
        return Err(InspectionFailure {
            code: BridgeDiagnosticCode::ManifestUnreadable,
            detail: "runtime manifest is not valid JSON".into(),
        });
    }

    let helper_bytes =
        read_bounded_file(executable, HELPER_LIMIT_BYTES).map_err(|error| InspectionFailure {
            code: BridgeDiagnosticCode::HelperInspectionFailed,
            detail: format!("could not hash this helper binary: {error}"),
        })?;
    let response = BridgeResponse {
        protocol_version: PROTOCOL_VERSION,
        success: true,
        helper_version: HELPER_VERSION.to_owned(),
        build_target: format!("{}-{}", env::consts::ARCH, env::consts::OS),
        helper_sha256: Some(sha256_hex(&helper_bytes)),
        runtime_visible: true,
        runtime_manifest_readable: true,
        runtime_path_windows: Some(display_windows_path(&canonical_runtime)),
        runtime_manifest_sha256: Some(sha256_hex(&manifest_bytes)),
        environment_marker_matches: env::var(lxmi_bridge_protocol::ENVIRONMENT_MARKER)
            .is_ok_and(|value| value == lxmi_bridge_protocol::ENVIRONMENT_MARKER_VALUE),
        nonce: Some(request.nonce.clone()),
        diagnostic: None,
    };
    Ok(response)
}

fn read_bounded_file(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file is not a bounded regular file",
        ));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeded the configured size limit while reading",
        ));
    }
    Ok(bytes)
}

fn reject_symlink_components(root: &Path, target: &Path) -> io::Result<()> {
    if !target.starts_with(root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "target is outside LXMI managed storage",
        ));
    }
    let mut cursor = root.to_path_buf();
    for component in target.strip_prefix(root).unwrap_or(target).components() {
        match component {
            Component::Normal(part) => cursor.push(part),
            Component::CurDir => continue,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "non-normal path component is not allowed",
                ));
            }
        }
        let metadata = fs::symlink_metadata(&cursor)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "symlinks are not allowed in the managed runtime path",
            ));
        }
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') || value.contains(':') || value.starts_with('/') {
        return false;
    }
    Path::new(value)
        .components()
        .all(|component| matches!(component, Component::Normal(part) if !part.is_empty()))
}

fn valid_nonce(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn display_windows_path(path: &Path) -> String {
    let path = path.to_string_lossy();
    path.strip_prefix("\\\\?\\").unwrap_or(&path).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lxmi_bridge_protocol::{BridgeOperation, BridgeRequest};
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn test_root() -> PathBuf {
        let suffix = NEXT.fetch_add(1, Ordering::Relaxed);
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after UNIX epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "lxmi-bridge-helper-{}-{tick}-{suffix}",
            std::process::id()
        ))
    }

    fn request(path: &str) -> BridgeRequest {
        BridgeRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: BridgeOperation::InspectManagedRuntime,
            runtime_relative_path: path.to_owned(),
            nonce: "0123456789abcdef0123456789abcdef".into(),
        }
    }

    fn fixture() -> (PathBuf, PathBuf) {
        let root = test_root();
        let helpers = root.join("helpers");
        let runtime = root.join("runtimes/test-runtime");
        fs::create_dir_all(&helpers).unwrap();
        fs::create_dir_all(&runtime).unwrap();
        fs::write(
            helpers.join("lxmi-bridge-helper.exe"),
            b"harmless helper fixture",
        )
        .unwrap();
        fs::write(runtime.join(MARKER_FILE), br#"{"runtime_id":"fixture"}"#).unwrap();
        (root, runtime)
    }

    #[test]
    fn detects_managed_runtime_and_environment_marker() {
        let (root, runtime) = fixture();
        let exe = root.join("helpers/lxmi-bridge-helper.exe");
        let response = handle_request(&request("runtimes/test-runtime"), &Ok(exe));
        assert!(response.success);
        assert!(response.runtime_visible);
        assert!(response.runtime_manifest_readable);
        assert!(response.helper_sha256.is_some());
        assert!(response.runtime_manifest_sha256.is_some());
        assert_eq!(
            response.nonce.as_deref(),
            Some("0123456789abcdef0123456789abcdef")
        );
        assert!(runtime.join(MARKER_FILE).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_traversal_absolute_and_windows_paths() {
        for path in ["../outside", "/etc", "C:/outside", "runtimes\\outside"] {
            let response = handle_request(&request(path), &Err(io::Error::other("unused")));
            assert_eq!(
                response
                    .diagnostic
                    .as_ref()
                    .map(|diagnostic| diagnostic.code),
                Some(BridgeDiagnosticCode::UnsafeRelativePath),
                "{path}"
            );
        }
    }

    #[test]
    fn rejects_wrong_protocol_and_bad_nonce() {
        let (root, _) = fixture();
        let exe = root.join("helpers/lxmi-bridge-helper.exe");
        let mut incompatible = request("runtimes/test-runtime");
        incompatible.protocol_version += 1;
        assert_eq!(
            handle_request(&incompatible, &Ok(exe.clone()))
                .diagnostic
                .unwrap()
                .code,
            BridgeDiagnosticCode::UnsupportedProtocol
        );
        let mut invalid_nonce = request("runtimes/test-runtime");
        invalid_nonce.nonce = "not-a-nonce".into();
        assert_eq!(
            handle_request(&invalid_nonce, &Ok(exe))
                .diagnostic
                .unwrap()
                .code,
            BridgeDiagnosticCode::InvalidNonce
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_symlink_in_managed_runtime() {
        let (root, _) = fixture();
        let outside = test_root();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join(MARKER_FILE), br#"{"runtime_id":"outside"}"#).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, root.join("runtimes/linked")).unwrap();
        #[cfg(unix)]
        {
            let exe = root.join("helpers/lxmi-bridge-helper.exe");
            let response = handle_request(&request("runtimes/linked"), &Ok(exe));
            assert_eq!(
                response
                    .diagnostic
                    .as_ref()
                    .map(|diagnostic| diagnostic.code),
                Some(BridgeDiagnosticCode::RuntimeNotVisible)
            );
        }
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn rejects_non_json_runtime_marker() {
        let (root, runtime) = fixture();
        fs::write(runtime.join(MARKER_FILE), b"not-json").unwrap();
        let exe = root.join("helpers/lxmi-bridge-helper.exe");
        let response = handle_request(&request("runtimes/test-runtime"), &Ok(exe));
        assert_eq!(
            response
                .diagnostic
                .as_ref()
                .map(|diagnostic| diagnostic.code),
            Some(BridgeDiagnosticCode::ManifestUnreadable)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
