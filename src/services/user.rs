use uuid::Uuid;

use crate::models::{UserResponse, UpdateUserNameRequest, UpdateHideBalanceRequest};
use crate::repositories::UserRepository;
use crate::utils::AppError;

#[derive(Clone)]
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.to_response())
    }

    pub async fn update_user_name(&self, id: Uuid, request: UpdateUserNameRequest) -> Result<UserResponse, AppError> {
        let user = self.repository.update_name(id, &request.name).await?;
        Ok(user.to_response())
    }

    pub async fn update_hide_balance(&self, id: Uuid, request: UpdateHideBalanceRequest) -> Result<UserResponse, AppError> {
        let user = self.repository.update_hide_balance(id, request.hide_balance).await?;
        Ok(user.to_response())
    }

    pub async fn list_users(&self) -> Result<Vec<UserResponse>, AppError> {
        let users = self.repository.list_all().await?;
        let user_responses = users.into_iter().map(|user| user.to_response()).collect();
        Ok(user_responses)
    }
}