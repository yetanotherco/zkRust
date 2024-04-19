use clap::{Args, Parser, Subcommand};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::process::Command;
use std::{io, fs, thread};
use core::time::Duration;

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
const SP1_GUEST_DEPS_STRING: &str = "sp1-zkvm = { git = \"https://github.com/succinctlabs/sp1.git\", tag = \"v0.0.2\" }\n";

// sp1-zkvm = { git = "https://github.com/succinctlabs/sp1.git", tag = "v0.0.2" }
const SP1_ELF_PATH: &str = ".tmp_guest/elf/riscv32im-succinct-zkvm-elf";

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

            println!("Compiling program with RISC V ...");
            let guest_path = fs::canonicalize("./.tmp_guest/").unwrap();

            println!("Guest path: {:?}", guest_path);

            Command::new("cargo")
                .arg("prove")
                .arg("build")
                .current_dir(guest_path)
                .output()
                .expect("Prove build failed");

            println!("Compilation finished");

            let elf_canonical_path = fs::canonicalize("./.tmp_guest/elf/riscv32im-succinct-zkvm-elf").unwrap();


            let mut f = File::open(&elf_canonical_path).expect("no file found");
            let metadata = fs::metadata(&elf_canonical_path).expect("unable to read metadata");
            let mut elf_data = vec![0; metadata.len() as usize];
            f.read(&mut elf_data).expect("buffer overflow");

            let mut stdin = SP1Stdin::new();
            let n = 10u32;
            stdin.write(&n);

            println!("Proving ...");

            // println!("Elf data: {:?}", elf_data);
            let mut proof = SP1Prover::prove(&elf_data, stdin).expect("proving failed");

            println!("Proving finished");


            // Read output.
            /* 
            let a = proof.stdout.read::<u32>();
            let b = proof.stdout.read::<u32>();
            println!("a: {}", a);
            println!("b: {}", b);
            */

            println!("Testing verification");

            // Verify proof.
            SP1Verifier::verify(&elf_data, &proof).expect("verification failed");

            println!("Verified!");


            // Save proof.
            proof
                .save("proof-with-io.json")
                .expect("saving proof failed");

            println!("succesfully generated and verified proof for the program!") 
        }
        Commands::ProveJolt(_) => {
            println!("Proving with jolt is not supported yet");
        }
    }
}
