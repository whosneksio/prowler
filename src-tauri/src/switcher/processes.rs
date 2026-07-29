use std::time::{Duration, Instant};

use sysinfo::System;

const RIOT_PROCESSES: [&str; 7] = [
    "RiotClientServices.exe",
    "RiotClientUx.exe",
    "RiotClientUxRender.exe",
    "RiotClientCrashHandler.exe",
    "LeagueClient.exe",
    "LeagueClientUx.exe",
    "LeagueClientUxRender.exe",
];

fn is_riot_process(name: &str) -> bool {
    RIOT_PROCESSES.iter().any(|p| name.eq_ignore_ascii_case(p))
}

fn running(sys: &System) -> usize {
    sys.processes()
        .values()
        .filter(|p| is_riot_process(&p.name().to_string_lossy()))
        .count()
}

pub fn kill_riot_processes() -> usize {
    let sys = System::new_all();
    let mut killed = 0;
    for process in sys.processes().values() {
        if is_riot_process(&process.name().to_string_lossy()) && process.kill() {
            killed += 1;
        }
    }
    killed
}

pub fn force_close(timeout: Duration) -> bool {
    for _ in 0..4 {
        kill_riot_processes();
        if running(&System::new_all()) == 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(450));
    }
    wait_until_closed(timeout)
}

pub fn wait_until_closed(timeout: Duration) -> bool {
    let start = Instant::now();
    loop {
        if running(&System::new_all()) == 0 {
            return true;
        }
        if start.elapsed() >= timeout {
            return false;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
}
