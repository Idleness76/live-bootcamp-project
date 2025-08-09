use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email},
    utils::generate_auth_cookie,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let user = state
        .user_store
        .read()
        .await
        .authenticate_user(&request.email, &request.password)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    let (jar, resp) = match user.requires_2fa {
        true => handle_2fa(&user.email, jar).await?,
        false => handle_no_2fa(&user.email, jar).await?,
    };
    Ok((jar, resp.into_response()))
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

async fn handle_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, axum::response::Response), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);
    let resp = TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: "123456".to_string(),
    };
    Ok((
        updated_jar,
        (
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(resp)),
        )
            .into_response(),
    ))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, axum::response::Response), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);
    Ok((
        updated_jar,
        (StatusCode::OK, Json(LoginResponse::RegularAuth)).into_response(),
    ))
}
