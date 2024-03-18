use std::env;

use jupiter_swap_api_client::{
    quote::QuoteRequest, quote::QuoteResponse, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, transaction::VersionedTransaction};
use solana_sdk::{pubkey::Pubkey, signature::NullSigner};
use ta::indicators::MovingAverageConvergenceDivergence as Macd;
use ta::Next;
use std::thread;
use std::time::Duration;
use tokio;

const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const TEST_WALLET: Pubkey = pubkey!("2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm"); // Coinbase 2 wallet



#[tokio::main]
async fn main() {
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);

    let mut macd = Macd::new(12, 26, 9).unwrap();
    
    let quote_request = QuoteRequest {
        amount: 100000,
        input_mint: NATIVE_MINT,
        output_mint: USDC_MINT,
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    let mut profit : f64 = 1000.0;
    let mut buys : u64 = 0;

    // GET /quote
    loop {
        let quote_response = jupiter_swap_api_client.quote(&quote_request).await.unwrap();
        let price = quote_response.out_amount as f64 / 100.0;
        let next = macd.next(price);
        println!("{price:#?}");
        println!("{next:#?}");
        if next.histogram < -0.1 && profit > price {
            profit = profit - price;
            buys += 1;
        }
        else if next.histogram > 0.1 && buys > 0 {
            profit = profit + price;
            buys -= 1;
        }

        println!("Buys: {buys:#?}");
        println!("Profit: {profit:#?}");

        thread::sleep(Duration::from_secs(2));
    }

}

async fn macd() {
}

async fn swap(jupiter_swap_api_client: JupiterSwapApiClient, quote_response: QuoteResponse) {
    // POST /swap
    let swap_response = jupiter_swap_api_client
        .swap(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response: quote_response.clone(),
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();

    println!("Raw tx len: {}", swap_response.swap_transaction.len());

    let versioned_transaction: VersionedTransaction =
        bincode::deserialize(&swap_response.swap_transaction).unwrap();

    // Replace with a keypair or other struct implementing signer
    let null_signer = NullSigner::new(&TEST_WALLET);
    let signed_versioned_transaction =
        VersionedTransaction::try_new(versioned_transaction.message, &[&null_signer]).unwrap();

    // send with rpc client...
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());

    // This will fail with "Transaction signature verification failure" as we did not really sign
    let error = rpc_client
        .send_and_confirm_transaction(&signed_versioned_transaction)
        .await
        .unwrap_err();
    println!("{error}");

    // POST /swap-instructions
    let swap_instructions = jupiter_swap_api_client
        .swap_instructions(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response,
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();
    println!("swap_instructions: {swap_instructions:?}");
}