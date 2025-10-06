use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseSummaryResponse {
    pub total_expenses: Decimal,
    pub total_transactions: i64,
    pub average_per_day: Decimal,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySummaryItem {
    pub category: Option<String>,
    pub total_amount: Decimal,
    pub transaction_count: i64,
    pub percentage: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySummaryResponse {
    pub categories: Vec<CategorySummaryItem>,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendItem {
    pub period: String,
    pub total_amount: Decimal,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendResponse {
    pub trends: Vec<TrendItem>,
    pub from_date: String,
    pub to_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentTransactionItem {
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
pub struct RecentTransactionsResponse {
    pub data: Vec<RecentTransactionItem>,
    pub limit: i32,
    pub count: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DateRangeQuery {
    #[validate(length(min = 10, max = 10, message = "Date must be in YYYY-MM-DD format"))]
    pub from_date: String,
    #[validate(length(min = 10, max = 10, message = "Date must be in YYYY-MM-DD format"))]
    pub to_date: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RecentTransactionsQuery {
    #[validate(range(min = 1, max = 50, message = "Limit must be between 1 and 50"))]
    pub limit: Option<i32>,
}

impl Default for RecentTransactionsQuery {
    fn default() -> Self {
        Self { limit: Some(10) }
    }
}