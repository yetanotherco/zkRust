pub fn input() {
    let pattern = "a+".to_string();
    let target_string = "an era of truth, not trust".to_string();

    // Write in a simple regex pattern.
    zkRust::write(&pattern);
    zkRust::write(&target_string);
}