use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state holding shared stores for users and banned tokens.
#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<RwLock<dyn UserStore + Send + Sync>>,
    pub banned_token_store: Arc<RwLock<dyn BannedTokenStore + Send + Sync>>,
    pub two_fa_code_store: Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>,
}

impl AppState {
    /// Creates a new `AppState` with the given stores.
    pub fn new(
        user_store: Arc<RwLock<dyn UserStore + Send + Sync>>,
        banned_token_store: Arc<RwLock<dyn BannedTokenStore + Send + Sync>>,
        two_fa_code_store: Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
        }
    }
}
