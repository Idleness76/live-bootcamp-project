#[derive(Debug)]
pub struct Email(String);

impl Email {
    pub fn parse(email: &str) -> Result<Self, String> {
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        // Split by @ and ensure there's exactly one @
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err("Email must contain exactly one @ symbol".to_string());
        }

        let user_part = parts[0];
        let domain_part = parts[1];

        // Validate user part (before @)
        if user_part.is_empty() {
            return Err("Email user part cannot be empty".to_string());
        }

        // Validate domain part (after @)
        if domain_part.is_empty() {
            return Err("Email domain part cannot be empty".to_string());
        }

        if !domain_part.contains('.') {
            return Err("Email domain must contain at least one dot".to_string());
        }

        // Check for consecutive dots
        if domain_part.contains("..") {
            return Err("Email domain cannot contain consecutive dots".to_string());
        }

        // Check domain doesn't start or end with dot
        if domain_part.starts_with('.') || domain_part.ends_with('.') {
            return Err("Email domain cannot start or end with a dot".to_string());
        }

        Ok(Email(email.to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        let valid_emails = vec![
            "user@example.com",
            "test.email@domain.org",
            "user123@test-domain.co.uk",
            "a@b.c",
        ];

        for email in valid_emails {
            assert!(
                Email::parse(email).is_ok(),
                "Expected {} to be valid",
                email
            );
        }
    }

    #[test]
    fn test_empty_email() {
        let result = Email::parse("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email cannot be empty");
    }

    #[test]
    fn test_missing_at_symbol() {
        let result = Email::parse("userexample.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email format");
    }

    #[test]
    fn test_multiple_at_symbols() {
        let result = Email::parse("user@@example.com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Email must contain exactly one @ symbol"
        );

        let result2 = Email::parse("user@example@com");
        assert!(result2.is_err());
        assert_eq!(
            result2.unwrap_err(),
            "Email must contain exactly one @ symbol"
        );
    }

    #[test]
    fn test_empty_user_part() {
        let result = Email::parse("@example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email user part cannot be empty");
    }

    #[test]
    fn test_empty_domain_part() {
        let result = Email::parse("user@");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email domain part cannot be empty");
    }

    #[test]
    fn test_domain_without_dot() {
        let result = Email::parse("user@domain");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Email domain must contain at least one dot"
        );
    }

    #[test]
    fn test_domain_with_consecutive_dots() {
        let result = Email::parse("user@example..com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Email domain cannot contain consecutive dots"
        );
    }

    #[test]
    fn test_domain_starts_with_dot() {
        let result = Email::parse("user@.example.com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Email domain cannot start or end with a dot"
        );
    }

    #[test]
    fn test_domain_ends_with_dot() {
        let result = Email::parse("user@example.com.");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Email domain cannot start or end with a dot"
        );
    }

    #[test]
    fn test_email_access() {
        let email = Email::parse("test@example.com").unwrap();
        assert_eq!(email.as_ref(), "test@example.com");
    }
}
