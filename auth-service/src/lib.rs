use crate::utils::{
    env::ALLOWED_ORIGINS_ENV_VAR, make_span_with_request_id, on_request, on_response,
    DEFAULT_ALLOWED_ORIGINS,
};
use app_state::AppState;
use axum::{
    http::{HeaderValue, Method},
    response::Html,
    routing::{get, post},
    serve::Serve,
    Router,
};
use redis::{Client, RedisResult};
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, error::Error};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
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
                .layer(cors)
                .layer(
                    // New!
                    // Add a TraceLayer for HTTP requests to enable detailed tracing
                    // This layer will create spans for each request using the make_span_with_request_id function,
                    // and log events at the start and end of each request using on_request and on_response functions.
                    TraceLayer::new_for_http()
                        .make_span_with(make_span_with_request_id)
                        .on_request(on_request)
                        .on_response(on_response),
                ),
        );
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
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

pub async fn get_postgres_pool(url: Secret<String>) -> Result<PgPool, sqlx::Error> {
    // Create a new PostgreSQL connection pool
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.expose_secret())
        .await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
