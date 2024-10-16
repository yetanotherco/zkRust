use aligned_sdk::core::types::ProvingSystemId;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::error;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
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
            let proof_data_dir = PathBuf::from(&args.proof_data_directory_path);
            if !proof_data_dir.exists() {
                std::fs::create_dir_all(proof_data_dir).unwrap_or(info!(
                    "Saving Proofs to: {:?}",
                    &args.proof_data_directory_path
                ));
            }
            if utils::validate_directory_structure(&args.guest_path) {
                let Some(home_dir) = dirs::home_dir() else {
                    error!("Failed to locate home directory");
                    return Ok(());
                };
                let Ok(current_dir) = std::env::current_dir() else {
                    error!("Failed to locate current directory");
                    return Ok(());
                };
                let home_dir = home_dir.join(".zkRust");
                utils::prepare_workspace(
                    &PathBuf::from(&args.guest_path),
                    &home_dir.join(sp1::SP1_SRC_DIR),
                    &home_dir.join(sp1::SP1_GUEST_CARGO_TOML),
                    &home_dir.join("workspaces/sp1/script"),
                    &home_dir.join("workspaces/sp1/script/Cargo.toml"),
                    &home_dir.join(sp1::SP1_BASE_HOST_CARGO_TOML),
                    &home_dir.join(sp1::SP1_BASE_GUEST_CARGO_TOML),
                )?;

                let Ok(imports) = utils::get_imports(&home_dir.join(sp1::SP1_GUEST_MAIN)) else {
                    error!("Failed to extract imports");
                    return Ok(());
                };

                let main_path = home_dir.join(sp1::SP1_GUEST_MAIN);
                let Ok(function_bodies) = utils::extract_function_bodies(
                    &main_path,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to extract function bodies");
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
                    &home_dir.join(sp1::SP1_GUEST_MAIN),
                )?;
                sp1::prepare_host(
                    &function_bodies[1],
                    &function_bodies[2],
                    &imports,
                    &home_dir.join(sp1::SP1_BASE_HOST),
                    &home_dir.join(sp1::SP1_HOST_MAIN),
                )?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true) // Open the file in append mode
                        .open(home_dir.join(sp1::SP1_GUEST_CARGO_TOML))?;

                    writeln!(toml_file, "{}", sp1::SP1_ACCELERATION_IMPORT)?;
                }

                let script_dir = home_dir.join(sp1::SP1_SCRIPT_DIR);
                let program_dir = home_dir.join(sp1::SP1_SRC_DIR);
                if sp1::generate_sp1_proof(&script_dir, &program_dir, &current_dir)?.success() {
                    info!("SP1 proof and ELF generated");

                    utils::replace(
                        &home_dir.join(sp1::SP1_GUEST_CARGO_TOML),
                        sp1::SP1_ACCELERATION_IMPORT,
                        "",
                    )?;

                    // Submit to aligned
                    if args.submit_to_aligned {
                        submit_proof_to_aligned(
                            sp1::SP1_PROOF_PATH,
                            sp1::SP1_ELF_PATH,
                            Some(sp1::SP1_PUB_INPUT_PATH),
                            args,
                            ProvingSystemId::SP1,
                        )
                        .await
                        .map_err(|e| {
                            error!("Proof not submitted to Aligned");
                            io::Error::other(e.to_string())
                        })?;
                        info!("SP1 proof submitted and verified on Aligned");
                    }

                    std::fs::copy(
                        home_dir.join(sp1::SP1_BASE_HOST_FILE),
                        home_dir.join(sp1::SP1_HOST_MAIN),
                    )
                    .map_err(|e| {
                        error!("Failed to clear SP1 host file");
                        e
                    })?;
                    return Ok(());
                }
                info!("SP1 proof generation failed");

                // Clear host
                std::fs::copy(
                    home_dir.join(sp1::SP1_BASE_HOST_FILE),
                    home_dir.join(sp1::SP1_HOST_MAIN),
                )?;
                return Ok(());
            } else {
                error!("zkRust directory structure invalid please consult the README",);
                return Ok(());
            }
        }

        Commands::ProveRisc0(args) => {
            info!("Proving with Risc0, program in: {}", args.guest_path);

            // Perform sanitation checks on directory
            if utils::validate_directory_structure(&args.guest_path) {
                let proof_data_dir = PathBuf::from(&args.proof_data_directory_path);
                if !proof_data_dir.exists() {
                    std::fs::create_dir_all(proof_data_dir).unwrap_or(info!(
                        "Saving generated proofs to: {:?}",
                        &args.proof_data_directory_path
                    ));
                }
                let Some(home_dir) = dirs::home_dir() else {
                    error!("Failed to locate home directory");
                    return Ok(());
                };
                let Ok(current_dir) = std::env::current_dir() else {
                    error!("Failed to locate current directory");
                    return Ok(());
                };
                let home_dir = home_dir.join(".zkRust");
                utils::prepare_workspace(
                    &PathBuf::from(&args.guest_path),
                    &home_dir.join(risc0::RISC0_SRC_DIR),
                    &home_dir.join(risc0::RISC0_GUEST_CARGO_TOML),
                    &home_dir.join("workspaces/risc0/host"),
                    &home_dir.join("workspaces/risc0/host/Cargo.toml"),
                    &home_dir.join(risc0::RISC0_BASE_HOST_CARGO_TOML),
                    &home_dir.join(risc0::RISC0_BASE_GUEST_CARGO_TOML),
                )?;

                let Ok(imports) = utils::get_imports(&home_dir.join(risc0::RISC0_GUEST_MAIN))
                else {
                    error!("Failed to extract imports");
                    return Ok(());
                };
                let main_path = home_dir.join(risc0::RISC0_GUEST_MAIN);
                let Ok(function_bodies) = utils::extract_function_bodies(
                    &main_path,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to extract function bodies");
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
                    &home_dir.join(risc0::RISC0_GUEST_MAIN),
                )?;
                risc0::prepare_host(
                    &function_bodies[1],
                    &function_bodies[2],
                    &imports,
                    &home_dir.join(risc0::RISC0_BASE_HOST),
                    &home_dir.join(risc0::RISC0_HOST_MAIN),
                )?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true)
                        .open(home_dir.join(risc0::RISC0_GUEST_CARGO_TOML))?;

                    writeln!(toml_file, "{}", risc0::RISC0_ACCELERATION_IMPORT)?;
                }

                let workspace_dir = home_dir.join(risc0::RISC0_WORKSPACE_DIR);
                if risc0::generate_risc0_proof(&workspace_dir, &current_dir)?.success() {
                    info!("Risc0 proof and Image ID generated");

                    utils::replace(
                        &home_dir.join(risc0::RISC0_GUEST_CARGO_TOML),
                        risc0::RISC0_ACCELERATION_IMPORT,
                        "",
                    )?;

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
                            error!("Error submitting proofs to Aligned: {:?}", e);
                            io::Error::other(e.to_string())
                        })?;

                        info!("Risc0 proof submitted and verified on Aligned");
                    }

                    // Clear Host file
                    std::fs::copy(
                        home_dir.join(risc0::RISC0_BASE_HOST_FILE),
                        home_dir.join(risc0::RISC0_HOST_MAIN),
                    )
                    .map_err(|e| {
                        error!("Failed to clear Risc0 host file");
                        e
                    })?;
                    return Ok(());
                }
                info!("Risc0 proof generation failed");

                // Clear Host file
                std::fs::copy(
                    home_dir.join(risc0::RISC0_BASE_HOST_FILE),
                    home_dir.join(risc0::RISC0_HOST_MAIN),
                )?;
                return Ok(());
            } else {
                error!("zkRust directory structure incorrect please consult the README",);
                return Ok(());
            }
        }
    }
}
