use zkRust::io;

pub fn input() {
    let pattern = "a+".to_string();
    let target_string = "an era of truth, not trust".to_string();

    // Write in a simple regex pattern.
    io::write(&pattern);
    io::write(&target_string);
}