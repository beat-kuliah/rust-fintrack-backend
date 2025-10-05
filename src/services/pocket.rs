use uuid::Uuid;

use crate::models::{PocketResponse, CreatePocketRequest, UpdatePocketRequest};
use crate::repositories::PocketRepository;
use crate::utils::AppError;

#[derive(Clone)]
pub struct PocketService<R: PocketRepository> {
    repository: R,
}

impl<R: PocketRepository> PocketService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_pocket_by_id(&self, id: Uuid, user_id: Uuid) -> Result<PocketResponse, AppError> {
        let pocket = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Pocket not found".to_string()))?;

        // Check ownership
        if pocket.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(pocket.to_response())
    }

    pub async fn get_user_pockets(&self, user_id: Uuid) -> Result<Vec<PocketResponse>, AppError> {
        let pockets = self.repository.find_by_user_id(user_id).await?;
        let pocket_responses = pockets.into_iter().map(|pocket| pocket.to_response()).collect();
        Ok(pocket_responses)
    }

    pub async fn create_pocket(&self, user_id: Uuid, request: CreatePocketRequest) -> Result<PocketResponse, AppError> {
        let pocket = self.repository.create(user_id, &request).await?;
        Ok(pocket.to_response())
    }

    pub async fn update_pocket(&self, id: Uuid, user_id: Uuid, request: UpdatePocketRequest) -> Result<PocketResponse, AppError> {
        let pocket = self.repository.update(id, user_id, &request).await?;
        Ok(pocket.to_response())
    }

    pub async fn delete_pocket(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.repository.delete(id, user_id).await
    }
}