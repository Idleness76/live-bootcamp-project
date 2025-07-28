use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
            // Missing email field - this should cause 422
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
            // Missing password field - this should cause 422
        }),
        serde_json::json!({
            "email": "invalid-email",
            "password": "password123"
            // Missing requires2FA field - this should cause 422
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let new_user = serde_json::json!({
        "email": get_random_email(),
        "requires2FA": true,
        "password": "password123"
    });

    let response = app.post_signup(&new_user).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed for input: {:?}",
        new_user
    );
}
