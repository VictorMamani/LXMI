use std::collections::HashSet;
use std::path::PathBuf;

use lxmi_core::{
    game_for_steam_app_id, GameDistribution, GameInstallation, GameInstallationStatus,
    ProtonCompatData, SteamDetectionStatus, SteamDetector, SteamLibrary, SteamScanResult,
};
use tracing::{debug, info};

use crate::compatdata::inspect_compatdata;
use crate::game_scanner::{
    find_expected_game_executable, SteamAppInstallation, SteamAppScanIssue, SteamAppScanStatus,
    SteamAppScanner,
};
use crate::SteamScanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamGameDiscoveryStatus {
    NotAvailable,
    Complete,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamGameInstallation {
    pub installation: GameInstallation,
    pub steam_app_id: u32,
    pub manifest_name: String,
    pub steam_library: PathBuf,
    pub compatdata: ProtonCompatData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamGameDiscoveryResult {
    pub status: SteamGameDiscoveryStatus,
    pub manifests_parsed: usize,
    pub ignored_unknown_apps: usize,
    pub games: Vec<SteamGameInstallation>,
    pub issues: Vec<SteamAppScanIssue>,
}

impl SteamGameDiscoveryResult {
    fn unavailable() -> Self {
        Self {
            status: SteamGameDiscoveryStatus::NotAvailable,
            manifests_parsed: 0,
            ignored_unknown_apps: 0,
            games: Vec::new(),
            issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamDiscoveryResult {
    pub steam: SteamScanResult,
    pub games: SteamGameDiscoveryResult,
}

pub struct SteamDiscoveryScanner {
    steam_scanner: SteamScanner,
    app_scanner: SteamAppScanner,
}

impl SteamDiscoveryScanner {
    pub fn from_environment() -> Self {
        Self {
            steam_scanner: SteamScanner::from_environment(),
            app_scanner: SteamAppScanner,
        }
    }

    pub fn with_paths(home_directory: Option<PathBuf>, xdg_data_home: Option<PathBuf>) -> Self {
        Self {
            steam_scanner: SteamScanner::with_paths(home_directory, xdg_data_home),
            app_scanner: SteamAppScanner,
        }
    }

    pub fn scan(&self) -> SteamDiscoveryResult {
        let steam = self.steam_scanner.scan();
        let games = self.scan_games(&steam);
        SteamDiscoveryResult { steam, games }
    }

    fn scan_games(&self, steam: &SteamScanResult) -> SteamGameDiscoveryResult {
        if steam.status != SteamDetectionStatus::Detected {
            return SteamGameDiscoveryResult::unavailable();
        }

        let mut libraries = Vec::<SteamLibrary>::new();
        let mut seen_libraries = HashSet::new();
        for installation in &steam.installations {
            for library in &installation.libraries {
                if seen_libraries.insert(library.path.clone()) {
                    libraries.push(library.clone());
                }
            }
        }

        let app_scan = self.app_scanner.scan(&libraries);
        let mut games = Vec::new();
        let mut issues = app_scan.issues;
        let mut ignored_unknown_apps = 0;

        for app in app_scan.apps {
            let Some(game) = game_for_steam_app_id(app.manifest.app_id) else {
                ignored_unknown_apps += 1;
                debug!(
                    app_id = app.manifest.app_id,
                    app_name = %app.manifest.name,
                    "Unknown Steam app ignored"
                );
                continue;
            };

            info!(
                app_id = app.manifest.app_id,
                game = game.id,
                "Known game detected"
            );
            let (compatdata, compatdata_issue) =
                inspect_compatdata(&app.library_path, app.manifest.app_id);
            if let Some(issue) = compatdata_issue {
                issues.push(issue);
            }
            games.push(to_game_installation(app, game, compatdata));
        }

        games.sort_by(|left, right| {
            left.steam_app_id
                .cmp(&right.steam_app_id)
                .then_with(|| left.steam_library.cmp(&right.steam_library))
        });
        let status = if issues.is_empty() {
            match app_scan.status {
                SteamAppScanStatus::Complete => SteamGameDiscoveryStatus::Complete,
                SteamAppScanStatus::Partial => SteamGameDiscoveryStatus::Partial,
            }
        } else {
            SteamGameDiscoveryStatus::Partial
        };

        SteamGameDiscoveryResult {
            status,
            manifests_parsed: app_scan.manifests_parsed,
            ignored_unknown_apps,
            games,
            issues,
        }
    }
}

fn to_game_installation(
    app: SteamAppInstallation,
    game: lxmi_core::Game,
    compatdata: ProtonCompatData,
) -> SteamGameInstallation {
    let status = match app.status {
        crate::SteamAppInstallStatus::Installed => GameInstallationStatus::Installed,
        crate::SteamAppInstallStatus::DirectoryMissing => GameInstallationStatus::DirectoryMissing,
        crate::SteamAppInstallStatus::DirectoryInvalid => GameInstallationStatus::DirectoryInvalid,
        crate::SteamAppInstallStatus::PermissionDenied => GameInstallationStatus::PermissionDenied,
    };
    let executable = find_expected_game_executable(&app.game_path, game.expected_executable_names);
    SteamGameInstallation {
        installation: GameInstallation {
            game,
            install_path: app.game_path,
            status,
            distribution: GameDistribution::Steam,
            executable_path: executable.path,
            executable_status: executable.status,
        },
        steam_app_id: app.manifest.app_id,
        manifest_name: app.manifest.name,
        steam_library: app.library_path,
        compatdata,
    }
}

#[cfg(test)]
mod tests {
    use super::{SteamDiscoveryScanner, SteamGameDiscoveryStatus};
    use crate::game_scanner::SteamAppScanIssueCode;
    use lxmi_core::GameInstallationStatus;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    const WUWA_APP_ID: u32 = 3_513_350;
    const ZZZ_APP_ID: u32 = 4_162_040;

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("lxmi-discovery-test-{}-{id}", std::process::id()));
            fs::create_dir_all(&root).expect("temporary fixture root should be created");
            Self(root)
        }

        fn home(&self) -> PathBuf {
            self.0.join("home")
        }

        fn steam_library(&self) -> PathBuf {
            self.home().join(".local/share/Steam")
        }

        fn configure_steam(&self, extra_libraries: &[PathBuf]) {
            let library = self.steam_library();
            let steamapps = library.join("steamapps");
            fs::create_dir_all(&steamapps).expect("steamapps fixture should be created");
            let entries = std::iter::once(library.clone())
                .chain(extra_libraries.iter().cloned())
                .enumerate()
                .map(|(index, path)| format!("\"{index}\" {{ \"path\" \"{}\" }}", path.display()))
                .collect::<Vec<_>>()
                .join(" ");
            fs::write(
                steamapps.join("libraryfolders.vdf"),
                format!("\"libraryfolders\" {{ {entries} }}"),
            )
            .expect("synthetic library configuration should be written");
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn manifest(app_id: u32, name: &str, install_dir: &str) -> String {
        format!(
            r#""AppState" {{ "appid" "{app_id}" "name" "{name}" "installdir" "{install_dir}" }}"#
        )
    }

    fn scanner(tree: &TempTree) -> SteamDiscoveryScanner {
        SteamDiscoveryScanner::with_paths(Some(tree.home()), None)
    }

    #[test]
    fn discovers_wuthering_waves_and_passive_compatdata_prefix() {
        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        fs::create_dir_all(library.join("steamapps/common/Wuthering Waves"))
            .expect("synthetic game directory should be created");
        fs::write(
            library.join(format!("steamapps/appmanifest_{WUWA_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/valid.acf"),
        )
        .expect("synthetic manifest should be written");
        let prefix = library
            .join("steamapps/compatdata")
            .join(WUWA_APP_ID.to_string())
            .join("pfx");
        fs::create_dir_all(&prefix).expect("synthetic prefix candidate should be created");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.status, SteamGameDiscoveryStatus::Complete);
        assert_eq!(result.games.manifests_parsed, 1);
        assert_eq!(result.games.games.len(), 1);
        let found = &result.games.games[0];
        assert_eq!(found.steam_app_id, WUWA_APP_ID);
        assert_eq!(found.installation.game.name, "Wuthering Waves");
        assert_eq!(
            found.installation.install_path,
            library.join("steamapps/common/Wuthering Waves")
        );
        assert_eq!(found.compatdata.prefix_path, Some(prefix));
    }

    #[test]
    fn discovers_zenless_zone_zero_manifest_executable_and_compatdata() {
        use lxmi_core::{GameDistribution, GameExecutableStatus, ProtonCompatDataStatus};

        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        let install_path = library.join("steamapps/common/Zenless Zone Zero");
        let executable = install_path.join("games/ZenlessZoneZero Game/ZenlessZoneZero.exe");
        fs::create_dir_all(executable.parent().expect("exe parent"))
            .expect("synthetic ZZZ directory should be created");
        fs::write(&executable, b"synthetic executable placeholder")
            .expect("synthetic executable placeholder should be written");
        fs::write(
            library.join(format!("steamapps/appmanifest_{ZZZ_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/zenless_zone_zero.acf"),
        )
        .expect("synthetic ZZZ manifest should be written");
        let prefix = library
            .join("steamapps/compatdata")
            .join(ZZZ_APP_ID.to_string())
            .join("pfx");
        fs::create_dir_all(&prefix).expect("synthetic prefix candidate should be created");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.games.len(), 1);
        let found = &result.games.games[0];
        assert_eq!(found.installation.game.id, "zenless-zone-zero");
        assert_eq!(found.steam_app_id, ZZZ_APP_ID);
        assert_eq!(found.manifest_name, "Zenless Zone Zero");
        assert_eq!(found.installation.distribution, GameDistribution::Steam);
        assert_eq!(found.installation.install_path, install_path);
        assert_eq!(found.installation.executable_path, Some(executable));
        assert_eq!(
            found.installation.executable_status,
            GameExecutableStatus::Found
        );
        assert_eq!(found.compatdata.status, ProtonCompatDataStatus::PrefixFound);
    }

    #[test]
    fn zzz_game_directory_without_expected_executable_is_not_complete_evidence() {
        use lxmi_core::GameExecutableStatus;

        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        fs::create_dir_all(library.join("steamapps/common/Zenless Zone Zero"))
            .expect("synthetic install directory should be created");
        fs::write(
            library.join(format!("steamapps/appmanifest_{ZZZ_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/zenless_zone_zero.acf"),
        )
        .expect("synthetic ZZZ manifest should be written");

        let result = scanner(&tree).scan();
        let found = result.games.games.first().expect("ZZZ manifest recognized");

        assert_eq!(
            found.installation.executable_status,
            GameExecutableStatus::NotFound
        );
        assert!(found.installation.executable_path.is_none());
    }

    #[test]
    fn ignores_unknown_app_but_counts_manifest_and_finds_known_game_in_other_library() {
        let tree = TempTree::new();
        let primary = tree.steam_library();
        let extra = tree.0.join("library-extra");
        tree.configure_steam(std::slice::from_ref(&extra));
        fs::create_dir_all(primary.join("steamapps/common/Unknown Game"))
            .expect("unknown app directory should be created");
        fs::write(
            primary.join("steamapps/appmanifest_100.acf"),
            manifest(100, "Synthetic Unknown", "Unknown Game"),
        )
        .expect("unknown manifest should be written");
        fs::create_dir_all(extra.join("steamapps/common/Wuthering Waves"))
            .expect("Wuthering Waves folder should be created");
        fs::write(
            extra.join(format!("steamapps/appmanifest_{WUWA_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/valid.acf"),
        )
        .expect("Wuthering Waves manifest should be written");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.manifests_parsed, 2);
        assert_eq!(result.games.ignored_unknown_apps, 1);
        assert_eq!(result.games.games.len(), 1);
        assert_eq!(
            result.games.games[0].steam_library,
            fs::canonicalize(extra).expect("synthetic Steam library should resolve")
        );
    }

    #[test]
    fn continues_after_corrupt_manifest_and_reports_missing_game_folder() {
        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        fs::write(library.join("steamapps/appmanifest_1.acf"), "broken {")
            .expect("broken manifest should be written");
        fs::write(
            library.join(format!("steamapps/appmanifest_{WUWA_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/valid.acf"),
        )
        .expect("valid manifest should be written");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.games.len(), 1);
        assert_eq!(
            result.games.games[0].installation.status,
            GameInstallationStatus::DirectoryMissing
        );
        assert!(result
            .games
            .issues
            .iter()
            .any(|issue| { issue.code == SteamAppScanIssueCode::ManifestInvalid }));
        assert!(result
            .games
            .issues
            .iter()
            .any(|issue| { issue.code == SteamAppScanIssueCode::GameDirectoryMissing }));
        assert_eq!(result.games.status, SteamGameDiscoveryStatus::Partial);
    }

    #[test]
    fn reports_wuthering_waves_not_found_when_no_manifest_matches_registry() {
        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        fs::create_dir_all(library.join("steamapps/common/Other Game"))
            .expect("unknown game directory should be created");
        fs::write(
            library.join("steamapps/appmanifest_123.acf"),
            manifest(123, "Another Synthetic Game", "Other Game"),
        )
        .expect("unknown manifest should be written");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.status, SteamGameDiscoveryStatus::Complete);
        assert!(result.games.games.is_empty());
        assert_eq!(result.games.ignored_unknown_apps, 1);
    }

    #[test]
    fn reports_complete_scan_for_a_library_without_manifests() {
        let tree = TempTree::new();
        tree.configure_steam(&[]);

        let result = scanner(&tree).scan();

        assert_eq!(result.games.status, SteamGameDiscoveryStatus::Complete);
        assert_eq!(result.games.manifests_parsed, 0);
        assert!(result.games.games.is_empty());
        assert!(result.games.issues.is_empty());
    }

    #[test]
    fn rejects_manifest_when_filename_and_payload_app_ids_differ() {
        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        fs::write(
            library.join("steamapps/appmanifest_123.acf"),
            manifest(WUWA_APP_ID, "Wuthering Waves", "Wuthering Waves"),
        )
        .expect("synthetic manifest should be written");

        let result = scanner(&tree).scan();

        assert!(result.games.games.is_empty());
        assert_eq!(result.games.manifests_parsed, 0);
        assert!(result
            .games
            .issues
            .iter()
            .any(|issue| { issue.code == SteamAppScanIssueCode::ManifestIdMismatch }));
    }

    #[cfg(unix)]
    #[test]
    fn ignores_symlinked_app_manifests_without_reading_the_target() {
        use std::os::unix::fs::symlink;

        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        let outside_manifest = tree.0.join("outside.acf");
        fs::write(
            &outside_manifest,
            include_str!("../tests/fixtures/appmanifests/valid.acf"),
        )
        .expect("outside synthetic manifest should be written");
        symlink(
            &outside_manifest,
            library.join(format!("steamapps/appmanifest_{WUWA_APP_ID}.acf")),
        )
        .expect("manifest symlink should be created");

        let result = scanner(&tree).scan();

        assert!(result.games.games.is_empty());
        assert!(result
            .games
            .issues
            .iter()
            .any(|issue| { issue.code == SteamAppScanIssueCode::ManifestNotRegularFile }));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_symlinked_common_directory() {
        use std::os::unix::fs::symlink;

        let tree = TempTree::new();
        let library = tree.steam_library();
        tree.configure_steam(&[]);
        let steamapps = library.join("steamapps");
        let outside = tree.0.join("outside-common");
        fs::create_dir_all(outside.join("Wuthering Waves"))
            .expect("outside game fixture should be created");
        symlink(&outside, steamapps.join("common"))
            .expect("synthetic common symlink should be created");
        fs::write(
            steamapps.join(format!("appmanifest_{WUWA_APP_ID}.acf")),
            include_str!("../tests/fixtures/appmanifests/valid.acf"),
        )
        .expect("synthetic manifest should be written");

        let result = scanner(&tree).scan();

        assert_eq!(result.games.games.len(), 1);
        assert_eq!(
            result.games.games[0].installation.status,
            GameInstallationStatus::DirectoryInvalid
        );
        assert!(!result.games.issues.is_empty());
    }

    #[test]
    fn does_not_scan_games_when_steam_is_not_detected() {
        let tree = TempTree::new();
        let result = scanner(&tree).scan();
        assert_eq!(result.games.status, SteamGameDiscoveryStatus::NotAvailable);
        assert!(result.games.games.is_empty());
    }
}
