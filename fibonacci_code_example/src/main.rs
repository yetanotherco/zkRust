/* 
#![no_main]
sp1_zkvm::entrypoint!(main);
*/

<<<<<<< HEAD
fn fibonacci() -> u32 {
    let n = 2;
    let mut nums = vec![1, 1];
    for _ in 0..n {
        let mut c = nums[nums.len() - 1] + nums[nums.len() - 2];
        c %= 7919;
        nums.push(c);
=======
fn fib() -> u128 {
    let n = 80;
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    let mut sum: u128;
    for _ in 1..n {
        sum = a + b;
        a = b;
        b = sum;
>>>>>>> 598505f (Update repo)
    }

    b
}
