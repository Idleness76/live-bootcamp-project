use crate::helpers::TestApp;
use auth_service::ErrorResponse;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let malformed_inputs = [
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
    ];

    for input in malformed_inputs {
        let response = app.post_login(&input).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            input
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let input = [
        serde_json::json!({
            "email": "test@example.com",
            "password": ""
        }),
        serde_json::json!({
            "email": "",
            "password": "secret123"
        }),
    ];

    for i in input.iter() {
        let response = app.post_login(&input).await;
        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", i);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}
