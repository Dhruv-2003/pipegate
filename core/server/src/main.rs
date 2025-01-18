use std::{env, str::FromStr};

use alloy::primitives::{Address, Bytes, U256};
use axum::{routing::get, Router};

use pipegate::{
    channel::{close_channel, ChannelState},
    types::PaymentChannel,
};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() {
    use alloy::primitives::aliases::I96;
    use axum::middleware;
    use pipegate::{
        middleware::{
            onetime_payment_auth_middleware, superfluid_streams_auth_middleware,
            OneTimePaymentMiddlewareState, PipegateMiddlewareLayer,
            SuperfluidStreamsMiddlewareState,
        },
        types::{tx::StreamsConfig, OneTimePaymentConfig},
    };

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

    let onetime_payment_config = OneTimePaymentConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
        amount: U256::from(1000000), // 1 USDC
        period: U256::from(0),
        rpc_url: rpc_url.to_string(),
    };

    let onetime_state = OneTimePaymentMiddlewareState {
        config: onetime_payment_config,
    };

    let stream_payment_config = StreamsConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x1650581f573ead727b92073b5ef8b4f5b94d1648").unwrap(),
        amount: "76103500761".parse::<I96>().unwrap(), // 2 USDC per month
        cfa_forwarder: Address::from_str("0xcfA132E353cB4E398080B9700609bb008eceB125").unwrap(),
        rpc_url: rpc_url.to_string(),
    };

    let stream_state = SuperfluidStreamsMiddlewareState {
        config: stream_payment_config,
    };

    let app = Router::new()
        .route(
            "/",
            get(root).route_layer(PipegateMiddlewareLayer::new(state.clone(), payment_amount)),
        )
        .route(
            "/one-time",
            get(one_time).route_layer(middleware::from_fn_with_state(
                onetime_state,
                onetime_payment_auth_middleware,
            )),
        )
        .route(
            "/stream",
            get(stream).route_layer(middleware::from_fn_with_state(
                stream_state,
                superfluid_streams_auth_middleware,
            )),
        );

    // run our server on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Listening on: http://localhost:3000");
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn one_time() -> &'static str {
    "One Time Payment Authenticated"
}

async fn stream() -> &'static str {
    "Stream Payment Authenticated"
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn close_and_withdraw(_state: &ChannelState) {
    // let payment_channel = state.get_channel(U256::from(1)).await.unwrap();

    use alloy::primitives::PrimitiveSignature;

    let payment_channel = PaymentChannel {
        address: Address::from_str("0x4cf93d3b7cd9d50ecfba2082d92534e578fe46f6").unwrap(),
        sender: Address::from_str("0x898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33").unwrap(),
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        balance: U256::from(1000000),
        nonce: U256::from(0),
        expiration: U256::from(1734391330),
        channel_id: U256::from(1),
    };

    let signature  = PrimitiveSignature::from_str("0x9dbbaab8fb419ad1fc50d2d7d0c037f6621d8fc22701b92c503d80e262081d2a11343599127d064b9ca054cd0ae29c7025394f658b47b4c5c102bfd631d7bcb91b").unwrap();

    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");

    let raw_body = Bytes::from("0x");

    let tx_hash = close_channel(
        rpc_url,
        private_key.as_str(),
        &payment_channel,
        &signature,
        raw_body,
    );

    println!("Transaction Hash: {:?}", tx_hash.await);
}
