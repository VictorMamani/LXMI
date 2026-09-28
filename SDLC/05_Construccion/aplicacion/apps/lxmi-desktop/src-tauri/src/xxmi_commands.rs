use crate::xxmi_service::{self, BridgeServiceError, IntegrationStatus, RuntimeBridgePanel};
use lxmi_xxmi::{
    ErrorCode, InstallationPlan, LaunchTopologyPlan, ManagedRuntime, OfficialPackageKind,
    PackageManifest, Result, RuntimeAssemblyPlan, UpstreamRelease, XxmiError,
};

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| {
        XxmiError::new(
            ErrorCode::Io,
            None,
            format!("La operación terminó inesperadamente: {e}"),
        )
    })?
}

async fn blocking_bridge<T: Send + 'static>(
    f: impl FnOnce() -> std::result::Result<T, BridgeServiceError> + Send + 'static,
) -> std::result::Result<T, BridgeServiceError> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|error| BridgeServiceError {
            code: "bridge_task_failed".into(),
            detail: format!("La tarea del bridge terminó inesperadamente: {error}"),
            stderr: None,
        })?
}
#[tauri::command]
pub async fn xxmi_status() -> Result<IntegrationStatus> {
    blocking(xxmi_service::status).await
}
#[tauri::command]
pub async fn xxmi_check_official_releases() -> Result<Vec<UpstreamRelease>> {
    blocking(xxmi_service::check_official_releases).await
}
#[tauri::command]
pub async fn xxmi_download_official_package(
    package_kind: OfficialPackageKind,
    tag: String,
) -> Result<PackageManifest> {
    blocking(move || xxmi_service::download_official_package(package_kind, &tag)).await
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

#[tauri::command]
pub async fn prepare_zzmi_runtime(zzmi_id: String, libraries_id: String) -> Result<ManagedRuntime> {
    blocking(move || xxmi_service::prepare_zzmi_runtime(&zzmi_id, &libraries_id)).await
}

#[tauri::command]
pub async fn review_zzmi_assembly(
    zzmi_id: String,
    libraries_id: String,
) -> Result<RuntimeAssemblyPlan> {
    blocking(move || xxmi_service::review_zzmi_assembly(&zzmi_id, &libraries_id)).await
}

#[tauri::command]
pub async fn inspect_zzmi_launch_topology(
    zzmi_id: String,
    libraries_id: String,
) -> Result<LaunchTopologyPlan> {
    blocking(move || xxmi_service::inspect_zzmi_launch_topology(&zzmi_id, &libraries_id)).await
}

#[tauri::command]
pub async fn inspect_runtime_bridge(
    zzmi_id: Option<String>,
    libraries_id: Option<String>,
) -> std::result::Result<RuntimeBridgePanel, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::inspect_runtime_bridge(zzmi_id.as_deref(), libraries_id.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn run_runtime_bridge_test(
    zzmi_id: String,
    libraries_id: String,
    proton_script: String,
) -> std::result::Result<lxmi_bridge::BridgeTestResult, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::run_runtime_bridge_test(&zzmi_id, &libraries_id, &proton_script)
    })
    .await
}
