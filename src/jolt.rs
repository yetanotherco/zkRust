use std::{fs, io::{self, Write}, process::Command};

use crate::utils;

pub const JOLT_PROOF_PATH: &str = "./jolt.proof";
pub const JOLT_ELF_PATH: &str = "./jolt.elf";
pub const JOLT_WORKSPACE_DIR: &str = "./workspaces/jolt";
pub const JOLT_SRC_DIR: &str = "./workspaces/jolt/guest/src";
pub const JOLT_GUEST_MAIN: &str = "./workspaces/jolt/guest/src/main.rs";

pub const JOLT_GUEST_CARGO_TOML: &str = "./workspaces/jolt/guest/Cargo.toml";
pub const JOLT_BASE_CARGO_TOML: &str = "./workspaces/base_files/jolt";

pub const JOLT_GUEST_PROGRAM_HEADER_STD: &str = "#![no_main]\n";
pub const JOLT_GUEST_PROC_MACRO: &str = "\n#[jolt::provable]\n";

//pub const JOLT_GUEST_DEPS: &str =
//   "\njolt = { package = \"jolt-sdk\", git = \"https://github.com/a16z/jolt\", features = [\"guest-std\"] }";

pub fn prepare_jolt_guest() -> io::Result<()> {
    /*
        #![no_main]
    */
    utils::prepend_to_file(JOLT_GUEST_MAIN, JOLT_GUEST_PROGRAM_HEADER_STD)?;

    // Find and replace function name
    let content = fs::read_to_string(JOLT_GUEST_MAIN).unwrap();

    let modified_content = content.replace("main()", "method()");

    /*
    #[jolt::provable]
    */
    let modified_content =
        utils::add_before_substring(&modified_content, "fn method()", JOLT_GUEST_PROC_MACRO);

    let mut file = fs::File::create(JOLT_GUEST_MAIN).unwrap();
    file.write_all(modified_content.as_bytes()).unwrap();
    Ok(())
}

pub fn generate_jolt_proof() -> io::Result<()> {
    let guest_path = fs::canonicalize(JOLT_WORKSPACE_DIR)?;

    Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(guest_path)
        .status()
        .unwrap();
    Ok(())
}