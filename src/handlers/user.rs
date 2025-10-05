use axum::{extract::State, Json, response::IntoResponse};

use crate::middleware::AuthUser;
use crate::models::{UpdateUserNameRequest, UpdateHideBalanceRequest};
use crate::services::UserService;
use crate::repositories::PostgresUserRepository;
use crate::utils::{AppError, ValidatedJson, success_response};

pub async fn get_me(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.get_user_by_id(auth_user.id).await?;
    Ok(success_response(user))
}

pub async fn update_name(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
    ValidatedJson(update_request): ValidatedJson<UpdateUserNameRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.update_user_name(auth_user.id, update_request).await?;
    Ok(success_response(user))
}

pub async fn update_hide_balance(
    auth_user: AuthUser,
    State(user_service): State<UserService<PostgresUserRepository>>,
    Json(update_request): Json<UpdateHideBalanceRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_service.update_hide_balance(auth_user.id, update_request).await?;
    Ok(success_response(user))
}

pub async fn list_users(
    State(user_service): State<UserService<PostgresUserRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let users = user_service.list_users().await?;
    Ok(success_response(users))
}