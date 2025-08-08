use app_state::AppState;
use axum::{
    http::{HeaderValue, Method},
    response::Html,
    routing::{get, post},
    serve::Serve,
    Router,
};

use crate::utils::{env::ALLOWED_ORIGINS_ENV_VAR, DEFAULT_ALLOWED_ORIGINS};
use std::{env, error::Error};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

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

        let cors = {
            let base = || {
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST])
                    .allow_credentials(true)
            };
            match load_allowed_origins()? {
                None => base().allow_origin(Any),
                Some(list) if list.is_empty() => base().allow_origin(Any),
                Some(list) => base().allow_origin(list),
            }
        };

        let server = axum::serve(
            listener,
            Router::new()
                .route("/", get(serve_index))
                .route("/signup", post(routes::signup))
                .route("/login", post(routes::login))
                .route("/verify-2fa", post(routes::verify_2fa))
                .route("/logout", post(routes::logout))
                .route("/verify-token", post(routes::verify_token))
                .nest_service("/assets", ServeDir::new("assets"))
                .with_state(app_state)
                .layer(cors),
        );
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}

/// Load allowed origins from env; "*" => wildcard (Any).
fn load_allowed_origins() -> Result<Option<Vec<HeaderValue>>, Box<dyn Error>> {
    let raw = env::var(ALLOWED_ORIGINS_ENV_VAR)
        .unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string())
        .trim()
        .to_string();
    if raw == "*" {
        return Ok(None);
    }
    let parsed = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(HeaderValue::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(parsed))
}
