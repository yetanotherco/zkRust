use zk_rust_io;

fn main() {
    let n: u32 = zk_rust_io::read();
    zk_rust_io::commit(&n);

    let is_even: bool = n % 2 == 0;

    zk_rust_io::commit(&is_even);
}

fn input() {
    let n = 16u32;
    zk_rust_io::write(&n);
}

fn output() {
    let (n, is_even): (u32, bool) = zk_rust_io::out();

    println!("is_even: {}", is_even);
}
