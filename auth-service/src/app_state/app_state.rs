use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::UserStore;

// Using trait objects with Arc<RwLock<dyn UserStore>>
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}
