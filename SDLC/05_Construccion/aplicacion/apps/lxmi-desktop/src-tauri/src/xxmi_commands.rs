use crate::xxmi_service::{self, IntegrationStatus};
use lxmi_xxmi::{ErrorCode, InstallationPlan, PackageManifest, Result, XxmiError};

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| {
        XxmiError::new(
            ErrorCode::Io,
            None,
            format!("La operación terminó inesperadamente: {e}"),
        )
    })?
}
#[tauri::command]
pub async fn xxmi_status() -> Result<IntegrationStatus> {
    blocking(xxmi_service::status).await
}
#[tauri::command]
pub async fn import_xxmi_directory(path: String) -> Result<PackageManifest> {
    blocking(move || xxmi_service::import_directory(std::path::Path::new(&path))).await
}
#[tauri::command]
pub async fn review_xxmi_plan(
    game_id: String,
    package_id: String,
    libraries_id: Option<String>,
) -> Result<InstallationPlan> {
    blocking(move || xxmi_service::plan(&game_id, &package_id, libraries_id.as_deref())).await
}
