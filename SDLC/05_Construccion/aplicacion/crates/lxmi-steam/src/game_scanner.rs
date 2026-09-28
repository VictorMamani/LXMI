use std::collections::VecDeque;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use lxmi_core::SteamLibrary;
use tracing::{info, warn};

use crate::{parse_app_manifest, SteamAppManifest};

const MAX_APP_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_EXECUTABLE_SEARCH_DEPTH: usize = 8;
const MAX_EXECUTABLE_SEARCH_ENTRIES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameExecutableDiscovery {
    pub path: Option<PathBuf>,
    pub status: lxmi_core::GameExecutableStatus,
}

/// Searches a bounded tree without following symlinks or opening/executing files.
pub fn find_expected_game_executable(
    install_path: &Path,
    expected_names: &[&str],
) -> GameExecutableDiscovery {
    use lxmi_core::GameExecutableStatus;
    use std::fs;

    if expected_names.is_empty() {
        return GameExecutableDiscovery {
            path: None,
            status: GameExecutableStatus::NotScanned,
        };
    }

    let root_metadata = match fs::symlink_metadata(install_path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => metadata,
        _ => {
            return GameExecutableDiscovery {
                path: None,
                status: GameExecutableStatus::SearchIncomplete,
            }
        }
    };
    let _ = root_metadata;

    let mut pending = VecDeque::from([(install_path.to_owned(), 0usize)]);
    let mut examined = 0usize;
    let mut incomplete = false;
    while !pending.is_empty() && examined < MAX_EXECUTABLE_SEARCH_ENTRIES {
        let level_len = pending.len();
        let mut found = Vec::new();
        for _ in 0..level_len {
            let Some((directory, depth)) = pending.pop_front() else {
                break;
            };
            let entries = match fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(_) => {
                    incomplete = true;
                    continue;
                }
            };
            for entry in entries {
                examined += 1;
                if examined > MAX_EXECUTABLE_SEARCH_ENTRIES {
                    incomplete = true;
                    break;
                }
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => {
                        incomplete = true;
                        continue;
                    }
                };
                let path = entry.path();
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        incomplete = true;
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() {
                    continue;
                }
                if metadata.is_file() {
                    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                        continue;
                    };
                    if let Some(priority) = expected_names
                        .iter()
                        .position(|expected| name.eq_ignore_ascii_case(expected))
                    {
                        found.push((priority, path));
                    }
                } else if metadata.is_dir() {
                    if depth < MAX_EXECUTABLE_SEARCH_DEPTH {
                        pending.push_back((path, depth + 1));
                    } else {
                        incomplete = true;
                    }
                }
            }
        }
        if !found.is_empty() {
            found.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
            return GameExecutableDiscovery {
                path: found.into_iter().next().map(|(_, path)| path),
                status: GameExecutableStatus::Found,
            };
        }
    }

    GameExecutableDiscovery {
        path: None,
        status: if incomplete || !pending.is_empty() {
            GameExecutableStatus::SearchIncomplete
        } else {
            GameExecutableStatus::NotFound
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamAppInstallStatus {
    Installed,
    DirectoryMissing,
    DirectoryInvalid,
    PermissionDenied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAppInstallation {
    pub manifest: SteamAppManifest,
    pub library_path: PathBuf,
    pub game_path: PathBuf,
    pub status: SteamAppInstallStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamAppScanStatus {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamAppScanIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamAppScanIssueCode {
    SteamAppsUnavailable,
    InvalidManifestFilename,
    ManifestNotRegularFile,
    ManifestTooLarge,
    ManifestInvalid,
    ManifestIdMismatch,
    GameDirectoryMissing,
    GameDirectoryInvalid,
    GameDirectoryPermissionDenied,
    CompatDataInvalid,
    PermissionDenied,
    FilesystemError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAppScanIssue {
    pub code: SteamAppScanIssueCode,
    pub severity: SteamAppScanIssueSeverity,
    pub path: PathBuf,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAppScanResult {
    pub status: SteamAppScanStatus,
    pub manifests_parsed: usize,
    pub apps: Vec<SteamAppInstallation>,
    pub issues: Vec<SteamAppScanIssue>,
}

pub struct SteamAppScanner;

impl SteamAppScanner {
    pub fn scan(&self, libraries: &[SteamLibrary]) -> SteamAppScanResult {
        let mut apps = Vec::new();
        let mut issues = Vec::new();
        let mut manifests_parsed = 0;

        for library in libraries {
            let steamapps = library.path.join("steamapps");
            match fs::symlink_metadata(&steamapps) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::SteamAppsUnavailable,
                        SteamAppScanIssueSeverity::Error,
                        steamapps,
                        Some("steamapps must be a regular directory".to_owned()),
                    );
                    continue;
                }
                Ok(_) => {}
                Err(error) => {
                    let (code, severity) = classify_directory_error(&error);
                    push_issue(
                        &mut issues,
                        code,
                        severity,
                        steamapps,
                        Some(error.to_string()),
                    );
                    continue;
                }
            }

            let entries = match fs::read_dir(&steamapps) {
                Ok(entries) => entries,
                Err(error) => {
                    let (code, severity) = classify_directory_error(&error);
                    push_issue(
                        &mut issues,
                        code,
                        severity,
                        steamapps,
                        Some(error.to_string()),
                    );
                    continue;
                }
            };

            let mut manifest_paths = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if let Some(app_id) = manifest_app_id(&path) {
                            manifest_paths.push((app_id, path));
                        } else if has_manifest_shape(&path) {
                            push_issue(
                                &mut issues,
                                SteamAppScanIssueCode::InvalidManifestFilename,
                                SteamAppScanIssueSeverity::Warning,
                                path,
                                None,
                            );
                        }
                    }
                    Err(error) => push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::FilesystemError,
                        SteamAppScanIssueSeverity::Error,
                        steamapps.clone(),
                        Some(error.to_string()),
                    ),
                }
            }
            manifest_paths.sort_by(|left, right| left.1.cmp(&right.1));

            for (filename_app_id, manifest_path) in manifest_paths {
                let metadata = match fs::symlink_metadata(&manifest_path) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        let (code, severity) = classify_manifest_error(&error);
                        push_issue(
                            &mut issues,
                            code,
                            severity,
                            manifest_path,
                            Some(error.to_string()),
                        );
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::ManifestNotRegularFile,
                        SteamAppScanIssueSeverity::Warning,
                        manifest_path,
                        None,
                    );
                    continue;
                }
                if metadata.len() > MAX_APP_MANIFEST_BYTES {
                    push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::ManifestTooLarge,
                        SteamAppScanIssueSeverity::Warning,
                        manifest_path,
                        Some("appmanifest exceeds the 2 MiB limit".to_owned()),
                    );
                    continue;
                }

                let file = match fs::File::open(&manifest_path) {
                    Ok(file) => file,
                    Err(error) => {
                        let (code, severity) = classify_manifest_error(&error);
                        push_issue(
                            &mut issues,
                            code,
                            severity,
                            manifest_path,
                            Some(error.to_string()),
                        );
                        continue;
                    }
                };
                let mut contents = String::new();
                if let Err(error) = file
                    .take(MAX_APP_MANIFEST_BYTES + 1)
                    .read_to_string(&mut contents)
                {
                    let (code, severity) = classify_manifest_error(&error);
                    push_issue(
                        &mut issues,
                        code,
                        severity,
                        manifest_path,
                        Some(error.to_string()),
                    );
                    continue;
                }
                if contents.len() as u64 > MAX_APP_MANIFEST_BYTES {
                    push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::ManifestTooLarge,
                        SteamAppScanIssueSeverity::Warning,
                        manifest_path,
                        Some("appmanifest exceeds the 2 MiB limit".to_owned()),
                    );
                    continue;
                }

                let manifest = match parse_app_manifest(&contents) {
                    Ok(manifest) => manifest,
                    Err(error) => {
                        warn!(path = %manifest_path.display(), error = %error, "Invalid app manifest");
                        push_issue(
                            &mut issues,
                            SteamAppScanIssueCode::ManifestInvalid,
                            SteamAppScanIssueSeverity::Warning,
                            manifest_path,
                            Some(error.to_string()),
                        );
                        continue;
                    }
                };
                if manifest.app_id != filename_app_id {
                    push_issue(
                        &mut issues,
                        SteamAppScanIssueCode::ManifestIdMismatch,
                        SteamAppScanIssueSeverity::Warning,
                        manifest_path,
                        Some(format!(
                            "filename appid {filename_app_id} differs from manifest appid {}",
                            manifest.app_id
                        )),
                    );
                    continue;
                }

                manifests_parsed += 1;
                let game_path = steamapps.join("common").join(&manifest.install_dir);
                let install_status = inspect_game_directory(&game_path, &mut issues);
                info!(
                    app_id = manifest.app_id,
                    app_name = %manifest.name,
                    path = %game_path.display(),
                    "App manifest parsed"
                );
                apps.push(SteamAppInstallation {
                    manifest,
                    library_path: library.path.clone(),
                    game_path,
                    status: install_status,
                });
            }
        }

        apps.sort_by(|left, right| {
            left.manifest
                .app_id
                .cmp(&right.manifest.app_id)
                .then_with(|| left.library_path.cmp(&right.library_path))
        });
        let status = if issues
            .iter()
            .any(|issue| issue.severity == SteamAppScanIssueSeverity::Error)
        {
            SteamAppScanStatus::Partial
        } else {
            SteamAppScanStatus::Complete
        };

        SteamAppScanResult {
            status,
            manifests_parsed,
            apps,
            issues,
        }
    }
}

fn inspect_game_directory(
    path: &Path,
    issues: &mut Vec<SteamAppScanIssue>,
) -> SteamAppInstallStatus {
    let Some(common_directory) = path.parent() else {
        return SteamAppInstallStatus::DirectoryInvalid;
    };
    match fs::symlink_metadata(common_directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryInvalid,
                SteamAppScanIssueSeverity::Warning,
                common_directory.to_owned(),
                Some("steamapps/common must be a regular directory".to_owned()),
            );
            return SteamAppInstallStatus::DirectoryInvalid;
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryMissing,
                SteamAppScanIssueSeverity::Warning,
                path.to_owned(),
                None,
            );
            return SteamAppInstallStatus::DirectoryMissing;
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryPermissionDenied,
                SteamAppScanIssueSeverity::Error,
                common_directory.to_owned(),
                Some(error.to_string()),
            );
            return SteamAppInstallStatus::PermissionDenied;
        }
        Err(error) => {
            push_issue(
                issues,
                SteamAppScanIssueCode::FilesystemError,
                SteamAppScanIssueSeverity::Error,
                common_directory.to_owned(),
                Some(error.to_string()),
            );
            return SteamAppInstallStatus::DirectoryInvalid;
        }
    }

    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryInvalid,
                SteamAppScanIssueSeverity::Warning,
                path.to_owned(),
                Some("game install path must be a regular directory".to_owned()),
            );
            SteamAppInstallStatus::DirectoryInvalid
        }
        Ok(_) => SteamAppInstallStatus::Installed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryMissing,
                SteamAppScanIssueSeverity::Warning,
                path.to_owned(),
                None,
            );
            SteamAppInstallStatus::DirectoryMissing
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            push_issue(
                issues,
                SteamAppScanIssueCode::GameDirectoryPermissionDenied,
                SteamAppScanIssueSeverity::Error,
                path.to_owned(),
                Some(error.to_string()),
            );
            SteamAppInstallStatus::PermissionDenied
        }
        Err(error) => {
            push_issue(
                issues,
                SteamAppScanIssueCode::FilesystemError,
                SteamAppScanIssueSeverity::Error,
                path.to_owned(),
                Some(error.to_string()),
            );
            SteamAppInstallStatus::DirectoryInvalid
        }
    }
}

fn manifest_app_id(path: &Path) -> Option<u32> {
    let filename = path.file_name()?.to_str()?;
    let app_id = filename
        .strip_prefix("appmanifest_")?
        .strip_suffix(".acf")?;
    app_id.parse().ok().filter(|id| *id > 0)
}

fn has_manifest_shape(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"))
}

fn classify_directory_error(
    error: &std::io::Error,
) -> (SteamAppScanIssueCode, SteamAppScanIssueSeverity) {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        (
            SteamAppScanIssueCode::PermissionDenied,
            SteamAppScanIssueSeverity::Error,
        )
    } else {
        (
            SteamAppScanIssueCode::FilesystemError,
            SteamAppScanIssueSeverity::Error,
        )
    }
}

fn classify_manifest_error(
    error: &std::io::Error,
) -> (SteamAppScanIssueCode, SteamAppScanIssueSeverity) {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        (
            SteamAppScanIssueCode::PermissionDenied,
            SteamAppScanIssueSeverity::Error,
        )
    } else {
        (
            SteamAppScanIssueCode::FilesystemError,
            SteamAppScanIssueSeverity::Error,
        )
    }
}

fn push_issue(
    issues: &mut Vec<SteamAppScanIssue>,
    code: SteamAppScanIssueCode,
    severity: SteamAppScanIssueSeverity,
    path: PathBuf,
    detail: Option<String>,
) {
    issues.push(SteamAppScanIssue {
        code,
        severity,
        path,
        detail,
    });
}

#[cfg(test)]
mod tests {
    use super::{has_manifest_shape, manifest_app_id};
    use std::path::Path;

    #[test]
    fn recognizes_only_numeric_acf_appmanifest_names() {
        assert_eq!(
            manifest_app_id(Path::new("appmanifest_3513350.acf")),
            Some(3513350)
        );
        assert_eq!(manifest_app_id(Path::new("appmanifest_x.acf")), None);
        assert!(has_manifest_shape(Path::new("appmanifest_x.acf")));
        assert!(!has_manifest_shape(Path::new("manifest_123.acf")));
    }
}
