use axum::{
    body::Body,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::utils::{validation_error, AppError};

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| AppError::BadRequest("Invalid JSON".to_string()))?;

        value
            .validate()
            .map_err(validation_error)?;

        Ok(ValidatedJson(value))
    }
}

// Helper function to validate data manually
pub fn validate_data<T: Validate>(data: &T) -> Result<(), AppError> {
    data.validate().map_err(validation_error)
}