//! Launch a Steam game with the selected mod profile through the official XXMI loader.
//!
//! Two strategies, chosen per game by the caller (`LoadMode`):
//! - `Hook` (ZZZ, validated on Steam + Proton 2026-10-15): the helper enters the wineserver
//!   of the launcher Steam started, installs `HookLibrary` before the game exists and
//!   removes it once the module is seen in the new game process.
//! - `DirectInject` (WWMI, experimental): upstream's default since v2.2.0. The helper waits
//!   for the exact game image and calls `Inject` on its PID. Not validated in game.
//!
//! `preflight` runs static checks before anything starts. Nothing is written to Steam,
//! the game installation or its prefix.
mod config;
mod context;
mod events;
mod host;
mod preflight;
mod session;
mod workdir;

pub use config::{configure_loader, ConfigError};
pub use context::{find_steam_process, ProcessQuery, SteamProcessContext};
pub use events::HelperEvent;
pub use host::{LoadMode, ModLaunchTarget, SteamModLaunchHost, IMPORTER_FOLDERS};
pub use preflight::{preflight, PreflightInput, ALLOWED_LAUNCH_ARGUMENTS};
pub use session::{run_launch, HelperProcess, HelperRead, LaunchHost, LaunchPhase, LaunchStatus};
pub use workdir::{prepare_launch_directory, PreparedLaunch, HELPER_NAME};
