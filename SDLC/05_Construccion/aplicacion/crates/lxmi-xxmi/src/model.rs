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
    #[serde(default)]
    pub upstream: Option<UpstreamProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageAuthenticity {
    NotAuthenticated,
    MissingSignature,
    SignatureInvalid,
    OfficialReleaseVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficialPackageKind {
    Zzmi,
    XxmiLibraries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTrust {
    Official,
    Untrusted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub id: u64,
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamRelease {
    pub package_kind: OfficialPackageKind,
    pub repository: String,
    pub release_id: u64,
    pub tag: String,
    pub commit: String,
    pub release_url: String,
    pub version: String,
    pub published_at: String,
    pub metadata_retrieved_at: String,
    pub signature_base64: Option<String>,
    pub assets: Vec<ReleaseAsset>,
    pub source_trust: SourceTrust,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamProvenance {
    pub repository: String,
    pub release_id: u64,
    pub tag: String,
    pub commit: String,
    pub release_url: String,
    pub asset_name: String,
    pub asset_url: String,
    pub published_at: String,
    pub metadata_retrieved_at: String,
    pub downloaded_at: String,
    pub download_sha256: String,
    pub expected_asset_sha256: Option<String>,
    pub signature_base64: Option<String>,
    pub signature_status: SignatureStatus,
    pub companion_asset_name: Option<String>,
    pub companion_asset_sha256: Option<String>,
    pub component_signatures_verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    Verified,
    Invalid,
    Missing,
    Unsupported,
    NotChecked,
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
    pub max_archive_bytes: u64,
    pub max_archive_entries: usize,
    pub max_compression_ratio: u64,
}
impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_files: 4096,
            max_file_bytes: 128 * 1024 * 1024,
            max_total_bytes: 512 * 1024 * 1024,
            max_depth: 24,
            max_archive_bytes: 32 * 1024 * 1024,
            max_archive_entries: 4096,
            max_compression_ratio: 1_000,
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
    InvalidSignature,
    MissingSignature,
    UnsafeArchive,
    Network,
    RedirectRejected,
    ReleaseUnavailable,
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
    pub deployment_mapping: Vec<DeploymentMapping>,
    pub dry_run: DryRunInstallation,
    pub executable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentOperation {
    MergePackageIntoConfiguredImporterDirectory,
    DeployRuntimeDllToConfiguredImporterDirectory,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeploymentMapping {
    pub source_package_id: String,
    pub source_relative_path: String,
    pub target_relative_path: String,
    pub operation: DeploymentOperation,
    pub target_root_basis: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DryRunStatus {
    WouldCreate,
    WouldReplace,
    AlreadyMatches,
    TargetMissing,
    Conflict,
    PermissionIssue,
    UnsafeTarget,
    Unresolved,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallSafetyState {
    NeedsReview,
    PlatformCompatibilityUnverified,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryRunFile {
    pub source_package_id: String,
    pub source_path: PathBuf,
    pub target_relative_path: String,
    pub comparison_path: Option<PathBuf>,
    pub expected_sha256: String,
    pub existing_sha256: Option<String>,
    pub status: DryRunStatus,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryRunInstallation {
    /// None means LXMI has not captured XXMI's configurable importer_folder.
    pub configured_target_root: Option<PathBuf>,
    /// The executable directory is inspected as a candidate only; it is not asserted as target.
    pub comparison_root_candidate: Option<PathBuf>,
    pub comparison_root_evidence: String,
    pub root_is_authoritative: bool,
    pub files: Vec<DryRunFile>,
    pub safety: InstallSafetyState,
    pub apply_allowed: bool,
    pub writes_performed: bool,
}
