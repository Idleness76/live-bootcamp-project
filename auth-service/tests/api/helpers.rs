use reqwest::cookie::Jar;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState,
    services::{HashmapTwoFACodeStore, HashmapUserStore, HashsetBannedTokenStore, MockEmailClient},
    utils::test,
    Application,
};
use uuid::Uuid;

/// Test application wrapper that provides HTTP client functionality for integration tests.
/// This struct encapsulates a running server instance and an HTTP client for making requests.
pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<RwLock<HashsetBannedTokenStore>>,
    pub two_fa_code_store: Arc<RwLock<HashmapTwoFACodeStore>>,
    pub email_client: Arc<MockEmailClient>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_client = Arc::new(MockEmailClient);
        let app_state = AppState::new(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );

        // Build application on random port for test isolation
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        // Format the full HTTP address for client requests
        let address = format!("http://{}", app.address.clone());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        // Start the server in background - we don't need to await it
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        TestApp {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }

    /// Makes a GET request to the root endpoint ("/")
    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the signup endpoint with properly formatted JSON body
    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the login endpoint with username and password
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the logout endpoint (no body required)
    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the token verification endpoint
    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the 2FA verification endpoint with a token
    pub async fn post_verify_2fa(&self, token: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .header("Content-Type", "application/json")
            // Format JSON body with 2FA token
            .body(format!(r#"{{"token":"{}"}}"#, token))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
