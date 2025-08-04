use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token.to_string());
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_banned_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(store.add_banned_token("token123").await.is_ok());
    }

    #[tokio::test]
    async fn test_is_token_banned() -> Result<(), BannedTokenStoreError> {
        let mut store = HashsetBannedTokenStore::default();
        let token = "token123";

        // Token not banned initially
        assert!(!store.is_token_banned(token).await?);

        // Ban the token
        store.add_banned_token(token).await?;

        // Now it should be banned
        assert!(store.is_token_banned(token).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_add_duplicate_token_is_idempotent() -> Result<(), BannedTokenStoreError> {
        let mut store = HashsetBannedTokenStore::default();
        let token = "token123";

        // Add token twice - both should succeed
        store.add_banned_token(token).await?;
        store.add_banned_token(token).await?;

        // Should still be banned (only once in the set)
        assert!(store.is_token_banned(token).await?);

        Ok(())
    }
}
