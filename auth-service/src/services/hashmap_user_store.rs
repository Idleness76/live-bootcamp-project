use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

/// A simple in-memory user store implementation using HashMap.
///
/// This store maps user email addresses to User objects and provides
/// basic CRUD operations for user management.
#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;
        if user.password == password {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let email = "test@example.com".to_string();
        let password = "secret123".to_string();
        let user = User::new(email.clone(), password.clone(), true);
        assert!(user_store.add_user(user).is_ok());
    }

    #[test]
    fn test_get_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = "test@example.com".to_string();
        let password = "secret123".to_string();
        let user = User::new(email.clone(), password.clone(), true);
        let email = user.email.clone();
        user_store.add_user(user)?;
        let retrieved_user = user_store.get_user(&email)?;

        assert_eq!(retrieved_user.email, email);
        Ok(())
    }

    #[test]
    fn test_validate_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = "test@example.com".to_string();
        let password = "secret123".to_string();
        let user = User::new(email.clone(), password.clone(), true);
        user_store.add_user(user)?;

        // Test valid credentials
        assert!(user_store.validate_user(&email, &password).is_ok());

        // Test invalid password
        assert!(user_store.validate_user(&email, "wrong_password").is_err());

        Ok(())
    }
}
