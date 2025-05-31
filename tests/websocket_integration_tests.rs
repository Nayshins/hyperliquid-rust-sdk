use hyperliquid_rust_sdk::{make_ws_backend, Subscription};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "fast-ws")]
async fn test_fast_ws_backend_creation() {
    // Test that we can create a fast WebSocket backend without connecting
    // This validates the backend trait implementation and type safety
    use hyperliquid_rust_sdk::WsBackend;

    // Test backend creation (this will fail to connect but shouldn't panic)
    let result = make_ws_backend("ws://localhost:9999/invalid", false).await;

    // We expect this to fail since there's no server, but it should be a graceful error
    match result {
        Ok(_) => {
            // If it somehow succeeds (unlikely), that's also fine
            println!("Backend creation succeeded unexpectedly");
        }
        Err(e) => {
            // Expected: connection should fail gracefully
            println!("Backend creation failed as expected: {}", e);
            // The important thing is that it didn't panic
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(feature = "fast-ws"))]
async fn test_legacy_ws_backend_creation() {
    // Test that legacy backend creation works when fast-ws is disabled
    let result = make_ws_backend("ws://localhost:9999/invalid", false).await;

    // We expect this to fail since there's no server, but it should be a graceful error
    match result {
        Ok(_) => {
            println!("Legacy backend creation succeeded unexpectedly");
        }
        Err(e) => {
            println!("Legacy backend creation failed as expected: {}", e);
            // The important thing is that it didn't panic
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_subscription_creation() {
    // Test that we can create various subscription types
    use ethers::types::H160;

    let _all_mids = Subscription::AllMids;
    let _user_events = Subscription::UserEvents { user: H160::zero() };
    let _l2_book = Subscription::L2Book {
        coin: "BTC".to_string(),
    };

    // If we get here without panicking, subscription creation works
}

// Integration test that actually connects to the WebSocket
#[tokio::test(flavor = "multi_thread")]
#[ignore] // Ignore by default since it requires network access
async fn test_real_websocket_connection() {
    let ws = make_ws_backend("wss://api.hyperliquid.xyz/ws", false)
        .await
        .expect("Should be able to create WebSocket backend");

    let mut rx = ws
        .subscribe(Subscription::AllMids)
        .await
        .expect("Should be able to subscribe to AllMids");

    // Wait for multiple messages to ensure the connection is stable
    for i in 0..3 {
        let msg = tokio::time::timeout(Duration::from_secs(30), rx.recv())
            .await
            .expect(&format!(
                "Should receive message {} within 30 seconds",
                i + 1
            ));

        assert!(msg.is_ok(), "Message {} should be valid", i + 1);
        println!("Received message {}: {:?}", i + 1, msg.unwrap());
    }
}
