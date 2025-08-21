use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    Email, TwoFACodeStoreError, {LoginAttemptId, TwoFACode, TwoFACodeStore},
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(&email);
        // 2. Create a TwoFATuple instance.
        let tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        // 3. Use serde_json::to_string to serialize the TwoFATuple instance into a JSON string.
        // Return TwoFACodeStoreError::UnexpectedError if serialization fails.
        let serialized =
            serde_json::to_string(&tuple).map_err(|_e| TwoFACodeStoreError::UnexpectedError)?;
        // 4. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        // The value should be the serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUTES_IN_SECONDS.
        // Return TwoFACodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let mut conn = self.conn.write().await;
        let _: redis::Value = conn
            .set_ex(key, serialized, TEN_MINUTES_IN_SECONDS)
            .map_err(|_e| TwoFACodeStoreError::UnexpectedError)?;
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(&email);
        // 2. Call the del command on the Redis connection to delete the 2FA code entry.
        // Return TwoFACodeStoreError::UnexpectedError if the operation fails.
        let mut conn = self.conn.write().await;
        let deleted: i32 = conn
            .del(key)
            .map_err(|_e| TwoFACodeStoreError::UnexpectedError)?;
        if deleted >= 1 {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::UnexpectedError)
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        let key = get_key(&email);
        // 2. Call the get command on the Redis connection to get the value stored for the key.
        // Return TwoFACodeStoreError::LoginAttemptIdNotFound if the operation fails.
        // If the operation succeeds, call serde_json::from_str to parse the JSON string into a TwoFATuple.
        // Then, parse the login attempt ID string and 2FA code string into a LoginAttemptId and TwoFACode type respectively.
        // Return TwoFACodeStoreError::UnexpectedError if parsing fails.
        let mut conn = self.conn.write().await;
        let value: String = conn
            .get(key)
            .map_err(|_e: redis::RedisError| TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        // Parse the JSON string into a TwoFATuple
        let tuple: TwoFATuple =
            serde_json::from_str(&value).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        // Parse the login attempt ID and 2FA code into their respective types
        let login_attempt_id: LoginAttemptId =
            LoginAttemptId::parse(&tuple.0).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code: TwoFACode =
            TwoFACode::parse(&tuple.1).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
