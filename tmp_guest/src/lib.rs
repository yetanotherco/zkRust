#![cfg_attr(feature = "guest", no_std)]
#![no_main]
/* 
#![no_main]
sp1_zkvm::entrypoint!(main);
*/

#[jolt::provable]
fn fib() -> u128 {
    let n = 80;
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
    }

    b
}
