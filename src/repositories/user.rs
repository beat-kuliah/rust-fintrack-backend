use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::User;
use crate::utils::AppError;

#[async_trait::async_trait]
pub trait UserRepository: Clone + Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: User) -> Result<User, AppError>;
    async fn update_name(&self, id: Uuid, name: &str) -> Result<User, AppError>;
    async fn update_hide_balance(&self, id: Uuid, hide_balance: bool) -> Result<User, AppError>;
    async fn list_all(&self) -> Result<Vec<User>, AppError>;
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, email, password, hide_balance, created_at, updated_at
             FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
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

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT id, name, email, password, hide_balance, created_at, updated_at
             FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        
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

    async fn create(&self, user: User) -> Result<User, AppError> {
        let row = sqlx::query(
            "INSERT INTO users (id, name, email, password, hide_balance, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, name, email, password, hide_balance, created_at, updated_at"
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.hide_balance)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .fetch_one(&self.pool)
        .await?;
        
        let created_user = User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            hide_balance: row.get("hide_balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        Ok(created_user)
    }

    async fn update_name(&self, id: Uuid, name: &str) -> Result<User, AppError> {
        let row = sqlx::query(
            "UPDATE users SET name = $1, updated_at = NOW()
             WHERE id = $2
             RETURNING id, name, email, password, hide_balance, created_at, updated_at"
        )
        .bind(name)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        let updated_user = User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            hide_balance: row.get("hide_balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        Ok(updated_user)
    }

    async fn update_hide_balance(&self, id: Uuid, hide_balance: bool) -> Result<User, AppError> {
        let row = sqlx::query(
            "UPDATE users SET hide_balance = $1, updated_at = NOW()
             WHERE id = $2
             RETURNING id, name, email, password, hide_balance, created_at, updated_at"
        )
        .bind(hide_balance)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        let updated_user = User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            hide_balance: row.get("hide_balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        Ok(updated_user)
    }

    async fn list_all(&self) -> Result<Vec<User>, AppError> {
        let rows = sqlx::query(
            "SELECT id, name, email, password, hide_balance, created_at, updated_at
             FROM users
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let users = rows.into_iter().map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            hide_balance: row.get("hide_balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();
        
        Ok(users)
    }
}