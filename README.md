# hyperliquid-rust-sdk

SDK for Hyperliquid API trading with Rust.

## Usage Examples

See `src/bin` for examples. You can run any example with `cargo run --bin [EXAMPLE]`.

## Connection Pooling

The SDK now uses a shared HTTP connection pool for all REST API calls, significantly improving performance:

- **Single TCP/TLS handshake** per process, reused for all requests
- **Connection keep-alive** allows multiple requests over the same connection
- **32 idle connections per host** with 90-second keep-alive
- **Automatic connection reuse** across all SDK instances

### Default Usage
```rust
// Uses shared connection pool automatically
let client = ExchangeClient::new(wallet, Some(BaseUrl::Mainnet), None, None).await?;
let info_client = InfoClient::new(Some(BaseUrl::Mainnet)).await?;
```

### Custom Client
If you need specific HTTP client settings:
```rust
let custom_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

// Use Box::leak to get 'static lifetime for custom client
let static_client = Box::leak(Box::new(custom_client));

let client = ExchangeClient::with_client(
    static_client,
    wallet,
    Some(BaseUrl::Mainnet),
    None,
    None
).await?;
```

This results in:
- **Lower latency** due to connection reuse
- **Better resource efficiency** with fewer TCP connections
- **Improved p95 latency** especially under load

## Installation

`cargo add hyperliquid_rust_sdk`

## License

This project is licensed under the terms of the `MIT` license. See [LICENSE](LICENSE.md) for more details.

```bibtex
@misc{hyperliquid-rust-sdk,
  author = {Hyperliquid},
  title = {SDK for Hyperliquid API trading with Rust.},
  year = {2024},
  publisher = {GitHub},
  journal = {GitHub repository},
  howpublished = {\url{https://github.com/hyperliquid-dex/hyperliquid-rust-sdk}}
}
```
