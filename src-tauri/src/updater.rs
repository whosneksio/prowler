use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::sync::Mutex;

use crate::commands::log;
use crate::state::AppState;

pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[derive(Serialize, Clone, Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

fn info_of(update: &Update) -> UpdateInfo {
    UpdateInfo {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
        date: update.date.map(|d| d.to_string()),
    }
}

async fn fetch(app: &AppHandle) -> Result<Option<Update>, String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let pending = app.state::<PendingUpdate>();
    *pending.0.lock().await = update.clone();
    let info = update.as_ref().map(info_of);
    let _ = app.emit("prowler://update", &info);
    Ok(update)
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    match fetch(&app).await {
        Ok(Some(update)) => {
            log(
                &app,
                format!(
                    "Update {} is available (you have {}).",
                    update.version, update.current_version
                ),
            );
            Ok(Some(info_of(&update)))
        }
        Ok(None) => {
            log(&app, "Prowler is up to date.");
            Ok(None)
        }
        Err(e) => {
            log(&app, format!("Update check failed: {e}"));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    pending: tauri::State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = pending
        .0
        .lock()
        .await
        .take()
        .ok_or("no pending update - check for updates first")?;

    log(&app, format!("Downloading update {}…", update.version));

    let progress_app = app.clone();
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    let mut downloaded: u64 = 0;
    update
        .download_and_install(
            move |chunk, total| {
                downloaded += chunk as u64;
                if last_emit.elapsed() >= Duration::from_millis(200) {
                    last_emit = Instant::now();
                    let _ = progress_app.emit(
                        "prowler://update-progress",
                        UpdateProgress { downloaded, total },
                    );
                }
            },
            {
                let app = app.clone();
                move || log(&app, "Download complete - running the installer.")
            },
        )
        .await
        .map_err(|e| {
            let msg = format!("Update failed: {e}");
            log(&app, msg.clone());
            msg
        })?;

    app.restart();
}

pub async fn auto_check(app: AppHandle, state: Arc<AppState>) {
    tokio::time::sleep(Duration::from_secs(5)).await;
    let mut reported_failure = false;
    loop {
        let enabled = state.config.read().await.updates.auto_check;
        if enabled {
            match fetch(&app).await {
                Ok(Some(update)) => {
                    log(
                        &app,
                        format!(
                            "Update {} is available (you have {}).",
                            update.version, update.current_version
                        ),
                    );
                    reported_failure = false;
                }
                Ok(None) => {
                    reported_failure = false;
                }
                Err(e) => {
                    if !reported_failure {
                        log(&app, format!("Update check failed: {e}"));
                        reported_failure = true;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
    }
}
