use crate::domain::{Email, Password, User};
use color_eyre::eyre::{eyre, Context, Report, Result};
use secrecy::Secret;
use serde::Serialize;
use thiserror::Error;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
    async fn authenticate_user(
        &self,
        email: &str,
        password: &Secret<String>,
    ) -> Result<User, UserStoreError>;
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;

    /// Atomically ensure token is banned; returns Ok(true) if newly inserted, Ok(false) if already banned.
    async fn ban_if_not_present(&mut self, token: &str) -> Result<bool, BannedTokenStoreError> {
        if self.is_token_banned(token).await? {
            return Ok(false);
        }
        self.add_banned_token(token).await?;
        Ok(true)
    }
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

// NOTE: We are using the "parse don't validate" principle.
// LoginAttemptId and TwoFACode are wrappers around a string type, similar to the Email and Password types.

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(s: &str) -> Result<Self> {
        let parsed_id = uuid::Uuid::parse_str(s).wrap_err("Invalid login attempt id")?; // Updated!
        Ok(Self(parsed_id.to_string()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: &str) -> Result<Self> {
        if code.len() == 6 && code.chars().all(|c| c.is_digit(10)) {
            Ok(Self(code.to_string()))
        } else {
            Err(eyre!("Invalid 2FA code format"))
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..1_000_000);
        Self(format!("{:06}", code))
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
