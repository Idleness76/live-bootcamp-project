use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::LoginAttemptId;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let login_attempt_id = LoginAttemptId::default();

    let test_cases = [
        // Missing login_attempt_id
        serde_json::json!({
            "code": "123456"
        }),
        // Missing code
        serde_json::json!({
            "login_attempt_id": login_attempt_id
        }),
        // Both fields missing
        serde_json::json!({}),
        // login_attempt_id wrong type
        serde_json::json!({
            "login_attempt_id": 123,
            "code": "123456"
        }),
        // code wrong type
        serde_json::json!({
            "login_attempt_id": login_attempt_id,
            "code": 123456
        }),
    ];

    for case in test_cases.iter() {
        let response = app.post_verify_2fa(case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Expected 422 for input: {:?}",
            case
        );
    }
}
