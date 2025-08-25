use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User, UserStoreError},
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String,
}

//#[axum::debug_handler]
#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<(StatusCode, impl IntoResponse), AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    state
        .user_store
        .write()
        .await
        .add_user(User::new(email, password, request.requires_2fa))
        .await
        .map_err(|e| match e {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            _ => AuthAPIError::UnexpectedError,
        })?;
    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            message: "User created successfully!".to_string(),
        }),
    ))
}
