mod compatdata;
mod discovery;
mod game_scanner;
mod manifest;
mod scanner;
mod vdf;

pub use discovery::{
    SteamDiscoveryResult, SteamDiscoveryScanner, SteamGameDiscoveryResult,
    SteamGameDiscoveryStatus, SteamGameInstallation,
};
pub use game_scanner::{
    find_expected_game_executable, GameExecutableDiscovery, SteamAppInstallStatus,
    SteamAppInstallation, SteamAppScanIssue, SteamAppScanIssueCode, SteamAppScanIssueSeverity,
    SteamAppScanResult, SteamAppScanStatus, SteamAppScanner,
};
pub use manifest::{parse_app_manifest, SteamAppManifest, SteamManifestError};
pub use scanner::SteamScanner;
pub use vdf::{parse_key_values_document, parse_library_folders, KeyValuesValue, VdfParseError};
