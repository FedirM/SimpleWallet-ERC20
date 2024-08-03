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
struct CollectETHRequest {
    from: Vec<String>,
    to: String,
}

#[derive(Serialize, Deserialize)]
struct CollectERC20Request {
    token: String,
    from: Vec<String>,
    to: String,
}

#[derive(Serialize, Deserialize)]
struct WithdrawETHRequest {
    to: Vec<String>,
    amounts: Option<Vec<U256>>,
    percentages: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct WithdrawERC20Request {
    token: String,
    to: Vec<String>,
    amounts: Option<Vec<U256>>,
    percentages: Option<Vec<u8>>,
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

    let deposit_eth_route = warp::post()
        .and(warp::path("deposit_ether"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(deposit_ether_handler);

    let deposit_erc20_route = warp::post()
        .and(warp::path("deposit_erc20"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(deposit_erc20_handler);

    let withdraw_ether_route = warp::post()
        .and(warp::path("withdraw_ether"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_ether_handler);

    let withdraw_erc20_route = warp::post()
        .and(warp::path("withdraw_erc20"))
        .and(warp::body::json())
        .and(with_contract(contract.clone()))
        .and(with_client(client.clone()))
        .and_then(withdraw_erc20_handler);

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

    let allowance_route = warp::get()
        .and(warp::path("allowance"))
        .and(warp::query::<AllowanceRequest>())
        .and(with_client(client.clone()))
        .and_then(allowance_handler);

    let routes = deposit_erc20_route
        .or(deposit_eth_route)
        .or(withdraw_erc20_route)
        .or(withdraw_ether_route)
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
    body: CollectERC20Request,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let from: Result<Vec<Address>, _> = body.from.iter().map(|s| s.parse()).collect();
    let from = match from {
        Ok(addresses) => addresses,
        Err(e) => {
            let error_msg = format!("Error parsing addresses: {:?}", e);
            eprintln!("{}", error_msg);
            return Ok(warp::reply::with_status(error_msg, StatusCode::BAD_REQUEST));
        }
    };

    let to = match body.to.parse::<Address>() {
        Ok(address) => address,
        Err(e) => {
            let error_msg = format!("Error parsing addresses: {:?}", e);
            eprintln!("{}", error_msg);
            return Ok(warp::reply::with_status(error_msg, StatusCode::BAD_REQUEST));
        }
    };

    // Debug logs
    println!("Token: {}", body.token.clone());
    println!("From: {:?}", from);
    println!("Amounts: {:?}", to);

    match contract
        .collect_erc20(body.token.parse().unwrap(), from, to)
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
    body: CollectETHRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    _client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    let from: Result<Vec<Address>, _> = body.from.iter().map(|s| s.parse()).collect();
    let from = match from {
        Ok(addresses) => addresses,
        Err(e) => {
            let error_msg = format!("Error parsing addresses: {:?}", e);
            eprintln!("{}", error_msg);
            return Ok(warp::reply::with_status(error_msg, StatusCode::BAD_REQUEST));
        }
    };

    let to = match body.to.parse::<Address>() {
        Ok(addr) => addr,
        Err(e) => {
            let error_msg = format!("Error parsing addresses: {:?}", e);
            eprintln!("{}", error_msg);
            return Ok(warp::reply::with_status(error_msg, StatusCode::BAD_REQUEST));
        }
    };

    // Debug logs
    println!("From: {:?}", from);
    println!("To: {:?}", to);

    match contract.collect_eth(from, to).send().await {
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

async fn withdraw_ether_handler(
    body: WithdrawETHRequest,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl Reply, Infallible> {
    if body.amounts.is_none() && body.percentages.is_none() {
        return Ok(warp::reply::with_status(
            String::from("Wrong params! There should be 'amounts' or 'percentages' field!"),
            StatusCode::BAD_REQUEST,
        ));
    }

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

    let amounts: Vec<U256> = if let Some(amounts) = body.amounts {
        let balance = get_eth_balance(Arc::clone(&client), contract.address())
            .await
            .unwrap();

        let total = amounts
            .clone()
            .into_iter()
            .reduce(|acc, v| acc + v)
            .unwrap();

        if balance < total {
            return Ok(warp::reply::with_status(
                String::from("Not enough ETH"),
                StatusCode::BAD_REQUEST,
            ));
        }
        amounts
    } else {
        if body
            .percentages
            .clone()
            .unwrap()
            .into_iter()
            .reduce(|acc, val| acc + val)
            .unwrap()
            > 100
        {
            return Ok(warp::reply::with_status(
                String::from("Wrong percentages value!"),
                StatusCode::BAD_REQUEST,
            ));
        }
        let total = get_eth_balance(Arc::clone(&client), contract.address())
            .await
            .unwrap();
        body.percentages
            .unwrap()
            .into_iter()
            .map(|p| (total * p) / 100)
            .collect()
    };

    println!(
        "Withdraw (amounts) ETH\n\t TO:{:?} \n\t AMOUNTS: {:?}",
        &to, &amounts
    );

    match contract.withdraw_eth(to, amounts).send().await {
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

async fn withdraw_erc20_handler(
    body: WithdrawERC20Request,
    contract: Arc<SimpleWallet<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<impl warp::Reply, Infallible> {
    let token = if let Ok(addr) = body.token.parse::<Address>() {
        addr
    } else {
        return Ok(warp::reply::with_status(
            String::from("Bad token parameter"),
            StatusCode::BAD_REQUEST,
        ));
    };

    if body.amounts.is_none() && body.percentages.is_none() {
        return Ok(warp::reply::with_status(
            String::from("Wrong params! There should be 'amounts' or 'percentages' field!"),
            StatusCode::BAD_REQUEST,
        ));
    }

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

    let amounts: Vec<U256> = if let Some(amounts) = body.amounts {
        let balance = get_erc20_balance(Arc::clone(&client), token, contract.address())
            .await
            .unwrap();

        let total = amounts
            .clone()
            .into_iter()
            .reduce(|acc, v| acc + v)
            .unwrap();

        if balance < total {
            return Ok(warp::reply::with_status(
                String::from("Not enough tokens"),
                StatusCode::BAD_REQUEST,
            ));
        }
        amounts
    } else {
        if body
            .percentages
            .clone()
            .unwrap()
            .into_iter()
            .reduce(|acc, val| acc + val)
            .unwrap()
            > 100
        {
            return Ok(warp::reply::with_status(
                String::from("Wrong percentages value!"),
                StatusCode::BAD_REQUEST,
            ));
        }
        let total = get_eth_balance(Arc::clone(&client), contract.address())
            .await
            .unwrap();
        body.percentages
            .unwrap()
            .into_iter()
            .map(|p| (total * p) / 100)
            .collect()
    };

    println!(
        "Withdraw (amounts) ETH\n\t TO:{:?} \n\t AMOUNTS: {:?}",
        &to, &amounts
    );

    match contract.withdraw_erc20(token, to, amounts).send().await {
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

    match get_erc20_balance(Arc::clone(&client), token_address, contract_address).await {
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

    match get_eth_balance(client, contract_address).await {
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

// Helpers

async fn get_eth_balance(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    address: Address,
) -> Result<U256, signer::SignerMiddlewareError<Provider<Http>, LocalWallet>> {
    match client.get_balance(address, None).await {
        Ok(balance) => Ok(balance),
        Err(e) => Err(e),
    }
}

async fn get_erc20_balance(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
    account: Address,
) -> Result<U256, ContractError<SignerMiddleware<Provider<Http>, LocalWallet>>> {
    let erc20_contract = ERC20::new(token, client);
    match erc20_contract.balance_of(account).call().await {
        Ok(balance) => Ok(balance),
        Err(e) => Err(e),
    }
}
