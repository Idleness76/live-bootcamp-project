use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email};
use crate::utils::generate_auth_cookie;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[axum::debug_handler]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<axum::response::Response, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let (stored_login_attempt_id, stored_two_fa_code) =
        match state.two_fa_code_store.read().await.get_code(&email).await {
            Ok(codes) => codes,
            Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
        };

    if stored_login_attempt_id.as_ref() != request.login_attempt_id
        || stored_two_fa_code.as_ref() != request.two_fa_code
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };
    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    pub login_attempt_id: String,
    pub two_fa_code: String,
}
