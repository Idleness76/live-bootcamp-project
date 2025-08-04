use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{validate_token, JWT_COOKIE_NAME},
};

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value();

    if validate_token(&token).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    if app_state
        .banned_token_store
        .write()
        .await
        .add_banned_token(token)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    (jar.remove(JWT_COOKIE_NAME), Ok(StatusCode::OK))
}
