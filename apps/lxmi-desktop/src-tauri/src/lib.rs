mod commands;
mod game_runtime_commands;
mod mod_commands;
mod mod_launch;
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

    desktop_builder().run(tauri::generate_context!())?;

    Ok(())
}

/// Development smoke: prepares a «Play with mods» launch copy for the saved profile
/// without starting Steam, Proton or the game.
pub fn prepare_mod_launch_copy(game_id: &str) -> Result<lxmi_launch::PreparedLaunch, String> {
    mod_launch::prepare_copy(game_id).map_err(|e| e.detail)
}

/// Shared native configuration, including the real command handler.
/// Development smoke examples attach their own lifecycle hooks to this builder.
pub fn desktop_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(runtime_observation::RuntimeObservationController::default())
        .manage(mod_launch::ModLaunchController::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::scan_steam,
            game_runtime_commands::inspect_game_mod_runtime,
            game_runtime_commands::prepare_game_mod_runtime,
            game_runtime_commands::review_mod_profile_activation,
            game_runtime_commands::activate_mod_profile,
            game_runtime_commands::rollback_mod_profile_activation,
            game_runtime_commands::launch_game_without_mods,
            mod_launch::start_game_with_mods,
            mod_launch::game_with_mods_status,
            mod_launch::cancel_game_with_mods,
            mod_commands::list_mod_library,
            mod_commands::import_local_mod,
            mod_commands::save_mod_profile,
            mod_commands::prepare_mod_profile,
            mod_commands::list_mod_details,
            mod_commands::generate_mod_preview,
            mod_commands::get_mod_preview,
            mod_commands::clear_mod_previews,
            xxmi_commands::xxmi_status,
            xxmi_commands::inspect_zzz_prefix_preflight,
            xxmi_commands::run_zzz_prefix_recovery,
            xxmi_commands::reconcile_zzz_prefix_recovery,
            xxmi_commands::review_zzz_steam_protected_state,
            xxmi_commands::inspect_linux_environment_qualification,
            xxmi_commands::qualify_zzz_prefix_executor,
            xxmi_commands::prepare_zzz_harmless_prefix_entry,
            xxmi_commands::create_zzz_harmless_prefix_entry_experiment,
            xxmi_commands::inspect_zzz_harmless_prefix_entry_history,
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
}
