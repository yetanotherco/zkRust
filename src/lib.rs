use std::str::FromStr;
use std::sync::Arc;

use aligned_sdk::sdk::{submit, verify_proof_onchain};
use aligned_sdk::types::{AlignedVerificationData, Chain, VerificationData};
use dialoguer::Confirm;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Wallet};
use ethers::types::Address;

const BATCHER_URL: &str = "wss://batcher.alignedlayer.com";
const BATCHER_PAYMENTS_ADDRESS: &str = "0x815aeCA64a974297942D2Bbf034ABEe22a38A003";

pub async fn submit_proof_and_wait_for_verification(
    verification_data: VerificationData,
    wallet: Wallet<SigningKey>,
    rpc_url: String,
) -> anyhow::Result<AlignedVerificationData> {
    let res = submit(BATCHER_URL, &verification_data, wallet)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit proof for verification: {:?}", e))?;

    match res {
        Some(aligned_verification_data) => {
            println!(
                "Proof submitted successfully on batch {}, waiting for verification...",
                hex::encode(aligned_verification_data.batch_merkle_root)
            );

            for _ in 0..10 {
                if verify_proof_onchain(
                    aligned_verification_data.clone(),
                    Chain::Holesky,
                    rpc_url.as_str(),
                )
                .await
                .is_ok_and(|r| r)
                {
                    return Ok(aligned_verification_data);
                }

                println!("Proof not verified yet. Waiting 10 seconds before checking again...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }

            anyhow::bail!("Proof verification failed");
        }
        None => {
            anyhow::bail!("Proof submission failed, no verification data");
        }
    }
}

pub async fn pay_batcher(
    from: Address,
    signer: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> anyhow::Result<()> {
    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("We are going to pay 0.004eth for the proof submission to aligned. Do you want to continue?")
        .interact()
        .expect("Failed to read user input")
    {
        anyhow::bail!("Payment cancelled")
    }

    let addr = Address::from_str(BATCHER_PAYMENTS_ADDRESS).map_err(|e| anyhow::anyhow!(e))?;

    let tx = TransactionRequest::new()
        .from(from)
        .to(addr)
        .value(4000000000000000u128);

    match signer
        .send_transaction(tx, None)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send tx {}", e))?
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit tx {}", e))?
    {
        Some(receipt) => {
            println!(
                "Payment sent. Transaction hash: {:x}",
                receipt.transaction_hash
            );
            Ok(())
        }
        None => {
            anyhow::bail!("Payment failed");
        }
    }
}
