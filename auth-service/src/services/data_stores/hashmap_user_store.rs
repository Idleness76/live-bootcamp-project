use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

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
        self.users.insert(user.email.clone(), user);
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
        match self.users.get(email) {
            Some(user) if &user.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn authenticate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError> {
        let email =
            Email::parse(email.to_owned()).map_err(|_| UserStoreError::InvalidCredentials)?;
        let password =
            Password::parse(password.to_owned()).map_err(|_| UserStoreError::InvalidCredentials)?;
        match self.users.get(&email) {
            Some(user) if user.password == password => Ok(user.clone()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password = Password::parse("Password123!".to_string()).unwrap();
        let user = User::new(email, password, true);
        assert!(user_store.add_user(user).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password = Password::parse("Password123!".to_string()).unwrap();
        let user = User::new(email.clone(), password, true);
        user_store.add_user(user).await?;
        let retrieved_user = user_store.get_user(&email).await?;

        assert_eq!(retrieved_user.email, email);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_user() -> Result<(), UserStoreError> {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password = Password::parse("Password123!".to_string()).unwrap();
        let user = User::new(email.clone(), password.clone(), true);
        user_store.add_user(user).await?;

        assert!(user_store.validate_user(&email, &password).await.is_ok());

        let wrong_password = Password::parse("WrongPassword123!".to_string()).unwrap();
        assert!(user_store
            .validate_user(&email, &wrong_password)
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_add_duplicate_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password = Password::parse("Password123!".to_string()).unwrap();
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
        let email = Email::parse("nonexistent@example.com".to_string()).unwrap();
        assert_eq!(
            user_store.get_user(&email).await.unwrap_err(),
            UserStoreError::UserNotFound
        );
    }
}
