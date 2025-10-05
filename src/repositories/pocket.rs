use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{Pocket, CreatePocketRequest, UpdatePocketRequest};
use crate::utils::AppError;

#[async_trait::async_trait]
pub trait PocketRepository: Clone + Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pocket>, AppError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError>;
    async fn create(&self, user_id: Uuid, request: &CreatePocketRequest) -> Result<Pocket, AppError>;
    async fn update(&self, id: Uuid, user_id: Uuid, request: &UpdatePocketRequest) -> Result<Pocket, AppError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct PostgresPocketRepository {
    pool: PgPool,
}

impl PostgresPocketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PocketRepository for PostgresPocketRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pocket>, AppError> {
        let row = sqlx::query(
            "SELECT id, user_id, name, emoji, balance, created_at, updated_at 
             FROM pockets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let pocket = Pocket {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            emoji: row.get("emoji"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
                Ok(Some(pocket))
            }
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Pocket>, AppError> {
        let rows = sqlx::query(
            "SELECT id, user_id, name, emoji, balance, created_at, updated_at 
             FROM pockets WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let pockets = rows.into_iter().map(|row| Pocket {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            emoji: row.get("emoji"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(pockets)
    }

    async fn create(&self, user_id: Uuid, request: &CreatePocketRequest) -> Result<Pocket, AppError> {
        let pocket_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = sqlx::query(
            "INSERT INTO pockets (id, user_id, name, emoji, balance, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING id, user_id, name, emoji, balance, created_at, updated_at"
        )
        .bind(pocket_id)
        .bind(user_id)
        .bind(&request.name)
        .bind(&request.emoji)
        .bind(Decimal::ZERO)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let pocket = Pocket {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            emoji: row.get("emoji"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(pocket)
    }

    async fn update(&self, id: Uuid, user_id: Uuid, request: &UpdatePocketRequest) -> Result<Pocket, AppError> {
        // First check if pocket exists and belongs to user
        let existing_pocket = self.find_by_id(id).await?;
        let pocket = existing_pocket.ok_or_else(|| AppError::NotFound("Pocket not found".to_string()))?;
        
        if pocket.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        // Simple update with all fields
        let row = sqlx::query(
            "UPDATE pockets SET name = COALESCE($1, name), emoji = COALESCE($2, emoji), updated_at = NOW()
             WHERE id = $3 AND user_id = $4
             RETURNING id, user_id, name, emoji, balance, created_at, updated_at"
        )
        .bind(request.name.as_ref())
        .bind(request.emoji.as_ref())
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let pocket = Pocket {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            emoji: row.get("emoji"),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(pocket)
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM pockets WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Pocket not found or access denied".to_string()));
        }

        Ok(())
    }
}