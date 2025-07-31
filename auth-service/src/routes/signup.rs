use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User, UserStoreError},
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Parse and validate email using domain type
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Parse and validate password using domain type
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = User::new(email, password, request.requires_2fa);

    {
        let mut user_store = state.user_store.write().await;

        // Handle add_user result properly instead of using unwrap
        match user_store.add_user(user).await {
            Ok(()) => {
                let response = Json(SignupResponse {
                    message: "User created successfully!".to_string(),
                });
                Ok((StatusCode::CREATED, response))
            }
            Err(UserStoreError::UserAlreadyExists) => Err(AuthAPIError::UserAlreadyExists),
            Err(_) => Err(AuthAPIError::UnexpectedError),
        }
    }
}

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
