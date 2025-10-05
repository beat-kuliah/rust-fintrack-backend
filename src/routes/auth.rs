use axum::{routing::post, Router};

use crate::handlers::auth::{login, register};
use crate::services::AuthService;
use crate::repositories::PostgresAuthRepository;

pub fn auth_routes() -> Router<AuthService<PostgresAuthRepository>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}