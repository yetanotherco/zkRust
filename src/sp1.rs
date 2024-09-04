use std::{
    collections::HashSet,
    fs, io,
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
pub const SP1_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// SP1 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const SP1_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2-v0-10-8 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha2\", branch = \"patch-sha2-v0.10.8\" }\nsha3-v0-10-8 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha3\", branch = \"patch-sha3-v0.10.8\" }\ncrypto-bigint = { git = \"https://github.com/sp1-patches/RustCrypto-bigint\", branch = \"patch-v0.5.5\" }\ntiny-keccak = { git = \"https://github.com/sp1-patches/tiny-keccak\", branch = \"patch-v2.0.2\" }\ned25519-consensus = { git = \"https://github.com/sp1-patches/ed25519-consensus\", branch = \"patch-v2.1.0\" }\necdsa-core = { git = \"https://github.com/sp1-patches/signatures\", package = \"ecdsa\", branch = \"patch-ecdsa-v0.16.9\" }\n";

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
    //TODO: remove output & input functions after copying
    //TODO: only read file once

    let input_path = format!("{}/src/input.rs", guest_path);
    let input_imports = utils::extract_imports(&input_path)?;
    // let input = utils::extract_regex(&input_path, r"pub\sfn\sinput\(\)\s*\{([^}]*)\}")?.unwrap();
    println!();
    println!("input imports {:?}", input_imports);
    println!();

    // Extract input body
    let input =
        utils::extract_till_last_occurence(&input_path, r"pub fn input() ", "{", "}")?.unwrap();
    // let input = utils::extract_regex(&input_path, r"pub\sfn\sinput\(\)\s*\{([^}]*)\}")?.unwrap();
    println!();
    println!("input body {:?}", input);
    println!();

    // Extract output body
    let output_path = format!("{}/src/output.rs", guest_path);
    let output_imports = utils::extract_imports(&output_path)?;
    println!();
    println!("output imports {:?}", output_imports);
    println!();
    let output =
        utils::extract_till_last_occurence(&output_path, r"pub fn output() ", "{", "}")?.unwrap();
    //let output = utils::extract_regex(&output_path, r"pub\sfn\soutput\(\)\s*\{([^}]*)\}")?.unwrap();

    println!();
    println!("output body {:?}", output);
    println!();

    let mut import_set = HashSet::new();
    let mut imports = String::new();

    for import in input_imports.into_iter().chain(output_imports) {
        if import_set.insert(import.clone()) {
            imports.push_str(&import);
        }
    }

    println!();
    println!("imports {:?}", imports);
    println!();

    //prepend imports
    //prepend imports
    //utils::prepend_to_file(SP1_HOST_MAIN, &input_imports)?;
    //utils::prepend_to_file(SP1_HOST_MAIN, &output_imports)?;
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
