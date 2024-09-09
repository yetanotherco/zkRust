use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Account {
    pub account_name: String,
    pub balance: u32,
}
