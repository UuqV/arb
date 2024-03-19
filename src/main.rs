use std::env;

use jupiter_swap_api_client::{
    quote::QuoteRequest,
    JupiterSwapApiClient,
};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use ta::indicators::MovingAverageConvergenceDivergence as Macd;
use ta::Next;
use std::thread;
use std::time::Duration;
use solana_sdk::{signature::read_keypair_file, signature::Keypair};
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use tokio;

mod logic;
mod trade;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDC_DECIMALS: f64 = 0.000001;

const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const NATIVE_DECIMALS: f64 = 0.000000001;

pub const TEST_WALLET: Pubkey = pubkey!("EVx7u3fzMPcNixmSNtriDCmpEZngHWH6LffhLzSeitCx");

#[tokio::main]
async fn main() {
    match env::var("ARBOT_KEY") {
        Ok(key_string) => {
            match read_keypair_file(key_string) {
                Ok(keypair) => {
                    macd(keypair).await;
                }
                Err(_e) => {
                    println!("Error: {_e:#?}");
                }
            }
        }
        Err(e) => match e {
            std::env::VarError::NotPresent => println!("Key not found."),
            std::env::VarError::NotUnicode(os_string) => println!("Environment variable contains invalid unicode data: {:?}", os_string),
        }
    }
}

async fn get_token_account_balance(rpc_client: &RpcClient, token_address: Pubkey) -> f64 {
    let associated_token_address = get_associated_token_address(&TEST_WALLET, &token_address);
    let account_data = rpc_client.get_token_account_balance(&associated_token_address).await.unwrap();
    return account_data.ui_amount.unwrap();
}

async fn get_sol_balance(rpc_client: &RpcClient) -> f64 {
    let account_data = rpc_client.get_account(&TEST_WALLET).await.unwrap();
    let lamports = account_data.lamports;
    return (lamports / 1000000000) as f64;
}



async fn macd(keypair: Keypair) {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut macd = Macd::new(12, 26, 9).unwrap();

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
        
    let sell_request = QuoteRequest {
        amount: 100,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let initial_funding: f64 = get_token_account_balance(&rpc_client, USDC_MINT).await;
    let mut sol : f64 = get_sol_balance(&rpc_client).await;
    let mut usdc : f64 = initial_funding;
    let mut profit: f64 = 0.0;
    println!("Initial funding: {initial_funding:#?}");
    println!("Algorithm: Solid Buy");
    println!("Sell, Histogram, SOL, USDC, Profit");

    // GET /quote
    loop {
        match jupiter_swap_api_client.quote(&sell_request).await {
            Ok(sell_response) => {


                let price = sell_response.out_amount as f64 * USDC_DECIMALS;
                let next = macd.next(price);
                let hist = next.histogram;

                if logic::should_sell(hist, usdc, sol) {
                    usdc = usdc + price;
                    if usdc > initial_funding {
                        profit = profit + (usdc - initial_funding);
                        usdc = initial_funding;
                    }
                    sol = sol - 1.0;
                }

                let buy_request = QuoteRequest {
                    amount: sell_response.out_amount,
                    input_mint: USDC_MINT,
                    output_mint: NATIVE_MINT,
                    slippage_bps: 50,
                    ..QuoteRequest::default()
                };

                match jupiter_swap_api_client.quote(&buy_request).await {
                    Ok(buy_response) => {
                        if logic::should_buy(hist, usdc, price) {
                            usdc = usdc - price;
                            sol = sol + buy_response.out_amount as f64 * NATIVE_DECIMALS;
                        }
                        println!("{price:#?}, {hist:#?}, {sol:#?}, {usdc:#?}, {profit:#?}");
                    },
                    Err(_e) => {
                        thread::sleep(Duration::from_secs(2));

                    }
                }

                thread::sleep(Duration::from_secs(2));
            },
            Err(_) => {
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

}
