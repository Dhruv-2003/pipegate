# Scheme: `stream` `superfluid`

## Summary

Superfluid streams enable continuous, real-time micropayments for subscription-based APIs through the Superfluid money streaming protocol. Users create ongoing payment streams that flow continuously to API providers, allowing for pay-per-second usage models without individual transactions.

## Complete Flow

### 1. Initial Setup

- **Provider Configuration**: API provider configures acceptable flow rates, tokens, and recipient address
- **Stream Creation**: Consumer creates a Superfluid stream to provider's address with required flow rate
- **Protocol Registration**: Stream registered in Superfluid CFAv1 contract with continuous flow parameters

### 2. API Request Flow

1. **Consumer Request**: Consumer makes API request without payment header
2. **402 Response**: Server returns `402 Payment Required` with stream requirements
3. **Stream Verification**: Consumer signs proof of stream ownership and includes in X-Payment header
4. **Stream Validation**: Server verifies active stream exists with sufficient flow rate
5. **API Access**: Server processes the API request and returns the response
6. **Continuous Flow**: Tokens continue flowing in real-time from consumer to provider

### 3. Stream Management

- **Real-time Settlement**: Tokens flow continuously without manual intervention
- **Stream Monitoring**: WebSocket listeners track stream events for real-time updates
- **Automatic Termination**: Stream stops when consumer balance insufficient or manually cancelled

> **Note: Evolution of x402 Format**  
> The current specification uses a unified `X-Payment` header with structured JSON payload. Earlier versions used separate headers for each parameter and different response formats. The current x402 v1 format provides better structure and extensibility for multiple payment schemes.

## 402 Payment Required Response

When a stream payment is required, the server returns a `402 Payment Required` response with the following structure:

```json
{
  "x402Version": 1,
  "accepts": [
    {
      "scheme": "stream",
      "network": "base",
      "amount": "2",
      "payTo": "0x..address",
      "asset": "0x..token_address",
      "resource": "example.com/api/resource",
      "description": "Access to example.com API resource",
      "maxTimeoutSeconds": 300,
      "extra": null
    }
  ],
  "error": "Payment Required"
}
```

### Payment Requirements Fields

- **scheme** (string): Always "stream" for Superfluid streams
- **network** (string): Blockchain network with Superfluid support (e.g., "base", "polygon")
- **amount** (string): Required flow rate (tokens per month)
- **payTo** (string): API provider's address to receive stream payments
- **asset** (string): SuperToken or underlying token contract address
- **resource** (string): The API resource being accessed
- **description** (string, optional): Human-readable description
- **maxTimeoutSeconds** (number, optional): Maximum time to process payment
- **extra** (null): Currently unused for stream payments

## `X-Payment` header payload

To make a stream payment request, construct the `X-Payment` header with this structure:

```json
{
  "x402Version": 1,
  "network": "base",
  "scheme": "stream",
  "payload": {
    "signature": "0x..sig",
    "sender": "0x..sender_address"
  }
}
```

### Header Fields

- **x402Version** (number): 1, protocol version
- **network** (string): Must match the network from payment requirements
- **scheme** (string): Always "stream" for Superfluid streams

### Payload Fields

- **signature** (string): ECDSA signature proving ownership of the stream sender address
- **sender** (string): Address of the stream creator (consumer who created the stream)

### Signature Generation

The signature is created to prove ownership of the sender address:

```javascript
// Create message proving address ownership
const message = `Authorize stream access for ${sender}`;

// Create Ethereum signed message hash
const ethSignedMessage = keccak256(
  "\x19Ethereum Signed Message:\n" + message.length + message
);

// Sign with consumer's private key
const signature = sign(ethSignedMessage, privateKey);
```

> **Note: Implementation Note**  
> The signature serves as proof that the API consumer controls the address that created the stream. The server uses this to verify stream ownership before granting access.

## Verification

The middleware performs verification in the following steps:

### 1. Header Validation

- Parse and validate X-Payment JSON structure
- Verify scheme matches "stream"
- Extract sender address and signature from payload

### 2. Signature Verification

```rust
// Verify signature proves ownership of sender address
let message = format!("Authorize stream access for {}", payload.sender);
let message_hash = keccak256(message.as_bytes());

// Recover signer address from signature
let recovered_address = ecrecover(message_hash, payload.signature);

// Verify recovered address matches sender
assert(recovered_address == payload.sender);
```

### 3. Stream State Validation

**Stream Verification Process:**

- Query Superfluid CFAv1 contract for active streams from sender to provider
- Verify flow rate meets or exceeds minimum requirements
- Check stream hasn't been terminated or cancelled

**Cache Management:**

- Check local cache for existing verified stream (performance optimization)
- If cached and not expired, grant immediate access
- If cache miss or expired, perform on-chain verification
- Update cache with new verification results

**Stream State Update:**

- Store verified stream information in cache with expiration time
- Monitor WebSocket events for real-time stream status updates
- Invalidate cache immediately when stream termination detected

## Settlement

Stream payments are settled continuously and automatically:

### Real-time Settlement

The Superfluid protocol handles settlement automatically:

1. **Continuous Flow**: Tokens flow per-second from consumer to provider
2. **Automatic Distribution**: Provider receives streamed tokens in real-time
3. **No Manual Settlement**: No additional settlement transactions required
4. **Immediate Access**: Provider can access streamed funds immediately

### Stream Management

**Stream Modification:**

- Consumers can increase/decrease flow rates anytime
- Consumers can cancel streams to stop payments
- Changes take effect immediately on-chain

**Balance Management:**

- Stream continues until consumer balance depleted
- Automatic termination when insufficient funds

### State Management Considerations

**Performance Optimization:**

- Local cache provides sub-50ms verification times
- WebSocket listeners enable real-time stream monitoring
- Fallback to on-chain queries ensures accuracy

**Cache Invalidation:**

- Stream termination events trigger immediate cache updates
- Periodic cache refresh prevents stale data
- Event-driven updates maintain consistency

## Appendix

### Superfluid Protocol Integration

#### CFAv1 (Constant Flow Agreement)

- **Contract**: Manages continuous flow agreements between addresses
- **Flow Rate**: Tokens per second streaming rate
- **SuperTokens**: Streaming-enabled token wrappers

#### Supported Networks

- **All Superfluid Networks - Base, Arbitrum, Optimism**: Any network with Superfluid protocol deployment

### Integration Examples

#### Client-Side Stream Request

```typescript
// Example X-Payment header construction
const senderAddress = "0x742d35Cc6635C0532925a3b8D4031fd3b3B80b1A";
const message = `Authorize stream access for ${senderAddress}`;
const signature = await signMessage(message, privateKey);

const paymentHeader = {
  x402Version: 1,
  network: "base",
  scheme: "stream",
  payload: {
    signature: signature,
    sender: senderAddress,
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
// Return 402 when stream payment required
const paymentRequiredResponse = {
  x402Version: 1,
  accepts: [
    {
      scheme: "stream",
      network: "base",
      amount: "2", // 2 tokens per month
      payTo: "0x123d35Cc6635C0532925a3b8D4031fd3b3B80456",
      asset: "0x46fd5cfB4c12D87acD3a13e92BAa53240C661D93", // USDCx SuperToken
      resource: "/api/resource",
      description: "Access to premium streaming API endpoint",
      maxTimeoutSeconds: 300,
      extra: null,
    },
  ],
  error: "Payment Required",
};

res.status(402).json(paymentRequiredResponse);
```

#### Server-Side Middleware Setup

```rust
use pipegate::middleware::{PaymentsLayer, SchemeConfig, Scheme};

let stream_config = SchemeConfig::new(
    Scheme::SuperfluidStreams,
    "https://base-rpc.publicnode.com",
    recipient_address,
    super_token_address, // USDCx or other SuperToken
    "2" // 2 tokens per month
).await;

// WebSocket configuration for real-time updates
let stream_listener_config = StreamListenerConfig {
    wss_url: "wss://base-rpc.publicnode.com",
    cfa: CFAv1_ADDRESS,
};

let app = Router::new()
    .layer(PaymentsLayer::new(state, config))
    .layer(StreamListener::new(stream_listener_config));
```

### Flow Rate Calculations

Convert human-readable rates to per-second flow rates:

```typescript
// Monthly rate to per-second
const monthlyRate = 2; // 2 tokens per month
const flowRate = (monthlyRate * 1e18) / (30.44 * 24 * 60 * 60); // Accounting for 18 decimals

// Yearly rate to per-second
const yearlyRate = 24; // 24 tokens per year
const flowRate = (yearlyRate * 1e18) / (365 * 24 * 60 * 60);
```

### Contract Addresses

#### CFAv1 Forwarder (Universal)

- **Address**: `0xcfA132E353cB4E398080B9700609bb008eceB125`
- **Function**: All Superfluid-supported networks

#### Network-Specific CFA Addresses

Refer to [Superfluid Documentation](https://docs.superfluid.org/docs/protocol/contract-addresses) for network-specific contract addresses.

### Performance Metrics

- **Cache Hit Verification**: < 50ms response time
- **On-chain Verification**: 200-500ms response time
- **WebSocket Event Processing**: < 100ms cache invalidation
- **Stream Creation**: Standard blockchain transaction time
