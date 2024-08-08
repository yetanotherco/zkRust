pub fn main() {
    let (prove_fib, verify_fib) = guest::build_method();

    let (output, proof) = prove_method(50);
    let is_valid = verify_method(proof);

    println!("output: {}", output);
    println!("valid: {}", is_valid);
}
