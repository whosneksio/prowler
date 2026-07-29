use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::commands::log;
use crate::state::AppState;

const READY_CHECK_URI: &str = "/lol-matchmaking/v1/ready-check";

pub async fn run(app: AppHandle, state: Arc<AppState>, token: CancellationToken) {
    log(&app, "Auto Accept enabled.");
    let mut rx = state.ws_events.subscribe();
    let mut tick = tokio::time::interval(Duration::from_millis(1000));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = tick.tick() => {
                if !state.ws_live() {
                    check(&app, &state, &token).await;
                }
            }
            event = rx.recv() => {
                if let Ok(e) = event {
                    if e.uri == READY_CHECK_URI {
                        check(&app, &state, &token).await;
                    }
                }
            }
        }
    }
    log(&app, "Auto Accept disabled.");
}

async fn check(app: &AppHandle, state: &AppState, token: &CancellationToken) {
    let Ok(resp) = state.lcu(Method::GET, READY_CHECK_URI, None).await else {
        return;
    };
    if !resp.ok() || !should_accept(&resp.body) {
        return;
    }

    let delay = state.config.read().await.auto_accept.delay_seconds;
    tokio::select! {
        _ = token.cancelled() => return,
        _ = tokio::time::sleep(Duration::from_secs_f64(delay.max(0.0))) => {}
    }

    match state
        .lcu(Method::POST, "/lol-matchmaking/v1/ready-check/accept", None)
        .await
    {
        Ok(r) if r.ok() => log(app, "Match found - accepted."),
        Ok(r) => log(app, format!("Auto Accept failed: HTTP {}", r.status)),
        Err(e) => log(app, format!("Auto Accept failed: {e}")),
    }
}

fn should_accept(ready_check: &Value) -> bool {
    ready_check.get("state").and_then(|v| v.as_str()) == Some("InProgress")
        && ready_check.get("playerResponse").and_then(|v| v.as_str()) == Some("None")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_unanswered_ready_check() {
        assert!(should_accept(
            &json!({"state": "InProgress", "playerResponse": "None"})
        ));
    }

    #[test]
    fn skips_answered_or_absent_ready_check() {
        assert!(!should_accept(
            &json!({"state": "InProgress", "playerResponse": "Accepted"})
        ));
        assert!(!should_accept(
            &json!({"state": "Invalid", "playerResponse": "None"})
        ));
        assert!(!should_accept(&json!({"errorCode": "RPC_ERROR"})));
    }
}
