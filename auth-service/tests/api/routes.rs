use crate::helpers::TestApp;

// Tokio's test macro is used to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn signup_route_works() {
    let app = TestApp::new().await;
    let response = app.post_signup("testuser", "password123").await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_route_works() {
    let app = TestApp::new().await;
    let response = app.post_login("testuser", "password123").await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_route_works() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_2fa_route_works() {
    let app = TestApp::new().await;
    let response = app.post_verify_2fa("123456").await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_route_works() {
    let app = TestApp::new().await;
    let response = app.post_verify_token("valid_token").await;
    assert_eq!(response.status().as_u16(), 200);
}
