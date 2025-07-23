use axum::{http::StatusCode, response::IntoResponse};

/// Handles user logout requests
/// Currently returns 200 OK as a placeholder implementation
pub async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
