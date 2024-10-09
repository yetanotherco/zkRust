use aligned_sdk::communication::serialization::cbor_serialize;
use aligned_sdk::core::errors::{AlignedError, SubmitError};
use ethers::utils::format_units;
use log::{error, info};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use aligned_sdk::core::types::{
    AlignedVerificationData, Network, PriceEstimate, ProvingSystemId, VerificationData,
};
use aligned_sdk::sdk::{
    deposit_to_aligned, estimate_fee, get_balance_in_aligned, get_chain_id, get_next_nonce,
    submit_and_wait_verification,
};
use clap::{Args, ValueEnum};
use dialoguer::Confirm;
use ethers::prelude::*;
use ethers::providers::Http;
use ethers::signers::LocalWallet;
use ethers::types::U256;

pub mod risc0;
pub mod sp1;
pub mod utils;

const BATCHER_URL: &str = "wss://batcher.alignedlayer.com";

// Make proof_data path optional
// Make keystore unneeded
#[derive(Args, Debug)]
pub struct ProofArgs {
    pub guest_path: String,
    #[clap(long = "submit-to-aligned")]
    pub submit_to_aligned: bool,
    #[clap(
        name = "Path to Wallet Key Store",
        long = "keystore-path",
        required_if_eq("submit_to_aligned", "true")
    )]
    pub keystore_path: Option<PathBuf>,
    #[clap(
        name = "URL of an Ethereum RPC Node",
        long = "rpc-url",
        default_value("https://ethereum-holesky-rpc.publicnode.com")
    )]
    pub rpc_url: String,
    #[clap(
        name = "The working network's name",
        long = "network",
        default_value = "holesky"
    )]
    pub network: NetworkArg,
    #[clap(
        name = "Payment send to the BatcherServicContract to fund Proof submission (Wei)",
        long = "batcher-payment",
        default_value("4000000000000000")
    )]
    pub batcher_payment: u128,
    #[clap(
        name = "Enables zkVM Acceleration via VM Precompiles",
        long = "precompiles"
    )]
    pub precompiles: bool,
    #[arg(
        name = "Aligned verification data directory Path",
        long = "aligned-verification-data-path",
        default_value = "./aligned_verification_data/"
    )]
    pub batch_inclusion_data_directory_path: String,
    #[arg(
        name = "Proof data directory path",
        long = "proof-data-path",
        default_value = "./proof_data"
    )]
    pub proof_data_directory_path: String,
}

#[derive(Debug, Clone, ValueEnum, Copy)]
pub enum NetworkArg {
    Devnet,
    Holesky,
    HoleskyStage,
}

impl From<NetworkArg> for Network {
    fn from(env_arg: NetworkArg) -> Self {
        match env_arg {
            NetworkArg::Devnet => Network::Devnet,
            NetworkArg::Holesky => Network::Holesky,
            NetworkArg::HoleskyStage => Network::HoleskyStage,
        }
    }
}

pub async fn submit_proof_to_aligned(
    proof_path: &str,
    elf_path: &str,
    pub_input_path: Option<&str>,
    args: &ProofArgs,
    proof_system_id: ProvingSystemId,
) -> Result<(), AlignedError> {
    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .map_err(|e| AlignedError::SubmitError(SubmitError::WalletSignerError(e.to_string())))?;

    let network: Network = args.network.into();
    //TODO: required if submission enabled. Therefore we unwrap().
    let keystore_path  = args.keystore_path.clone().unwrap();
    let local_wallet = LocalWallet::decrypt_keystore(&keystore_path, keystore_password)
        .map_err(|e| AlignedError::SubmitError(SubmitError::WalletSignerError(e.to_string())))?;
    let chain_id = get_chain_id(&args.rpc_url).await?;
    let wallet = local_wallet.with_chain_id(chain_id);

    let proof = std::fs::read(proof_path)
        .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;

    let elf_data = std::fs::read(elf_path)
        .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;

    // Public inputs are optional.
    let pub_input = match pub_input_path {
        Some(path) => Some(
            std::fs::read(path)
                .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?,
        ),
        None => None,
    };

    let provider = Provider::<Http>::try_from(&args.rpc_url)
        .map_err(|e| SubmitError::EthereumProviderError(e.to_string()))?;

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());

    // Estimate fee for proof based on default price estimate.
    let estimated_fee = estimate_fee(&args.rpc_url, PriceEstimate::Default).await?;

    let user_address = wallet.address();
    //TODO: Need to implement Aligned Error for Balance Error
    let user_balance = get_balance_in_aligned(user_address, &args.rpc_url, network)
        .await
        .map_err(|_| {
            SubmitError::GenericError("Failed to retrive user balance from Aligned".to_string())
        })?;

    let format_estimated_fee = format_units(estimated_fee, "ether").map_err(|e| {
        error!("Unable to convert estimate proof submision price");
        SubmitError::GenericError(e.to_string())
    })?;

    let format_user_balance = format_units(user_balance, "ether").map_err(|e| {
        error!("Unable to convert estimate proof submision price");
        SubmitError::GenericError(e.to_string())
    })?;

    if user_balance < estimated_fee {
        info!("Insufficient Balance balance for {:?}: User Balance {:?} eth  < Proof Submission Fee {:?} eth", user_address, format_user_balance, format_estimated_fee);
        if Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(format!(
                "Would you like to deposit {:?} eth to Aligned to fund proof submission?",
                format_estimated_fee
            ))
            .interact()
            .map_err(|e| {
                error!("Failed to read user input");
                SubmitError::GenericError(e.to_string())
            })?
        {
            info!("Submitting Payment to Batcher");
            let Ok(tx_receipt) =
                deposit_to_aligned(U256::from(estimated_fee), signer, network).await
            else {
                return Err(SubmitError::GenericError(
                    "Failed to Deposit Funds into the Batcher".to_string(),
                ))?;
            };
            info!(
                "Payment sent to the batcher successfully. Tx: 0x{:x}",
                tx_receipt.transaction_hash
            );
        } else {
            info!("Batcher Payment Cancelled");
            return Err(SubmitError::GenericError(
                "Insufficient User Balance on Aligned".to_string(),
            ))?;
        }
    }

    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!(
            "Would you like to pay {:?} eth to submit your proof to Aligned?",
            format_estimated_fee
        ))
        .interact()
        .map_err(|e| {
            error!("Failed to read user input");
            SubmitError::GenericError(e.to_string())
        })?
    {
        info!("Batcher Payment Cancelled");
        return Err(SubmitError::GenericError(
            "Insufficient User Balance on Aligned".to_string(),
        ))?;
    }

    let verification_data = VerificationData {
        proving_system: proof_system_id,
        proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(elf_data),
        verification_key: None,
        pub_input,
    };

    let nonce = get_next_nonce(&args.rpc_url, wallet.address(), network)
        .await
        .map_err(|e| {
            AlignedError::SubmitError(SubmitError::EthereumProviderError(e.to_string()))
        })?;

    info!("Submitting proof to Aligned for Verification");

    let aligned_verification_data = submit_and_wait_verification(
        BATCHER_URL,
        &args.rpc_url,
        network,
        &verification_data,
        estimated_fee,
        wallet,
        nonce,
    )
    .await
    .map_err(|e| AlignedError::SubmitError(SubmitError::GenericError(e.to_string())))?;

    info!("Proof Submitted to Aligned!");
    info!(
        "https://explorer.alignedlayer.com/batches/0x{}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );
    save_response(
        PathBuf::from(&args.batch_inclusion_data_directory_path),
        &aligned_verification_data,
    )?;
    println!(
        "Aligned Verification Data saved {:?}",
        args.batch_inclusion_data_directory_path
    );
    Ok(())
}

fn save_response(
    batch_inclusion_data_directory_path: PathBuf,
    aligned_verification_data: &AlignedVerificationData,
) -> Result<(), SubmitError> {
    if !batch_inclusion_data_directory_path.exists() {
        std::fs::create_dir_all(&batch_inclusion_data_directory_path)
            .map_err(|e| SubmitError::IoError(batch_inclusion_data_directory_path.clone(), e))?;
    }
    let batch_merkle_root = &hex::encode(aligned_verification_data.batch_merkle_root)[..8];
    let batch_inclusion_data_file_name = batch_merkle_root.to_owned()
        + "_"
        + &aligned_verification_data.index_in_batch.to_string()
        + ".json";

    let batch_inclusion_data_path =
        batch_inclusion_data_directory_path.join(batch_inclusion_data_file_name);

    let data = cbor_serialize(&aligned_verification_data)?;

    let mut file = File::create(&batch_inclusion_data_path)
        .map_err(|e| SubmitError::IoError(batch_inclusion_data_path.clone(), e))?;
    file.write_all(data.as_slice())
        .map_err(|e| SubmitError::IoError(batch_inclusion_data_path.clone(), e))?;
    info!(
        "Batch inclusion data written into {}",
        batch_inclusion_data_path.display()
    );

    Ok(())
}
