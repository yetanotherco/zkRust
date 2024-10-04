use log::{error, info};
use std::path::PathBuf;

use aligned_sdk::core::types::{Network, ProvingSystemId, VerificationData, AlignedVerificationData};
use aligned_sdk::sdk::{deposit_to_aligned, get_next_nonce, submit_and_wait_verification, get_chain_id};
use dialoguer::Confirm;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::types::U256;
use aligned_sdk::core::errors::{SubmitError, AlignedError};

pub mod risc0;
pub mod sp1;
pub mod utils;

const BATCHER_URL: &str = "wss://batcher.alignedlayer.com";

//NOTE: we default to submitting to the testnet. When mainnet is live will make mainnet submission the default, testnet an option
pub async fn submit_proof_to_aligned(
    keystore_path: &PathBuf,
    proof_path: &str,
    elf_path: &str,
    pub_input_path: Option<&str>,
    rpc_url: &str,
    network: Network,
    max_fee: &u128,
    proof_system_id: ProvingSystemId,
) -> anyhow::Result<AlignedVerificationData, AlignedError> {
    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .map_err(|e| AlignedError::SubmitError(SubmitError::WalletSignerError(e.to_string())))?;

    let local_wallet = LocalWallet::decrypt_keystore(keystore_path, keystore_password)
        .map_err(|e| AlignedError::SubmitError(SubmitError::WalletSignerError(e.to_string())))?;

    let chain_id = get_chain_id(rpc_url).await?;
    let wallet = local_wallet.with_chain_id(chain_id);

    let proof = std::fs::read(proof_path)
        .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;

    let elf_data = std::fs::read(elf_path)
        .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;
        
    let pub_input = match pub_input_path {
        Some(path) => Some(std::fs::read(path)
            .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?),
        None => None,
    };

    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| AlignedError::SubmitError(SubmitError::EthereumProviderError(e.to_string())))?;

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());

    let amount_in_wei = 4000000000000000u128; //0.004eth

    if Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
    .with_prompt("Do you want to deposit 0.004eth in Aligned ?\nIf you already deposited Ethereum to Aligned before, this is not needed")
    .interact()
    .expect("Failed to read user input") {  
        let _ = deposit_to_aligned(amount_in_wei.into(), signer, network)
            .await
            .map(|receipt| info!("Payment sent to the batcher successfully. Tx: 0x{:x}", receipt.transaction_hash))
            .map_err(|e| error!("Transaction failed: {:?}", e));
    }

    let max_fee = U256::from(*max_fee);

    let verification_data = VerificationData {
        proving_system: proof_system_id,
        proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(elf_data),
        verification_key: None,
        pub_input,
    };

    let nonce = get_next_nonce(rpc_url, wallet.address(), network).await
        .map_err(|e| AlignedError::SubmitError(SubmitError::EthereumProviderError(e.to_string())))?;

    info!("Submitting proof to Aligned for Verification");

    let aligned_verification_data = submit_and_wait_verification(
        BATCHER_URL,
        rpc_url,
        network,
        &verification_data,
        max_fee,
        wallet,
        nonce
    )
    .await
    .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;

    info!("Proof Submitted to Aligned!");
    info!(
        "https://explorer.alignedlayer.com/batches/0x{}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );
    Ok(aligned_verification_data)
}
