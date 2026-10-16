//! Native boundary: storage roots and protected paths are always backend-derived.
use lxmi_steam::SteamDiscoveryScanner;
use lxmi_xxmi::{
    ErrorCode, ModLibrary, ModLibrarySnapshot, ModManifest, PreparedModProfile, Result, XxmiError,
};

fn library() -> Result<ModLibrary> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    Ok(ModLibrary::new(crate::xxmi_service::store_for(&discovery)?))
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|_| {
        XxmiError::new(ErrorCode::Io, None, "La operación de mods terminó inesperadamente. Actualiza la biblioteca antes de repetirla.")
    })?
}

#[tauri::command]
pub async fn list_mod_library(game_id: String) -> Result<ModLibrarySnapshot> {
    blocking(move || library()?.list(&game_id)).await
}

#[tauri::command]
pub async fn import_local_mod(
    game_id: String,
    name: String,
    source: String,
) -> Result<ModManifest> {
    blocking(move || library()?.import(&game_id, &name, std::path::Path::new(&source))).await
}

#[tauri::command]
pub async fn save_mod_profile(
    game_id: String,
    name: String,
    mod_ids: Vec<String>,
    expected_revision: u64,
) -> Result<ModLibrarySnapshot> {
    blocking(move || library()?.save_profile(&game_id, &name, &mod_ids, expected_revision)).await
}

#[tauri::command]
pub async fn prepare_mod_profile(
    game_id: String,
    name: String,
    expected_revision: u64,
) -> Result<PreparedModProfile> {
    blocking(move || library()?.prepare_profile(&game_id, &name, expected_revision)).await
}
