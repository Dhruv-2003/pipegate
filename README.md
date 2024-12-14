# PipeGate - ETHBangkok 2024

<!-- Pay-per-Call API Monetisation - The Web3 Stripe for APIs -->

The Web3 Stripe for APIs. Create payment channels, make API calls, payments happen automatically. No API keys, no gas fees per request, just connect wallet and start building.

<img width="952" alt="Screenshot 2024-11-17 at 12 32 48 AM" src="https://github.com/user-attachments/assets/fe1b3926-224d-48e6-8dea-44214e471406">

## Description

PipeGate is a decentralized API monetization protocol that changes how APIs handle payments and access control. By replacing traditional API keys with payment channels, it enables true pay-per-call pricing without gas fees for each request.

**The protocol consists of three main components:**

- A client-side middleware that automatically handles payment channel creation, request signing, and payment management
- A server-side middleware that verifies signatures and manages payment channel states
- A smart contract for a new payment channel creation

**Key Features:**

- Gasless microtransactions using payment channels
- Automatic request signing and payment handling
- No API keys needed - just your wallet
- Real-time balance updates
- Self-served onboarding 

**This solves three critical problems:**

- Too many API keys
- Complex API & Auth management for providers
- Payment gateway fees
  
## Demo

- [Youtube Video](https://youtu.be/8KZ1sLNRUwY)

## How it's made

PipeGate is built using a stack of modern Web3 technologies and standard web protocols:

**Core Components:**

1. [Smart Contracts (Solidity)](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract):

   - Payment Channel Factory for channel creation
   - Channel contracts for handling payments
   - Beacon Proxy pattern for low deployment fees

2. [Client SDK (TypeScript)](https://github.com/Dhruv-2003/pipegate/tree/main/core/client):

   - Built with viem for blockchain interactions
   - Axios interceptors for seamless request modification
   - State management for channel tracking

3. [Server Middleware (Rust)](https://github.com/Dhruv-2003/pipegate/tree/main/core/server):

   - Signature verification
   - Payment channel state management

## Architecture & Flow

<img width="983" alt="Screenshot 2024-12-12 at 12 09 55 PM" src="https://github.com/user-attachments/assets/f17b00a5-08f1-40d5-965d-7435559ca799" />

## Publish SDKs & Libraries

- [Rust crate](https://crates.io/crates/pipegate)
- [TypeScript SDK](https://www.npmjs.com/package/pipegate-sdk)

## How to use

### For API Providers

1. **Register your API:**

   - Add your pricing info to ChannelFactory by registering yourselves.
   - Can directly interact with contract using cast [here](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract#for-api-providers)
   - Or using a scripts [here](https://github.com/Dhruv-2003/pipegate/blob/main/example/ts/scripts/1_registerAsProvider.ts)

2. **Add the server-side middleware:**

   - Add the PipeGate server middleware to your API server
   - Supported with axum in rust [lib](https://github.com/Dhruv-2003/pipegate/tree/main/core/server)

3. **Close the channel & withdraw**

   - Directly interact with contract using cast with the [command](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract#for-api-providers)
   - Using the rust library as well [example](https://github.com/Dhruv-2003/pipegate/tree/main/core/server#closing-channel--withdraw)

### For API Consumers

1. **Create a payment channel:**

   - Use the client-side SDK to create a payment channel with this [script](https://github.com/Dhruv-2003/pipegate/blob/main/example/ts/scripts/2_createChannel.ts)
   - Supported with sdk in typescript [lib](https://github.com/Dhruv-2003/pipegate/tree/main/core/client)
   - Or directly interact with contract using cast [here](https://github.com/Dhruv-2003/pipegate/tree/main/core/contract#for-api-consumers)

2. **Make API calls:**

   - Use the client-side SDK to add interceptor to your axios instance with this [sdk](https://github.com/Dhruv-2003/pipegate/tree/main/core/client)

With this project, we've tried to make the complex payment channel system completely invisible to both API providers and consumers, while maintaining security and efficiency.

## Team

- [Dhruv Agarwal](https://bento.me/0xdhruv) - Server Side SDK & Smart Contract Development
- [Kushagra Sarathe](https://bento.me/kushagrasarathe) - CLient Side SDK
