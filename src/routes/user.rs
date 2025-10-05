use axum::{
    middleware,
    routing::{get, patch},
    Router,
};

use crate::handlers::user::{get_me, list_users, update_hide_balance, update_name};
use crate::middleware::auth::auth_middleware;
use crate::repositories::PostgresUserRepository;
use crate::services::UserService;

pub fn user_routes() -> Router<UserService<PostgresUserRepository>> {
    Router::new()
        .route("/me", get(get_me))
        .route("/name", patch(update_name))
        .route("/hide-balance", patch(update_hide_balance))
        .route("/", get(list_users))
        .route_layer(middleware::from_fn(auth_middleware))
}