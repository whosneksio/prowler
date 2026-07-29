use std::ffi::OsString;
use sysinfo::System;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LcuCredentials {
    pub port: u16,
    pub token: String,
}

const LEAGUE_UX_PROCESS: &str = "LeagueClientUx.exe";
const RIOT_PROCESSES: [&str; 2] = ["RiotClientUx.exe", "RiotClientServices.exe"];

const LEAGUE_PORT_ARG: &str = "--app-port=";
const LEAGUE_TOKEN_ARG: &str = "--remoting-auth-token=";
const RIOT_PORT_ARG: &str = "--riotclient-app-port=";
const RIOT_TOKEN_ARG: &str = "--riotclient-auth-token=";

pub fn find_league_credentials() -> Option<LcuCredentials> {
    scan(&[LEAGUE_UX_PROCESS], LEAGUE_PORT_ARG, LEAGUE_TOKEN_ARG)
}

pub fn find_riot_credentials() -> Option<LcuCredentials> {
    scan(
        &[LEAGUE_UX_PROCESS, RIOT_PROCESSES[0], RIOT_PROCESSES[1]],
        RIOT_PORT_ARG,
        RIOT_TOKEN_ARG,
    )
}

fn scan(process_names: &[&str], port_arg: &str, token_arg: &str) -> Option<LcuCredentials> {
    let sys = System::new_all();
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy();
        if !process_names.iter().any(|p| name.eq_ignore_ascii_case(p)) {
            continue;
        }
        let cmd = process.cmd();
        let port = extract_arg(cmd, port_arg).and_then(|p| p.parse::<u16>().ok());
        let token = extract_arg(cmd, token_arg);
        if let (Some(port), Some(token)) = (port, token) {
            return Some(LcuCredentials { port, token });
        }
    }
    None
}

fn extract_arg(cmd: &[OsString], prefix: &str) -> Option<String> {
    for arg in cmd {
        let s = arg.to_string_lossy();
        if let Some(rest) = s.strip_prefix(prefix) {
            return Some(rest.to_string());
        }
    }
    None
}
