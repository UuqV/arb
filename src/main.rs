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
use tokio;

mod logic;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDC_DECIMALS: f64 = 0.000001;

const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const NATIVE_DECIMALS: f64 = 0.000000001;

pub const TEST_WALLET: Pubkey = pubkey!("2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm"); // Coinbase 2 wallet



#[tokio::main]
async fn main() {
    macd().await;
}

async fn macd() {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut macd = Macd::new(12, 26, 9).unwrap();

        
    let sell_request = QuoteRequest {
        amount: 1000000000,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let initial_funding: f64 = 1000.0;
    let mut usdc : f64 = initial_funding;
    let mut profit: f64 = 0.0;
    let mut sol : f64 = 0.0;
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
