use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::models::Claims;
use crate::utils::AppError;

#[derive(Clone)]
pub struct JwtConfig {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub validation: Validation,
}

impl JwtConfig {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            validation: Validation::default(),
        }
    }

    pub fn create_token(&self, user_id: Uuid, email: String) -> Result<String, AppError> {
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims::new(user_id, email, exp);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalServerError(format!("Token creation failed: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))
    }
}