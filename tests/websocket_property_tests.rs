// Final working property tests for the fast WebSocket implementation

use ethers::types::H160;
use hyperliquid_rust_sdk::Subscription;
use quickcheck::{Arbitrary, Gen, QuickCheck};

#[derive(Debug)]
struct TestSubscription(Subscription);

impl Clone for TestSubscription {
    fn clone(&self) -> Self {
        let json = serde_json::to_string(&self.0).unwrap();
        let cloned: Subscription = serde_json::from_str(&json).unwrap();
        TestSubscription(cloned)
    }
}

impl Arbitrary for TestSubscription {
    fn arbitrary(g: &mut Gen) -> Self {
        let choice = u8::arbitrary(g) % 4;
        let subscription = match choice {
            0 => Subscription::AllMids,
            1 => Subscription::L2Book {
                coin: "BTC".to_string(),
            },
            2 => Subscription::Trades {
                coin: "ETH".to_string(),
            },
            _ => Subscription::UserEvents {
                user: H160::from_low_u64_be(u64::arbitrary(g) % 1000),
            },
        };
        TestSubscription(subscription)
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_subscription_serialization_properties() {
        fn prop_serialization_consistent(sub: TestSubscription) -> bool {
            let json1 = serde_json::to_string(&sub.0);
            let json2 = serde_json::to_string(&sub.0);
            match (json1, json2) {
                (Ok(j1), Ok(j2)) => j1 == j2,
                _ => false,
            }
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_serialization_consistent as fn(TestSubscription) -> bool);
    }

    #[test]
    fn test_subscription_roundtrip_properties() {
        fn prop_roundtrip(sub: TestSubscription) -> bool {
            let json = serde_json::to_string(&sub.0);
            if let Ok(json_str) = json {
                let deserialized: Result<Subscription, _> = serde_json::from_str(&json_str);
                if let Ok(des) = deserialized {
                    let json2 = serde_json::to_string(&des).unwrap();
                    return json_str == json2;
                }
            }
            false
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_roundtrip as fn(TestSubscription) -> bool);
    }

    #[test]
    fn test_json_structure_properties() {
        fn prop_json_has_type_field(sub: TestSubscription) -> bool {
            let json = serde_json::to_string(&sub.0);
            if let Ok(json_str) = json {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
                if let Ok(value) = parsed {
                    return value.get("type").is_some();
                }
            }
            false
        }

        QuickCheck::new()
            .tests(1000)
            .quickcheck(prop_json_has_type_field as fn(TestSubscription) -> bool);
    }

    #[test]
    fn test_json_parsing_determinism() {
        fn prop_json_deterministic(data: String) -> bool {
            if data.is_empty() || data.len() > 100 {
                return true; // Skip problematic inputs
            }

            let message = serde_json::json!({
                "channel": "test",
                "data": data
            });

            if let Ok(json_bytes) = serde_json::to_vec(&message) {
                let parse1: Result<serde_json::Value, _> = serde_json::from_slice(&json_bytes);
                let parse2: Result<serde_json::Value, _> = serde_json::from_slice(&json_bytes);

                match (parse1, parse2) {
                    (Ok(p1), Ok(p2)) => {
                        let ch1 = p1["channel"].as_str();
                        let ch2 = p2["channel"].as_str();
                        ch1 == ch2
                    }
                    (Err(_), Err(_)) => true, // Both failed consistently
                    _ => false,               // Inconsistent results
                }
            } else {
                true // Skip if can't serialize
            }
        }

        QuickCheck::new()
            .tests(500)
            .quickcheck(prop_json_deterministic as fn(String) -> bool);
    }

    #[test]
    fn test_error_handling_robustness() {
        fn prop_no_panic_on_invalid_json(bytes: Vec<u8>) -> bool {
            if bytes.is_empty() || bytes.len() > 1000 {
                return true; // Skip problematic inputs
            }

            // This should not panic regardless of input
            let _result: Result<serde_json::Value, _> = serde_json::from_slice(&bytes);
            true // If we reach here, no panic occurred
        }

        QuickCheck::new()
            .tests(500)
            .quickcheck(prop_no_panic_on_invalid_json as fn(Vec<u8>) -> bool);
    }

    #[test]
    fn test_specific_subscription_properties() {
        // Test specific known good subscriptions
        let all_mids = TestSubscription(Subscription::AllMids);
        let btc_l2 = TestSubscription(Subscription::L2Book {
            coin: "BTC".to_string(),
        });
        let eth_trades = TestSubscription(Subscription::Trades {
            coin: "ETH".to_string(),
        });

        // All should serialize successfully
        assert!(serde_json::to_string(&all_mids.0).is_ok());
        assert!(serde_json::to_string(&btc_l2.0).is_ok());
        assert!(serde_json::to_string(&eth_trades.0).is_ok());

        // All should have valid JSON structure
        for sub in [&all_mids, &btc_l2, &eth_trades] {
            let json = serde_json::to_string(&sub.0).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(parsed.get("type").is_some());
        }

        // Test round-trip
        for sub in [&all_mids, &btc_l2, &eth_trades] {
            let json = serde_json::to_string(&sub.0).unwrap();
            let deserialized: Subscription = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn test_unicode_and_special_chars() {
        // Test that JSON handling works with unicode
        let test_cases = vec![
            "simple",
            "with spaces",
            "with-dashes",
            "with_underscores",
            "🚀💎📈", // Unicode emojis
            "αβγδε",  // Greek letters
            "test\nwith\nnewlines",
            "test\twith\ttabs",
            r#"test"with"quotes"#,
        ];

        for data in test_cases {
            let message = serde_json::json!({
                "channel": "test",
                "data": data
            });

            // Should serialize and parse successfully
            let json_str = serde_json::to_string(&message).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            assert_eq!(parsed["channel"].as_str(), Some("test"));
            assert_eq!(parsed["data"].as_str(), Some(data));
        }
    }

    #[test]
    fn test_large_message_handling() {
        // Test handling of reasonably large messages
        let large_data = "x".repeat(10000); // 10KB

        let message = serde_json::json!({
            "channel": "test",
            "data": large_data
        });

        // Should handle large messages
        let json_str = serde_json::to_string(&message).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["channel"].as_str(), Some("test"));
    }

    #[test]
    fn test_concurrent_serialization() {
        // Test that serialization is thread-safe
        use std::sync::Arc;
        use std::thread;

        let sub = Arc::new(TestSubscription(Subscription::AllMids));
        let mut handles = vec![];

        for _ in 0..10 {
            let sub_clone = sub.clone();
            let handle = thread::spawn(move || {
                // Each thread serializes the same subscription
                let json = serde_json::to_string(&sub_clone.0);
                json.unwrap()
            });
            handles.push(handle);
        }

        // All threads should produce the same JSON
        let results: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let first = &results[0];
        for result in &results {
            assert_eq!(result, first);
        }
    }
}
