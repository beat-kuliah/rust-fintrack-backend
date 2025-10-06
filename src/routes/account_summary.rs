use axum::{
    routing::get,
    Router,
};

use crate::handlers::account_summary::get_account_summary;
use crate::middleware::auth_middleware;
use crate::services::AccountSummaryService;
use crate::repositories::{PostgresPocketRepository, PostgresTransactionRepository};

pub fn account_summary_routes() -> Router<AccountSummaryService<PostgresPocketRepository, PostgresTransactionRepository>> {
    Router::new()
        .route("/account-summary", get(get_account_summary))
        .layer(axum::middleware::from_fn(auth_middleware))
}