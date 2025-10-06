use crate::models::{
    ExpenseSummaryResponse, CategorySummaryResponse, CategorySummaryItem,
    TrendResponse, TrendItem, RecentTransactionsResponse, RecentTransactionItem,
    DateRangeQuery, RecentTransactionsQuery
};
use crate::repositories::TransactionRepository;
use crate::utils::AppError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::info;

#[derive(Clone)]
pub struct ExpenseAnalyticsService<T>
where
    T: TransactionRepository,
{
    transaction_repo: T,
}

impl<T> ExpenseAnalyticsService<T>
where
    T: TransactionRepository,
{
    pub fn new(transaction_repo: T) -> Self {
        Self { transaction_repo }
    }

    pub async fn get_expense_summary(
        &self,
        user_id: uuid::Uuid,
        query: DateRangeQuery,
    ) -> Result<ExpenseSummaryResponse, AppError> {
        info!("Getting expense summary for user {} from {} to {}", user_id, query.from_date, query.to_date);

        // Parse dates
        let from_date = NaiveDate::parse_from_str(&query.from_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid from_date format".to_string()))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let to_date = NaiveDate::parse_from_str(&query.to_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid to_date format".to_string()))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        // Get expense transactions (negative amounts)
        let transactions = self.transaction_repo
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let expense_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .collect();

        let total_expenses = expense_transactions
            .iter()
            .map(|t| t.amount.abs())
            .sum::<Decimal>();

        let total_transactions = expense_transactions.len() as i64;

        // Calculate days between dates
        let days = (to_date.date_naive() - from_date.date_naive()).num_days() + 1;
        let average_per_day = if days > 0 {
            total_expenses / Decimal::from(days)
        } else {
            Decimal::ZERO
        };

        Ok(ExpenseSummaryResponse {
            total_expenses,
            total_transactions,
            average_per_day,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_category_summary(
        &self,
        user_id: uuid::Uuid,
        query: DateRangeQuery,
    ) -> Result<CategorySummaryResponse, AppError> {
        info!("Getting expense category summary for user {} from {} to {}", user_id, query.from_date, query.to_date);

        // Parse dates
        let from_date = NaiveDate::parse_from_str(&query.from_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid from_date format".to_string()))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let to_date = NaiveDate::parse_from_str(&query.to_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid to_date format".to_string()))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        // Get expense transactions (negative amounts)
        let transactions = self.transaction_repo
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let expense_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .collect();

        // Group by category
        let mut category_totals: HashMap<String, (Decimal, i64)> = HashMap::new();
        let mut total_expenses = Decimal::ZERO;

        for transaction in &expense_transactions {
            let amount = transaction.amount.abs();
            total_expenses += amount;
            
            let category = transaction.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
            let (current_amount, current_count) = category_totals.entry(category).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format
        let mut categories: Vec<CategorySummaryItem> = category_totals
            .into_iter()
            .map(|(category, (amount, count))| {
                let percentage = if total_expenses > Decimal::ZERO {
                    (amount / total_expenses) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                CategorySummaryItem {
                    category: Some(category),
                    total_amount: amount,
                    transaction_count: count,
                    percentage,
                }
            })
            .collect();

        // Sort by amount descending
        categories.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());

        Ok(CategorySummaryResponse {
            categories,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_monthly_trend(
        &self,
        user_id: uuid::Uuid,
        query: DateRangeQuery,
    ) -> Result<TrendResponse, AppError> {
        info!("Getting expense monthly trend for user {} from {} to {}", user_id, query.from_date, query.to_date);

        // Parse dates
        let from_date = NaiveDate::parse_from_str(&query.from_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid from_date format".to_string()))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let to_date = NaiveDate::parse_from_str(&query.to_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid to_date format".to_string()))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        // Get expense transactions (negative amounts)
        let transactions = self.transaction_repo
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let expense_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .collect();

        // Group by month
        let mut monthly_totals: HashMap<String, (Decimal, i64)> = HashMap::new();

        for transaction in &expense_transactions {
            let month_key = transaction.transaction_date.format("%Y-%m").to_string();
            let amount = transaction.amount.abs();
            
            let (current_amount, current_count) = monthly_totals.entry(month_key).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format and sort by period
        let mut trends: Vec<TrendItem> = monthly_totals
            .into_iter()
            .map(|(period, (amount, count))| TrendItem {
                period,
                total_amount: amount,
                transaction_count: count,
            })
            .collect();

        trends.sort_by(|a, b| a.period.cmp(&b.period));

        Ok(TrendResponse {
            trends,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_daily_trend(
        &self,
        user_id: uuid::Uuid,
        query: DateRangeQuery,
    ) -> Result<TrendResponse, AppError> {
        info!("Getting expense daily trend for user {} from {} to {}", user_id, query.from_date, query.to_date);

        // Parse dates
        let from_date = NaiveDate::parse_from_str(&query.from_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid from_date format".to_string()))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let to_date = NaiveDate::parse_from_str(&query.to_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid to_date format".to_string()))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        // Get expense transactions (negative amounts)
        let transactions = self.transaction_repo
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let expense_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .collect();

        // Group by day
        let mut daily_totals: HashMap<String, (Decimal, i64)> = HashMap::new();

        for transaction in &expense_transactions {
            let day_key = transaction.transaction_date.format("%Y-%m-%d").to_string();
            let amount = transaction.amount.abs();
            
            let (current_amount, current_count) = daily_totals.entry(day_key).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format and sort by period
        let mut trends: Vec<TrendItem> = daily_totals
            .into_iter()
            .map(|(period, (amount, count))| TrendItem {
                period,
                total_amount: amount,
                transaction_count: count,
            })
            .collect();

        trends.sort_by(|a, b| a.period.cmp(&b.period));

        Ok(TrendResponse {
            trends,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_recent_transactions(
        &self,
        user_id: uuid::Uuid,
        query: RecentTransactionsQuery,
    ) -> Result<RecentTransactionsResponse, AppError> {
        let limit = query.limit.unwrap_or(10);
        info!("Getting {} recent expense transactions for user {}", limit, user_id);

        // Get recent transactions
        let transactions = self.transaction_repo
            .find_by_user_id(user_id, &crate::models::ListTransactionsQuery {
                page: Some(1),
                limit: Some(limit),
                category: None,
                from_date: None,
                to_date: None,
                transaction_type: Some("expense".to_string()),
            })
            .await?;

        // Filter for expenses only (negative amounts) and convert to response format
        let expense_transactions: Vec<RecentTransactionItem> = transactions
            .into_iter()
            .filter(|t| t.amount < Decimal::ZERO)
            .take(limit as usize)
            .map(|t| RecentTransactionItem {
                id: t.id,
                user_id: t.user_id,
                account_id: t.account_id,
                description: t.description,
                amount: t.amount.abs(), // Return positive amount for display
                category: t.category.clone(),
                transaction_date: t.transaction_date,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect();

        let count = expense_transactions.len() as i64;

        Ok(RecentTransactionsResponse {
            data: expense_transactions,
            limit,
            count,
        })
    }
}