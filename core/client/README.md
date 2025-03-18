# PipeGate Client SDK Documentation

## Overview

PipeGate SDK enables developers to interact with the PipeGate protocol for API monetization using payment channels with stablecoins. This guide covers SDK installation, configuration, and usage for both API consumers and providers.

## Installation

```bash
npm install pipegate-sdk
# or
yarn add pipegate-sdk
# or
bun add pipegate-sdk
```

## Basic Setup

1. Create a `.env` file in your project root:

```env
WALLET_PRIVATE_KEY=your_private_key_here
```

2. Initialize the SDK:

```typescript
import { ClientInterceptor } from "pipegate-sdk";

// Create SDK instance
const pipeGate = new ClientInterceptor();
await pipeGate.initialize();
```

## Core Features

### Creating a Payment Channel

To start using an API, first create a payment channel & add it:

```typescript
const channelParams = {
  recipient: "0x...", // API provider's address
  duration: 2592000, // Channel duration in seconds (30 days)
  tokenAddress: "0x...", // Payment token address (e.g., USDC)
  amount: "100", // Amount in tokens to deposit (in decimals ) 100 USDC
};

const channel = await pipeGate.createPaymentChannel(channelParams);

await pipeGate.addNewChannel(channel.channelId, channel);
```

### Making API Calls with Payment Channel method

#### Using with Axios

1. Create an Axios instance with PipeGate interceptors:

```typescript
import axios from "axios";

const api = axios.create({
  baseURL: "https://api.example.com",
});

// Add request interceptor for automatic signing
api.interceptors.request.use(
  pipeGate.createPaymentChannelRequestInterceptor(channelId).request
);

// Add response interceptor for state management
api.interceptors.response.use(
  pipeGate.createPaymentChannelResponseInterceptor().response
);

// Make API calls as normal
const response = await api.get("/endpoint");
```

### Monitoring Channel State

```typescript
// Get current channel state
const channelState = pipeGate.getChannelState(channelId);
console.log("Current Balance:", channelState?.balance);
console.log("Channel Status:", channelState?.status);
```

### Adding a new Channel

```typescript
// Get current channel state
await pipeGate.addNewChannel(channel.channelId, channel);
```

### Making API Calls with One time Payment method

#### Using with Axios

Create an Axios instance with PipeGate interceptors and the txHash of the transaction that was used to pay for the request:

```typescript
import axios from "axios";

const api = axios.create({
  baseURL: "https://api.example.com",
});

// Add request interceptor for automatic signing
api.interceptors.request.use(
  pipeGate.createOneTimePaymentRequestInterceptor(txHash).request
);

// Make API calls as normal
const response = await api.get("/endpoint");
```

### Making API Calls with Superfluid streams method

#### Using with Axios

Create an Axios instance with PipeGate interceptors and the txHash of the transaction that was used to pay for the request:

```typescript
import axios from "axios";

const api = axios.create({
  baseURL: "https://api.example.com",
});

const streamSender = "0x898d0DBd5850e086E6C09D2c83A26Bb5F1ff8C33";

// Add request interceptor for automatic signing
api.interceptors.request.use(
  pipeGate.createStreamRequestInterceptor(streamSender).request
);

// Make API calls as normal
const response = await api.get("/endpoint");
```

## Advanced Usage

### Manual Request Signing

If you need to sign requests manually:

```typescript
const channelState = pipeGate.getChannelState(channelId);
if (!channelState) throw new Error("Channel not found");

const requestBody = { foo: "bar" };
const signedRequest = await pipeGate.signRequest(channelState, requestBody);

// Use the signed request in your API call
const response = await fetch("https://api.example.com/endpoint", {
  method: "POST",
  headers: {
    "x-Message": signedRequest.message,
    "x-Signature": signedRequest.signature,
    "x-Timestamp": signedRequest.timestamp,
    "x-Payment": JSON.stringify(channelState),
  },
  body: JSON.stringify(requestBody),
});
```

### WASM exported functions - Verifiers & Extra

`PaymentChannelVerifier` can be used as such:

```typescript
await init();

const rpc_url = "https://base-sepolia-rpc.publicnode.com";

const verifier = new PaymentChannelVerifier(rpc_url);

const updatedChannel = await verifier.verify_request(
  data.message,
  data.signature,
  data.paymentChannelJSON,
  data.paymentAmount,
  data.bodyBytes
);

console.log(updatedChannel);
```

Test script can be found in `tests/wasm.ts`

## Best Practices

1. **Channel Management**

   - Create channels with appropriate duration and funding
   - Monitor channel balance and top up when needed
   - Close channels when they're no longer needed

2. **Error Handling**

   - Always handle potential errors in channel creation
   - Monitor for signature verification failures
   - Handle channel state updates appropriately

3. **Security**
   - Never commit private keys or `.env` files
   - Validate channel states before making requests
   - Keep track of nonces to prevent replay attacks

## Types Reference

```typescript
interface CreateChannelParams {
  recipient: `0x${string}`;
  duration: number;
  tokenAddress: `0x${string}`;
  amount: number;
}

interface CreateChannelResponse {
  channelId: bigint;
  channelAddress: `0x${string}`;
  sender: `0x${string}`;
  recipient: `0x${string}`;
  duration: bigint;
  tokenAddress: `0x${string}`;
  amount: bigint;
  price: bigint;
  timestamp: bigint;
}

interface PaymentChannelResponse {
  address: string;
  sender: string;
  recipient: string;
  balance: string;
  nonce: string;
  expiration: string;
  channel_id: string;
}

interface SignedRequest {
  message: string;
  signature: string;
  timestamp: string;
}
```

## Error Handling

The SDK throws specific errors that should be handled in your application:

```typescript
try {
  await pipeGate.createPaymentChannel(params);
} catch (error) {
  if (error.message.includes("insufficient funds")) {
    // Handle insufficient funds error
  } else if (error.message.includes("invalid recipient")) {
    // Handle invalid recipient error
  } else {
    // Handle other errors
  }
}
```

## Examples

### Complete API Integration Example with recurring Payment Channel method

```typescript
import { ClientInterceptor } from "pipegate-sdk";
import axios from "axios";

async function setupApiClient() {
  // Initialize SDK
  const pipeGate = new ClientInterceptor();
  await pipeGate.initialize();

  // Create payment channel
  const channel = await pipeGate.createPaymentChannel({
    recipient: "0x123...",
    duration: (365 / 12) * 24 * 60 * 60, // 30 days
    tokenAddress: "0x456...",
    amount: "100",
  });

  await pipeGate.addNewChannel(channel.channelId, channel);

  // Setup API client
  const api = axios.create({
    baseURL: "https://api.example.com",
  });

  // Add interceptors
  api.interceptors.request.use(
    pipeGate.createRequestInterceptor(channel.channelId).request
  );
  api.interceptors.response.use(pipeGate.createResponseInterceptor().response);

  return api;
}

// Usage
const api = await setupApiClient();
const data = await api.get("/endpoint");
```

### Complete API Integration Example with One time Payment method

```typescript
import { ClientInterceptor } from "pipegate-sdk";
import axios from "axios";

async function setupApiClient() {
  // Initialize SDK
  const pipeGate = new ClientInterceptor();
  await pipeGate.initialize();

  // Define the txHash
  const txHash = "0x123...";

  // Setup API client
  const api = axios.create({
    baseURL: "https://api.example.com",
  });

  // Add interceptors
  api.interceptors.request.use(
    pipeGate.createOneTimePaymentRequestInterceptor(txHash).request
  );

  return api;
}

// Usage
const api = await setupApiClient();
const data = await api.get("/endpoint");
```

**Note**: This SDK is designed to work with the PipeGate protocol. Ensure you're using the latest version of the SDK and smart contracts for compatibility.
