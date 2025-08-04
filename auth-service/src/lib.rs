use app_state::AppState;
use axum::{
    http::{HeaderValue, Method},
    response::Html,
    routing::{get, post},
    serve::Serve,
    Router,
};

use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Bind to the specified address and get the actual bound address
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        let allowed_origins = [
            "https://idlelgr.duckdns.org".parse::<HeaderValue>()?,
            "http://localhost:8000".parse::<HeaderValue>()?,
        ];

        let cors = CorsLayer::new()
            // Allow GET and POST requests
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        // Create and configure router in one expression to minimize its scope
        let server = axum::serve(
            listener,
            Router::new()
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
                .with_state(app_state)
                .layer(cors),
        );

        // Return the configured application
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}
