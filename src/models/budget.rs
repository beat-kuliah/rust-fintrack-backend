use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Budget {
    pub id: i64,
    pub user_id: Uuid,
    pub category: String,
    pub target_amount: rust_decimal::Decimal,
    pub period_type: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetResponse {
    pub id: i64,
    pub category: String,
    pub target_amount: String,
    pub period_type: String,
    pub period_start: String,
    pub period_end: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBudgetRequest {
    #[validate(length(min = 1, max = 100, message = "Category must be between 1 and 100 characters"))]
    pub category: String,
    
    #[validate(range(min = 0.01, message = "Target amount must be greater than 0"))]
    pub target_amount: f64,
    
    #[validate(custom(function = "validate_period_type"))]
    pub period_type: String,
    
    pub period_start: String, // YYYY-MM-DD format
    pub period_end: String,   // YYYY-MM-DD format
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBudgetRequest {
    #[validate(length(min = 1, max = 100, message = "Category must be between 1 and 100 characters"))]
    pub category: Option<String>,
    
    #[validate(range(min = 0.01, message = "Target amount must be greater than 0"))]
    pub target_amount: Option<f64>,
    
    #[validate(custom(function = "validate_period_type"))]
    pub period_type: Option<String>,
    
    pub period_start: Option<String>, // YYYY-MM-DD format
    pub period_end: Option<String>,   // YYYY-MM-DD format
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListBudgetsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub category: Option<String>,
    pub period_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListBudgetsResponse {
    pub data: Vec<BudgetResponse>,
    pub page: i64,
    pub limit: i64,
    pub total_items: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetSummaryResponse {
    pub total_budgets: i64,
    pub active_budgets: i64,
    pub total_target_amount: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPerformanceItem {
    pub category: String,
    pub target_amount: String,
    pub spent_amount: String,
    pub remaining_amount: String,
    pub percentage_used: f64,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPerformanceResponse {
    pub budgets: Vec<BudgetPerformanceItem>,
    pub total_target: String,
    pub total_spent: String,
    pub total_remaining: String,
    pub overall_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetSuggestionItem {
    pub category: String,
    pub suggested_amount: String,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetSuggestionsResponse {
    pub suggestions: Vec<BudgetSuggestionItem>,
}

pub fn validate_period_type(period_type: &str) -> Result<(), validator::ValidationError> {
    match period_type {
        "weekly" | "monthly" | "quarterly" | "yearly" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_period_type")),
    }
}

impl Budget {
    pub fn to_response(&self) -> BudgetResponse {
        BudgetResponse {
            id: self.id,
            category: self.category.clone(),
            target_amount: self.target_amount.to_string(),
            period_type: self.period_type.clone(),
            period_start: self.period_start.format("%Y-%m-%d").to_string(),
            period_end: self.period_end.format("%Y-%m-%d").to_string(),
            is_active: self.is_active,
            created_at: self.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            updated_at: self.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }
    }
}

impl From<Budget> for BudgetResponse {
    fn from(budget: Budget) -> Self {
        budget.to_response()
    }
}