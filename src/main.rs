use std::env;

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};

use jupiter_swap_api_client::{
    quote::QuoteRequest,
    JupiterSwapApiClient,
};
use solana_sdk::pubkey;
use ta::indicators::MovingAverageConvergenceDivergence as Macd;
use ta::Next;
use std::thread;
use std::time::Duration;
use tokio;

mod logic;
mod trade;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");


#[tokio::main]
async fn main() {
    macd().await;
}

async fn macd() {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut macd = Macd::new(12, 26, 9).unwrap();
    
    let quote_request = QuoteRequest {
        amount: 100,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let initial_funding: f64 = 1000.0;
    let mut funding : f64 = initial_funding;
    let mut profit: f64 = 0.0;
    let mut buys : u64 = 0;
    println!("Initial funding: {initial_funding:#?}");
    println!("Algorithm: Solid Buy");
    println!("Price, Histogram, Buys, Funding, Profit");

    // GET /quote
    match jupiter_swap_api_client.quote(&quote_request).await {
        Ok(quote_response) => {
            println!("Quote response: {quote_response:#?}");

            trade::swap(quote_response, jupiter_swap_api_client).await;
        },
        Err(_e) => {
            println!("Quote error: {_e:#?}");
        }
    }

}
