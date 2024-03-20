use std::{env, sync::Arc};
use jupiter_swap_api_client::{
    quote::QuoteRequest, quote::QuoteResponse, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{read_keypair_file, Keypair}, pubkey, transaction::VersionedTransaction};
use solana_sdk::{pubkey::Pubkey, signature::NullSigner};
use tokio::{self, sync::Mutex};


pub const TEST_WALLET: Pubkey = pubkey!("EVx7u3fzMPcNixmSNtriDCmpEZngHWH6LffhLzSeitCx");

pub async fn swap(quote_response: Arc<Mutex<QuoteResponse>>) {

    let unwrapped_quote = quote_response.lock().await;
    let api_base_url = env::var("API_BASE_URL").unwrap_or("https://quote-api.jup.ag/v6".into());
    println!("Using base url: {}", api_base_url);

    let jupiter_swap_api_client = JupiterSwapApiClient::new(api_base_url);
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());

    match jupiter_swap_api_client
        .swap(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response: unwrapped_quote.clone(),
            config: TransactionConfig::default(),
        })
        .await {
            Ok(swap_response) => {
                match env::var("ARBOT_KEY") {
                    Ok(key_string) => {
                        match read_keypair_file(key_string) {
                            Ok(keypair) => {
                                let mut versioned_transaction: VersionedTransaction = bincode::deserialize(&swap_response.swap_transaction).unwrap();

                                //Get the latest blockhash with rpc client
                                let latest_blockhash = rpc_client
                                    .get_latest_blockhash()
                                    .await
                                    .unwrap();
                            
                                //Set recent_blockhash to the latest_blockhash obtained
                                versioned_transaction.message.set_recent_blockhash(latest_blockhash);
    
                                match VersionedTransaction::try_new(versioned_transaction.message, &[&keypair]) {
                                    Ok(signed_versioned_transaction) => {
                                        match rpc_client.send_and_confirm_transaction(&signed_versioned_transaction).await {
                                            Ok(transaction_sig) => {
                                            }
                                            Err(_e) => {
                                                println!("{_e}");
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        println!("Signer error");
                                    }
                                };
                            },
                            Err(e) => {
                                println!("Pubkey error");
                            }
                        };
                    },
                    Err(_e) => {
                        println!("Var error");
                    },
                }
            },
            Err(_e) => {
                println!("Error: {_e:#?}");
            }
        }

}
