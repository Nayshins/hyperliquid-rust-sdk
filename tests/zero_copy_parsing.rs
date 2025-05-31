#[cfg(feature = "fast-ws")]
#[cfg(test)]
mod tests {
    use hyperliquid_rust_sdk::ws::Message;

    #[test]
    fn test_zero_copy_all_mids_parsing() {
        let json_data = r#"{"channel":"allMids","data":{"mids":{"BTC":"50000.0","ETH":"3000.0","SOL":"100.0"}}}"#;
        let bytes = json_data.as_bytes();

        // This should use the zero-copy parser for allMids
        if let Some(msg) = hyperliquid_rust_sdk::ws::fast::zero_copy::parse_zero_copy(bytes) {
            match msg {
                Message::AllMids(all_mids) => {
                    assert_eq!(all_mids.data.mids.get("BTC"), Some(&"50000.0".to_string()));
                    assert_eq!(all_mids.data.mids.get("ETH"), Some(&"3000.0".to_string()));
                    assert_eq!(all_mids.data.mids.get("SOL"), Some(&"100.0".to_string()));
                }
                _ => panic!("Expected AllMids message"),
            }
        } else {
            panic!("Failed to parse AllMids message");
        }
    }

    #[test]
    fn test_zero_copy_l2_book_parsing() {
        let json_data = r#"{"channel":"l2Book","data":{"coin":"BTC","time":1234567890,"levels":[[{"px":"50000.0","sz":"1.0","n":1}],[{"px":"49999.0","sz":"2.0","n":2}]]}}"#;
        let bytes = json_data.as_bytes();

        // This should use the zero-copy parser for l2Book
        if let Some(msg) = hyperliquid_rust_sdk::ws::fast::zero_copy::parse_zero_copy(bytes) {
            match msg {
                Message::L2Book(l2_book) => {
                    assert_eq!(l2_book.data.coin, "BTC");
                    assert_eq!(l2_book.data.time, 1234567890);
                    assert_eq!(l2_book.data.levels.len(), 2);
                    assert_eq!(l2_book.data.levels[0][0].px, "50000.0");
                    assert_eq!(l2_book.data.levels[0][0].sz, "1.0");
                    assert_eq!(l2_book.data.levels[0][0].n, 1);
                }
                _ => panic!("Expected L2Book message"),
            }
        } else {
            panic!("Failed to parse L2Book message");
        }
    }

    #[test]
    fn test_non_high_throughput_channel_returns_none() {
        let json_data = r#"{"channel":"trades","data":{"coin":"BTC","time":1234567890}}"#;
        let bytes = json_data.as_bytes();

        // Should return None for non-high-throughput channels, allowing fallback to serde
        let result = hyperliquid_rust_sdk::ws::fast::zero_copy::parse_zero_copy(bytes);
        assert!(result.is_none());
    }
}

#[cfg(not(feature = "fast-ws"))]
#[cfg(test)]
mod tests {
    #[test]
    fn test_zero_copy_disabled_without_fast_ws() {
        let json_data = r#"{"channel":"allMids","data":{"mids":{"BTC":"50000.0"}}}"#;
        let bytes = json_data.as_bytes();

        // Should return None when fast-ws feature is not enabled
        let result = hyperliquid_rust_sdk::ws::fast::zero_copy::parse_zero_copy(bytes);
        assert!(result.is_none());
    }
}
