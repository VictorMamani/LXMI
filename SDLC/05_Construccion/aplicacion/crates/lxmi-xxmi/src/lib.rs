//! Offline XXMI package inspection, managed staging and declarative planning.
//! No process execution and no installation executor are provided.
mod archive;
mod cache;
mod crypto;
mod filesystem;
mod mapping;
mod model;
mod package;
mod planning;
mod release;
mod store;

pub use cache::{CachedAsset, DownloadCache};
pub use crypto::{
    verify_component_signature, verify_file as verify_signature_file, verify_release_file,
    verify_release_signature,
};
pub use filesystem::validate_relative_path;
pub use model::*;
pub use package::{inspect_runtime, validate_directory, WWMI_REQUIRED_FILES, ZZMI_REQUIRED_FILES};
pub use planning::{
    assess, assess_for_integration, integration_for_game, plan_installation,
    plan_installation_for_integration,
};
pub use release::{
    asset_for_release, download_and_import_official, parse_github_release,
    repository as official_repository, FixtureReleaseProvider, GitHubReleaseProvider,
    ReleaseProvider,
};
pub use store::{ManagedStore, VerifiedPackage};
