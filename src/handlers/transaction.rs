use axum::{
    extract::{Path, Query, State, Extension},
    response::IntoResponse,
};

use crate::middleware::AuthUser;
use crate::models::{CreateTransactionRequest, UpdateTransactionRequest, ListTransactionsQuery};
use crate::services::TransactionService;
use crate::repositories::PostgresTransactionRepository;
use crate::utils::{AppError, ValidatedJson, success_response, created_response, no_content_response, CacheService};

pub async fn get_transactions(
    State(service): State<TransactionService<PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListTransactionsQuery>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    // Try to get from cache first
    let cache_key = format!(
        "transactions:{}:page:{}:limit:{}:category:{}:from:{}:to:{}:type:{}",
        auth_user.id,
        query.page.unwrap_or(1),
        query.limit.unwrap_or(20),
        query.category.as_deref().unwrap_or(""),
        query.from_date.as_deref().unwrap_or(""),
        query.to_date.as_deref().unwrap_or(""),
        query.transaction_type.as_deref().unwrap_or("")
    );

    if let Some(cached_response) = cache.get::<crate::models::ListTransactionsResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.list_transactions(auth_user.id, query).await?;

    // Cache the response for 5 minutes
    let _ = cache.set(&cache_key, &response, Some(300)).await;

    Ok(success_response(response))
}

pub async fn get_transaction_by_id(
    State(service): State<TransactionService<PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.get_transaction_by_id(id, auth_user.id).await?;
    Ok(success_response(response))
}

pub async fn create_transaction(
    State(service): State<TransactionService<PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    ValidatedJson(request): ValidatedJson<CreateTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.create_transaction(auth_user.id, request).await?;

    // Invalidate user cache
    let _ = cache.delete(&format!("user:{}", auth_user.id)).await;
    let _ = cache.delete(&format!("user:{}:pockets", auth_user.id)).await;

    Ok(created_response(response))
}

pub async fn update_transaction(
    State(service): State<TransactionService<PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    Path(id): Path<i64>,
    ValidatedJson(request): ValidatedJson<UpdateTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = service.update_transaction(id, auth_user.id, request).await?;

    // Invalidate user cache
    let _ = cache.delete(&format!("user:{}", auth_user.id)).await;
    let _ = cache.delete(&format!("user:{}:pockets", auth_user.id)).await;

    Ok(success_response(response))
}

pub async fn delete_transaction(
    State(service): State<TransactionService<PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    service.delete_transaction(id, auth_user.id).await?;

    // Invalidate user cache
    let _ = cache.delete(&format!("user:{}", auth_user.id)).await;
    let _ = cache.delete(&format!("user:{}:pockets", auth_user.id)).await;

    Ok(no_content_response())
}