use std::{fs, io, process::Command};

use crate::utils;

/// RISC0 workspace directories
pub const RISC0_PROOF_PATH: &str = "./risc_zero.proof";
pub const RISC0_IMAGE_PATH: &str = "./risc_zero_image_id.bin";
pub const RISC0_WORKSPACE_DIR: &str = "./workspaces/risc0/";
pub const RISC0_GUEST_DIR: &str = "./workspaces/risc0/methods/guest/";
pub const RISC0_SRC_DIR: &str = "./workspaces/risc0/methods/guest/src";
pub const RISC0_GUEST_MAIN: &str = "./workspaces/risc0/methods/guest/src/main.rs";
pub const RISC0_BASE_CARGO_TOML: &str = "./workspaces/base_files/risc0";
pub const RISC0_GUEST_CARGO_TOML: &str = "./workspaces/risc0/methods/guest/Cargo.toml";

/// RISC0 header added to programs for generating proofs of their execution
pub const RISC0_GUEST_PROGRAM_HEADER_STD: &str =
    "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

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
pub fn generate_risc0_proof() -> io::Result<()> {
    let guest_path = fs::canonicalize(RISC0_WORKSPACE_DIR)?;

    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(guest_path)
        .status()
        .unwrap();

    Ok(())
}
