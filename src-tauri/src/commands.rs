use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::config::Config;
use crate::lcu::models::CurrentSummoner;
use crate::state::AppState;
use crate::switcher::vault::{self, AccountMeta};
use crate::switcher::{launcher, processes};

pub(crate) fn log(app: &AppHandle, msg: impl Into<String>) {
    let _ = app.emit("prowler://log", msg.into());
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub summoner: Option<CurrentSummoner>,
    pub phase: Option<String>,
}

async fn fetch_status(state: &AppState) -> ConnectionStatus {
    match state
        .lcu(Method::GET, "/lol-summoner/v1/current-summoner", None)
        .await
    {
        Ok(resp) if resp.ok() => {
            let summoner: CurrentSummoner = serde_json::from_value(resp.body).unwrap_or_default();
            let phase = state
                .lcu(Method::GET, "/lol-gameflow/v1/gameflow-phase", None)
                .await
                .ok()
                .and_then(|r| r.body.as_str().map(|s| s.to_string()));
            ConnectionStatus {
                connected: true,
                summoner: Some(summoner),
                phase,
            }
        }
        _ => {
            state.refresh_creds().await;
            ConnectionStatus::default()
        }
    }
}

pub async fn connection_monitor(app: AppHandle, state: Arc<AppState>) {
    loop {
        let status = fetch_status(&state).await;
        let _ = app.emit("lcu://status", &status);
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
}

#[tauri::command]
pub async fn get_connection_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<ConnectionStatus, String> {
    Ok(fetch_status(&state).await)
}

#[tauri::command]
pub async fn get_config(state: tauri::State<'_, Arc<AppState>>) -> Result<Config, String> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn set_config(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    config: Config,
) -> Result<(), String> {
    *state.config.write().await = config;
    state.persist_config().await;
    crate::automations::sync(&app, &state).await;
    Ok(())
}

async fn shutdown_clients(state: &AppState) -> Result<(), String> {
    let graceful = state
        .riot(Method::POST, "/process-control/v1/process/quit", None)
        .await
        .is_ok();

    let closed = tauri::async_runtime::spawn_blocking(move || {
        if graceful && processes::wait_until_closed(Duration::from_secs(8)) {
            return true;
        }
        processes::force_close(Duration::from_secs(10))
    })
    .await
    .map_err(|e| e.to_string())?;
    if !closed {
        return Err("Riot/League processes did not exit in time - try again.".into());
    }
    state.creds.write().await.take();
    state.riot_creds.write().await.take();
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}

#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<AccountMeta>, String> {
    vault::list_accounts(&state.vault_dir)
}

#[tauri::command]
pub async fn save_current_account(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    label: Option<String>,
) -> Result<AccountMeta, String> {
    let summoner: Option<CurrentSummoner> = match state
        .lcu(Method::GET, "/lol-summoner/v1/current-summoner", None)
        .await
    {
        Ok(resp) if resp.ok() => serde_json::from_value(resp.body).ok(),
        _ => None,
    };
    let region = match state
        .lcu(Method::GET, "/riotclient/region-locale", None)
        .await
    {
        Ok(resp) if resp.ok() => resp
            .body
            .get("region")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    };

    let existing = vault::list_accounts(&state.vault_dir)?;
    let known = summoner
        .as_ref()
        .filter(|s| !s.puuid.is_empty())
        .and_then(|s| existing.iter().find(|a| a.puuid == s.puuid).cloned());

    let meta = match known {
        Some(mut meta) => {
            if let Some(s) = &summoner {
                meta.game_name = s.game_name.clone();
                meta.tag_line = s.tag_line.clone();
                meta.profile_icon_id = s.profile_icon_id;
            }
            if !region.is_empty() {
                meta.region = region;
            }
            if let Some(l) = label.filter(|l| !l.trim().is_empty()) {
                meta.label = l.trim().to_string();
            }
            meta
        }
        None => {
            let now = vault::now_ms();
            let default_label = summoner
                .as_ref()
                .map(|s| s.riot_id())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Account {}", existing.len() + 1));
            AccountMeta {
                id: format!("acct-{now}"),
                label: label
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.trim().to_string())
                    .unwrap_or(default_label),
                game_name: summoner
                    .as_ref()
                    .map(|s| s.game_name.clone())
                    .unwrap_or_default(),
                tag_line: summoner
                    .as_ref()
                    .map(|s| s.tag_line.clone())
                    .unwrap_or_default(),
                puuid: summoner
                    .as_ref()
                    .map(|s| s.puuid.clone())
                    .unwrap_or_default(),
                region,
                profile_icon_id: summoner.as_ref().map(|s| s.profile_icon_id).unwrap_or(0),
                created_ms: now,
            }
        }
    };

    log(&app, "Closing Riot/League to flush the session to disk…");
    shutdown_clients(&state).await?;
    vault::save_account(&state.vault_dir, &meta)?;
    log(
        &app,
        format!("Saved session for \"{}\" - relaunching…", meta.label),
    );
    launcher::launch_league()?;
    Ok(meta)
}

#[tauri::command]
pub async fn switch_account(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let meta = vault::load_meta(&state.vault_dir, &id)?;

    let outgoing = match state
        .lcu(Method::GET, "/lol-summoner/v1/current-summoner", None)
        .await
    {
        Ok(resp) if resp.ok() => serde_json::from_value::<CurrentSummoner>(resp.body).ok(),
        _ => None,
    }
    .filter(|s| !s.puuid.is_empty())
    .and_then(|s| {
        vault::list_accounts(&state.vault_dir)
            .ok()?
            .into_iter()
            .find(|a| a.puuid == s.puuid && a.id != id)
    });

    log(
        &app,
        format!("Switching to \"{}\" - closing Riot/League…", meta.label),
    );
    shutdown_clients(&state).await?;

    if let Some(prev) = outgoing {
        if vault::live_session_has_tokens() {
            match vault::save_account(&state.vault_dir, &prev) {
                Ok(()) => log(
                    &app,
                    format!("Refreshed saved session for \"{}\".", prev.label),
                ),
                Err(e) => log(&app, format!("Could not refresh \"{}\": {e}", prev.label)),
            }
        }
    }

    vault::restore_session(&state.vault_dir, &id)?;
    log(&app, "Session restored - launching Riot Client…");
    launcher::launch_league()?;
    log(
        &app,
        format!("\"{}\" should sign in automatically.", meta.label),
    );
    Ok(())
}

#[tauri::command]
pub async fn rename_account(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    label: String,
) -> Result<AccountMeta, String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("Label cannot be empty.".into());
    }
    let mut meta = vault::load_meta(&state.vault_dir, &id)?;
    meta.label = label.to_string();
    vault::save_meta(&state.vault_dir, &meta)?;
    Ok(meta)
}

#[tauri::command]
pub async fn delete_account(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    let meta = vault::load_meta(&state.vault_dir, &id)?;
    vault::delete_account(&state.vault_dir, &id)?;
    log(&app, format!("Deleted account \"{}\".", meta.label));
    Ok(())
}
