# Scheme: `channel` `evm`

## Summary

Payment channels enable high-frequency micropayments between API consumers and providers by creating off-chain payment commitments backed by on-chain escrow. Users deposit tokens into a smart contract and make API requests with cryptographic signatures that authorize payments, eliminating gas fees for individual API calls.

## Complete Flow

### 1. Initial Setup

- **Provider Registration**: API provider registers their price per request in the ChannelFactory contract
- **Channel Creation**: Consumer deposits tokens and creates a payment channel via the ChannelFactory
- **Channel Deployment**: Factory deploys a minimal proxy contract pointing to the PaymentChannel implementation

### 2. API Request Flow

1. **Consumer Request**: Consumer makes API request without payment header
2. **402 Response**: Server returns `402 Payment Required` with payment requirements and channel state (if exists)
3. **Payment Construction**: Consumer constructs X-Payment header with signature and channel state
4. **Payment Verification**: Server validates signature, channel state, and deducts payment amount
5. **API Access**: Server processes the API request and returns the response
6. **State Update**: Both client and server update their local channel state

### 3. Settlement

- **Ongoing**: Server accumulates signatures for each successful payment
- **Channel Closure**: Provider can close channel anytime using the latest signature to withdraw earned funds
- **Timeout Recovery**: Consumer can claim remaining funds if channel expires and provider doesn't close it

> **Note: Evolution of x402 Format**  
> The current specification uses a unified `X-Payment` header with structured JSON payload. Earlier versions used separate headers for each parameter and different response formats. The current x402 v1 format provides better structure and extensibility for multiple payment schemes.

## 402 Payment Required Response

When a payment channel is required, the server returns a `402 Payment Required` response with the following structure:

```json
{
  "x402Version": 1,
  "accepts": [
    {
      "scheme": "channel",
      "network": "base",
      "amount": "0.001",
      "payTo": "0x..address",
      "asset": "0x..token_address",
      "resource": "example.com/api/resource",
      "description": "Access to example.com API resource",
      "maxTimeoutSeconds": 300,
      "extra": paymentChannelState | null
    }
  ],
  "error": "Payment Required"
}
```

### Payment Requirements Fields

- **scheme** (string): Always "channel" for payment channels
- **network** (string): Blockchain network (e.g., "base", "ethereum")
- **amount** (string): Price per API request in decimal units
- **payTo** (string): API provider's address to receive payments
- **asset** (string): ERC-20 token contract address for payments
- **resource** (string): The API resource being accessed
- **description** (string, optional): Human-readable description
- **maxTimeoutSeconds** (number, optional): Maximum time to process payment
- **extra** (object|null): Current channel state if channel exists, null for new channels

## `X-Payment` header payload

To make a payment channel request, construct the `X-Payment` header with this structure:

```json
{
  "x402Version": 1,
  "network": "base",
  "scheme": "channel",
  "payload": {
    "signature": "0x..sig",
    "message": "0x..message_hash",
    "paymentChannel": {
      "channelId": "1234567890",
      "address": "0x..channel_contract_address",
      "sender": "0x..consumer_address",
      "recipient": "0x..provider_address",
      "balance": "95.5",
      "nonce": 42,
      "expiration": 1672531200
    },
    "timestamp": 1672531200
  }
}
```

### Header Fields

- **x402Version** (number): 1, protocol version
- **network** (string): Must match the network from payment requirements
- **scheme** (string): Always "channel" for payment channels

### Payload Fields

- **signature** (string): ECDSA signature authorizing the payment
- **message** (string): Keccak256 hash of the signed message
- **paymentChannel** (object): Complete channel state information
  - **channelId** (string): Unique identifier for the payment channel
  - **address** (string): Payment channel contract address
  - **sender** (string): API consumer's address (channel creator)
  - **recipient** (string): API provider's address (payment receiver)
  - **balance** (string): Current token balance remaining in channel
  - **nonce** (number): Monotonically increasing counter for replay protection
  - **expiration** (number): Unix timestamp when channel expires
- **timestamp** (number): Unix timestamp when payment was created (must be within 5 minutes)

### Signature Generation

The signature is created by signing a message constructed from the payment channel data:

```javascript
// Construct message components
const channelId = paymentChannel.channelId;
const balance = paymentChannel.balance;
const nonce = paymentChannel.nonce;
const bodyBytes = new Uint8Array(0); // Currently set to empty (0) in implementation

// Create message hash
const message = keccak256(channelId + balance + nonce + bodyBytes);

// Create Ethereum signed message hash
const ethSignedMessage = keccak256(
  "\x19Ethereum Signed Message:\n32" + message
);

// Sign with consumer's private key
const signature = sign(ethSignedMessage, privateKey);
```

> **Note**  
> In the current implementation, `bodyBytes` is set to an empty array (0). This may change in future versions to include request body verification.

The resulting `message` hash and `signature` are included in the payload.

## Verification

The middleware performs verification in the following steps:

### 1. Header Validation

- Parse and validate X-Payment JSON structure
- Verify scheme matches "channel"
- Check timestamp is within acceptable window (5 minutes)

### 2. Message Reconstruction and Signature Verification

```rust
// Extract data from X-Payment payload
let channel_id = payload.paymentChannel.channelId;
let balance = payload.paymentChannel.balance;
let nonce = payload.paymentChannel.nonce;
let body_bytes = &[]; // Currently empty in implementation

// Reconstruct the signed message
let reconstructed_message = keccak256(channel_id + balance + nonce + body_bytes);

// Verify message hash matches
assert(payload.message == reconstructed_message);

// Recover signer address from signature
let recovered_address = ecrecover(reconstructed_message, payload.signature);

// Verify recovered address matches channel sender
assert(recovered_address == payload.paymentChannel.sender);
```

### 3. Channel State Validation

**For existing channels:**

- Verify nonce is greater than last processed nonce from server state
- Calculate payment amount as the difference between server balance and submitted balance
- Ensure channel hasn't expired (current_time < paymentChannel.expiration)
- Validate channel contract address matches known channel

**For new channels:**

- Query channel contract on-chain at `paymentChannel.address`
- Verify channel exists and matches submitted parameters
- Confirm channel balance, expiration, sender, and recipient
- Ensure nonce starts at 0 for first payment

**Channel State Update:**

- Deduct payment amount from server's local channel balance
- Update nonce to prevent replay attacks
- Store latest signature for potential on-chain settlement

## Settlement

Payment channels support multiple settlement mechanisms:

### Provider-Initiated Closure

```solidity
function close(
    uint256 channelBalance,
    uint256 nonce,
    bytes calldata rawBody,
    bytes calldata signature
) external onlyRecipient
```

**Process:**

1. Provider submits latest signature with final channel state
2. Contract validates signature using ECDSA recovery
3. Earned amount (initial_balance - channel_balance) transferred to provider
4. Remaining balance refunded to consumer
5. Channel marked as permanently closed

### Consumer Timeout Claims

```solidity
function claimTimeout() external onlySender
```

**Process:**

1. Available after channel expiration timestamp
2. Only if provider hasn't closed channel first
3. Consumer recovers all remaining funds
4. Emergency mechanism for unresponsive providers

### State Management Considerations

**Critical Limitations:**

- Server state maintained in-memory only
- State lost on server restart/crash
- No automatic recovery mechanism

**Recovery Options:**

- Consumers can claim expired channel funds directly
- Providers can close channels using latest signatures
- Unused credits refunded through proper channel closure

## Appendix

### Smart Contract Architecture

#### ChannelFactory Contract

- **Address**: `0x5acfbe1f9B0183Ef7F2F8d8993d76f24B862092d` (Base Sepolia)
- **Purpose**: Creates payment channels using minimal proxy pattern
- **Gas Optimization**: Proxy deployment reduces channel creation costs

#### PaymentChannel Contract

- **States**: Uninitialized → Open → Closed
- **Security**: Nonce-based replay protection, signature verification
- **Escrow**: Holds deposited tokens until settlement

### Supported Networks

- **Base Sepolia Testnet**: Primary testing environment
- **Token Support**: Any ERC-20 token (USDC recommended)
- **Future**: Mainnet support planned

### Integration Examples

#### Client-Side Payment Request

```typescript
// Example X-Payment header construction
const paymentHeader = {
  x402Version: 1,
  network: "base",
  scheme: "channel",
  payload: {
    signature: "0x1234567890abcdef...",
    message: "0x0abcdef1234567890...",
    paymentChannel: {
      channelId: "1234567890",
      address: "0x5acfbe1f9B0183Ef7F2F8d8993d76f24B862092d",
      sender: "0x742d35Cc6635C0532925a3b8D4031fd3b3B80b1A",
      recipient: "0x123d35Cc6635C0532925a3b8D4031fd3b3B80456",
      balance: "99.001",
      nonce: 1,
      expiration: 1672531200,
    },
    timestamp: 1672444800,
  },
};

// Include in request headers
axios.get("/api/resource", {
  headers: {
    "X-Payment": JSON.stringify(paymentHeader),
  },
});
```

#### Server-Side 402 Response

```typescript
// Return 402 when payment required
const paymentRequiredResponse = {
  x402Version: 1,
  accepts: [
    {
      scheme: "channel",
      network: "base",
      amount: "0.001",
      payTo: "0x123d35Cc6635C0532925a3b8D4031fd3b3B80456",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e", // USDC
      resource: "/api/resource",
      description: "Access to premium API endpoint",
      maxTimeoutSeconds: 300,
      extra: existingChannelState || null,
    },
  ],
  error: "Payment Required",
};

res.status(402).json(paymentRequiredResponse);
```

#### Server-Side Middleware Setup

```rust
use pipegate::middleware::{PaymentsLayer, SchemeConfig, Scheme};

let channel_config = SchemeConfig::new(
    Scheme::PaymentChannels,
    "https://base-sepolia-rpc.publicnode.com",
    token_address,
    recipient_address,
    "0.001"
).await;

let app = Router::new()
    .layer(PaymentsLayer::new(state, config));
```
