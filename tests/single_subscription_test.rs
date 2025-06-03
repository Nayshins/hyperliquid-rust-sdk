use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Subscription, Message};
use tokio::sync::mpsc;
use std::time::Duration;

#[tokio::test]
async fn test_single_subscription() {
    // Simple test with just one subscription
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let mut info_client = InfoClient::new(Some(BaseUrl::Mainnet))
        .await
        .expect("Failed to create InfoClient");

    println!("Subscribing to ETH trades...");
    
    info_client
        .subscribe(Subscription::Trades { coin: "ETH".to_string() }, tx)
        .await
        .expect("Failed to subscribe to Trades");

    println!("Waiting for messages...");
    
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);
    
    let mut message_count = 0;
    
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                message_count += 1;
                println!("Message #{}: {:?}", message_count, msg);
                
                if let Message::Trades(trades) = &msg {
                    if let Some(trade) = trades.data.first() {
                        println!("Trade details: {} {} {} @ {}", 
                            trade.coin, trade.side, trade.sz, trade.px
                        );
                    }
                }
                
                if message_count >= 5 {
                    println!("Received {} messages, test passed!", message_count);
                    break;
                }
            }
            _ = &mut timeout => {
                println!("Timeout reached after {} messages", message_count);
                break;
            }
        }
    }
    
    assert!(message_count > 0, "Did not receive any messages");
}