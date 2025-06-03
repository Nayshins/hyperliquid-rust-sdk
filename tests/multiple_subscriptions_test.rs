use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Subscription, Message};
use tokio::sync::mpsc;
use std::time::Duration;

#[tokio::test]
async fn test_multiple_subscriptions_live() {
    // Live test against real Hyperliquid WebSocket
    let (l2_tx, mut l2_rx) = mpsc::unbounded_channel::<Message>();
    let (bbo_tx, mut bbo_rx) = mpsc::unbounded_channel::<Message>();
    let (trades_tx, mut trades_rx) = mpsc::unbounded_channel::<Message>();

    let mut info_client = InfoClient::new(Some(BaseUrl::Mainnet))
        .await
        .expect("Failed to create InfoClient");

    // Subscribe to all three types for ETH
    info_client
        .subscribe(Subscription::L2Book { coin: "ETH".to_string() }, l2_tx)
        .await
        .expect("Failed to subscribe to L2Book");
    
    info_client
        .subscribe(Subscription::Bbo { coin: "ETH".to_string() }, bbo_tx)
        .await
        .expect("Failed to subscribe to BBO");
    
    info_client
        .subscribe(Subscription::Trades { coin: "ETH".to_string() }, trades_tx)
        .await
        .expect("Failed to subscribe to Trades");

    println!("Waiting for messages...");
    
    let mut got_l2 = false;
    let mut got_bbo = false;
    let mut got_trades = false;
    
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);
    
    loop {
        tokio::select! {
            Some(msg) = l2_rx.recv() => {
                println!("L2 channel received: {:?}", msg);
                if let Message::L2Book(book) = msg {
                    println!("✓ L2Book: {} levels for {}", 
                        book.data.levels.get(0).map(|l| l.len()).unwrap_or(0),
                        book.data.coin
                    );
                    got_l2 = true;
                }
            }
            Some(msg) = bbo_rx.recv() => {
                println!("BBO channel received: {:?}", msg);
                if let Message::Bbo(bbo) = msg {
                    if let Some(level) = bbo.data.bbo.first() {
                        println!("✓ BBO: {} @ px={} sz={}", 
                            bbo.data.coin, level.px, level.sz
                        );
                        got_bbo = true;
                    }
                }
            }
            Some(msg) = trades_rx.recv() => {
                println!("Trades channel received: {:?}", msg);
                if let Message::Trades(trades) = msg {
                    if let Some(trade) = trades.data.first() {
                        println!("✓ Trade: {} {} {} @ {}", 
                            trade.coin, trade.side, trade.sz, trade.px
                        );
                        got_trades = true;
                    }
                }
            }
            _ = &mut timeout => {
                println!("Timeout reached");
                break;
            }
        }
        
        if got_l2 && got_bbo && got_trades {
            println!("\n✅ Successfully received all three message types!");
            break;
        }
    }
    
    assert!(got_l2, "Did not receive L2Book messages");
    assert!(got_bbo, "Did not receive BBO messages");
    assert!(got_trades, "Did not receive Trades messages");
}