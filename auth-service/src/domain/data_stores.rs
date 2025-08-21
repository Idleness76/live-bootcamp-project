use serde::Serialize;

use crate::domain::{Email, Password, User};

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
    async fn authenticate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
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

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    UnexpectedError,
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

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(s: &str) -> Result<Self, String> {
        uuid::Uuid::parse_str(s)
            .map(|_| Self(s.to_string()))
            .map_err(|_| "Invalid UUID format".to_string())
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
    pub fn parse(code: &str) -> Result<Self, String> {
        if code.len() == 6 && code.chars().all(|c| c.is_digit(10)) {
            Ok(Self(code.to_string()))
        } else {
            Err("Invalid 2FA code format".to_string())
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
