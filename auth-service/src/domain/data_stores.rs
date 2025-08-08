use crate::domain::{Email, Password, User};

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
    async fn authenticate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<User, UserStoreError>;
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
