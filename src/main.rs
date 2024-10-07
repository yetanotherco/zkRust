use aligned_sdk::core::types::ProvingSystemId;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::error;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::io;
use zkRust::{risc0, sp1, submit_proof_to_aligned, utils, ProofArgs};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate a proof of execution of a program using SP1")]
    ProveSp1(ProofArgs),
    #[clap(about = "Generate a proof of execution of a program using RISC0")]
    ProveRisc0(ProofArgs),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            info!("Proving with SP1, program in: {}", args.guest_path);

            // Perform sanitation checks on directory
            if utils::validate_directory_structure(&args.guest_path) {
                utils::prepare_workspace(
                    &args.guest_path,
                    sp1::SP1_SRC_DIR,
                    sp1::SP1_GUEST_CARGO_TOML,
                    "./workspaces/sp1/script",
                    "./workspaces/sp1/script/Cargo.toml",
                    sp1::SP1_BASE_HOST_CARGO_TOML,
                    sp1::SP1_BASE_GUEST_CARGO_TOML,
                )?;

                let Ok(imports) = utils::get_imports(sp1::SP1_GUEST_MAIN) else {
                    error!("Failed to Extract Imports");
                    return Ok(());
                };
                let Ok(function_bodies) = utils::extract_function_bodies(
                    sp1::SP1_GUEST_MAIN,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to Extract Function Bodies");
                    return Ok(());
                };
                /*
                    Adds header to the guest & replace I/O imports
                    risc0:

                        #![no_main]
                        sp1_zkvm::entrypoint!(main);
                */
                utils::prepare_guest(
                    &imports,
                    &function_bodies[0],
                    sp1::SP1_GUEST_PROGRAM_HEADER,
                    sp1::SP1_IO_READ,
                    sp1::SP1_IO_COMMIT,
                    sp1::SP1_GUEST_MAIN,
                )?;
                sp1::prepare_host(&function_bodies[1], &function_bodies[2], &imports)?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true) // Open the file in append mode
                        .open(sp1::SP1_GUEST_CARGO_TOML)?;

                    writeln!(toml_file, "{}", sp1::SP1_ACCELERATION_IMPORT)?;
                }

                if sp1::generate_sp1_proof()?.success() {
                    info!("SP1 proof and ELF generated");

                    utils::replace(sp1::SP1_GUEST_CARGO_TOML, sp1::SP1_ACCELERATION_IMPORT, "")?;

                    // Submit to aligned
                    if args.submit_to_aligned {
                        submit_proof_to_aligned(
                            sp1::SP1_PROOF_PATH,
                            sp1::SP1_ELF_PATH,
                            None,
                            args,
                            ProvingSystemId::SP1,
                        )
                        .await
                        .map_err(|e| {
                            error!("Error Submitting Proof to Aligned: {:?}", e);
                            io::Error::other(e.to_string())
                        })?;
                        info!("SP1 proof submitted and verified on Aligned");
                    }

                    std::fs::copy(sp1::SP1_BASE_HOST_FILE, sp1::SP1_HOST_MAIN).map_err(|e| {
                        error!("Failed to clear SP1 Host File");
                        e
                    })?;

                    return Ok(());
                }
                info!("SP1 proof generation failed");
                // Clear host
                std::fs::copy(sp1::SP1_BASE_HOST_FILE, sp1::SP1_HOST_MAIN)?;
                return Ok(());
            } else {
                error!("zkRust Directory structure incorrect please consult the README",);
                return Ok(());
            }
        }

        Commands::ProveRisc0(args) => {
            info!("Proving with Risc0, program in: {}", args.guest_path);

            // Perform sanitation checks on directory
            if utils::validate_directory_structure(&args.guest_path) {
                utils::prepare_workspace(
                    &args.guest_path,
                    risc0::RISC0_SRC_DIR,
                    risc0::RISC0_GUEST_CARGO_TOML,
                    "./workspaces/risc0/host",
                    "./workspaces/risc0/host/Cargo.toml",
                    risc0::RISC0_BASE_HOST_CARGO_TOML,
                    risc0::RISC0_BASE_GUEST_CARGO_TOML,
                )?;

                let Ok(imports) = utils::get_imports(risc0::RISC0_GUEST_MAIN) else {
                    error!("Failed to Extract Imports");
                    return Ok(());
                };
                let Ok(function_bodies) = utils::extract_function_bodies(
                    risc0::RISC0_GUEST_MAIN,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to Extract Function Bodies");
                    return Ok(());
                };

                /*
                    Adds header to the guest & replace I/O imports
                    risc0:

                        #![no_main]
                        risc0_zkvm::guest::entry!(main);
                */
                utils::prepare_guest(
                    &imports,
                    &function_bodies[0],
                    risc0::RISC0_GUEST_PROGRAM_HEADER,
                    risc0::RISC0_IO_READ,
                    risc0::RISC0_IO_COMMIT,
                    risc0::RISC0_GUEST_MAIN,
                )?;
                risc0::prepare_host(&function_bodies[1], &function_bodies[2], &imports)?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true)
                        .open(risc0::RISC0_GUEST_CARGO_TOML)?;

                    writeln!(toml_file, "{}", risc0::RISC0_ACCELERATION_IMPORT)?;
                }

                if risc0::generate_risc0_proof()?.success() {
                    info!("Risc0 proof and Image ID generated");

                    // Submit to aligned
                    if args.submit_to_aligned {
                        submit_proof_to_aligned(
                            risc0::PROOF_FILE_PATH,
                            risc0::IMAGE_ID_FILE_PATH,
                            Some(risc0::PUBLIC_INPUT_FILE_PATH),
                            args,
                            ProvingSystemId::Risc0,
                        )
                        .await
                        .map_err(|e| {
                            error!("Error Submitting Proof to Aligned: {:?}", e);
                            io::Error::other(e.to_string())
                        })?;

                        info!("Risc0 proof submitted and verified on Aligned");
                    }

                    // Clear Host file
                    std::fs::copy(risc0::RISC0_BASE_HOST_FILE, risc0::RISC0_HOST_MAIN).map_err(
                        |e| {
                            error!("Failed to Clear Risc0 Host File");
                            e
                        },
                    )?;

                    return Ok(());
                }
                info!("Risc0 proof generation failed");

                // Clear Host file
                std::fs::copy(risc0::RISC0_BASE_HOST_FILE, risc0::RISC0_HOST_MAIN)?;
                return Ok(());
            } else {
                error!("zkRust Directory structure incorrect please consult the README",);
                return Ok(());
            }
        }
    }
}
