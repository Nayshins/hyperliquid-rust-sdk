//! tests/ws_type_mismatch.rs
//! `cargo test --tests` will run it.

use serde::Deserialize;

/// ---------- 1️⃣  The structs you currently have  (px / sz = String) ----------
#[derive(Debug, Deserialize)]
struct OldTrade {
    coin: String,
    side: String,
    px: String, // ⬅️ wrong type
    sz: String, // ⬅️ wrong type
    time: u64,
    hash: String,
    tid: u64,
}

#[derive(Debug, Deserialize)]
struct OldBboLevel {
    px: String, // ⬅️ wrong type
    sz: String, // ⬅️ wrong type
    n: u64,
}

/// ---------- 2️⃣  Fixed structs  (px / sz = f64, plus helper for back-compat) ----------
mod fixed {
    use super::*;
    use serde::Deserializer;

    // helper: accept either string OR number
    pub fn de_str_or_f64<'de, D>(d: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = f64;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("string or float")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<f64, E> {
                v.parse::<f64>().map_err(E::custom)
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<f64, E> {
                Ok(v)
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<f64, E> {
                Ok(v as f64)
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<f64, E> {
                Ok(v as f64)
            }
        }
        d.deserialize_any(V)
    }

    #[derive(Debug, Deserialize)]
    pub struct Trade {
        coin: String,
        side: String,
        #[serde(deserialize_with = "de_str_or_f64")]
        pub px: f64,
        #[serde(deserialize_with = "de_str_or_f64")]
        pub sz: f64,
        time: u64,
        hash: String,
        tid: u64,
    }

    #[derive(Debug, Deserialize)]
    pub struct BboLevel {
        #[serde(deserialize_with = "de_str_or_f64")]
        pub px: f64,
        #[serde(deserialize_with = "de_str_or_f64")]
        pub sz: f64,
        n: u64,
    }
}

/// ---------- 3️⃣  Sample JSON straight from the wire (May-2025) ----------
const TRADE_JSON: &str = r#"
{ "coin":"ETH","side":"b","px":2692.6,"sz":0.48,
  "time":1717325379123,"hash":"abc","tid":123 }
"#;

const BBO_JSON: &str = r#"
{ "px":2692.6, "sz":8.5, "n":1 }
"#;

/// ---------- 4️⃣  Tests --------------------------------------------------------

#[test]
fn old_structs_fail_to_parse() {
    // The old structs expect strings, but JSON supplies numbers – this must error.
    assert!(
        serde_json::from_str::<OldTrade>(TRADE_JSON).is_err(),
        "OldTrade unexpectedly succeeded!"
    );
    assert!(
        serde_json::from_str::<OldBboLevel>(BBO_JSON).is_err(),
        "OldBboLevel unexpectedly succeeded!"
    );
}

#[test]
fn fixed_structs_parse_successfully() {
    let t: fixed::Trade = serde_json::from_str(TRADE_JSON).unwrap();
    let l: fixed::BboLevel = serde_json::from_str(BBO_JSON).unwrap();

    assert_eq!(t.px, 2692.6);
    assert_eq!(t.sz, 0.48);
    assert_eq!(l.px, 2692.6);
    assert_eq!(l.sz, 8.5);
}
