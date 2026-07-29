use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use reqwest::Method;
use serde_json::{json, Value};

use crate::config::RunePage;
use crate::modules::loadout::apply_page;
use crate::state::AppState;

const PAGE_NAME: &str = "Prowler";

const STAT_SHARD_ROWS: [[i64; 3]; 3] = [[5008, 5005, 5007], [5008, 5010, 5001], [5011, 5013, 5001]];

pub async fn rune_data(state: &AppState) -> Result<Value, String> {
    if let Some(cached) = state.rune_data.read().await.as_ref() {
        return Ok(cached.clone());
    }

    let styles = state
        .lcu_checked(Method::GET, "/lol-perks/v1/styles", None)
        .await?;
    let perks = state
        .lcu_checked(Method::GET, "/lol-perks/v1/perks", None)
        .await?;

    let mut icon_paths: Vec<String> = Vec::new();
    collect_icon_paths(&styles, &mut icon_paths);
    collect_icon_paths(&perks, &mut icon_paths);
    icon_paths.sort();
    icon_paths.dedup();

    let mut icons: HashMap<String, String> = HashMap::new();
    for path in &icon_paths {
        if let Some(url) = fetch_icon_data_url(state, path).await {
            icons.insert(path.clone(), url);
        }
    }

    let payload = build_payload(&styles, &perks, &icons);
    *state.rune_data.write().await = Some(payload.clone());
    Ok(payload)
}

fn collect_icon_paths(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == "iconPath" {
                    if let Some(s) = v.as_str() {
                        if !s.is_empty() {
                            out.push(s.to_string());
                        }
                    }
                } else {
                    collect_icon_paths(v, out);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_icon_paths(item, out);
            }
        }
        _ => {}
    }
}

async fn fetch_icon_data_url(state: &AppState, icon_path: &str) -> Option<String> {
    let endpoint = if icon_path.starts_with('/') {
        icon_path.to_string()
    } else {
        format!("/{icon_path}")
    };
    let bytes = state.lcu_bytes(&endpoint).await.ok()?;
    if bytes.is_empty() {
        return None;
    }
    let mime = if icon_path.to_ascii_lowercase().ends_with(".jpg")
        || icon_path.to_ascii_lowercase().ends_with(".jpeg")
    {
        "image/jpeg"
    } else {
        "image/png"
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

fn build_payload(styles: &Value, perks: &Value, icons: &HashMap<String, String>) -> Value {
    let icon = |v: &Value| -> String {
        v.get("iconPath")
            .and_then(|p| p.as_str())
            .and_then(|p| icons.get(p).cloned())
            .unwrap_or_default()
    };

    let mut perk_map = serde_json::Map::new();
    for p in perks.as_array().into_iter().flatten() {
        let Some(id) = p.get("id").and_then(|v| v.as_i64()) else {
            continue;
        };
        perk_map.insert(
            id.to_string(),
            json!({
                "id": id,
                "name": p.get("name").and_then(|v| v.as_str()).unwrap_or_default(),
                "desc": p.get("shortDesc").and_then(|v| v.as_str()).unwrap_or_default(),
                "icon": icon(p),
            }),
        );
    }

    let mut trees = Vec::new();
    for s in styles.as_array().into_iter().flatten() {
        let Some(id) = s.get("id").and_then(|v| v.as_i64()) else {
            continue;
        };
        if id <= 0 {
            continue;
        }
        let slots: Vec<Value> = s
            .get("slots")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|slot| {
                let slot_type = slot.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if slot_type.eq_ignore_ascii_case("kStatMod") {
                    return None;
                }
                let perks: Vec<i64> = slot
                    .get("perks")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_i64())
                    .collect();
                if perks.is_empty() {
                    return None;
                }
                Some(json!({ "type": slot_type, "perks": perks }))
            })
            .collect();
        if slots.is_empty() {
            continue;
        }
        trees.push(json!({
            "id": id,
            "name": s.get("name").and_then(|v| v.as_str()).unwrap_or_default(),
            "icon": icon(s),
            "slots": slots,
        }));
    }

    let shards = stat_shard_rows(styles);

    json!({
        "trees": trees,
        "perks": Value::Object(perk_map),
        "shards": shards,
    })
}

fn stat_shard_rows(styles: &Value) -> Vec<Vec<i64>> {
    for s in styles.as_array().into_iter().flatten() {
        let rows: Vec<Vec<i64>> = s
            .get("slots")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter(|slot| {
                slot.get("type")
                    .and_then(|v| v.as_str())
                    .is_some_and(|t| t.eq_ignore_ascii_case("kStatMod"))
            })
            .filter_map(|slot| {
                let perks: Vec<i64> = slot
                    .get("perks")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_i64())
                    .collect();
                (!perks.is_empty()).then_some(perks)
            })
            .collect();
        if rows.len() == 3 {
            return rows;
        }
    }
    STAT_SHARD_ROWS.iter().map(|r| r.to_vec()).collect()
}

fn page_body(page: &RunePage) -> Value {
    json!({
        "name": PAGE_NAME,
        "primaryStyleId": page.primary_style_id,
        "subStyleId": page.sub_style_id,
        "selectedPerkIds": page.selected_perk_ids,
        "current": true,
    })
}

#[tauri::command]
pub async fn get_rune_trees(state: tauri::State<'_, Arc<AppState>>) -> Result<Value, String> {
    rune_data(&state).await
}

#[tauri::command]
pub async fn apply_rune_page(
    state: tauri::State<'_, Arc<AppState>>,
    page: RunePage,
) -> Result<(), String> {
    if page.selected_perk_ids.is_empty() {
        return Err("This rune page has no runes selected.".into());
    }
    apply_page(&state, page_body(&page)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_body_has_perks_and_current() {
        let page = RunePage {
            name: "My Page".into(),
            primary_style_id: 8100,
            sub_style_id: 8300,
            selected_perk_ids: vec![8112, 8143, 8138, 8135, 8345, 8347, 5008, 5008, 5002],
        };
        let body = page_body(&page);
        assert_eq!(body["name"], PAGE_NAME);
        assert_eq!(body["primaryStyleId"], 8100);
        assert_eq!(body["subStyleId"], 8300);
        assert_eq!(body["selectedPerkIds"].as_array().unwrap().len(), 9);
        assert_eq!(body["current"], true);
    }

    #[test]
    fn collects_icon_paths_recursively() {
        let v = json!({
            "id": 8100,
            "iconPath": "/a/b.png",
            "slots": [{ "perks": [{ "id": 1, "iconPath": "/c/d.png" }] }]
        });
        let mut out = Vec::new();
        collect_icon_paths(&v, &mut out);
        out.sort();
        assert_eq!(out, vec!["/a/b.png".to_string(), "/c/d.png".to_string()]);
    }

    #[test]
    fn stat_shards_read_live_kstatmod_slots() {
        let styles = json!([{
            "id": 0,
            "slots": [
                { "type": "kStatMod", "perks": [5008, 5005, 5007] },
                { "type": "kStatMod", "perks": [5008, 5010, 5001] },
                { "type": "kStatMod", "perks": [5011, 5013, 5001] }
            ]
        }]);
        let rows = stat_shard_rows(&styles);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec![5008, 5005, 5007]);
    }

    #[test]
    fn stat_shards_fall_back_when_absent() {
        let rows = stat_shard_rows(&json!([{ "id": 8100, "slots": [] }]));
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows,
            STAT_SHARD_ROWS
                .iter()
                .map(|r| r.to_vec())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn build_payload_splits_trees_perks_and_shards() {
        let styles = json!([
            {
                "id": 8100,
                "name": "Domination",
                "iconPath": "/tree.png",
                "slots": [
                    { "type": "kKeyStone", "perks": [8112, 8124] },
                    { "type": "kStatMod", "perks": [5008, 5005, 5007] }
                ]
            },
            { "id": 0, "name": "None", "slots": [] }
        ]);
        let perks = json!([
            { "id": 8112, "name": "Electrocute", "shortDesc": "zap", "iconPath": "/p.png" }
        ]);
        let mut icons = HashMap::new();
        icons.insert(
            "/tree.png".to_string(),
            "data:image/png;base64,AAA".to_string(),
        );
        icons.insert(
            "/p.png".to_string(),
            "data:image/png;base64,BBB".to_string(),
        );

        let payload = build_payload(&styles, &perks, &icons);
        let trees = payload["trees"].as_array().unwrap();
        assert_eq!(trees.len(), 1);
        assert_eq!(trees[0]["name"], "Domination");
        assert_eq!(trees[0]["icon"], "data:image/png;base64,AAA");
        assert_eq!(trees[0]["slots"].as_array().unwrap().len(), 1);
        assert_eq!(payload["perks"]["8112"]["name"], "Electrocute");
        assert_eq!(
            payload["perks"]["8112"]["icon"],
            "data:image/png;base64,BBB"
        );
    }
}
