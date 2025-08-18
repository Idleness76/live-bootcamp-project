use auth_service::domain::BannedTokenStore;
use auth_service::{domain::ErrorResponse, utils::JWT_COOKIE_NAME};
use reqwest::cookie::CookieStore;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_and_ban_jwt_token_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Extract JWT from cookie jar
    let jwt_token = app
        .cookie_jar
        .cookies(&app.address.parse::<Url>().unwrap())
        .and_then(|header_value| header_value.to_str().ok().map(|s| s.to_string()))
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|part| part.trim().starts_with(&format!("{}=", JWT_COOKIE_NAME)))
                .and_then(|part| part.split('=').nth(1).map(|s| s.to_string()))
        })
        .expect("JWT cookie not found");

    // Logout should succeed
    let response = app.post_logout().await;
    assert_eq!(response.status(), 200);

    // Verify token was banned
    let is_banned = app
        .banned_token_store
        .read()
        .await
        .is_token_banned(&jwt_token)
        .await
        .expect("Failed to check banned token");

    assert!(is_banned, "JWT should be banned after logout");

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    // First logout should succeed
    let response = app.post_logout().await;
    assert_eq!(response.status(), 200);

    // Second logout with same cookie should fail
    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(error_response.error, "Missing token");

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status(), 400);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(error_response.error, "Missing token");

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status(), 401);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(error_response.error, "Invalid token");

    app.clean_up().await.unwrap();
}
