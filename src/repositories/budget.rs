use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{Budget, CreateBudgetRequest, UpdateBudgetRequest, ListBudgetsQuery};
use crate::utils::AppError;

#[async_trait]
pub trait BudgetRepository: Send + Sync + Clone {
    async fn find_by_id(&self, id: i64) -> Result<Option<Budget>, AppError>;
    async fn find_by_user_id(&self, user_id: Uuid, query: &ListBudgetsQuery) -> Result<Vec<Budget>, AppError>;
    async fn create(&self, user_id: Uuid, request: &CreateBudgetRequest) -> Result<Budget, AppError>;
    async fn update(&self, id: i64, user_id: Uuid, request: &UpdateBudgetRequest) -> Result<Budget, AppError>;
    async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError>;
    async fn count_by_user_id(&self, user_id: Uuid, query: &ListBudgetsQuery) -> Result<i64, AppError>;
    async fn get_categories(&self, user_id: Uuid) -> Result<Vec<String>, AppError>;
    async fn get_budget_performance(&self, user_id: Uuid) -> Result<Vec<(Budget, Decimal)>, AppError>;
}

#[derive(Clone)]
pub struct PostgresBudgetRepository {
    pool: PgPool,
}

impl PostgresBudgetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn build_where_clause(&self, query: &ListBudgetsQuery) -> String {
        let mut conditions = vec!["user_id = $1".to_string()];
        let mut param_count = 2;

        if query.category.is_some() {
            conditions.push(format!("category ILIKE ${}", param_count));
            param_count += 1;
        }

        if query.period_type.is_some() {
            conditions.push(format!("period_type = ${}", param_count));
            param_count += 1;
        }

        if query.is_active.is_some() {
            conditions.push(format!("is_active = ${}", param_count));
        }

        if conditions.len() > 1 {
            format!("WHERE {}", conditions.join(" AND "))
        } else {
            "WHERE user_id = $1".to_string()
        }
    }
}

#[async_trait]
impl BudgetRepository for PostgresBudgetRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<Budget>, AppError> {
        let budget = sqlx::query_as::<_, Budget>(
            "SELECT id, user_id, category, target_amount, period_type, period_start, period_end, is_active, created_at, updated_at 
             FROM budgets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(budget)
    }

    async fn find_by_user_id(&self, user_id: Uuid, query: &ListBudgetsQuery) -> Result<Vec<Budget>, AppError> {
        let where_clause = self.build_where_clause(query);
        let limit = query.limit.unwrap_or(20);
        let offset = (query.page.unwrap_or(1) - 1) * limit;

        let sql = format!(
            "SELECT id, user_id, category, target_amount, period_type, period_start, period_end, is_active, created_at, updated_at 
             FROM budgets {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause, limit, offset
        );

        let mut sql_query = sqlx::query_as::<_, Budget>(&sql)
            .bind(user_id);

        if let Some(ref category) = query.category {
            sql_query = sql_query.bind(format!("%{}%", category));
        }

        if let Some(ref period_type) = query.period_type {
            sql_query = sql_query.bind(period_type);
        }

        if let Some(is_active) = query.is_active {
            sql_query = sql_query.bind(is_active);
        }

        let budgets = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(budgets)
    }

    async fn create(&self, user_id: Uuid, request: &CreateBudgetRequest) -> Result<Budget, AppError> {
        // Parse dates
        let period_start = NaiveDate::parse_from_str(&request.period_start, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid period_start format. Use YYYY-MM-DD".to_string()))?;
        
        let period_end = NaiveDate::parse_from_str(&request.period_end, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid period_end format. Use YYYY-MM-DD".to_string()))?;

        // Validate date range
        if period_end <= period_start {
            return Err(AppError::ValidationError("Period end must be after period start".to_string()));
        }

        // Convert amount to Decimal
        let target_amount = Decimal::from_f64_retain(request.target_amount)
            .ok_or_else(|| AppError::ValidationError("Invalid target amount".to_string()))?;

        // Check for duplicate active budget in same category and period
        let existing = sqlx::query(
            "SELECT id FROM budgets 
             WHERE user_id = $1 AND category = $2 AND is_active = true 
             AND (period_start <= $4 AND period_end >= $3)"
        )
        .bind(user_id)
        .bind(&request.category)
        .bind(period_start)
        .bind(period_end)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(AppError::ValidationError("An active budget already exists for this category in the specified period".to_string()));
        }

        let budget = sqlx::query_as::<_, Budget>(
            "INSERT INTO budgets (user_id, category, target_amount, period_type, period_start, period_end, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, true, $7, $7)
             RETURNING id, user_id, category, target_amount, period_type, period_start, period_end, is_active, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&request.category)
        .bind(target_amount)
        .bind(&request.period_type)
        .bind(period_start)
        .bind(period_end)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(budget)
    }

    async fn update(&self, id: i64, user_id: Uuid, request: &UpdateBudgetRequest) -> Result<Budget, AppError> {
        // First check if budget exists and belongs to user
        let existing = self.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Budget not found".to_string()))?;

        if existing.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        let mut update_fields = Vec::new();
        let mut param_count = 1;

        if request.category.is_some() {
            update_fields.push(format!("category = ${}", param_count));
            param_count += 1;
        }

        if request.target_amount.is_some() {
            update_fields.push(format!("target_amount = ${}", param_count));
            param_count += 1;
        }

        if request.period_type.is_some() {
            update_fields.push(format!("period_type = ${}", param_count));
            param_count += 1;
        }

        if request.period_start.is_some() {
            update_fields.push(format!("period_start = ${}", param_count));
            param_count += 1;
        }

        if request.period_end.is_some() {
            update_fields.push(format!("period_end = ${}", param_count));
            param_count += 1;
        }

        if request.is_active.is_some() {
            update_fields.push(format!("is_active = ${}", param_count));
            param_count += 1;
        }

        if update_fields.is_empty() {
            return Ok(existing);
        }

        update_fields.push(format!("updated_at = ${}", param_count));
        param_count += 1;

        let sql = format!(
            "UPDATE budgets SET {} WHERE id = ${} AND user_id = ${}
             RETURNING id, user_id, category, target_amount, period_type, period_start, period_end, is_active, created_at, updated_at",
            update_fields.join(", "), param_count, param_count + 1
        );

        let mut query = sqlx::query_as::<_, Budget>(&sql);
        
        if let Some(ref category) = request.category {
            query = query.bind(category);
        }

        if let Some(target_amount) = request.target_amount {
            let decimal_amount = Decimal::from_f64_retain(target_amount)
                .ok_or_else(|| AppError::ValidationError("Invalid target amount".to_string()))?;
            query = query.bind(decimal_amount);
        }

        if let Some(ref period_type) = request.period_type {
            query = query.bind(period_type);
        }

        if let Some(ref period_start) = request.period_start {
            let start_date = NaiveDate::parse_from_str(period_start, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("Invalid period_start format. Use YYYY-MM-DD".to_string()))?;
            query = query.bind(start_date);
        }

        if let Some(ref period_end) = request.period_end {
            let end_date = NaiveDate::parse_from_str(period_end, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("Invalid period_end format. Use YYYY-MM-DD".to_string()))?;
            query = query.bind(end_date);
        }

        if let Some(is_active) = request.is_active {
            query = query.bind(is_active);
        }

        query = query.bind(Utc::now()).bind(id).bind(user_id);

        let budget = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(budget)
    }

    async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM budgets WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Budget not found".to_string()));
        }

        Ok(())
    }

    async fn count_by_user_id(&self, user_id: Uuid, query: &ListBudgetsQuery) -> Result<i64, AppError> {
        let where_clause = self.build_where_clause(query);

        let sql = format!("SELECT COUNT(*) FROM budgets {}", where_clause);
        let mut sql_query = sqlx::query(&sql)
            .bind(user_id);

        if let Some(ref category) = query.category {
            sql_query = sql_query.bind(format!("%{}%", category));
        }

        if let Some(ref period_type) = query.period_type {
            sql_query = sql_query.bind(period_type);
        }

        if let Some(is_active) = query.is_active {
            sql_query = sql_query.bind(is_active);
        }

        let row = sql_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.get::<i64, _>(0))
    }

    async fn get_categories(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let categories = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT category FROM budgets WHERE user_id = $1 ORDER BY category"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(categories)
    }

    async fn get_budget_performance(&self, user_id: Uuid) -> Result<Vec<(Budget, Decimal)>, AppError> {
        let rows = sqlx::query(
            "SELECT b.id, b.user_id, b.category, b.target_amount, b.period_type, b.period_start, b.period_end, 
                    b.is_active, b.created_at, b.updated_at,
                    COALESCE(SUM(CASE WHEN t.transaction_type = 'expense' THEN t.amount ELSE 0 END), 0) as spent_amount
             FROM budgets b
             LEFT JOIN transactions t ON t.user_id = b.user_id 
                 AND t.category = b.category 
                 AND t.transaction_date >= b.period_start 
                 AND t.transaction_date <= b.period_end
             WHERE b.user_id = $1 AND b.is_active = true
             GROUP BY b.id, b.user_id, b.category, b.target_amount, b.period_type, b.period_start, b.period_end, 
                      b.is_active, b.created_at, b.updated_at
             ORDER BY b.created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let budget = Budget {
                id: row.get("id"),
                user_id: row.get("user_id"),
                category: row.get("category"),
                target_amount: row.get("target_amount"),
                period_type: row.get("period_type"),
                period_start: row.get("period_start"),
                period_end: row.get("period_end"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            let spent_amount: Decimal = row.get("spent_amount");
            results.push((budget, spent_amount));
        }

        Ok(results)
    }
}