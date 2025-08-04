use crate::helpers::TestApp;

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
