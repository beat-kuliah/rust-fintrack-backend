use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: i64,
    pub user_id: Uuid,
    pub account_id: Option<Uuid>,
    pub description: String,
    pub amount: Decimal,
    pub category: Option<String>,
    pub transaction_type: String,
    pub transaction_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: i64,
    pub user_id: Uuid,
    pub account_id: Option<Uuid>,
    pub description: String,
    pub amount: String,
    pub category: Option<String>,
    pub transaction_type: String,
    pub transaction_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    pub account_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500, message = "Description must be between 1 and 500 characters"))]
    pub description: String,
    #[validate(length(min = 1, message = "Amount is required"))]
    pub amount: String,
    #[validate(length(min = 1, max = 100, message = "Category must be between 1 and 100 characters"))]
    pub category: String,
    #[validate(custom(function = "validate_transaction_type"))]
    pub transaction_type: String,
    pub transaction_date: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTransactionRequest {
    pub account_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500, message = "Description must be between 1 and 500 characters"))]
    pub description: String,
    #[validate(length(min = 1, message = "Amount is required"))]
    pub amount: String,
    #[validate(length(min = 1, max = 100, message = "Category must be between 1 and 100 characters"))]
    pub category: String,
    #[validate(custom(function = "validate_transaction_type"))]
    pub transaction_type: String,
    pub transaction_date: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub category: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub transaction_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTransactionsResponse {
    pub data: Vec<TransactionResponse>,
    pub page: i32,
    pub limit: i32,
    pub total_items: i64,
}

fn validate_transaction_type(transaction_type: &str) -> Result<(), validator::ValidationError> {
    if transaction_type == "income" || transaction_type == "expense" {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_transaction_type"))
    }
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        Self {
            id: transaction.id,
            user_id: transaction.user_id,
            account_id: transaction.account_id,
            description: transaction.description,
            amount: transaction.amount.to_string(),
            category: transaction.category,
            transaction_type: transaction.transaction_type,
            transaction_date: transaction.transaction_date,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}

impl Transaction {
    pub fn to_response(self) -> TransactionResponse {
        TransactionResponse::from(self)
    }
}