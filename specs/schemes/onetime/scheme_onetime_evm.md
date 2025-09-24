# Scheme: `onetime` `evm`

## Summary

One-time payments enable pay-per-use API access through direct blockchain transactions. Users make a single payment transaction to the API provider's address and use the transaction hash as proof of payment for subsequent API requests within a configured time window.

## Complete Flow

### 1. Initial Setup

- **Provider Configuration**: API provider configures accepted tokens, payment amounts, and session parameters
- **Address Publication**: Provider publishes their receiving address for direct payments

### 2. Payment Flow

1. **Consumer Payment**: Consumer sends ERC-20 tokens directly to provider's address via blockchain transaction
2. **Transaction Confirmation**: Payment confirmed on blockchain with unique transaction hash
3. **Proof Creation**: Consumer signs transaction hash to prove ownership

### 3. API Request Flow

1. **Consumer Request**: Consumer makes API request without payment header initially
2. **402 Response**: Server returns `402 Payment Required` with payment requirements and session parameters
3. **Payment Proof**: Consumer constructs X-Payment header with transaction hash and signature
4. **Transaction Verification**: Server verifies transaction on-chain and validates session limits
5. **Session Creation**: Successful verification creates time-limited access session
6. **API Access**: Consumer can make multiple API calls during valid session period

### 4. Session Management

- **Time Limits**: Sessions expire based on configured TTL after first redemption
- **Usage Limits**: Optional maximum number of API calls per payment
- **Window Validation**: Payments must be used within configured time window

> **Note: Evolution of x402 Format**  
> The current specification uses a unified `X-Payment` header with structured JSON payload. Earlier versions used separate headers for each parameter and different response formats. The current x402 v1 format provides better structure and extensibility for multiple payment schemes.

## 402 Payment Required Response

When a one-time payment is required, the server returns a `402 Payment Required` response with the following structure:

```json
{
  "x402Version": 1,
  "accepts": [
    {
      "scheme": "one-time",
      "network": "base",
      "amount": "0.001",
      "payTo": "0x..address",
      "asset": "0x..token_address",
      "resource": "example.com/api/resource",
      "description": "Access to example.com API resource",
      "maxTimeoutSeconds": 300,
      "extra": {
        "absWindowSeconds": 172800,
        "sessionTTLSeconds": 3600,
        "maxRedemptions": 5
      }
    }
  ],
  "error": "Payment Required"
}
```

### Payment Requirements Fields

- **scheme** (string): Always "one-time" for one-time payments
- **network** (string): Blockchain network where payment should be made (e.g., "base", "ethereum")
- **amount** (string): Required payment amount in decimal units
- **payTo** (string): API provider's address to receive the payment
- **asset** (string): ERC-20 token contract address for payments
- **resource** (string): The API resource being accessed
- **description** (string, optional): Human-readable description
- **maxTimeoutSeconds** (number, optional): Maximum time to process payment
- **extra** (object): Session configuration parameters
  - **absWindowSeconds** (number): Payment must be made within this window (e.g., 48 hours)
  - **sessionTTLSeconds** (number): Session validity duration after first redemption (e.g., 1 hour)
  - **maxRedemptions** (number, optional): Maximum API calls per payment (e.g., 5)

## `X-Payment` header payload

To make a one-time payment request, construct the `X-Payment` header with this structure:

```json
{
  "x402Version": 1,
  "network": "base",
  "scheme": "one-time",
  "payload": {
    "signature": "0x..sig",
    "tx_hash": "0x..transaction_hash"
  }
}
```

### Header Fields

- **x402Version** (number): Always 1, protocol version
- **network** (string): Must match the network from payment requirements
- **scheme** (string): Always "one-time" for one-time payments

### Payload Fields

- **signature** (string): ECDSA signature proving ownership of the transaction
- **tx_hash** (string): Hash of the payment transaction on the blockchain

### Signature Generation

The signature is created to prove ownership of the transaction:

```javascript
// Create message proving transaction ownership
const message = `Authorize payment access for transaction ${tx_hash}`;

// Create Ethereum signed message hash
const ethSignedMessage = keccak256(
  "\x19Ethereum Signed Message:\n" + message.length + message
);

// Sign with consumer's private key (transaction sender)
const signature = sign(ethSignedMessage, privateKey);
```

> **Note: Implementation Note**  
> The signature serves as proof that the API consumer controls the address that sent the payment transaction. The server uses this to verify transaction ownership before granting access.

## Verification

The middleware performs verification in the following steps:

### 1. Header Validation

- Parse and validate X-Payment JSON structure
- Verify scheme matches "one-time"
- Extract transaction hash and signature from payload

### 2. Signature Verification

```rust
// Verify signature proves ownership of transaction
let message = format!("Authorize payment access for transaction {}", payload.tx_hash);
let message_hash = keccak256(message.as_bytes());

// Recover signer address from signature
let recovered_address = ecrecover(message_hash, payload.signature);

// This recovered address will be validated against the transaction sender
```

### 3. Transaction Validation

**On-Chain Verification:**

- Query blockchain using RPC to get transaction details
- Verify transaction exists and is confirmed
- Check transaction amount meets minimum payment requirement
- Validate transaction was sent to the configured recipient address
- Ensure transaction sender matches the recovered signature address

**Timing Validation:**

- Check transaction timestamp is within the payment window (e.g., 48 hours)
- Validate transaction hasn't expired based on absolute window

### 4. Session Management

**Session State Validation:**

- Check if payment has already been redeemed
- Verify current redemption count against maximum allowed
- Check if existing session has expired

**Session Creation/Update:**

- Create new session if first redemption within limits
- Update redemption count and access timestamp
- Set session expiration based on TTL configuration

## Settlement

One-time payments are settled immediately on-chain:

### Direct Settlement

The payment is settled when the blockchain transaction is confirmed:

1. **Immediate Settlement**: Provider receives tokens immediately upon transaction confirmation
2. **No Additional Processing**: No manual settlement or additional transactions required
3. **Blockchain Finality**: Settlement guaranteed by blockchain consensus

### Session-Based Access Management

**Session Lifecycle:**

- Session created upon successful payment verification
- Multiple API calls allowed within session limits
- Automatic expiration based on time and usage limits

**Rate Limiting:**

- **Payment Window**: Prevents reuse of very old transactions
- **Session TTL**: Limits how long after first use the payment remains valid
- **Max Redemptions**: Optional limit on total API calls per payment

### State Management Considerations

**Server-Side Session Storage:**

- In-memory storage of payment sessions
- Transaction hash as unique identifier
- Redemption counting and expiration tracking

**Session Cleanup:**

- Automatic cleanup of expired sessions
- Prevents indefinite memory growth
- Configurable cleanup intervals

## Appendix

### Network Support

#### EVM Compatibility

- **Mainnet Support**: All EVM based mainnet networks - Ethereum, Base, Polygon, Arbitrum, Optimism
- **Testnet Support**: All EVM testnets for development
- **Token Support**: Any ERC-20 token with standard transfer functionality

#### RPC Requirements

- Standard JSON-RPC interface for transaction queries
- Block confirmation tracking
- Transaction receipt validation

### Integration Examples

#### Client-Side Payment Request

```typescript
// Example transaction hash from payment
const txHash = "0x742d35cc6635c0532925a3b8d4031fd3b3b80b1a123456789abcdef";

// Sign transaction ownership proof
const message = `Authorize payment access for transaction ${txHash}`;
const signature = await signMessage(message, privateKey);

const paymentHeader = {
  x402Version: 1,
  network: "base",
  scheme: "one-time",
  payload: {
    signature: signature,
    tx_hash: txHash,
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
// Return 402 when one-time payment required
const paymentRequiredResponse = {
  x402Version: 1,
  accepts: [
    {
      scheme: "one-time",
      network: "base",
      amount: "0.001", // 0.001 tokens
      payTo: "0x123d35Cc6635C0532925a3b8D4031fd3b3B80456",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e", // USDC
      resource: "/api/resource",
      description: "Access to premium API endpoint",
      maxTimeoutSeconds: 300,
      extra: {
        absWindowSeconds: 172800, // 48 hours payment window
        sessionTTLSeconds: 3600, // 1 hour session validity
        maxRedemptions: 5, // Maximum 5 API calls per payment
      },
    },
  ],
  error: "Payment Required",
};

res.status(402).json(paymentRequiredResponse);
```

#### Server-Side Middleware Setup

```rust
use pipegate::middleware::{PaymentsLayer, SchemeConfig, Scheme};

let onetime_config = SchemeConfig::new(
    Scheme::OneTimePayments,
    "https://base-rpc.publicnode.com",
    token_address, // USDC or other ERC-20 token
    recipient_address, // Provider's address
    "0.001" // 0.001 tokens per payment
).await;

let app = Router::new()
    .layer(PaymentsLayer::new(state, config));
```

### Session Configuration Examples

```rust
// Configuration for different use cases

// Quick access (5 minutes, 1 call)
let quick_config = SessionConfig {
    abs_window_seconds: 3600, // 1 hour payment window
    session_ttl_seconds: 300, // 5 minutes session
    max_redemptions: Some(1), // Single use only
};

// Extended access (1 hour, 10 calls)
let extended_config = SessionConfig {
    abs_window_seconds: 86400, // 24 hour payment window
    session_ttl_seconds: 3600, // 1 hour session
    max_redemptions: Some(10), // Up to 10 API calls
};

// Flexible access (24 hours, unlimited calls)
let flexible_config = SessionConfig {
    abs_window_seconds: 172800, // 48 hour payment window
    session_ttl_seconds: 86400, // 24 hour session
    max_redemptions: None, // No call limit
};
```

### Payment Flow Examples

#### Direct Token Transfer

```bash
# Using cast CLI to make payment
cast send 0x036CbD53842c5426634e7929541eC2318f3dCF7e \
  "transfer(address,uint256)" \
  0x123d35Cc6635C0532925a3b8D4031fd3b3B80456 \
  1000 \
  --rpc-url https://base-rpc.publicnode.com \
  --private-key $PRIVATE_KEY
```

#### Wallet Integration

```javascript
// Using ethers.js for payment
const contract = new ethers.Contract(tokenAddress, erc20ABI, signer);
const tx = await contract.transfer(providerAddress, paymentAmount);
const receipt = await tx.wait();
const txHash = receipt.transactionHash;

// Use txHash for API access
```

### Error Handling

Common error scenarios and handling:

- **Transaction Not Found**: Payment not yet confirmed or invalid hash
- **Insufficient Amount**: Payment amount below required minimum
- **Wrong Recipient**: Payment sent to incorrect address
- **Expired Window**: Payment made outside allowed time window
- **Session Expired**: Previous session has expired, need new payment
- **Redemption Limit**: Maximum API calls per payment exceeded
