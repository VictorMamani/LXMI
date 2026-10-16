//! Runtime/profile orchestration. Paths and AppIDs are derived exclusively in Rust.
use lxmi_core::{GameExecutableStatus, GameInstallationStatus};
use lxmi_steam::{SteamDiscoveryResult, SteamDiscoveryScanner};
use lxmi_xxmi::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct GameModRuntimeStatus {
    pub game_id: String,
    pub installed: bool,
    pub importer_packages: Vec<PackageManifest>,
    pub libraries_packages: Vec<PackageManifest>,
    pub base_runtime: Option<ManagedRuntime>,
    pub selected_runtime: Option<ManagedRuntime>,
    pub selection: ActiveProfileSelection,
    pub launch_with_mods_available: bool,
    pub launch_blockers: Vec<String>,
}

fn context() -> Result<(SteamDiscoveryResult, ManagedStore)> {
    let discovery = SteamDiscoveryScanner::from_environment().scan();
    let store = crate::xxmi_service::store_for(&discovery)?;
    Ok((discovery, store))
}

fn base_runtime(
    discovery: &SteamDiscoveryResult,
    store: &ManagedStore,
    game: &str,
    importer: &str,
    libraries: &str,
) -> Result<ManagedRuntime> {
    let platform = crate::xxmi_service::platform_plan(discovery, game)?;
    existing_importer_runtime(
        store,
        &platform,
        importer,
        libraries,
        &crate::xxmi_service::protected_paths(discovery),
    )?
    .ok_or_else(|| {
        XxmiError::new(
            ErrorCode::NotFound,
            None,
            "Prepara primero el runtime de este juego.",
        )
    })
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|_| {
        XxmiError::new(
            ErrorCode::Io,
            None,
            "La tarea del runtime terminó inesperadamente.",
        )
    })?
}

#[tauri::command]
pub async fn inspect_game_mod_runtime(
    game_id: String,
    importer_id: Option<String>,
    libraries_id: Option<String>,
) -> Result<GameModRuntimeStatus> {
    blocking(move || {
        let kind = integration_for_game(&game_id).ok_or_else(|| {
            XxmiError::new(ErrorCode::WrongGameIntegration, None, "Juego desconocido.")
        })?;
        let (discovery, store) = context()?;
        let packages = store.list()?;
        let installed = discovery.games.games.iter().any(|g| {
            g.installation.game.id == game_id
                && g.installation.status == GameInstallationStatus::Installed
                && g.installation.executable_status == GameExecutableStatus::Found
        });
        let base = match (importer_id, libraries_id) {
            (Some(importer), Some(libraries))
                if installed && !importer.is_empty() && !libraries.is_empty() =>
            {
                let platform = crate::xxmi_service::platform_plan(&discovery, &game_id)?;
                existing_importer_runtime(
                    &store,
                    &platform,
                    &importer,
                    &libraries,
                    &crate::xxmi_service::protected_paths(&discovery),
                )?
            }
            _ => None,
        };
        let selected_runtime = base
            .as_ref()
            .map(|base| resolve_profile_runtime(&store, base))
            .transpose()?;
        let selection = profile_activation_status(&store, &game_id)?;
        let launch_blockers = crate::mod_launch::launch_blockers(&game_id, &selection);
        Ok(GameModRuntimeStatus {
            game_id: game_id.clone(),
            installed,
            importer_packages: packages
                .iter()
                .filter(|p| p.kind == PackageKind::GameIntegration(kind))
                .cloned()
                .collect(),
            libraries_packages: packages
                .iter()
                .filter(|p| p.kind == PackageKind::XxmiLibraries)
                .cloned()
                .collect(),
            base_runtime: base,
            selected_runtime,
            launch_with_mods_available: installed && launch_blockers.is_empty(),
            launch_blockers,
            selection,
        })
    })
    .await
}

#[tauri::command]
pub async fn prepare_game_mod_runtime(
    game_id: String,
    importer_id: String,
    libraries_id: String,
) -> Result<ManagedRuntime> {
    blocking(move || {
        let (discovery, store) = context()?;
        let platform = crate::xxmi_service::platform_plan(&discovery, &game_id)?;
        assemble_importer_runtime(
            &store,
            &platform,
            &importer_id,
            &libraries_id,
            &crate::xxmi_service::protected_paths(&discovery),
        )
    })
    .await
}

#[tauri::command]
pub async fn review_mod_profile_activation(
    game_id: String,
    importer_id: String,
    libraries_id: String,
    profile_name: String,
    profile_revision: u64,
    selection_revision: u64,
) -> Result<ActivationPlan> {
    blocking(move || {
        let (discovery, store) = context()?;
        let base = base_runtime(&discovery, &store, &game_id, &importer_id, &libraries_id)?;
        review_profile_activation(
            &store,
            &base,
            &profile_name,
            profile_revision,
            selection_revision,
        )
    })
    .await
}

#[tauri::command]
pub async fn activate_mod_profile(
    game_id: String,
    importer_id: String,
    libraries_id: String,
    profile_name: String,
    profile_revision: u64,
    selection_revision: u64,
    plan_id: String,
) -> Result<ActiveProfileSelection> {
    blocking(move || {
        let (discovery, store) = context()?;
        let base = base_runtime(&discovery, &store, &game_id, &importer_id, &libraries_id)?;
        activate_profile(
            &store,
            &base,
            &profile_name,
            profile_revision,
            selection_revision,
            &plan_id,
        )
    })
    .await
}

#[tauri::command]
pub async fn rollback_mod_profile_activation(
    game_id: String,
    selection_revision: u64,
) -> Result<ActiveProfileSelection> {
    blocking(move || {
        let (_, store) = context()?;
        rollback_profile_activation(&store, &game_id, selection_revision)
    })
    .await
}

#[derive(Serialize)]
pub struct SteamLaunchRequest {
    game_id: String,
    steam_app_id: u32,
    request_sent: bool,
    mods_loaded: bool,
}

#[tauri::command]
pub async fn launch_game_without_mods(game_id: String) -> Result<SteamLaunchRequest> {
    blocking(move || {
        let (discovery, _) = context()?;
        let installation = discovery
            .games
            .games
            .iter()
            .find(|g| {
                g.installation.game.id == game_id
                    && g.installation.status == GameInstallationStatus::Installed
                    && g.installation.executable_status == GameExecutableStatus::Found
            })
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::NotFound,
                    None,
                    "No se encontró una instalación lista en Steam.",
                )
            })?;
        // Fixed registry AppID only, direct argv, no shell/URI supplied by the webview.
        let app_id = installation.steam_app_id;
        let mut command = std::process::Command::new("steam");
        command.arg("-applaunch").arg(app_id.to_string());
        if game_id == "wuthering-waves" {
            command.arg("-dx11");
        }
        let mut child = command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                XxmiError::new(
                    ErrorCode::Io,
                    None,
                    format!("No se pudo enviar la solicitud a Steam: {e}"),
                )
            })?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(SteamLaunchRequest {
            game_id,
            steam_app_id: app_id,
            request_sent: true,
            mods_loaded: false,
        })
    })
    .await
}
