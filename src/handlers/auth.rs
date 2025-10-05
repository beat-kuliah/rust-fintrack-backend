use axum::{extract::State, response::IntoResponse};

use crate::models::{LoginRequest, RegisterRequest};
use crate::services::AuthService;
use crate::repositories::PostgresAuthRepository;
use crate::utils::{AppError, ValidatedJson, success_response, created_response};

pub async fn register(
    State(auth_service): State<AuthService<PostgresAuthRepository>>,
    ValidatedJson(request): ValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = auth_service.register(request).await?;
    Ok(created_response(response))
}

pub async fn login(
    State(auth_service): State<AuthService<PostgresAuthRepository>>,
    ValidatedJson(request): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = auth_service.login(request).await?;
    Ok(success_response(response))
}