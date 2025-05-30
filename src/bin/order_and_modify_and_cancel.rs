use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientModifyRequest, ClientOrder,
    ClientOrderRequest, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // Place initial order
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };

    // Wait to see the order
    sleep(Duration::from_secs(5));

    // Modify the order price
    let modify_order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1750.0, // Lower price
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let modify = ClientModifyRequest {
        oid,
        order: modify_order,
    };

    let response = exchange_client.modify(modify, None).await.unwrap();
    info!("Order modified: {response:?}");

    // Wait to see the modified order
    sleep(Duration::from_secs(5));

    // Cancel the order
    let cancel = ClientCancelRequest {
        asset: "ETH".to_string(),
        oid,
    };

    let response = exchange_client.cancel(cancel, None).await.unwrap();
    info!("Order cancelled: {response:?}");
}
