use log::info;

use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut info_client = InfoClient::new(Some(BaseUrl::Testnet)).await.unwrap();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(
            Subscription::Bbo {
                coin: "ETH".to_string(),
            },
            sender,
        )
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from BBO data");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    // This loop ends when we unsubscribe
    while let Some(Message::Bbo(bbo)) = receiver.recv().await {
        info!("Received BBO data: {bbo:?}");
    }
}
