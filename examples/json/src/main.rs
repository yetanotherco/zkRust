use lib::{Account, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn main() {
    let data_str: String = zk_rust_io::read();
    let key: String = zk_rust_io::read();

    // read custom struct example inputs.
    let mut old_account_state: Account = zk_rust_io::read();
    let txs: Vec<Transaction> = zk_rust_io::read();

    let v: Value = serde_json::from_str(&data_str).unwrap();
    println!("net_worth {:?}", v[key]);

    let new_account_state = &mut old_account_state;
    for tx in txs {
        if tx.from == new_account_state.account_name {
            new_account_state.balance -= tx.amount;
        }
        if tx.to == new_account_state.account_name {
            new_account_state.balance += tx.amount;
        }
    }
}

fn input() {
    let data_str = r#"
    {
        "name": "Jane Doe",
        "age": "25",
        "net_worth" : "$1000000"
    }"#
    .to_string();
    let key = "net_worth".to_string();

    let mut account_state = Account {
        account_name: "Bill".to_string(),
        balance: 200,
    };
    let txs = vec![
        Transaction {
            from: "Bill".to_string(),
            to: "Tom".to_string(),
            amount: 50,
        },
        Transaction {
            from: "Bill".to_string(),
            to: "Tom".to_string(),
            amount: 100,
        },
    ];

    zk_rust_io::write(&data_str);
    zk_rust_io::write(&key);
    zk_rust_io::write(&account_state);
    zk_rust_io::write(&txs);
}

fn output() {
    let account_state: Account = zk_rust_io::read();
    println!(
        "Final account state: {}",
        serde_json::to_string(&account_state).unwrap()
    );
}
