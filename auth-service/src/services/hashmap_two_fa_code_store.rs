use std::collections::HashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .try_reserve(1)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .map(|_| ())
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
            .map_err(|e| match e {
                TwoFACodeStoreError::LoginAttemptIdNotFound => e,
                _ => TwoFACodeStoreError::UnexpectedError,
            })
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
            .map_err(|e| match e {
                TwoFACodeStoreError::LoginAttemptIdNotFound => e,
                _ => TwoFACodeStoreError::UnexpectedError,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, LoginAttemptId, TwoFACode};

    fn email(s: &str) -> Email {
        Email::parse(s.to_string()).unwrap()
    }

    #[tokio::test]
    async fn add_and_get_code_works() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email("user@example.com");
        let attempt = LoginAttemptId::default();
        let code = TwoFACode::default();
        store
            .add_code(email.clone(), attempt.clone(), code.clone())
            .await
            .unwrap();
        let (got_attempt, got_code) = store.get_code(&email).await.unwrap();
        assert_eq!(got_attempt, attempt);
        assert_eq!(got_code, code);
    }

    #[tokio::test]
    async fn remove_code_deletes_entry() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email("user@example.com");
        let attempt = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), attempt, code).await.unwrap();
        store.remove_code(&email).await.unwrap();
        let err = store.get_code(&email).await.unwrap_err();
        assert!(matches!(err, TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

    #[tokio::test]
    async fn remove_code_missing_returns_not_found() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email("missing@example.com");
        let err = store.remove_code(&email).await.unwrap_err();
        assert!(matches!(err, TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

    #[tokio::test]
    async fn get_code_missing_returns_not_found() {
        let store = HashmapTwoFACodeStore::default();
        let email = email("missing@example.com");
        let err = store.get_code(&email).await.unwrap_err();
        assert!(matches!(err, TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

    #[tokio::test]
    async fn add_code_overwrites_existing() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email("user@example.com");
        let attempt1 = LoginAttemptId::default();
        let code1 = TwoFACode::default();
        store
            .add_code(email.clone(), attempt1.clone(), code1.clone())
            .await
            .unwrap();
        let attempt2 = LoginAttemptId::default();
        let code2 = TwoFACode::default();
        store
            .add_code(email.clone(), attempt2.clone(), code2.clone())
            .await
            .unwrap();
        let (got_attempt, got_code) = store.get_code(&email).await.unwrap();
        assert_eq!(got_attempt, attempt2);
        assert_eq!(got_code, code2);
    }
}
