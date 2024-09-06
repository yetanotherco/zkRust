use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    process::{Command, ExitStatus},
};

use crate::utils;

/// SP1 workspace directories
pub const SP1_SCRIPT_DIR: &str = "./workspaces/sp1/script";
pub const SP1_GUEST_DIR: &str = "./workspaces/sp1/program/";
pub const SP1_SRC_DIR: &str = "./workspaces/sp1/program/src";
pub const SP1_GUEST_MAIN: &str = "./workspaces/sp1/program/src/main.rs";
pub const SP1_HOST_MAIN: &str = "./workspaces/sp1/script/src/main.rs";
pub const SP1_BASE_GUEST_CARGO_TOML: &str = "./workspaces/base_files/sp1/cargo_guest";
pub const SP1_BASE_HOST_CARGO_TOML: &str = "./workspaces/base_files/sp1/cargo_host";
pub const SP1_BASE_HOST: &str = "./workspaces/base_files/sp1/host";
pub const SP1_GUEST_CARGO_TOML: &str = "./workspaces/sp1/program/Cargo.toml";

// Proof data generation paths
pub const SP1_ELF_PATH: &str = "./proof_data/sp1/sp1.elf";
pub const SP1_PROOF_PATH: &str = "./proof_data/sp1/sp1.proof";

/// SP1 header added to programs for generating proofs of their execution
pub const SP1_GUEST_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// SP1 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const SP1_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2-v0-10-8 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha2\", branch = \"patch-sha2-v0.10.8\" }\nsha3-v0-10-8 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha3\", branch = \"patch-sha3-v0.10.8\" }\ncrypto-bigint = { git = \"https://github.com/sp1-patches/RustCrypto-bigint\", branch = \"patch-v0.5.5\" }\ntiny-keccak = { git = \"https://github.com/sp1-patches/tiny-keccak\", branch = \"patch-v2.0.2\" }\ned25519-consensus = { git = \"https://github.com/sp1-patches/ed25519-consensus\", branch = \"patch-v2.1.0\" }\necdsa-core = { git = \"https://github.com/sp1-patches/signatures\", package = \"ecdsa\", branch = \"patch-ecdsa-v0.16.9\" }\n";

/// SP1 User I/O
// Host
pub const SP1_HOST_WRITE: &str = "stdin.write";
pub const SP1_HOST_READ: &str = "proof.public_values.read();";

// Guest
pub const SP1_IO_READ: &str = "sp1_zkvm::io::read();";
pub const SP1_IO_COMMIT: &str = "sp1_zkvm::io::commit";

//TODO: eliminate dedup w/ risc0
/// This function mainly adds this header to the guest in order for it to be proven by
/// sp1:
///
///    #![no_main]
///    sp1_zkvm::entrypoint!(main);
///
pub fn prepare_guest(imports: &str, main_func_code: &str) -> io::Result<()> {
    let mut guest_program = SP1_GUEST_PROGRAM_HEADER.to_string();
    guest_program.push_str(imports);
    guest_program.push_str("pub fn main() {\n");
    guest_program.push_str(main_func_code);
    guest_program.push_str("\n}");

    // Replace zkRust::read()
    let guest_program = guest_program.replace(utils::IO_READ, SP1_IO_READ);

    // Replace zkRust::commit()
    let guest_program = guest_program.replace(utils::IO_COMMIT, SP1_IO_COMMIT);

    // Write to guest
    let mut file = fs::File::create(SP1_GUEST_MAIN)?;
    file.write_all(guest_program.as_bytes())?;
    Ok(())
}

//TODO: Replace in string before writing to file.
//TODO: Still find and replace in file.
//TODO: in line this
pub fn prepare_host(input: &str, output: &str, imports: &str) -> io::Result<()> {
    utils::prepend_to_file(SP1_HOST_MAIN, &imports)?;

    // Insert input body
    utils::insert(SP1_HOST_MAIN, &input, utils::HOST_INPUT)?;
    // Insert output body
    utils::insert(SP1_HOST_MAIN, &output, utils::HOST_OUTPUT)?;

    //Remove imports of zk_rust_io

    // replace zkRust::write
    utils::replace(SP1_HOST_MAIN, utils::IO_WRITE, SP1_HOST_WRITE)?;
    // replace zkRust::out()
    utils::replace(SP1_HOST_MAIN, utils::IO_OUT, SP1_HOST_READ)?;
    Ok(())
}

/// Generates SP1 proof and ELF
pub fn generate_sp1_proof() -> io::Result<ExitStatus> {
    let guest_path = fs::canonicalize(SP1_SCRIPT_DIR)?;
    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(guest_path)
        .status()
}
