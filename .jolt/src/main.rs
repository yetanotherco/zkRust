use std::{io::Write, fs};

pub fn main() {
    let (prove_method, verify_method) = guest::build_method();

    let (program, _) = guest::preprocess_method();

    // Write elf to file outside of tmp directory
    let elf = fs::read(program.elf.unwrap()).unwrap();
    let mut file = fs::File::create("../jolt.elf").unwrap();
    file.write_all(&elf).unwrap();

    let (output, proof) = prove_method();
    proof.save_to_file("../jolt.proof").unwrap();
    let is_valid = verify_method(proof);

    println!("output: {:?}", output);
    println!("valid: {:?}", is_valid);
}
