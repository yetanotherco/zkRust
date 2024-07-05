use aligned_sdk::types::ProvingSystemId;
use clap::{Args, Parser, Subcommand};
use std::fs::OpenOptions;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};
use zkRust::submit_proof_to_aligned;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Add flags that specify using proof system specific patches such as rand, sha, etc.

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    ProveSp1(ProofArgs),
    ProveRisc0(ProofArgs),
    ProveJolt(ProofArgs)
}

#[derive(Args, Debug)]
struct ProofArgs {
    guest_path: String,
    output_proof_path: String,
    #[clap(long)]
    submit_to_aligned_with_keystore: Option<PathBuf>,
    #[clap(long)]
    std: bool,
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn add_text_after_substring(original_string: &str, substring: &str, text_to_add: &str) -> String {
    if let Some(index) = original_string.find(substring) {
        let mut modified_string = String::with_capacity(original_string.len() + text_to_add.len());
        modified_string.push_str(&original_string[..index + substring.len()]);
        modified_string.push_str(text_to_add);
        modified_string.push_str(&original_string[index + substring.len()..]);
        modified_string
    } else {
        original_string.to_string()
    }
}

fn add_before_substring(original_string: &str, substring: &str, text_to_add: &str) -> String {
    if let Some(index) = original_string.find(substring) {
        let mut modified_string = String::with_capacity(original_string.len() + text_to_add.len());
        modified_string.push_str(&original_string[..index]);
        modified_string.push_str(text_to_add);
        modified_string.push_str(&original_string[index..]);
        modified_string
    } else {
        original_string.to_string()
    }
}

fn prepend_to_file(file_path: &str, text_to_prepend: &str) -> io::Result<()> {
    // Open the file in read mode to read its existing content
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Read the existing content of the file
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Move the file cursor to the beginning of the file
    file.seek(io::SeekFrom::Start(0))?;

    // Write the text to prepend followed by the existing content back to the file
    file.write_all(text_to_prepend.as_bytes())?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}

fn add_dependency_to_toml(path: &str, dep_string: &str) -> io::Result<()> {
    // Open the file in read write mode to read its existing content
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    // Read the existing content of the file
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    content = add_text_after_substring(&content, "[dependencies]", dep_string);

    file.set_len(0)?;
    file.seek(io::SeekFrom::Start(0))?;
    file.write_all(content.as_bytes())?;
    file.set_len(content.len() as u64)?;
    file.flush()?;

    Ok(())
}

fn copy_dependencies(toml_path: &str, guest_toml_path: &str) {
    let mut toml = std::fs::File::open(toml_path).unwrap();
    let mut content = String::new();
    toml.read_to_string(&mut content).unwrap();

    if let Some(start_index) = content.find("[dependencies]") {
        // Get all text after the search string
        let dependencies = &content[start_index + "[dependencies]".len()..];
        // Open the output file in append mode
        let mut guest_toml = OpenOptions::new()
            .create(true)
            .append(true)
            .open(guest_toml_path)
            .unwrap();

        // Write the text after the search string to the output file
        guest_toml.write_all(dependencies.as_bytes()).unwrap();
    } else {
        println!("Failed to copy dependencies in Guest Toml file, plese check");
    }
}

fn remove_dependencies(guest_toml_path: &str, guest_dependency_path: &str) {
    let mut content = fs::read_to_string(guest_toml_path).unwrap();
    if let Some(pos) = content.find("[dependencies]") {
        // Get all text after the search string
        content.truncate(pos + "[dependencies]".len());

        // Write the text after the search string to the output file
        let mut toml = fs::File::create(guest_toml_path).unwrap();
        toml.write_all(content.as_bytes()).unwrap();
        // Append cargo toml dependency
        add_dependency_to_toml(guest_toml_path, guest_dependency_path).unwrap();
    } else {
        println!("Failed to clear dependencies in SP1 Toml file, plaese check");
    }
}

// SP1 File additions
const SP1_SCRIPT_DIR: &str = "./.sp1/script";

const SP1_GUEST_DIR: &str = "./.sp1/program/src";

const SP1_GUEST_MAIN: &str = "./.sp1/program/src/main.rs";

const SP1_GUEST_CARGO_TOML: &str = "./.sp1/program/Cargo.toml";

const SP1_GUEST_DEPS: &str = "\nsp1-zkvm = { git = \"https://github.com/succinctlabs/sp1.git\" }\n";

const SP1_ELF_PATH: &str = "./sp1.elf";

const SP1_PROOF_PATH: &str = "./sp1.proof";

const SP1_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

// Jolt File Paths and Additions
const JOLT_PROOF_PATH: &str = "./jolt.proof";

const JOLT_ELF_PATH: &str = "./jolt.elf";

const JOLT_DIR: &str = "./.jolt/";

const JOLT_GUEST_DIR: &str = "./.jolt/guest/src";

const JOLT_GUEST_MAIN: &str = "./.jolt/guest/src/lib.rs";

const JOLT_GUEST_CARGO_TOML: &str = "./.jolt/guest/Cargo.toml";

const JOLT_GUEST_PROGRAM_HEADER_STD: &str = "#![no_main]\n";

const JOLT_GUEST_PROC_MACRO: &str = "\n#[jolt::provable]\n";

const JOLT_GUEST_DEPS: &str =
    "\njolt = { package = \"jolt-sdk\", git = \"https://github.com/a16z/jolt\", features = [\"guest-std\"] }";

// Risc0 File Paths and Additions
const RISC0_PROOF_PATH: &str = "./risc_zero.proof";

const RISC0_IMAGE_PATH: &str = "./risc_zero_image_id.bin";

const RISC0_DIR: &str = "./.risc_zero/";

const RISC0_GUEST_DIR: &str = "./.risc_zero/methods/guest/src";

const RISC0_GUEST_MAIN: &str = "./.risc_zero/methods/guest/src/main.rs";

const RISC0_GUEST_CARGO_TOML: &str = "./.risc_zero/methods/guest/Cargo.toml";

const RISC0_GUEST_PROGRAM_HEADER_STD: &str = "#![no_main]\n\nrisc0_zkvm::guest::entry!(main);\n";

const RISC0_GUEST_DEPS: &str =
    "\nrisc0-zkvm = { git = \"https://github.com/risc0/risc0\", features = [\"std\", \"getrandom\"] }";

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            println!("'Proving with SP1 program in: {}", args.guest_path);
            fs::remove_dir_all(SP1_GUEST_DIR).unwrap();
            fs::create_dir(SP1_GUEST_DIR).unwrap();

            // Copy the source main to the destination directory
            let guest_path = format!("{}/src/", args.guest_path);
            copy_dir_all(guest_path, SP1_GUEST_DIR).unwrap();

            // Copy dependencies to from guest toml to risc0 project template
            let toml_path = format!("{}/Cargo.toml", args.guest_path);
            copy_dependencies(&toml_path, SP1_GUEST_CARGO_TOML);

            /*
                #![no_main]
                sp1_zkvm::entrypoint!(main);
            */
            prepend_to_file(SP1_GUEST_MAIN, SP1_PROGRAM_HEADER).unwrap();

            let guest_path = fs::canonicalize(SP1_SCRIPT_DIR).unwrap();
            //TODO: propogate errors from this command to stdout/stderr
            Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(guest_path)
                .status()
                .expect("Prove build failed");
            println!("Proof and ELF generated!");

            // Clear toml of dependencies
            remove_dependencies(SP1_GUEST_CARGO_TOML, SP1_GUEST_DEPS);
            println!("Proof Generated");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    SP1_PROOF_PATH,
                    SP1_ELF_PATH,
                    //TODO: Change when Jolt PR is upstreamed
                    ProvingSystemId::SP1,
                )
                .unwrap();
                println!("Proof submitted and verified on aligned");
            }
        }
        Commands::ProveJolt(args) => {
            println!("'Proving with Jolt program in: {}", args.guest_path);
            // Clear contents of src directory
            fs::remove_dir_all(JOLT_GUEST_DIR).unwrap();
            fs::create_dir(JOLT_GUEST_DIR).unwrap();

            // Copy the source main to the destination directory
            let guest_path = format!("{}/src/", args.guest_path);
            copy_dir_all(guest_path, JOLT_GUEST_DIR).unwrap();
            fs::rename("./.jolt/guest/src/main.rs", JOLT_GUEST_MAIN).unwrap();

            // Copy dependencies to from guest toml to risc0 project template
            let toml_path = format!("{}/Cargo.toml", args.guest_path);
            copy_dependencies(&toml_path, JOLT_GUEST_CARGO_TOML);

            /*
               #![no_main]
            */
            prepend_to_file(JOLT_GUEST_MAIN, JOLT_GUEST_PROGRAM_HEADER_STD).unwrap();

            // Find and replace function name
            let content = fs::read_to_string(JOLT_GUEST_MAIN).unwrap();

            let modified_content = content.replace("main()", "method()");

            /*
               #[jolt::provable]
            */
            let modified_content =
                add_before_substring(&modified_content, "fn method()", JOLT_GUEST_PROC_MACRO);

            let mut file = fs::File::create(JOLT_GUEST_MAIN).unwrap();
            file.write_all(modified_content.as_bytes()).unwrap();

            let guest_path = fs::canonicalize(JOLT_DIR).unwrap();
            //TODO: propogate errors from this command to stdout/stderr
            Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(guest_path)
                .status()
                .expect("Prove build failed");
            println!("Proof and Proof Image generated!");

            // Clear toml of dependencies
            remove_dependencies(JOLT_GUEST_CARGO_TOML, JOLT_GUEST_DEPS);
            println!("Proof Generated");

            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    JOLT_PROOF_PATH,
                    JOLT_ELF_PATH,
                    //TODO: Change when Jolt PR is upstreamed
                    ProvingSystemId::SP1,
                )
                .unwrap();
                println!("Proof submitted and verified on aligned");
            }
        }
        Commands::ProveRisc0(args) => {
            println!("Proving with Risc0 program in: {}", args.guest_path);
            // Clear contents of src directory
            fs::remove_dir_all(RISC0_GUEST_DIR).unwrap();
            fs::create_dir(RISC0_GUEST_DIR).unwrap();

            // Copy the source main to the destination directory
            let guest_path = format!("{}/src/", args.guest_path);
            copy_dir_all(guest_path, RISC0_GUEST_DIR).unwrap();

            // Copy dependencies to from guest toml to risc0 project template
            let toml_path = format!("{}/Cargo.toml", args.guest_path);
            copy_dependencies(&toml_path, RISC0_GUEST_CARGO_TOML);

            /*
               #![no_main]
               risc0_zkvm::guest::entry!(main);
            */
            prepend_to_file(RISC0_GUEST_MAIN, RISC0_GUEST_PROGRAM_HEADER_STD).unwrap();

            let guest_path = fs::canonicalize(RISC0_DIR).unwrap();
            //TODO: propogate errors from this command to stdout/stderr
            Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(guest_path)
                .status()
                .expect("Prove build failed");
            println!("Proof and Proof Image generated!");

            // Clear toml of dependencies
            remove_dependencies(RISC0_GUEST_CARGO_TOML, RISC0_GUEST_DEPS);
            // Submit to aligned
            if let Some(keystore_path) = args.submit_to_aligned_with_keystore.clone() {
                submit_proof_to_aligned(
                    keystore_path,
                    RISC0_PROOF_PATH,
                    RISC0_IMAGE_PATH,
                    ProvingSystemId::Risc0,
                )
                .unwrap();
                println!("Proof submitted and verified on aligned");
            }
        }
    }
}
