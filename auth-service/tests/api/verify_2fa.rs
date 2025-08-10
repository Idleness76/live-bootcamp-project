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

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    // Simulate a valid login attempt (you may need to create one in your setup)
    let login_attempt_id = LoginAttemptId::default();

    let body = serde_json::json!({
        "login_attempt_id": login_attempt_id,
        "code": "wrong-code"
    });

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_user_does_not_exist() {
    let app = TestApp::new().await;

    // Use a random login_attempt_id that doesn't exist
    let login_attempt_id = LoginAttemptId::default();

    let body = serde_json::json!({
        "login_attempt_id": login_attempt_id,
        "code": "123456"
    });

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}
