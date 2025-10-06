use crate::models::{
    IncomeSummaryResponse, IncomeCategorySummaryResponse, IncomeCategorySummaryItem,
    IncomeTrendResponse, IncomeTrendItem, RecentIncomeTransactionsResponse, RecentIncomeTransactionItem,
    IncomeDateRangeQuery, IncomeRecentTransactionsQuery
};
use crate::repositories::TransactionRepository;
use crate::utils::AppError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::info;

#[derive(Clone)]
pub struct IncomeAnalyticsService<T>
where
    T: TransactionRepository,
{
    transaction_repository: T,
}

impl<T> IncomeAnalyticsService<T>
where
    T: TransactionRepository,
{
    pub fn new(transaction_repository: T) -> Self {
        Self { transaction_repository }
    }

    pub async fn get_income_summary(
        &self,
        user_id: uuid::Uuid,
        query: IncomeDateRangeQuery,
    ) -> Result<IncomeSummaryResponse, AppError> {
        info!("Getting income summary for user {} from {} to {}", user_id, query.from_date, query.to_date);

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

        // Get income transactions (positive amounts)
        let transactions = self.transaction_repository
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let income_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount > Decimal::ZERO)
            .collect();

        let total_income = income_transactions
            .iter()
            .map(|t| t.amount)
            .sum::<Decimal>();

        let total_transactions = income_transactions.len() as i64;

        // Calculate days between dates
        let days = (to_date.date_naive() - from_date.date_naive()).num_days() + 1;
        let average_per_day = if days > 0 {
            total_income / Decimal::from(days)
        } else {
            Decimal::ZERO
        };

        Ok(IncomeSummaryResponse {
            total_income,
            total_transactions,
            average_per_day,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_category_summary(
        &self,
        user_id: uuid::Uuid,
        query: IncomeDateRangeQuery,
    ) -> Result<IncomeCategorySummaryResponse, AppError> {
        info!("Getting income category summary for user {} from {} to {}", user_id, query.from_date, query.to_date);

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

        // Get income transactions (positive amounts)
        let transactions = self.transaction_repository
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let income_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount > Decimal::ZERO)
            .collect();

        // Group by category
        let mut category_totals: HashMap<String, (Decimal, i64)> = HashMap::new();
        let mut total_income = Decimal::ZERO;

        for transaction in &income_transactions {
            let amount = transaction.amount;
            total_income += amount;
            
            let category = transaction.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
            let (current_amount, current_count) = category_totals.entry(category).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format
        let mut categories: Vec<IncomeCategorySummaryItem> = category_totals
            .into_iter()
            .map(|(category, (amount, count))| {
                let percentage = if total_income > Decimal::ZERO {
                    (amount / total_income) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                IncomeCategorySummaryItem {
                    category: Some(category),
                    total_amount: amount,
                    transaction_count: count,
                    percentage,
                }
            })
            .collect();

        // Sort by amount descending
        categories.sort_by(|a, b| b.total_amount.partial_cmp(&a.total_amount).unwrap());

        Ok(IncomeCategorySummaryResponse {
            categories,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_monthly_trend(
        &self,
        user_id: uuid::Uuid,
        query: IncomeDateRangeQuery,
    ) -> Result<IncomeTrendResponse, AppError> {
        info!("Getting income monthly trend for user {} from {} to {}", user_id, query.from_date, query.to_date);

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

        // Get income transactions (positive amounts)
        let transactions = self.transaction_repository
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let income_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount > Decimal::ZERO)
            .collect();

        // Group by month
        let mut monthly_totals: HashMap<String, (Decimal, i64)> = HashMap::new();

        for transaction in &income_transactions {
            let month_key = transaction.transaction_date.format("%Y-%m").to_string();
            let amount = transaction.amount;
            
            let (current_amount, current_count) = monthly_totals.entry(month_key).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format and sort by period
        let mut trends: Vec<IncomeTrendItem> = monthly_totals
            .into_iter()
            .map(|(period, (amount, count))| IncomeTrendItem {
                period,
                total_amount: amount,
                transaction_count: count,
            })
            .collect();

        trends.sort_by(|a, b| a.period.cmp(&b.period));

        Ok(IncomeTrendResponse {
            trends,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_daily_trend(
        &self,
        user_id: uuid::Uuid,
        query: IncomeDateRangeQuery,
    ) -> Result<IncomeTrendResponse, AppError> {
        info!("Getting income daily trend for user {} from {} to {}", user_id, query.from_date, query.to_date);

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

        // Get income transactions (positive amounts)
        let transactions = self.transaction_repository
            .find_by_date_range(user_id, from_date, to_date)
            .await?;

        let income_transactions: Vec<_> = transactions
            .into_iter()
            .filter(|t| t.amount > Decimal::ZERO)
            .collect();

        // Group by day
        let mut daily_totals: HashMap<String, (Decimal, i64)> = HashMap::new();

        for transaction in &income_transactions {
            let day_key = transaction.transaction_date.format("%Y-%m-%d").to_string();
            let amount = transaction.amount;
            
            let (current_amount, current_count) = daily_totals.entry(day_key).or_insert((Decimal::ZERO, 0));
            *current_amount += amount;
            *current_count += 1;
        }

        // Convert to response format and sort by period
        let mut trends: Vec<IncomeTrendItem> = daily_totals
            .into_iter()
            .map(|(period, (amount, count))| IncomeTrendItem {
                period,
                total_amount: amount,
                transaction_count: count,
            })
            .collect();

        trends.sort_by(|a, b| a.period.cmp(&b.period));

        Ok(IncomeTrendResponse {
            trends,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    }

    pub async fn get_recent_transactions(
        &self,
        user_id: uuid::Uuid,
        query: IncomeRecentTransactionsQuery,
    ) -> Result<RecentIncomeTransactionsResponse, AppError> {
        let limit = query.limit.unwrap_or(10);
        info!("Getting {} recent income transactions for user {}", limit, user_id);

        // Get recent transactions
        let query = crate::models::ListTransactionsQuery {
            category: None,
            transaction_type: Some("income".to_string()),
            from_date: None,
            to_date: None,
            limit: Some(limit),
            page: Some(1),
        };
        
        let transactions = self.transaction_repository
            .find_by_user_id(user_id, &query)
            .await?;

        // Filter for income only (positive amounts) and convert to response format
        let income_transactions: Vec<RecentIncomeTransactionItem> = transactions
            .into_iter()
            .filter(|t| t.amount > Decimal::ZERO)
            .take(limit as usize)
            .map(|t| RecentIncomeTransactionItem {
                id: t.id,
                user_id: t.user_id,
                account_id: t.account_id,
                description: t.description,
                amount: t.amount,
                category: t.category.clone(),
                transaction_date: t.transaction_date,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect();

        let count = income_transactions.len() as i64;

        Ok(RecentIncomeTransactionsResponse {
            data: income_transactions,
            limit,
            count,
        })
    }
}