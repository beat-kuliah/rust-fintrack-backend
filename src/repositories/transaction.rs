use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::models::{Transaction, CreateTransactionRequest, UpdateTransactionRequest, ListTransactionsQuery};
use crate::utils::AppError;

#[async_trait::async_trait]
pub trait TransactionRepository: Clone + Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<Transaction>, AppError>;
    async fn find_by_user_id(&self, user_id: Uuid, query: &ListTransactionsQuery) -> Result<Vec<Transaction>, AppError>;
    async fn find_by_date_range(&self, user_id: Uuid, from_date: chrono::DateTime<Utc>, to_date: chrono::DateTime<Utc>) -> Result<Vec<Transaction>, AppError>;
    async fn create(&self, user_id: Uuid, request: &CreateTransactionRequest) -> Result<Transaction, AppError>;
    async fn update(&self, id: i64, user_id: Uuid, request: &UpdateTransactionRequest) -> Result<Transaction, AppError>;
    async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError>;
    async fn count_by_user_id(&self, user_id: Uuid, query: &ListTransactionsQuery) -> Result<i64, AppError>;
}

#[derive(Clone)]
pub struct PostgresTransactionRepository {
    pool: PgPool,
}

impl PostgresTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn build_where_clause(&self, query: &ListTransactionsQuery) -> (String, Vec<String>) {
        let mut conditions = vec!["user_id = $1".to_string()];
        let mut params = Vec::new();
        let mut param_count = 2;

        if let Some(category) = &query.category {
            conditions.push(format!("category = ${}", param_count));
            params.push(category.clone());
            param_count += 1;
        }

        if let Some(transaction_type) = &query.transaction_type {
            conditions.push(format!("transaction_type = ${}", param_count));
            params.push(transaction_type.clone());
            param_count += 1;
        }

        if let Some(from_date) = &query.from_date {
            conditions.push(format!("transaction_date >= ${}", param_count));
            params.push(from_date.clone());
            param_count += 1;
        }

        if let Some(to_date) = &query.to_date {
            conditions.push(format!("transaction_date <= ${}", param_count));
            params.push(to_date.clone());
        }

        let where_clause = if conditions.len() > 1 {
            format!("WHERE {}", conditions.join(" AND "))
        } else {
            "WHERE user_id = $1".to_string()
        };

        (where_clause, params)
    }
}

#[async_trait::async_trait]
impl TransactionRepository for PostgresTransactionRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<Transaction>, AppError> {
        let row = sqlx::query(
            "SELECT id, user_id, account_id, description, amount, category, transaction_type, transaction_date, created_at, updated_at
             FROM transactions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(row) => {
                let transaction = Transaction {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    account_id: row.get("account_id"),
                    description: row.get("description"),
                    amount: row.get("amount"),
                    category: row.get("category"),
                    transaction_type: row.get("transaction_type"),
                    transaction_date: row.get("transaction_date"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(transaction))
            }
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: Uuid, query: &ListTransactionsQuery) -> Result<Vec<Transaction>, AppError> {
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20);
        let offset = (page - 1) * limit;

        let (where_clause, params) = self.build_where_clause(query);
        
        let sql = format!(
            "SELECT id, user_id, account_id, description, amount, category, transaction_type, transaction_date, created_at, updated_at
             FROM transactions
             {}
             ORDER BY transaction_date DESC, created_at DESC
             LIMIT {} OFFSET {}",
            where_clause, limit, offset
        );

        let mut query_builder = sqlx::query(&sql).bind(user_id);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        
        let transactions = rows.into_iter().map(|row| Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            account_id: row.get("account_id"),
            description: row.get("description"),
            amount: row.get("amount"),
            category: row.get("category"),
            transaction_type: row.get("transaction_type"),
            transaction_date: row.get("transaction_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();
        
        Ok(transactions)
    }

    async fn create(&self, user_id: Uuid, request: &CreateTransactionRequest) -> Result<Transaction, AppError> {
        // Parse amount
        let amount = Decimal::from_str(&request.amount)
            .map_err(|_| AppError::ValidationError("Invalid amount format".to_string()))?;

        // Parse transaction date
        let transaction_date = NaiveDate::parse_from_str(&request.transaction_date, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid date format. Use YYYY-MM-DD".to_string()))?;

        let now = Utc::now();

        let row = sqlx::query(
            "INSERT INTO transactions (user_id, account_id, description, amount, category, transaction_type, transaction_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, user_id, account_id, description, amount, category, transaction_type, transaction_date, created_at, updated_at"
        )
        .bind(user_id)
        .bind(request.account_id)
        .bind(&request.description)
        .bind(amount)
        .bind(&request.category)
        .bind(&request.transaction_type)
        .bind(transaction_date)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        let transaction = Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            account_id: row.get("account_id"),
            description: row.get("description"),
            amount: row.get("amount"),
            category: row.get("category"),
            transaction_type: row.get("transaction_type"),
            transaction_date: row.get("transaction_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        Ok(transaction)
    }

    async fn update(&self, id: i64, user_id: Uuid, request: &UpdateTransactionRequest) -> Result<Transaction, AppError> {
        // Parse amount
        let amount = Decimal::from_str(&request.amount)
            .map_err(|_| AppError::ValidationError("Invalid amount format".to_string()))?;

        // Parse transaction date
        let transaction_date = NaiveDate::parse_from_str(&request.transaction_date, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid date format. Use YYYY-MM-DD".to_string()))?;

        let now = Utc::now();

        let row = sqlx::query(
            "UPDATE transactions 
             SET account_id = $1, description = $2, amount = $3, category = $4, transaction_type = $5, transaction_date = $6, updated_at = $7
             WHERE id = $8 AND user_id = $9
             RETURNING id, user_id, account_id, description, amount, category, transaction_type, transaction_date, created_at, updated_at"
        )
        .bind(request.account_id)
        .bind(&request.description)
        .bind(amount)
        .bind(&request.category)
        .bind(&request.transaction_type)
        .bind(transaction_date)
        .bind(now)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(row) => {
                let transaction = Transaction {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    account_id: row.get("account_id"),
                    description: row.get("description"),
                    amount: row.get("amount"),
                    category: row.get("category"),
                    transaction_type: row.get("transaction_type"),
                    transaction_date: row.get("transaction_date"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(transaction)
            }
            None => Err(AppError::NotFound("Transaction not found or access denied".to_string())),
        }
    }

    async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM transactions WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Transaction not found or access denied".to_string()));
        }
        
        Ok(())
    }

    async fn find_by_date_range(&self, user_id: Uuid, from_date: chrono::DateTime<Utc>, to_date: chrono::DateTime<Utc>) -> Result<Vec<Transaction>, AppError> {
        let sql = "
            SELECT id, user_id, account_id, amount, description, category, 
                   transaction_type, transaction_date, created_at, updated_at
            FROM transactions 
            WHERE user_id = $1 AND transaction_date >= $2 AND transaction_date <= $3
            ORDER BY transaction_date DESC
        ";

        let rows = sqlx::query(sql)
            .bind(user_id)
            .bind(from_date)
            .bind(to_date)
            .fetch_all(&self.pool)
            .await?;

        let mut transactions = Vec::new();
        for row in rows {
            let transaction = Transaction {
                id: row.get("id"),
                user_id: row.get("user_id"),
                account_id: row.get("account_id"),
                amount: row.get("amount"),
                description: row.get("description"),
                category: row.get("category"),
                transaction_type: row.get("transaction_type"),
                transaction_date: row.get("transaction_date"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    async fn count_by_user_id(&self, user_id: Uuid, query: &ListTransactionsQuery) -> Result<i64, AppError> {
        let (where_clause, params) = self.build_where_clause(query);
        
        let sql = format!(
            "SELECT COUNT(*) as count FROM transactions {}",
            where_clause
        );

        let mut query_builder = sqlx::query(&sql).bind(user_id);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let row = query_builder.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        
        Ok(count)
    }
}