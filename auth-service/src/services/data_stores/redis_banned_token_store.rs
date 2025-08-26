use color_eyre::eyre::eyre;
use redis::{Commands, Connection};
use std::sync::Arc;
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
    #[tracing::instrument(name = "Add Banned Token", skip_all)]
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key = get_key(token);

        let ttl = u64::try_from(TOKEN_TTL_SECONDS).map_err(|e| {
            BannedTokenStoreError::UnexpectedError(eyre!(
                "tfailed to cast TOKEN_TTL_SECONDS to u64: {}",
                e
            ))
        })?;

        let mut conn = self.conn.write().await;

        let _: redis::Value = conn
            .set_ex(key, true, ttl)
            .map_err(|e: redis::RedisError| {
                BannedTokenStoreError::UnexpectedError(eyre!(
                    "failed to set banned token in Redis: {}",
                    e
                ))
            })?;

        Ok(())
    }

    #[tracing::instrument(name = "Is Token Banned", skip_all)]
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);

        let mut conn = self.conn.write().await;

        let exists: bool = conn.exists(key).map_err(|e: redis::RedisError| {
            BannedTokenStoreError::UnexpectedError(eyre!(
                "failed to check if token exists in Redis: {}",
                e
            ))
        })?;

        Ok(exists)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
