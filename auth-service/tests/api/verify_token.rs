use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        "",                  // Empty body
        "{invalid_json",     // Malformed JSON
        "{}",                // Missing required fields
        r#"{"token": 123}"#, // Wrong type
    ];

    for payload in test_cases {
        let response = app
            .http_client
            .post(&format!("{}/verify-token", app.address))
            .header("content-type", "application/json")
            .body(payload)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status().as_u16(), 422);
    }
}
