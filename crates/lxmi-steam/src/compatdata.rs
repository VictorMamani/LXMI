use std::fs;
use std::path::Path;

use lxmi_core::{ProtonCompatData, ProtonCompatDataStatus};

use crate::{SteamAppScanIssue, SteamAppScanIssueCode, SteamAppScanIssueSeverity};

pub(crate) fn inspect_compatdata(
    library: &Path,
    app_id: u32,
) -> (ProtonCompatData, Option<SteamAppScanIssue>) {
    let compatdata_path = library
        .join("steamapps")
        .join("compatdata")
        .join(app_id.to_string());
    let not_found = || ProtonCompatData {
        app_id,
        compatdata_path: compatdata_path.clone(),
        prefix_path: None,
        status: ProtonCompatDataStatus::NotFound,
    };

    let metadata = match fs::symlink_metadata(&compatdata_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (not_found(), None),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return (
                ProtonCompatData {
                    status: ProtonCompatDataStatus::PermissionDenied,
                    ..not_found()
                },
                Some(issue(
                    SteamAppScanIssueCode::PermissionDenied,
                    &compatdata_path,
                    error.to_string(),
                )),
            );
        }
        Err(error) => {
            return (
                ProtonCompatData {
                    status: ProtonCompatDataStatus::IoError,
                    ..not_found()
                },
                Some(issue(
                    SteamAppScanIssueCode::FilesystemError,
                    &compatdata_path,
                    error.to_string(),
                )),
            );
        }
    };

    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return (
            ProtonCompatData {
                status: ProtonCompatDataStatus::Invalid,
                ..not_found()
            },
            Some(issue(
                SteamAppScanIssueCode::CompatDataInvalid,
                &compatdata_path,
                "compatdata path must be a regular directory".to_owned(),
            )),
        );
    }

    let prefix_path = compatdata_path.join("pfx");
    match fs::symlink_metadata(&prefix_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => (
            ProtonCompatData {
                app_id,
                compatdata_path: compatdata_path.clone(),
                prefix_path: None,
                status: ProtonCompatDataStatus::Invalid,
            },
            Some(issue(
                SteamAppScanIssueCode::CompatDataInvalid,
                &prefix_path,
                "pfx must be a regular directory".to_owned(),
            )),
        ),
        Ok(_) => (
            ProtonCompatData {
                app_id,
                compatdata_path,
                prefix_path: Some(prefix_path),
                status: ProtonCompatDataStatus::PrefixFound,
            },
            None,
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            ProtonCompatData {
                app_id,
                compatdata_path,
                prefix_path: None,
                status: ProtonCompatDataStatus::CompatDataFound,
            },
            None,
        ),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => (
            ProtonCompatData {
                app_id,
                compatdata_path: compatdata_path.clone(),
                prefix_path: None,
                status: ProtonCompatDataStatus::PermissionDenied,
            },
            Some(issue(
                SteamAppScanIssueCode::PermissionDenied,
                &prefix_path,
                error.to_string(),
            )),
        ),
        Err(error) => (
            ProtonCompatData {
                app_id,
                compatdata_path: compatdata_path.clone(),
                prefix_path: None,
                status: ProtonCompatDataStatus::IoError,
            },
            Some(issue(
                SteamAppScanIssueCode::FilesystemError,
                &prefix_path,
                error.to_string(),
            )),
        ),
    }
}

fn issue(code: SteamAppScanIssueCode, path: &Path, detail: String) -> SteamAppScanIssue {
    SteamAppScanIssue {
        code,
        severity: SteamAppScanIssueSeverity::Error,
        path: path.to_owned(),
        detail: Some(detail),
    }
}

#[cfg(test)]
mod tests {
    use super::inspect_compatdata;
    use lxmi_core::ProtonCompatDataStatus;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    const APP_ID: u32 = 3513350;

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("lxmi-compatdata-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&root).expect("temporary fixture root should be created");
            Self(root)
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn reports_compatdata_not_found() {
        let tree = TempTree::new();
        let (result, issue) = inspect_compatdata(&tree.0, APP_ID);
        assert_eq!(result.status, ProtonCompatDataStatus::NotFound);
        assert_eq!(issue, None);
    }

    #[test]
    fn reports_compatdata_without_pfx() {
        let tree = TempTree::new();
        let compatdata = tree.0.join("steamapps/compatdata").join(APP_ID.to_string());
        fs::create_dir_all(&compatdata).expect("compatdata fixture should be created");

        let (result, issue) = inspect_compatdata(&tree.0, APP_ID);
        assert_eq!(result.status, ProtonCompatDataStatus::CompatDataFound);
        assert_eq!(result.compatdata_path, compatdata);
        assert_eq!(result.prefix_path, None);
        assert_eq!(issue, None);
    }

    #[test]
    fn reports_pfx_as_a_prefix_candidate() {
        let tree = TempTree::new();
        let compatdata = tree.0.join("steamapps/compatdata").join(APP_ID.to_string());
        let prefix = compatdata.join("pfx");
        fs::create_dir_all(&prefix).expect("synthetic pfx should be created");

        let (result, issue) = inspect_compatdata(&tree.0, APP_ID);
        assert_eq!(result.status, ProtonCompatDataStatus::PrefixFound);
        assert_eq!(result.prefix_path, Some(prefix));
        assert_eq!(issue, None);
    }

    #[test]
    fn rejects_non_directory_compatdata_without_following_it() {
        let tree = TempTree::new();
        let compatdata = tree.0.join("steamapps/compatdata").join(APP_ID.to_string());
        fs::create_dir_all(compatdata.parent().expect("parent path"))
            .expect("compatdata parent should be created");
        fs::write(&compatdata, "synthetic file").expect("fixture file should be written");

        let (result, issue) = inspect_compatdata(&tree.0, APP_ID);
        assert_eq!(result.status, ProtonCompatDataStatus::Invalid);
        assert!(issue.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_pfx_directory() {
        use std::os::unix::fs::symlink;

        let tree = TempTree::new();
        let compatdata = tree.0.join("steamapps/compatdata").join(APP_ID.to_string());
        let outside = tree.0.join("outside-prefix");
        fs::create_dir_all(&compatdata).expect("compatdata fixture should be created");
        fs::create_dir_all(&outside).expect("outside directory should be created");
        symlink(outside, compatdata.join("pfx")).expect("pfx symlink should be created");

        let (result, issue) = inspect_compatdata(&tree.0, APP_ID);
        assert_eq!(result.status, ProtonCompatDataStatus::Invalid);
        assert_eq!(result.prefix_path, None);
        assert!(issue.is_some());
    }
}
