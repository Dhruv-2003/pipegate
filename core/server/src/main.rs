use alloy::{primitives::U256, providers::ProviderBuilder};
use axum::{routing::get, Router};
use super_mario_luigi::{channel::ChannelState, middleware::auth_middleware};

#[tokio::main]
pub async fn main() {
    // a mock server implementation using axum
    // build our application with a route

    // add middleware we created for protecting routes
    // Create a new instance of our state
    let rpc_url = "https://eth.merkle.io".parse().unwrap();
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // NOTE: Re check on this to work around payment amount
    let payment_amount = U256::from(1_000_000_000_000_000u128); // 0.001 ETH in wei

    let state = ChannelState::new();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .layer(axum::middleware::from_fn(move |req, next| {
            let state = state.clone();
            auth_middleware(state, payment_amount, req, next)
        }));

    // run our server on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
