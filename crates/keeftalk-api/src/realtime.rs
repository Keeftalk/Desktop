//! Supabase Realtime Phoenix WebSocket helper for live Postgres changes.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};

use crate::{ApiError, BackendEndpoints};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeChangeEvent {
    pub table: String,
    pub event_type: String,
    pub record: Value,
}

pub fn build_realtime_ws_url(endpoints: &BackendEndpoints) -> Result<String, ApiError> {
    let mut url = endpoints.base_url.clone();
    let ws_scheme = match url.scheme() {
        "https" => "wss",
        "http" => "ws",
        other => {
            return Err(ApiError::Realtime(format!(
                "unsupported URL scheme for realtime: {other}"
            )))
        }
    };
    url.set_scheme(ws_scheme)
        .map_err(|_| ApiError::Realtime("failed to switch scheme to ws/wss".into()))?;
    url.set_path("realtime/v1/websocket");
    url.set_query(Some(&format!("apikey={}&vsn=1.0.0", endpoints.anon_key)));
    Ok(url.to_string())
}

/// Connects to Supabase Realtime and subscribes to `public:{table}` Postgres changes.
pub async fn subscribe_table(
    endpoints: &BackendEndpoints,
    access_token: &str,
    table: &str,
) -> Result<mpsc::Receiver<RealtimeChangeEvent>, ApiError> {
    let ws_url = build_realtime_ws_url(endpoints)?;
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ApiError::Realtime(e.to_string()))?;

    let (mut write, mut read) = ws_stream.split();
    let topic = format!("realtime:public:{table}");

    let join_msg = serde_json::json!({
        "topic": topic,
        "event": "phx_join",
        "payload": {
            "config": {
                "postgres_changes": [{
                    "event": "*",
                    "schema": "public",
                    "table": table
                }]
            },
            "access_token": access_token
        },
        "ref": "1"
    });

    write
        .send(Message::Text(join_msg.to_string().into()))
        .await
        .map_err(|e| ApiError::Realtime(e.to_string()))?;

    let (tx, rx) = mpsc::channel(128);
    let table_name = table.to_string();

    tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(25));
        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    let hb = serde_json::json!({
                        "topic": "phoenix",
                        "event": "heartbeat",
                        "payload": {},
                        "ref": "hb"
                    });
                    if write.send(Message::Text(hb.to_string().into())).await.is_err() {
                        break;
                    }
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(txt))) => {
                            if let Ok(v) = serde_json::from_str::<Value>(&txt)
                                && v.get("event").and_then(Value::as_str) == Some("postgres_changes")
                                && let Some(data) = v.pointer("/payload/data")
                            {
                                let ev = RealtimeChangeEvent {
                                    table: table_name.clone(),
                                    event_type: data.get("type").and_then(Value::as_str).unwrap_or("UNKNOWN").to_string(),
                                    record: data.get("record").cloned().unwrap_or(Value::Null),
                                };
                                if tx.send(ev).await.is_err() {
                                    break;
                                }
                            } else {
                                debug!("Realtime control frame received");
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Err(e)) => {
                            warn!("Realtime stream error: {e}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_wss_url_from_https_endpoint() {
        let ep = BackendEndpoints::from_raw("https://example.supabase.co", "anon-key").expect("ep");
        let ws = build_realtime_ws_url(&ep).expect("ws url");
        assert_eq!(
            ws,
            "wss://example.supabase.co/realtime/v1/websocket?apikey=anon-key&vsn=1.0.0"
        );
    }
}
