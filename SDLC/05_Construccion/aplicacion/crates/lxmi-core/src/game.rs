use std::path::PathBuf;

/// Registry entry for a game LXMI can identify from a Steam app manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Game {
    pub id: &'static str,
    pub name: &'static str,
    pub steam_app_ids: &'static [u32],
}

pub const WUTHERING_WAVES_APP_ID: u32 = 3_513_350;

const WUTHERING_WAVES_APP_IDS: &[u32] = &[WUTHERING_WAVES_APP_ID];

pub const SUPPORTED_GAMES: &[Game] = &[Game {
    id: "wuthering-waves",
    name: "Wuthering Waves",
    steam_app_ids: WUTHERING_WAVES_APP_IDS,
}];

pub fn game_for_steam_app_id(app_id: u32) -> Option<Game> {
    SUPPORTED_GAMES
        .iter()
        .find(|game| game.steam_app_ids.contains(&app_id))
        .copied()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameInstallationStatus {
    Installed,
    DirectoryMissing,
    DirectoryInvalid,
    PermissionDenied,
}

/// A game installation independent of the Steam manifest that discovered it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInstallation {
    pub game: Game,
    pub install_path: PathBuf,
    pub status: GameInstallationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtonCompatDataStatus {
    NotFound,
    CompatDataFound,
    PrefixFound,
    Invalid,
    PermissionDenied,
    IoError,
}

/// Passive filesystem observations only; this does not prove Proton is active or healthy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtonCompatData {
    pub app_id: u32,
    pub compatdata_path: PathBuf,
    pub prefix_path: Option<PathBuf>,
    pub status: ProtonCompatDataStatus,
}

#[cfg(test)]
mod tests {
    use super::{game_for_steam_app_id, WUTHERING_WAVES_APP_ID};

    #[test]
    fn identifies_supported_game_by_steam_app_id() {
        let game = game_for_steam_app_id(WUTHERING_WAVES_APP_ID).expect("known game");
        assert_eq!(game.id, "wuthering-waves");
        assert_eq!(game.name, "Wuthering Waves");
    }

    #[test]
    fn leaves_unknown_steam_apps_unrecognized() {
        assert_eq!(game_for_steam_app_id(42), None);
    }
}
