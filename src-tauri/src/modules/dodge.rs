use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

const QUIT_ENDPOINT: &str = r#"/lol-login/v1/session/invoke?destination=lcdsServiceProxy&method=call&args=["","teambuilder-draft","quitV2",""]"#;
const CANCEL_ENDPOINT: &str = "/lol-lobby/v1/lobby/custom/cancel-champ-select";
const PHASE_ENDPOINT: &str = "/lol-gameflow/v1/gameflow-phase";

const DODGE_ATTEMPTS: usize = 10;
const DODGE_RETRY_MS: u64 = 250;

fn is_error_envelope(body: &Value) -> bool {
    body.get("errorCode").is_some()
        || body
            .get("httpStatus")
            .and_then(|s| s.as_u64())
            .is_some_and(|s| s >= 400)
}

async fn in_champ_select(state: &AppState) -> bool {
    match state.lcu(Method::GET, PHASE_ENDPOINT, None).await {
        Ok(resp) if resp.ok() => resp.body.as_str() == Some("ChampSelect"),
        _ => true,
    }
}

#[tauri::command]
pub async fn dodge(app: AppHandle, state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    if !in_champ_select(&state).await {
        return Err("Not in champ select.".into());
    }

    let args = json!(["", "teambuilder-draft", "quitV2", ""]);
    let mut last_err = String::new();
    for attempt in 1..=DODGE_ATTEMPTS {
        match state
            .lcu(Method::POST, QUIT_ENDPOINT, Some(args.clone()))
            .await
        {
            Ok(resp) if resp.ok() && !is_error_envelope(&resp.body) => {}
            Ok(resp) => {
                let snippet: String = resp.body.to_string().chars().take(200).collect();
                last_err = format!("HTTP {} {}", resp.status, snippet);
            }
            Err(e) => last_err = e.to_string(),
        }
        if !in_champ_select(&state).await {
            log(&app, "Dodged champ select.");
            return Ok(());
        }
        if attempt < DODGE_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(DODGE_RETRY_MS)).await;
        }
    }

    let _ = state.lcu(Method::POST, CANCEL_ENDPOINT, None).await;
    if !in_champ_select(&state).await {
        log(&app, "Dodged champ select.");
        return Ok(());
    }

    log(&app, format!("Dodge failed: {last_err}"));
    Err(format!(
        "Dodge failed after {DODGE_ATTEMPTS} attempts: {last_err}"
    ))
}
