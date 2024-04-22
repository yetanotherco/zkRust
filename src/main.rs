#![feature(string_remove_matches)]

use clap::{Args, Parser, Subcommand};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;
use std::process::Command;
use std::{io, fs};
use std::fmt::{Debug, Pointer};
use crate::Commands::ProveJolt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
struct Function {
    name: String,
    parameters: Vec<String>,
    return_type: String,
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
            println!("'Proving with jolt, program in: {}", args.guest_path);
            copy_dir_all(&args.guest_path, "./tmp_guest").unwrap();
            prepend_to_file("./tmp_guest/src/main.rs",
                            "#![cfg_attr(feature = \"guest\", no_std)]\n#![no_main]\n")
                .unwrap();
            process_file("./tmp_guest/src/main.rs", "./tmp_guest/src/lib.rs").unwrap();
            create_guest_files("./tmp_guest").unwrap();
            //  Host part
            let mut host_main : String = String::from(HOST_MAIN);
            if let Ok(mut file) = File::open("./tmp_guest/guest/src/lib.rs") {
                match parse_rust_file(&mut file) {
                    Ok(parsed_functions) => {
                        for (func_name, params) in parsed_functions {
                            host_main.push_str(&*HOST_PROVE_VERIFY.replace("{foo}", &func_name));
                        }
                    }
                    Err(e) => eprintln!("Error parsing Rust file: {}", e),
                }
            } else {
                eprintln!("Error opening Rust file.");
            }
            host_main.push_str("\n}");
            println!("{}", host_main);
            create_host_file("./tmp_guest/main.rs", host_main).unwrap();
            let output = Command::new("cargo")
                .current_dir("./tmp_guest") // Change to the correct directory
                .arg("run")
                .output()
                .expect("Error running cargo");
            println!("{:?}", output);
        }
    }
}
fn parse_function(line: &str) -> Option<Function> {
    let func_regex = r"fn\s+([a-zA-Z_]\w*)\s*\((.*?)\)\s*->\s*(.*?)\s*\{";
    let param_regex = r"([a-zA-Z_]\w*)\s*:\s*\w+\s*(?:,|\))";


    if let Some(captures) = regex::Regex::new(func_regex).unwrap().captures(line) {
        let name = captures[1].to_string();
        let param_list = &captures[2];
        let return_type = captures[3].to_string();

        let mut parameters = Vec::new();
        for param in regex::Regex::new(param_regex).unwrap().captures_iter(param_list) {
            let param_type = param[2].to_string();

            parameters.push(param_type);
        }
        Some(Function {
            name,
            parameters,
            return_type,
        })
    } else {
        None
    }
}

fn parse_rust_file(file: &mut File) -> Result<Vec<(String, Vec<(String, String)>)>, io::Error> {
    let mut parsed_functions = Vec::new();
    let mut inside_provable_func = false;
    let mut current_function: Option<(String, Vec<(String, String)>)> = None;

    for line in io::BufReader::new(file).lines() {
        let line = line?;

        // Check if the line contains #[jolt::provable]
        if line.contains("#[jolt::provable]") {
            inside_provable_func = true;
            continue; // Skip to the next line
        }

        // If we are inside a #[jolt::provable] function definition
        if inside_provable_func {
            // Check if the line contains a function definition
            if let Some(func) = line.strip_prefix("fn ") {
                let mut func_parts = func.splitn(2, '(');
                if let Some(name) = func_parts.next() {
                    let name = name.trim().to_string();
                    let mut param_types = Vec::new();
                    if let Some(param_list) = func_parts.next() {
                        for param in param_list.split(',') {
                            if let Some((param_name, param_type)) = param.split_once(":") {
                                let param_name = param_name.trim().to_string();
                                let param_type = param_type.trim().to_string();
                                param_types.push((param_name, param_type));
                            }
                        }
                    }
                    current_function = Some((name, param_types));
                }
            }
            inside_provable_func = false; // Reset the flag
        }

        if let Some((name, params)) = current_function.take() {
            parsed_functions.push((name, params));
        }
    }
    Ok(parsed_functions)
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
            //writeln!(output_file, "#[jolt::provable]")?;
        }
        writeln!(output_file, "{}", current_line)?;

        // Store the current line to use in the next iteration
        previous_line = current_line;
    }

    Ok(())
}
fn create_host_file(name: &str, host_main: String) -> Result<(), io::Error> {
    let mut cargo_file = File::create(format!("{}", name))?;
    cargo_file.write_all(host_main.as_ref())?;
    Ok(())
}

fn create_guest_files(name: &str) -> Result<(), io::Error> {
    let mut cargo_file = File::create(format!("{}/Cargo.toml", name))?;
    cargo_file.write_all(GUEST_CARGO.as_bytes())?;
    Ok(())
}

const HOST_MAIN: &str = r#"pub fn main() {
"#;
const HOST_PROVE_VERIFY: &str = r#"
    let (prove_{foo}, verify_{foo}) = guest::build_{foo}();

    let (output, proof) = prove_{foo}();
    let is_valid = verify_{foo}(proof);

    println!("output: {}", output);
    println!("valid: {}", is_valid);
"#;

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