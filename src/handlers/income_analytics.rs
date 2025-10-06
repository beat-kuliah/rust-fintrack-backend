use axum::{
    extract::{Query, State},
    response::Json,
    Extension,
};
use tracing::{error, info};
use validator::Validate;

use crate::models::{
    IncomeDateRangeQuery, IncomeRecentTransactionsQuery, IncomeSummaryResponse,
    IncomeCategorySummaryResponse, IncomeTrendResponse, RecentIncomeTransactionsResponse,
};
use crate::services::IncomeAnalyticsService;
use crate::repositories::PostgresTransactionRepository;
use crate::utils::{AppError, CacheService};

pub async fn get_income_summary(
    State(service): State<IncomeAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<IncomeDateRangeQuery>,
) -> Result<Json<IncomeSummaryResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting income summary for user {}", user_id);

    // Create cache key
    let cache_key = format!("income_summary:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<IncomeSummaryResponse>(&cache_key).await {
        info!("Returning cached income summary for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_income_summary(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache income summary");
    }

    Ok(Json(response))
}

pub async fn get_income_category_summary(
    State(service): State<IncomeAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<IncomeDateRangeQuery>,
) -> Result<Json<IncomeCategorySummaryResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting income category summary for user {}", user_id);

    // Create cache key
    let cache_key = format!("income_category_summary:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<IncomeCategorySummaryResponse>(&cache_key).await {
        info!("Returning cached income category summary for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_category_summary(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache income category summary");
    }

    Ok(Json(response))
}

pub async fn get_income_monthly_trend(
    State(service): State<IncomeAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<IncomeDateRangeQuery>,
) -> Result<Json<IncomeTrendResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting income monthly trend for user {}", user_id);

    // Create cache key
    let cache_key = format!("income_monthly_trend:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<IncomeTrendResponse>(&cache_key).await {
        info!("Returning cached income monthly trend for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_monthly_trend(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache income monthly trend");
    }

    Ok(Json(response))
}

pub async fn get_income_daily_trend(
    State(service): State<IncomeAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<IncomeDateRangeQuery>,
) -> Result<Json<IncomeTrendResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    info!("Getting income daily trend for user {}", user_id);

    // Create cache key
    let cache_key = format!("income_daily_trend:{}:{}:{}", user_id, query.from_date, query.to_date);

    // Try to get from cache first
    if let Some(cached_response) = cache.get::<IncomeTrendResponse>(&cache_key).await {
        info!("Returning cached income daily trend for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_daily_trend(user_id, query).await?;

    // Cache the response for 15 minutes
    if !cache.set(&cache_key, &response, Some(900)).await {
        error!("Failed to cache income daily trend");
    }

    Ok(Json(response))
}

pub async fn get_recent_income_transactions(
    State(service): State<IncomeAnalyticsService<PostgresTransactionRepository>>,
    Extension(cache): Extension<CacheService>,
    Extension(user_id): Extension<uuid::Uuid>,
    Query(query): Query<IncomeRecentTransactionsQuery>,
) -> Result<Json<RecentIncomeTransactionsResponse>, AppError> {
    // Validate query parameters
    query.validate().map_err(|e| {
        AppError::BadRequest(format!("Validation error: {}", e))
    })?;

    let limit = query.limit.unwrap_or(10);
    info!("Getting {} recent income transactions for user {}", limit, user_id);

    // Create cache key
    let cache_key = format!("recent_income_transactions:{}:{}", user_id, limit);

    // Try to get from cache first (shorter cache time for recent data)
    if let Some(cached_response) = cache.get::<RecentIncomeTransactionsResponse>(&cache_key).await {
        info!("Returning cached recent income transactions for user {}", user_id);
        return Ok(Json(cached_response));
    }

    // Get from service
    let response = service.get_recent_transactions(user_id, query).await?;

    // Cache the response for 5 minutes (shorter for recent data)
    if !cache.set(&cache_key, &response, Some(300)).await {
        error!("Failed to cache recent income transactions");
    }

    Ok(Json(response))
}