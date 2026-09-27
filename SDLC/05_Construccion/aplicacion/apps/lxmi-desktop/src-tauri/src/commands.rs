use serde::Serialize;
use std::path::Path;

use lxmi_core::{
    GameInstallationStatus, ProtonCompatDataStatus, SteamDetectionIssue, SteamDetectionIssueCode,
    SteamDetectionStatus, SteamInstallation, SteamLibrary, SteamScanResult, SystemInfo,
};
use lxmi_steam::{
    SteamAppScanIssue, SteamAppScanIssueCode, SteamAppScanIssueSeverity, SteamDiscoveryResult,
    SteamDiscoveryScanner, SteamGameDiscoveryResult, SteamGameDiscoveryStatus,
    SteamGameInstallation,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoDto {
    os: String,
    architecture: String,
    home_directory: Option<String>,
    xdg_data_home: Option<String>,
    xdg_data_dirs: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamScanDto {
    status: &'static str,
    installations: Vec<SteamInstallationDto>,
    issues: Vec<SteamIssueDto>,
    game_scan_status: &'static str,
    manifests_parsed: usize,
    ignored_unknown_apps: usize,
    games: Vec<SteamGameDto>,
    game_issues: Vec<SteamGameIssueDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGameDto {
    id: &'static str,
    name: &'static str,
    steam_app_id: u32,
    manifest_name: String,
    install_path: String,
    install_status: &'static str,
    steam_library: String,
    compatdata_status: &'static str,
    compatdata_path: String,
    prefix_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGameIssueDto {
    code: &'static str,
    severity: &'static str,
    path: String,
    detail: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamInstallationDto {
    root_path: String,
    libraries: Vec<SteamLibraryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamLibraryDto {
    path: String,
    is_default: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamIssueDto {
    code: &'static str,
    path: Option<String>,
    detail: Option<String>,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfoDto {
    SystemInfoDto::from(SystemInfo::current())
}

#[tauri::command]
pub async fn scan_steam() -> SteamScanDto {
    let discovery = match tauri::async_runtime::spawn_blocking(|| {
        SteamDiscoveryScanner::from_environment().scan()
    })
    .await
    {
        Ok(discovery) => discovery,
        Err(error) => SteamDiscoveryResult {
            steam: SteamScanResult {
                status: SteamDetectionStatus::InternalError,
                installations: Vec::new(),
                issues: vec![SteamDetectionIssue {
                    code: SteamDetectionIssueCode::FilesystemError,
                    path: None,
                    detail: Some(error.to_string()),
                }],
            },
            games: SteamGameDiscoveryResult {
                status: SteamGameDiscoveryStatus::NotAvailable,
                manifests_parsed: 0,
                ignored_unknown_apps: 0,
                games: Vec::new(),
                issues: Vec::new(),
            },
        },
    };

    SteamScanDto::from(discovery)
}

impl From<SystemInfo> for SystemInfoDto {
    fn from(info: SystemInfo) -> Self {
        Self {
            os: match info.os.as_str() {
                "linux" => "Linux".to_owned(),
                other => other.to_owned(),
            },
            architecture: info.architecture,
            home_directory: info.home_directory.as_deref().map(path_to_string),
            xdg_data_home: info.xdg_data_home.as_deref().map(path_to_string),
            xdg_data_dirs: info
                .xdg_data_dirs
                .iter()
                .map(|path| path_to_string(path))
                .collect(),
        }
    }
}

impl From<SteamDiscoveryResult> for SteamScanDto {
    fn from(discovery: SteamDiscoveryResult) -> Self {
        Self {
            status: status_code(&discovery.steam.status),
            installations: discovery
                .steam
                .installations
                .iter()
                .map(SteamInstallationDto::from)
                .collect(),
            issues: discovery
                .steam
                .issues
                .iter()
                .map(SteamIssueDto::from)
                .collect(),
            game_scan_status: game_scan_status_code(&discovery.games.status),
            manifests_parsed: discovery.games.manifests_parsed,
            ignored_unknown_apps: discovery.games.ignored_unknown_apps,
            games: discovery
                .games
                .games
                .iter()
                .map(SteamGameDto::from)
                .collect(),
            game_issues: discovery
                .games
                .issues
                .iter()
                .map(SteamGameIssueDto::from)
                .collect(),
        }
    }
}

impl From<&SteamGameInstallation> for SteamGameDto {
    fn from(game: &SteamGameInstallation) -> Self {
        Self {
            id: game.installation.game.id,
            name: game.installation.game.name,
            steam_app_id: game.steam_app_id,
            manifest_name: game.manifest_name.clone(),
            install_path: path_to_string(&game.installation.install_path),
            install_status: installation_status_code(&game.installation.status),
            steam_library: path_to_string(&game.steam_library),
            compatdata_status: compatdata_status_code(&game.compatdata.status),
            compatdata_path: path_to_string(&game.compatdata.compatdata_path),
            prefix_path: game.compatdata.prefix_path.as_deref().map(path_to_string),
        }
    }
}

impl From<&SteamAppScanIssue> for SteamGameIssueDto {
    fn from(issue: &SteamAppScanIssue) -> Self {
        Self {
            code: game_issue_code(&issue.code),
            severity: match issue.severity {
                SteamAppScanIssueSeverity::Warning => "warning",
                SteamAppScanIssueSeverity::Error => "error",
            },
            path: path_to_string(&issue.path),
            detail: issue.detail.clone(),
        }
    }
}

impl From<&SteamInstallation> for SteamInstallationDto {
    fn from(installation: &SteamInstallation) -> Self {
        Self {
            root_path: path_to_string(&installation.root_path),
            libraries: installation
                .libraries
                .iter()
                .map(SteamLibraryDto::from)
                .collect(),
        }
    }
}

impl From<&SteamLibrary> for SteamLibraryDto {
    fn from(library: &SteamLibrary) -> Self {
        Self {
            path: path_to_string(&library.path),
            is_default: library.is_default,
        }
    }
}

impl From<&SteamDetectionIssue> for SteamIssueDto {
    fn from(issue: &SteamDetectionIssue) -> Self {
        Self {
            code: issue_code(&issue.code),
            path: issue.path.as_deref().map(path_to_string),
            detail: issue.detail.clone(),
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn status_code(status: &SteamDetectionStatus) -> &'static str {
    match status {
        SteamDetectionStatus::NotInstalled => "not_installed",
        SteamDetectionStatus::Detected => "detected",
        SteamDetectionStatus::ConfigurationMissing => "configuration_missing",
        SteamDetectionStatus::InvalidConfiguration => "invalid_configuration",
        SteamDetectionStatus::PermissionDenied => "permission_denied",
        SteamDetectionStatus::HomeDirectoryUnavailable => "home_directory_unavailable",
        SteamDetectionStatus::InternalError => "internal_error",
    }
}

fn issue_code(code: &SteamDetectionIssueCode) -> &'static str {
    match code {
        SteamDetectionIssueCode::SteamRootUnreadable => "steam_root_unreadable",
        SteamDetectionIssueCode::SteamappsMissing => "steamapps_missing",
        SteamDetectionIssueCode::ConfigurationMissing => "configuration_missing",
        SteamDetectionIssueCode::ConfigurationUnreadable => "configuration_unreadable",
        SteamDetectionIssueCode::ConfigurationNotRegularFile => "configuration_not_regular_file",
        SteamDetectionIssueCode::InvalidConfiguration => "invalid_configuration",
        SteamDetectionIssueCode::InvalidLibraryPath => "invalid_library_path",
        SteamDetectionIssueCode::LibraryMissing => "library_missing",
        SteamDetectionIssueCode::LibraryPermissionDenied => "library_permission_denied",
        SteamDetectionIssueCode::FilesystemError => "filesystem_error",
    }
}

fn game_scan_status_code(status: &SteamGameDiscoveryStatus) -> &'static str {
    match status {
        SteamGameDiscoveryStatus::NotAvailable => "not_available",
        SteamGameDiscoveryStatus::Complete => "complete",
        SteamGameDiscoveryStatus::Partial => "partial",
    }
}

fn installation_status_code(status: &GameInstallationStatus) -> &'static str {
    match status {
        GameInstallationStatus::Installed => "installed",
        GameInstallationStatus::DirectoryMissing => "directory_missing",
        GameInstallationStatus::DirectoryInvalid => "directory_invalid",
        GameInstallationStatus::PermissionDenied => "permission_denied",
    }
}

fn compatdata_status_code(status: &ProtonCompatDataStatus) -> &'static str {
    match status {
        ProtonCompatDataStatus::NotFound => "not_found",
        ProtonCompatDataStatus::CompatDataFound => "compatdata_found",
        ProtonCompatDataStatus::PrefixFound => "prefix_found",
        ProtonCompatDataStatus::Invalid => "invalid",
        ProtonCompatDataStatus::PermissionDenied => "permission_denied",
        ProtonCompatDataStatus::IoError => "io_error",
    }
}

fn game_issue_code(code: &SteamAppScanIssueCode) -> &'static str {
    match code {
        SteamAppScanIssueCode::SteamAppsUnavailable => "steamapps_unavailable",
        SteamAppScanIssueCode::InvalidManifestFilename => "invalid_manifest_filename",
        SteamAppScanIssueCode::ManifestNotRegularFile => "manifest_not_regular_file",
        SteamAppScanIssueCode::ManifestTooLarge => "manifest_too_large",
        SteamAppScanIssueCode::ManifestInvalid => "manifest_invalid",
        SteamAppScanIssueCode::ManifestIdMismatch => "manifest_id_mismatch",
        SteamAppScanIssueCode::GameDirectoryMissing => "game_directory_missing",
        SteamAppScanIssueCode::GameDirectoryInvalid => "game_directory_invalid",
        SteamAppScanIssueCode::GameDirectoryPermissionDenied => "game_directory_permission_denied",
        SteamAppScanIssueCode::PermissionDenied => "permission_denied",
        SteamAppScanIssueCode::CompatDataInvalid => "compatdata_invalid",
        SteamAppScanIssueCode::FilesystemError => "filesystem_error",
    }
}

#[cfg(test)]
mod tests {
    use super::status_code;
    use lxmi_core::SteamDetectionStatus;

    #[test]
    fn serializes_error_status_as_a_stable_frontend_code() {
        assert_eq!(
            status_code(&SteamDetectionStatus::ConfigurationMissing),
            "configuration_missing"
        );
    }
}
