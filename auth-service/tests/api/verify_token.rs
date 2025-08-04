use auth_service::{domain::ErrorResponse, utils::JWT_COOKIE_NAME};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({}),                       // Missing token field
        serde_json::json!({"token": 123}),           // Wrong type
        serde_json::json!({"token": null}),          // Null value
        serde_json::json!({"wrong_field": "value"}), // Wrong field
    ];

    for payload in test_cases {
        let response = app.post_verify_token(&payload).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({"token": "invalid_token"}),
        serde_json::json!({"token": "expired.jwt.token"}),
        serde_json::json!({"token": "malformed.jwt"}),
        serde_json::json!({"token": ""}), // Empty token
    ];

    for payload in test_cases {
        let response = app.post_verify_token(&payload).await;
        assert_eq!(response.status().as_u16(), 401);

        let error_response = response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body");

        assert_eq!(error_response.error, "Invalid token");
    }
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Create user
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });
    app.post_signup(&signup_body).await;

    // Login to get valid JWT
    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!"
    });

    let login_response = app.post_login(&login_body).await;
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    // Verify the token
    let verify_body = serde_json::json!({"token": auth_cookie.value()});
    let response = app.post_verify_token(&verify_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Create and login user
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });
    app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!"
    });

    let login_response = app.post_login(&login_body).await;
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    // Ban the token via logout
    app.post_logout().await;

    // Try to verify the now-banned token
    let verify_body = serde_json::json!({"token": auth_cookie.value()});
    let response = app.post_verify_token(&verify_body).await;

    assert_eq!(response.status().as_u16(), 401);

    let error_response = response
        .json::<ErrorResponse>()
        .await
        .expect("Could not deserialize response body");

    assert_eq!(error_response.error, "Invalid token");
}
