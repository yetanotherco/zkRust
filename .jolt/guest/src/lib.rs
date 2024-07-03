#![no_main]

#[jolt::provable]
fn fibonacci(n: u32) -> u32 {
    let mut nums = vec![1, 1];
    for _ in 0..n {
        let mut c = nums[nums.len() - 1] + nums[nums.len() - 2];
        c %= 7919;
        nums.push(c);
    }
    nums[nums.len() - 1]
}

