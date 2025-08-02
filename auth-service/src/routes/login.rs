use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Handles user login requests
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Parse and validate email using domain type
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Parse and validate password using domain type
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let _user = {
        let user_store = state.user_store.read().await;

        // Validate user with parsed domain types
        user_store
            .validate_user(&email, &password)
            .await
            .map_err(|_| AuthAPIError::IncorrectCredentials)?;

        // Get the user
        user_store
            .get_user(&email)
            .await
            .map_err(|_| AuthAPIError::IncorrectCredentials)?
    };

    Ok(StatusCode::OK.into_response())
}
