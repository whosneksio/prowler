use std::path::PathBuf;

pub fn find_riot_client() -> Result<PathBuf, String> {
    let program_data = std::env::var("ProgramData")
        .map_err(|_| "ProgramData environment variable not set".to_string())?;
    let installs_path = PathBuf::from(program_data)
        .join("Riot Games")
        .join("RiotClientInstalls.json");

    let text = std::fs::read_to_string(&installs_path)
        .map_err(|e| format!("cannot read {}: {e}", installs_path.display()))?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("invalid RiotClientInstalls.json: {e}"))?;

    for key in ["rc_default", "rc_live", "rc_beta"] {
        if let Some(path) = json.get(key).and_then(|v| v.as_str()) {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
    }
    Err("RiotClientServices.exe not found - is the Riot Client installed?".into())
}

pub fn launch_league() -> Result<(), String> {
    let exe = find_riot_client()?;
    std::process::Command::new(&exe)
        .args([
            "--launch-product=league_of_legends",
            "--launch-patchline=live",
        ])
        .spawn()
        .map_err(|e| format!("failed to launch {}: {e}", exe.display()))?;
    Ok(())
}
