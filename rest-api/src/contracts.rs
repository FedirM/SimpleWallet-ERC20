use ethers::contract::abigen;

abigen!(
    SimpleWallet,
    "../smart-contracts/out/SimpleWallet.sol/SimpleWallet.json"
);

abigen!(ERC20, "../smart-contracts/out/ERC20.sol/ERC20.json");
