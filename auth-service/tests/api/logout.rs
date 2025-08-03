use auth_service::{utils::JWT_COOKIE_NAME, ErrorResponse};
use reqwest::Url;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status(), 400);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse response body");

    assert_eq!(error_response.error, "Missing auth token");
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

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
}
