use std::sync::Arc;

use reqwest::Method;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::commands::log;
use crate::state::AppState;

#[tauri::command]
pub async fn reveal_lobby(
    app: AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let participants = state
        .riot_checked(Method::GET, "/chat/v5/participants", None)
        .await?;
    let names: Vec<String> = participants
        .get("participants")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|p| {
                    p.get("cid")
                        .and_then(|v| v.as_str())
                        .is_some_and(|cid| cid.contains("champ-select"))
                })
                .filter_map(|p| {
                    let name = p.get("game_name")?.as_str()?;
                    let tag = p.get("game_tag")?.as_str()?;
                    if name.is_empty() {
                        None
                    } else {
                        Some(format!("{name}#{tag}"))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if names.is_empty() {
        return Err("No champ-select lobby found - reveal only works during champ select.".into());
    }

    let region = state
        .lcu_checked(Method::GET, "/riotclient/region-locale", None)
        .await?
        .get("region")
        .and_then(|v| v.as_str())
        .unwrap_or("euw")
        .to_lowercase();

    let provider = state.config.read().await.lobby_reveal.provider.clone();
    let url = build_reveal_url(&provider, &region, &names);

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("failed to open browser: {e}"))?;
    log(
        &app,
        format!("Revealed {} players on {provider}.", names.len()),
    );
    Ok(url)
}

fn build_reveal_url(provider: &str, region: &str, names: &[String]) -> String {
    let joined = names.join(",");
    let encoded = urlencoding::encode(&joined);
    match provider {
        "porofessor" => format!("https://porofessor.gg/pregame/{region}/{encoded}"),
        "ugg" => {
            let platform = if region.chars().last().is_some_and(|c| c.is_ascii_digit()) {
                region.to_string()
            } else {
                format!("{region}1")
            };
            format!("https://u.gg/multisearch?summoners={encoded}&region={platform}")
        }
        _ => format!("https://op.gg/multisearch/{region}?summoners={encoded}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["Faker#KR1".into(), "Hide on bush#KR1".into()]
    }

    #[test]
    fn opgg_url() {
        assert_eq!(
            build_reveal_url("opgg", "euw", &names()),
            "https://op.gg/multisearch/euw?summoners=Faker%23KR1%2CHide%20on%20bush%23KR1"
        );
    }

    #[test]
    fn porofessor_url() {
        assert_eq!(
            build_reveal_url("porofessor", "euw", &names()),
            "https://porofessor.gg/pregame/euw/Faker%23KR1%2CHide%20on%20bush%23KR1"
        );
    }

    #[test]
    fn ugg_url_appends_platform_digit() {
        assert_eq!(
            build_reveal_url("ugg", "euw", &names()),
            "https://u.gg/multisearch?summoners=Faker%23KR1%2CHide%20on%20bush%23KR1&region=euw1"
        );
    }

    #[test]
    fn unknown_provider_falls_back_to_opgg() {
        assert!(build_reveal_url("nope", "na", &names()).starts_with("https://op.gg/"));
    }
}
