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
- Real-time balance updates
- No API keys needed - just connect your wallet
- Self-service API monetization
- Pay-as-you-go pricing model

**This solves three critical problems:**

- High gas fees for micropayments
- Complex API key management
- Separate payment and access control systems

## Demo

- [Youtube Video](https://youtu.be/8KZ1sLNRUwY)

## How it's made

PipeGate is built using a stack of modern Web3 technologies and standard web protocols:

**Core Components:**

1. Smart Contracts (Solidity):

   - Payment Channel Factory for channel creation
   - Channel contracts for handling payments
   - Proxy pattern for upgradability

2. Client SDK (TypeScript):

   - Built with ethers.js for blockchain interactions
   - Uses viem for efficient byte encoding
   - Axios interceptors for seamless request modification
   - State management for channel tracking

3. Server Middleware (Rust):
   - High-performance signature verification
   - Payment channel state management
   - Request validation and processing

**Technical Innovations:**

1. Request Signing:

   - Custom message encoding using ABI encoder
   - Efficient byte concatenation for headers
   - Timestamp and nonce management for security

2. Payment Channels:

   - Off-chain state management
   - On-chain settlement
   - Automatic nonce tracking
   - Real-time balance updates

3. Middleware Architecture:
   - Interceptor-based design for easy integration
   - Automatic header injection
   - State synchronization between client and server


With this project, we've tried to make the complex payment channel system completely invisible to both API providers and consumers, while maintaining security and efficiency.

## Team 
- [Dhruv Agarwal](https://bento.me/0xdhruv) - Server Side SDK & Smart Contract Development
- [Kushagra Sarathe](https://bento.me/kushagrasarathe) - CLient Side SDK
