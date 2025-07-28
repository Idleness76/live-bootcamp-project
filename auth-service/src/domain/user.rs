use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl User {
    /// Creates a new User instance with the provided credentials and 2FA requirement.
    ///
    /// # Arguments
    /// * `email` - The user's email address
    /// * `password` - The user's password (should be hashed before storage)
    /// * `requires_2fa` - Whether the user requires two-factor authentication
    ///
    /// # Returns
    /// A new `User` instance with the specified properties
    ///
    /// # Example
    /// ```
    /// use auth_service::domain::User;
    ///
    /// let user = User::new(
    ///     "user@example.com".to_string(),
    ///     "hashed_password".to_string(),
    ///     true
    /// );
    /// ```
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }
}
