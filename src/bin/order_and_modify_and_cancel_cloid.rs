use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequestCloid, ClientLimit, ClientModifyRequestCloid, ClientOrder,
    ClientOrderRequest, ExchangeClient,
};
use std::{thread::sleep, time::Duration};
use uuid::Uuid;

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

    // Use the same UUID for order, modify, and cancel operations
    let cloid = Uuid::new_v4();
    info!("Using CLOID: {cloid}");

    // Place initial order
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.01,
        cloid: Some(cloid),
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    // Wait to see the order
    sleep(Duration::from_secs(5));

    // Modify the order price using CLOID
    let modify_order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1750.0, // Lower price
        sz: 0.01,
        cloid: Some(cloid), // Same CLOID
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let modify = ClientModifyRequestCloid {
        cloid,
        order: modify_order,
    };

    let response = exchange_client.modify_by_cloid(modify, None).await.unwrap();
    info!("Order modified: {response:?}");

    // Wait to see the modified order
    sleep(Duration::from_secs(5));

    // Cancel the order using CLOID
    let cancel = ClientCancelRequestCloid {
        asset: "ETH".to_string(),
        cloid,
    };

    let response = exchange_client.cancel_by_cloid(cancel, None).await.unwrap();
    info!("Order cancelled: {response:?}");
}
