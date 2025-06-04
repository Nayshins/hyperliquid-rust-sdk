pub(super) fn route(buf: &[u8]) -> Option<String> {
    use simd_json::{to_borrowed_value, BorrowedValue, ValueAccess};

    let mut buf_copy = buf.to_vec();
    let v: BorrowedValue<'_> = to_borrowed_value(&mut buf_copy).ok()?;
    let channel = v.get("channel")?.as_str()?;

    // For coin-specific channels, include the coin in the identifier
    match channel {
        "bbo" | "l2Book" | "candle" => {
            if let Some(data) = v.get("data") {
                if let Some(coin) = data.get("coin").and_then(|c| c.as_str()) {
                    return Some(format!("{}:{}", channel, coin));
                }
            }
            Some(channel.to_string())
        }
        "trades" => {
            // For trades, coin is in each trade object within the data array
            if let Some(data) = v.get("data") {
                if let Some(trades_array) = data.as_array() {
                    // Skip empty trades arrays - they don't contain useful data
                    if trades_array.is_empty() {
                        return None;
                    }
                    if let Some(first_trade) = trades_array.first() {
                        if let Some(coin) = first_trade.get("coin").and_then(|c| c.as_str()) {
                            return Some(format!("{}:{}", channel, coin));
                        }
                    }
                }
            }
            // Log when we can't route trades properly
            log::debug!("Unable to determine coin for trades message, skipping");
            None
        }
        other => Some(other.to_string()),
    }
}
