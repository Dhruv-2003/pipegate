use axum::{routing::get, Router};
use pipegate::{
    channel::ChannelState,
    middleware::PipegateMiddlewareLayer,
    types::{Url, U256},
};

#[tokio::main]
pub async fn main() {
    let rpc_url: Url = "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let state = ChannelState::new(rpc_url.clone());
    let payment_amount = U256::from(1000); // 0.001 USDC in this case

    let app = Router::new()
        .route("/", get(root))
        .layer(PipegateMiddlewareLayer::new(state, payment_amount));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Listening on: http://localhost:3000");
}

async fn root() -> &'static str {
    "Hello, World!"
}
