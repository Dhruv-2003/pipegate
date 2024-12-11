# PipeGate Smart Contracts

Payment channel smart contracts for the PipeGate protocol - The Web3 Stripe for APIs.

## Deployed Contracts

**Base Sepolia Testnet**

- Channel Factory: `0x09443Ec32E54916366927ccDC9D372474324F427`
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
# Register with a price of 1000 per request without decimals in the terms of token
cast send $FACTORY_ADDRESS "register(uint256)" 1000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

2. **Close the channel to witdraw funds**

```bash
# Close the channel to withdraw 1 USDC with a nonce of 1000, along with the signature received during the API calls
cast send $FACTORY_ADDRESS "close(uint256 channelBalance,uint256 nonce,bytes calldata rawBody,bytes calldata signature)" 1000 1000 0x0 $SIGNATURE \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

### For API Consumers

1. **Approve USDC for Channel Factory**

```bash
# Approve 1,000,000 USDC units
cast send $USDC_ADDRESS "approve(address,uint256)" \
    $FACTORY_ADDRESS 1000000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

2. **Create Payment Channel**

```bash
# Parameters:
# - recipient: API provider address
# - duration: 30 days in seconds (2592000)
# - tokenAddress: USDC contract address
# - amount: Amount to deposit (e.g., 1000000)
cast send $FACTORY_ADDRESS "createChannel(address,uint256,address,uint256)" \
    $RECIPIENT_ADDRESS 2592000 $USDC_ADDRESS 1000000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

3. **Check Channel Balance**

```bash
# Replace CHANNEL_ADDRESS with your payment channel address
cast call $CHANNEL_ADDRESS "getBalance()" \
    --rpc-url $RPC_URL
```

## Environment Setup

Create a `.env` file:

```env
PRIVATE_KEY=your_private_key
RPC_URL=https://base-sepolia-rpc.publicnode.com
FACTORY_ADDRESS=0x09443Ec32E54916366927ccDC9D372474324F427
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

- Manages API provider registration
- Creates new payment channels
- Handles initial token deposits

Key features:

- Provider price registration
- Channel creation and deployment
- Token approval and transfer handling

### PaymentChannel

- Handles individual payment channels
- Manages channel state and balances
- Processes payment claims

## Security Considerations

1. Never share or commit private keys
2. Always verify token approvals
3. Check channel durations and amounts
4. Verify API provider addresses

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
