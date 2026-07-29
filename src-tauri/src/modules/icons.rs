use std::sync::Arc;

use reqwest::Method;
use serde_json::json;
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn set_profile_icon(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    icon_id: i64,
) -> Result<(), String> {
    if icon_id < 0 {
        return Err("Icon id must be a positive number.".into());
    }
    state
        .lcu_checked(
            Method::PUT,
            "/lol-summoner/v1/current-summoner/icon",
            Some(json!({ "profileIconId": icon_id })),
        )
        .await?;
    log(&app, format!("Profile icon set to {icon_id}."));
    Ok(())
}

#[tauri::command]
pub async fn set_client_icon(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    icon_id: i64,
) -> Result<(), String> {
    if icon_id < 0 {
        return Err("Icon id must be a positive number.".into());
    }
    state
        .lcu_checked(
            Method::PUT,
            "/lol-chat/v1/me",
            Some(json!({ "icon": icon_id })),
        )
        .await?;
    log(&app, format!("Client-only icon set to {icon_id}."));
    Ok(())
}
