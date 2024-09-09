//For acceleration we require the user defines the respective crate import since they are specific and needed to compile
use sha2::{Digest, Sha256};

fn main() {
    let data: String = "RISCV IS COOL!!!".to_string();
    let digest = Sha256::digest(&data.as_bytes());
    println!("{:?}", &digest.as_slice());
}
