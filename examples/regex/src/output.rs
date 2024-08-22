use zkRust::io;

pub fn output() {
    // Read the output.
    let res: bool = io::out();
    println!("res: {}", res);
}