use crate::{app_state::AppState, domain::AuthAPIError, utils::validate_token};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(app_state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<(StatusCode, impl IntoResponse), AuthAPIError> {
    let banned_store = app_state.banned_token_store.read().await;
    validate_token(&request.token, &*banned_store)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;
    Ok((StatusCode::OK, Json("Token valid".to_string())))
}
