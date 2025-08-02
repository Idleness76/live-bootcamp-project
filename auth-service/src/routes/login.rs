use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Handles user login requests
/// Currently returns 200 OK as a placeholder implementation
pub async fn login(Json(_request): Json<LoginRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
