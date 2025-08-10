use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::{LoginAttemptId, TwoFACode, TwoFACodeStore},
    utils::JWT_COOKIE_NAME,
};

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
        "email": "user@example.com",
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": "wrong-code"
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
        "email": "user@example.com",
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": "123456"
    });

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;
    let email = get_random_email();
    let parsed_email = auth_service::domain::Email::parse(email.clone()).expect("Invalid email");
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    app.two_fa_code_store
        .write()
        .await
        .add_code(
            parsed_email.clone(),
            login_attempt_id.clone(),
            two_fa_code.clone(),
        )
        .await
        .expect("Failed to store two_fa_code");

    let body = serde_json::json!({
        "email": parsed_email.as_ref(),
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": two_fa_code.as_ref()
    });

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Check for the auth cookie in the response headers
    let cookies: Vec<_> = response.headers().get_all("set-cookie").iter().collect();
    assert!(
        cookies.iter().any(|c| c
            .to_str()
            .unwrap()
            .contains(&format!("{}=", JWT_COOKIE_NAME))),
        "Expected {} cookie in response",
        JWT_COOKIE_NAME
    );
}
