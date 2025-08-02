use crate::helpers::TestApp;
use auth_service::ErrorResponse;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let input = [
        serde_json::json!({
            "username": "testuser",
            // Missing password field
        }),
        serde_json::json!({
            "password": "secret123",
            // Missing email field
        }),
        serde_json::json!({}), // Empty object
        serde_json::json!({
            "wrong_field": "value"
            // Completely wrong fields
        }),
        serde_json::json!({
            "email": null,
            "password": "secret123"
            // Null values
        }),
        serde_json::json!({
            "email": 12345,
            "password": "secret123"
            // Wrong data type
        }),
        serde_json::json!({
            "email": "test@example.com",
            "password": ""
            // Empty password - validation failure
        }),
        serde_json::json!({
            "email": "",
            "password": "secret123"
            // Empty email - validation failure
        }),
    ];

    for i in input.iter() {
        let response = app.post_signup(&i).await;
        assert_eq!(response.status().as_u16(), 422, "Failed for input: {:?}", i);
    }
}

#[tokio::test]
async fn should_return_400_if_user_does_not_exist() {
    let app = TestApp::new().await;

    // Valid format but nonexistent user
    let nonexistent_user_creds = serde_json::json!({
        "email": "nonexistent@example.com",
        "password": "somepassword"
    });

    let response = app.post_login(&nonexistent_user_creds).await;
    assert_eq!(response.status().as_u16(), 400);

    let error_response = response
        .json::<ErrorResponse>()
        .await
        .expect("Could not deserialize response body to ErrorResponse");

    assert_eq!(error_response.error, "Invalid credentials");
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    // First create a user
    let signup_body = serde_json::json!({
        "email": "test@example.com",
        "password": "correct_password"
    });
    app.post_signup(&signup_body).await;

    // Try login with wrong password
    let login_body = serde_json::json!({
        "email": "test@example.com",
        "password": "wrong_password"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 401);
}
