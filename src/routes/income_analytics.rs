use axum::{
    routing::get,
    Router,
};

use crate::handlers::income_analytics::{
    get_income_summary, get_income_category_summary, get_income_monthly_trend,
    get_income_daily_trend, get_recent_income_transactions,
};
use crate::middleware::auth_middleware;
use crate::services::IncomeAnalyticsService;
use crate::repositories::PostgresTransactionRepository;

pub fn income_analytics_routes() -> Router<IncomeAnalyticsService<PostgresTransactionRepository>> {
    Router::new()
        .route("/income-analytics/summary", get(get_income_summary))
        .route("/income-analytics/category-summary", get(get_income_category_summary))
        .route("/income-analytics/monthly-trend", get(get_income_monthly_trend))
        .route("/income-analytics/daily-trend", get(get_income_daily_trend))
        .route("/income-analytics/recent", get(get_recent_income_transactions))
        .layer(axum::middleware::from_fn(auth_middleware))
}