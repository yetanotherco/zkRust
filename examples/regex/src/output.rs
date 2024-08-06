pub fn output() {
    // Read the output.
    let res: bool = zkRust::out();
    println!("res: {}", res);
}