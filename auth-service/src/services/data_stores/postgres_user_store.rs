use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::{PgPool, Row};

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let query = r#"
        INSERT INTO users (email, password_hash, requires_2fa)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO NOTHING
        "#;

        let result = sqlx::query(query)
            .bind(user.email.as_ref())
            .bind(password_hash)
            .bind(user.requires_2fa)
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserAlreadyExists);
        }

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let query = r#"
        SELECT * FROM users
        WHERE email = $1
        "#;

        let row = sqlx::query(query)
            .bind(email.as_ref())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let row = match row {
            Some(row) => row,
            None => return Err(UserStoreError::UserNotFound),
        };

        let user = User {
            email: Email::parse(row.get::<&str, _>("email").to_string())
                .map_err(|_| UserStoreError::UnexpectedError)?,
            password: Password::parse(row.get::<&str, _>("password_hash").to_string())
                .map_err(|_| UserStoreError::UnexpectedError)?,
            requires_2fa: row.get::<bool, _>("requires_2fa"),
        };

        Ok(user)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let query = r#"
        SELECT password_hash FROM users
        WHERE email = $1
        "#;

        let row = sqlx::query(query)
            .bind(email.as_ref())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let password_hash = match row {
            Some(row) => row.get::<&str, _>("password_hash").to_string(),
            None => return Err(UserStoreError::UserNotFound),
        };

        verify_password_hash(password_hash.to_string(), password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }

    async fn authenticate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError> {
        let email =
            Email::parse(email.to_owned()).map_err(|_| UserStoreError::InvalidCredentials)?;
        let password =
            Password::parse(password.to_owned()).map_err(|_| UserStoreError::InvalidCredentials)?;

        let query = r#"
        SELECT * FROM users
        WHERE email = $1
        "#;

        let row = sqlx::query(query)
            .bind(email.as_ref())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let row = match row {
            Some(row) => row,
            None => return Err(UserStoreError::UserNotFound),
        };

        let password_hash = row.get::<&str, _>("password_hash");

        verify_password_hash(password_hash.to_string(), password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        let user = User {
            email,
            password,
            requires_2fa: row.get::<bool, _>("requires_2fa"),
        };

        Ok(user)
    }
}

async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&expected_password_hash)?;
        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &parsed_hash)
            .map_err(|e| e.into())
    })
    .await?
}

async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut rand::thread_rng());
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
    })
    .await??;

    Ok(password_hash)
}
