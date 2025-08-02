use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    // Attempt to login with malformed credentials
    let response = app
        .post_login(&serde_json::json!({
            "username": "testuser",
            // Missing password field
        }))
        .await;
    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        response
    );
}
