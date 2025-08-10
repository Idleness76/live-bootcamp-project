use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email};
use crate::utils::generate_auth_cookie;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
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

    state
        .two_fa_code_store
        .write()
        .await
        .remove_code(&email)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;

    let updated_jar = jar.add(auth_cookie);

    Ok((updated_jar, StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
