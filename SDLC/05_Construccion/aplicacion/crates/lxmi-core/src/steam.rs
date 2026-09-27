use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamDetectionStatus {
    NotInstalled,
    Detected,
    ConfigurationMissing,
    InvalidConfiguration,
    PermissionDenied,
    HomeDirectoryUnavailable,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamDetectionIssueCode {
    SteamRootUnreadable,
    SteamappsMissing,
    ConfigurationMissing,
    ConfigurationUnreadable,
    ConfigurationNotRegularFile,
    InvalidConfiguration,
    InvalidLibraryPath,
    LibraryMissing,
    LibraryPermissionDenied,
    FilesystemError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamDetectionIssue {
    pub code: SteamDetectionIssueCode,
    pub path: Option<PathBuf>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLibrary {
    pub path: PathBuf,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamInstallation {
    pub root_path: PathBuf,
    pub libraries: Vec<SteamLibrary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamScanResult {
    pub status: SteamDetectionStatus,
    pub installations: Vec<SteamInstallation>,
    pub issues: Vec<SteamDetectionIssue>,
}

pub trait SteamDetector {
    fn scan(&self) -> SteamScanResult;
}
