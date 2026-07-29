use std::sync::Arc;

use reqwest::Method;
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn count_friends(state: tauri::State<'_, Arc<AppState>>) -> Result<usize, String> {
    let friends = state
        .lcu_checked(Method::GET, "/lol-chat/v1/friends", None)
        .await?;
    Ok(friends.as_array().map(|a| a.len()).unwrap_or(0))
}

#[tauri::command]
pub async fn remove_all_friends(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    let friends = state
        .lcu_checked(Method::GET, "/lol-chat/v1/friends", None)
        .await?;
    let pids: Vec<String> = friends
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.get("pid").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut removed = 0;
    for pid in &pids {
        let endpoint = format!("/lol-chat/v1/friends/{}", urlencoding::encode(pid));
        if state
            .lcu_checked(Method::DELETE, &endpoint, None)
            .await
            .is_ok()
        {
            removed += 1;
        }
    }
    log(
        &app,
        format!("Removed {removed} of {} friends.", pids.len()),
    );
    Ok(removed)
}
