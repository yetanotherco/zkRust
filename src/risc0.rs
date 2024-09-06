use std::{
    fs,
    io::{self, Write},
    process::{Command, ExitStatus},
};

use crate::utils;

/// RISC0 workspace directories
pub const RISC0_WORKSPACE_DIR: &str = "./workspaces/risc0/";
pub const RISC0_GUEST_DIR: &str = "./workspaces/risc0/methods/guest/";
pub const RISC0_SRC_DIR: &str = "./workspaces/risc0/methods/guest/src";
pub const RISC0_GUEST_MAIN: &str = "./workspaces/risc0/methods/guest/src/main.rs";
pub const RISC0_HOST_MAIN: &str = "./workspaces/risc0/host/src/main.rs";
pub const RISC0_BASE_HOST_CARGO_TOML: &str = "./workspaces/base_files/risc0/cargo_host";
pub const RISC0_BASE_GUEST_CARGO_TOML: &str = "./workspaces/base_files/risc0/cargo_guest";
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

//TODO: should we use std or no_std header
/// RISC0 header added to programs for generating proofs of their execution
pub const RISC0_GUEST_PROGRAM_HEADER: &str = "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

/// RISC0 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const RISC0_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/risc0/RustCrypto-hashes\", tag = \"sha2-v0.10.6-risczero.0\" }\nk256 = { git = \"https://github.com/risc0/RustCrypto-elliptic-curves\", tag = \"k256/v0.13.1-risczero.1\"  }\ncrypto-bigint = { git = \"https://github.com/risc0/RustCrypto-crypto-bigint\", tag = \"v0.5.2-risczero.0\" }";

/// This function mainly adds this header to the guest in order for it to be proven by
/// risc0:
///
///    #![no_main]
///    risc0_zkvm::guest::entry!(main);
///
pub fn prepare_guest(imports: &str, main_func_code: &str) -> io::Result<()> {
    let mut guest_program = RISC0_GUEST_PROGRAM_HEADER.to_string();
    guest_program.push_str(imports);
    guest_program.push_str("pub fn main() {\n");
    guest_program.push_str(main_func_code);
    guest_program.push_str("\n}");
    // replace zkRust::read()
    let guest_program = guest_program.replace(utils::IO_READ, RISC0_IO_READ);

    // replace zkRust::commit()
    let guest_program = guest_program.replace(utils::IO_COMMIT, RISC0_IO_COMMIT);

    // Write to guest
    let mut file = fs::File::create(RISC0_GUEST_MAIN)?;
    file.write_all(guest_program.as_bytes())?;
    Ok(())
}

//TODO: Replace in string before writing to file.
//TODO: Still find and replace in file.
//TODO: in line this
pub fn prepare_host(input: &str, output: &str, imports: &str) -> io::Result<()> {
    utils::prepend_to_file(RISC0_HOST_MAIN, &imports)?;

    // Insert input body
    utils::insert(RISC0_HOST_MAIN, &input, utils::HOST_INPUT)?;
    // Insert output body
    utils::insert(RISC0_HOST_MAIN, &output, utils::HOST_OUTPUT)?;

    // Extract Variable names from host and add them to the ExecutorEnv::builder()
    let values = utils::extract_regex(
        RISC0_HOST_MAIN,
        &format!("{}[(](.*?)[)]", regex::escape(utils::IO_WRITE)),
    )?;

    // Construct new Environment Builder
    let mut new_builder = RISC0_ENV_BUILDER.to_string();
    for value in values {
        new_builder.push_str(&format!(".write({}).unwrap()", value));
    }
    new_builder.push_str(".build().unwrap();");

    // Replace environment builder in host with new one
    //TODO: can just write to marker in file no need for it to be specifically this.
    utils::replace(
        RISC0_HOST_MAIN,
        "let env = ExecutorEnv::builder().build().unwrap();",
        &new_builder,
    )?;

    //TODO: FRAGILE! -> switch to remove regex pattern
    //Delete lines that contain zkRust::write(; -> Delete things from within zk_rust_io -> );
    utils::remove_lines(RISC0_HOST_MAIN, "zk_rust_io::write(")?;

    // replace zkRust::out()
    utils::replace(RISC0_HOST_MAIN, utils::IO_OUT, RISC0_IO_OUT)?;
    Ok(())
}

/// Generates RISC0 proof and image ID
pub fn generate_risc0_proof() -> io::Result<ExitStatus> {
    let guest_path = fs::canonicalize(RISC0_WORKSPACE_DIR)?;

    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(guest_path)
        .status()
}
