## Project Description

This project demonstrates how to build and interact with a simple Ethereum smart contract using Foundry and Rust. The smart contract, written in Solidity, is a basic counter that can increment and set a number. The Rust application provides a REST API using Warp to interact with this smart contract deployed on a local Ethereum testnet (Anvil).

### Tech Stack

- **Smart Contract**: Solidity, Foundry
- **Blockchain Testnet**: Anvil (local Ethereum testnet)
- **Backend**: Rust (Warp, Ethers-rs, Tokio)

## Project Structure

```
.
├── rest-api
│   ├── src
│   │   ├── contracts.rs                # Mod for loading smart contracts ABI
│   │   └── main.rs                     # REST API application based on Warp crate
│   │
│   └── .env                            # Environment vars
│
└── smart-contracts
    ├── ...
    ├── script
    │   └── SimpleWallet.s.sol          # Main script to broadcast SimpleWallet contract
    │
    ├── src
    │   ├── DummyToken.sol              # openzeppelin ERC20 based token (mock for real tests)
    │   └── SimpleWallet.sol            # SimpleWallet definition
    │
    └── test
        └── SimpleWallet.t.sol          # Test for SimpleWallet basic functionallity

```

## Run Proces

1. Start Anvil: (start up local Ethereum testnet)

```
$> anvil --chain-id 137
```

2. Build smart contract: (new terminal in project dir)

```
$> cd ./smart-contracts && forge build
```

3. Deploy smart contract to the testnet:

```
$> forge script script/SimpleWallet.s.sol:SimpleWalletScript --chain-id 137 --rpc-url http://127.0.0.1:8545 --broadcast --private-key <Your_Private_Key>

```

4. You should create `.env` file inside project directory:

```
TODO: Add example of .env file
```

5. Run web-server: (new terminal in project dir)

```
$> cd ./rest-api && cargo run
```
