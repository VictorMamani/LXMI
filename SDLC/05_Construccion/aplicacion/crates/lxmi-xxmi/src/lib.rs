//! Offline XXMI package inspection, managed staging and declarative planning.
//! No process execution and no installation executor are provided.
mod filesystem;
mod model;
mod package;
mod planning;
mod store;

pub use filesystem::validate_relative_path;
pub use model::*;
pub use package::{inspect_runtime, validate_directory, WWMI_REQUIRED_FILES, ZZMI_REQUIRED_FILES};
pub use planning::{
    assess, assess_for_integration, integration_for_game, plan_installation,
    plan_installation_for_integration,
};
pub use store::{ManagedStore, VerifiedPackage};
