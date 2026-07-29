use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

fn chat_request(offline: bool) -> (&'static str, Option<Value>) {
    if offline {
        ("/chat/v1/suspend", Some(json!({ "config": "disable" })))
    } else {
        ("/chat/v1/resume", None)
    }
}

#[tauri::command]
pub async fn set_chat_offline(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    offline: bool,
) -> Result<(), String> {
    let (endpoint, body) = chat_request(offline);
    state.riot_checked(Method::POST, endpoint, body).await?;
    log(
        &app,
        if offline {
            "Chat disconnected - you now appear offline."
        } else {
            "Chat reconnected - you appear online again."
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspend_sends_disable_config_body() {
        let (endpoint, body) = chat_request(true);
        assert_eq!(endpoint, "/chat/v1/suspend");
        assert_eq!(body, Some(json!({ "config": "disable" })));
    }

    #[test]
    fn resume_has_no_body() {
        let (endpoint, body) = chat_request(false);
        assert_eq!(endpoint, "/chat/v1/resume");
        assert_eq!(body, None);
    }
}
