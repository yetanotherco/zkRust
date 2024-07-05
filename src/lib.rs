use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use aligned_sdk::sdk::{submit, verify_proof_onchain};
use aligned_sdk::types::{AlignedVerificationData, Chain, ProvingSystemId, VerificationData};
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

pub fn submit_proof_to_aligned(
    keystore_path: PathBuf,
    proof_path: &str,
    elf_path: &str,
    proof_system_id: ProvingSystemId,
) -> anyhow::Result<()> {
    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");

    let wallet = LocalWallet::decrypt_keystore(keystore_path, keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(17000u64);

    let proof = fs::read(proof_path).expect("failed to serialize proof");
    let elf_data = fs::read(elf_path).expect("failed to serialize elf");

    let rpc_url = "https://ethereum-holesky-rpc.publicnode.com";

    let provider = Provider::<Http>::try_from(rpc_url).expect("Failed to connect to provider");

    let signer = Arc::new(SignerMiddleware::new(provider.clone(), wallet.clone()));

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    runtime
        .block_on(pay_batcher(wallet.address(), signer.clone()))
        .expect("Failed to pay for proof submission");

    let verification_data = VerificationData {
        proving_system: proof_system_id,
        proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(elf_data),
        verification_key: None,
        pub_input: None,
    };

    println!("Submitting proof to aligned for verification");

    runtime
        .block_on(submit_proof_and_wait_for_verification(
            verification_data,
            wallet,
            rpc_url.to_string(),
        ))
        .expect("failed to submit proof");
    Ok(())
}
