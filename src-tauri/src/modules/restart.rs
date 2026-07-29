use std::sync::Arc;

use reqwest::Method;
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn restart_ux(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .lcu_checked(Method::POST, "/riotclient/kill-and-restart-ux", None)
        .await?;
    log(&app, "Client UX restarting…");
    Ok(())
}
