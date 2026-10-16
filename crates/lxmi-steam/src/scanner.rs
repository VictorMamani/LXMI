use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use lxmi_core::{
    SteamDetectionIssue, SteamDetectionIssueCode, SteamDetectionStatus, SteamDetector,
    SteamInstallation, SteamLibrary, SteamScanResult,
};
use tracing::{info, warn};

use crate::parse_library_folders;

const MAX_LIBRARYFOLDERS_BYTES: u64 = 2 * 1024 * 1024;

pub struct SteamScanner {
    home_directory: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
}

impl SteamScanner {
    pub fn from_environment() -> Self {
        let home_directory = env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        let xdg_data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());

        Self {
            home_directory,
            xdg_data_home,
        }
    }

    pub fn with_paths(home_directory: Option<PathBuf>, xdg_data_home: Option<PathBuf>) -> Self {
        Self {
            home_directory: home_directory.filter(|path| path.is_absolute()),
            xdg_data_home: xdg_data_home.filter(|path| path.is_absolute()),
        }
    }

    fn candidates(&self) -> Vec<PathBuf> {
        let Some(home) = self.home_directory.as_ref() else {
            return Vec::new();
        };

        let mut candidates = vec![
            home.join(".steam/steam"),
            home.join(".steam/root"),
            home.join(".local/share/Steam"),
        ];
        if let Some(data_home) = self.xdg_data_home.as_ref() {
            candidates.push(data_home.join("Steam"));
        }
        candidates
    }
}

impl SteamDetector for SteamScanner {
    fn scan(&self) -> SteamScanResult {
        if self.home_directory.is_none() {
            return result(
                SteamDetectionStatus::HomeDirectoryUnavailable,
                vec![],
                vec![],
            );
        }

        info!("Steam detection started");
        let mut installations = Vec::new();
        let mut issues = Vec::new();
        let mut seen_roots = HashSet::new();
        let mut root_candidate_found = false;
        let mut saw_missing_configuration = false;
        let mut saw_invalid_configuration = false;
        let mut saw_permission_denied = false;
        let mut saw_internal_error = false;

        for candidate in self.candidates() {
            let metadata = match fs::metadata(&candidate) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    saw_permission_denied = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::SteamRootUnreadable,
                        Some(candidate.clone()),
                        Some(error.to_string()),
                    ));
                    continue;
                }
                Err(error) => {
                    saw_internal_error = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::FilesystemError,
                        Some(candidate.clone()),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            };

            if !metadata.is_dir() {
                continue;
            }
            root_candidate_found = true;

            let root = match fs::canonicalize(&candidate) {
                Ok(root) => root,
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    saw_permission_denied = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::SteamRootUnreadable,
                        Some(candidate.clone()),
                        Some(error.to_string()),
                    ));
                    continue;
                }
                Err(error) => {
                    saw_internal_error = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::FilesystemError,
                        Some(candidate.clone()),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            };

            if !seen_roots.insert(root.clone()) {
                continue;
            }

            info!(steam_root = %root.display(), "Steam root found");
            let steamapps = root.join("steamapps");
            match fs::metadata(&steamapps) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    saw_missing_configuration = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::SteamappsMissing,
                        Some(steamapps),
                        None,
                    ));
                    continue;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    saw_missing_configuration = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::SteamappsMissing,
                        Some(steamapps),
                        None,
                    ));
                    continue;
                }
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    saw_permission_denied = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::SteamRootUnreadable,
                        Some(steamapps),
                        Some(error.to_string()),
                    ));
                    continue;
                }
                Err(error) => {
                    saw_internal_error = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::FilesystemError,
                        Some(steamapps),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            }

            let config_path = steamapps.join("libraryfolders.vdf");
            let config_metadata = match fs::symlink_metadata(&config_path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    saw_missing_configuration = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::ConfigurationMissing,
                        Some(config_path),
                        None,
                    ));
                    continue;
                }
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    saw_permission_denied = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::ConfigurationUnreadable,
                        Some(config_path),
                        Some(error.to_string()),
                    ));
                    continue;
                }
                Err(error) => {
                    saw_internal_error = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::FilesystemError,
                        Some(config_path),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            };

            if config_metadata.file_type().is_symlink() || !config_metadata.is_file() {
                saw_invalid_configuration = true;
                issues.push(issue(
                    SteamDetectionIssueCode::ConfigurationNotRegularFile,
                    Some(config_path),
                    None,
                ));
                continue;
            }
            if config_metadata.len() > MAX_LIBRARYFOLDERS_BYTES {
                saw_invalid_configuration = true;
                issues.push(issue(
                    SteamDetectionIssueCode::InvalidConfiguration,
                    Some(config_path),
                    Some("libraryfolders.vdf exceeds the 2 MiB limit".to_owned()),
                ));
                continue;
            }

            let file = match fs::File::open(&config_path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    saw_permission_denied = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::ConfigurationUnreadable,
                        Some(config_path),
                        Some(error.to_string()),
                    ));
                    continue;
                }
                Err(error) => {
                    saw_internal_error = true;
                    issues.push(issue(
                        SteamDetectionIssueCode::FilesystemError,
                        Some(config_path),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            };
            let mut content = String::new();
            if let Err(error) = file
                .take(MAX_LIBRARYFOLDERS_BYTES + 1)
                .read_to_string(&mut content)
            {
                saw_invalid_configuration = true;
                issues.push(issue(
                    SteamDetectionIssueCode::InvalidConfiguration,
                    Some(config_path),
                    Some(error.to_string()),
                ));
                continue;
            }
            if content.len() as u64 > MAX_LIBRARYFOLDERS_BYTES {
                saw_invalid_configuration = true;
                issues.push(issue(
                    SteamDetectionIssueCode::InvalidConfiguration,
                    Some(config_path),
                    Some("libraryfolders.vdf exceeds the 2 MiB limit".to_owned()),
                ));
                continue;
            }

            let registered_libraries = match parse_library_folders(&content) {
                Ok(paths) => paths,
                Err(error) => {
                    saw_invalid_configuration = true;
                    warn!(
                        path = %config_path.display(),
                        error = %error,
                        "Invalid libraryfolders.vdf"
                    );
                    issues.push(issue(
                        SteamDetectionIssueCode::InvalidConfiguration,
                        Some(config_path),
                        Some(error.to_string()),
                    ));
                    continue;
                }
            };

            let mut libraries = vec![SteamLibrary {
                path: root.clone(),
                is_default: true,
            }];
            let mut seen_libraries = HashSet::from([root.clone()]);

            for path in registered_libraries {
                if !path.is_absolute() {
                    issues.push(issue(
                        SteamDetectionIssueCode::InvalidLibraryPath,
                        Some(path),
                        None,
                    ));
                    continue;
                }

                match validate_library(&path) {
                    Ok(Some(path)) => {
                        if seen_libraries.insert(path.clone()) {
                            info!(library = %path.display(), "Steam library found");
                            libraries.push(SteamLibrary {
                                path,
                                is_default: false,
                            });
                        }
                    }
                    Ok(None) => issues.push(issue(
                        SteamDetectionIssueCode::LibraryMissing,
                        Some(path),
                        None,
                    )),
                    Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                        saw_permission_denied = true;
                        issues.push(issue(
                            SteamDetectionIssueCode::LibraryPermissionDenied,
                            Some(path),
                            Some(error.to_string()),
                        ));
                    }
                    Err(error) => {
                        saw_internal_error = true;
                        issues.push(issue(
                            SteamDetectionIssueCode::FilesystemError,
                            Some(path),
                            Some(error.to_string()),
                        ));
                    }
                }
            }

            installations.push(SteamInstallation {
                root_path: root,
                libraries,
            });
        }

        let status = if !installations.is_empty() {
            SteamDetectionStatus::Detected
        } else if saw_permission_denied {
            SteamDetectionStatus::PermissionDenied
        } else if saw_invalid_configuration {
            SteamDetectionStatus::InvalidConfiguration
        } else if saw_missing_configuration || root_candidate_found {
            SteamDetectionStatus::ConfigurationMissing
        } else if saw_internal_error {
            SteamDetectionStatus::InternalError
        } else {
            SteamDetectionStatus::NotInstalled
        };

        result(status, installations, issues)
    }
}

fn validate_library(path: &Path) -> std::io::Result<Option<PathBuf>> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() {
        return Ok(None);
    }

    let canonical = fs::canonicalize(path)?;
    let steamapps = canonical.join("steamapps");
    let metadata = match fs::metadata(steamapps) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() {
        return Ok(None);
    }

    Ok(Some(canonical))
}

fn issue(
    code: SteamDetectionIssueCode,
    path: Option<PathBuf>,
    detail: Option<String>,
) -> SteamDetectionIssue {
    SteamDetectionIssue { code, path, detail }
}

fn result(
    status: SteamDetectionStatus,
    installations: Vec<SteamInstallation>,
    issues: Vec<SteamDetectionIssue>,
) -> SteamScanResult {
    SteamScanResult {
        status,
        installations,
        issues,
    }
}
