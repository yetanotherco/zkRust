use aligned_sdk::core::types::ProvingSystemId;
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
    #[clap(about = "Generate a proof of execution of a program using SP1")]
    ProveSp1(ProofArgs),
    #[clap(about = "Generate a proof of execution of a program using RISC0")]
    ProveRisc0(ProofArgs),
}

#[derive(Args, Debug)]
struct ProofArgs {
    guest_path: String,
    output_proof_path: String,
    #[clap(long)]
    submit_to_aligned_with_keystore: Option<PathBuf>,
    #[clap(long)]
    io: bool,
    #[clap(long)]
    precompiles: bool,
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
                "./workspaces/sp1/script/src",
                "./workspaces/sp1/script/Cargo.toml",
                sp1::SP1_BASE_HOST_CARGO_TOML,
                sp1::SP1_BASE_GUEST_CARGO_TOML,
            )?;
            std::fs::copy(sp1::SP1_BASE_HOST, sp1::SP1_HOST_MAIN).unwrap();

            sp1::prepare_sp1_program()?;
            if args.io {
                sp1::prepare_guest_io()?;
                sp1::prepare_host_io(&args.guest_path)?;
            }

            if args.precompiles {
                utils::insert(
                    sp1::SP1_GUEST_CARGO_TOML,
                    sp1::SP1_ACCELERATION_IMPORT,
                    "[workspace]",
                )
                .unwrap();
            }

            if sp1::generate_sp1_proof()?.success() {
                info!("sp1 proof and ELF generated");

                utils::replace(sp1::SP1_GUEST_CARGO_TOML, sp1::SP1_ACCELERATION_IMPORT, "")
                    .unwrap();

                // Submit to aligned
                if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                    submit_proof_to_aligned(
                        keystore_path,
                        sp1::SP1_PROOF_PATH,
                        sp1::SP1_ELF_PATH,
                        None,
                        ProvingSystemId::SP1,
                    )
                    .unwrap();
                    info!("sp1 proof submitted and verified on aligned");
                }

                // Clear host & guest
                std::fs::copy(sp1::SP1_BASE_HOST, sp1::SP1_HOST_MAIN).unwrap();

                return Ok(());
            }

            info!("SP1 proof generation failed");
            // Clear host
            std::fs::copy(sp1::SP1_BASE_HOST, sp1::SP1_HOST_MAIN).unwrap();
            Ok(())
        }

        Commands::ProveRisc0(args) => {
            info!("proving with risc0, program in: {}", args.guest_path);

            //TODO: to add input to the guest we need to modify the host....
            // This shouldn't be ridiculously hard if we get
            utils::prepare_workspace(
                &args.guest_path,
                risc0::RISC0_SRC_DIR,
                risc0::RISC0_GUEST_CARGO_TOML,
                "./workspaces/risc0/host/src",
                "./workspaces/risc0/host/Cargo.toml",
                risc0::RISC0_BASE_HOST_CARGO_TOML,
                risc0::RISC0_BASE_GUEST_CARGO_TOML,
            )?;
            std::fs::copy(risc0::RISC0_BASE_HOST, risc0::RISC0_HOST_MAIN).unwrap();

            risc0::prepare_risc0_guest()?;
            if args.io {
                risc0::prepare_guest_io()?;
                risc0::prepare_host_io(&args.guest_path)?;
            }

            if args.precompiles {
                utils::insert(
                    risc0::RISC0_GUEST_CARGO_TOML,
                    risc0::RISC0_ACCELERATION_IMPORT,
                    "[workspace]",
                )
                .unwrap();
            }

            if risc0::generate_risc0_proof()?.success() {
                info!("Risc0 proof and Image ID generated");

                utils::replace(
                    risc0::RISC0_GUEST_CARGO_TOML,
                    risc0::RISC0_ACCELERATION_IMPORT,
                    "",
                )
                .unwrap();

                // Submit to aligned
                if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                    submit_proof_to_aligned(
                        keystore_path,
                        risc0::PROOF_FILE_PATH,
                        risc0::IMAGE_ID_FILE_PATH,
                        Some(risc0::PUBLIC_INPUT_FILE_PATH),
                        ProvingSystemId::Risc0,
                    )
                    .unwrap();

                    info!("risc0 proof submitted and verified on aligned");
                }

                // Clear Host file
                std::fs::copy(risc0::RISC0_BASE_HOST, risc0::RISC0_HOST_MAIN).unwrap();

                return Ok(());
            }

            info!("Risc0 proof generation failed");

            // Clear Host file
            std::fs::copy(risc0::RISC0_BASE_HOST, risc0::RISC0_HOST_MAIN).unwrap();
            Ok(())
        }
    }
}
