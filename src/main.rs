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
    repositories::{PostgresAuthRepository, PostgresPocketRepository, PostgresUserRepository, PostgresTransactionRepository, PostgresBudgetRepository},
    routes::{auth_routes, pocket_routes, user_routes, transaction_routes, budget_routes, account_summary_routes, expense_analytics_routes, income_analytics_routes},
    services::{AuthService, PocketService, UserService, TransactionService, BudgetService, AccountSummaryService, ExpenseAnalyticsService, IncomeAnalyticsService},
    utils::CacheService,
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

    // Create Redis cache service
    let cache_service = CacheService::new(&config.redis).await;

    // Create JWT config
    let jwt_config = JwtConfig::new(&config.jwt_secret);

    // Create repositories
    let auth_repository = PostgresAuthRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());
    let pocket_repository = PostgresPocketRepository::new(pool.clone());
    let transaction_repository = PostgresTransactionRepository::new(pool.clone());
    let budget_repository = PostgresBudgetRepository::new(pool.clone());

    // Create services
    let auth_service = AuthService::new(auth_repository, jwt_config.clone());
    let user_service = UserService::new(user_repository);
    let pocket_service = PocketService::new(pocket_repository.clone());
    let transaction_service = TransactionService::new(transaction_repository.clone());
    let budget_service = BudgetService::new(budget_repository);
    let account_summary_service = AccountSummaryService::new(pocket_repository.clone(), transaction_repository.clone());
    let expense_analytics_service = ExpenseAnalyticsService::new(transaction_repository.clone());
    let income_analytics_service = IncomeAnalyticsService::new(transaction_repository);

    // Build application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes().with_state(auth_service))
        .nest("/users", user_routes().with_state(user_service))
        .nest("/pockets", pocket_routes().with_state(pocket_service))
        .merge(transaction_routes().with_state(transaction_service))
        .merge(budget_routes().with_state(budget_service))
        .merge(account_summary_routes().with_state(account_summary_service))
        .merge(expense_analytics_routes().with_state(expense_analytics_service))
        .merge(income_analytics_routes().with_state(income_analytics_service))
        .layer(cors_layer())
        .layer(logging_layer())
        .layer(Extension(pool))
        .layer(Extension(jwt_config))
        .layer(Extension(cache_service));

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