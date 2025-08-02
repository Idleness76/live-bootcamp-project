use crate::domain::{AuthAPIError, Email, Password};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Handles user login requests
pub async fn login(Json(request): Json<LoginRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    // Parse and validate email using domain type
    let _email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Parse and validate password using domain type
    let _password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // TODO: Implement actual authentication logic
    Ok(StatusCode::OK.into_response())
}
