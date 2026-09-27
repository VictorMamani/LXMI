mod commands;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::scan_steam
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
