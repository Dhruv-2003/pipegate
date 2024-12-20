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
alloy = { version = "0.1", features = ["providers"] }
```

## Basic Setup

### Simple Server Implementation

```rust
use alloy::{primitives::U256, providers::ProviderBuilder};
use axum::{routing::get, Router};
use pipegate::{channel::ChannelState, middleware::auth_middleware};

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
        .layer(axum::middleware::from_fn(move |req, next| {
            let state = state.clone();
            auth_middleware(state, payment_amount, req, next)
        }));

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
use pipegate::errors::PaymentError;
pub async fn close_and_withdraw(_state: &ChannelState) {

    // Read the payment channel state
    // let payment_channel = state.get_channel(U256::from(1)).await.unwrap();
    //or
    // Define the payment channe
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

## Error Handling

```rust
use pipegate::errors::PaymentError;

async fn handle_request() -> Result<Response, PaymentError> {
    match process_request().await {
        Ok(response) => Ok(response),
        Err(PaymentError::InsufficientBalance) => {
            // Handle insufficient balance
        },
        Err(PaymentError::InvalidSignature) => {
            // Handle invalid signature
        },
        Err(PaymentError::ChannelExpired) => {
            // Handle expired channel
        },
        Err(e) => {
            // Handle other errors
        }
    }
}
```

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
```

**Note**: This middleware is part of the PipeGate protocol. Ensure you're using compatible versions of both client SDK and server middleware.

## Credits & Refrences

- [mahmudsudo/crypto_axum](https://github.com/mahmudsudo/crypto_axum)
