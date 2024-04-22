use clap::{Args, Parser, Subcommand};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;
use std::process::Command;
use std::{io, fs};
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    ProveSp1(ProofArgs),
    ProveJolt(ProofArgs),
}

#[derive(Args, Debug)]
struct ProofArgs {
    guest_path: String,
    output_proof_path: String,
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
        modified_string.push_str("\n");
        modified_string.push_str(text_to_add);
        modified_string.push_str(&original_string[index + substring.len()..]);
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
    Ok(())
}

fn add_dependency_to_toml(path: &str, dep_string: &str) -> io::Result<()> {
    // Open the file in read write mode to read its existing content
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;

    // Read the existing content of the file
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    println!("Toml Content: {}",content);


    content = add_text_after_substring(&content, "[dependencies]", dep_string);

    println!("Toml Content after add: {}",content);

    file.set_len(0)?;
    file.seek(io::SeekFrom::Start(0))?;
    file.write_all(content.as_bytes())?;
    file.set_len(content.len() as u64)?;

    Ok(())
}
const SP1_GUEST_DEPS_STRING: &str = "sp1-core = { git = \"https://github.com/succinctlabs/sp1.git\", tag = \"v0.0.2\" }\n";
const JOLT_GUEST_DEPS_STRING: &str = "jolt = 0.1.0\n";
const JOLT_ELF_PATH: &str = "tmp_guest/elf/riscv32i-jolt-zkvm-elf";
const SP1_ELF_PATH: &str = ".tmp_guest/elf/riscv32im-succinct-zkvm-elf";
#[cfg_attr(not(feature = "stable"), feature(proc_macro_tracked_env))]
fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::ProveSp1(args) => {
            println!("'Proving with sp1 program in: {}", args.guest_path);
            // We create a temporary directory to edit the main and leave it as SP1 needs it
            copy_dir_all(&args.guest_path, "./.tmp_guest/").unwrap();
            prepend_to_file("./.tmp_guest/src/main.rs",
                            "#![no_main]\nsp1_zkvm::entrypoint!(main);\n").unwrap();

            /* 
            sp1-core = { git = "https://github.com/succinctlabs/sp1.git" }
             */
            add_dependency_to_toml("./.tmp_guest/Cargo.toml", SP1_GUEST_DEPS_STRING).unwrap();

            /* 
            cd program
            cargo prove build

            fs::canonicalize("../a/../foo.txt")?;
            */
            let guest_path = fs::canonicalize("./.tmp_guest/").unwrap();
            Command::new("cargo")
                .arg("prove")
                .arg("build")
                .current_dir(guest_path)
                .output()
                .expect("Prove build failed");

            let elf_canonical_path = fs::canonicalize("./.tmp_guest/elf/riscv32im-succinct-zkvm-elf").unwrap();

            println!("Elf: {:?}", elf_canonical_path);

            let mut f = File::open(&elf_canonical_path).expect("no file found");
            let metadata = fs::metadata(&elf_canonical_path).expect("unable to read metadata");
            let mut elf_data = vec![0; metadata.len() as usize];
            f.read(&mut elf_data).expect("buffer overflow");

            let mut stdin = SP1Stdin::new();
            let n = 500u32;
            stdin.write(&n);

            println!("Elf data: {:?}", elf_data);
            let mut proof = SP1Prover::prove(&elf_data, stdin).expect("proving failed");

            // Read output.
            let a = proof.stdout.read::<u32>();
            let b = proof.stdout.read::<u32>();
            println!("a: {}", a);
            println!("b: {}", b);

            // Verify proof.
            SP1Verifier::verify(&elf_data, &proof).expect("verification failed");

            // Save proof.
            proof
                .save("proof-with-io.json")
                .expect("saving proof failed");

            println!("succesfully generated and verified proof for the program!")
        }
        Commands::ProveJolt(args) => {
            println!("'Proving with jolt program in: {}", args.guest_path);
            copy_dir_all(&args.guest_path, "./tmp_guest/guest").unwrap();
            prepend_to_file("./tmp_guest/guest/src/main.rs",
                            "#![cfg_attr(feature = \"guest\", no_std)]\n#![no_main]\n")
                .unwrap();
            process_file("./tmp_guest/guest/src/main.rs", "./tmp_guest/guest/src/lib.rs").unwrap();
            create_guest_files("./tmp_guest").unwrap();
            //  Host part
            let guest_path = fs::canonicalize("./tmp_guest/").unwrap();
            // Build and run the Jolt program
            let file_content = fs::read_to_string("./tmp_guest/guest/src/lib.rs")
                .expect("Unable to read file");

            // Initialize variables to store function name and parameter
            let mut func_name = String::new();
            let mut param_list = String::new();

            // Flag to indicate if we are inside a #[jolt::provable] function definition
            let mut inside_provable_func = false;

            // Parse the content to extract function names and parameters
            for line in file_content.lines() {
                // Check if the line contains #[jolt::provable]
                if line.contains("#[jolt::provable]") {
                    inside_provable_func = true;
                    continue; // Skip to the next line
                }

                // If we are inside a #[jolt::provable] function definition
                if inside_provable_func {
                    // Check if the line contains a function definition
                    if let Some(func) = line.strip_prefix("fn ") {
                        if let Some(name) = func.split('(').next() {
                            func_name = name.trim().to_string();
                        }
                        // Capture the parameter list
                        if let Some(param) = line.strip_prefix("fn").and_then(|s| s.split('(').nth(1)) {
                            param_list = param.trim().to_string();
                        }
                    }
                    inside_provable_func = false; // Reset the flag
                    continue; // Skip to the next line
                }

                if let Some(func) = line.strip_prefix("#[jolt::provable] fn ") {
                    if let Some(name) = func.split('(').next() {
                        func_name = name.trim().to_string();
                    }
                }
            }
            let host_main = HOST_MAIN
                .replace("{foo}", &func_name)
                .replace("{param}", &param_list);
            // Execute the generated main function
            println!("{}", host_main);
        }
    }
}



fn process_file(input_filename: &str, output_filename: &str) -> Result<(), std::io::Error> {
    // Determine the output filename
    let mut output_f = String::from(output_filename);
    if output_filename.is_empty() {
        output_f = String::from(input_filename);
    }

    // Open the input file
    let input_file = File::open(input_filename)?;

    // Create or open the output file
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_f)?;

    // Create a reader for the input file
    let input_reader = BufReader::new(input_file);

    // Iterate through each line in the input file
    let mut previous_line = String::new();
    for line in input_reader.lines() {
        let current_line = line?;

        // Write the line to the output file

        // Check if the current line starts with "fn"
        if current_line.trim().starts_with("fn") {
            // If it does, write something to the previous line
            writeln!(output_file, "#[jolt::provable]")?;
        }
        writeln!(output_file, "{}", current_line)?;

        // Store the current line to use in the next iteration
        previous_line = current_line;
    }

    Ok(())
}


fn create_guest_files(name: &str) -> Result<(), io::Error> {
    let mut cargo_file = File::create(format!("{}/guest/Cargo.toml", name))?;
    cargo_file.write_all(GUEST_CARGO.as_bytes())?;
    Ok(())
}
const RUST_TOOLCHAIN: &str = r#"[toolchain]
channel = "nightly-2023-09-22"
targets = ["riscv32i-jolt-zkvm-elf"]
"#;

const HOST_CARGO_TEMPLATE: &str = r#"[package]
name = "{NAME}"
version = "0.1.0"
edition = "2021"

[workspace]
members = ["guest"]

[profile.release]
debug = 1
codegen-units = 1
lto = "fat"

[dependencies]
jolt = { package = "jolt-sdk", git = "https://github.com/a16z/jolt", features = ["std"] }
guest = { path = "./guest" }

[patch.crates-io]
ark-ff = { git = "https://github.com/a16z/arkworks-algebra", branch = "optimize/field-from-u64" }
ark-ec = { git = "https://github.com/a16z/arkworks-algebra", branch = "optimize/field-from-u64" }
ark-serialize = { git = "https://github.com/a16z/arkworks-algebra", branch = "optimize/field-from-u64" }
"#;

const HOST_MAIN: &str = r#"pub fn main() {
    let (prove_{foo}, verify_{foo}) = guest::build_{foo}();

    let param = {param};
    let (output, proof) = prove_{foo}(param);
    let is_valid = verify_{foo}(proof);

    println!("output: {}", output);
    println!("valid: {}", is_valid);
}
"#;

const GITIGNORE: &str = "target";

const GUEST_CARGO: &str = r#"[package]
name = "guest"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "guest"
path = "./src/lib.rs"

[features]
guest = []

[dependencies]
jolt = { package = "jolt-sdk", git = "https://github.com/a16z/jolt" }
"#;