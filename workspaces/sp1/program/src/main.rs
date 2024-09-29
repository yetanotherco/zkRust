#![no_main]
sp1_zkvm::entrypoint!(main);
use zk_rust_io;
pub fn main() {
let n: u32 = sp1_zkvm::io::read();
    sp1_zkvm::io::commit(&n);

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow.
        a = b;
        b = c;
    }

    sp1_zkvm::io::commit(&a);
    sp1_zkvm::io::commit(&b);
}