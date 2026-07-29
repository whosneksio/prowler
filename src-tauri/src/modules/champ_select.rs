use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::AppHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::commands::log;
use crate::state::AppState;

const SESSION_URI: &str = "/lol-champ-select/v1/session";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Champion {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalAction {
    pub id: i64,
    pub kind: String,
    pub in_progress: bool,
    pub completed: bool,
    pub champion_id: i64,
}

struct RequestedLock {
    module: &'static str,
    verb: &'static str,
    name: String,
    role: String,
    champ: i64,
}

#[derive(Default)]
struct SelectRuntime {
    first_seen: HashMap<i64, Instant>,
    hovered: HashMap<i64, i64>,
    requested: HashMap<i64, RequestedLock>,
    confirmed: HashSet<i64>,
    warned: HashSet<i64>,
    hover_warned: HashSet<i64>,
}

impl SelectRuntime {
    fn clear(&mut self) {
        self.first_seen.clear();
        self.hovered.clear();
        self.requested.clear();
        self.confirmed.clear();
        self.warned.clear();
        self.hover_warned.clear();
    }
}

pub async fn run(app: AppHandle, state: Arc<AppState>, token: CancellationToken) {
    log(&app, "Champ-select automation enabled.");
    let mut rx = state.ws_events.subscribe();
    let mut tick = tokio::time::interval(Duration::from_millis(400));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut rt = SelectRuntime::default();

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = tick.tick() => {
                handle_session_poll(&app, &state, &mut rt).await;
            }
            event = rx.recv() => {
                if let Ok(e) = event {
                    if e.uri == SESSION_URI {
                        if e.event_type == "Delete" {
                            rt.clear();
                        } else {
                            handle_session(&app, &state, &mut rt, &e.data).await;
                        }
                    }
                }
            }
        }
    }
    log(&app, "Champ-select automation disabled.");
}

async fn handle_session_poll(app: &AppHandle, state: &AppState, rt: &mut SelectRuntime) {
    match state.lcu(Method::GET, SESSION_URI, None).await {
        Ok(resp) if resp.ok() => handle_session(app, state, rt, &resp.body).await,
        Ok(_) => rt.clear(),
        Err(_) => {}
    }
}

async fn handle_session(
    app: &AppHandle,
    state: &AppState,
    rt: &mut SelectRuntime,
    session: &Value,
) {
    let actions = local_actions(session);
    if actions.is_empty() {
        return;
    }
    let cfg = state.config.read().await.clone();
    let role = local_assigned_position(session);
    let role_label = role.as_deref().unwrap_or("Default").to_string();

    for a in &actions {
        if a.completed && !rt.confirmed.contains(&a.id) {
            if let Some(req) = rt.requested.get(&a.id) {
                if a.champion_id == req.champ {
                    log(
                        app,
                        format!("{}: {} {} ({}).", req.module, req.verb, req.name, req.role),
                    );
                }
            }
            rt.confirmed.insert(a.id);
        }
    }

    let banned = banned_champion_ids(session);
    let intents = teammate_pick_intents(session);
    let empty: HashSet<i64> = HashSet::new();
    let no_filter: Option<HashSet<i64>> = None;

    let prepick_id = if cfg.instalock.prepick {
        actions
            .iter()
            .find(|a| a.kind == "pick" && !a.completed && !a.in_progress)
            .map(|a| a.id)
    } else {
        None
    };
    let has_live_pick = actions
        .iter()
        .any(|a| a.kind == "pick" && a.in_progress && !a.completed);

    let prepick_wants_pickable =
        prepick_id.is_some() && !cfg.instalock.champions.for_role(role.as_deref()).is_empty();
    let pickable = if prepick_wants_pickable || (has_live_pick && cfg.instalock.enabled) {
        pickable_ids(state).await
    } else {
        None
    };

    if let Some(pick_id) = prepick_id {
        let candidates = cfg.instalock.champions.for_role(role.as_deref());
        if let Some((champ, name)) =
            choose_candidate(state, candidates, &banned, &empty, &pickable, true).await
        {
            if rt.hovered.get(&pick_id) != Some(&champ) {
                let endpoint = format!("{SESSION_URI}/actions/{pick_id}");
                let body = json!({ "championId": champ, "completed": false });
                match state.lcu(Method::PATCH, &endpoint, Some(body)).await {
                    Ok(r) if r.ok() && !is_lcu_error(&r.body) => {
                        rt.hovered.insert(pick_id, champ);
                        log(app, format!("Prepick: hovering {name} ({role_label})."));
                    }
                    Ok(r) => {
                        if rt.hover_warned.insert(pick_id) {
                            log(
                                app,
                                format!(
                                    "Prepick: hover failed (HTTP {}) {}.",
                                    r.status,
                                    brief(&r.body)
                                ),
                            );
                        }
                    }
                    Err(e) => {
                        if rt.hover_warned.insert(pick_id) {
                            log(app, format!("Prepick: hover failed ({e})."));
                        }
                    }
                }
            }
        }
    }

    for a in actions.iter().filter(|a| a.in_progress && !a.completed) {
        if rt.confirmed.contains(&a.id) {
            continue;
        }
        let (module, verb, list, delay) = match a.kind.as_str() {
            "pick" if cfg.instalock.enabled => (
                "Instalock",
                "locked",
                &cfg.instalock.champions,
                cfg.instalock.delay_seconds,
            ),
            "ban" if cfg.autoban.enabled => (
                "Autoban",
                "banned",
                &cfg.autoban.champions,
                cfg.autoban.delay_seconds,
            ),
            _ => continue,
        };
        let candidates = list.for_role(role.as_deref());
        if candidates.is_empty() {
            continue;
        }

        let now = Instant::now();
        let since = *rt.first_seen.entry(a.id).or_insert(now);
        if now.duration_since(since) < Duration::from_secs_f64(delay.max(0.0)) {
            continue;
        }

        let avoid = if a.kind == "ban" { &intents } else { &empty };
        let pick_filter = if a.kind == "pick" {
            &pickable
        } else {
            &no_filter
        };
        let allow_random = a.kind == "pick";

        match choose_candidate(state, candidates, &banned, avoid, pick_filter, allow_random).await {
            Some((champ, name)) => {
                let endpoint = format!("{SESSION_URI}/actions/{}", a.id);
                let body = json!({ "championId": champ, "completed": true });
                match state.lcu(Method::PATCH, &endpoint, Some(body)).await {
                    Ok(r) if r.ok() && !is_lcu_error(&r.body) => {
                        rt.requested.insert(
                            a.id,
                            RequestedLock {
                                module,
                                verb,
                                name,
                                role: role_label.clone(),
                                champ,
                            },
                        );
                    }
                    Ok(r) => {
                        if rt.warned.insert(a.id) {
                            log(
                                app,
                                format!(
                                    "{module}: {name} failed (HTTP {}) {}.",
                                    r.status,
                                    brief(&r.body)
                                ),
                            );
                        }
                    }
                    Err(e) => {
                        if rt.warned.insert(a.id) {
                            log(app, format!("{module}: {name} failed ({e})."));
                        }
                    }
                }
            }
            None => {
                if rt.warned.insert(a.id) {
                    log(
                        app,
                        format!("{module}: all {} choices unavailable.", candidates.len()),
                    );
                }
            }
        }
    }
}

async fn choose_candidate(
    state: &AppState,
    candidates: &[String],
    banned: &HashSet<i64>,
    avoid: &HashSet<i64>,
    pickable: &Option<HashSet<i64>>,
    allow_random: bool,
) -> Option<(i64, String)> {
    for name in candidates {
        let champ = if name.eq_ignore_ascii_case("random") {
            if !allow_random {
                continue;
            }
            random_pickable(state).await
        } else {
            resolve_champion_id(state, name).await
        };
        let Some(champ) = champ else { continue };
        if banned.contains(&champ) || avoid.contains(&champ) {
            continue;
        }
        if let Some(ids) = pickable {
            if !ids.contains(&champ) {
                continue;
            }
        }
        return Some((champ, name.clone()));
    }
    None
}

fn is_lcu_error(body: &Value) -> bool {
    body.get("errorCode").is_some()
        || body
            .get("httpStatus")
            .and_then(|s| s.as_u64())
            .is_some_and(|s| s >= 400)
}

fn brief(body: &Value) -> String {
    if body.is_null() {
        return String::new();
    }
    body.to_string().chars().take(200).collect()
}

pub fn local_actions(session: &Value) -> Vec<LocalAction> {
    let Some(local_cell) = session.get("localPlayerCellId").and_then(|v| v.as_i64()) else {
        return Vec::new();
    };
    session
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|group| group.as_array())
        .flatten()
        .filter(|a| a.get("actorCellId").and_then(|v| v.as_i64()) == Some(local_cell))
        .filter_map(|a| {
            Some(LocalAction {
                id: a.get("id")?.as_i64()?,
                kind: a.get("type")?.as_str()?.to_string(),
                in_progress: a
                    .get("isInProgress")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                completed: a
                    .get("completed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                champion_id: a.get("championId").and_then(|v| v.as_i64()).unwrap_or(0),
            })
        })
        .collect()
}

pub fn local_assigned_position(session: &Value) -> Option<String> {
    let cell = session.get("localPlayerCellId")?.as_i64()?;
    session
        .get("myTeam")?
        .as_array()?
        .iter()
        .find(|p| p.get("cellId").and_then(|v| v.as_i64()) == Some(cell))?
        .get("assignedPosition")?
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_uppercase())
}

pub fn banned_champion_ids(session: &Value) -> HashSet<i64> {
    let mut banned = HashSet::new();
    if let Some(bans) = session.get("bans") {
        for side in ["myTeamBans", "theirTeamBans"] {
            for id in bans
                .get(side)
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(id) = id.as_i64() {
                    banned.insert(id);
                }
            }
        }
    }
    for action in session
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|group| group.as_array())
        .flatten()
    {
        if action.get("type").and_then(|v| v.as_str()) == Some("ban")
            && action.get("completed").and_then(|v| v.as_bool()) == Some(true)
        {
            if let Some(id) = action.get("championId").and_then(|v| v.as_i64()) {
                banned.insert(id);
            }
        }
    }
    banned.remove(&0);
    banned
}

pub fn teammate_pick_intents(session: &Value) -> HashSet<i64> {
    let local = session.get("localPlayerCellId").and_then(|v| v.as_i64());
    session
        .get("myTeam")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter(|p| p.get("cellId").and_then(|v| v.as_i64()) != local)
        .filter_map(|p| p.get("championPickIntent").and_then(|v| v.as_i64()))
        .filter(|id| *id > 0)
        .collect()
}

async fn champions(state: &AppState) -> Result<Vec<Champion>, String> {
    if let Some(list) = state.champions.read().await.as_ref() {
        return Ok(list.clone());
    }
    let body = state
        .lcu_checked(
            Method::GET,
            "/lol-game-data/assets/v1/champion-summary.json",
            None,
        )
        .await?;
    let list = parse_champion_summary(&body);
    if list.is_empty() {
        return Err("Champion list is empty - is the client fully loaded?".into());
    }
    *state.champions.write().await = Some(list.clone());
    Ok(list)
}

pub fn parse_champion_summary(body: &Value) -> Vec<Champion> {
    let mut list: Vec<Champion> = body
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| serde_json::from_value::<Champion>(v.clone()).ok())
        .filter(|c| c.id > 0)
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

async fn resolve_champion_id(state: &AppState, name: &str) -> Option<i64> {
    let list = champions(state).await.ok()?;
    resolve_in(&list, name)
}

pub fn resolve_in(list: &[Champion], name: &str) -> Option<i64> {
    let needle = name.trim();
    list.iter()
        .find(|c| c.name.eq_ignore_ascii_case(needle) || c.alias.eq_ignore_ascii_case(needle))
        .map(|c| c.id)
}

async fn pickable_ids(state: &AppState) -> Option<HashSet<i64>> {
    let ids: HashSet<i64> = state
        .lcu_checked(
            Method::GET,
            "/lol-champ-select/v1/pickable-champion-ids",
            None,
        )
        .await
        .ok()?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

async fn random_pickable(state: &AppState) -> Option<i64> {
    let ids: Vec<i64> = pickable_ids(state).await?.into_iter().collect();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    Some(ids[nanos % ids.len()])
}

#[tauri::command]
pub async fn list_champions(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<Champion>, String> {
    champions(&state).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> Value {
        json!({
            "localPlayerCellId": 2,
            "actions": [
                [
                    {"id": 1, "actorCellId": 0, "type": "ban", "isInProgress": true, "completed": false},
                    {"id": 2, "actorCellId": 2, "type": "ban", "isInProgress": true, "completed": false}
                ],
                [
                    {"id": 10, "actorCellId": 2, "type": "pick", "isInProgress": false, "completed": false},
                    {"id": 11, "actorCellId": 2, "type": "pick", "isInProgress": true, "completed": true}
                ]
            ]
        })
    }

    #[test]
    fn finds_only_local_in_progress_uncompleted_actions() {
        let pending: Vec<_> = local_actions(&sample_session())
            .into_iter()
            .filter(|a| a.in_progress && !a.completed)
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, 2);
        assert_eq!(pending[0].kind, "ban");
    }

    #[test]
    fn local_actions_includes_ban_phase_pick_for_prepick() {
        let actions = local_actions(&sample_session());
        assert_eq!(actions.len(), 3);
        let prepick = actions
            .iter()
            .find(|a| a.kind == "pick" && !a.completed && !a.in_progress);
        assert_eq!(prepick.map(|a| a.id), Some(10));
    }

    #[test]
    fn empty_when_no_local_cell() {
        assert!(local_actions(&json!({"actions": []})).is_empty());
        assert!(local_actions(&json!({})).is_empty());
    }

    #[test]
    fn detects_lcu_error_envelope() {
        assert!(is_lcu_error(
            &json!({"errorCode": "RPC_ERROR", "message": "x"})
        ));
        assert!(is_lcu_error(&json!({"httpStatus": 500})));
        assert!(!is_lcu_error(&json!({"httpStatus": 204})));
        assert!(!is_lcu_error(&Value::Null));
        assert!(!is_lcu_error(&json!([1, 2, 3])));
    }

    #[test]
    fn reads_local_assigned_position() {
        let session = json!({
            "localPlayerCellId": 2,
            "myTeam": [
                {"cellId": 0, "assignedPosition": "top"},
                {"cellId": 2, "assignedPosition": "jungle"}
            ]
        });
        assert_eq!(local_assigned_position(&session), Some("JUNGLE".into()));
    }

    #[test]
    fn no_position_when_empty_or_missing() {
        let blank = json!({
            "localPlayerCellId": 2,
            "myTeam": [{"cellId": 2, "assignedPosition": ""}]
        });
        assert_eq!(local_assigned_position(&blank), None);
        assert_eq!(
            local_assigned_position(&json!({"localPlayerCellId": 2})),
            None
        );
        assert_eq!(local_assigned_position(&json!({})), None);
    }

    #[test]
    fn collects_banned_champion_ids() {
        let session = json!({
            "bans": {"myTeamBans": [103, 0], "theirTeamBans": [62]},
            "actions": [[
                {"id": 1, "actorCellId": 0, "type": "ban", "completed": true, "championId": 11},
                {"id": 2, "actorCellId": 1, "type": "ban", "completed": false, "championId": 55},
                {"id": 3, "actorCellId": 2, "type": "pick", "completed": true, "championId": 99}
            ]]
        });
        let banned = banned_champion_ids(&session);
        assert_eq!(banned, HashSet::from([103, 62, 11]));
    }

    #[test]
    fn collects_teammate_pick_intents() {
        let session = json!({
            "localPlayerCellId": 2,
            "myTeam": [
                {"cellId": 0, "championPickIntent": 103},
                {"cellId": 1, "championPickIntent": 0},
                {"cellId": 2, "championPickIntent": 62}
            ]
        });
        assert_eq!(teammate_pick_intents(&session), HashSet::from([103]));
    }

    #[test]
    fn parses_and_resolves_champions() {
        let body = json!([
            {"id": -1, "name": "None", "alias": "None"},
            {"id": 103, "name": "Ahri", "alias": "Ahri"},
            {"id": 62, "name": "Wukong", "alias": "MonkeyKing"}
        ]);
        let list = parse_champion_summary(&body);
        assert_eq!(list.len(), 2);
        assert_eq!(resolve_in(&list, "ahri"), Some(103));
        assert_eq!(resolve_in(&list, "monkeyking"), Some(62));
        assert_eq!(resolve_in(&list, "Wukong"), Some(62));
        assert_eq!(resolve_in(&list, "Teemo"), None);
    }
}
