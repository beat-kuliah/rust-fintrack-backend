use axum::{extract::{State, Extension}, Json, response::IntoResponse};

use crate::middleware::AuthUser;
use crate::models::{UpdateUserNameRequest, UpdateHideBalanceRequest};
use crate::services::UserService;
use crate::repositories::PostgresUserRepository;
use crate::utils::{AppError, ValidatedJson, success_response, CacheService, user_cache_key};

pub async fn get_me(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
    Extension(cache_service): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = user_cache_key(&auth_user.id);
    
    // Try to get from cache first
    if let Some(cached_user) = cache_service.get(&cache_key).await {
        return Ok(success_response(cached_user));
    }
    
    // If not in cache, get from database
    let user = user_service.get_user_by_id(auth_user.id).await?;
    
    // Cache the result for 5 minutes
    cache_service.set(&cache_key, &user, Some(300)).await;
    
    Ok(success_response(user))
}

pub async fn update_name(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
    Extension(cache_service): Extension<CacheService>,
    ValidatedJson(update_request): ValidatedJson<UpdateUserNameRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.update_user_name(auth_user.id, update_request).await?;
    
    // Invalidate user cache after update
    let cache_key = user_cache_key(&auth_user.id);
    cache_service.delete(&cache_key).await;
    
    Ok(success_response(user))
}

pub async fn update_hide_balance(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
    Extension(cache_service): Extension<CacheService>,
    Json(update_request): Json<UpdateHideBalanceRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.update_hide_balance(auth_user.id, update_request).await?;
    
    // Invalidate user cache after update
    let cache_key = user_cache_key(&auth_user.id);
    cache_service.delete(&cache_key).await;
    
    Ok(success_response(user))
}

pub async fn list_users(
    State(user_service): State<UserService<PostgresUserRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let users = user_service.list_users().await?;
    Ok(success_response(users))
}