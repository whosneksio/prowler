use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::{Deserialize, Serialize};

use super::crypto;

const BUNDLE_FILE: &str = "session.bin";
const META_FILE: &str = "meta.json";

struct SnapshotItem {
    rel_path: &'static str,
    is_dir: bool,
    required: bool,
    ignored_names: &'static [&'static str],
}

const SNAPSHOT_ITEMS: &[SnapshotItem] = &[
    SnapshotItem {
        rel_path: "Riot Client/Data/RiotGamesPrivateSettings.yaml",
        is_dir: false,
        required: true,
        ignored_names: &[],
    },
    SnapshotItem {
        rel_path: "Riot Client/Data/Sessions",
        is_dir: true,
        required: false,
        ignored_names: &[],
    },
    SnapshotItem {
        rel_path: "Riot Client/Config",
        is_dir: true,
        required: false,
        ignored_names: &["lockfile"],
    },
    SnapshotItem {
        rel_path: "League of Legends/Data/RiotGamesPrivateSettings.yaml",
        is_dir: false,
        required: false,
        ignored_names: &[],
    },
];

const BUNDLE_V2: u32 = 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
    #[serde(default)]
    pub puuid: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub profile_icon_id: i64,
    #[serde(default)]
    pub created_ms: u64,
}

#[derive(Serialize, Deserialize)]
struct SessionBundle {
    version: u32,
    files: Vec<BundleFile>,
}

#[derive(Serialize, Deserialize)]
struct BundleFile {
    rel_path: String,
    data_b64: String,
}

fn riot_games_dir() -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "LOCALAPPDATA environment variable not set".to_string())?;
    Ok(PathBuf::from(local).join("Riot Games"))
}

fn item_live_path(root: &Path, item: &SnapshotItem) -> PathBuf {
    root.join(item.rel_path.replace('/', "\\"))
}

fn account_dir(vault_dir: &Path, id: &str) -> PathBuf {
    vault_dir.join(id)
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn live_session_has_tokens() -> bool {
    let Ok(root) = riot_games_dir() else {
        return false;
    };
    let settings = item_live_path(&root, &SNAPSHOT_ITEMS[0]);
    match fs::read_to_string(&settings) {
        Ok(contents) => yaml_has_auth_tokens(&contents),
        Err(_) => false,
    }
}

fn capture_session() -> Result<SessionBundle, String> {
    let root = riot_games_dir()?;

    let settings = item_live_path(&root, &SNAPSHOT_ITEMS[0]);
    let contents = fs::read_to_string(&settings).map_err(|_| {
        "No saved Riot session found. Log in to the Riot Client with \
         \"Stay signed in\" checked, then save again."
            .to_string()
    })?;
    if !yaml_has_auth_tokens(&contents) {
        return Err(
            "The Riot session on disk has no persistent login tokens. Make sure \
             \"Stay signed in\" was checked when you logged in, then try again."
                .into(),
        );
    }

    let mut files = Vec::new();
    for item in SNAPSHOT_ITEMS {
        let path = item_live_path(&root, item);
        if item.is_dir {
            if path.is_dir() {
                collect_dir(&mut files, &root, &path, item.ignored_names)?;
            }
        } else if path.exists() {
            push_file(&mut files, &root, &path)?;
        } else if item.required {
            return Err(format!("required session file missing: {}", path.display()));
        }
    }

    Ok(SessionBundle {
        version: BUNDLE_V2,
        files,
    })
}

fn collect_dir(
    files: &mut Vec<BundleFile>,
    root: &Path,
    dir: &Path,
    ignored_names: &[&str],
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if ignored_names.iter().any(|i| i.eq_ignore_ascii_case(&name)) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_dir(files, root, &path, ignored_names)?;
        } else {
            push_file(files, root, &path)?;
        }
    }
    Ok(())
}

fn push_file(files: &mut Vec<BundleFile>, root: &Path, path: &Path) -> Result<(), String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("{} escapes session root", path.display()))?;
    let data = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    files.push(BundleFile {
        rel_path: rel.to_string_lossy().replace('\\', "/"),
        data_b64: base64::engine::general_purpose::STANDARD.encode(data),
    });
    Ok(())
}

pub fn save_account(vault_dir: &Path, meta: &AccountMeta) -> Result<(), String> {
    let bundle = capture_session()?;
    let json = serde_json::to_vec(&bundle).map_err(|e| e.to_string())?;
    let encrypted = crypto::protect(&json)?;

    let dir = account_dir(vault_dir, &meta.id);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create vault dir: {e}"))?;
    fs::write(dir.join(BUNDLE_FILE), encrypted).map_err(|e| format!("cannot write bundle: {e}"))?;
    save_meta(vault_dir, meta)
}

pub fn restore_session(vault_dir: &Path, id: &str) -> Result<(), String> {
    let encrypted = fs::read(account_dir(vault_dir, id).join(BUNDLE_FILE))
        .map_err(|e| format!("cannot read session bundle: {e}"))?;
    let json = crypto::unprotect(&encrypted)?;
    let bundle: SessionBundle =
        serde_json::from_slice(&json).map_err(|e| format!("corrupt session bundle: {e}"))?;

    let root = riot_games_dir()?;

    let clear_count = if bundle.version >= BUNDLE_V2 {
        SNAPSHOT_ITEMS.len()
    } else {
        2
    };
    for item in &SNAPSHOT_ITEMS[..clear_count] {
        let path = item_live_path(&root, item);
        if item.is_dir {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }

    for file in &bundle.files {
        let rel_path = if bundle.version >= BUNDLE_V2 {
            file.rel_path.clone()
        } else {
            format!("Riot Client/Data/{}", file.rel_path)
        };
        let rel = Path::new(&rel_path);
        if rel.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        }) {
            return Err(format!("unsafe path in bundle: {}", file.rel_path));
        }
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let data = base64::engine::general_purpose::STANDARD
            .decode(&file.data_b64)
            .map_err(|e| format!("corrupt bundle data: {e}"))?;
        fs::write(&target, data).map_err(|e| format!("cannot write {}: {e}", target.display()))?;
    }
    Ok(())
}

pub fn load_meta(vault_dir: &Path, id: &str) -> Result<AccountMeta, String> {
    let text = fs::read_to_string(account_dir(vault_dir, id).join(META_FILE))
        .map_err(|e| format!("cannot read account metadata: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("corrupt account metadata: {e}"))
}

pub fn save_meta(vault_dir: &Path, meta: &AccountMeta) -> Result<(), String> {
    let dir = account_dir(vault_dir, &meta.id);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create vault dir: {e}"))?;
    let json = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(dir.join(META_FILE), json).map_err(|e| format!("cannot write metadata: {e}"))
}

pub fn list_accounts(vault_dir: &Path) -> Result<Vec<AccountMeta>, String> {
    let mut accounts = Vec::new();
    let Ok(entries) = fs::read_dir(vault_dir) else {
        return Ok(accounts);
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(meta) = load_meta(vault_dir, &entry.file_name().to_string_lossy()) {
            accounts.push(meta);
        }
    }
    accounts.sort_by_key(|a| a.created_ms);
    Ok(accounts)
}

pub fn delete_account(vault_dir: &Path, id: &str) -> Result<(), String> {
    fs::remove_dir_all(account_dir(vault_dir, id))
        .map_err(|e| format!("cannot delete account: {e}"))
}

fn yaml_has_auth_tokens(contents: &str) -> bool {
    fn value_for_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        let rest = line.strip_prefix(key)?;
        let rest = rest.strip_prefix(':')?;
        Some(rest.trim())
    }

    fn strip_yaml_comment(value: &str) -> &str {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;
        let mut previous_was_whitespace = true;

        for (index, ch) in value.char_indices() {
            if in_double_quote && escaped {
                escaped = false;
                continue;
            }
            if in_double_quote && ch == '\\' {
                escaped = true;
                continue;
            }
            match ch {
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '#' if !in_single_quote
                    && !in_double_quote
                    && (index == 0 || previous_was_whitespace) =>
                {
                    return value[..index].trim_end();
                }
                _ => {}
            }
            previous_was_whitespace = ch.is_whitespace();
        }
        value.trim_end()
    }

    fn normalized_yaml_value(value: &str) -> &str {
        strip_yaml_comment(value.trim()).trim()
    }

    fn is_empty_yaml_value(value: &str) -> bool {
        let value = normalized_yaml_value(value);
        value.is_empty()
            || value == "{}"
            || value == "[]"
            || value == "''"
            || value == "\"\""
            || value.eq_ignore_ascii_case("null")
            || value == "~"
    }

    #[derive(Default)]
    struct CookieEntry {
        indent: usize,
        is_ssid: bool,
        has_value: bool,
    }

    fn update_cookie_entry(entry: &mut CookieEntry, line: &str) {
        if let Some(value) = value_for_key(line, "name") {
            entry.is_ssid =
                normalized_yaml_value(value).trim_matches(|ch| ch == '"' || ch == '\'') == "ssid";
        }
        if let Some(value) = value_for_key(line, "value") {
            entry.has_value = !is_empty_yaml_value(value);
        }
    }

    let mut has_private = false;
    let mut has_sessions = false;
    let mut has_token = false;
    let mut cookie_entry: Option<CookieEntry> = None;
    let mut pending_sessions_indent: Option<usize> = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        let meaningful = !trimmed.is_empty() && !trimmed.starts_with('#');

        if let Some(sessions_indent) = pending_sessions_indent {
            if meaningful {
                if indent > sessions_indent {
                    has_sessions = true;
                }
                pending_sessions_indent = None;
            }
        }

        if let Some(after_dash) = trimmed.strip_prefix('-') {
            if let Some(previous) = cookie_entry.take() {
                has_token |= previous.is_ssid && previous.has_value;
            }
            let mut entry = CookieEntry {
                indent,
                ..Default::default()
            };
            update_cookie_entry(&mut entry, after_dash.trim());
            cookie_entry = Some(entry);
        } else if let Some(entry) = cookie_entry.as_mut() {
            if meaningful && indent <= entry.indent {
                let previous = cookie_entry.take().expect("cookie entry exists");
                has_token |= previous.is_ssid && previous.has_value;
            } else if meaningful {
                update_cookie_entry(entry, trimmed);
            }
        }
        if let Some(value) = value_for_key(trimmed, "private") {
            if !is_empty_yaml_value(value) {
                has_private = true;
            }
        }
        if let Some(value) = value_for_key(trimmed, "sessions") {
            let value = normalized_yaml_value(value);
            if value.is_empty() {
                pending_sessions_indent = Some(indent);
            } else if !is_empty_yaml_value(value) {
                has_sessions = true;
            }
        }
        for key in ["access_token", "refresh_token", "id_token"] {
            if value_for_key(trimmed, key).is_some_and(|value| !is_empty_yaml_value(value)) {
                has_token = true;
            }
        }
    }
    if let Some(entry) = cookie_entry {
        has_token |= entry.is_ssid && entry.has_value;
    }

    has_private || has_sessions || has_token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_roundtrip() {
        let dir = std::env::temp_dir().join("prowler-vault-test");
        let _ = fs::remove_dir_all(&dir);
        let meta = AccountMeta {
            id: "acct-1".into(),
            label: "Main".into(),
            game_name: "Player".into(),
            tag_line: "EUW".into(),
            puuid: "abc".into(),
            region: "EUW".into(),
            profile_icon_id: 29,
            created_ms: 1,
        };
        save_meta(&dir, &meta).unwrap();
        let listed = list_accounts(&dir).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].label, "Main");
        delete_account(&dir, "acct-1").unwrap();
        assert!(list_accounts(&dir).unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_private_and_sessions_are_not_ready() {
        let yaml = "install:\n  private: ''\n  sessions: {}\n";
        assert!(!yaml_has_auth_tokens(yaml));
        assert!(!yaml_has_auth_tokens("private:\nsessions:\n"));
    }

    #[test]
    fn non_empty_private_blob_is_ready() {
        assert!(yaml_has_auth_tokens("private: dGhpcyBpcyBhIHRva2Vu\n"));
    }

    #[test]
    fn populated_sessions_map_is_ready() {
        let yaml = "sessions:\n  abc-123:\n    token: xyz\n";
        assert!(yaml_has_auth_tokens(yaml));
    }

    #[test]
    fn token_entries_are_ready() {
        assert!(yaml_has_auth_tokens("refresh_token: abc.def.ghi\n"));
        assert!(!yaml_has_auth_tokens("refresh_token: ''\n"));
    }

    #[test]
    fn cookie_format_with_ssid_is_ready() {
        let yaml = concat!(
            "riot-login:\n",
            "  persist:\n",
            "    session:\n",
            "      cookies:\n",
            "        - domain: auth.riotgames.com\n",
            "          name: ssid\n",
            "          value: eyJhbGciOi\n",
        );
        assert!(yaml_has_auth_tokens(yaml));
    }

    #[test]
    fn cookie_format_with_only_tracking_cookies_is_not_ready() {
        let yaml = concat!(
            "riot-login:\n",
            "  persist:\n",
            "    session:\n",
            "      cookies:\n",
            "        - domain: riotgames.com\n",
            "          name: tdid\n",
            "          value: some-tracking-id\n",
        );
        assert!(!yaml_has_auth_tokens(yaml));
    }

    #[test]
    fn ssid_cookie_requires_a_non_empty_value_in_the_same_entry() {
        let yaml = concat!(
            "cookies:\n",
            "  - name: ssid\n",
            "    value: ''\n",
            "  - name: tdid\n",
            "    value: real-value\n",
        );
        assert!(!yaml_has_auth_tokens(yaml));
    }

    #[test]
    fn ssid_cookie_accepts_value_before_name() {
        let yaml = concat!("cookies:\n", "  - value: abc\n", "    name: ssid\n");
        assert!(yaml_has_auth_tokens(yaml));
    }

    #[test]
    fn keys_that_merely_start_with_private_do_not_match() {
        assert!(!yaml_has_auth_tokens("privateKey: abc\nsessionsCount: 3\n"));
    }
}
