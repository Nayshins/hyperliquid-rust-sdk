#[cfg(feature = "fast-ws")]
use crate::ws::{AllMids, AllMidsData, BookLevel, L2Book, L2BookData, Message};
#[cfg(feature = "fast-ws")]
use simd_json::{BorrowedValue, ValueAccess};
#[cfg(feature = "fast-ws")]
use std::collections::HashMap;

#[cfg(feature = "fast-ws")]
pub fn parse_zero_copy(bytes: &[u8]) -> Option<Message> {
    let mut buf = bytes.to_vec();
    let borrowed: BorrowedValue<'_> = simd_json::to_borrowed_value(&mut buf).ok()?;
    
    let channel = borrowed.get("channel")?.as_str()?;
    
    match channel {
        "allMids" => parse_all_mids_zero_copy(&borrowed),
        "l2Book" => parse_l2_book_zero_copy(&borrowed),
        _ => None, // Fall back to regular serde parsing for other message types
    }
}

#[cfg(feature = "fast-ws")]
fn parse_all_mids_zero_copy(borrowed: &BorrowedValue<'_>) -> Option<Message> {
    let data_obj = borrowed.get("data")?;
    let mids_obj = data_obj.get("mids")?;
    
    let mut mids = HashMap::new();
    
    // Zero-copy extraction of mids data
    if let Some(obj) = mids_obj.as_object() {
        for (key, value) in obj {
            if let Some(price_str) = value.as_str() {
                mids.insert(key.to_string(), price_str.to_string());
            }
        }
    }
    
    Some(Message::AllMids(AllMids {
        data: AllMidsData { mids },
    }))
}

#[cfg(feature = "fast-ws")]
fn parse_l2_book_zero_copy(borrowed: &BorrowedValue<'_>) -> Option<Message> {
    let data_obj = borrowed.get("data")?;
    
    let coin = data_obj.get("coin")?.as_str()?.to_string();
    let time = data_obj.get("time")?.as_u64()?;
    
    let levels_array = data_obj.get("levels")?.as_array()?;
    let mut levels = Vec::new();
    
    // Parse levels array (typically [bids, asks])
    for level_group in levels_array {
        if let Some(group_array) = level_group.as_array() {
            let mut level_group_vec = Vec::new();
            
            for level_item in group_array {
                if let Some(level_obj) = level_item.as_object() {
                    let px = level_obj.get("px")?.as_str()?.to_string();
                    let sz = level_obj.get("sz")?.as_str()?.to_string();
                    let n = level_obj.get("n")?.as_u64()?;
                    
                    level_group_vec.push(BookLevel { px, sz, n });
                }
            }
            levels.push(level_group_vec);
        }
    }
    
    Some(Message::L2Book(L2Book {
        data: L2BookData { coin, time, levels },
    }))
}

#[cfg(not(feature = "fast-ws"))]
pub(super) fn parse_zero_copy(_bytes: &[u8]) -> Option<Message> {
    None
}