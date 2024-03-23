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
use tokio::{self, try_join};

mod buy_logic;
mod sell_logic;
mod trade;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDC_DECIMALS: f64 = 0.000001;

const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const NATIVE_DECIMALS: f64 = 0.000000001;

pub const TEST_WALLET: Pubkey = pubkey!("EVx7u3fzMPcNixmSNtriDCmpEZngHWH6LffhLzSeitCx");

pub const SELL_AMOUNT_LAMP: u64 = 1_000_000_000; // 1_000_000_000 = 1 SOL
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

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut sell_macd = Macd::new(12, 26, 9).unwrap();

    let mut buy_macd = Macd::new(12, 26, 9).unwrap();

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
        
    let sell_request = QuoteRequest {
        amount: SELL_AMOUNT_LAMP,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 100,
        ..QuoteRequest::default()
    };

    let buy_request = QuoteRequest {
        amount: 200000000,
        input_mint: USDC_MINT,
        output_mint: NATIVE_MINT,
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
    println!("Buy Price, Buy Histogram, USDC, ACT_USDC, Sell Price, Sell Histogram, SOL, ACT_SOL, Buy/Sell, Total");

    // GET /quote
    loop {
        match try_join!(jupiter_swap_api_client.quote(&sell_request), jupiter_swap_api_client.quote(&buy_request)) {
            Ok((sell_response, buy_response)) => {

                let mut buy_sell_flag: &str = "";

                let sell_amount: u64 = sell_response.out_amount;
                let sell_price = sell_amount as f64 * USDC_DECIMALS * 0.995;
                let sell_hist = sell_macd.next(sell_price).histogram;


                if sell_logic::should_sell(HIST_THRESHOLD, sell_hist, sol) {
                    buy_sell_flag = "SELL";
                    let sell = trade::swap(sell_response, &jupiter_swap_api_client, &rpc_client).await;
                    if sell {
                        usdc = usdc + sell_price;
                        sol = sol - SELL_AMOUNT_SOL;
                    }
                    else {
                        buy_sell_flag = "ERROR";
                    }
                }

                let buy_amount: u64 = buy_response.out_amount;
                let buy_price = buy_amount as f64 * NATIVE_DECIMALS * 0.995;
                let buy_hist = buy_macd.next(buy_price).histogram;

                if buy_logic::should_buy(HIST_THRESHOLD * 0.0025, buy_hist, usdc, sell_price) {
                    buy_sell_flag = "BUY";
                    let buy = trade::swap(buy_response, &jupiter_swap_api_client, &rpc_client).await;
                    if buy {
                        usdc = usdc - 200.0;
                        sol = sol + buy_price;
                    }
                    else {
                        buy_sell_flag = "ERROR";
                    }
                }


                let act_usdc: f64 = get_token_account_balance(&rpc_client, USDC_MINT).await;
                let act_sol: f64 = get_sol_balance(&rpc_client).await;
                let total: f64 = act_usdc + (act_sol * sell_price);

                println!("{buy_price:.9}, {buy_hist:.9}, {usdc:.6}, {act_usdc:.6}, {sell_price:.6}, {sell_hist:.9}, {sol:.9}, {act_sol:.9}, {buy_sell_flag}, {total}");

                thread::sleep(Duration::from_secs(20));
            },
            Err(_e) => {
                thread::sleep(Duration::from_secs(10));
            }
        }
    }

}
