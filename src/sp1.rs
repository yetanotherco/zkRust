use std::{fs, io, process::Command};

use crate::utils;

/// SP1 workspace directories
pub const SP1_SCRIPT_DIR: &str = "./workspaces/sp1/script";
pub const SP1_GUEST_DIR: &str = "./workspaces/sp1/program/";
pub const SP1_SRC_DIR: &str = "./workspaces/sp1/program/src";
pub const SP1_GUEST_MAIN: &str = "./workspaces/sp1/program/src/main.rs";
pub const SP1_HOST_MAIN: &str = "./workspaces/sp1/script/src/main.rs";
pub const SP1_BASE_CARGO_TOML: &str = "./workspaces/base_files/sp1/cargo";
pub const SP1_BASE_HOST: &str = "./workspaces/base_files/sp1/host";
pub const SP1_GUEST_CARGO_TOML: &str = "./workspaces/sp1/program/Cargo.toml";
pub const SP1_ELF_PATH: &str = "./proof_data/sp1/sp1.elf";
pub const SP1_PROOF_PATH: &str = "./proof_data/sp1/sp1.proof";

/// SP1 header added to programs for generating proofs of their execution
pub const SP1_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// SP1 User I/O
// Host
pub const SP1_HOST_WRITE: &str = "stdin.write";
pub const SP1_HOST_READ: &str = "proof.public_values.read();";

// Guest
pub const SP1_IO_READ: &str = "sp1_zkvm::io::read();";
pub const SP1_IO_COMMIT: &str = "sp1_zkvm::io::commit";

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

pub fn prepare_guest_io() -> io::Result<()> {

    // replace zkRust::read()
    utils::replace(SP1_GUEST_MAIN, utils::IO_READ, SP1_IO_READ)?;

    // replace zkRust::commit()
    utils::replace(SP1_GUEST_MAIN, utils::IO_COMMIT, SP1_IO_COMMIT)?;
    Ok(())
}

pub fn prepare_host_io(guest_path: &str) -> io::Result<()> {
    // Extract input body
    let input_path = format!("{}/src/input.rs", guest_path);
    let input = utils::extract(&input_path, utils::INPUT_FUNC, 1)?.unwrap();
    // Extract output body
    let output_path = format!("{}/src/output.rs", guest_path);
    let output = utils::extract(&output_path, utils::OUTPUT_FUNC, 1)?.unwrap();
    // Insert input body
    utils::insert(SP1_HOST_MAIN, &input, utils::HOST_INPUT)?;
    // Insert output body
    utils::insert(SP1_HOST_MAIN, &output, utils::HOST_OUTPUT)?;
    // replace zkRust::write
    utils::replace(SP1_HOST_MAIN, utils::IO_WRITE, SP1_HOST_WRITE)?;
    // replace zkRust::out()
    utils::replace(SP1_HOST_MAIN, utils::IO_OUT, SP1_HOST_READ)?;
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
