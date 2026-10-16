//! Launch a Steam game with the selected mod profile through the official XXMI hook.
//!
//! Validated for ZZZ on Steam + Proton (2026-10-15): the helper enters the wineserver
//! of the launcher Steam started, installs `HookLibrary` before the game exists and
//! removes it as soon as the module is seen in the new game process. ZZMI's DllMain
//! rejects every other process. Nothing is written to Steam, the game or its prefix.
mod config;
mod context;
mod events;
mod host;
mod session;
mod workdir;

pub use config::{configure_loader, ConfigError};
pub use context::{find_steam_process, ProcessQuery, SteamProcessContext};
pub use events::HelperEvent;
pub use host::{ModLaunchTarget, SteamModLaunchHost};
pub use session::{run_launch, HelperProcess, HelperRead, LaunchHost, LaunchPhase, LaunchStatus};
pub use workdir::{prepare_launch_directory, PreparedLaunch, HELPER_NAME};
