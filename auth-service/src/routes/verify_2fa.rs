use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode, TwoFACodeStore};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[axum::debug_handler]
pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::IncorrectCredentials)?;

    let (stored_login_attempt_id, stored_two_fa_code) = state
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    if stored_login_attempt_id.as_ref() != request.login_attempt_id
        || stored_two_fa_code.as_ref() != request.two_fa_code
    {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    pub login_attempt_id: String,
    pub two_fa_code: String,
}
