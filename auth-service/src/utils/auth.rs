use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};
use crate::domain::{BannedTokenStore, Email};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}

// Create cookie with a new JWT auth token
#[tracing::instrument(name = "Generate Auth Cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create cookie and set the value to the passed-in token string
#[tracing::instrument(name = "Create Auth Cookie", skip_all)]
fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .path("/") // apple cookie to all URLs on the server
        .http_only(true) // prevent JavaScript from accessing the cookie
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .build();

    cookie
}

// Create JWT auth token
#[tracing::instrument(name = "Generate Auth Token", skip_all)]
fn generate_auth_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("failed to create 10 minute time delta")?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("failed to add 10 minutes to current time"))?
        .timestamp();

    // Cast exp to a usize, which is what Claims expects
    let exp: usize = exp.try_into().wrap_err(format!(
        "failed to cast exp time to usize. exp time: {}",
        exp
    ))?;

    let sub = email.as_ref().expose_secret().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims)
}

/// Check if JWT auth token is valid by decoding it using the JWT secret
#[tracing::instrument(name = "Validate Token", skip_all)]
pub async fn validate_token(token: &str, banned_store: &dyn BannedTokenStore) -> Result<Claims> {
    match banned_store.is_token_banned(token).await {
        Ok(value) => {
            if value {
                return Err(eyre!("token is banned"));
            }
        }
        Err(e) => return Err(e.into()),
    }

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .wrap_err("failed to decode token")
}

/// Decode JWT and return claims without consulting banned store.
#[tracing::instrument(name = "Decode Claims", skip_all)]
pub fn decode_claims(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )?
    .claims)
}

// Create JWT auth token by encoding claims using the JWT secret
#[tracing::instrument(name = "Create Token", skip_all)]
fn create_token(claims: &Claims) -> Result<String> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .wrap_err("failed to create token")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::RedisBannedTokenStore;
    use secrecy::Secret;
    use std::sync::Arc;
    use tokio::{sync::RwLock, task};

    // Create a fresh Redis store per test by flushing the DB on creation to avoid test interference.
    async fn make_redis_store() -> RedisBannedTokenStore {
        let conn = task::spawn_blocking(|| {
            // Use a non-zero DB index derived from the process id to isolate test data
            // from any other Redis clients that might be using the default DB 0.
            let db_index = (std::process::id() % 15) + 1; // pick DB 1..15
            let redis_url = format!("redis://127.0.0.1/{}", db_index);
            let client = redis::Client::open(redis_url.as_str()).expect("redis url");
            let mut conn = client.get_connection().expect("redis conn");
            let _: () = redis::cmd("FLUSHDB").query(&mut conn).expect("flushdb");
            // Sanity check: ensure there are no banned_token keys left after FLUSHDB.
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg("banned_token:*")
                .query(&mut conn)
                .expect("failed to fetch keys");
            assert!(keys.is_empty(), "redis not empty after FLUSHDB: {:?}", keys);
            conn
        })
        .await
        .expect("spawn_blocking");
        RedisBannedTokenStore::new(Arc::new(RwLock::new(conn)))
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_cookie_returns_jwt() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        let value = cookie.value();
        assert_eq!(value.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_decode_claims_with_valid_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let claims = decode_claims(&token).expect("should decode claims");
        assert_eq!(claims.sub, "test@example.com");
        assert!(claims.exp > Utc::now().timestamp() as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let banned_store = make_redis_store().await;

        let res = validate_token(&token, &banned_store).await;
        assert!(
            res.is_ok(),
            "expected token to validate, got: {:?}",
            res.err()
        );
        let result = res.unwrap();

        assert_eq!(result.sub, "test@example.com");
        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::minutes(9))
            .expect("valid timestamp")
            .timestamp();
        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let banned_store = make_redis_store().await;
        let result = validate_token("invalid_token", &banned_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_with_banned_token() {
        let email = Email::parse(Secret::new("test@example.com".to_owned())).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let mut banned_store = make_redis_store().await;

        banned_store.add_banned_token(&token).await.unwrap();
        let result = validate_token(&token, &banned_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_banned_token_isolation() {
        let email1 = Email::parse(Secret::new("one@example.com".to_owned())).unwrap();
        let email2 = Email::parse(Secret::new("two@example.com".to_owned())).unwrap();
        let token1 = generate_auth_token(&email1).unwrap();
        let token2 = generate_auth_token(&email2).unwrap();
        let mut banned_store = make_redis_store().await;

        // Ban token1 only
        banned_store.add_banned_token(&token1).await.unwrap();

        // token1 should be rejected
        let res1 = validate_token(&token1, &banned_store).await;
        assert!(res1.is_err(), "banned token should not validate");

        // token2 should still be valid
        let res2 = validate_token(&token2, &banned_store).await;
        assert!(res2.is_ok(), "non-banned token should validate");
    }
}
