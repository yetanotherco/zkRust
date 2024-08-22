use zkRust::io;

pub fn output() {
    // Read the output.
    let res: bool = zk_rust_io::out();
    println!("res: {}", res);
}