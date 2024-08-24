use std::{fs, io, path::PathBuf, process::Command};
use clap::Args;

use crate::utils;

/// RISC0 workspace directories
pub const RISC0_WORKSPACE_DIR: &str = "./workspaces/risc0/";
pub const RISC0_GUEST_DIR: &str = "./workspaces/risc0/methods/guest/";
pub const RISC0_SRC_DIR: &str = "./workspaces/risc0/methods/guest/src";
pub const RISC0_GUEST_MAIN: &str = "./workspaces/risc0/methods/guest/src/main.rs";
pub const RISC0_BASE_CARGO_TOML: &str = "./workspaces/base_files/risc0";
pub const RISC0_GUEST_CARGO_TOML: &str = "./workspaces/risc0/methods/guest/Cargo.toml";

// Proof data generation paths
pub const PROOF_FILE_PATH: &str = "./proof_data/risc0/risc0.proof";
pub const IMAGE_ID_FILE_PATH: &str = "./proof_data/risc0/risc0.imageid";
pub const PUBLIC_INPUT_FILE_PATH: &str = "./proof_data/risc0/risc0_pub_input.pub";

/// RISC0 header added to programs for generating proofs of their execution
pub const RISC0_GUEST_PROGRAM_HEADER_STD: &str =
    "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

/// RISC0 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const RISC0_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/risc0/RustCrypto-hashes\", tag = \"sha2-v0.10.6-risczero.0\" }\nk256 = { git = \"https://github.com/risc0/RustCrypto-elliptic-curves\", tag = \"k256/v0.13.1-risczero.1\"  }\ncrypto-bigint = { git = \"https://github.com/risc0/RustCrypto-crypto-bigint\", tag = \"v0.5.5-risczero.0\" }";

#[derive(Args, Debug)]
pub struct Risc0Args {
    pub guest_path: String,
    pub output_proof_path: String,
    #[clap(long)]
    pub submit_to_aligned_with_keystore: Option<PathBuf>,
    #[clap(long)]
    pub std: bool,
    #[clap(long)]
    pub precompiles: bool,
    #[clap(long)]
    pub cuda: bool,
    #[clap(long)]
    pub metal: bool,
}

/// This function mainly adds this header to the guest in order for it to be proven by
/// risc0:
///
///    #![no_main]
///    risc0_zkvm::guest::entry!(main);
///
pub fn prepare_risc0_guest() -> io::Result<()> {
    utils::prepend_to_file(RISC0_GUEST_MAIN, RISC0_GUEST_PROGRAM_HEADER_STD)?;
    Ok(())
}

/// Generates RISC0 proof and image ID
pub fn generate_risc0_proof(args: &Risc0Args) -> io::Result<()> {
    let guest_path = fs::canonicalize(RISC0_WORKSPACE_DIR)?;

    if args.cuda {
        Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("-F")
            .arg("cuda")
            .current_dir(guest_path)
            .status()
            .unwrap();

    } else if args.metal {
        Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("-F")
            .arg("metal")
            .current_dir(guest_path)
            .status()
            .unwrap();
    } else {
        Command::new("cargo")
            .arg("run")
            .arg("--release")
            .current_dir(guest_path)
            .status()
            .unwrap();
    }

    Ok(())
}
