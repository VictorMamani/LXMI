//! Native boundary: storage roots and protected paths are always backend-derived.
use base64::Engine;
use lxmi_steam::SteamDiscoveryScanner;
use lxmi_xxmi::{
    ErrorCode, ModDetails, ModLibrary, ModLibrarySnapshot, ModManifest, PreparedModProfile,
    PreviewView, Result, XxmiError,
};

fn library() -> Result<ModLibrary> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    Ok(ModLibrary::new(crate::xxmi_service::store_for(&discovery)?))
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|_| {
        XxmiError::new(
            ErrorCode::Io,
            None,
            "The mod operation ended unexpectedly. Refresh the library before retrying.",
        )
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

/// Kind and detected character of every mod; cheap enough to call on each refresh.
#[tauri::command]
pub async fn list_mod_details(game_id: String) -> Result<Vec<ModDetails>> {
    blocking(move || library()?.mod_details(&game_id)).await
}

/// Renders one mod; the interface queues mods one at a time so it can stop.
#[tauri::command]
pub async fn generate_mod_preview(game_id: String, mod_id: String) -> Result<ModDetails> {
    blocking(move || library()?.generate_mod_preview(&game_id, &mod_id)).await
}

/// A PNG data URL, ready for an `<img>` without widening the asset protocol scope.
#[tauri::command]
pub async fn get_mod_preview(game_id: String, mod_id: String, view: PreviewView) -> Result<String> {
    let png = blocking(move || library()?.mod_preview(&game_id, &mod_id, view)).await?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

#[tauri::command]
pub async fn clear_mod_previews(game_id: String) -> Result<()> {
    blocking(move || library()?.clear_mod_previews(&game_id)).await
}
