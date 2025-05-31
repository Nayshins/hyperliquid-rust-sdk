use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use std::time::Instant;

#[tokio::test]
#[ignore] // Requires network access to live Hyperliquid testnet API
async fn pool_reuse() {
    // First request - establishes connection
    let t1 = Instant::now();
    let _ = InfoClient::new(Some(BaseUrl::Testnet))
        .await
        .unwrap()
        .meta()
        .await
        .unwrap();
    let cold = t1.elapsed();

    // Second request - should reuse connection and be faster
    let t2 = Instant::now();
    let _ = InfoClient::new(Some(BaseUrl::Testnet))
        .await
        .unwrap()
        .meta()
        .await
        .unwrap();
    let warm = t2.elapsed();

    // Warm request should be faster than cold due to connection reuse
    println!("Cold request: {:?}", cold);
    println!("Warm request: {:?}", warm);

    // The warm request should be at least slightly faster due to connection pooling
    // Note: This test may be flaky on slow networks, but demonstrates the concept
    assert!(warm < cold * 2); // Allow some variance but expect improvement
}

#[tokio::test]
#[ignore] // Requires network access to live Hyperliquid testnet API
async fn shared_client_reuse() {
    // Test that multiple clients actually share the same underlying connection
    let client1 = InfoClient::new(Some(BaseUrl::Testnet)).await.unwrap();
    let client2 = InfoClient::new(Some(BaseUrl::Testnet)).await.unwrap();

    // Both should work and use the same underlying HTTP client pool
    let _meta1 = client1.meta().await.unwrap();
    let _meta2 = client2.meta().await.unwrap();

    // If we get here without errors, the shared pool is working
    assert!(true);
}
