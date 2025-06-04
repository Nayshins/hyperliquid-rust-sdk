mod connection;
pub(crate) mod router;
mod types;
pub mod zero_copy;

use crate::ws::backend::{MsgRx, WsBackend};
use crate::{prelude::*, ws::Subscription};
use connection::FastWs;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::protocol::Message as WsMsg;
use types::*;

pub async fn make_ws_backend(url: &str, reconnect: bool) -> Result<Arc<dyn WsBackend>> {
    Ok(Arc::new(FastWs::new(url, reconnect).await?))
}

#[async_trait::async_trait]
impl WsBackend for FastWs {
    async fn subscribe(&self, s: Subscription) -> Result<MsgRx> {
        let ident = match &s {
            Subscription::UserEvents { .. } => Identifier::UserEvents,
            Subscription::OrderUpdates { .. } => Identifier::OrderUpdates,
            Subscription::AllMids => Identifier::Str(Box::from("allMids")),
            Subscription::L2Book { coin } => Identifier::Str(Box::from(format!("l2Book:{}", coin))),
            Subscription::Trades { coin } => Identifier::Str(Box::from(format!("trades:{}", coin))),
            Subscription::Candle { coin, .. } => {
                Identifier::Str(Box::from(format!("candle:{}", coin)))
            }
            Subscription::Notification { .. } => Identifier::Str(Box::from("notification")),
            Subscription::Bbo { coin } => Identifier::Str(Box::from(format!("bbo:{}", coin))),
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
            Subscription::AllMids => Identifier::Str(Box::from("allMids")),
            Subscription::L2Book { coin } => Identifier::Str(Box::from(format!("l2Book:{}", coin))),
            Subscription::Trades { coin } => Identifier::Str(Box::from(format!("trades:{}", coin))),
            Subscription::Candle { coin, .. } => {
                Identifier::Str(Box::from(format!("candle:{}", coin)))
            }
            Subscription::Notification { .. } => Identifier::Str(Box::from("notification")),
            Subscription::Bbo { coin } => Identifier::Str(Box::from(format!("bbo:{}", coin))),
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
