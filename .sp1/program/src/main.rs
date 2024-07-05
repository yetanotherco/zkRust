#![no_main]
sp1_zkvm::entrypoint!(main);
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub account_name: String,
    pub balance: u32,
}

fn main() {
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

    let v: Value = serde_json::from_str(&data_str).unwrap();
    println!("net_worth {:?}", v[key]);

    let new_account_state = &mut account_state;
    for tx in txs {
        if tx.from == new_account_state.account_name {
            new_account_state.balance -= tx.amount;
        }
        if tx.to == new_account_state.account_name {
            new_account_state.balance += tx.amount;
        }
    }
}