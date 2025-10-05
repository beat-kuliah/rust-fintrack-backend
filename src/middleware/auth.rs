use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::utils::AppError;

#[derive(Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
    }
}

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract JWT config from request extensions
    let jwt_config = request
        .extensions()
        .get::<JwtConfig>()
        .ok_or_else(|| AppError::InternalServerError("JWT config not found".to_string()))?
        .clone();

    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization header format".to_string()))?;

    let claims = jwt_config.verify_token(token)?;

    let auth_user = AuthUser {
        id: claims.sub,
        email: claims.email,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

// Extension trait to easily extract AuthUser from request
pub trait AuthUserExt {
    fn auth_user(&self) -> Result<&AuthUser, AppError>;
}

impl AuthUserExt for Request {
    fn auth_user(&self) -> Result<&AuthUser, AppError> {
        self.extensions()
            .get::<AuthUser>()
            .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
    }
}