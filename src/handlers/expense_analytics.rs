use axum::{
    extract::{Query, State},
    response::Json,
    Extension,
};
use tracing::{error, info};
use validator::Validate;

use crate::models::{
    DateRangeQuery, RecentTransactionsQuery, ExpenseSummaryResponse,
    CategorySummaryResponse, TrendResponse, RecentTransactionsResponse,
};
use crate::services::ExpenseAnalyticsService;
use crate::repositories::PostgresTransactionRepository;
use crate::utils::{AppError, CacheService};

pub async fn get_expense_summary(
    State(service): State<ExpenseAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ExpenseSummaryResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting expense summary for user {}", user_id);

    // Create cache key
    let cache_key = format!("expense_summary:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<ExpenseSummaryResponse>(&cache_key).await {
        info!("Returning cached expense summary for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_expense_summary(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache expense summary");
    }

    Ok(Json(response))
}

pub async fn get_expense_category_summary(
    State(service): State<ExpenseAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<CategorySummaryResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting expense category summary for user {}", user_id);

    // Create cache key
    let cache_key = format!("expense_category_summary:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<CategorySummaryResponse>(&cache_key).await {
        info!("Returning cached expense category summary for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_category_summary(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache expense category summary");
    }

    Ok(Json(response))
}

pub async fn get_expense_monthly_trend(
    State(service): State<ExpenseAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting expense monthly trend for user {}", user_id);

    // Create cache key
    let cache_key = format!("expense_monthly_trend:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<TrendResponse>(&cache_key).await {
        info!("Returning cached expense monthly trend for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_monthly_trend(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache expense monthly trend");
    }

    Ok(Json(response))
}

pub async fn get_expense_daily_trend(
    State(service): State<ExpenseAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting expense daily trend for user {}", user_id);

    // Create cache key
    let cache_key = format!("expense_daily_trend:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<TrendResponse>(&cache_key).await {
        info!("Returning cached expense daily trend for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_daily_trend(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache expense daily trend");
    }

    Ok(Json(response))
}

pub async fn get_recent_expense_transactions(
    State(service): State<ExpenseAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<RecentTransactionsQuery>,
) -> Result<Json<RecentTransactionsResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    let limit = query.limit.unwrap_or(10);
    info!("Getting {} recent expense transactions for user {}", limit, user_id);

    // Create cache key
    let cache_key = format!("recent_expense_transactions:{}:{}", user_id, limit);

    // Try to get from cache first (shorter cache time for recent data)
    if let Some(cached_response) = cache.get::<RecentTransactionsResponse>(&cache_key).await {
        info!("Returning cached recent expense transactions for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_recent_transactions(user_id, query).await?;

    // Cache the response for 5 minutes (shorter for recent data)
    if !cache.set(&cache_key, &response, Some(300)).await {
        error!("Failed to cache recent expense transactions");
    }

    Ok(Json(response))
}