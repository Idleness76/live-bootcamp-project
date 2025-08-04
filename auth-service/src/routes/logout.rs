use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{validate_token, JWT_COOKIE_NAME},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value();

    // Get write lock once and keep it
    let mut banned_store = app_state.banned_token_store.write().await;

    if let Err(_) = validate_token(&token, &*banned_store).await {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    if let Err(_) = banned_store.add_banned_token(token).await {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    (jar.remove(JWT_COOKIE_NAME), Ok(StatusCode::OK))
}
