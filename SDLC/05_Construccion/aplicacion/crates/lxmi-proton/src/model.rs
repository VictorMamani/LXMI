use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolSource {
    SteamLibrary,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolKind {
    Proton,
    SteamLinuxRuntime,
    Other,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolStatus {
    Valid,
    Incomplete,
    InvalidMetadata,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityTool {
    pub internal_id: Option<String>,
    pub display_name: String,
    pub path: PathBuf,
    pub metadata_path: Option<PathBuf>,
    pub source: CompatibilityToolSource,
    pub kind: CompatibilityToolKind,
    pub version: Option<String>,
    pub status: CompatibilityToolStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolDiscoveryStatus {
    NotAvailable,
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolIssueCode {
    CommonDirectoryInvalid,
    CustomDirectoryInvalid,
    DirectoryUnreadable,
    EntryUnreadable,
    SymlinkRejected,
    FileNotRegular,
    FileTooLarge,
    MetadataInvalid,
    InstallPathUnsafe,
    ToolDirectoryMissing,
    FilesystemError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityToolIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityToolIssue {
    pub code: CompatibilityToolIssueCode,
    pub severity: CompatibilityToolIssueSeverity,
    pub path: PathBuf,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityToolDiscoveryResult {
    pub status: CompatibilityToolDiscoveryStatus,
    pub tools: Vec<CompatibilityTool>,
    pub issues: Vec<CompatibilityToolIssue>,
}

impl CompatibilityToolDiscoveryResult {
    pub fn not_available() -> Self {
        Self {
            status: CompatibilityToolDiscoveryStatus::NotAvailable,
            tools: Vec::new(),
            issues: Vec::new(),
        }
    }
}
