use std::sync::Arc;

use reqwest::Method;
use serde_json::json;
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn set_status_message(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    message: String,
) -> Result<(), String> {
    state
        .lcu_checked(
            Method::PUT,
            "/lol-chat/v1/me",
            Some(json!({ "statusMessage": message })),
        )
        .await?;
    log(
        &app,
        if message.is_empty() {
            "Status message cleared.".to_string()
        } else {
            format!("Status message set to \"{message}\".")
        },
    );
    Ok(())
}
