use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::Connector;

use crate::lcu::connector::LcuCredentials;
use crate::state::AppState;

const SUBSCRIPTIONS: [&str; 3] = [
    "OnJsonApiEvent_lol-matchmaking_v1_ready-check",
    "OnJsonApiEvent_lol-champ-select_v1_session",
    "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase",
];

#[derive(Clone, Debug)]
pub struct WsEvent {
    pub uri: String,
    pub event_type: String,
    pub data: Value,
}

pub async fn run_event_bus(state: Arc<AppState>) {
    loop {
        let creds = match state.refresh_creds().await {
            Some(c) => c,
            None => {
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };
        if let Ok(ws) = connect(&creds).await {
            state.ws_connected.store(true, Ordering::Relaxed);
            pump(ws, &state).await;
            state.ws_connected.store(false, Ordering::Relaxed);
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(creds: &LcuCredentials) -> Result<WsStream, String> {
    let mut request = format!("wss://127.0.0.1:{}/", creds.port)
        .into_client_request()
        .map_err(|e| e.to_string())?;
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("riot:{}", creds.token));
    request.headers_mut().insert(
        "Authorization",
        format!("Basic {auth}").parse().map_err(|_| "bad header")?,
    );

    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let (ws, _resp) = tokio_tungstenite::connect_async_tls_with_config(
        request,
        None,
        false,
        Some(Connector::NativeTls(tls)),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(ws)
}

async fn pump(ws: WsStream, state: &AppState) {
    let (mut write, mut read) = ws.split();
    for event in SUBSCRIPTIONS {
        if write
            .send(Message::Text(format!("[5, \"{event}\"]")))
            .await
            .is_err()
        {
            return;
        }
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Some(event) = parse_ws_message(&text) {
                    let _ = state.ws_events.send(event);
                }
            }
            Ok(Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_)) => {}
            Ok(Message::Close(_)) | Err(_) => return,
        }
    }
}

pub fn parse_ws_message(text: &str) -> Option<WsEvent> {
    let value: Value = serde_json::from_str(text).ok()?;
    let arr = value.as_array()?;
    if arr.first()?.as_i64()? != 8 {
        return None;
    }
    let payload = arr.get(2)?;
    Some(WsEvent {
        uri: payload.get("uri")?.as_str()?.to_string(),
        event_type: payload
            .get("eventType")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        data: payload.get("data").cloned().unwrap_or(Value::Null),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_api_event() {
        let text = r#"[8,"OnJsonApiEvent_lol-gameflow_v1_gameflow-phase",{"data":"ChampSelect","eventType":"Update","uri":"/lol-gameflow/v1/gameflow-phase"}]"#;
        let event = parse_ws_message(text).unwrap();
        assert_eq!(event.uri, "/lol-gameflow/v1/gameflow-phase");
        assert_eq!(event.event_type, "Update");
        assert_eq!(event.data, Value::String("ChampSelect".into()));
    }

    #[test]
    fn ignores_subscribe_ack_and_junk() {
        assert!(parse_ws_message("").is_none());
        assert!(parse_ws_message(r#"[3,"OnJsonApiEvent"]"#).is_none());
        assert!(parse_ws_message("not json").is_none());
    }
}
