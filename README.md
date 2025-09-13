# PipeGate

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Dhruv-2003/pipegate)

<!-- Pay-per-Call API Monetisation - The Web3 Stripe for APIs -->

The Web3 Stripe for APIs. Create payment channels or streams, make API calls, payments happen automatically. No API keys, no gas fees per request, just use your wallet and start building.

<img width="952" alt="Screenshot 2024-11-17 at 12 32 48 AM" src="https://github.com/user-attachments/assets/fe1b3926-224d-48e6-8dea-44214e471406">

## Description

PipeGate transforms how APIs handle payments by replacing traditional API keys with stablecoin payments and clever cryptography. Instead of managing countless API keys and dealing with complex billing systems, developers can simply connect their wallet and pay for API usage through payment channels, streams, or one-time transactions.

### 🆕 Now with x402 Standard Support

As of version 0.6.0, PipeGate implements the [x402 payment protocol](https://x402.org), providing standardized payment headers and seamless integration across different payment schemes. This means better interoperability and a unified payment experience.

### Detailed documentation: [docs.pipegate.xyz](https://docs.pipegate.xyz)

**The protocol consists of three main components:**

- A client-side middleware that automatically handles payment channel creation, request signing, and payment management
- A server-side middleware that verifies signatures and manages state with minimal integration effort
- A smart contract for a new payment channel creation

**What you get:**

- Pay-per-call pricing without gas fees for each request
- Multiple payment options: channels, streams, or one-time payments
- No API key management - just connect your wallet
- Real-time usage tracking and automatic payments
- Self-service onboarding for both providers and consumers

**Problems solved:**

- Too many API keys for each product
- Complex API & Authentication infrastructure for providers
- High payment processor fees eating into margin

## Demo

- [With Payment channels](https://youtu.be/8KZ1sLNRUwY)
- [With Streams](https://www.youtube.com/live/lxjodEw3YQo?si=R7FWGjJ0uCrenwqH&t=410)

## How it's made

PipeGate is built with a focus on developer experience and standards compliance, implementing the x402 payment protocol for maximum interoperability.

**Core Components:**

1. **[Server Middleware (Rust)](https://github.com/Dhruv-2003/pipegate/tree/main/core/server)**:

   - Unified `PaymentsLayer` supporting all payment schemes (v0.6.0+)
   - x402-compliant header parsing and verification
   - Automatic scheme detection (one-time, streams, channels)
   - WASM compatibility for browser environments
   - Legacy per-scheme middleware for backward compatibility

2. **[Client SDK (TypeScript)](https://github.com/Dhruv-2003/pipegate/tree/main/core/client)**:

   - Single `withPaymentInterceptor` function for all payment types
   - Automatic 402 Payment Required handling and retry logic
   - x402 standard compliant payment headers
   - Axios interceptors with state management
   - Legacy interceptors available for migration

3. **[Smart Contracts (Solidity)](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract)**:
   - Payment Channel Factory with provider registration
   - Efficient channel contracts using Beacon Proxy pattern
   - Integration with Superfluid for streaming payments
   - One-time payment verification through transaction logs

**Payment Schemes Supported:**

- **Payment Channels**: Gasless microtransactions with off-chain state updates
- **Superfluid Streams**: Continuous payment flows for subscription-like access
- **One-time Payments**: Simple pay-per-request using on-chain transactions

**x402 Integration:**
All payment schemes follow the x402 standard with unified `X-Payment` headers containing `{ x402Version, network, scheme, payload }`, making PipeGate compatible with other x402-compliant services.

## x402 Standard Implementation

PipeGate implements the [x402 payment protocol](https://x402.org) for standardized API payment flows:

**Payment Flow:**

1. Client requests API endpoint
2. Server responds with `402 Payment Required` containing accepted payment schemes
3. Client automatically selects scheme, signs payment, and retries with `X-Payment` header
4. Server verifies payment and processes request

**Supported Schemes:**

- `one-time`: Pay-per-request using transaction hashes
- `stream`: Continuous payments via Superfluid streams
- `channel`: Gasless microtransactions through payment channels

See our [x402 implementation spec](./x402.md) for detailed payment header formats.

## Architecture & Flow

### With Payment channels

<img width="983" alt="Screenshot 2024-12-12 at 12 09 55 PM" src="https://github.com/user-attachments/assets/9ab25e8b-35b2-4f9e-a131-166e80643bf7" />

### With Streams

<img width="988" alt="Screenshot 2025-01-23 at 10 02 26 PM" src="https://github.com/user-attachments/assets/0ad5a98f-c8a6-4c03-bf11-8618db3cb22f" />

## Published Libraries

**Latest (v0.6.0+) - x402 Standard Support:**

- [Rust Crate](https://crates.io/crates/pipegate) - Unified server middleware
- [TypeScript SDK](https://www.npmjs.com/package/pipegate-sdk) - Universal client interceptor

Both libraries support all payment schemes through a single unified API, replacing the need for separate per-scheme implementations.

## Quick Start

### For API Providers

**1. Add server middleware (Recommended - x402 unified approach)**

```rust
use pipegate::middleware::{PaymentsLayer, PaymentsState, Scheme, SchemeConfig, MiddlewareConfig};

// Support multiple payment schemes with one middleware
let config = MiddlewareConfig::new(vec![
    SchemeConfig::new(Scheme::OneTimePayments, "1".to_string()).await,
    SchemeConfig::new(Scheme::SuperfluidStreams, "2".to_string()).await,
    SchemeConfig::new(Scheme::PaymentChannels, "0.001".to_string()).await,
]);

let app = Router::new()
    .route("/api", get(handler))
    .layer(PaymentsLayer::new(PaymentsState::new(), config));
```

**2. Register your API** (for payment channels)

- Add pricing info to the ChannelFactory contract
- [Registration guide](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract#for-api-providers)

### For API Consumers

**x402 Unified Client (Recommended)**

```typescript
import { withPaymentInterceptor } from "pipegate-sdk";

// Works with any payment scheme
const client = withPaymentInterceptor(
  axios.create({ baseURL: "https://api.example.com" }),
  PRIVATE_KEY,
  { oneTimePaymentTxHash: "0x..." } // or streamSender, or channel
);

// Automatic payment handling
const response = await client.get("/api/endpoint");
```

**Legacy usage instructions and detailed setup guides are available in our [documentation](https://docs.pipegate.xyz).**

## Team

- [Dhruv Agarwal](https://0xdhruv.me) - Server Side SDK & Smart Contract Development
- [Kushagra Sarathe](https://bento.me/kushagrasarathe) - CLient Side SDK
