use regex::Regex;
use zkRust::io;

pub fn main() {
    // Read two inputs from the prover: a regex pattern and a target string.
    let pattern: String = io::read();
    let target_string: String = io::read();

    // Try to compile the regex pattern. If it fails, write `false` as output and return.
    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => {
            panic!("Invalid regex pattern");
        }
    };

    // Perform the regex search on the target string.
    let result = regex.is_match(&target_string);

    // Write the result (true or false) to the output.
    io::commit(&result);
}

pub fn output() {
    // Read the output.
    let res: bool = io::out();
    println!("res: {}", res);
}

pub fn input() {
    let pattern = "a+".to_string();
    let target_string = "an era of truth, not trust".to_string();

    // Write in a simple regex pattern.
    io::write(&pattern);
    io::write(&target_string);
}