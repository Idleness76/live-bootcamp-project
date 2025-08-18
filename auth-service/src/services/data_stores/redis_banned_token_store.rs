use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key = get_key(token);
        let ttl =
            u64::try_from(TOKEN_TTL_SECONDS).map_err(|_| BannedTokenStoreError::UnexpectedError)?;
        let mut conn = self.conn.write().await;
        conn.set_ex(key, true, ttl)
            .map_err(|_e: redis::RedisError| BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut conn = self.conn.write().await;
        conn.exists(key)
            .map_err(|_e: redis::RedisError| BannedTokenStoreError::UnexpectedError)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
