mod connection;
pub(crate) mod router;
mod types;

#[cfg(feature = "fast-ws")]
use crate::ws::backend::{MsgRx, WsBackend};
#[cfg(feature = "fast-ws")]
use crate::{prelude::*, ws::Subscription};
#[cfg(feature = "fast-ws")]
use connection::FastWs;
#[cfg(feature = "fast-ws")]
use std::sync::Arc;
#[cfg(feature = "fast-ws")]
use tokio_tungstenite::tungstenite::protocol::Message as WsMsg;
#[cfg(feature = "fast-ws")]
use types::*;

#[cfg(feature = "fast-ws")]
pub async fn make_ws_backend(url: &str, reconnect: bool) -> Result<Arc<dyn WsBackend>> {
    Ok(Arc::new(FastWs::new(url, reconnect).await?))
}

#[cfg(not(feature = "fast-ws"))]
pub async fn make_ws_backend(
    _url: &str,
    _reconnect: bool,
) -> Result<std::sync::Arc<dyn crate::ws::backend::WsBackend>> {
    Err(crate::Error::GenericRequest(
        "fast-ws feature not enabled".to_string(),
    ))
}

#[cfg(feature = "fast-ws")]
#[async_trait::async_trait]
impl WsBackend for FastWs {
    async fn subscribe(&self, s: Subscription) -> Result<MsgRx> {
        let ident = match &s {
            Subscription::UserEvents { .. } => Identifier::UserEvents,
            Subscription::OrderUpdates { .. } => Identifier::OrderUpdates,
            _ => Identifier::Str(Box::from(
                serde_json::to_string(&s).map_err(|e| crate::Error::JsonParse(e.to_string()))?,
            )),
        };

        // Send subscribe frame only first time
        if self.bus.get(&ident).is_none() {
            let frame = WsMsg::Text(
                serde_json::to_string(&crate::ws::SubscriptionSendData {
                    method: "subscribe",
                    subscription: &serde_json::to_value(&s)
                        .map_err(|e| crate::Error::JsonParse(e.to_string()))?,
                })
                .map_err(|e| crate::Error::JsonParse(e.to_string()))?,
            );

            self.writer
                .send(frame)
                .map_err(|e| crate::Error::WsSend(e.to_string()))?;
        }

        Ok(self.subscribe_internal(&ident))
    }

    async fn unsubscribe(&self, s: Subscription) -> Result<()> {
        let ident = match &s {
            Subscription::UserEvents { .. } => Identifier::UserEvents,
            Subscription::OrderUpdates { .. } => Identifier::OrderUpdates,
            _ => Identifier::Str(Box::from(
                serde_json::to_string(&s).map_err(|e| crate::Error::JsonParse(e.to_string()))?,
            )),
        };

        // Remove from bus and send unsubscribe frame
        self.bus.remove(&ident);

        let frame = WsMsg::Text(
            serde_json::to_string(&crate::ws::SubscriptionSendData {
                method: "unsubscribe",
                subscription: &serde_json::to_value(&s)
                    .map_err(|e| crate::Error::JsonParse(e.to_string()))?,
            })
            .map_err(|e| crate::Error::JsonParse(e.to_string()))?,
        );

        self.writer
            .send(frame)
            .map_err(|e| crate::Error::WsSend(e.to_string()))?;

        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Close the writer channel to signal shutdown
        // The writer task will exit when all senders are dropped
        Ok(())
    }
}
