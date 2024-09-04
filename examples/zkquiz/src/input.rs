use utils::ask_question;
mod utils;

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
