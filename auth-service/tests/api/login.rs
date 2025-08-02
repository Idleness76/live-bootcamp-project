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
async fn should_return_400_if_invalid_credentials() {
    let app = TestApp::new().await;

    // Valid format but wrong credentials
    let invalid_creds = serde_json::json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let response = app.post_login(&invalid_creds).await;
    assert_eq!(response.status().as_u16(), 400);

    let error_response = response
        .json::<ErrorResponse>()
        .await
        .expect("Could not deserialize response body to ErrorResponse");

    assert_eq!(error_response.error, "Invalid credentials");
}
