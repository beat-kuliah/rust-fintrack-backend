use axum::{
    extract::{State, Extension},
    response::IntoResponse,
};

use crate::middleware::AuthUser;
use crate::services::AccountSummaryService;
use crate::repositories::{PostgresPocketRepository, PostgresTransactionRepository};
use crate::utils::{AppError, success_response, CacheService};

pub async fn get_account_summary(
    State(service): State<AccountSummaryService<PostgresPocketRepository, PostgresTransactionRepository>>,
    Extension(auth_user): Extension<AuthUser>,
    Extension(cache): Extension<CacheService>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("account_summary:{}", auth_user.id);

    if let Some(cached_response) = cache.get::<crate::models::AccountSummaryResponse>(&cache_key).await {
        return Ok(success_response(cached_response));
    }

    let response = service.get_account_summary(auth_user.id).await?;

    // Cache the response for 5 minutes
    let _ = cache.set(&cache_key, &response, Some(300)).await;

    Ok(success_response(response))
}