use dotenv::dotenv;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::sync::Arc;
use warp::{http::StatusCode, reply::Reply, Filter};

mod contracts;
use contracts::{SimpleWallet, ERC20};

#[derive(Serialize, Deserialize)]
struct DepositETHRequest {
    from: String,
    amount: U256,
}

#[derive(Serialize, Deserialize)]
struct WithdrawETHRequest {
    to: Vec<String>,
    amounts: Vec<U256>,
}

#[derive(Serialize, Deserialize)]
struct WithdrawETHPercentageRequest {
    to: Vec<String>,
    percentages: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct DepositRequest {
    token: String,
    from: Vec<String>,
    amounts: Vec<U256>,
}

#[derive(Serialize, Deserialize)]
struct WithdrawRequest {
    token: String,
    to: Vec<String>,
    amounts: Vec<U256>,
}

#[derive(Serialize, Deserialize)]
struct WithdrawPercentageRequest {
    token: String,
    to: Vec<String>,
    percentages: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct BalanceRequest {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct AllowanceRequest {
    token: String,
    owner: String,
    spender: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let contract_address = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set");
    let chain_id: u64 = env::var("CHAIN_ID")
        .expect("CHAIN_ID must be set")
        .parse()
        .expect("Invalid CHAIN_ID");

    let client = Arc::new(SignerMiddleware::new(
        Provider::<Http>::try_from(rpc_url).expect("Invalid RPC URL"),
        (&private_key)
            .parse::<LocalWallet>()
            .expect("Invalid private key")
            .with_chain_id(chain_id),
    ));

    let contract_address: Address = contract_address.parse().expect("Invalid contract address");

    let contract = Arc::new(SimpleWallet::new(contract_address, Arc::clone(&client)));

    let deposit_erc20_route = warp::post()
        .and(warp::path("deposit_erc20"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(deposit_erc20_handler);

    let withdraw_amounts_erc20_route = warp::post()
        .and(warp::path("withdraw_amounts_erc20"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_amounts_erc20_handler);

    let withdraw_percentages_erc20_route = warp::post()
        .and(warp::path("withdraw_percentages_erc20"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_percentages_erc20_handler);

    let balance_erc20_route = warp::get()
        .and(warp::path("balance_erc20"))
        .and(warp::query::<BalanceRequest>())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(balance_erc20_handler);

    let balance_ether_route = warp::get()
        .and(warp::path("balance_ether"))
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(balance_ether_handler);

    let deposit_eth_route = warp::post()
        .and(warp::path("deposit_ether"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(deposit_ether_handler);

    let withdraw_amounts_ether_route = warp::post()
        .and(warp::path("withdraw_amounts_ether"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_amounts_ether_handler);

    let withdraw_percentages_ether_route = warp::post()
        .and(warp::path("withdraw_percentages_ether"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_percentages_ether_handler);

    let allowance_route = warp::get()
        .and(warp::path("allowance"))
        .and(warp::query::<AllowanceRequest>())
        .and(with_client(client.clone()))
        .and_then(allowance_handler);

    let routes = deposit_erc20_route
        .or(withdraw_amounts_erc20_route)
        .or(withdraw_percentages_erc20_route)
        .or(deposit_eth_route)
        .or(withdraw_amounts_ether_route)
        .or(withdraw_percentages_ether_route)
        .or(balance_erc20_route)
        .or(balance_ether_route)
        .or(allowance_route);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_contract(
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
) -> impl Filter<
    Extract = (Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,),
    Error = Infallible,
> + Clone {
    warp::any().map(move || contract.clone())
}

fn with_client(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> impl Filter<Extract = (Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,), Error = Infallible>
       + Clone {
    warp::any().map(move || client.clone())
}

async fn allowance_handler(
    query: AllowanceRequest,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let token_address: Address = query.token.parse().expect("Invalid token address");
    let owner_address: Address = query.owner.parse().expect("Invalid owner address");
    let spender_address: Address = query.spender.parse().expect("Invalid spender address");

    let erc20_contract = ERC20::new(token_address, Arc::clone(&client));

    match erc20_contract
        .allowance(owner_address, spender_address)
        .call()
        .await
    {
        Ok(allowance) => Ok(warp::reply::with_status(
            allowance.to_string(),
            StatusCode::OK,
        )),
        Err(e) => {
            eprintln!("Allowance check error: {:?}", e);
            Ok(warp::reply::with_status(
                String::from("Internal system error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn deposit_erc20_handler(
    body: DepositRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let from: Result<Vec<Address>, _> = body.from.iter().map(|s| s.parse()).collect();

    let from = match from {
        Ok(addresses) => addresses,
        Err(e) => {
            let error_msg = format!("Error parsing addresses: {:?}", e);
            eprintln!("{}", error_msg);
            return Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let amounts: Vec<U256> = body.amounts.iter().cloned().collect();

    // Debug logs
    println!("Token: {}", body.token.clone());
    println!("From: {:?}", from);
    println!("Amounts: {:?}", amounts);

    for (addr, amount) in from.iter().zip(&amounts) {
        match ERC20::new(
            body.token.parse::<Address>().unwrap(),
            Arc::new(client.provider()),
        )
        .allowance(*addr, contract.address())
        .call()
        .await
        {
            Ok(allowance) => {
                println!(
                    "Allowance \n\towner - {} \n\tspender - {} \n\tAmount: {:#?}",
                    addr,
                    contract.address(),
                    allowance
                );
                if allowance < *amount {
                    eprintln!("Insufficient allowance for address: {:?}", addr);
                    return Ok(warp::reply::with_status(
                        format!("Insufficient allowance for address: {:?}", addr),
                        StatusCode::BAD_REQUEST,
                    ));
                }
            }
            Err(e) => {
                eprintln!("Error checking allowance for address {:?}: {:?}", addr, e);
                return Ok(warp::reply::with_status(
                    format!("Error checking allowance for address {:?}: {:?}", addr, e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    match contract
        .deposit(body.token.parse().unwrap(), from, amounts)
        .send()
        .await
    {
        Ok(res) => {
            let tx_receipt = res.await.unwrap();
            Ok(warp::reply::with_status(
                tx_receipt.unwrap().transaction_hash.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            let error_msg = format!("Error during deposit: {:?}", e);
            eprintln!("{}", error_msg);
            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn deposit_ether_handler(
    body: DepositETHRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let from: Address = match body.from.parse() {
        Ok(value) => value,
        Err(_e) => {
            return Ok(warp::reply::with_status(
                String::from("Internal Server Error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let amount: U256 = body.amount;

    let tx = TransactionRequest {
        from: Some(from),
        to: Some(ethers::types::NameOrAddress::Address(contract.address())),
        value: Some(amount),
        data: None,
        nonce: None,
        gas_price: None,
        gas: None,
        chain_id: None,
    };

    match client.send_transaction(tx, None).await {
        Ok(tx_hash) => {
            let tx = tx_hash.await.unwrap();
            let th = tx.unwrap().transaction_hash;
            println!("Transaction hash: {:?}", &th);
            Ok(warp::reply::with_status(th.to_string(), StatusCode::OK))
        }
        Err(e) => {
            let error_msg = format!("Error sending transaction: {:?}", e);
            eprintln!("{}", error_msg);
            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn withdraw_amounts_ether_handler(
    body: WithdrawETHRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let to = body.to.iter().map(|s| s.parse()).collect();
    let to: Vec<Address> = match to {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Error during parse list of addresses: {:?}", e);
            return Ok(warp::reply::with_status(
                String::from("Internal Server Error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let amounts: Vec<U256> = body.amounts.iter().cloned().collect();

    println!(
        "Withdraw (amounts) ETH\n\t TO:{:?} \n\t AMOUNTS: {:?}",
        &to, &amounts
    );

    let balance = _client
        .provider()
        .get_balance(contract.address(), None)
        .await
        .unwrap();

    let total = amounts
        .clone()
        .into_iter()
        .reduce(|acc, v| acc + v)
        .unwrap();

    println!("Balance: {:?}\nTotal: {:?}", balance, total);
    if balance < total {
        return Ok(warp::reply::with_status(
            String::from("Not enough ETH"),
            StatusCode::BAD_REQUEST,
        ));
    }

    match contract.withdraw_eth_amounts(to, amounts).send().await {
        Ok(res) => {
            let tx_receipt = res.await.unwrap();
            Ok(warp::reply::with_status(
                tx_receipt.unwrap().transaction_hash.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            let error_msg = format!("Error during withdrawal: {:?}", e);
            eprintln!("{}", &error_msg);

            if let Some(revert_reason) = e.as_revert() {
                let decoded_revert_reason = String::from_utf8_lossy(&revert_reason.0);
                eprintln!("Revert reason: {}", decoded_revert_reason);
            }

            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn withdraw_percentages_ether_handler(
    body: WithdrawETHPercentageRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl warp::Reply, Infallible> {
    let to: Vec<Address> = body.to.iter().map(|s| s.parse().unwrap()).collect();
    let percentages: Vec<u8> = body.percentages.iter().cloned().collect();

    match contract
        .withdraw_percentages(Address::default(), to, percentages)
        .send()
        .await
    {
        Ok(res) => {
            let tx_receipt = res.await.unwrap();
            Ok(warp::reply::with_status(
                tx_receipt.unwrap().transaction_hash.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            let error_msg = format!("Error during withdrawal: {:?}", e);
            eprintln!("{}", &error_msg);

            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn withdraw_amounts_erc20_handler(
    body: WithdrawRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl warp::Reply, Infallible> {
    let token = if let Ok(addr) = body.token.parse::<Address>() {
        addr
    } else {
        return Ok(warp::reply::with_status(
            String::from("Bad token parameter"),
            StatusCode::BAD_REQUEST,
        ));
    };

    let to = body.to.iter().map(|s| s.parse()).collect();
    let to: Vec<Address> = match to {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Error during parse list of addresses: {:?}", e);
            return Ok(warp::reply::with_status(
                String::from("Internal Server Error"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let amounts: Vec<U256> = body.amounts.iter().cloned().collect();

    println!(
        "Withdraw amounts of ERC20 \n\t TOKEN: {:?} \n\t TO:{:?} \n\t AMOUNTS: {:?}",
        token, to, amounts
    );

    println!("Contract owner: {:?}", contract.owner());
    println!("Sender: {:?}", _client.address());

    match contract.withdraw_amounts(token, to, amounts).send().await {
        Ok(res) => {
            let tx_receipt = res.await.unwrap();
            Ok(warp::reply::with_status(
                tx_receipt.unwrap().transaction_hash.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            let error_msg = format!("Error during withdrawal: {:?}", e);
            eprintln!("{}", error_msg);
            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn withdraw_percentages_erc20_handler(
    body: WithdrawPercentageRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl warp::Reply, Infallible> {
    let to: Vec<Address> = body.to.iter().map(|s| s.parse().unwrap()).collect();
    let percentages: Vec<u8> = body.percentages.iter().cloned().collect();

    match contract
        .withdraw_percentages(body.token.parse().unwrap(), to, percentages)
        .send()
        .await
    {
        Ok(res) => {
            let tx_receipt = res.await.unwrap();
            Ok(warp::reply::with_status(
                tx_receipt.unwrap().transaction_hash.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            let error_msg = format!("Error during withdrawal: {:?}", e);
            eprintln!("{}", error_msg);
            Ok(warp::reply::with_status(
                error_msg,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn balance_erc20_handler(
    query: BalanceRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let token_address: Address = query.token.parse().expect("Invalid token address");
    let contract_address = contract.address();

    let erc20_contract = ERC20::new(token_address, Arc::clone(&client));

    match erc20_contract.balance_of(contract_address).call().await {
        Ok(balance) => {
            println!("ERC20 wallet balance: {}", &balance);
            Ok(warp::reply::with_status(
                balance.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            eprintln!("ERC20 wallet balance check error.\n{}", e);
            Ok(warp::reply::with_status(
                "Internal Server Error".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn balance_ether_handler(
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let contract_address = contract.address();
    match client.get_balance(contract_address, None).await {
        Ok(balance) => {
            println!("ERC20 wallet balance: {}", &balance);
            Ok(warp::reply::with_status(
                balance.to_string(),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            eprintln!("ERC20 wallet balance check error.\n{}", e);
            Ok(warp::reply::with_status(
                "Internal Server Error".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
