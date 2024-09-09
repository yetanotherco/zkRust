use aligned_sdk::core::types::ProvingSystemId;
use clap::{Args, Parser, Subcommand};
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
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
    submit_to_aligned: bool,
    #[clap(long, default_value = "./keystore")]
    keystore_path: Option<PathBuf>,
    #[clap(
        long,
        default_value("https://ethereum-holesky-rpc.publicnode.com"),
        required_if_eq("submit_to_aligned", "true")
    )]
    rpc_url: String,
    #[clap(
        long,
        default_value("17000"),
        required_if_eq("submit_to_aligned", "true")
    )]
    chain_id: u64,
    #[clap(long)]
    precompiles: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            info!("Proving with SP1, program in: {}", args.guest_path);

            // Perform sanitation checks on directory
            match utils::validate_directory_structure(&args.guest_path) {
                Ok(_) => {
                    //TODO: to add input to the guest we need to modify the host....
                    utils::prepare_workspace(
                        &args.guest_path,
                        sp1::SP1_SRC_DIR,
                        sp1::SP1_GUEST_CARGO_TOML,
                        "./workspaces/sp1/script",
                        "./workspaces/sp1/script/Cargo.toml",
                        sp1::SP1_BASE_HOST_CARGO_TOML,
                        sp1::SP1_BASE_GUEST_CARGO_TOML,
                    )?;

                    //TODO: fetch these from program directory file.......
                    let imports = utils::get_imports(sp1::SP1_GUEST_MAIN).unwrap();
                    let function_bodies = utils::extract_function_bodies(
                        sp1::SP1_GUEST_MAIN,
                        vec![
                            "fn main()".to_string(),
                            "fn input()".to_string(),
                            "fn output()".to_string(),
                        ],
                    )
                    .unwrap();
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

                        utils::replace(sp1::SP1_GUEST_CARGO_TOML, sp1::SP1_ACCELERATION_IMPORT, "")
                            .unwrap();

                        // Submit to aligned
                        if args.submit_to_aligned {
                            submit_proof_to_aligned(
                                &args.keystore_path.as_ref().unwrap(),
                                sp1::SP1_PROOF_PATH,
                                sp1::SP1_ELF_PATH,
                                None,
                                &args.rpc_url,
                                &args.chain_id,
                                ProvingSystemId::SP1,
                            )
                            .unwrap();
                            info!("SP1 proof submitted and verified on aligned");
                        }

                        // Clear host & guest
                        std::fs::copy(sp1::SP1_BASE_HOST_FILE, sp1::SP1_HOST_MAIN).unwrap();

                        return Ok(());
                    }
                    info!("SP1 proof generation failed");
                    // Clear host
                    std::fs::copy(sp1::SP1_BASE_HOST_FILE, sp1::SP1_HOST_MAIN)?;
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }

        Commands::ProveRisc0(args) => {
            info!("Proving with Risc0, program in: {}", args.guest_path);

            // Perform sanitation checks on directory
            match utils::validate_directory_structure(&args.guest_path) {
                Ok(_) => {
                    utils::prepare_workspace(
                        &args.guest_path,
                        risc0::RISC0_SRC_DIR,
                        risc0::RISC0_GUEST_CARGO_TOML,
                        "./workspaces/risc0/host",
                        "./workspaces/risc0/host/Cargo.toml",
                        risc0::RISC0_BASE_HOST_CARGO_TOML,
                        risc0::RISC0_BASE_GUEST_CARGO_TOML,
                    )?;

                    let imports = utils::get_imports(risc0::RISC0_GUEST_MAIN).unwrap();
                    let function_bodies = utils::extract_function_bodies(
                        risc0::RISC0_GUEST_MAIN,
                        vec![
                            "fn main()".to_string(),
                            "fn input()".to_string(),
                            "fn output()".to_string(),
                        ],
                    )
                    .unwrap();

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
                                &args.keystore_path.as_ref().unwrap(),
                                risc0::PROOF_FILE_PATH,
                                risc0::IMAGE_ID_FILE_PATH,
                                Some(risc0::PUBLIC_INPUT_FILE_PATH),
                                &args.rpc_url,
                                &args.chain_id,
                                ProvingSystemId::Risc0,
                            )
                            .unwrap();

                            info!("Risc0 proof submitted and verified on aligned");
                        }

                        // Clear Host file
                        std::fs::copy(risc0::RISC0_BASE_HOST_FILE, risc0::RISC0_HOST_MAIN).unwrap();

                        return Ok(());
                    }
                    info!("Risc0 proof generation failed");

                    // Clear Host file
                    std::fs::copy(risc0::RISC0_BASE_HOST_FILE, risc0::RISC0_HOST_MAIN).unwrap();
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
    }
}
