use aligned_sdk::types::ProvingSystemId;
use clap::{Args, Parser, Subcommand};
use log::info;
use std::io;
use std::path::PathBuf;
use zkRust::risc0;
use zkRust::sp1;
use zkRust::submit_proof_to_aligned;
use zkRust::utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Add flags that specify using proof system specific patches such as rand, sha, etc.

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    ProveSp1(ProofArgs),
    ProveRisc0(ProofArgs),
}

#[derive(Args, Debug)]
struct ProofArgs {
    guest_path: String,
    output_proof_path: String,
    #[clap(long)]
    submit_to_aligned_with_keystore: Option<PathBuf>,
    #[clap(long)]
    std: bool,
}

fn main() -> io::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            info!("proving with sp1, program in: {}", args.guest_path);

            utils::prepare_workspace(
                &args.guest_path,
                sp1::SP1_SRC_DIR,
                sp1::SP1_GUEST_CARGO_TOML,
                sp1::SP1_BASE_CARGO_TOML,
            )?;

            sp1::prepare_sp1_program()?;
            sp1::generate_sp1_proof()?;

            info!("sp1 proof and ELF generated");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    sp1::SP1_PROOF_PATH,
                    sp1::SP1_ELF_PATH,
                    ProvingSystemId::SP1,
                )
                .unwrap();
                info!("sp1 proof submitted and verified on aligned");
            }

            Ok(())
        }

        Commands::ProveRisc0(args) => {
            info!("proving with risc0, program in: {}", args.guest_path);

            utils::prepare_workspace(
                &args.guest_path,
                risc0::RISC0_SRC_DIR,
                risc0::RISC0_GUEST_CARGO_TOML,
                risc0::RISC0_BASE_CARGO_TOML,
            )?;

            risc0::prepare_risc0_guest()?;
            risc0::generate_risc0_proof()?;

            info!("risc0 proof and image ID generated");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    risc0::RISC0_PROOF_PATH,
                    risc0::RISC0_IMAGE_PATH,
                    ProvingSystemId::Risc0,
                )
                .unwrap();

                info!("risc0 proof submitted and verified on aligned");
            }

            Ok(())
        }
    }
}
