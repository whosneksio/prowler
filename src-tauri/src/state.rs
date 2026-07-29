use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use reqwest::Method;
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::lcu::client::{LcuClient, LcuError, LcuResponse};
use crate::lcu::connector::{self, LcuCredentials};
use crate::lcu::ws::WsEvent;
use crate::modules::champ_select::Champion;

pub struct AppState {
    pub client: LcuClient,
    pub creds: RwLock<Option<LcuCredentials>>,
    pub riot_creds: RwLock<Option<LcuCredentials>>,
    pub config: RwLock<Config>,
    pub config_path: PathBuf,
    pub vault_dir: PathBuf,
    pub ws_events: broadcast::Sender<WsEvent>,
    pub ws_connected: AtomicBool,
    pub automations: RwLock<HashMap<&'static str, CancellationToken>>,
    pub champions: RwLock<Option<Vec<Champion>>>,
    pub rune_data: RwLock<Option<serde_json::Value>>,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf, vault_dir: PathBuf) -> Self {
        let (ws_events, _) = broadcast::channel(256);
        Self {
            client: LcuClient::new(),
            creds: RwLock::new(None),
            riot_creds: RwLock::new(None),
            config: RwLock::new(config),
            config_path,
            vault_dir,
            ws_events,
            ws_connected: AtomicBool::new(false),
            automations: RwLock::new(HashMap::new()),
            champions: RwLock::new(None),
            rune_data: RwLock::new(None),
        }
    }

    pub fn ws_live(&self) -> bool {
        self.ws_connected.load(Ordering::Relaxed)
    }

    async fn ensure_creds(&self) -> Option<LcuCredentials> {
        if let Some(c) = self.creds.read().await.as_ref() {
            return Some(c.clone());
        }
        self.refresh_creds().await
    }

    pub async fn refresh_creds(&self) -> Option<LcuCredentials> {
        let found = connector::find_league_credentials();
        *self.creds.write().await = found.clone();
        found
    }

    pub async fn lcu(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<LcuResponse, LcuError> {
        let creds = self.ensure_creds().await.ok_or(LcuError::NotRunning)?;
        match self
            .client
            .request(&creds, method.clone(), endpoint, body.clone())
            .await
        {
            Ok(resp) => Ok(resp),
            Err(LcuError::Transport(_)) => {
                let creds = self.refresh_creds().await.ok_or(LcuError::NotRunning)?;
                self.client.request(&creds, method, endpoint, body).await
            }
            Err(e) => Err(e),
        }
    }

    async fn ensure_riot_creds(&self) -> Option<LcuCredentials> {
        if let Some(c) = self.riot_creds.read().await.as_ref() {
            return Some(c.clone());
        }
        self.refresh_riot_creds().await
    }

    pub async fn refresh_riot_creds(&self) -> Option<LcuCredentials> {
        let found = connector::find_riot_credentials();
        *self.riot_creds.write().await = found.clone();
        found
    }

    pub async fn riot(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<LcuResponse, LcuError> {
        let creds = self.ensure_riot_creds().await.ok_or(LcuError::NotRunning)?;
        match self
            .client
            .request(&creds, method.clone(), endpoint, body.clone())
            .await
        {
            Ok(resp) => Ok(resp),
            Err(LcuError::Transport(_)) => {
                let creds = self
                    .refresh_riot_creds()
                    .await
                    .ok_or(LcuError::NotRunning)?;
                self.client.request(&creds, method, endpoint, body).await
            }
            Err(e) => Err(e),
        }
    }

    pub async fn riot_checked(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let resp = self
            .riot(method, endpoint, body)
            .await
            .map_err(|e| e.to_string())?;
        if resp.ok() {
            Ok(resp.body)
        } else {
            Err(format!("Riot client returned HTTP {}", resp.status))
        }
    }

    pub async fn lcu_checked(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let resp = self
            .lcu(method, endpoint, body)
            .await
            .map_err(|e| e.to_string())?;
        if resp.ok() {
            Ok(resp.body)
        } else {
            Err(format!("LCU returned HTTP {}", resp.status))
        }
    }

    pub async fn persist_config(&self) {
        let cfg = self.config.read().await.clone();
        let _ = cfg.save(&self.config_path);
    }

    pub async fn lcu_bytes(&self, endpoint: &str) -> Result<Vec<u8>, String> {
        let creds = self.ensure_creds().await.ok_or("client not running")?;
        let (status, bytes) = match self.client.request_bytes(&creds, endpoint).await {
            Ok(v) => v,
            Err(LcuError::Transport(_)) => {
                let creds = self.refresh_creds().await.ok_or("client not running")?;
                self.client
                    .request_bytes(&creds, endpoint)
                    .await
                    .map_err(|e| e.to_string())?
            }
            Err(e) => return Err(e.to_string()),
        };
        if (200..300).contains(&status) {
            Ok(bytes)
        } else {
            Err(format!("LCU returned HTTP {status}"))
        }
    }
}
