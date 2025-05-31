use ethers::types::H160;
use hyperliquid_rust_sdk::{Error, Message, MsgRx, Subscription, WsBackend};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

// Mock WebSocket backend for testing
#[derive(Debug)]
pub struct MockWsBackend {
    channels: Arc<tokio::sync::Mutex<HashMap<String, broadcast::Sender<Arc<Message>>>>>,
    test_messages: Vec<(String, Message)>, // (subscription_id, message) pairs
}

impl Default for MockWsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWsBackend {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            test_messages: Vec::new(),
        }
    }

    pub fn with_test_messages(mut self, messages: Vec<(String, Message)>) -> Self {
        self.test_messages = messages;
        self
    }

    async fn get_or_create_channel(
        &self,
        subscription_id: &str,
    ) -> broadcast::Sender<Arc<Message>> {
        let mut channels = self.channels.lock().await;
        channels
            .entry(subscription_id.to_string())
            .or_insert_with(|| broadcast::channel(1024).0)
            .clone()
    }

    // Simulate sending a message to subscribers
    pub async fn send_message(&self, subscription_id: &str, message: Message) -> Result<(), Error> {
        let sender = self.get_or_create_channel(subscription_id).await;
        sender
            .send(Arc::new(message))
            .map_err(|e| Error::WsSend(e.to_string()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WsBackend for MockWsBackend {
    async fn subscribe(&self, sub: Subscription) -> Result<MsgRx, Error> {
        let subscription_id =
            serde_json::to_string(&sub).map_err(|e| Error::JsonParse(e.to_string()))?;

        let sender = self.get_or_create_channel(&subscription_id).await;
        let receiver = sender.subscribe();

        // Send any pre-configured test messages for this subscription
        for (test_sub_id, test_message) in &self.test_messages {
            if test_sub_id == &subscription_id {
                let _ = sender.send(Arc::new(test_message.clone()));
            }
        }

        Ok(receiver)
    }

    async fn unsubscribe(&self, sub: Subscription) -> Result<(), Error> {
        let subscription_id =
            serde_json::to_string(&sub).map_err(|e| Error::JsonParse(e.to_string()))?;

        // Remove the channel for this subscription
        let mut channels = self.channels.lock().await;
        channels.remove(&subscription_id);
        Ok(())
    }

    async fn close(&self) -> Result<(), Error> {
        // Clear all channels
        let mut channels = self.channels.lock().await;
        channels.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_mock_subscription_basic() {
        let mock_backend = MockWsBackend::new();

        // Test basic subscription
        let subscription = Subscription::AllMids;
        let mut rx = mock_backend.subscribe(subscription.clone()).await.unwrap();

        // Send a test message
        let test_message = Message::NoData; // Simple test message
        let subscription_id = serde_json::to_string(&subscription).unwrap();
        mock_backend
            .send_message(&subscription_id, test_message)
            .await
            .unwrap();

        // Verify we receive the message
        let received = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(received.is_ok(), "Should receive message within timeout");

        let message = received.unwrap().unwrap();
        matches!(*message, Message::NoData);
    }

    #[tokio::test]
    async fn test_mock_multiple_subscriptions() {
        let mock_backend = MockWsBackend::new();

        // Create multiple subscriptions
        let sub1 = Subscription::AllMids;
        let sub2 = Subscription::L2Book {
            coin: "BTC".to_string(),
        };

        let mut rx1 = mock_backend.subscribe(sub1.clone()).await.unwrap();
        let mut rx2 = mock_backend.subscribe(sub2.clone()).await.unwrap();

        // Send messages to different subscriptions
        let sub1_id = serde_json::to_string(&sub1).unwrap();
        let sub2_id = serde_json::to_string(&sub2).unwrap();

        mock_backend
            .send_message(&sub1_id, Message::NoData)
            .await
            .unwrap();
        mock_backend
            .send_message(&sub2_id, Message::NoData)
            .await
            .unwrap();

        // Both should receive their respective messages
        let msg1 = tokio::time::timeout(Duration::from_millis(100), rx1.recv()).await;
        let msg2 = tokio::time::timeout(Duration::from_millis(100), rx2.recv()).await;

        assert!(msg1.is_ok(), "Subscription 1 should receive message");
        assert!(msg2.is_ok(), "Subscription 2 should receive message");
    }

    #[tokio::test]
    async fn test_mock_pre_configured_messages() {
        let subscription = Subscription::Trades {
            coin: "ETH".to_string(),
        };
        let subscription_id = serde_json::to_string(&subscription).unwrap();

        // Create mock with pre-configured messages
        let test_messages = vec![(subscription_id.clone(), Message::NoData)];

        let mock_backend = MockWsBackend::new().with_test_messages(test_messages);

        // Subscribe and immediately receive pre-configured message
        let mut rx = mock_backend.subscribe(subscription).await.unwrap();

        let received = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(received.is_ok(), "Should receive pre-configured message");
    }

    #[tokio::test]
    async fn test_mock_unsubscribe() {
        let mock_backend = MockWsBackend::new();
        let subscription = Subscription::AllMids;

        // Subscribe
        let _rx = mock_backend.subscribe(subscription.clone()).await.unwrap();

        // Unsubscribe should work without error
        let result = mock_backend.unsubscribe(subscription).await;
        assert!(result.is_ok(), "Unsubscribe should succeed");
    }

    #[tokio::test]
    async fn test_mock_concurrent_subscriptions() {
        use tokio::task::JoinSet;

        let mock_backend = Arc::new(MockWsBackend::new());
        let mut join_set = JoinSet::new();

        // Create multiple concurrent subscriptions
        for i in 0..10 {
            let backend = mock_backend.clone();
            join_set.spawn(async move {
                let subscription = Subscription::L2Book {
                    coin: format!("COIN{}", i),
                };

                let mut rx = backend.subscribe(subscription.clone()).await.unwrap();

                // Send a message to this specific subscription
                let sub_id = serde_json::to_string(&subscription).unwrap();
                backend
                    .send_message(&sub_id, Message::NoData)
                    .await
                    .unwrap();

                // Try to receive it
                let received = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

                received.is_ok()
            });
        }

        // All tasks should succeed
        let mut success_count = 0;
        while let Some(result) = join_set.join_next().await {
            if result.unwrap() {
                success_count += 1;
            }
        }

        assert_eq!(
            success_count, 10,
            "All concurrent subscriptions should work"
        );
    }

    #[tokio::test]
    async fn test_mock_subscription_isolation() {
        let mock_backend = MockWsBackend::new();

        let sub1 = Subscription::AllMids;
        let sub2 = Subscription::L2Book {
            coin: "BTC".to_string(),
        };

        let mut rx1 = mock_backend.subscribe(sub1.clone()).await.unwrap();
        let mut rx2 = mock_backend.subscribe(sub2.clone()).await.unwrap();

        // Send message only to subscription 1
        let sub1_id = serde_json::to_string(&sub1).unwrap();
        mock_backend
            .send_message(&sub1_id, Message::NoData)
            .await
            .unwrap();

        // Only rx1 should receive the message
        let msg1 = tokio::time::timeout(Duration::from_millis(100), rx1.recv()).await;
        assert!(msg1.is_ok(), "Subscription 1 should receive message");

        // rx2 should timeout (no message)
        let msg2 = tokio::time::timeout(Duration::from_millis(50), rx2.recv()).await;
        assert!(msg2.is_err(), "Subscription 2 should not receive message");
    }

    #[tokio::test]
    async fn test_mock_error_handling() {
        let mock_backend = MockWsBackend::new();

        // Test invalid subscription handling
        let subscription = Subscription::UserEvents { user: H160::zero() };
        let result = mock_backend.subscribe(subscription).await;
        assert!(result.is_ok(), "Even complex subscriptions should work");

        // Test close
        let close_result = mock_backend.close().await;
        assert!(close_result.is_ok(), "Close should succeed");
    }
}
