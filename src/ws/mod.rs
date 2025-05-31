pub mod backend;
mod message_types;
mod sub_structs;
mod ws_manager;

#[cfg(feature = "fast-ws")]
pub mod fast;
#[cfg(feature = "fast-ws")]
pub use fast::make_ws_backend;

#[cfg(not(feature = "fast-ws"))]
mod legacy;
#[cfg(not(feature = "fast-ws"))]
pub async fn make_ws_backend(
    url: &str,
    reconnect: bool,
) -> crate::prelude::Result<std::sync::Arc<dyn backend::WsBackend>> {
    use legacy::WsManager;
    use std::sync::Arc;
    Ok(Arc::new(WsManager::new(url.to_string(), reconnect).await?))
}

pub use backend::{MsgRx, WsBackend};
pub use message_types::*;
pub use sub_structs::*;
pub use ws_manager::{Message, Subscription, SubscriptionSendData};

// Re-export for backwards compatibility
