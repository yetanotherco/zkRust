use aligned_sdk::types::ProvingSystemId;
use clap::{Args, Parser, Subcommand};
use std::io;
use std::path::PathBuf;
use zkRust::risc0;
use zkRust::sp1;
use zkRust::jolt;
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
    ProveJolt(ProofArgs),
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
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            println!("Proving with SP1, program in: {}", args.guest_path);

            utils::prepare_workspace(
                &args.guest_path,
                sp1::SP1_SRC_DIR,
                sp1::SP1_GUEST_CARGO_TOML,
                sp1::SP1_BASE_CARGO_TOML,
            )?;

            sp1::prepare_sp1_program()?;
            sp1::generate_sp1_proof()?;

            println!("Proof and ELF generated!");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    sp1::SP1_PROOF_PATH,
                    sp1::SP1_ELF_PATH,
                    ProvingSystemId::SP1,
                )
                .unwrap();
                println!("Proof submitted and verified on aligned");
            }

            Ok(())
        }

        Commands::ProveRisc0(args) => {
            println!("Proving with Risc0, program in: {}", args.guest_path);

            utils::prepare_workspace(
                &args.guest_path,
                risc0::RISC0_SRC_DIR,
                risc0::RISC0_GUEST_CARGO_TOML,
                risc0::RISC0_BASE_CARGO_TOML,
            )?;

            risc0::prepare_risc0_guest()?;
            risc0::generate_risc0_proof()?;

            println!("Proof and Proof Image generated!");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    risc0::RISC0_PROOF_PATH,
                    risc0::RISC0_IMAGE_PATH,
                    ProvingSystemId::Risc0,
                )
                .unwrap();

                println!("Proof submitted and verified on aligned");
            }

            Ok(())
        }
        Commands::ProveJolt(args) => {
            println!("Proving with Jolt, program in: {}", args.guest_path);

            utils::prepare_workspace(
                &args.guest_path,
                jolt::JOLT_SRC_DIR,
                jolt::JOLT_GUEST_CARGO_TOML,
                jolt::JOLT_BASE_CARGO_TOML,
            )?;

            jolt::prepare_jolt_guest()?;
            jolt::generate_jolt_proof()?;

            println!("Proof and Proof Image generated!");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    jolt::JOLT_PROOF_PATH,
                    jolt::JOLT_ELF_PATH,
                    //TODO: Change this to Jolt when upstream change is made
                    ProvingSystemId::Risc0,
                )
                .unwrap();

                println!("Proof submitted and verified on aligned");
            }

            Ok(())
        }
    }
}
