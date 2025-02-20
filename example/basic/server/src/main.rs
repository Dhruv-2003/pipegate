use std::str::FromStr;

use axum::{routing::get, Router};
use pipegate::{
    middleware::stream_payment::{
        state::StreamState, types::StreamsConfig, StreamListner, StreamMiddlewareLayer,
    },
    utils::{Address, Url, I96},
};

#[tokio::main]
async fn main() {
    // Configure RPC endpoint
    let rpc_url: Url = "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let stream_payment_config = StreamsConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x1650581f573ead727b92073b5ef8b4f5b94d1648").unwrap(),
        amount: "761035007610".parse::<I96>().unwrap(), // 2 USDC per month
        cfa_forwarder: Address::from_str("0xcfA132E353cB4E398080B9700609bb008eceB125").unwrap(),
        rpc_url: rpc_url.to_string(),
        cache_time: 900,
    };

    let stream_state = StreamState::new();
    let stream_state_clone = stream_state.clone();
    let stream_payment_config_clone = stream_payment_config.clone();

    // Create router with middleware
    let app = Router::new().route(
        "/",
        get(root).layer(StreamMiddlewareLayer::new(
            stream_payment_config,
            stream_state,
        )),
    );

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let stream_listener = tokio::spawn(async move {
        println!("Spawning event listener ...");
        if let Err(e) = StreamListner::start(stream_state_clone, stream_payment_config_clone).await
        {
            eprintln!("Event listener error: {:?}", e);
        }
    });

    axum::serve(listener, app).await.unwrap();
    stream_listener.await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
