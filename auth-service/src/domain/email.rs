use validator::validate_email;

#[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Email {
    pub fn parse(s: String) -> Result<Email, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email.", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Word;
    use fake::Fake;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_valid_emails_are_idempotent(email: String) -> bool {
        match Email::parse(email.clone()) {
            Ok(parsed) => Email::parse(parsed.as_ref().to_string()).is_ok(),
            Err(_) => true,
        }
    }

    #[quickcheck]
    fn test_email_without_at_is_invalid(mut s: String) -> bool {
        s.retain(|c| c != '@');
        if s.is_empty() {
            return true;
        }
        Email::parse(s).is_err()
    }

    #[quickcheck]
    fn test_email_with_multiple_at_is_invalid(s: String) -> bool {
        if s.matches('@').count() > 1 {
            Email::parse(s).is_err()
        } else {
            true
        }
    }

    #[quickcheck]
    fn test_empty_parts_are_invalid(prefix: String, suffix: String) -> bool {
        let test_cases = vec![format!("@{}", suffix), format!("{}@", prefix)];

        test_cases
            .into_iter()
            .all(|email| Email::parse(email).is_err())
    }

    #[test]
    fn test_generated_safe_emails_are_valid() {
        for _ in 0..100 {
            let email: String = SafeEmail().fake();
            assert!(
                Email::parse(email.clone()).is_ok(),
                "Generated safe email should be valid: {}",
                email
            );
        }
    }

    #[test]
    fn test_random_strings_mostly_invalid() {
        let valid_count = (0..1000)
            .map(|_| Word().fake::<String>())
            .filter(|s| Email::parse(s.clone()).is_ok())
            .count();

        assert!(
            valid_count < 100,
            "Too many random strings were valid emails: {}/1000",
            valid_count
        );
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
                Email::parse(email.to_string()).is_ok(),
                "Expected {} to be valid",
                email
            );
        }
    }

    #[test]
    fn test_email_access() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        assert_eq!(email.as_ref(), "test@example.com");
    }
}
