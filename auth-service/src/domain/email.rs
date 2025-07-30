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
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Word;
    use fake::Fake;
    use quickcheck_macros::quickcheck;

    // QuickCheck property tests (highest priority)
    #[quickcheck]
    fn test_valid_emails_are_idempotent(email: String) -> bool {
        // If parsing succeeds once, it should always succeed
        match Email::parse(&email) {
            Ok(parsed) => Email::parse(parsed.as_ref()).is_ok(),
            Err(_) => true, // Invalid emails should stay invalid
        }
    }

    #[quickcheck]
    fn test_email_without_at_is_invalid(mut s: String) -> bool {
        // Remove all @ symbols
        s.retain(|c| c != '@');
        if s.is_empty() {
            return true; // Empty strings are handled separately
        }
        Email::parse(&s).is_err()
    }

    #[quickcheck]
    fn test_email_with_multiple_at_is_invalid(s: String) -> bool {
        if s.matches('@').count() > 1 {
            Email::parse(&s).is_err()
        } else {
            true // Not testing single @ case here
        }
    }

    #[quickcheck]
    fn test_empty_parts_are_invalid(prefix: String, suffix: String) -> bool {
        let test_cases = vec![
            format!("@{}", suffix), // Empty user part
            format!("{}@", prefix), // Empty domain part
        ];

        test_cases.iter().all(|email| Email::parse(email).is_err())
    }

    // Fake-based tests (medium priority)
    #[test]
    fn test_generated_safe_emails_are_valid() {
        for _ in 0..100 {
            let email: String = SafeEmail().fake();
            assert!(
                Email::parse(&email).is_ok(),
                "Generated safe email should be valid: {}",
                email
            );
        }
    }

    #[test]
    fn test_random_strings_mostly_invalid() {
        let mut valid_count = 0;
        let total_tests = 1000;

        for _ in 0..total_tests {
            let random_string: String = Word().fake();
            if Email::parse(&random_string).is_ok() {
                valid_count += 1;
            }
        }

        // Most random words shouldn't be valid emails
        assert!(
            valid_count < total_tests / 10,
            "Too many random strings were valid emails: {}/{}",
            valid_count,
            total_tests
        );
    }

    // Custom tests (lowest priority) - only for specific edge cases not covered above
    #[test]
    fn test_specific_domain_validation_edge_cases() {
        let invalid_patterns = vec![
            ("", "Email cannot be empty"),
            (
                "user@.starts-with-dot.com",
                "Email domain cannot start or end with a dot",
            ),
            (
                "user@ends-with-dot.com.",
                "Email domain cannot start or end with a dot",
            ),
            (
                "user@consecutive..dots.com",
                "Email domain cannot contain consecutive dots",
            ),
            ("user@domain", "Email domain must contain at least one dot"),
        ];

        for (pattern, expected_error) in invalid_patterns {
            let result = Email::parse(pattern);
            assert!(result.is_err(), "Should reject: {}", pattern);
            assert_eq!(result.unwrap_err(), expected_error);
        }
    }

    #[test]
    fn test_email_access() {
        let email = Email::parse("test@example.com").unwrap();
        assert_eq!(email.as_ref(), "test@example.com");
    }

    #[test]
    fn test_known_valid_examples() {
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
}
