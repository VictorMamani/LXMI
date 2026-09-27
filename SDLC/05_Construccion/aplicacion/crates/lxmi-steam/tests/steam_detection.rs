use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use lxmi_core::{SteamDetectionIssueCode, SteamDetectionStatus, SteamDetector};
use lxmi_steam::SteamScanner;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("lxmi-steam-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("temporary test directory should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_steam_config(root: &Path, content: &str) {
    let steamapps = root.join("steamapps");
    fs::create_dir_all(&steamapps).expect("steamapps fixture should be created");
    fs::write(steamapps.join("libraryfolders.vdf"), content)
        .expect("VDF fixture should be written");
}

#[test]
fn detects_default_and_registered_steam_libraries() {
    let temp = TestDirectory::new();
    let home = temp.path().join("home");
    let default_library = home.join(".local/share/Steam");
    let extra_library = temp.path().join("extra-library");
    fs::create_dir_all(default_library.join("steamapps")).expect("default library should exist");
    fs::create_dir_all(extra_library.join("steamapps")).expect("extra library should exist");

    let content = format!(
        r#""libraryfolders" {{ "0" {{ "path" "{}" }} "1" {{ "path" "{}" }} }}"#,
        default_library.display(),
        extra_library.display()
    );
    write_steam_config(&default_library, &content);

    let result = SteamScanner::with_paths(Some(home), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::Detected);
    assert_eq!(result.installations.len(), 1);
    let libraries = &result.installations[0].libraries;
    assert_eq!(libraries.len(), 2);
    assert!(libraries[0].is_default);
    assert_eq!(
        libraries[0].path,
        fs::canonicalize(&default_library).expect("canonical root")
    );
    assert!(!libraries[1].is_default);
    assert_eq!(
        libraries[1].path,
        fs::canonicalize(&extra_library).expect("canonical library")
    );
}

#[test]
fn reports_missing_steam_without_touching_other_paths() {
    let temp = TestDirectory::new();
    let result = SteamScanner::with_paths(Some(temp.path().join("empty-home")), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::NotInstalled);
    assert!(result.installations.is_empty());
    assert!(result.issues.is_empty());
}

#[test]
fn reports_missing_libraryfolders_configuration() {
    let temp = TestDirectory::new();
    let home = temp.path().join("home");
    fs::create_dir_all(home.join(".local/share/Steam/steamapps"))
        .expect("Steam directory should be created");

    let result = SteamScanner::with_paths(Some(home), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::ConfigurationMissing);
    assert!(result
        .issues
        .iter()
        .any(|issue| { issue.code == SteamDetectionIssueCode::ConfigurationMissing }));
}

#[test]
fn reports_invalid_vdf_and_keeps_a_specific_error_code() {
    let temp = TestDirectory::new();
    let home = temp.path().join("home");
    let root = home.join(".local/share/Steam");
    write_steam_config(&root, r#""libraryfolders" { "0" {"#);

    let result = SteamScanner::with_paths(Some(home), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::InvalidConfiguration);
    assert!(result
        .issues
        .iter()
        .any(|issue| { issue.code == SteamDetectionIssueCode::InvalidConfiguration }));
}

#[test]
fn ignores_missing_registered_libraries_but_reports_them() {
    let temp = TestDirectory::new();
    let home = temp.path().join("home");
    let root = home.join(".local/share/Steam");
    let missing_library = temp.path().join("does-not-exist");
    let content = format!(
        r#""libraryfolders" {{ "0" {{ "path" "{}" }} "1" {{ "path" "{}" }} }}"#,
        root.display(),
        missing_library.display()
    );
    write_steam_config(&root, &content);

    let result = SteamScanner::with_paths(Some(home), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::Detected);
    assert_eq!(result.installations[0].libraries.len(), 1);
    assert!(result.issues.iter().any(|issue| {
        issue.code == SteamDetectionIssueCode::LibraryMissing
            && issue.path.as_ref() == Some(&missing_library)
    }));
}

#[test]
fn reports_home_directory_unavailable() {
    let result = SteamScanner::with_paths(None, None).scan();
    assert_eq!(
        result.status,
        SteamDetectionStatus::HomeDirectoryUnavailable
    );
}

#[cfg(unix)]
#[test]
fn canonicalizes_symlinked_steam_roots_and_deduplicates_them() {
    use std::os::unix::fs::symlink;

    let temp = TestDirectory::new();
    let home = temp.path().join("home");
    let root = home.join(".local/share/Steam");
    fs::create_dir_all(root.join("steamapps")).expect("Steam root should exist");
    let content = format!(
        r#""libraryfolders" {{ "0" {{ "path" "{}" }} }}"#,
        root.display()
    );
    write_steam_config(&root, &content);
    fs::create_dir_all(home.join(".steam")).expect("Steam aliases directory should exist");
    symlink(&root, home.join(".steam/steam")).expect("Steam alias should be created");
    symlink(&root, home.join(".steam/root")).expect("second Steam alias should be created");

    let result = SteamScanner::with_paths(Some(home), None).scan();

    assert_eq!(result.status, SteamDetectionStatus::Detected);
    assert_eq!(result.installations.len(), 1);
}

#[test]
fn fixture_covers_steam_libraryfolder_formats() {
    let fixture = include_str!("fixtures/libraryfolders.vdf");
    let paths = lxmi_steam::parse_library_folders(fixture).expect("fixture should parse");
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0], PathBuf::from("/home/tester/.local/share/Steam"));
    assert_eq!(paths[1], PathBuf::from("/mnt/games/SteamLibrary"));
}
