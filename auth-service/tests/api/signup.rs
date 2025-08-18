use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::ErrorResponse, routes::SignupResponse};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "Password123!",
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
            "password": "Password123!"
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

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let input = [
        serde_json::json!({
            "email": "not-an-email",  // Invalid email format
            "password": "Password123!",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "123",  // Too short password
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "",  // Empty email
            "password": "Password123!",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "",  // Empty password
            "requires2FA": true
        }),
    ];

    for i in input.iter() {
        let response = app.post_signup(i).await;
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

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });

    // First signup - should succeed (201 Created)
    let first_response = app.post_signup(&signup_body).await;
    assert_eq!(first_response.status().as_u16(), 201);

    // Second signup with same email - should fail with 409 Conflict
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let mut app = TestApp::new().await;

    let new_user = serde_json::json!({
        "email": get_random_email(),
        "requires2FA": true,
        "password": "Password123!"
    });

    let response = app.post_signup(&new_user).await;
    assert_eq!(response.status().as_u16(), 201);

    // Validate response body
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to SignupResponse")
            .message,
        "User created successfully!"
    );

    app.clean_up().await.unwrap();
}
