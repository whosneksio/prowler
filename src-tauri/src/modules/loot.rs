use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::commands::log;
use crate::state::AppState;

/// Grants that need no player choice: (grant id, select body).
/// Grants requiring a selection (e.g. "pick 1 of 3") are counted as skipped.
fn grant_selects(grants: &Value) -> (Vec<(String, Value)>, usize) {
    let mut auto = Vec::new();
    let mut skipped = 0;
    for g in grants.as_array().map(|a| a.as_slice()).unwrap_or_default() {
        let info = g.get("info").unwrap_or(g);
        let Some(id) = info.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let group = g.get("rewardGroup").unwrap_or(&Value::Null);
        let group_id = group
            .get("id")
            .and_then(|v| v.as_str())
            .or_else(|| info.get("rewardGroupId").and_then(|v| v.as_str()))
            .unwrap_or_default();
        let needs_choice = group
            .get("selectionStrategyConfig")
            .and_then(|c| c.get("minSelectionsAllowed"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0;
        if needs_choice {
            skipped += 1;
        } else {
            auto.push((
                id.to_string(),
                json!({ "grantId": id, "rewardGroupId": group_id, "selections": [] }),
            ));
        }
    }
    (auto, skipped)
}

/// Pluck ids (string or numeric) from an array of objects.
fn ids(list: &Value, key: &str) -> Vec<String> {
    list.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| match e.get(key) {
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(Value::Number(n)) => Some(n.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Events with unclaimed rewards: (event id, unclaimed count).
/// claim-all returns 204 even when there is nothing to claim, so we must
/// filter on unclaimedRewardCount instead of trusting the POST status.
fn claimable_events(events: &Value) -> Vec<(String, u32)> {
    events
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let id = e.get("eventId").and_then(|v| v.as_str())?;
                    let unclaimed = e
                        .get("eventInfo")
                        .and_then(|i| i.get("unclaimedRewardCount"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    (unclaimed > 0).then(|| (id.to_string(), unclaimed as u32))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub async fn claim_all_rewards(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<u32, String> {
    let mut claimed: u32 = 0;

    // 1. Reward grants awaiting selection (level-up capsules etc.).
    // Valid LolRewardsGrantStatus values: PENDING_SELECTION, PENDING_FULFILLMENT,
    // FULFILLED, FAILED (verified live via /Help?target=LolRewardsGrantStatus).
    if let Ok(grants) = state
        .lcu_checked(
            Method::GET,
            "/lol-rewards/v1/grants?status=PENDING_SELECTION",
            None,
        )
        .await
    {
        let (auto, skipped) = grant_selects(&grants);
        for (id, body) in auto {
            let endpoint = format!("/lol-rewards/v1/grants/{id}/select");
            if state
                .lcu_checked(Method::POST, &endpoint, Some(body))
                .await
                .is_ok()
            {
                claimed += 1;
            }
        }
        if skipped > 0 {
            log(
                &app,
                format!("Skipped {skipped} reward grant(s) that need a manual choice."),
            );
        }
    }

    // 2. Loot milestones (empty [] when the account has none).
    if let Ok(milestones) = state
        .lcu_checked(Method::GET, "/lol-loot/v1/milestones", None)
        .await
    {
        for id in ids(&milestones, "id") {
            let endpoint = format!("/lol-loot/v1/milestones/{id}/claim");
            if state
                .lcu_checked(Method::POST, &endpoint, None)
                .await
                .is_ok()
            {
                claimed += 1;
            }
        }
    }

    // 3. Event pass reward tracks. claim-all answers 204 even with nothing to
    // claim, so only POST where unclaimedRewardCount > 0 and count that number.
    if let Ok(events) = state
        .lcu_checked(Method::GET, "/lol-event-hub/v1/events", None)
        .await
    {
        for (id, unclaimed) in claimable_events(&events) {
            let endpoint = format!("/lol-event-hub/v1/events/{id}/reward-track/claim-all");
            if state
                .lcu_checked(Method::POST, &endpoint, None)
                .await
                .is_ok()
            {
                claimed += unclaimed;
            }
        }
    }

    if claimed > 0 {
        log(&app, format!("Claimed {claimed} reward(s)."));
    } else {
        log(&app, "No rewards to claim.");
    }
    Ok(claimed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_auto_grants_and_skips_choices() {
        let grants = json!([
            {
                "info": { "id": "g1" },
                "rewardGroup": {
                    "id": "rg1",
                    "selectionStrategyConfig": { "minSelectionsAllowed": 0, "maxSelectionsAllowed": 0 }
                }
            },
            {
                "info": { "id": "g2" },
                "rewardGroup": {
                    "id": "rg2",
                    "selectionStrategyConfig": { "minSelectionsAllowed": 1, "maxSelectionsAllowed": 1 }
                }
            },
            { "info": { "id": "g3", "rewardGroupId": "rg3" } }
        ]);
        let (auto, skipped) = grant_selects(&grants);
        assert_eq!(skipped, 1);
        assert_eq!(auto.len(), 2);
        assert_eq!(auto[0].0, "g1");
        assert_eq!(auto[0].1["rewardGroupId"], "rg1");
        assert_eq!(auto[1].1["rewardGroupId"], "rg3");
        assert_eq!(auto[0].1["selections"], json!([]));
    }

    #[test]
    fn only_events_with_unclaimed_rewards_are_claimable() {
        let events = json!([
            { "eventId": "e1", "eventInfo": { "unclaimedRewardCount": 0 } },
            { "eventId": "e2", "eventInfo": { "unclaimedRewardCount": 3 } },
            { "eventId": "e3" }
        ]);
        assert_eq!(claimable_events(&events), vec![("e2".to_string(), 3)]);
        assert!(claimable_events(&json!([])).is_empty());
    }

    #[test]
    fn plucks_string_and_numeric_ids() {
        let list = json!([{ "id": "abc" }, { "id": 42 }, { "name": "no-id" }]);
        assert_eq!(ids(&list, "id"), vec!["abc".to_string(), "42".to_string()]);
        assert!(ids(&json!({}), "id").is_empty());
    }
}
