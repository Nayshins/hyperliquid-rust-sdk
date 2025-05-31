use crate::prelude::*;
use crate::ws::{Message, Subscription};
use std::sync::Arc;
use tokio::sync::broadcast;

pub type MsgRx = broadcast::Receiver<Arc<Message>>;

#[async_trait::async_trait]
pub trait WsBackend: Send + Sync + 'static + std::fmt::Debug {
    async fn subscribe(&self, sub: Subscription) -> Result<MsgRx>;
    async fn unsubscribe(&self, sub: Subscription) -> Result<()>;
    /// clean shutdown
    async fn close(&self) -> Result<()>;
}
