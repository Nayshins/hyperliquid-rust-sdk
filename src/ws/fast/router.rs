#[cfg(feature = "fast-ws")]
pub(super) fn route(buf: &[u8]) -> Option<String> {
    use simd_json::{to_borrowed_value, BorrowedValue, ValueAccess};

    let mut buf_copy = buf.to_vec();
    let v: BorrowedValue<'_> = to_borrowed_value(&mut buf_copy).ok()?;
    let channel = v.get("channel")?.as_str()?;

    // For coin-specific channels, include the coin in the identifier
    match channel {
        "bbo" | "l2Book" | "trades" | "candle" => {
            if let Some(data) = v.get("data") {
                if let Some(coin) = data.get("coin").and_then(|c| c.as_str()) {
                    return Some(format!("{}:{}", channel, coin));
                }
            }
            Some(channel.to_string())
        }
        other => Some(other.to_string()),
    }
}

#[cfg(not(feature = "fast-ws"))]
pub(super) fn route(_buf: &[u8]) -> Option<String> {
    None
}
