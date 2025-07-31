use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

/// A simple in-memory user store implementation using HashMap.
///
/// This store maps user email addresses to User objects and provides
/// basic CRUD operations for user management.
#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        let email = user.email.clone();
        self.users.insert(email, user);
        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        if &self.get_user(email).await?.password == password {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let password = Password::parse("Password123!").unwrap();
        let user = User::new(email, password, true);
        assert!(user_store.add_user(user).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let password = Password::parse("Password123!").unwrap();
        let user = User::new(email.clone(), password, true);
        user_store.add_user(user).await?;
        let retrieved_user = user_store.get_user(&email).await?;

        assert_eq!(retrieved_user.email, email);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let password = Password::parse("Password123!").unwrap();
        let user = User::new(email.clone(), password.clone(), true);
        user_store.add_user(user).await?;

        // Test valid credentials
        assert!(user_store.validate_user(&email, &password).await.is_ok());

        // Test invalid password
        let wrong_password = Password::parse("WrongPassword123!").unwrap();
        assert!(user_store
            .validate_user(&email, &wrong_password)
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_add_duplicate_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let password = Password::parse("Password123!").unwrap();
        let user1 = User::new(email.clone(), password.clone(), true);
        let user2 = User::new(email, password, false);

        assert!(user_store.add_user(user1).await.is_ok());
        assert_eq!(
            user_store.add_user(user2).await.unwrap_err(),
            UserStoreError::UserAlreadyExists
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let user_store = HashmapUserStore::default();
        let email = Email::parse("nonexistent@example.com").unwrap();
        assert_eq!(
            user_store.get_user(&email).await.unwrap_err(),
            UserStoreError::UserNotFound
        );
    }
}
