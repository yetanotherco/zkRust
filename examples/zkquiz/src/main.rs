use tiny_keccak::{Hasher, Sha3};
use utils::ask_question;
pub mod utils;
use zk_rust_io;

pub fn main() {
    let answers: String = zk_rust_io::read();
    let mut sha3 = Sha3::v256();
    let mut output = [0u8; 32];

    sha3.update(&answers.as_bytes());

    sha3.finalize(&mut output);

    if output
        != [
            232, 202, 155, 157, 82, 242, 126, 73, 75, 22, 197, 34, 41, 170, 163, 190, 22, 29, 192,
            5, 99, 134, 186, 25, 77, 128, 188, 154, 238, 70, 245, 229,
        ]
    {
        panic!("Answers do not match");
    }
}

pub fn input() {
    println!("Welcome to the quiz! Please answer the following questions to generate a proof for the program.");
    println!(
        "You will be asked 3 questions. Please answer with the corresponding letter (a, b or c)."
    );

    let mut user_answers = "".to_string();
    let question1 = "Who invented bitcoin";
    let answers1 = ["Sreeram Kannan", "Vitalik Buterin", "Satoshi Nakamoto"];
    user_answers.push(ask_question(question1, &answers1));

    let question2 = "What is the largest ocean on Earth?";
    let answers2 = ["Atlantic", "Indian", "Pacific"];
    user_answers.push(ask_question(question2, &answers2));

    let question3 = "What is the most aligned color";
    let answers3 = ["Green", "Red", "Blue"];
    user_answers.push(ask_question(question3, &answers3));

    zk_rust_io::write(&user_answers);
}

pub fn output() {}
