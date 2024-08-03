## Project Description

This project demonstrates how to build and interact with a simple Ethereum smart contract using Foundry and Rust. The smart contract, written in Solidity, is a basic counter that can increment and set a number. The Rust application provides a REST API using Warp to interact with this smart contract deployed on a local Ethereum testnet (Anvil).

### Tech Stack

Please check all dependencies are installed on your machine!

- **Smart Contract**: Solidity, Foundry
- **Backend**: Rust (Warp, Ethers-rs, Tokio)
- **Blockchain Testnet**: any Ethereum testnet

#### RESOURCES

1. Foundry - Check this [tutorial](https://book.getfoundry.sh/getting-started/installation) to get Foundry with all needed deps.
2. Rust - [official download](https://www.rust-lang.org/tools/install)
3. Also you need to install solc [official download](https://docs.soliditylang.org/en/latest/installing-solidity.html)

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
├── smart-contracts
│   ├── ...
│   ├── script
│   │   └── SimpleWallet.s.sol          # Main script to broadcast SimpleWallet contract
│   │
│   ├── src
│   │   ├── DummyToken.sol              # openzeppelin ERC20 based token (mock for real tests)
│   │   └── SimpleWallet.sol            # SimpleWallet definition
│   │
│   └── test
│       └── SimpleWallet.t.sol          # Test for SimpleWallet basic functionallity
│
└─── Simple-Wallet.postman_collection   # Here you could find POSTMAN collection

```

## Run Proces (DEV/TEST ONLY)

1. Build smart contract: (new terminal in project dir)

```
$> cd ./smart-contracts && forge build
```

2. Deploy smart contract to the testnet:

```
$> forge script script/SimpleWallet.s.sol:SimpleWalletScript --chain-id <Chain_ID> --rpc-url <RPC_URL> --broadcast --private-key <Your_Private_Key>

```

3. You should create `.env` file inside project directory:

```
TODO: Add example of .env file
```

4. Run web-server: (new terminal in project dir)

```
$> cd ./rest-api && cargo run
```
