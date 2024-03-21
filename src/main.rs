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

pub const HIST_THRESHOLD: f64 = SELL_AMOUNT_SOL * 0.05;

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

    let mut sol_macd = Macd::new(12, 26, 9).unwrap();
    let mut sol_last: f64 = 0.0;

    let mut usdc_macd = Macd::new(12, 26, 9).unwrap();
    let mut usdc_last: f64 = 0.0;

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
        
    let sell_request = QuoteRequest {
        amount: SELL_AMOUNT_LAMP,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let buy_request = QuoteRequest {
        amount: 200000000,
        input_mint: USDC_MINT,
        output_mint: NATIVE_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };


    let initial_funding: f64 = 1000.0;
    let mut sol : f64 = 5.0;
    let mut usdc : f64 = initial_funding;
    println!("Initial funding: {initial_funding:#?}");
    println!("Sell amount: {SELL_AMOUNT_SOL:#?}");
    println!("Hist threshold: {HIST_THRESHOLD:#?}");
    println!("Algorithm: Solid Buy");
    println!("Price, Histogram, ROC, ExSOL, ExUSDC, SOL, USDC, Buy, Sell");

    // GET /quote
    loop {
        match try_join!(jupiter_swap_api_client.quote(&sell_request), jupiter_swap_api_client.quote(&buy_request)) {
            Ok((sell_response, buy_response)) => {

                let mut buy_flag: &str = "0";
                let mut sell_flag: &str = "0";

                let sell_amount: u64 = sell_response.out_amount;
                let usdc_price = sell_amount as f64 * USDC_DECIMALS;
                let sol_hist = sol_macd.next(usdc_price).histogram;
                let sol_roc = sol_hist - sol_last;


                if sell_logic::should_sell(HIST_THRESHOLD, sol_hist, sol_roc, sol) {
                    sell_flag = "1";
                    //let sell = trade::swap(sell_response, &jupiter_swap_api_client, &rpc_client).await;
                    //if sell {
                        usdc = usdc + usdc_price;
                        sol = sol - SELL_AMOUNT_SOL;
                    //}
                }

                let buy_amount: u64 = buy_response.out_amount;
                let sol_price = buy_amount as f64 * NATIVE_DECIMALS;
                let usdc_hist = usdc_macd.next(sol_price).histogram;
                let usdc_roc = usdc_hist - usdc_last;

                if buy_logic::should_buy(HIST_THRESHOLD * 0.0002, usdc_hist, usdc_roc, usdc, usdc_price) {
                    buy_flag = "1";
                    //let buy = trade::swap(buy_response, &jupiter_swap_api_client, &rpc_client).await;
                    //if buy {
                        usdc = usdc - 200.0;
                        sol = sol + sol_price;
                    //}
                }

                println!("----------------------------------------------------------------------------");
                println!("SELL SOL: {usdc_price:.6}, {sol_hist:.9}, {sol_roc:.9}, {sol:.9}, {buy_flag}");
                println!("BUY  SOL: {sol_price:.9}, {usdc_hist:.9}, {usdc_roc:.9}, {usdc:.6}, {sell_flag}");

                sol_last = sol_hist;
                usdc_last = usdc_hist;

                thread::sleep(Duration::from_secs(10));
            },
            Err(_e) => {
                thread::sleep(Duration::from_secs(10));
            }
        }
    }

}
