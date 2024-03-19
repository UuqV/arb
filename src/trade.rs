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

pub async fn swap(quote_response: QuoteResponse, jupiter_swap_api_client: JupiterSwapApiClient) {

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

                let versioned_transaction: VersionedTransaction = bincode::deserialize(&swap_response.swap_transaction).unwrap();
                match env::var("ARBOT_KEY") {
                    Ok(key_string) => {
                        match read_keypair_file(key_string) {
                            Ok(keypair) => {
                                match VersionedTransaction::try_new(versioned_transaction.message, &[&keypair]) {
                                    Ok(signed_versioned_transaction) => {
                                        // send with rpc client...
                                        let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".into());
                                        let transaction = rpc_client
                                            .send_and_confirm_transaction(&signed_versioned_transaction)
                                            .await
                                            .unwrap();
    
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
                        env::VarError::NotPresent => println!("Environment variable not found."),
                        env::VarError::NotUnicode(os_string) => println!("Environment variable contains invalid unicode data: {:?}", os_string),
                    },
                }
            },
            Err(_e) => {
                println!("Error: {_e:#?}");
            }
        }

}
