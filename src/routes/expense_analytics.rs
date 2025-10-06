use axum::{
    routing::get,
    Router,
};

use crate::handlers::expense_analytics::{
    get_expense_summary, get_expense_category_summary, get_expense_monthly_trend,
    get_expense_daily_trend, get_recent_expense_transactions,
};
use crate::middleware::auth_middleware;
use crate::services::ExpenseAnalyticsService;
use crate::repositories::PostgresTransactionRepository;

pub fn expense_analytics_routes() -> Router<ExpenseAnalyticsService<PostgresTransactionRepository>> {
    Router::new()
        .route("/expense-analytics/summary", get(get_expense_summary))
        .route("/expense-analytics/category-summary", get(get_expense_category_summary))
        .route("/expense-analytics/monthly-trend", get(get_expense_monthly_trend))
        .route("/expense-analytics/daily-trend", get(get_expense_daily_trend))
        .route("/expense-analytics/recent", get(get_recent_expense_transactions))
        .layer(axum::middleware::from_fn(auth_middleware))
}