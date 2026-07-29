use std::sync::Arc;

use reqwest::Method;
use serde_json::json;
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn set_background(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    skin_id: i64,
) -> Result<(), String> {
    if skin_id < 0 {
        return Err("Skin id must be a positive number.".into());
    }
    state
        .lcu_checked(
            Method::POST,
            "/lol-summoner/v1/current-summoner/summoner-profile",
            Some(json!({ "key": "backgroundSkinId", "value": skin_id })),
        )
        .await?;
    log(&app, format!("Profile background set to skin {skin_id}."));
    Ok(())
}
