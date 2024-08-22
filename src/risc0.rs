use std::{fs, io, process::Command};

use crate::utils;

/// RISC0 workspace directories
pub const RISC0_WORKSPACE_DIR: &str = "./workspaces/risc0/";
pub const RISC0_GUEST_DIR: &str = "./workspaces/risc0/methods/guest/";
pub const RISC0_SRC_DIR: &str = "./workspaces/risc0/methods/guest/src";
pub const RISC0_GUEST_MAIN: &str = "./workspaces/risc0/methods/guest/src/main.rs";
pub const RISC0_HOST_MAIN: &str = "./workspaces/risc0/host/src/main.rs";
pub const RISC0_BASE_CARGO_TOML: &str = "./workspaces/base_files/risc0/cargo";
pub const RISC0_BASE_HOST: &str = "./workspaces/base_files/risc0/host";
pub const RISC0_GUEST_CARGO_TOML: &str = "./workspaces/risc0/methods/guest/Cargo.toml";

// Proof data generation paths
pub const PROOF_FILE_PATH: &str = "./proof_data/risc0/risc0.proof";
pub const IMAGE_ID_FILE_PATH: &str = "./proof_data/risc0/risc0.imageid";
pub const PUBLIC_INPUT_FILE_PATH: &str = "./proof_data/risc0/risc0_pub_input.pub";

/// RISC0 User I/O Markers
// HOST
pub const RISC0_ENV_BUILDER: &str = "let env = ExecutorEnv::builder()";
pub const RISC0_IO_HOST: &str = "risc0_zkvm::ExecutorEnv::builder()";
pub const RISC0_IO_HOST_BUILD: &str = ".build().unwrap();";

// GUEST
pub const RISC0_IO_READ: &str = "risc0_zkvm::guest::env::read();";
pub const RISC0_IO_WRITE: &str = "risc0_zkvm::guest::env::write";
pub const RISC0_IO_COMMIT: &str = "risc0_zkvm::guest::env::commit";
pub const RISC0_IO_OUT: &str = "receipt.journal.decode().unwrap();";

/// RISC0 header added to programs for generating proofs of their execution
pub const RISC0_GUEST_PROGRAM_HEADER_STD: &str =
    "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

/// RISC0 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const RISC0_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/risc0/RustCrypto-hashes\", tag = \"sha2-v0.10.6-risczero.0\" }\nk256 = { git = \"https://github.com/risc0/RustCrypto-elliptic-curves\", tag = \"k256/v0.13.1-risczero.1\"  }\ncrypto-bigint = { git = \"https://github.com/risc0/RustCrypto-crypto-bigint\", tag = \"v0.5.2-risczero.0\" }";

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

pub fn prepare_guest_io() -> io::Result<()> {
    // replace zkRust::read()
    utils::replace(RISC0_GUEST_MAIN, utils::IO_READ, RISC0_IO_READ)?;

    // replace zkRust::commit()
    utils::replace(RISC0_GUEST_MAIN, utils::IO_COMMIT, RISC0_IO_COMMIT)?;

    Ok(())
}

pub fn prepare_host_io(guest_path: &str) -> io::Result<()> {
    //TODO: remove output & input functions after copying
    let input_path = format!("{}/src/input.rs", guest_path);
    let input = utils::extract(&input_path, utils::INPUT_FUNC, 2)?.unwrap();
    // Extract output body
    let output_path = format!("{}/src/output.rs", guest_path);
    let output = utils::extract(&output_path, utils::OUTPUT_FUNC, 2)?.unwrap();
    // Insert input body
    utils::insert(RISC0_HOST_MAIN, &input, utils::HOST_INPUT)?;
    // Insert output body
    utils::insert(RISC0_HOST_MAIN, &output, utils::HOST_OUTPUT)?;


    // Extract Variable names from host and add them to the ExecutorEnv::builder()
    let values = utils::extract_values(RISC0_HOST_MAIN, utils::IO_WRITE)?;

    // Construct new Environment Builder
    let mut new_builder = RISC0_ENV_BUILDER.to_string();
    for value in values {
        new_builder.push_str(&format!(".write({}).unwrap()", value));
    }
    new_builder.push_str(".build().unwrap();");

    // Replace environment builder in host with new one
    utils::replace(RISC0_HOST_MAIN, "let env = ExecutorEnv::builder().build().unwrap();", &new_builder)?;

    //TODO: FRAGILE!
    //Delete lines that contain zkRust::write(;
    utils::remove_lines(RISC0_HOST_MAIN, "zk_rust_io::write(")?;

    // replace zkRust::out()
    utils::replace(RISC0_HOST_MAIN, utils::IO_OUT, RISC0_IO_OUT)?;
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
