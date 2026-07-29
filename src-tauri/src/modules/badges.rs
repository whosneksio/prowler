use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

async fn update_badges(state: &AppState, ids: Vec<i64>) -> Result<(), String> {
    state
        .lcu_checked(
            Method::POST,
            "/lol-challenges/v1/update-player-preferences/",
            Some(json!({ "challengeIds": ids })),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_badges(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    mode: String,
) -> Result<(), String> {
    match mode.as_str() {
        "clear" => {
            update_badges(&state, vec![]).await?;
            log(&app, "Badges cleared.");
        }
        "glitch" => {
            update_badges(&state, vec![0, 0, 0]).await?;
            log(&app, "Badges glitched.");
        }
        other => return Err(format!("Unknown badge mode: {other}")),
    }
    Ok(())
}
