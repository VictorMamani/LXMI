mod commands;
mod prefix_recovery_service;
mod runtime_observation;
mod runtime_observation_commands;
mod xxmi_commands;
mod xxmi_service;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tauri::Builder::default()
        .manage(runtime_observation::RuntimeObservationController::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::scan_steam,
            xxmi_commands::xxmi_status,
            xxmi_commands::inspect_zzz_prefix_preflight,
            xxmi_commands::run_zzz_prefix_recovery,
            xxmi_commands::inspect_linux_environment_qualification,
            xxmi_commands::prepare_zzz_harmless_prefix_entry,
            xxmi_commands::run_zzz_harmless_prefix_entry,
            xxmi_commands::xxmi_check_official_releases,
            xxmi_commands::xxmi_download_official_package,
            xxmi_commands::import_xxmi_directory,
            xxmi_commands::review_xxmi_plan,
            xxmi_commands::prepare_zzmi_runtime,
            xxmi_commands::review_zzmi_assembly,
            xxmi_commands::inspect_zzmi_launch_topology,
            xxmi_commands::inspect_zzz_launch_readiness,
            xxmi_commands::inspect_runtime_bridge,
            xxmi_commands::run_runtime_bridge_test,
            xxmi_commands::run_context_lab_test,
            xxmi_commands::run_direct_context_test,
            xxmi_commands::run_context_reentry_test,
            xxmi_commands::inspect_loader_lab,
            xxmi_commands::prepare_loader_lab,
            xxmi_commands::run_loader_lab_experiment,
            runtime_observation_commands::start_zzz_runtime_observation,
            runtime_observation_commands::get_zzz_runtime_observation,
            runtime_observation_commands::stop_zzz_runtime_observation
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
