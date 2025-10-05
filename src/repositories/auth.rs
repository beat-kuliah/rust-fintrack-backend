use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{User, RegisterRequest};
use crate::utils::AppError;

#[async_trait::async_trait]
pub trait AuthRepository: Clone + Send + Sync {
    async fn create_user(&self, request: &RegisterRequest, hashed_password: String) -> Result<User, AppError>;
    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
}

#[derive(Clone)]
pub struct PostgresAuthRepository {
    pool: PgPool,
}

impl PostgresAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AuthRepository for PostgresAuthRepository {
    async fn create_user(&self, request: &RegisterRequest, hashed_password: String) -> Result<User, AppError> {
        let user_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = sqlx::query(
            "INSERT INTO users (id, name, email, password, hide_balance, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, name, email, password, hide_balance, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&request.name)
        .bind(&request.email)
        .bind(&hashed_password)
        .bind(false) // default hide_balance to false
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::Conflict("Email already exists".to_string())
            } else {
                AppError::DatabaseError(e.to_string())
            }
        })?;

        let user = User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            hide_balance: row.get("hide_balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, email, password, hide_balance, created_at, updated_at 
             FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    name: row.get("name"),
                    email: row.get("email"),
                    password: row.get("password"),
                    hide_balance: row.get("hide_balance"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }
}