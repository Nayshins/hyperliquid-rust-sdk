use hyperliquid_rust_sdk::{BboData, BboLevel, BookLevel, L2BookData};

#[test]
fn test_bbo_message_parsing() {
    // Test BBO message with array format
    let bbo_json = r#"{
        "coin": "ETH",
        "time": 1234567890,
        "bbo": [
            ["2500.5", "10.2", 5],
            ["2501.0", "8.5", 3]
        ]
    }"#;

    let parsed: BboData = serde_json::from_str(bbo_json).expect("Failed to parse BBO data");

    assert_eq!(parsed.coin, "ETH");
    assert_eq!(parsed.time, 1234567890);
    assert_eq!(parsed.bbo.len(), 2);

    assert_eq!(parsed.bbo[0].px, "2500.5");
    assert_eq!(parsed.bbo[0].sz, "10.2");
    assert_eq!(parsed.bbo[0].n, 5);

    assert_eq!(parsed.bbo[1].px, "2501.0");
    assert_eq!(parsed.bbo[1].sz, "8.5");
    assert_eq!(parsed.bbo[1].n, 3);
}

#[test]
fn test_l2book_message_parsing() {
    // Test L2Book message with nested array format
    let l2book_json = r#"{
        "coin": "BTC",
        "time": 1234567890,
        "levels": [
            [
                ["50000.0", "0.5", 2],
                ["49999.5", "1.0", 3],
                ["49999.0", "2.0", 5]
            ],
            [
                ["50000.5", "0.8", 1],
                ["50001.0", "1.2", 2],
                ["50001.5", "0.5", 1]
            ]
        ]
    }"#;

    let parsed: L2BookData =
        serde_json::from_str(l2book_json).expect("Failed to parse L2Book data");

    assert_eq!(parsed.coin, "BTC");
    assert_eq!(parsed.time, 1234567890);
    assert_eq!(parsed.levels.len(), 2); // bid and ask sides

    // Check bid side (index 0)
    assert_eq!(parsed.levels[0].len(), 3);
    assert_eq!(parsed.levels[0][0].px, "50000.0");
    assert_eq!(parsed.levels[0][0].sz, "0.5");
    assert_eq!(parsed.levels[0][0].n, 2);

    // Check ask side (index 1)
    assert_eq!(parsed.levels[1].len(), 3);
    assert_eq!(parsed.levels[1][0].px, "50000.5");
    assert_eq!(parsed.levels[1][0].sz, "0.8");
    assert_eq!(parsed.levels[1][0].n, 1);
}

#[test]
fn test_full_websocket_message_parsing() {
    use hyperliquid_rust_sdk::Message;

    // Test full WebSocket message for BBO
    let ws_message = r#"{
        "channel": "bbo",
        "data": {
            "coin": "ETH",
            "time": 1234567890,
            "bbo": [
                ["2500.5", "10.2", 5],
                ["2501.0", "8.5", 3]
            ]
        }
    }"#;

    let parsed: Message =
        serde_json::from_str(ws_message).expect("Failed to parse WebSocket message");

    if let Message::Bbo(bbo) = parsed {
        assert_eq!(bbo.data.coin, "ETH");
        assert_eq!(bbo.data.bbo.len(), 2);
    } else {
        panic!("Expected Bbo message");
    }
}
