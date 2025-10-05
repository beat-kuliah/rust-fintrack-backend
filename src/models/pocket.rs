use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pocket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub emoji: String,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PocketResponse {
    pub id: Uuid,
    pub name: String,
    pub emoji: String,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePocketRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 10, message = "Emoji must be between 1 and 10 characters"))]
    pub emoji: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePocketRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 10, message = "Emoji must be between 1 and 10 characters"))]
    pub emoji: Option<String>,
}

impl From<Pocket> for PocketResponse {
    fn from(pocket: Pocket) -> Self {
        Self {
            id: pocket.id,
            name: pocket.name,
            emoji: pocket.emoji,
            balance: pocket.balance,
            created_at: pocket.created_at,
            updated_at: pocket.updated_at,
        }
    }
}

impl Pocket {
    pub fn to_response(self) -> PocketResponse {
        PocketResponse::from(self)
    }
}