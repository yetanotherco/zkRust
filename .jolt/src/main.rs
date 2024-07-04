use std::{io::Write, fs};

pub fn main() {
    let (prove_fibonacci, verify_fibonacci) = guest::build_fibonacci();

    let (program, _) = guest::preprocess_fibonacci();

    // Write elf to file outside of tmp directory
    let elf = fs::read(program.elf.unwrap()).unwrap();
    let mut file = fs::File::create("../guest.elf").unwrap();
    file.write_all(&elf).unwrap();

    let (output, proof) = prove_fibonacci(50);
    proof.save_to_file("../guest.proof").unwrap();
    let is_valid = verify_fibonacci(proof);

    println!("output: {}", output);
    println!("valid: {}", is_valid);
}
