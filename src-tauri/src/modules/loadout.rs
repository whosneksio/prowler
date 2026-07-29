use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use serde_json::{json, Value};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::commands::log;
use crate::state::AppState;

use super::champ_select::local_assigned_position;

const SESSION_URI: &str = "/lol-champ-select/v1/session";
const PAGE_NAME: &str = "Prowler";

pub const SPELLS: [(&str, i64); 11] = [
    ("Flash", 4),
    ("Smite", 11),
    ("Teleport", 12),
    ("Ignite", 14),
    ("Heal", 7),
    ("Ghost", 6),
    ("Exhaust", 3),
    ("Barrier", 21),
    ("Cleanse", 1),
    ("Snowball", 32),
    ("Clarity", 13),
];

pub fn spell_id(name: &str) -> Option<i64> {
    let needle = name.trim();
    SPELLS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(needle))
        .map(|(_, id)| *id)
}

pub async fn run(app: AppHandle, state: Arc<AppState>, token: CancellationToken) {
    log(&app, "Loadout automation enabled.");
    let mut rx = state.ws_events.subscribe();
    let mut tick = tokio::time::interval(Duration::from_millis(400));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut applied_for: Option<i64> = None;

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = tick.tick() => {
                if !state.ws_live() {
                    match state.lcu(Method::GET, SESSION_URI, None).await {
                        Ok(resp) if resp.ok() => {
                            handle_session(&app, &state, &mut applied_for, &resp.body).await
                        }
                        Ok(_) => applied_for = None,
                        Err(_) => {}
                    }
                }
            }
            event = rx.recv() => {
                if let Ok(e) = event {
                    if e.uri == SESSION_URI {
                        if e.event_type == "Delete" {
                            applied_for = None;
                        } else {
                            handle_session(&app, &state, &mut applied_for, &e.data).await;
                        }
                    }
                }
            }
        }
    }
    log(&app, "Loadout automation disabled.");
}

async fn handle_session(
    app: &AppHandle,
    state: &AppState,
    applied_for: &mut Option<i64>,
    session: &Value,
) {
    let Some(champion_id) = locked_champion_id(session) else {
        return;
    };
    if *applied_for == Some(champion_id) {
        return;
    }
    *applied_for = Some(champion_id);

    let cfg = state.config.read().await.clone();
    let role = local_assigned_position(session);

    if cfg.auto_spells.enabled {
        apply_spells(app, state, &cfg.auto_spells.roles, role.as_deref()).await;
    }
    if cfg.auto_runes.enabled {
        apply_runes(app, state, champion_id, role.as_deref()).await;
    }
}

pub fn locked_champion_id(session: &Value) -> Option<i64> {
    let cell = session.get("localPlayerCellId")?.as_i64()?;
    let pick_done = session
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|group| group.as_array())
        .flatten()
        .any(|a| {
            a.get("actorCellId").and_then(|v| v.as_i64()) == Some(cell)
                && a.get("type").and_then(|v| v.as_str()) == Some("pick")
                && a.get("completed").and_then(|v| v.as_bool()) == Some(true)
        });
    let champion_id = session
        .get("myTeam")?
        .as_array()?
        .iter()
        .find(|p| p.get("cellId").and_then(|v| v.as_i64()) == Some(cell))?
        .get("championId")?
        .as_i64()?;
    let has_pick_actions = session
        .get("actions")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|group| group.as_array())
        .flatten()
        .any(|a| a.get("type").and_then(|v| v.as_str()) == Some("pick"));
    if champion_id > 0 && (pick_done || !has_pick_actions) {
        Some(champion_id)
    } else {
        None
    }
}

async fn apply_spells(
    app: &AppHandle,
    state: &AppState,
    roles: &crate::config::RoleSpells,
    role: Option<&str>,
) {
    let pair = roles.for_role(role);
    let (Some(spell1), Some(spell2)) = (spell_id(&pair[0]), spell_id(&pair[1])) else {
        log(app, format!("Loadout: unknown summoner spell in {pair:?}."));
        return;
    };
    let body = json!({ "spell1Id": spell1, "spell2Id": spell2 });
    match state
        .lcu(
            Method::PATCH,
            &format!("{SESSION_URI}/my-selection"),
            Some(body),
        )
        .await
    {
        Ok(r) if r.ok() => log(
            app,
            format!("Loadout: summoners set to {} / {}.", pair[0], pair[1]),
        ),
        Ok(r) => log(
            app,
            format!("Loadout: setting summoners failed (HTTP {}).", r.status),
        ),
        Err(e) => log(app, format!("Loadout: setting summoners failed: {e}")),
    }
}

async fn apply_runes(app: &AppHandle, state: &AppState, champion_id: i64, role: Option<&str>) {
    let position = role
        .map(|r| r.to_ascii_lowercase())
        .unwrap_or_else(|| "none".into());
    let map_id = current_map_id(state).await.unwrap_or(11);

    let endpoint = format!(
        "/lol-perks/v1/recommended-pages/champion/{champion_id}/position/{position}/map/{map_id}"
    );
    let pages = match state.lcu_checked(Method::GET, &endpoint, None).await {
        Ok(v) => v,
        Err(e) => {
            log(app, format!("Loadout: no rune recommendation ({e})."));
            return;
        }
    };
    let Some(body) = recommended_to_page_body(&pages) else {
        log(app, "Loadout: could not parse the recommended rune page.");
        return;
    };

    match apply_page(state, body).await {
        Ok(()) => log(
            app,
            format!("Loadout: applied recommended runes ({position})."),
        ),
        Err(e) => log(app, format!("Loadout: applying runes failed: {e}")),
    }
}
async fn current_map_id(state: &AppState) -> Option<i64> {
    state
        .lcu_checked(Method::GET, "/lol-gameflow/v1/session", None)
        .await
        .ok()?
        .get("map")?
        .get("id")?
        .as_i64()
}

pub fn recommended_to_page_body(recommended: &Value) -> Option<Value> {
    let page = recommended.as_array()?.first()?;
    let primary = page
        .get("primaryPerkStyleId")
        .or_else(|| page.get("primaryStyleId"))?
        .as_i64()?;
    let sub = page
        .get("secondaryPerkStyleId")
        .or_else(|| page.get("subStyleId"))?
        .as_i64()?;
    let perks: Vec<i64> = page
        .get("perks")?
        .as_array()?
        .iter()
        .filter_map(|p| p.get("id").and_then(|v| v.as_i64()).or_else(|| p.as_i64()))
        .collect();
    if perks.is_empty() {
        return None;
    }
    Some(json!({
        "name": PAGE_NAME,
        "primaryStyleId": primary,
        "subStyleId": sub,
        "selectedPerkIds": perks,
        "current": true,
    }))
}

pub async fn apply_page(state: &AppState, body: Value) -> Result<(), String> {
    let pages = state
        .lcu_checked(Method::GET, "/lol-perks/v1/pages", None)
        .await?;
    let pages = pages.as_array().ok_or("unexpected pages payload")?;

    let prowler_page_id = pages
        .iter()
        .filter(|p| {
            p.get("isEditable").and_then(|v| v.as_bool()) != Some(false)
                && p.get("name")
                    .and_then(|v| v.as_str())
                    .is_some_and(|n| n.starts_with(PAGE_NAME))
        })
        .filter_map(|p| p.get("id").and_then(|v| v.as_i64()))
        .next();

    if let Some(id) = prowler_page_id {
        state
            .lcu_checked(
                Method::PUT,
                &format!("/lol-perks/v1/pages/{id}"),
                Some(body),
            )
            .await?;
        return Ok(());
    }

    let owned = state
        .lcu_checked(Method::GET, "/lol-perks/v1/inventory", None)
        .await?
        .get("ownedPageCount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let editable = pages
        .iter()
        .filter(|p| p.get("isEditable").and_then(|v| v.as_bool()) != Some(false))
        .count() as i64;
    if editable >= owned {
        return Err("no free rune page slot (delete a page or rename one to \"Prowler\")".into());
    }
    state
        .lcu_checked(Method::POST, "/lol-perks/v1/pages", Some(body))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_spell_names() {
        assert_eq!(spell_id("flash"), Some(4));
        assert_eq!(spell_id("Smite"), Some(11));
        assert_eq!(spell_id("Zilean"), None);
    }

    #[test]
    fn locked_champion_requires_completed_pick_in_draft() {
        let hovering = json!({
            "localPlayerCellId": 2,
            "myTeam": [{"cellId": 2, "championId": 103}],
            "actions": [[
                {"actorCellId": 2, "type": "pick", "completed": false}
            ]]
        });
        assert_eq!(locked_champion_id(&hovering), None);

        let locked = json!({
            "localPlayerCellId": 2,
            "myTeam": [{"cellId": 2, "championId": 103}],
            "actions": [[
                {"actorCellId": 2, "type": "pick", "completed": true}
            ]]
        });
        assert_eq!(locked_champion_id(&locked), Some(103));
    }

    #[test]
    fn locked_champion_without_pick_actions_is_aram() {
        let aram = json!({
            "localPlayerCellId": 2,
            "myTeam": [{"cellId": 2, "championId": 62}],
            "actions": []
        });
        assert_eq!(locked_champion_id(&aram), Some(62));
    }

    #[test]
    fn parses_recommended_page() {
        let recommended = json!([{
            "primaryPerkStyleId": 8100,
            "secondaryPerkStyleId": 8300,
            "perks": [{"id": 8112}, {"id": 8143}, {"id": 8138}, {"id": 8135},
                      {"id": 8345}, {"id": 8347}, {"id": 5008}, {"id": 5008}, {"id": 5002}]
        }]);
        let body = recommended_to_page_body(&recommended).unwrap();
        assert_eq!(body["primaryStyleId"], 8100);
        assert_eq!(body["subStyleId"], 8300);
        assert_eq!(body["selectedPerkIds"].as_array().unwrap().len(), 9);
        assert_eq!(body["current"], true);
    }
}
