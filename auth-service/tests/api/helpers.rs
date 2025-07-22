use auth_service::Application;

/// Test application wrapper that provides HTTP client functionality for integration tests.
/// This struct encapsulates a running server instance and an HTTP client for making requests.
pub struct TestApp {
    /// Base URL of the running test server (e.g., "http://127.0.0.1:12345")
    pub address: String,
    /// HTTP client for making requests to the test server
    pub http_client: reqwest::Client,
}

impl TestApp {
    /// Creates a new test application instance with a server running on a random port.
    ///
    /// This method:
    /// 1. Builds the application on a random available port (127.0.0.1:0)
    /// 2. Spawns the server in a background task
    /// 3. Creates an HTTP client for making test requests
    ///
    /// # Returns
    /// A configured `TestApp` instance ready for testing
    pub async fn new() -> Self {
        // Build application on random port for test isolation
        let app = Application::build("127.0.0.1:0")
            .await
            .expect("Failed to build app");

        // Format the full HTTP address for client requests
        let address = format!("http://{}", app.address.clone());

        // Start the server in background - we don't need to await it
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        // Create HTTP client for making test requests
        let http_client = reqwest::Client::new();

        TestApp {
            address,
            http_client,
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

    /// Makes a POST request to the signup endpoint with username and password
    pub async fn post_signup(&self, username: &str, password: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .header("Content-Type", "application/json")
            // Format JSON body with user credentials
            .body(format!(
                r#"{{"username":"{}","password":"{}"}}"#,
                username, password
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Makes a POST request to the login endpoint with username and password
    pub async fn post_login(&self, username: &str, password: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .header("Content-Type", "application/json")
            // Format JSON body with login credentials
            .body(format!(
                r#"{{"username":"{}","password":"{}"}}"#,
                username, password
            ))
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

    /// Makes a POST request to the token verification endpoint
    pub async fn post_verify_token(&self, token: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .header("Content-Type", "application/json")
            // Format JSON body with token to verify
            .body(format!(r#"{{"token":"{}"}}"#, token))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
