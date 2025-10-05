use axum::{
    extract::Extension,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber;

use rust_fintrack_backend::{
    config::{create_pool, AppConfig, JwtConfig},
    middleware::{cors_layer, logging_layer},
    repositories::{PostgresAuthRepository, PostgresPocketRepository, PostgresUserRepository},
    routes::{auth_routes, pocket_routes, user_routes},
    services::{AuthService, PocketService, UserService},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Starting server with config: {:?}", config);

    // Create database connection pool
    let pool = create_pool().await?;
    info!("Database connection pool created");

    // Create JWT config
    let jwt_config = JwtConfig::new(&config.jwt_secret);

    // Create repositories
    let auth_repository = PostgresAuthRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let pocket_repository = PostgresPocketRepository::new(pool.clone());

    // Create services
    let auth_service = AuthService::new(auth_repository, jwt_config.clone());
    let user_service = UserService::new(user_repository);
    let pocket_service = PocketService::new(pocket_repository);

    // Build application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes().with_state(auth_service))
        .nest("/users", user_routes().with_state(user_service))
        .nest("/pockets", pocket_routes().with_state(pocket_service))
        .layer(cors_layer())
        .layer(logging_layer())
        .layer(Extension(pool))
        .layer(Extension(jwt_config));

    // Start server
    let listener = TcpListener::bind(&config.server_address()).await?;
    info!("Server listening on {}", config.server_address());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ok",
        "message": "Rust Fintrack Backend is running"
    })))
}