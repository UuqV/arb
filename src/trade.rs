use std::env;
use jupiter_swap_api_client::{
    quote::QuoteRequest, quote::QuoteResponse, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{read_keypair_file, Keypair}, pubkey, transaction::VersionedTransaction};
use solana_sdk::{pubkey::Pubkey, signature::NullSigner};
use tokio;


pub const TEST_WALLET: Pubkey = pubkey!("EVx7u3fzMPcNixmSNtriDCmpEZngHWH6LffhLzSeitCx");

pub async fn swap(quote_response: QuoteResponse, jupiter_swap_api_client: JupiterSwapApiClient, rpc_client: RpcClient) {

    println!("swap");

    match jupiter_swap_api_client
        .swap(&SwapRequest {
            user_public_key: TEST_WALLET,
            quote_response: quote_response.clone(),
            config: TransactionConfig::default(),
        })
        .await {
            Ok(swap_response) => {

                println!("Raw tx len: {}", swap_response.swap_transaction.len());

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
                                                println!("{transaction_sig}");
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
                    Err(e) => match e {
                        env::VarError::NotPresent => println!("Key not found."),
                        env::VarError::NotUnicode(os_string) => println!("Environment variable contains invalid unicode data: {:?}", os_string),
                    },
                }
            },
            Err(_e) => {
                println!("Error: {_e:#?}");
            }
        }

}
