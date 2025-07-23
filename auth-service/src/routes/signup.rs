use axum::{http::StatusCode, response::IntoResponse};

/// Handles user registration requests
/// Currently returns 200 OK as a placeholder implementation
pub async fn signup() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
