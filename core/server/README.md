# PipeGate Server Middleware Documentation

## Overview

The `pipegate` middleware provides server-side verification and payment channel management for the PipeGate protocol. This guide covers the setup and configuration for API providers using the Rust implementation.

NOTE : Only live on Base sepolia ( rpc: "https://base-sepolia-rpc.publicnode.com" )

## Installation

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
pipegate = { version = "0.1.0" }  # PipeGate server middleware
axum = "0.7"                               # Web framework
tokio = { version = "1.0", features = ["full"] }
alloy = { version = "0.1", features = ["full"] }
```

## Basic Setup

### Simple Server Implementation for Payment Channel middleware

```rust
use alloy::{primitives::U256};
use axum::{routing::get, Router};
use pipegate::{channel::ChannelState, middleware::PipegateMiddlewareLayer};

#[tokio::main]
async fn main() {
    // Configure RPC endpoint
    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    // Configure payment amount per request ( not in decimals, parsed down )
    let payment_amount = U256::from(1000); // 0.001 USDC

    // Initialize channel state
    let state = ChannelState::new(rpc_url.clone());

    // Create router with middleware
    let app = Router::new()
        .route("/", get(root))
        .layer(PipegateMiddlewareLayer::new(state.clone(), payment_amount));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
```

### Simple Server Implementation for One Time payment middleware

```rust
use alloy::{primitives::{U256,Address}};
use axum::{routing::get, Router};
use pipegate::{middleware::{onetime_payment_auth_middleware OneTimePaymentMiddlewareState},types::OneTimePaymentConfig,};

#[tokio::main]
async fn main() {
    // Configure RPC endpoint
    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

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

    // Create router with middleware
    let app = Router::new()
        .route("/", get(root))
        .layer(middleware::from_fn_with_state(
                onetime_state,
                onetime_payment_auth_middleware,
            ));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
```

## Closing channel & withdraw

```rust
use pipegate::{
    channel::{close_channel, ChannelState},
    types::PaymentChannel,
};
pub async fn close_and_withdraw(_state: &ChannelState) {
    // Read the payment channel state
    // let payment_channel = state.get_channel(U256::from(1)).await.unwrap();
    //or
    // Define the payment channel
    let payment_channel = PaymentChannel {
        address: Address::from_str("0x4cf93d3b7cd9d50ecfba2082d92534e578fe46f6").unwrap(),
        sender: Address::from_str("0x898d0dbd5850e086e6c09d2c83a26bb5f1ff8c33").unwrap(),
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        balance: U256::from(1000000),
        nonce: U256::from(0),
        expiration: U256::from(1734391330),
        channel_id: U256::from(1),
    };

    // Can be temporarily retrieved from the logs the latest one
    let signature : Signature = Signature::from_str("0x...").unwrap();

    // raw body of the same request
    let raw_body = Bytes::from("0x");

    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");

    let tx_hash = close_channel(
        rpc_url,
        private_key.as_str(),
        &payment_channel,
        &signature,
        raw_body,
    );
}
```

## Verify one time payment tx

Use the `verify_tx` function to verify a one-time payment transaction. The function takes a `SignedPaymentTx` and `OneTimePaymentConfig` as input and returns a `Result` with the verification status or an error.

```rust
async fn verify_onetime_payment_tx() {
    let rpc_url = "https://base-sepolia-rpc.publicnode.com";

    // Payment config for one time payment set by server owner
    let onetime_payment_config = OneTimePaymentConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
        amount: U256::from(1000000), // 1 USDC
        period: U256::from(0),
        rpc_url: rpc_url.to_string(),
    };

    // Example Signed Payment info from request headers
    let signed_payment_tx = SignedPaymentTx {
        signature: PrimitiveSignature::from_str("0xe3ebb83954309b86cc6d27e7e70b5dbcb0447cf79f8d74fc3806a6e814138fb573d3df3c1fcae6fd8fe1dca34ba8bb2748da3b68790df8ce45108016b601c12a1b").unwrap(),
        tx_hash: FixedBytes::<32>::from_str("0xe88140d4787b1305c24961dcef2f7f73d583bb862b3cbde4b7eec854f61a0248").unwrap(),
    };

    let result = verify_tx(signed_payment_tx, onetime_payment_config).await;
    println!("Result: {:?}", result);
}
```

## Verification functions

1. [Verify and Update Channel State](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/verify.rs#verify_and_update_channel): `verify_and_update_channel` - Verifies the signed request and updates the channel state. `ChannelState` is required to be persisted, better suited for serverful applications.

2. [Verify Channel](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/verify.rs#verify_channel): `verify_channel` - Verifies the signed request and returns the updated channel. `ChannelState` is not required to be persisted, better suited for serverless applications, but would add latency due to extra RPC calls to verify the channel info on each call.

## Helper functions

1. [Parse headers for payment channel with axum](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/utils/headers.rs#parse_headers): `parse_headers` - Parsing and extracting signed request with payment channel from request headers

2. [Modify headers for updated channel with axum](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/utils/headers.rs#modify_headers_axum): `modify_headers_axum` - Modifying response headers with updated channel state in axum.

3. [Modify headers for updated channel with HTTP](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/utils/headers.rs#modify_headers):
   `modify_headers` - Modifying response headers with updated channel state in HTTP.

4. [Parse headers for onetime payment tx](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/utils/headers.rs#parse_tx_headers_axum): `parse_tx_headers_axum` - Parsing and extracting signed request with onetime payment tx from request headers

## Error Handling

```rust
use pipegate::errors::AuthError;

async fn handle_request() -> Result<Response, AuthError> {
    match process_request().await {
        Ok(response) => Ok(response),
        Err(AuthError::InsufficientBalance) => {
            // Handle insufficient balance
        },
        Err(AuthError::InvalidSignature) => {
            // Handle invalid signature
        },
        Err(AuthError::ChannelExpired) => {
            // Handle expired channel
        },
        Err(e) => {
            // Handle other errors
        }
    }
}
```

## WASM Compatibility

The PipeGate middleware can be compiled to WebAssembly (WASM) for use in browser-based applications. The middleware can be compiled using the `wasm-pack` tool and integrated into web applications using JavaScript. Only a subset of functions are exported currently with `wasm-bindgen` and can be found in [wasm.rs](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/wasm.rs). But other functions which are available can be easily exported as needed.

```bash
wasm-pack build --target web
```

Example usage can be found at [tests/index.ts](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/tests/index.ts)

## Middleware Configuration Options

### Environment Variables

```bash
# .env
RPC_URL=https://base-sepolia-rpc.publicnode.com
MIN_PAYMENT_AMOUNT=1000
CHANNEL_FACTORY_ADDRESS=0x...
```

### Loading Configuration

```rust
use dotenv::dotenv;
use std::env;

async fn load_config() -> MiddlewareConfig {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL")
        .expect("RPC_URL must be set");

    let min_payment = env::var("MIN_PAYMENT_AMOUNT")
        .map(|s| U256::from_dec_str(&s).unwrap())
        .expect("MIN_PAYMENT_AMOUNT must be set");

    // Return configuration
    MiddlewareConfig {
        rpc_url: rpc_url.parse().unwrap(),
        min_payment,
        // Other config options
    }
}
```

## Best Practices

1. **Security**

   - Always verify signatures and nonces
   - Implement rate limiting
   - Monitor for suspicious activity
   - Keep RPC endpoint secure

2. **Performance**

   - Use connection pooling for RPC calls
   - Implement caching for channel states
   - Monitor middleware performance

3. **Maintenance**

   - Regularly update dependencies
   - Monitor channel states
   - Implement proper logging
   - Set up monitoring and alerts

4. **Error Handling**
   - Implement comprehensive error handling
   - Provide meaningful error messages
   - Log errors appropriately
   - Handle edge cases

<!--
## Example Implementation with Logging and Monitoring

```rust
use tracing::{info, error, Level};
use metrics::{counter, gauge};

async fn setup_server() {
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Initialize metrics
    let state = ChannelState::new(rpc_url.clone());

    let app = Router::new()
        .route("/", get(root))
        .layer(axum::middleware::from_fn(move |req, next| {
            let state = state.clone();

            // Track metrics
            counter!("request_count", 1);

            async move {
                let start = std::time::Instant::now();
                let result = auth_middleware(state, payment_amount, req, next).await;

                // Record request duration
                gauge!("request_duration", start.elapsed().as_secs_f64());

                result
            }
        }));

    // Start server
    info!("Starting server on port 3000");
    axum::serve(listener, app).await.unwrap();
}
``` -->

**Note**: This middleware is part of the PipeGate protocol. Ensure you're using compatible versions of both client SDK and server middleware.

## Credits & Refrences

- [mahmudsudo/crypto_axum](https://github.com/mahmudsudo/crypto_axum)
