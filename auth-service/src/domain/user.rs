use serde::Deserialize;

use crate::domain::{Email, Password};

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub email: Email,
    pub password: Password,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl User {
    /// Creates a new `User` instance.
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }
}
