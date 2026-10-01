use crate::runtime_observation::{RuntimeObservationController, RuntimeObservationError};
use lxmi_runtime::RuntimeObservationSnapshot;
use tauri::State;

#[tauri::command]
pub async fn start_zzz_runtime_observation(
    state: State<'_, RuntimeObservationController>,
    user_confirmed_manual_launch: bool,
) -> Result<RuntimeObservationSnapshot, RuntimeObservationError> {
    let controller = state.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        controller.start_zzz(user_confirmed_manual_launch)
    })
    .await
    .unwrap_or_else(|error| {
        Err(RuntimeObservationError {
            code: "observer_task_failed".into(),
            detail: format!("La tarea de observación terminó inesperadamente: {error}"),
        })
    });
    match &result {
        Ok(snapshot) => tracing::info!(
            session_id = snapshot.session_id,
            status = ?snapshot.status,
            active = snapshot.active,
            "Runtime observation snapshot returned to UI"
        ),
        Err(error) => tracing::warn!(
            code = %error.code,
            "Runtime observation start failed"
        ),
    }
    result
}

#[tauri::command]
pub fn get_zzz_runtime_observation(
    state: State<'_, RuntimeObservationController>,
) -> Option<RuntimeObservationSnapshot> {
    let snapshot = state.current();
    if let Some(snapshot) = &snapshot {
        tracing::debug!(
            session_id = snapshot.session_id,
            status = ?snapshot.status,
            active = snapshot.active,
            accepted_candidates = snapshot.scan_diagnostics.accepted_candidates,
            "Runtime observation snapshot returned to UI"
        );
    }
    snapshot
}

#[tauri::command]
pub fn stop_zzz_runtime_observation(
    state: State<'_, RuntimeObservationController>,
) -> Option<RuntimeObservationSnapshot> {
    let snapshot = state.stop();
    if let Some(snapshot) = &snapshot {
        tracing::info!(
            session_id = snapshot.session_id,
            status = ?snapshot.status,
            active = snapshot.active,
            "Runtime observation stop response returned to UI"
        );
    }
    snapshot
}
