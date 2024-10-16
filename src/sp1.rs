use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
};

use crate::utils;

/// SP1 workspace directories
pub const SP1_SCRIPT_DIR: &str = "workspaces/sp1/script";
pub const SP1_SRC_DIR: &str = "workspaces/sp1/program";
pub const SP1_GUEST_MAIN: &str = "workspaces/sp1/program/src/main.rs";
pub const SP1_HOST_MAIN: &str = "workspaces/sp1/script/src/main.rs";
pub const SP1_BASE_GUEST_CARGO_TOML: &str = "workspaces/base_files/sp1/cargo_guest";
pub const SP1_BASE_HOST_CARGO_TOML: &str = "workspaces/base_files/sp1/cargo_host";
pub const SP1_BASE_HOST: &str = "workspaces/base_files/sp1/host";
pub const SP1_BASE_HOST_FILE: &str = "workspaces/base_files/sp1/host";
pub const SP1_GUEST_CARGO_TOML: &str = "workspaces/sp1/program/Cargo.toml";

// Proof data generation paths
pub const SP1_ELF_PATH: &str = "./proof_data/sp1/sp1.elf";
pub const SP1_PROOF_PATH: &str = "./proof_data/sp1/sp1.proof";
pub const SP1_PUB_INPUT_PATH: &str = "./proof_data/sp1/sp1.pub";

/// SP1 header added to programs for generating proofs of their execution
pub const SP1_GUEST_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// SP1 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const SP1_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha2\", branch = \"patch-sha2-v0.10.6\" }\nsha3 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha3\", branch = \"patch-sha3-v0.10.8\" }\ncrypto-bigint = { git = \"https://github.com/sp1-patches/RustCrypto-bigint\", branch = \"patch-v0.5.5\" }\ntiny-keccak = { git = \"https://github.com/sp1-patches/tiny-keccak\", branch = \"patch-v2.0.2\" }\ned25519-consensus = { git = \"https://github.com/sp1-patches/ed25519-consensus\", branch = \"patch-v2.1.0\" }\necdsa-core = { git = \"https://github.com/sp1-patches/signatures\", package = \"ecdsa\", branch = \"patch-ecdsa-v0.16.9\" }\n";

/// SP1 User I/O
// Host
pub const SP1_HOST_WRITE: &str = "stdin.write";
pub const SP1_HOST_READ: &str = "proof.public_values.read();";

// Guest
pub const SP1_IO_READ: &str = "sp1_zkvm::io::read();";
pub const SP1_IO_COMMIT: &str = "sp1_zkvm::io::commit";

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

    // replace zkRust::write
    let host_program = host_program.replace(utils::IO_WRITE, SP1_HOST_WRITE);
    // replace zkRust::out()
    let host_program = host_program.replace(utils::IO_OUT, SP1_HOST_READ);

    // Write to host
    let mut file = fs::File::create(host_main)?;
    file.write_all(host_program.as_bytes())?;
    Ok(())
}

/// Generates SP1 proof and ELF
pub fn generate_sp1_proof(script_dir: &PathBuf, current_dir: &PathBuf) -> io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg(current_dir)
        .current_dir(script_dir)
        .status()
}