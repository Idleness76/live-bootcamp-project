use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::AuthAPIError,
    utils::{validate_token, JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Check if the JWT cookie exists in the jar.
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value().to_owned();

    match validate_token(&token).await {
        Ok(_claims) => (jar, Ok(StatusCode::OK)),
        Err(_) => (jar, Err(AuthAPIError::InvalidToken)),
    }
}
