use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[axum::debug_handler]
pub async fn verify_2fa(Json(request): Json<Verify2FARequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    pub login_attempt_id: String,
    pub two_fa_code: String,
}
