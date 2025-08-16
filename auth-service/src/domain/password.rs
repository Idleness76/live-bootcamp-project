use sqlx::{postgres::PgValueRef, Decode, Postgres, Type};

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
pub struct Password(String);

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Password {
    pub fn parse(s: String) -> Result<Password, String> {
        if s.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if s.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }

        if !s.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if !s.chars().any(|c| c.is_lowercase()) {
            return Err("Password must contain at least one lowercase letter".to_string());
        }

        if !s.chars().any(|c| c.is_ascii_digit()) {
            return Err("Password must contain at least one digit".to_string());
        }

        if !s.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err("Password must contain at least one special character".to_string());
        }

        Ok(Password(s.to_string()))
    }
}

// Manual impls for sqlx traits
impl<'r> Decode<'r, Postgres> for Password {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        Password::parse(s).map_err(|e| e.into())
    }
}

impl Type<Postgres> for Password {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    #[derive(Clone, Debug)]
    struct ValidPassword(String);

    impl Arbitrary for ValidPassword {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let base_length = u8::arbitrary(g) % 20 + 8; // 8-27 characters
            let mut password = String::new();

            // Ensure at least one of each required character type
            let uppercase = (b'A' + (u8::arbitrary(g) % 26)) as char;
            password.push(uppercase);

            let lowercase = (b'a' + (u8::arbitrary(g) % 26)) as char;
            password.push(lowercase);

            // Generate digit using quickcheck approach
            let digit = (b'0' + (u8::arbitrary(g) % 10)) as char;
            password.push(digit);

            // Generate special character
            let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            let special_char = special_chars
                .chars()
                .nth(usize::arbitrary(g) % special_chars.len())
                .unwrap();
            password.push(special_char);

            // Fill remaining length with valid characters
            for _ in 4..base_length {
                let char_type = u8::arbitrary(g) % 4;
                match char_type {
                    0 => {
                        // Uppercase letter
                        let c = (b'A' + (u8::arbitrary(g) % 26)) as char;
                        password.push(c);
                    }
                    1 => {
                        // Lowercase letter
                        let c = (b'a' + (u8::arbitrary(g) % 26)) as char;
                        password.push(c);
                    }
                    2 => {
                        // Digit
                        let c = (b'0' + (u8::arbitrary(g) % 10)) as char;
                        password.push(c);
                    }
                    _ => {
                        // Special character
                        let c = special_chars
                            .chars()
                            .nth(usize::arbitrary(g) % special_chars.len())
                            .unwrap();
                        password.push(c);
                    }
                }
            }

            // Shuffle to randomize character positions
            let mut chars: Vec<char> = password.chars().collect();
            for i in 0..chars.len() {
                let j = usize::arbitrary(g) % chars.len();
                chars.swap(i, j);
            }

            ValidPassword(chars.into_iter().collect())
        }
    }

    #[quickcheck]
    fn prop_valid_passwords_always_parse_successfully(valid_pw: ValidPassword) -> bool {
        Password::parse(valid_pw.0).is_ok()
    }

    #[quickcheck]
    fn prop_parsed_password_equals_input(valid_pw: ValidPassword) -> bool {
        match Password::parse(valid_pw.0.clone()) {
            Ok(password) => password.as_ref() == valid_pw.0,
            Err(_) => false,
        }
    }

    #[quickcheck]
    fn prop_short_passwords_always_fail(short_input: String) -> bool {
        if short_input.len() < 8 {
            Password::parse(short_input).is_err()
        } else {
            true
        }
    }

    #[test]
    fn test_fake_short_passwords_fail() {
        use fake::faker::internet::en::Password as FakePassword;

        for _ in 0..50 {
            let fake_password: String = FakePassword(1..8).fake();
            assert!(Password::parse(fake_password).is_err());
        }
    }

    #[test]
    fn test_empty_password_fails() {
        assert!(Password::parse(String::new()).is_err());
    }

    #[test]
    fn test_known_valid_password() {
        assert!(Password::parse("Password123!".to_string()).is_ok());
    }
}
