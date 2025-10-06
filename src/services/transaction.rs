use uuid::Uuid;

use crate::models::{
    TransactionResponse, CreateTransactionRequest, UpdateTransactionRequest, 
    ListTransactionsQuery, ListTransactionsResponse
};
use crate::repositories::TransactionRepository;
use crate::utils::AppError;

#[derive(Clone)]
pub struct TransactionService<R: TransactionRepository> {
    repository: R,
}

impl<R: TransactionRepository> TransactionService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_transaction_by_id(&self, id: i64, user_id: Uuid) -> Result<TransactionResponse, AppError> {
        let transaction = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        // Check if transaction belongs to user
        if transaction.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(transaction.to_response())
    }

    pub async fn list_transactions(&self, user_id: Uuid, query: ListTransactionsQuery) -> Result<ListTransactionsResponse, AppError> {
        // Validate query parameters
        if let Some(page) = query.page {
            if page < 1 {
                return Err(AppError::ValidationError("Page must be greater than 0".to_string()));
            }
        }

        if let Some(limit) = query.limit {
            if limit < 1 || limit > 100 {
                return Err(AppError::ValidationError("Limit must be between 1 and 100".to_string()));
            }
        }

        // Validate transaction type if provided
        if let Some(ref transaction_type) = query.transaction_type {
            if transaction_type != "income" && transaction_type != "expense" {
                return Err(AppError::ValidationError("Transaction type must be 'income' or 'expense'".to_string()));
            }
        }

        // Validate date format if provided
        if let Some(ref from_date) = query.from_date {
            chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("Invalid from_date format. Use YYYY-MM-DD".to_string()))?;
        }

        if let Some(ref to_date) = query.to_date {
            chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("Invalid to_date format. Use YYYY-MM-DD".to_string()))?;
        }

        let transactions = self.repository.find_by_user_id(user_id, &query).await?;
        let total_items = self.repository.count_by_user_id(user_id, &query).await?;

        let transaction_responses = transactions
            .into_iter()
            .map(|transaction| transaction.to_response())
            .collect();

        Ok(ListTransactionsResponse {
            data: transaction_responses,
            page: query.page.unwrap_or(1),
            limit: query.limit.unwrap_or(20),
            total_items,
        })
    }

    pub async fn create_transaction(&self, user_id: Uuid, request: CreateTransactionRequest) -> Result<TransactionResponse, AppError> {
        let transaction = self.repository.create(user_id, &request).await?;
        Ok(transaction.to_response())
    }

    pub async fn update_transaction(&self, id: i64, user_id: Uuid, request: UpdateTransactionRequest) -> Result<TransactionResponse, AppError> {
        let transaction = self.repository.update(id, user_id, &request).await?;
        Ok(transaction.to_response())
    }

    pub async fn delete_transaction(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id, user_id).await
    }
}