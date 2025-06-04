use crate::{
    ws::message_types::{AllMids, Bbo, Candle, L2Book, OrderUpdates, Trades, User},
    ActiveAssetCtx, Notification, UserFills, UserFundings, UserNonFundingLedgerUpdates,
    WebData2,
};
use serde::{Deserialize, Serialize};
use ethers::types::H160;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Subscription {
    AllMids,
    Notification { user: H160 },
    WebData2 { user: H160 },
    Candle { coin: String, interval: String },
    L2Book { coin: String },
    Trades { coin: String },
    OrderUpdates { user: H160 },
    UserEvents { user: H160 },
    UserFills { user: H160 },
    UserFundings { user: H160 },
    UserNonFundingLedgerUpdates { user: H160 },
    ActiveAssetCtx { coin: String },
    Bbo { coin: String },
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "channel")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    NoData,
    HyperliquidError(String),
    AllMids(AllMids),
    Trades(Trades),
    L2Book(L2Book),
    User(User),
    UserFills(UserFills),
    Candle(Candle),
    SubscriptionResponse,
    OrderUpdates(OrderUpdates),
    UserFundings(UserFundings),
    UserNonFundingLedgerUpdates(UserNonFundingLedgerUpdates),
    Notification(Notification),
    WebData2(WebData2),
    ActiveAssetCtx(ActiveAssetCtx),
    Bbo(Bbo),
    Pong,
}

#[derive(Serialize)]
pub struct SubscriptionSendData<'a> {
    pub method: &'static str,
    pub subscription: &'a serde_json::Value,
}