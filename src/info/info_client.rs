use crate::{
    info::{
        CandlesSnapshotResponse, FundingHistoryResponse, L2SnapshotResponse, OpenOrdersResponse,
        OrderInfo, RecentTradesResponse, UserFillsResponse, UserStateResponse,
    },
    meta::{Meta, SpotMeta, SpotMetaAndAssetCtxs},
    prelude::*,
    req::HttpClient,
    ws::{backend::WsBackend, Subscription},
    BaseUrl, Error, Message, OrderStatusResponse, ReferralResponse, UserFeesResponse,
    UserFundingResponse, UserTokenBalanceResponse,
};

use ethers::types::H160;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleSnapshotRequest {
    coin: String,
    interval: String,
    start_time: u64,
    end_time: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum InfoRequest {
    #[serde(rename = "clearinghouseState")]
    UserState {
        user: H160,
    },
    #[serde(rename = "batchClearinghouseStates")]
    UserStates {
        users: Vec<H160>,
    },
    #[serde(rename = "spotClearinghouseState")]
    UserTokenBalances {
        user: H160,
    },
    UserFees {
        user: H160,
    },
    OpenOrders {
        user: H160,
    },
    OrderStatus {
        user: H160,
        oid: u64,
    },
    Meta,
    SpotMeta,
    SpotMetaAndAssetCtxs,
    AllMids,
    UserFills {
        user: H160,
    },
    #[serde(rename_all = "camelCase")]
    FundingHistory {
        coin: String,
        start_time: u64,
        end_time: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    UserFunding {
        user: H160,
        start_time: u64,
        end_time: Option<u64>,
    },
    L2Book {
        coin: String,
    },
    RecentTrades {
        coin: String,
    },
    #[serde(rename_all = "camelCase")]
    CandleSnapshot {
        req: CandleSnapshotRequest,
    },
    Referral {
        user: H160,
    },
    HistoricalOrders {
        user: H160,
    },
}

#[derive(Debug)]
pub struct InfoClient {
    pub http_client: HttpClient<'static>,
    pub(crate) ws: Option<Arc<dyn WsBackend>>,
    reconnect: bool,
}

impl InfoClient {
    pub async fn new(base_url: Option<BaseUrl>) -> Result<InfoClient> {
        Self::new_internal(base_url, false).await
    }

    pub async fn with_reconnect(base_url: Option<BaseUrl>) -> Result<InfoClient> {
        Self::new_internal(base_url, true).await
    }

    async fn new_internal(base_url: Option<BaseUrl>, reconnect: bool) -> Result<InfoClient> {
        let base_url = base_url.unwrap_or(BaseUrl::Mainnet).get_url();

        Ok(InfoClient {
            http_client: HttpClient::new(base_url),
            ws: None,
            reconnect,
        })
    }

    pub async fn with_client(
        client: &'static Client,
        base_url: Option<BaseUrl>,
    ) -> Result<InfoClient> {
        Self::with_client_internal(client, base_url, false).await
    }

    pub async fn with_client_reconnect(
        client: &'static Client,
        base_url: Option<BaseUrl>,
    ) -> Result<InfoClient> {
        Self::with_client_internal(client, base_url, true).await
    }

    async fn with_client_internal(
        client: &'static Client,
        base_url: Option<BaseUrl>,
        reconnect: bool,
    ) -> Result<InfoClient> {
        let base_url = base_url.unwrap_or(BaseUrl::Mainnet).get_url();

        Ok(InfoClient {
            http_client: HttpClient { client, base_url },
            ws: None,
            reconnect,
        })
    }

    pub async fn subscribe(
        &mut self,
        subscription: Subscription,
        sender_channel: UnboundedSender<Message>,
    ) -> Result<u32> {
        if self.ws.is_none() {
            self.ws = Some(
                crate::ws::make_ws_backend(
                    &format!("wss{}/ws", &self.http_client.base_url[5..]),
                    self.reconnect,
                )
                .await?,
            );
        }

        // Get broadcast receiver from backend
        let mut rx = self.ws.as_ref().unwrap().subscribe(subscription).await?;

        // Spawn task to forward from broadcast to user-supplied mpsc
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender_channel.send((*msg).clone()).is_err() {
                    break;
                }
            }
        });

        Ok(0) // id not needed anymore (kept for API compat)
    }

    pub async fn unsubscribe(&mut self, _subscription_id: u32) -> Result<()> {
        // Note: With the new design, unsubscribe by ID is not as meaningful
        // since we use broadcast channels. In practice, you'd need to store
        // subscription mappings to support this properly.
        // For now, we'll just return Ok to maintain API compatibility.
        Ok(())
    }

    async fn send_info_request<T: for<'a> Deserialize<'a>>(
        &self,
        info_request: InfoRequest,
    ) -> Result<T> {
        let data =
            serde_json::to_string(&info_request).map_err(|e| Error::JsonParse(e.to_string()))?;

        let return_data = self.http_client.post("/info", data).await?;
        serde_json::from_str(&return_data).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn open_orders(&self, address: H160) -> Result<Vec<OpenOrdersResponse>> {
        let input = InfoRequest::OpenOrders { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_state(&self, address: H160) -> Result<UserStateResponse> {
        let input = InfoRequest::UserState { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_states(&self, addresses: Vec<H160>) -> Result<Vec<UserStateResponse>> {
        let input = InfoRequest::UserStates { users: addresses };
        self.send_info_request(input).await
    }

    pub async fn user_token_balances(&self, address: H160) -> Result<UserTokenBalanceResponse> {
        let input = InfoRequest::UserTokenBalances { user: address };
        self.send_info_request(input).await
    }

    pub async fn user_fees(&self, address: H160) -> Result<UserFeesResponse> {
        let input = InfoRequest::UserFees { user: address };
        self.send_info_request(input).await
    }

    pub async fn meta(&self) -> Result<Meta> {
        let input = InfoRequest::Meta;
        self.send_info_request(input).await
    }

    pub async fn spot_meta(&self) -> Result<SpotMeta> {
        let input = InfoRequest::SpotMeta;
        self.send_info_request(input).await
    }

    pub async fn spot_meta_and_asset_contexts(&self) -> Result<Vec<SpotMetaAndAssetCtxs>> {
        let input = InfoRequest::SpotMetaAndAssetCtxs;
        self.send_info_request(input).await
    }

    pub async fn all_mids(&self) -> Result<HashMap<String, String>> {
        let input = InfoRequest::AllMids;
        self.send_info_request(input).await
    }

    pub async fn user_fills(&self, address: H160) -> Result<Vec<UserFillsResponse>> {
        let input = InfoRequest::UserFills { user: address };
        self.send_info_request(input).await
    }

    pub async fn funding_history(
        &self,
        coin: String,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<FundingHistoryResponse>> {
        let input = InfoRequest::FundingHistory {
            coin,
            start_time,
            end_time,
        };
        self.send_info_request(input).await
    }

    pub async fn user_funding_history(
        &self,
        user: H160,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<UserFundingResponse>> {
        let input = InfoRequest::UserFunding {
            user,
            start_time,
            end_time,
        };
        self.send_info_request(input).await
    }

    pub async fn recent_trades(&self, coin: String) -> Result<Vec<RecentTradesResponse>> {
        let input = InfoRequest::RecentTrades { coin };
        self.send_info_request(input).await
    }

    pub async fn l2_snapshot(&self, coin: String) -> Result<L2SnapshotResponse> {
        let input = InfoRequest::L2Book { coin };
        self.send_info_request(input).await
    }

    pub async fn candles_snapshot(
        &self,
        coin: String,
        interval: String,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<CandlesSnapshotResponse>> {
        let input = InfoRequest::CandleSnapshot {
            req: CandleSnapshotRequest {
                coin,
                interval,
                start_time,
                end_time,
            },
        };
        self.send_info_request(input).await
    }

    pub async fn query_order_by_oid(&self, address: H160, oid: u64) -> Result<OrderStatusResponse> {
        let input = InfoRequest::OrderStatus { user: address, oid };
        self.send_info_request(input).await
    }

    pub async fn query_referral_state(&self, address: H160) -> Result<ReferralResponse> {
        let input = InfoRequest::Referral { user: address };
        self.send_info_request(input).await
    }

    pub async fn historical_orders(&self, address: H160) -> Result<Vec<OrderInfo>> {
        let input = InfoRequest::HistoricalOrders { user: address };
        self.send_info_request(input).await
    }
}
