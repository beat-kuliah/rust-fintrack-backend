pub mod config;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod utils;

// Config exports
pub use config::*;

// Handler exports
pub use handlers::auth::{register, login};
pub use handlers::user::{get_me, update_name, update_hide_balance, list_users};
pub use handlers::pocket::{get_pockets, get_pocket_by_id, create_pocket, update_pocket, delete_pocket};

// Middleware exports
pub use middleware::auth::{AuthUser, auth_middleware};
pub use middleware::cors::*;
pub use middleware::logging::*;

// Model exports
pub use models::auth::*;
pub use models::user::*;
pub use models::pocket::*;

// Repository exports
pub use repositories::*;

// Route exports
pub use routes::auth::auth_routes;
pub use routes::user::user_routes;
pub use routes::pocket::pocket_routes;

// Service exports
pub use services::auth::AuthService;
pub use services::user::UserService;
pub use services::pocket::PocketService;

// Utility exports
pub use utils::*;