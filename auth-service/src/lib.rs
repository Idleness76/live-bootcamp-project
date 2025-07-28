use app_state::AppState;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    serve::Serve,
    Json, Router,
};
use domain::AuthAPIError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tower_http::services::ServeDir;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;

/// Application struct that encapsulates the HTTP server and its configuration.
/// This provides a clean separation between server construction and execution.
pub struct Application {
    /// The configured Axum server instance ready to be started
    server: Serve<Router, Router>,
    /// The actual address the server is bound to (useful for testing with random ports)
    pub address: String,
}

impl Application {
    /// Builds and configures the application server without starting it.
    ///
    /// # Arguments
    /// * `address` - The address to bind the server to (e.g., "127.0.0.1:3000" or "0.0.0.0:0")
    ///
    /// # Returns
    /// * `Ok(Application)` - Successfully configured application ready to run
    /// * `Err(Box<dyn Error>)` - Failed to bind to the address or configure the server
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Create router with API routes first
        let router = Router::new()
            // Add root route that serves the index.html
            .route("/", get(serve_index))
            // Register all authentication API endpoints with their handler functions
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/logout", post(routes::logout))
            .route("/verify-token", post(routes::verify_token))
            // Serve static files from /assets path instead of root
            .nest_service("/assets", ServeDir::new("assets"))
            .with_state(app_state);

        // Bind to the specified address and get the actual bound address
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        // Create the server instance with the listener and router
        let server = axum::serve(listener, router);

        // Return the configured application
        Ok(Application { server, address })
    }

    /// Starts the HTTP server and runs it until shutdown.
    /// This consumes the Application instance.
    ///
    /// # Returns
    /// * `Ok(())` - Server shut down gracefully
    /// * `Err(std::io::Error)` - Server encountered an error while running
    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        // Start the server and await its completion
        self.server.await
    }
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
