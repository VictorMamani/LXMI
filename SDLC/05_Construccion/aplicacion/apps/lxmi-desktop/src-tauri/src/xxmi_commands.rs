use crate::runtime_observation::RuntimeObservationController;
use crate::xxmi_service::{
    self, BridgeServiceError, IntegrationStatus, LoaderLabPanel, RuntimeBridgePanel,
};
use lxmi_xxmi::{
    ErrorCode, InstallationPlan, LaunchTopologyPlan, ManagedRuntime, OfficialPackageKind,
    PackageManifest, Result, RuntimeAssemblyPlan, UpstreamRelease, XxmiError,
};

#[tauri::command]
pub async fn inspect_zzz_prefix_preflight(
    capture_baseline: bool,
) -> std::result::Result<crate::prefix_recovery_service::PrefixRecoveryPanel, BridgeServiceError> {
    blocking_bridge(move || crate::prefix_recovery_service::inspect(capture_baseline)).await
}

#[tauri::command]
pub async fn run_zzz_prefix_recovery(
    action: crate::prefix_recovery_service::RecoveryAction,
) -> std::result::Result<crate::prefix_recovery_service::PrefixRecoveryPanel, BridgeServiceError> {
    blocking_bridge(move || crate::prefix_recovery_service::run(action)).await
}

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
pub async fn inspect_zzz_launch_readiness(
    zzmi_id: Option<String>,
    libraries_id: Option<String>,
    state: tauri::State<'_, RuntimeObservationController>,
) -> std::result::Result<lxmi_xxmi::ZzmiLaunchReadiness, BridgeServiceError> {
    let observation = state.current();
    blocking_bridge(move || {
        xxmi_service::inspect_zzz_launch_readiness(
            zzmi_id.as_deref(),
            libraries_id.as_deref(),
            observation.as_ref(),
        )
    })
    .await
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

#[tauri::command]
pub async fn inspect_loader_lab() -> std::result::Result<LoaderLabPanel, BridgeServiceError> {
    blocking_bridge(xxmi_service::inspect_loader_lab).await
}

#[tauri::command]
pub async fn prepare_loader_lab(
) -> std::result::Result<lxmi_bridge::LoaderLabStatus, BridgeServiceError> {
    blocking_bridge(xxmi_service::prepare_loader_lab).await
}

#[tauri::command]
pub async fn run_loader_lab_experiment(
    proton_script: String,
    mode: lxmi_bridge::LoaderExperimentMode,
    side_effects_acknowledged: bool,
) -> std::result::Result<lxmi_bridge::LoaderExperimentResult, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::run_loader_lab_experiment(&proton_script, mode, side_effects_acknowledged)
    })
    .await
}

#[tauri::command]
pub async fn run_context_lab_test(
    proton_script: String,
    mode: lxmi_bridge::ContextExperimentMode,
    side_effects_acknowledged: bool,
) -> std::result::Result<lxmi_bridge::ContextLabResult, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::run_context_lab_test(&proton_script, mode, side_effects_acknowledged)
    })
    .await
}

#[tauri::command]
pub async fn run_direct_context_test(
    proton_script: String,
    mode: lxmi_bridge::ContextExperimentMode,
    side_effects_acknowledged: bool,
) -> std::result::Result<lxmi_bridge::DirectContextResult, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::run_direct_context_test(&proton_script, mode, side_effects_acknowledged)
    })
    .await
}

#[tauri::command]
pub async fn run_context_reentry_test(
    proton_script: String,
    mode: lxmi_bridge::ContextExperimentMode,
    side_effects_acknowledged: bool,
) -> std::result::Result<lxmi_bridge::ContextReentryResult, BridgeServiceError> {
    blocking_bridge(move || {
        xxmi_service::run_context_reentry_test(&proton_script, mode, side_effects_acknowledged)
    })
    .await
}

#[tauri::command]
pub async fn prepare_zzz_harmless_prefix_entry(
) -> std::result::Result<crate::prefix_recovery_service::HarmlessEntryReviewPanel, BridgeServiceError>
{
    blocking_bridge(crate::prefix_recovery_service::prepare_entry).await
}
#[tauri::command]
pub async fn run_zzz_harmless_prefix_entry(
    review_token: String,
    consent_accepted: bool,
) -> std::result::Result<lxmi_bridge::RealPrefixEntryResult, BridgeServiceError> {
    blocking_bridge(move || {
        crate::prefix_recovery_service::run_entry(review_token, consent_accepted)
    })
    .await
}
