use super::types::*;
use crate::{prelude::*, ws::Message};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMsg};

#[derive(Debug)]
pub(super) struct FastWs {
    pub bus: Arc<DashMap<Identifier, broadcast::Sender<Decoded>>>,
    pub writer: mpsc::UnboundedSender<WsMsg>, // to send subscribe frames
}

impl FastWs {
    pub(super) async fn new(url: &str, reconnect: bool) -> Result<Self> {
        let (ws, _) = connect_async(url)
            .await
            .map_err(|e| crate::Error::Websocket(e.to_string()))?;
        let (mut w, mut r) = ws.split();

        let bus: Arc<DashMap<Identifier, broadcast::Sender<Decoded>>> = Arc::new(DashMap::new());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let ping_tx = tx.clone();

        // Writer task
        let write_bus = bus.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if w.send(msg).await.is_err() {
                    break;
                }
            }
            drop(write_bus);
        });

        // Reader task
        let read_bus = bus.clone();
        tokio::spawn(async move {
            loop {
                match r.next().await {
                    Some(Ok(WsMsg::Binary(b))) => {
                        let bytes = b;
                        if bytes.first() != Some(&b'{') {
                            continue;
                        }

                        if let Some(chan) = super::router::route(&bytes) {
                            // Try zero-copy parsing for high-throughput channels first
                            let msg = super::zero_copy::parse_zero_copy(&bytes)
                                .or_else(|| serde_json::from_slice::<Message>(&bytes).ok());

                            if let Some(msg) = msg {
                                let ident = ident_from_channel(&chan);
                                if let Some(tx) = read_bus.get(&ident) {
                                    let _ = tx.send(Arc::new(msg));
                                }
                            }
                        }
                    }
                    Some(Ok(WsMsg::Text(t))) => {
                        let bytes = t.into_bytes();
                        if bytes.first() != Some(&b'{') {
                            continue;
                        }

                        if let Some(chan) = super::router::route(&bytes) {
                            // Try zero-copy parsing for high-throughput channels first
                            let msg = super::zero_copy::parse_zero_copy(&bytes)
                                .or_else(|| serde_json::from_slice::<Message>(&bytes).ok());

                            if let Some(msg) = msg {
                                let ident = ident_from_channel(&chan);
                                if let Some(tx) = read_bus.get(&ident) {
                                    let _ = tx.send(Arc::new(msg));
                                } else {
                                    log::debug!(
                                        "No subscriber for channel: {} (ident: {:?})",
                                        chan,
                                        ident
                                    );
                                }
                            } else {
                                log::warn!("Failed to parse message for channel: {}", chan);
                                // Log first 200 chars of the message for debugging
                                let preview =
                                    String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
                                log::debug!("Message preview: {}", preview);
                            }
                        }
                    }
                    Some(Ok(WsMsg::Ping(_))) => {
                        // Send pong
                        let _ = ping_tx.send(WsMsg::Pong(vec![]));
                    }
                    Some(Ok(WsMsg::Pong(_))) => {
                        // Ignore pong messages
                    }
                    Some(Ok(WsMsg::Close(_))) => {
                        // Connection closed
                        break;
                    }
                    Some(Ok(WsMsg::Frame(_))) => {
                        // Ignore raw frames
                    }
                    Some(Err(e)) => {
                        eprintln!("ws err: {e}");
                        if !reconnect {
                            break;
                        }
                        // TODO: Implement reconnection logic here
                    }
                    None => break,
                }
            }
        });

        // Ping task
        let ping_writer = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(50));
            loop {
                interval.tick().await;
                let ping_msg = serde_json::json!({"method": "ping"}).to_string();
                if ping_writer.send(WsMsg::Text(ping_msg)).is_err() {
                    break;
                }
            }
        });

        Ok(Self { bus, writer: tx })
    }

    pub(super) fn subscribe_internal(&self, ident: &Identifier) -> broadcast::Receiver<Decoded> {
        self.bus
            .entry(ident.clone())
            .or_insert_with(|| broadcast::channel(1024).0)
            .subscribe()
    }
}

fn ident_from_channel(s: &str) -> Identifier {
    match s {
        "userEvents" => Identifier::UserEvents,
        "orderUpdates" => Identifier::OrderUpdates,
        other => Identifier::Str(Box::from(other)),
    }
}
