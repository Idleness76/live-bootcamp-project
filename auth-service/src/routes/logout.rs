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
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthAPIError::MissingToken)?;
    let token = cookie.value();
    let mut banned_store = app_state.banned_token_store.write().await;
    validate_token(&token, &*banned_store)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;
    banned_store
        .add_banned_token(token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;
    Ok((jar.remove(JWT_COOKIE_NAME), StatusCode::OK))
}
