use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummaryResponse {
    pub total_balance: String,
    pub accounts: Vec<AccountInfo>,
    pub total_income: String,
    pub total_expenses: String,
    pub net_worth: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub balance: String,
    pub account_type: String, // "pocket" for now, can be extended later
}