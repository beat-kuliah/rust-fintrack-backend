use axum::{
    extract::{Path, Query, State, Extension},
    response::IntoResponse,
};

use crate::middleware::AuthUser;
use crate::models::{CreateBudgetRequest, UpdateBudgetRequest, ListBudgetsQuery};
use crate::services::BudgetService;
use crate::repositories::PostgresBudgetRepository;
use crate::utils::{AppError, ValidatedJson, success_response, created_response, no_content_response, CacheService};

pub async fn get_budgets(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListBudgetsQuery>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    // Try to get from cache first
    let cache_key = format!(
        "budgets:{}:page:{}:limit:{}:category:{}:period_type:{}:active:{}",
        auth_user.id,
        query.page.unwrap_or(1),
        query.limit.unwrap_or(20),
        query.category.as_deref().unwrap_or(""),
        query.period_type.as_deref().unwrap_or(""),
        query.is_active.map(|b| b.to_string()).unwrap_or_default()
    );

    if let Some(cached_response) = cache.get::<crate::models::ListBudgetsResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.list_budgets(auth_user.id, query).await?;

    // Cache the response for 5 minutes
    let _ = cache.set(&cache_key, &response, Some(300)).await;

    Ok(success_response(response))
}

pub async fn get_budget_by_id(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.get_budget_by_id(id, auth_user.id).await?;
    Ok(success_response(response))
}

pub async fn create_budget(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    ValidatedJson(request): ValidatedJson<CreateBudgetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.create_budget(auth_user.id, request).await?;

    // Invalidate budget cache
    let _ = cache.delete(&format!("budgets:{}:*", auth_user.id)).await;

    Ok(created_response(response))
}

pub async fn update_budget(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    Path(id): Path<i64>,
    ValidatedJson(request): ValidatedJson<UpdateBudgetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.update_budget(id, auth_user.id, request).await?;

    // Invalidate budget cache
    let _ = cache.delete(&format!("budgets:{}:*", auth_user.id)).await;

    Ok(success_response(response))
}

pub async fn delete_budget(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_budget(id, auth_user.id).await?;

    // Invalidate budget cache
    let _ = cache.delete(&format!("budgets:{}:*", auth_user.id)).await;

    Ok(no_content_response())
}

pub async fn get_budget_summary(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("budget_summary:{}", auth_user.id);

    if let Some(cached_response) = cache.get::<crate::models::BudgetSummaryResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.get_budget_summary(auth_user.id).await?;

    // Cache the response for 10 minutes
    let _ = cache.set(&cache_key, &response, Some(600)).await;

    Ok(success_response(response))
}

pub async fn get_budget_performance(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("budget_performance:{}", auth_user.id);

    if let Some(cached_response) = cache.get::<crate::models::BudgetPerformanceResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.get_budget_performance(auth_user.id).await?;

    // Cache the response for 5 minutes (performance data changes more frequently)
    let _ = cache.set(&cache_key, &response, Some(300)).await;

    Ok(success_response(response))
}

pub async fn get_budget_categories(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("budget_categories:{}", auth_user.id);

    if let Some(cached_response) = cache.get::<Vec<String>>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.get_budget_categories(auth_user.id).await?;

    // Cache the response for 15 minutes
    let _ = cache.set(&cache_key, &response, Some(900)).await;

    Ok(success_response(response))
}

pub async fn get_budget_suggestions(
    State(service): State<BudgetService<PostgresBudgetRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("budget_suggestions:{}", auth_user.id);

    if let Some(cached_response) = cache.get::<crate::models::BudgetSuggestionsResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.get_budget_suggestions(auth_user.id).await?;

    // Cache the response for 30 minutes (suggestions don't change frequently)
    let _ = cache.set(&cache_key, &response, Some(1800)).await;

    Ok(success_response(response))
}