use axum::{
    extract::{Path, State, Extension},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{CreatePocketRequest, UpdatePocketRequest};
use crate::services::PocketService;
use crate::repositories::PostgresPocketRepository;
use crate::utils::{AppError, ValidatedJson, success_response, created_response, no_content_response, CacheService, user_pockets_cache_key};

pub async fn get_pockets(
    auth_user: AuthUser,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
    Extension(cache_service): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = user_pockets_cache_key(&auth_user.id);
    
    // Try to get from cache first
    if let Some(cached_pockets) = cache_service.get(&cache_key).await {
        return Ok(success_response(cached_pockets));
    }
    
    // If not in cache, get from database
    let pockets = pocket_service.get_user_pockets(auth_user.id).await?;
    
    // Cache the result for 3 minutes
    cache_service.set(&cache_key, &pockets, Some(180)).await;
    
    Ok(success_response(pockets))
}

pub async fn get_pocket_by_id(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let pocket = pocket_service.get_pocket_by_id(id, auth_user.id).await?;
    Ok(success_response(pocket))
}

pub async fn create_pocket(
    auth_user: AuthUser,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
    Extension(cache_service): Extension<CacheService>,
    ValidatedJson(create_request): ValidatedJson<CreatePocketRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pocket = pocket_service.create_pocket(auth_user.id, create_request).await?;
    
    // Invalidate user pockets cache after creation
    let cache_key = user_pockets_cache_key(&auth_user.id);
    cache_service.delete(&cache_key).await;
    
    Ok(created_response(pocket))
}

pub async fn update_pocket(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
    Extension(cache_service): Extension<CacheService>,
    ValidatedJson(update_request): ValidatedJson<UpdatePocketRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pocket = pocket_service.update_pocket(id, auth_user.id, update_request).await?;
    
    // Invalidate user pockets cache after update
    let cache_key = user_pockets_cache_key(&auth_user.id);
    cache_service.delete(&cache_key).await;
    
    Ok(success_response(pocket))
}

pub async fn delete_pocket(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
    Extension(cache_service): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    pocket_service.delete_pocket(id, auth_user.id).await?;
    
    // Invalidate user pockets cache after deletion
    let cache_key = user_pockets_cache_key(&auth_user.id);
    cache_service.delete(&cache_key).await;
    
    Ok(no_content_response())
}