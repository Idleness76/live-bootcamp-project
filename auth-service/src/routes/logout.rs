use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{decode_claims, JWT_COOKIE_NAME},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthAPIError::MissingToken)?;
    let token = cookie.value();

    // Decode outside of any lock
    decode_claims(token).map_err(|_| AuthAPIError::InvalidToken)?;

    // Atomic check+insert under one write lock
    let newly_banned = {
        let mut store = app_state.banned_token_store.write().await;
        store
            .ban_if_not_present(token)
            .await
            .map_err(|_| AuthAPIError::UnexpectedError)?
    };

    if !newly_banned {
        // Token already banned: idempotent success, better UX
        return Ok((jar.remove(JWT_COOKIE_NAME), StatusCode::OK));
    }

    Ok((jar.remove(JWT_COOKIE_NAME), StatusCode::OK))
}
