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

pub const SELL_AMOUNT_LAMP: u64 = 100_000_000; // 1_000_000_000 = 1 SOL
pub const SELL_AMOUNT_SOL: f64 = SELL_AMOUNT_LAMP as f64 * NATIVE_DECIMALS;

pub const HIST_THRESHOLD: f64 = SELL_AMOUNT_SOL * 0.1;

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
    return lamports as f64 * NATIVE_DECIMALS;
}



async fn macd(keypair: Keypair) {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut macd = Macd::new(12, 26, 9).unwrap();

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
        
    let sell_request = QuoteRequest {
        amount: SELL_AMOUNT_LAMP,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 100,
        ..QuoteRequest::default()
    };

    let initial_funding: f64 = get_token_account_balance(&rpc_client, USDC_MINT).await;
    let mut sol : f64 = get_sol_balance(&rpc_client).await;
    let mut usdc : f64 = initial_funding;
    println!("Initial funding: {initial_funding:#?}");
    println!("Sell amount: {SELL_AMOUNT_SOL:#?}");
    println!("Hist threshold: {HIST_THRESHOLD:#?}");
    println!("Algorithm: Solid Buy");
    println!("Price, Histogram, ExSOL, ActSOL, ExUSDC, ActUSDC, Buy");

    // GET /quote
    loop {
        match jupiter_swap_api_client.quote(&sell_request).await {
            Ok(sell_response) => {

                let mut buy_flag: &str = "None";
                let sell_amount: u64 = sell_response.out_amount;
                let price = sell_amount as f64 * USDC_DECIMALS;
                let next = macd.next(price);
                let hist = next.histogram;

                if logic::should_sell(hist, sol, SELL_AMOUNT_SOL, HIST_THRESHOLD) {
                    buy_flag = "Sell";
                    tokio::spawn(async move {
                        trade::swap(sell_response, &jupiter_swap_api_client, &rpc_client).await;
                    });
                }

                let buy_request = QuoteRequest {
                    amount: sell_amount,
                    input_mint: USDC_MINT,
                    output_mint: NATIVE_MINT,
                    slippage_bps: 100,
                    ..QuoteRequest::default()
                };

                match jupiter_swap_api_client.quote(&buy_request).await {
                    Ok(buy_response) => {
                        if logic::should_buy(hist, usdc, price, HIST_THRESHOLD) {
                            buy_flag = "Buy";
                            tokio::spawn(async move {
                                trade::swap(buy_response, &jupiter_swap_api_client, &rpc_client).await;
                            });
                        }
                        usdc = get_token_account_balance(&rpc_client, USDC_MINT).await;
                        sol = get_sol_balance(&rpc_client).await;

                        println!("{price:.6}, {hist:.9}, {sol:.9}, {usdc:.6}, {buy_flag}");
                    },
                    Err(_e) => {
                        thread::sleep(Duration::from_secs(2));
                    }
                }

                thread::sleep(Duration::from_secs(2));
            },
            Err(_e) => {
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

}
