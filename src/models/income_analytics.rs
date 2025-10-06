use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeSummaryResponse {
    pub total_income: Decimal,
    pub total_transactions: i64,
    pub average_per_day: Decimal,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeCategorySummaryItem {
    pub category: Option<String>,
    pub total_amount: Decimal,
    pub transaction_count: i64,
    pub percentage: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeCategorySummaryResponse {
    pub categories: Vec<IncomeCategorySummaryItem>,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeTrendItem {
    pub period: String,
    pub total_amount: Decimal,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeTrendResponse {
    pub trends: Vec<IncomeTrendItem>,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentIncomeTransactionItem {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub account_id: Option<uuid::Uuid>,
    pub description: String,
    pub amount: Decimal,
    pub category: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentIncomeTransactionsResponse {
    pub data: Vec<RecentIncomeTransactionItem>,
    pub limit: i32,
    pub count: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IncomeDateRangeQuery {
    #[validate(length(min = 10, max = 10, message = "Date must be in YYYY-MM-DD format"))]
    pub from_date: String,
    #[validate(length(min = 10, max = 10, message = "Date must be in YYYY-MM-DD format"))]
    pub to_date: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IncomeRecentTransactionsQuery {
    #[validate(range(min = 1, max = 50, message = "Limit must be between 1 and 50"))]
    pub limit: Option<i32>,
}

impl Default for IncomeRecentTransactionsQuery {
    fn default() -> Self {
        Self { limit: Some(10) }
    }
}