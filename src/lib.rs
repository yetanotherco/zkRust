use aligned_sdk::communication::serialization::cbor_serialize;
use aligned_sdk::core::errors::{AlignedError, SubmitError};
use ethers::utils::format_units;
use log::{error, info};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use aligned_sdk::core::types::{
    AlignedVerificationData, Network, ProvingSystemId, VerificationData,
};
use aligned_sdk::sdk::{deposit_to_aligned, get_next_nonce, submit_and_wait_verification};
use clap::{Args, ValueEnum};
use dialoguer::Confirm;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::types::U256;

pub mod risc0;
pub mod sp1;
pub mod utils;

const BATCHER_URL: &str = "wss://batcher.alignedlayer.com";

#[derive(Args, Debug)]
pub struct ProofArgs {
    pub guest_path: String,
    #[clap(long)]
    pub submit_to_aligned: bool,
    #[clap(long, required_if_eq("submit_to_aligned", "true"))]
    pub keystore_path: Option<PathBuf>,
    #[clap(long, default_value("https://ethereum-holesky-rpc.publicnode.com"))]
    pub rpc_url: String,
    #[clap(
        name = "The working network's name",
        long = "network",
        default_value = "devnet"
    )]
    pub network: NetworkArg,
    #[clap(long, default_value("100000000000000"))]
    pub max_fee: u128,
    #[clap(long, default_value("4000000000000000"))]
    pub batcher_payment: u128,
    #[clap(long)]
    pub precompiles: bool,
}

#[derive(Debug, Clone, ValueEnum, Copy)]
enum NetworkArg {
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
    args: ProofArgs,
    proof_system_id: ProvingSystemId,
) -> Result<(), AlignedError> {
    let Ok(keystore_password) = rpassword::prompt_password("Enter keystore password: ") else {
        error!("Failed to read keystore password");
        return Ok(());
    };

    let keystore_path = args.keystore_path.into();
    let network: Network = args.network.into();
    let local_wallet =
        LocalWallet::decrypt_keystore(keystore_path, keystore_password).map_err(|e| {
            error!("Failed to decrypt keystore");
            SubmitError::GenericError(e.to_string())
        })?;

    let wallet = local_wallet.with_chain_id(17000u64);

    let proof = std::fs::read(proof_path).map_err(|e| {
        error!("Failed to Read Proof");
        SubmitError::GenericError(e.to_string())
    })?;
    let elf_data = std::fs::read(elf_path).map_err(|e| {
        error!("Failed to Read ELF");
        SubmitError::GenericError(e.to_string())
    })?;

    let pub_input = if let Some(path) = pub_input_path {
        let pub_inputs = std::fs::read(path).map_err(|e| {
            error!("Failed to Read Public Inputs: {:?}", e);
            return Err(SubmitError::GenericError(
                "Batcher Payment cancelled".to_string(),
            ))?;
        })?;
        Some(pub_inputs)
    } else {
        error!("No Public Input Path Specified");
        None
    };

    let Ok(provider) = Provider::<Http>::try_from(args.rpc_url) else {
        error!("Failed to connect to provider");
        return Ok(());
    };

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());

    let payment = format_units(args.batcher_payment, "ether").map_err(|e| {
        error!("Unable to convert batcher payment amount, please convert to units of Wei");
        SubmitError::GenericError(e.to_string())
    })?;

    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("We are going to pay {:?} eth for the proof submission to aligned. Do you want to continue?", payment))
        .interact()
        .map_err(|e | {
            error!("Failed to read user input"); 
            SubmitError::GenericError(e.to_string())
        })? {
       return Err(SubmitError::GenericError("Batcher Payment cancelled".to_string()))?;
    }

    info!("Submitting Payment to  Batcher");
    let transaction_receipt =
        deposit_to_aligned(U256::from(args.batcher_payment), signer, network.clone()).await?;

    let max_fee = U256::from(args.max_fee);

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
        /*
        .map_err(|e| {
            error!("Could not retrieve nonce from batcher");
            return Err(e)?;
        })
        */?;

    info!("Submitting proof to Aligned for Verification");

    let Ok(aligned_verification_data) = submit_and_wait_verification(
        BATCHER_URL,
        &args.rpc_url,
        network,
        &verification_data,
        max_fee,
        wallet,
        nonce,
    )
    .await
    else {
        error!("Proof Submission to Aligned failed");
        return Ok(());
    };

    save_response(
        PathBuf::from(".".to_string()).clone(),
        &aligned_verification_data,
    )?;

    info!("Proof Submission to Aligned Succeeded. See the batch in the explorer:");
    info!(
        "https://explorer.alignedlayer.com/batches/0x{}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );
    println!("Aligned Verification Data saved to root");
    Ok(())
}

fn save_response(
    batch_inclusion_data_directory_path: PathBuf,
    aligned_verification_data: &AlignedVerificationData,
) -> Result<(), SubmitError> {
    let batch_merkle_root = &hex::encode(aligned_verification_data.batch_merkle_root)[..8];
    let batch_inclusion_data_file_name = batch_merkle_root.to_owned()
        + "_"
        + &aligned_verification_data.index_in_batch.to_string()
        + ".json";

    let batch_inclusion_data_path =
        batch_inclusion_data_directory_path.join(batch_inclusion_data_file_name);

    let data = cbor_serialize(&aligned_verification_data)
        .map_err(|e| SubmitError::SerializationError(e))?;

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
