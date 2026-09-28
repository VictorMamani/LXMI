mod commands;
mod xxmi_commands;
mod xxmi_service;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::scan_steam,
            xxmi_commands::xxmi_status,
            xxmi_commands::xxmi_check_official_releases,
            xxmi_commands::xxmi_download_official_package,
            xxmi_commands::import_xxmi_directory,
            xxmi_commands::review_xxmi_plan,
            xxmi_commands::prepare_zzmi_runtime,
            xxmi_commands::review_zzmi_assembly,
            xxmi_commands::inspect_zzmi_launch_topology,
            xxmi_commands::inspect_runtime_bridge,
            xxmi_commands::run_runtime_bridge_test
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
