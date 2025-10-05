use axum::{
    middleware,
    routing::get,
    Router,
};

use crate::handlers::pocket::{
    create_pocket, delete_pocket, get_pocket_by_id, get_pockets, update_pocket,
};
use crate::middleware::auth::auth_middleware;
use crate::repositories::PostgresPocketRepository;
use crate::services::PocketService;

pub fn pocket_routes() -> Router<PocketService<PostgresPocketRepository>> {
    Router::new()
        .route("/", get(get_pockets).post(create_pocket))
        .route("/{id}", get(get_pocket_by_id).put(update_pocket).delete(delete_pocket))
        .route_layer(middleware::from_fn(auth_middleware))
}