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

#[derive(sqlx::FromRow)]
struct UserPasswordRow {
    password_hash: String,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let result = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            ON CONFLICT (email) DO NOTHING
            "#,
            user.email.as_ref(),
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserAlreadyExists);
        }

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let row = sqlx::query_as!(
            User,
            r#"
            SELECT email as "email: _", password_hash as "password: _", requires_2fa
            FROM users
            WHERE email = $1
            "#,
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        row.ok_or(UserStoreError::UserNotFound)
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let row = sqlx::query_as!(
            UserPasswordRow,
            r#"
            SELECT password_hash FROM users WHERE email = $1
            "#,
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        let password_hash = match row {
            Some(row) => row.password_hash,
            None => return Err(UserStoreError::UserNotFound),
        };

        verify_password_hash(password_hash, password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }

    #[tracing::instrument(name = "Authenticating user from PostgreSQL", skip_all)]
    async fn authenticate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError> {
        let row = sqlx::query(
            r#"
            SELECT email, password_hash, requires_2fa
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email as &str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
        .ok_or(UserStoreError::UserNotFound)?;

        let user = User {
            email: Email::parse(row.get::<&str, _>("email").to_string())
                .map_err(|_| UserStoreError::InvalidCredentials)?,
            password: Password::parse(row.get::<&str, _>("password_hash").to_string())
                .map_err(|_| UserStoreError::InvalidCredentials)?,
            requires_2fa: row.get::<bool, _>("requires_2fa"),
        };

        verify_password_hash(user.password.as_ref().to_string(), password.to_string())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(user)
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let current_span: tracing::Span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let parsed_hash = PasswordHash::new(&expected_password_hash)?;
            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &parsed_hash)
                .map_err(|e| e.into())
        })
    })
    .await?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let current_span: tracing::Span = tracing::Span::current();
    let password_hash = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut rand::thread_rng());
            Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
        })
    })
    .await??;

    Ok(password_hash)
}
