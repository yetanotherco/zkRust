use std::{
    fs, io,
    process::{Command, Stdio},
};

use crate::utils;

/// SP1 workspace directories
pub const SP1_SCRIPT_DIR: &str = "./workspaces/sp1/script";
pub const SP1_GUEST_DIR: &str = "./workspaces/sp1/program/";
pub const SP1_SRC_DIR: &str = "./workspaces/sp1/program/src";
pub const SP1_GUEST_MAIN: &str = "./workspaces/sp1/program/src/main.rs";
pub const SP1_BASE_CARGO_TOML: &str = "./workspaces/base_files/sp1";
pub const SP1_GUEST_CARGO_TOML: &str = "./workspaces/sp1/program/Cargo.toml";
pub const SP1_ELF_PATH: &str = "./proof_data/sp1/sp1.elf";
pub const SP1_PROOF_PATH: &str = "./proof_data/sp1/sp1.proof";

/// SP1 header added to programs for generating proofs of their execution
pub const SP1_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// This function mainly adds this header to the guest in order for it to be proven by
/// sp1:
///
///    #![no_main]
///    sp1_zkvm::entrypoint!(main);
///
pub fn prepare_sp1_program() -> io::Result<()> {
    utils::prepend_to_file(SP1_GUEST_MAIN, SP1_PROGRAM_HEADER)?;
    Ok(())
}

/// Generates SP1 proof and ELF
pub fn generate_sp1_proof() -> io::Result<()> {
    let guest_path = fs::canonicalize(SP1_SCRIPT_DIR)?;
    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(guest_path)
        .status()?;

    Ok(())
}
