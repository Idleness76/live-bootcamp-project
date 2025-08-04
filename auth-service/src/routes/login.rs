use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::generate_auth_cookie,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

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
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user = state
        .user_store
        .read()
        .await
        .authenticate_user(&email, &password)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;
    let auth_cookie =
        generate_auth_cookie(&user.email).map_err(|_| AuthAPIError::UnexpectedError)?;
    Ok((jar.add(auth_cookie), StatusCode::OK))
}
