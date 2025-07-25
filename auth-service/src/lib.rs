use axum::{routing::post, serve::Serve, Router};
use std::error::Error;
use tower_http::services::ServeDir;

pub mod routes;

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
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        // Create router with static file serving from the "assets" directory
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            // Register all authentication API endpoints with their handler functions
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/logout", post(routes::logout))
            .route("/verify-token", post(routes::verify_token));

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
