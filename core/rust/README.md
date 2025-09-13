# PipeGate Server Middleware Documentation

## Overview

The `pipegate` middleware provides server-side verification and multi-scheme payment enforcement (one-time transfers, Superfluid streams, and payment channels) for the PipeGate protocol.

As of version `0.6.0` a unified middleware `PaymentsLayer` (alias of `PipegateMiddlewareLayer`) replaces the need to stack separate middleware layers. It automatically:

- Parses the `X-Payment` (x402) header
- Detects the payment scheme (`one-time`, `stream`, `channel`)
- Verifies and (if needed) initializes internal state lazily
- Updates response headers (for channels) with refreshed channel data
- Emits a proper `402 Payment Required` x402-formatted error if verification fails

Legacy per-scheme layers (`PaymentChannelMiddlewareLayer`, `OnetimePaymentMiddlewareLayer`, `StreamMiddlewareLayer`) are still documented below but deprecated and will be removed in a future major release. Prefer the unified approach for all new integrations.

NOTE: Payment channel on-chain deployment is currently live on Base Sepolia ( rpc: `https://base-sepolia-rpc.publicnode.com` ). Other schemes support multiple networks as long as supported by their respective infrastructure.

## Installation

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
pipegate = { version = "0.6.0" }  # PipeGate server middleware
axum = "0.7"                               # Web framework
tokio = { version = "1.0", features = ["full"] }
alloy = { version = "0.1", features = ["full"] }
```

**_Note_**: Axum v0.8.0 has breaking changes. The pipegate tower-based middleware layers are not yet supported in the latest version.

---

## Unified Payments Middleware (Preferred in >= 0.6.0)

### Quick Start

```rust
use std::str::FromStr;
use axum::{routing::get, Router};
use alloy::primitives::Address;
use pipegate::middleware::{PaymentsLayer, PaymentsState, Scheme, SchemeConfig, MiddlewareConfig};

#[tokio::main]
async fn main() {
    // Build scheme configurations (async helpers fetch chain id, name, token decimals, etc.)
    let one_time = SchemeConfig::new(
        Scheme::OneTimePayments,
        "https://base-sepolia-rpc.publicnode.com".to_string(),
        Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(), // USDC (example)
        Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(), // recipient
        "1".to_string(), // human amount (will be parsed using decimals)
    ).await;

    let stream = SchemeConfig::new(
        Scheme::SuperfluidStreams,
        "https://base-sepolia-rpc.publicnode.com".to_string(),
        Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(), // underlying token
        Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        "2".to_string(), // monthly amount in whole tokens (converted to flow rate internally)
    ).await;

    let channel = SchemeConfig::new(
        Scheme::PaymentChannels,
        "https://base-sepolia-rpc.publicnode.com".to_string(),
        Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
        Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        "0.001".to_string(), // amount per request (interpreted with token decimals)
    ).await;

    // Aggregate into middleware config
    let config = MiddlewareConfig::new(vec![one_time, stream, channel]);

    // State initializes lazily (internal option fields become Some when first scheme is used)
    let state = PaymentsState::new();

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .layer(PaymentsLayer::new(state, config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str { "ok" }
```

### Request Flow

1. Client sends `X-Payment: { x402Version, scheme, network, payload }` header.
2. Middleware selects matching `SchemeConfig` (or returns `402` if unsupported).
3. Verification path per scheme:
   - one-time: checks on-chain tx + local redemption rules
   - stream: verifies Superfluid flow (with caching & optional listener)
   - channel: verifies signed request & updates channel state, returning updated channel headers
4. Request continues to handler; for channels, response headers are augmented.

### Migrating from v0.5.x Per-Scheme Layers

| Old API (Deprecated)                             | New Unified API                                         |
| ------------------------------------------------ | ------------------------------------------------------- |
| `PaymentChannelMiddlewareLayer::new(state, cfg)` | `PaymentsLayer::new(state, config)`                     |
| `OnetimePaymentMiddlewareLayer::new(cfg, state)` | (Include one-time `SchemeConfig` in `MiddlewareConfig`) |
| `StreamMiddlewareLayer::new(cfg, state)`         | (Include stream `SchemeConfig` in `MiddlewareConfig`)   |
| Multiple `.layer(...)` calls                     | Single `.layer(PaymentsLayer::new(...))`                |

Steps:

1. Build one `MiddlewareConfig` by converting each old per-scheme config into a `SchemeConfig` via `SchemeConfig::new(...)` (async).
2. Replace stacked layers with one `PaymentsLayer` instance.
3. Remove direct uses of individual state types unless you need custom inspection (the unified `PaymentsState` holds them lazily).
4. Update docs/imports to prefer `PaymentsLayer`, `PaymentsState` and `Scheme` enums.
5. Resolve deprecation warnings; the old layers will be removed in a future major release.

### Header Format (x402)

Clients must send a single JSON header:

```
X-Payment: { "x402Version":1, "network":"base-sepolia", "scheme":"one-time", "payload": { ... } }
```

Exact `payload` shape depends on scheme (see original per-scheme sections below for structure reference). The unified middleware internally validates payload vs. scheme and returns `InvalidHeaders` if mismatched.

## Legacy (Deprecated) Middleware Guides

The following sections remain for reference and will be removed after the unified API fully replaces them.

## Basic Setup

### Simple Server Implementation for Payment Channel middleware (Deprecated)

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
    let config = PaymentChannelConfig {
        recipient: Address::from_str("YOUR_ADDRESS").unwrap(),
        token_address: Address::from_str("USDC_ADDRESS").unwrap(),
        amount: U256::from(1000), // 0.001 USDC in this case
        rpc_url: rpc_url.to_string(),
    };

    // Initialize channel state
    let state = ChannelState::new();

    // Create router with middleware
    let app = Router::new()
        .route("/", get(root))
        .layer(PipegateMiddlewareLayer::new(state, config));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
```

### Simple Server Implementation for One Time payment middleware (Deprecated)

```rust
use alloy::{primitives::{U256,Address}};
use axum::{routing::get, Router};
use pipegate::{middleware::{OnetimePaymentMiddlewareLayer},types::OneTimePaymentConfig,};

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

    // Create router with middleware
    let app = Router::new()
        .route("/", get(root))
        .layer(OnetimePaymentMiddlewareLayer::new(onetime_payment_config));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
```

### Simple Server Implementation for Stream middleware (Deprecated)

```rust
use alloy::{primitives::{U256,Address}};
use axum::{routing::get, Router};
use pipegate::{middleware::{StreamMiddlewareLayer,StreamState},types::tx::StreamsConfig};

#[tokio::main]
async fn main() {
    // Configure RPC endpoint
    let rpc_url: alloy::transports::http::reqwest::Url =
        "https://base-sepolia-rpc.publicnode.com".parse().unwrap();

    let stream_payment_config = StreamsConfig {
        recipient: Address::from_str("0x62c43323447899acb61c18181e34168903e033bf").unwrap(),
        token_address: Address::from_str("0x1650581f573ead727b92073b5ef8b4f5b94d1648").unwrap(),
        amount: "761035007610".parse::<I96>().unwrap(), // 2 USDC per month
        cfa_forwarder: Address::from_str("0xcfA132E353cB4E398080B9700609bb008eceB125").unwrap(),
        rpc_url: rpc_url.to_string(),
    };

    let stream_state = StreamState::new();

    // Create router with middleware
    let app = Router::new()
        .route("/", get(root))
        .layer(StreamMiddlewareLayer::new(stream_payment_config,stream_state));

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

1. [Verify and Update Channel State](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/payment_channel/verify.rs#verify_and_update_channel): `verify_and_update_channel` - Verifies the signed request and updates the channel state. `ChannelState` is required to be persisted, better suited for serverful applications.

2. [Verify Channel](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/payment_channel/verify.rs#verify_channel): `verify_channel` - Verifies the signed request and returns the updated channel. `ChannelState` is not required to be persisted, better suited for serverless applications, but would add latency due to extra RPC calls to verify the channel info on each call.

3. [Verify Stream](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/stream_payment/verify.rs#verify_stream): `verify_stream` - Verifies the stream payment request from onchain contracts

4. [Verify Stream via indexer](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/stream_payment/verify.rs#verify_stream_indexer): `verify_stream_indexer` - Verifies the stream payment request using indexer.

5. [Verify one time tx](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/one_time_payment/verify.rs#verify_stream_indexer): `verify_tx` - Verifies the one time payment tx from onchain records and logs

## Helper functions

1. [Parse headers for payment channel](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/payment_channel/utils.rs#parse_headers): `parse_headers` - Parsing and extracting signed request with payment channel from request headers

2. [Modify headers for updated channel with axum](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/payment_channel/utils.rs#modify_headers_axum): `modify_headers_axum` - Modifying response headers with updated channel state in axum.

3. [Modify headers for updated channel with HTTP](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/utils/headers.rs#modify_headers):
   `modify_headers` - Modifying response headers with updated channel state in HTTP.

4. [Parse headers for onetime payment tx](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/one_time_payment/utils.rs#parse_tx_headers): `parse_tx_headers` - Parsing and extracting signed request with onetime payment tx from request headers

5. [Parse headers for stream based request](https://github.com/Dhruv-2003/pipegate/blob/main/core/server/src/middleware/stream_payment/utils.rs#parse_stream_headers): `parse_stream_headers` - Parsing and extracting signed request with onetime payment tx from request headers

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
wasm-pack build --target bundler --no-opt
```

For ESM, web build

```bash
wasm-pack build --target web --no-opt --release --out-dir pkg/web
```

For CJS, nodejs build

```bash
wasm-pack build --target nodejs --no-opt --release --out-dir pkg/nodejs
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
