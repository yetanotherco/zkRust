use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
};

use crate::utils;

/// RISC0 workspace directories
pub const RISC0_WORKSPACE_DIR: &str = "workspaces/risc0/";
pub const RISC0_SRC_DIR: &str = "workspaces/risc0/methods/guest";
pub const RISC0_GUEST_MAIN: &str = "workspaces/risc0/methods/guest/src/main.rs";
pub const RISC0_HOST_MAIN: &str = "workspaces/risc0/host/src/main.rs";
pub const RISC0_BASE_HOST_CARGO_TOML: &str = "workspaces/base_files/risc0/cargo_host";
pub const RISC0_BASE_GUEST_CARGO_TOML: &str = "workspaces/base_files/risc0/cargo_guest";
pub const RISC0_BASE_HOST: &str = "workspaces/base_files/risc0/host";
pub const RISC0_BASE_HOST_FILE: &str = "workspaces/base_files/risc0/host";
pub const RISC0_GUEST_CARGO_TOML: &str = "workspaces/risc0/methods/guest/Cargo.toml";

// Proof data generation paths
pub const PROOF_FILE_PATH: &str = "./proof_data/risc0/risc0.proof";
pub const IMAGE_ID_FILE_PATH: &str = "./proof_data/risc0/risc0.imageid";
pub const PUBLIC_INPUT_FILE_PATH: &str = "./proof_data/risc0/risc0.pub";

//TODO: should we use std or no_std header
/// RISC0 header added to programs for generating proofs of their execution
pub const RISC0_GUEST_PROGRAM_HEADER: &str = "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

/// RISC0 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const RISC0_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/risc0/RustCrypto-hashes\", tag = \"sha2-v0.10.6-risczero.0\" }\nk256 = { git = \"https://github.com/risc0/RustCrypto-elliptic-curves\", tag = \"k256/v0.13.1-risczero.1\"  }\ncrypto-bigint = { git = \"https://github.com/risc0/RustCrypto-crypto-bigint\", tag = \"v0.5.5-risczero.0\" }";

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

pub fn prepare_host(
    input: &str,
    output: &str,
    imports: &str,
    host_dir: &PathBuf,
    host_main: &PathBuf,
) -> io::Result<()> {
    let mut host_program = imports.to_string();
    let contents = fs::read_to_string(host_dir)?;
    host_program.push_str(&contents);

    // Insert input body
    let host_program = host_program.replace(utils::HOST_INPUT, input);
    // Insert output body
    let host_program = host_program.replace(utils::HOST_OUTPUT, output);

    // Extract Variable names from host and add them to the ExecutorEnv::builder()
    let values = utils::extract_regex(
        host_main,
        &format!("{}[(](.*?)[)]", regex::escape(utils::IO_WRITE)),
    )?;

    // Construct new Environment Builder
    let mut new_builder = RISC0_ENV_BUILDER.to_string();
    for value in values {
        new_builder.push_str(&format!(".write({}).unwrap()", value));
    }
    new_builder.push_str(".build().unwrap();");

    // Replace environment builder in host with new one
    let host_program = host_program.replace(
        "let env = ExecutorEnv::builder().build().unwrap();",
        &new_builder,
    );

    // replace zkRust::out()
    let host_program = host_program.replace(utils::IO_OUT, RISC0_IO_OUT);

    let mut file = fs::File::create(host_main)?;
    file.write_all(host_program.as_bytes())?;

    utils::remove_lines(host_main, "zk_rust_io::write(")?;
    Ok(())
}

/// Generates RISC0 proof and image ID
pub fn generate_risc0_proof(guest_path: &PathBuf, current_dir: &PathBuf) -> io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg(current_dir)
        .current_dir(guest_path)
        .status()
}
