use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationKind {
    Wwmi,
    Zzmi,
    Gimi,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    XxmiLibraries,
    GameIntegration(IntegrationKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageSource {
    LocalDirectory {
        path: PathBuf,
    },
    LocalArchive {
        path: PathBuf,
    },
    ManagedCache {
        id: String,
    },
    OfficialRelease {
        repository: String,
        tag: String,
        asset: String,
    },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub raw: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageFile {
    pub relative_path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub schema_version: u32,
    pub id: String,
    pub ecosystem: String,
    pub kind: PackageKind,
    pub version: Option<VersionInfo>,
    pub source: PackageSource,
    pub imported_unix_seconds: u64,
    pub files: Vec<PackageFile>,
    /// Identifies the structural specification, not the origin/authenticity of local bytes.
    pub layout_reference: String,
    pub authenticity: PackageAuthenticity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageAuthenticity {
    NotAuthenticated,
}
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchCompatibility {
    NotVerified,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageInspection {
    pub kind: PackageKind,
    pub version: Option<VersionInfo>,
    pub files: Vec<PackageFile>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ImportLimits {
    pub max_files: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_depth: usize,
}
impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_files: 4096,
            max_file_bytes: 128 * 1024 * 1024,
            max_total_bytes: 512 * 1024 * 1024,
            max_depth: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnsupportedIntegration,
    InvalidPackage,
    MissingRequiredFile,
    ChecksumMismatch,
    UnsafePath,
    UnsupportedArchive,
    StorageUnavailable,
    PermissionDenied,
    Io,
    LimitExceeded,
    WrongGameIntegration,
    InvalidMetadata,
    Busy,
    NotFound,
}
#[derive(Debug, Clone, Serialize)]
pub struct XxmiError {
    pub code: ErrorCode,
    pub path: Option<PathBuf>,
    pub detail: String,
}
impl XxmiError {
    pub fn new(
        code: ErrorCode,
        path: impl Into<Option<PathBuf>>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
    pub(crate) fn io(path: &std::path::Path, error: std::io::Error) -> Self {
        let code = match error.raw_os_error() {
            Some(libc::ELOOP | libc::ENOTDIR) => ErrorCode::UnsafePath,
            _ if error.kind() == std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            _ if error.kind() == std::io::ErrorKind::PermissionDenied => {
                ErrorCode::PermissionDenied
            }
            _ => ErrorCode::Io,
        };
        Self::new(code, Some(path.to_owned()), error.to_string())
    }
}
impl fmt::Display for XxmiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail)
    }
}
impl std::error::Error for XxmiError {}
pub type Result<T> = std::result::Result<T, XxmiError>;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Absent,
    StructurallyPresent,
    Incomplete,
    Unreadable,
}
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDiscovery {
    pub path: PathBuf,
    pub integration_kind: Option<IntegrationKind>,
    pub integration: Presence,
    pub libraries: Presence,
    pub version: Option<VersionInfo>,
    pub evidence: Vec<String>,
    pub issues: Vec<XxmiError>,
    pub launch_compatibility: LaunchCompatibility,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationRequirementState {
    Satisfied,
    Unsatisfied,
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlatformVerification {
    Verified,
    Unverified,
    Unsupported,
}
#[derive(Debug, Clone, Serialize)]
pub struct IntegrationRequirement {
    pub name: String,
    pub state: IntegrationRequirementState,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct IntegrationAssessment {
    pub game_id: String,
    pub integration: IntegrationKind,
    pub distribution: String,
    pub game_support: IntegrationRequirementState,
    pub steam_support: PlatformVerification,
    pub linux_proton_support: PlatformVerification,
    /// Package/layout planning only. This does not authorize an apply operation.
    pub platform_compatibility_verified: bool,
    pub requirements: Vec<IntegrationRequirement>,
    pub planning_possible: bool,
    pub launch_compatibility: LaunchCompatibility,
}
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileAction {
    Create,
    ReplaceWithBackup,
    Unchanged,
}
#[derive(Debug, Clone, Serialize)]
pub struct PlannedFile {
    pub source: PathBuf,
    pub target: PathBuf,
    pub sha256: String,
    pub action: FileAction,
    pub previous_sha256: Option<String>,
    pub backup_required: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct InstallationPlan {
    pub assessment: IntegrationAssessment,
    pub files: Vec<PlannedFile>,
    pub managed_target: PathBuf,
    pub game_executable_candidate: Option<PathBuf>,
    pub configuration_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub executable: bool,
}
