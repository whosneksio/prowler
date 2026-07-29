use base64::Engine;
use reqwest::{Client, Method};
use serde_json::Value;

use super::connector::LcuCredentials;

#[derive(Debug, thiserror::Error)]
pub enum LcuError {
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("client not running")]
    NotRunning,
    #[error("HTTP {status}: {body}")]
    Status { status: u16, body: String },
}

pub struct LcuResponse {
    pub status: u16,
    pub body: Value,
}

impl LcuResponse {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

pub struct LcuClient {
    http: Client,
}

impl LcuClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .connect_timeout(std::time::Duration::from_secs(2))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");
        Self { http }
    }

    pub async fn request(
        &self,
        creds: &LcuCredentials,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<LcuResponse, LcuError> {
        let url = format!("https://127.0.0.1:{}{}", creds.port, endpoint);
        let auth =
            base64::engine::general_purpose::STANDARD.encode(format!("riot:{}", creds.token));

        let mut req = self
            .http
            .request(method, url)
            .header("Authorization", format!("Basic {auth}"))
            .header("Accept", "application/json");

        if let Some(b) = body {
            req = req.json(&b);
        } else {
            req = req.header("Content-Type", "application/json");
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let text = resp.text().await?;
        let body = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
        Ok(LcuResponse { status, body })
    }

    pub async fn request_bytes(
        &self,
        creds: &LcuCredentials,
        endpoint: &str,
    ) -> Result<(u16, Vec<u8>), LcuError> {
        let url = format!("https://127.0.0.1:{}{}", creds.port, endpoint);
        let auth =
            base64::engine::general_purpose::STANDARD.encode(format!("riot:{}", creds.token));

        let resp = self
            .http
            .get(url)
            .header("Authorization", format!("Basic {auth}"))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await?;
        Ok((status, bytes.to_vec()))
    }
}

impl Default for LcuClient {
    fn default() -> Self {
        Self::new()
    }
}
