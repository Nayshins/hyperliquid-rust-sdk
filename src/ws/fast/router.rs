#[cfg(feature = "fast-ws")]
pub(super) fn route(buf: &[u8]) -> Option<String> {
    use simd_json::{to_borrowed_value, BorrowedValue, ValueAccess};

    let mut buf_copy = buf.to_vec();
    let v: BorrowedValue<'_> = to_borrowed_value(&mut buf_copy).ok()?;
    v.get("channel")?.as_str().map(|s| s.to_string())
}

#[cfg(not(feature = "fast-ws"))]
pub(super) fn route(_buf: &[u8]) -> Option<String> {
    None
}
