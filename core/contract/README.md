# PipeGate Smart Contracts

Payment channel smart contracts for the PipeGate protocol - The Web3 Stripe for APIs.

## Deployed Contracts

**Base Sepolia Testnet**

- Channel Factory: `0x5acfbe1f9B0183Ef7F2F8d8993d76f24B862092d`
- USDC Token: `0x036CbD53842c5426634e7929541eC2318f3dCF7e`

## Development Setup

1. Install Foundry:

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

2. Clone the repository:

```bash
git clone https://github.com/yourusername/pipegate-contracts
cd pipegate-contracts
```

3. Install dependencies:

```bash
forge install
```

4. Build contracts:

```bash
forge build
```

## Test

Run the test suite:

```bash
forge test
```

Run tests with gas reporting:

```bash
forge test --gas-report
```

## Contract Interaction Guide

### For API Providers

1. **Register as an API Provider**

```bash
# Register with a price of 1000 per request (without decimals in terms of token)
cast send $FACTORY_ADDRESS "register(uint256)" 1000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

2. **Update Pricing** (Optional)

```bash
# Update your price per request
cast send $FACTORY_ADDRESS "register(uint256)" 2000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

3. **Close Channel and Withdraw Funds**

```bash
# Close the channel to withdraw funds with a nonce and signature received during API calls
# This must be called on the specific channel contract, not the factory
cast send $CHANNEL_ADDRESS "close(uint256,uint256,bytes,bytes)" $CHANNEL_BALANCE $NONCE $RAW_BODY $SIGNATURE \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

4. **View Your Channels**

```bash
# Get all channels where you're the recipient
cast call $FACTORY_ADDRESS "getRecipientChannels(address)" $YOUR_ADDRESS \
    --rpc-url $RPC_URL
```

### For API Consumers

1. **Approve USDC for Channel Factory**

```bash
# Approve tokens for the factory to spend (e.g., 1,000,000 USDC units)
cast send $USDC_ADDRESS "approve(address,uint256)" \
    $FACTORY_ADDRESS 1000000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

2. **Create Payment Channel**

```bash
# Parameters:
# - recipient: API provider address (must be registered)
# - duration: Channel duration in seconds (max 365 days)
# - tokenAddress: ERC20 token contract address
# - amount: Initial deposit amount
cast send $FACTORY_ADDRESS "createChannel(address,uint256,address,uint256)" \
    $RECIPIENT_ADDRESS 2592000 $USDC_ADDRESS 1000000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

3. **Add Funds to Existing Channel**

```bash
# Deposit additional funds to your channel
cast send $CHANNEL_ADDRESS "deposit(uint256)" 500000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

4. **Extend Channel Duration**

```bash
# Extend the expiration time (timestamp must be in the future)
cast send $CHANNEL_ADDRESS "extend(uint256)" 1735689600 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

5. **Claim Expired Channel Funds**

```bash
# Claim all remaining funds after channel expires
cast send $CHANNEL_ADDRESS "claimTimeout()" \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

6. **Check Channel Information**

```bash
# Get comprehensive channel information
cast call $FACTORY_ADDRESS "getChannelInfo(uint256)" $CHANNEL_ID \
    --rpc-url $RPC_URL

# Check channel balance
cast call $CHANNEL_ADDRESS "getBalance()" \
    --rpc-url $RPC_URL

# View your channels
cast call $FACTORY_ADDRESS "getSenderChannels(address)" $YOUR_ADDRESS \
    --rpc-url $RPC_URL
```

## Environment Setup

Create a `.env` file:

```env
PRIVATE_KEY=your_private_key
RPC_URL=https://base-sepolia-rpc.publicnode.com
FACTORY_ADDRESS=0x5acfbe1f9B0183Ef7F2F8d8993d76f24B862092d
USDC_ADDRESS=0x036CbD53842c5426634e7929541eC2318f3dCF7e
```

Load environment variables:

```bash
source .env
```

## Deployment

Deploy to testnet:

```bash
forge script script/DeployChannelFactory.s.sol:Deploy \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast
```

## Contract Architecture

### ChannelFactory

The factory contract manages the creation and tracking of payment channels using a minimal proxy pattern for gas efficiency.

**Key Functions:**

- `register(uint256 price)` - Register as API provider with pricing
- `createChannel(address,uint256,address,uint256)` - Create new payment channel
- `getSenderChannels(address)` - Get all channels created by an address
- `getRecipientChannels(address)` - Get all channels for a recipient
- `getChannelInfo(uint256)` - Get comprehensive channel information
- `isRegisteredProvider(address)` - Check if address is registered provider

**Key Features:**

- Provider price registration and updates
- Gas-efficient channel deployment using minimal proxies
- Channel tracking and user associations
- Comprehensive channel information queries

### PaymentChannel

Individual payment channel contracts handle micropayments and state management.

**Key Functions:**

- `init(...)` - Initialize channel (called by factory)
- `deposit(uint256)` - Add funds to channel (sender only)
- `close(uint256,uint256,bytes,bytes)` - Close channel with signature (recipient only)
- `extend(uint256)` - Extend channel expiration (sender only)
- `claimTimeout()` - Claim funds after expiration (sender only)
- `getBalance()` - Get current channel balance
- `getChannelInfo()` - Get detailed channel information

**Key Features:**

- Signature-based payment authorization
- Nonce-based replay protection
- Timeout mechanisms for fund recovery
- Deposit and withdrawal management
- Channel state tracking (Uninitialized, Open, Closed)

## Security Considerations

1. **Private Key Security**

   - Never share or commit private keys
   - Use hardware wallets for production
   - Consider using multi-sig wallets for large amounts

2. **Token Approvals**

   - Always verify token approvals before channel creation
   - Only approve the exact amount needed
   - Revoke approvals when no longer needed

3. **Channel Parameters**

   - Verify recipient addresses are registered providers
   - Check channel durations don't exceed maximum (365 days)
   - Ensure deposit amounts are reasonable for intended usage

4. **Signature Security**

   - Only sign channel closure messages you understand
   - Verify nonce values to prevent replay attacks
   - Keep track of channel balances and nonces

5. **State Management**
   - Monitor channel expiration times
   - Keep backup of channel addresses and IDs
   - Regularly check channel balances

## Development Commands

```bash
# Format code
forge fmt

# Run gas snapshots
forge snapshot

# Local node
anvil

# Contract verification
forge verify-contract $CONTRACT_ADDRESS ChannelFactory \
    --chain-id 84532 \
    --watch
```

## Support

For technical support:

- Create an issue in the GitHub repository
- Join our Discord community
- Check the documentation

## License

UNLICENSED

---

**Note**: This is a testnet deployment. For production use, additional security reviews and audits are recommended.
