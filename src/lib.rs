#![deny(unreachable_pub)]
mod consts;
mod errors;
mod exchange;
mod helpers;
mod info;
mod market_maker;
mod meta;
mod net;
mod prelude;
mod proxy_digest;
mod req;
mod signature;
pub mod ws;
pub use consts::{EPSILON, LOCAL_API_URL, MAINNET_API_URL, TESTNET_API_URL};
pub use errors::Error;
pub use exchange::*;
pub use helpers::{bps_diff, truncate_float, BaseUrl};
pub use info::{info_client::*, *};
pub use market_maker::{MarketMaker, MarketMakerInput, MarketMakerRestingOrder};
pub use meta::{AssetMeta, Meta};
pub use ws::*;

// Deprecation notice for the old client parameter pattern
#[deprecated(
    since = "0.6.1",
    note = "Use ExchangeClient::new(...) without reqwest::Client arg; pool is shared automatically. Use ExchangeClient::with_client(...) for custom clients."
)]
pub mod deprecated {
    pub use crate::exchange::ExchangeClient;
    pub use crate::info::info_client::InfoClient;
}
