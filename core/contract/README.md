## Foundry

**Foundry is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.**

Foundry consists of:

- **Forge**: Ethereum testing framework (like Truffle, Hardhat and DappTools).
- **Cast**: Swiss army knife for interacting with EVM smart contracts, sending transactions and getting chain data.
- **Anvil**: Local Ethereum node, akin to Ganache, Hardhat Network.
- **Chisel**: Fast, utilitarian, and verbose solidity REPL.

## NOTE - Contract Addresses

ChannelFactory

Base Sepolia - 0xf2Cabfa8B29bFB86956D1960fF748f27836E1E14

## Documentation

https://book.getfoundry.sh/

## Methods

### Registering as an API provider

cast send 0xf2Cabfa8B29bFB86956D1960fF748f27836E1E14 "register(uint price)" 1000 --rpc-url https://base-sepolia-rpc.publicnode.com --private-key PRIVATE_KEY

### Approving stablecoin to the Channel Factory

USDC : Base Sepolia - 0x036CbD53842c5426634e7929541eC2318f3dCF7e
EURC : Base Sepolia -

cast send 0x036CbD53842c5426634e7929541eC2318f3dCF7e "approve(address spender, uint256 value)" 0xf2Cabfa8B29bFB86956D1960fF748f27836E1E14 1000000 --rpc-url https://base-sepolia-rpc.publicnode.com --private-key PRIVATE_KEY

### Creating a payment channel

cast send 0xf2Cabfa8B29bFB86956D1960fF748f27836E1E14 "createChannel(address recipient, uint256 \_duration,address \_tokenAddress, uint256 \_amount)" 0x62C43323447899acb61C18181e34168903E033Bf 2592000 0x036CbD53842c5426634e7929541eC2318f3dCF7e 1000000 --rpc-url https://base-sepolia-rpc.publicnode.com --private-key PRIVATE_KEY

### Fetching balance for a payment channel

cast call 0x4cF93D3b7cD9D50ecfbA2082D92534E578Fe46F6 "getBalance()" --rpc-url https://base-sepolia-rpc.publicnode.com

## Usage

### Build

```shell
$ forge build
```

### Test

```shell
$ forge test
```

### Format

```shell
$ forge fmt
```

### Gas Snapshots

```shell
$ forge snapshot
```

### Anvil

```shell
$ anvil
```

### Deploy

```shell
$ forge script script/Counter.s.sol:CounterScript --rpc-url <your_rpc_url> --private-key <your_private_key>
```

### Cast

```shell
$ cast <subcommand>
```

### Help

```shell
$ forge --help
$ anvil --help
$ cast --help
```

s
