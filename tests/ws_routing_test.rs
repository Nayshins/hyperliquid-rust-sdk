use hyperliquid_rust_sdk::prelude::*;
use hyperliquid_rust_sdk::{InfoClient, Subscription, Message};
use tokio::sync::mpsc;
use std::time::Duration;

#[tokio::test]
#[ignore] // This is a live test - remove #[ignore] to run against real WebSocket
async fn test_multiple_subscriptions_routing() {
    // Create separate channels for each subscription type
    let (l2_tx, mut l2_rx) = mpsc::channel::<Message>(100);
    let (bbo_tx, mut bbo_rx) = mpsc::channel::<Message>(100);
    let (trades_tx, mut trades_rx) = mpsc::channel::<Message>(100);

    // Create InfoClient
    let info_client = InfoClient::new(Some(BaseUrl::Mainnet))
        .await
        .expect("Failed to create InfoClient");

    // Subscribe to different feeds for the same coin
    let coin = "ETH".to_string();

    let l2_id = info_client
        .subscribe(Subscription::L2Book { coin: coin.clone() }, l2_tx)
        .await
        .expect("Failed to subscribe to L2Book");
    println!("L2Book subscription ID: {}", l2_id);

    let bbo_id = info_client
        .subscribe(Subscription::Bbo { coin: coin.clone() }, bbo_tx)
        .await
        .expect("Failed to subscribe to BBO");
    println!("BBO subscription ID: {}", bbo_id);

    let trades_id = info_client
        .subscribe(Subscription::Trades { coin: coin.clone() }, trades_tx)
        .await
        .expect("Failed to subscribe to Trades");
    println!("Trades subscription ID: {}", trades_id);

    // Track which types of messages we receive on each channel
    let mut l2_types = Vec::new();
    let mut bbo_types = Vec::new();
    let mut trades_types = Vec::new();

    // Collect messages for a short time
    let timeout = Duration::from_secs(10);
    let start = tokio::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            break;
        }

        tokio::select! {
            Some(msg) = l2_rx.recv() => {
                let msg_type = match &msg {
                    Message::L2Book(_) => "L2Book",
                    Message::Bbo(_) => "BBO",
                    Message::Trades(_) => "Trades",
                    _ => "Other",
                };
                l2_types.push(msg_type);
                println!("L2 channel received: {}", msg_type);
            }
            Some(msg) = bbo_rx.recv() => {
                let msg_type = match &msg {
                    Message::L2Book(_) => "L2Book",
                    Message::Bbo(_) => "BBO",
                    Message::Trades(_) => "Trades",
                    _ => "Other",
                };
                bbo_types.push(msg_type);
                println!("BBO channel received: {}", msg_type);
            }
            Some(msg) = trades_rx.recv() => {
                let msg_type = match &msg {
                    Message::L2Book(_) => "L2Book",
                    Message::Bbo(_) => "BBO",
                    Message::Trades(_) => "Trades",
                    _ => "Other",
                };
                trades_types.push(msg_type);
                println!("Trades channel received: {}", msg_type);
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Continue looping
            }
        }
    }

    // Print summary
    println!("\n=== Summary ===");
    println!("L2 channel received {} messages: {:?}", l2_types.len(), l2_types);
    println!("BBO channel received {} messages: {:?}", bbo_types.len(), bbo_types);
    println!("Trades channel received {} messages: {:?}", trades_types.len(), trades_types);

    // Verify routing is correct
    // L2 channel should only receive L2Book messages
    assert!(l2_types.iter().all(|t| *t == "L2Book" || *t == "Other"),
        "L2 channel received non-L2Book messages: {:?}", l2_types);

    // BBO channel should only receive BBO messages
    assert!(bbo_types.iter().all(|t| *t == "BBO" || *t == "Other"),
        "BBO channel received non-BBO messages: {:?}", bbo_types);

    // Trades channel should only receive Trades messages
    assert!(trades_types.iter().all(|t| *t == "Trades" || *t == "Other"),
        "Trades channel received non-Trades messages: {:?}", trades_types);
}