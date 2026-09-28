//! Small versioned stdin/stdout contract shared by LXMI and its Windows helper.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const HELPER_VERSION: &str = "0.6.0";
pub const REQUEST_LIMIT_BYTES: usize = 64 * 1024;
pub const RESPONSE_LIMIT_BYTES: usize = 1024 * 1024;
pub const MARKER_FILE: &str = "runtime-manifest.json";
pub const ENVIRONMENT_MARKER: &str = "LXMI_BRIDGE_TEST_MARKER";
pub const ENVIRONMENT_MARKER_VALUE: &str = "lxmi-bridge-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeOperation {
    InspectManagedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub protocol_version: u16,
    pub operation: BridgeOperation,
    /// A normalized path relative to the LXMI data root, never an arbitrary host path.
    pub runtime_relative_path: String,
    /// 128 bits of randomness encoded as 32 lowercase hexadecimal characters.
    pub nonce: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeDiagnosticCode {
    UnsupportedProtocol,
    InvalidNonce,
    UnsafeRelativePath,
    RuntimeNotVisible,
    ManifestUnreadable,
    HelperInspectionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeDiagnostic {
    pub code: BridgeDiagnosticCode,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeResponse {
    pub protocol_version: u16,
    pub success: bool,
    pub helper_version: String,
    pub build_target: String,
    pub helper_sha256: Option<String>,
    pub runtime_visible: bool,
    pub runtime_manifest_readable: bool,
    pub runtime_path_windows: Option<String>,
    pub runtime_manifest_sha256: Option<String>,
    pub environment_marker_matches: bool,
    pub nonce: Option<String>,
    pub diagnostic: Option<BridgeDiagnostic>,
}

impl BridgeResponse {
    pub fn failure(
        nonce: Option<String>,
        code: BridgeDiagnosticCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            success: false,
            helper_version: HELPER_VERSION.to_owned(),
            build_target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            helper_sha256: None,
            runtime_visible: false,
            runtime_manifest_readable: false,
            runtime_path_windows: None,
            runtime_manifest_sha256: None,
            environment_marker_matches: std::env::var(ENVIRONMENT_MARKER)
                .is_ok_and(|value| value == ENVIRONMENT_MARKER_VALUE),
            nonce,
            diagnostic: Some(BridgeDiagnostic {
                code,
                detail: detail.into(),
            }),
        }
    }
}
