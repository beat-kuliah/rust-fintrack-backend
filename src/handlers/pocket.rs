use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{CreatePocketRequest, UpdatePocketRequest};
use crate::services::PocketService;
use crate::repositories::PostgresPocketRepository;
use crate::utils::{AppError, ValidatedJson, success_response, created_response, no_content_response};

pub async fn get_pockets(
    auth_user: AuthUser,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let pockets = pocket_service.get_user_pockets(auth_user.id).await?;
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
    ValidatedJson(create_request): ValidatedJson<CreatePocketRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pocket = pocket_service.create_pocket(auth_user.id, create_request).await?;
    Ok(created_response(pocket))
}

pub async fn update_pocket(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
    ValidatedJson(update_request): ValidatedJson<UpdatePocketRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pocket = pocket_service.update_pocket(id, auth_user.id, update_request).await?;
    Ok(success_response(pocket))
}

pub async fn delete_pocket(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(pocket_service): State<PocketService<PostgresPocketRepository>>,
) -> Result<impl IntoResponse, AppError> {
    pocket_service.delete_pocket(id, auth_user.id).await?;
    Ok(no_content_response())
}