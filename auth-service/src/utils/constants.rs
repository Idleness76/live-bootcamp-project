use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref POSTGRES_PASSWORD: String = set_postgres_password();
    pub static ref DATABASE_URL: String = set_database_url();
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    secret
}

fn set_postgres_password() -> String {
    dotenv().ok();
    let secret =
        std_env::var(env::POSTGRES_PASSWORD_ENV_VAR).expect("POSTGRES_PASSWORD must be set.");
    if secret.is_empty() {
        panic!("POSTGRES_PASSWORD must be set.");
    }
    secret
}

fn set_database_url() -> String {
    dotenv().ok();
    let secret = std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set.");
    if secret.is_empty() {
        panic!("DATABASE_URL must be set.");
    }
    secret
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const POSTGRES_PASSWORD_ENV_VAR: &str = "POSTGRES_PASSWORD";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const ALLOWED_ORIGINS_ENV_VAR: &str = "ALLOWED_ORIGINS";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_ALLOWED_ORIGINS: &str = "https://idlelgr.duckdns.org,http://localhost:8000";
