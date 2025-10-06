use axum::{
    routing::{get},
    Router,
};

use crate::handlers::transaction::{
    get_transactions, get_transaction_by_id, create_transaction, 
    update_transaction, delete_transaction
};
use crate::middleware::auth_middleware;
use crate::services::TransactionService;
use crate::repositories::PostgresTransactionRepository;

pub fn transaction_routes() -> Router<TransactionService<PostgresTransactionRepository>> {
    Router::new()
        .route("/transactions", get(get_transactions).post(create_transaction))
        .route("/transactions/{id}", get(get_transaction_by_id).put(update_transaction).delete(delete_transaction))
        .layer(axum::middleware::from_fn(auth_middleware))
}