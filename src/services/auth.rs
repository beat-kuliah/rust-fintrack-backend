use bcrypt::{hash, verify, DEFAULT_COST};

use crate::config::JwtConfig;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest};
use crate::repositories::AuthRepository;
use crate::utils::AppError;

#[derive(Clone)]
pub struct AuthService<R: AuthRepository> {
    repository: R,
    jwt_config: JwtConfig,
}

impl<R: AuthRepository> AuthService<R> {
    pub fn new(repository: R, jwt_config: JwtConfig) -> Self {
        Self {
            repository,
            jwt_config,
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        // Hash password
        let hashed_password = hash(&request.password, DEFAULT_COST)
            .map_err(|e| AppError::InternalServerError(format!("Password hashing failed: {}", e)))?;

        // Create user
        let user = self.repository.create_user(&request, hashed_password).await?;

        // Generate token
        let token = self.jwt_config.create_token(user.id, user.email.clone())?;

        Ok(AuthResponse {
            token,
            user: user.to_response(),
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        // Find user by email
        let user = self
            .repository
            .find_user_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = verify(&request.password, &user.password)
            .map_err(|e| AppError::InternalServerError(format!("Password verification failed: {}", e)))?;

        if !is_valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        // Generate token
        let token = self.jwt_config.create_token(user.id, user.email.clone())?;

        Ok(AuthResponse {
            token,
            user: user.to_response(),
        })
    }
}