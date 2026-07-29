use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::modules::{auto_accept, champ_select, loadout};
use crate::state::AppState;

pub const AUTO_ACCEPT: &str = "auto_accept";
pub const CHAMP_SELECT: &str = "champ_select";
pub const LOADOUT: &str = "loadout";

pub async fn sync(app: &AppHandle, state: &Arc<AppState>) {
    let cfg = state.config.read().await.clone();
    reconcile(app, state, AUTO_ACCEPT, cfg.auto_accept.enabled).await;
    reconcile(
        app,
        state,
        CHAMP_SELECT,
        cfg.instalock.enabled || cfg.autoban.enabled || cfg.instalock.prepick,
    )
    .await;
    reconcile(
        app,
        state,
        LOADOUT,
        cfg.auto_runes.enabled || cfg.auto_spells.enabled,
    )
    .await;

    let running: Vec<&str> = state.automations.read().await.keys().copied().collect();
    let _ = app.emit("prowler://automations", &running);
}

async fn reconcile(app: &AppHandle, state: &Arc<AppState>, key: &'static str, wanted: bool) {
    let mut tasks = state.automations.write().await;
    let running = tasks.contains_key(key);
    if wanted && !running {
        let token = CancellationToken::new();
        tasks.insert(key, token.clone());
        let (app, state) = (app.clone(), state.clone());
        tauri::async_runtime::spawn(async move {
            match key {
                AUTO_ACCEPT => auto_accept::run(app, state, token).await,
                CHAMP_SELECT => champ_select::run(app, state, token).await,
                LOADOUT => loadout::run(app, state, token).await,
                _ => {}
            }
        });
    } else if !wanted && running {
        if let Some(token) = tasks.remove(key) {
            token.cancel();
        }
    }
}

#[tauri::command]
pub async fn set_automation(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut cfg = state.config.write().await;
        match name.as_str() {
            "auto_accept" => cfg.auto_accept.enabled = enabled,
            "instalock" => cfg.instalock.enabled = enabled,
            "prepick" => cfg.instalock.prepick = enabled,
            "autoban" => cfg.autoban.enabled = enabled,
            "auto_runes" => cfg.auto_runes.enabled = enabled,
            "auto_spells" => cfg.auto_spells.enabled = enabled,
            other => return Err(format!("unknown automation \"{other}\"")),
        }
    }
    state.persist_config().await;
    sync(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn get_running_automations(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<String>, String> {
    Ok(state
        .automations
        .read()
        .await
        .keys()
        .map(|k| k.to_string())
        .collect())
}
