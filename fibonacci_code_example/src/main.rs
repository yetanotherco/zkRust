/* 
#![no_main]
sp1_zkvm::entrypoint!(main);
*/
use std::hint::black_box;

fn fibonacci() -> u32 {
    let n = 800;
    let mut nums = vec![1, 1];
    for _ in 0..n {
        let mut c = nums[nums.len() - 1] + nums[nums.len() - 2];
        c %= 7919;
        nums.push(c);
    }
    nums[nums.len() - 1]
}

pub fn main() {
    let result = black_box(fibonacci());
    println!("result: {}", result);
}
