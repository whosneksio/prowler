use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSummoner {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
    #[serde(default)]
    pub summoner_id: i64,
    #[serde(default)]
    pub account_id: i64,
    #[serde(default)]
    pub profile_icon_id: i64,
    #[serde(default)]
    pub summoner_level: i64,
    #[serde(default)]
    pub puuid: String,
}

impl CurrentSummoner {
    pub fn riot_id(&self) -> String {
        if !self.game_name.is_empty() && !self.tag_line.is_empty() {
            format!("{}#{}", self.game_name, self.tag_line)
        } else {
            self.display_name.clone()
        }
    }
}
