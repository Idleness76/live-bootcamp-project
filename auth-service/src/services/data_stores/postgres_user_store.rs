use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use color_eyre::eyre::{eyre, Context, Result};
use secrecy::{ExposeSecret, Secret};
use sqlx::{PgPool, Row};

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
        // Clone the secret password (we own `user`) and keep it wrapped while passing to the hashing helper
        let password_hash = compute_password_hash(user.password.as_ref().clone())
            .await
            .map_err(UserStoreError::UnexpectedError)?;

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
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

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
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

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
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        let password_hash = match row {
            Some(row) => row.password_hash,
            None => return Err(UserStoreError::UserNotFound),
        };

        // Wrap the stored hash in a Secret to avoid exposing raw strings at the callsite.
        let stored = Secret::new(password_hash);
        verify_password_hash(&stored, password.as_ref())
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }

    #[tracing::instrument(name = "Authenticating user from PostgreSQL", skip_all)]
    async fn authenticate_user(
        &self,
        email: &str,
        password: &Secret<String>,
    ) -> Result<User, UserStoreError> {
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
        .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?
        .ok_or(UserStoreError::UserNotFound)?;

        let user = User {
            email: Email::parse(row.get::<&str, _>("email").to_string())
                .map_err(|_| UserStoreError::InvalidCredentials)?,
            // Keep the DB password hash wrapped as a Secret when parsing into `Password`.
            password: Password::parse(Secret::new(row.get::<&str, _>("password_hash").to_string()))
                .map_err(|_| UserStoreError::InvalidCredentials)?,
            requires_2fa: row.get::<bool, _>("requires_2fa"),
        };

        // Compare the stored password hash (wrapped) with the incoming secret password.
        verify_password_hash(user.password.as_ref(), password)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(user)
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: &Secret<String>,
    password_candidate: &Secret<String>,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();

    // Clone the inner strings here (safe within the current scope) and move into blocking code.
    let expected_clone = expected_password_hash.expose_secret().clone();
    let candidate_clone = password_candidate.expose_secret().clone();

    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let parsed_hash = PasswordHash::new(expected_clone.as_str())?;
            Argon2::default()
                .verify_password(candidate_clone.as_bytes(), &parsed_hash)
                .wrap_err("failed to verify password hash")
        })
    })
    .await?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: Secret<String>) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current();

    let password_hash = tokio::task::spawn_blocking(move || -> Result<String> {
        current_span.in_scope(|| {
            let find_hash = password.expose_secret();
            let salt = SaltString::generate(&mut rand::thread_rng());
            let params = Params::new(15000, 2, 1, None)
                .map_err(|e| eyre!("invalid argon2 params: {}", e))?;
            let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let ph = argon
                .hash_password(find_hash.as_bytes(), &salt)
                .map_err(|e| eyre!("failed to hash password: {}", e))?;
            Ok(ph.to_string())
        })
    })
    .await??;

    Ok(password_hash)
}
