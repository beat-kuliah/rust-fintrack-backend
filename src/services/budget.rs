use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;

use crate::models::{
    BudgetResponse, CreateBudgetRequest, UpdateBudgetRequest, 
    ListBudgetsQuery, ListBudgetsResponse, BudgetSummaryResponse,
    BudgetPerformanceResponse, BudgetPerformanceItem, BudgetSuggestionsResponse,
    BudgetSuggestionItem
};
use crate::repositories::BudgetRepository;
use crate::utils::AppError;

#[derive(Clone)]
pub struct BudgetService<R: BudgetRepository> {
    repository: R,
}

impl<R: BudgetRepository> BudgetService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_budget_by_id(&self, id: i64, user_id: Uuid) -> Result<BudgetResponse, AppError> {
        let budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Budget not found".to_string()))?;

        // Check if budget belongs to user
        if budget.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(budget.to_response())
    }

    pub async fn list_budgets(&self, user_id: Uuid, query: ListBudgetsQuery) -> Result<ListBudgetsResponse, AppError> {
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

        // Validate period type if provided
        if let Some(ref period_type) = query.period_type {
            if !["weekly", "monthly", "quarterly", "yearly"].contains(&period_type.as_str()) {
                return Err(AppError::ValidationError("Period type must be 'weekly', 'monthly', 'quarterly', or 'yearly'".to_string()));
            }
        }

        let budgets = self.repository.find_by_user_id(user_id, &query).await?;
        let total_items = self.repository.count_by_user_id(user_id, &query).await?;

        let budget_responses = budgets
            .into_iter()
            .map(|budget| budget.to_response())
            .collect();

        Ok(ListBudgetsResponse {
            data: budget_responses,
            page: query.page.unwrap_or(1),
            limit: query.limit.unwrap_or(20),
            total_items,
        })
    }

    pub async fn create_budget(&self, user_id: Uuid, request: CreateBudgetRequest) -> Result<BudgetResponse, AppError> {
        let budget = self.repository.create(user_id, &request).await?;
        Ok(budget.to_response())
    }

    pub async fn update_budget(&self, id: i64, user_id: Uuid, request: UpdateBudgetRequest) -> Result<BudgetResponse, AppError> {
        let budget = self.repository.update(id, user_id, &request).await?;
        Ok(budget.to_response())
    }

    pub async fn delete_budget(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id, user_id).await
    }

    pub async fn get_budget_summary(&self, user_id: Uuid) -> Result<BudgetSummaryResponse, AppError> {
        let all_budgets_query = ListBudgetsQuery {
            page: None,
            limit: None,
            category: None,
            period_type: None,
            is_active: None,
        };

        let active_budgets_query = ListBudgetsQuery {
            page: None,
            limit: None,
            category: None,
            period_type: None,
            is_active: Some(true),
        };

        let total_budgets = self.repository.count_by_user_id(user_id, &all_budgets_query).await?;
        let active_budgets = self.repository.count_by_user_id(user_id, &active_budgets_query).await?;
        
        let budgets = self.repository.find_by_user_id(user_id, &active_budgets_query).await?;
        let total_target_amount: Decimal = budgets.iter().map(|b| b.target_amount).sum();
        
        let categories = self.repository.get_categories(user_id).await?;

        Ok(BudgetSummaryResponse {
            total_budgets,
            active_budgets,
            total_target_amount: total_target_amount.to_string(),
            categories,
        })
    }

    pub async fn get_budget_performance(&self, user_id: Uuid) -> Result<BudgetPerformanceResponse, AppError> {
        let budget_performance = self.repository.get_budget_performance(user_id).await?;
        
        let mut total_target = Decimal::new(0, 0);
        let mut total_spent = Decimal::new(0, 0);
        let mut performance_items = Vec::new();

        for (budget, spent_amount) in budget_performance {
            total_target += budget.target_amount;
            total_spent += spent_amount;

            let remaining_amount = budget.target_amount - spent_amount;
            let percentage_used = if budget.target_amount > Decimal::new(0, 0) {
                (spent_amount / budget.target_amount * Decimal::new(100, 0)).to_f64().unwrap_or(0.0)
            } else {
                0.0
            };

            performance_items.push(BudgetPerformanceItem {
                category: budget.category,
                target_amount: budget.target_amount.to_string(),
                spent_amount: spent_amount.to_string(),
                remaining_amount: remaining_amount.to_string(),
                percentage_used,
                period_start: budget.period_start.format("%Y-%m-%d").to_string(),
                period_end: budget.period_end.format("%Y-%m-%d").to_string(),
            });
        }

        let total_remaining = total_target - total_spent;
        let overall_percentage = if total_target > Decimal::new(0, 0) {
            (total_spent / total_target * Decimal::new(100, 0)).to_f64().unwrap_or(0.0)
        } else {
            0.0
        };

        Ok(BudgetPerformanceResponse {
            budgets: performance_items,
            total_target: total_target.to_string(),
            total_spent: total_spent.to_string(),
            total_remaining: total_remaining.to_string(),
            overall_percentage,
        })
    }

    pub async fn get_budget_categories(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        self.repository.get_categories(user_id).await
    }

    pub async fn get_budget_suggestions(&self, user_id: Uuid) -> Result<BudgetSuggestionsResponse, AppError> {
        // This is a simplified implementation. In a real application, you might want to:
        // 1. Analyze historical spending patterns
        // 2. Use machine learning algorithms
        // 3. Consider seasonal variations
        // 4. Factor in income changes
        
        let budget_performance = self.repository.get_budget_performance(user_id).await?;
        let mut suggestions = Vec::new();

        for (budget, spent_amount) in budget_performance {
            let percentage_used = if budget.target_amount > Decimal::new(0, 0) {
                (spent_amount / budget.target_amount * Decimal::new(100, 0)).to_f64().unwrap_or(0.0)
            } else {
                0.0
            };

            // Generate suggestions based on spending patterns
            if percentage_used > 90.0 {
                let suggested_increase = budget.target_amount * Decimal::new(120, 2); // 20% increase
                suggestions.push(BudgetSuggestionItem {
                    category: budget.category.clone(),
                    suggested_amount: suggested_increase.to_string(),
                    reason: "You're consistently exceeding this budget. Consider increasing it by 20%.".to_string(),
                    confidence: 0.85,
                });
            } else if percentage_used < 50.0 {
                let suggested_decrease = budget.target_amount * Decimal::new(80, 2); // 20% decrease
                suggestions.push(BudgetSuggestionItem {
                    category: budget.category.clone(),
                    suggested_amount: suggested_decrease.to_string(),
                    reason: "You're using less than 50% of this budget. Consider reducing it by 20%.".to_string(),
                    confidence: 0.75,
                });
            }
        }

        // Add suggestions for categories without budgets
        // This would require analyzing transaction categories that don't have budgets
        // For now, we'll keep it simple

        Ok(BudgetSuggestionsResponse { suggestions })
    }
}