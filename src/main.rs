#![feature(string_remove_matches)]

use clap::{Args, Parser, Subcommand};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::SeekFrom;
use regex::Regex;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ProveSp1(ProofArgs),
    ProveJolt(ProofArgs),
}

#[derive(Args, Debug)]
struct ProofArgs {
    guest_path: String,
    output_proof_path: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            handle_sp1_proof(&args.guest_path);
        },
        Commands::ProveJolt(args) => {
            handle_jolt_proof(&args.guest_path);
        }
    }
}

fn handle_sp1_proof(guest_path: &str) {
    let src_path = Path::new(guest_path);
    let dest_path = Path::new("./tmp_guest_sp1");

    if !src_path.exists() {
        eprintln!("Error: Source path does not exist: {}", src_path.display());
        return;
    }

    if let Err(e) = copy_dir_all(src_path, dest_path) {
        eprintln!("Failed to copy files: {}", e);
        return;
    }

    // Adjustments specific to SP1
    let main_rs_path = dest_path.join("src/main.rs");
    if let Err(e) = prepend_to_file(&main_rs_path, "#![no_main]\nsp1_zkvm::entrypoint!(main);\n") {
        eprintln!("Failed to prepend to file {}: {}", main_rs_path.display(), e);
        return;
    }

    // Assuming SP1 requires a special build process
    if let Err(e) = execute_cargo_build(dest_path) {
        eprintln!("Cargo build failed: {}", e);
    }
}

fn handle_jolt_proof(guest_path: &str) {
    let src_path = Path::new(guest_path);
    let dest_path = Path::new("./tmp_guest_jolt");

    if !src_path.exists() {
        eprintln!("Error: Source path does not exist: {}", src_path.display());
        return;
    }

    fn adjust_main_function(dest_path: &Path) -> io::Result<()> {
        // Implementation of the adjust_main_function function goes here
        Ok(())
    }

    if let Err(e) = copy_dir_all(src_path, dest_path) {
        eprintln!("Failed to copy files: {}", e);
        return;
    }

    // Adjustments specific to Jolt
    if let Err(e) = adjust_main_function(dest_path) {
        eprintln!("Failed to adjust main function: {}", e);
        return;
    }

    if let Err(e) = create_guest_files(dest_path) {
        eprintln!("Failed to create guest files: {}", e);
        return;
    }

    if let Err(e) = execute_cargo_run(dest_path) {
        eprintln!("Cargo run failed: {}", e);
    }
}

// Copy directory content from source to destination
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Source path is not a directory"))
    }
}

fn prepend_to_file(file_path: &Path, new_content: &str) -> io::Result<()> {
    let mut file = File::options().read(true).write(true).open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    if !content.contains("fn main()") {
        file.seek(SeekFrom::Start(0))?; // Move to the start of the file
        file.write_all(new_content.as_bytes())?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

fn create_guest_files(base_path: &Path) -> io::Result<()> {
    let cargo_toml_path = base_path.join("Cargo.toml");
    let cargo_contents = r#"[package]
name = "guest"
version = "0.1.0"
edition = "2021"

[dependencies]
jolt = { package = "jolt-sdk", git = "https://github.com/a16z/jolt" }
"#;
    fs::write(cargo_toml_path, cargo_contents)
}

fn execute_cargo_run(base_path: &Path) -> io::Result<()> {
    let output = Command::new("cargo")
        .current_dir(base_path)
        .arg("run")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Cargo run succeeded: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Cargo run failed with error: {}", stderr);
    }

    Ok(())
}

fn execute_cargo_build(base_path: &Path) -> io::Result<()> {
    let output = Command::new("cargo")
        .current_dir(base_path)
        .arg("build")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Cargo build succeeded: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Cargo build failed with error: {}", stderr);
    }

    Ok(())
}
