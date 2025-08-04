use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::ErrorResponse, utils::JWT_COOKIE_NAME};

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
        "password": "Password123!"
    });
    app.post_signup(&signup_body).await;

    // Try login with wrong password
    let login_body = serde_json::json!({
        "email": "test@example.com",
        "password": "!Password123" // Incorrect password
    });

    let response = app.post_login(&login_body).await;
    let status = response.status().as_u16();
    println!("Status: {}", status);
    println!("Body: {:?}", response.text().await);
    assert_eq!(status, 401);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password123!",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}
