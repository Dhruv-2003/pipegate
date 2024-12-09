use alloy::primitives::U256;
use axum::{routing::get, Router};
use super_mario_luigi::{channel::ChannelState, middleware::auth_middleware};

#[tokio::main]
pub async fn main() {
    // a mock server implementation using axum
    // build our application with a route

    // add middleware we created for protecting routes
    // Create a new instance of our state
    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    // Amount is not supposed to be in the decimal format, so parsed with the decimals of that token
    // E.g. if USDC is being used 1USDC = 1000000 after 6 decimal places in case of the USDC token
    let payment_amount = U256::from(1000); // 0.001 USDC in this case

    let state = ChannelState::new(rpc_url.clone());

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

    println!("Listening on: http://localhost:3000");
}

async fn root() -> &'static str {
    "Hello, World!"
}
