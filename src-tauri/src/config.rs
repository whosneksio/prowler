use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct RoleChampions {
    pub default: Vec<String>,
    pub top: Vec<String>,
    pub jungle: Vec<String>,
    pub middle: Vec<String>,
    pub bottom: Vec<String>,
    pub utility: Vec<String>,
}

impl RoleChampions {
    pub fn for_role(&self, role: Option<&str>) -> &[String] {
        let list = match role {
            Some("TOP") => &self.top,
            Some("JUNGLE") => &self.jungle,
            Some("MIDDLE") => &self.middle,
            Some("BOTTOM") => &self.bottom,
            Some("UTILITY") => &self.utility,
            _ => &self.default,
        };
        if list.is_empty() {
            &self.default
        } else {
            list
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct DelayRange {
    pub min: f64,
    pub max: f64,
}

impl DelayRange {
    pub fn fixed(v: f64) -> Self {
        Self { min: v, max: v }
    }

    pub fn sample(&self) -> f64 {
        let lo = self.min.max(0.0);
        let hi = self.max.max(lo);
        if hi <= lo {
            return lo;
        }
        let frac = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as f64
            / 1_000_000_000.0;
        lo + frac * (hi - lo)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct InstalockCfg {
    pub enabled: bool,
    pub prepick: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion: Option<String>,
    pub champions: RoleChampions,
    pub delay: DelayRange,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<f64>,
}

impl Default for InstalockCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            prepick: false,
            champion: None,
            champions: RoleChampions {
                default: vec!["Random".into()],
                ..RoleChampions::default()
            },
            delay: DelayRange::fixed(0.3),
            delay_seconds: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AutobanCfg {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion: Option<String>,
    pub champions: RoleChampions,
    pub delay: DelayRange,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<f64>,
}

impl Default for AutobanCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            champion: None,
            champions: RoleChampions::default(),
            delay: DelayRange::fixed(0.3),
            delay_seconds: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AutoAcceptCfg {
    pub enabled: bool,
    pub delay: DelayRange,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<f64>,
}

impl Default for AutoAcceptCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            delay: DelayRange::fixed(0.0),
            delay_seconds: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LobbyRevealCfg {
    pub provider: String,
}

impl Default for LobbyRevealCfg {
    fn default() -> Self {
        Self {
            provider: "opgg".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct RoleSpells {
    pub default: [String; 2],
    pub top: [String; 2],
    pub jungle: [String; 2],
    pub middle: [String; 2],
    pub bottom: [String; 2],
    pub utility: [String; 2],
}

impl Default for RoleSpells {
    fn default() -> Self {
        let pair = |a: &str, b: &str| [a.to_string(), b.to_string()];
        Self {
            default: pair("Flash", "Ignite"),
            top: pair("Flash", "Teleport"),
            jungle: pair("Flash", "Smite"),
            middle: pair("Flash", "Ignite"),
            bottom: pair("Flash", "Heal"),
            utility: pair("Flash", "Ignite"),
        }
    }
}

impl RoleSpells {
    pub fn for_role(&self, role: Option<&str>) -> &[String; 2] {
        match role {
            Some("TOP") => &self.top,
            Some("JUNGLE") => &self.jungle,
            Some("MIDDLE") => &self.middle,
            Some("BOTTOM") => &self.bottom,
            Some("UTILITY") => &self.utility,
            _ => &self.default,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct AutoRunesCfg {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct AutoSpellsCfg {
    pub enabled: bool,
    pub roles: RoleSpells,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct RunePage {
    pub name: String,
    pub primary_style_id: i64,
    pub sub_style_id: i64,
    pub selected_perk_ids: Vec<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct CustomRunesCfg {
    pub pages: Vec<RunePage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UiCfg {
    pub show_username: bool,
}

impl Default for UiCfg {
    fn default() -> Self {
        Self {
            show_username: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UpdatesCfg {
    pub auto_check: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_version: Option<String>,
}

impl Default for UpdatesCfg {
    fn default() -> Self {
        Self {
            auto_check: true,
            skipped_version: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub instalock: InstalockCfg,
    #[serde(default)]
    pub autoban: AutobanCfg,
    #[serde(default)]
    pub auto_accept: AutoAcceptCfg,
    #[serde(default)]
    pub lobby_reveal: LobbyRevealCfg,
    #[serde(default)]
    pub auto_runes: AutoRunesCfg,
    #[serde(default)]
    pub auto_spells: AutoSpellsCfg,
    #[serde(default)]
    pub custom_runes: CustomRunesCfg,
    #[serde(default)]
    pub ui: UiCfg,
    #[serde(default)]
    pub updates: UpdatesCfg,
}

impl Config {
    pub fn load_or_default(path: &PathBuf) -> Self {
        let mut cfg: Config = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        migrate_legacy(
            &mut cfg.instalock.champion,
            &mut cfg.instalock.champions,
            &InstalockCfg::default().champions,
        );
        migrate_legacy(
            &mut cfg.autoban.champion,
            &mut cfg.autoban.champions,
            &AutobanCfg::default().champions,
        );
        migrate_delay(&mut cfg.instalock.delay_seconds, &mut cfg.instalock.delay);
        migrate_delay(&mut cfg.autoban.delay_seconds, &mut cfg.autoban.delay);
        migrate_delay(
            &mut cfg.auto_accept.delay_seconds,
            &mut cfg.auto_accept.delay,
        );
        cfg
    }

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into());
        std::fs::write(path, json)
    }
}

fn migrate_delay(legacy: &mut Option<f64>, delay: &mut DelayRange) {
    if let Some(v) = legacy.take() {
        *delay = DelayRange::fixed(v);
    }
}

fn migrate_legacy(
    champion: &mut Option<String>,
    lists: &mut RoleChampions,
    untouched: &RoleChampions,
) {
    if let Some(name) = champion.take() {
        let name = name.trim().to_string();
        if lists != untouched {
            return;
        }
        if name.is_empty() || name.eq_ignore_ascii_case("none") {
            *lists = RoleChampions::default();
        } else {
            *lists = RoleChampions {
                default: vec![name],
                ..RoleChampions::default()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_role_prefers_role_then_default() {
        let lists = RoleChampions {
            default: vec!["Ahri".into()],
            jungle: vec!["Vi".into(), "Nocturne".into()],
            ..RoleChampions::default()
        };
        assert_eq!(lists.for_role(Some("JUNGLE")), ["Vi", "Nocturne"]);
        assert_eq!(lists.for_role(Some("TOP")), ["Ahri"]);
        assert_eq!(lists.for_role(None), ["Ahri"]);
        assert_eq!(lists.for_role(Some("BENCH")), ["Ahri"]);
    }

    #[test]
    fn migrates_legacy_config() {
        let json = r#"{
            "instalock": {"enabled": true, "champion": "Ahri", "delay_seconds": 0.5},
            "autoban": {"enabled": true, "champion": "None", "delay_seconds": 0.3}
        }"#;
        let mut cfg: Config = serde_json::from_str(json).unwrap();
        migrate_legacy(
            &mut cfg.instalock.champion,
            &mut cfg.instalock.champions,
            &InstalockCfg::default().champions,
        );
        migrate_legacy(
            &mut cfg.autoban.champion,
            &mut cfg.autoban.champions,
            &AutobanCfg::default().champions,
        );
        migrate_delay(&mut cfg.instalock.delay_seconds, &mut cfg.instalock.delay);
        migrate_delay(&mut cfg.autoban.delay_seconds, &mut cfg.autoban.delay);

        assert!(cfg.instalock.enabled);
        assert_eq!(cfg.instalock.delay, DelayRange::fixed(0.5));
        assert!(cfg.instalock.delay_seconds.is_none());
        assert_eq!(cfg.instalock.champions.default, vec!["Ahri".to_string()]);
        assert!(cfg.instalock.champion.is_none());

        assert_eq!(cfg.autoban.champions, RoleChampions::default());
        assert!(cfg.autoban.champion.is_none());

        let out = serde_json::to_string(&cfg).unwrap();
        assert!(!out.contains("\"champion\""));
        assert!(out.contains("\"champions\""));
    }

    #[test]
    fn new_fields_default_on_old_config() {
        let cfg: Config =
            serde_json::from_str(r#"{"lobby_reveal": {"provider": "opgg"}}"#).unwrap();
        assert!(!cfg.auto_runes.enabled);
        assert!(!cfg.auto_spells.enabled);
        assert_eq!(cfg.auto_spells.roles.for_role(Some("JUNGLE"))[1], "Smite");
        assert_eq!(cfg.instalock.champions.default, vec!["Random".to_string()]);
        assert!(cfg.updates.auto_check);
        assert!(cfg.updates.skipped_version.is_none());
    }
}
