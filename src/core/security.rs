//! Authentication & password helpers.
//!
//! - bcrypt for password hashing (cost = 10)
//! - JWT (HS256/HS384/HS512) for access tokens

use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use super::config::settings;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (usually the user email or ID)
    sub: String,
    /// Issued-at (Unix timestamp)
    iat: i64,
    /// Expiration (Unix timestamp)
    exp: i64,
}

fn algorithm() -> Algorithm {
    match settings().algorithm.as_str() {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        other => {
            tracing::warn!(%other, "unrecognized ALGORITHM, defaulting to HS256");
            Algorithm::HS256
        }
    }
}

// ---------------------------------------------------------------------------
// Password helpers
// ---------------------------------------------------------------------------

/// Hash a password with bcrypt (cost factor 10).
///
/// ```
/// use task_manager::core::security::{hash_password, verify_password};
///
/// let hashed = hash_password("hunter2hunter2").unwrap();
/// assert!(hashed.starts_with("$2b$"));
/// assert!(verify_password("hunter2hunter2", &hashed));
/// assert!(!verify_password("wrong-password", &hashed));
/// ```
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    // Cost 10 is a good balance of security vs. latency on modern hardware.
    bcrypt::hash(password, 10)
}

/// Verify a plaintext password against a bcrypt hash.
/// Returns `false` on any error (including malformed hashes).
pub fn verify_password(password: &str, hashed: &str) -> bool {
    bcrypt::verify(password, hashed).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// JWT helpers
// ---------------------------------------------------------------------------

/// Create a signed access token for the given subject.
pub fn create_access_token(subject: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let cfg = settings();
    let now = Utc::now();
    let expire = now + Duration::minutes(cfg.access_token_expire_minutes);

    let claims = Claims {
        sub: subject.to_string(),
        iat: now.timestamp(),
        exp: expire.timestamp(),
    };

    encode(
        &Header::new(algorithm()),
        &claims,
        &EncodingKey::from_secret(cfg.secret_key.as_bytes()),
    )
}

/// Decode and validate an access token.
/// Returns the subject (`sub` claim) on success, `None` otherwise.
///
/// ```
/// use task_manager::core::security::{create_access_token, decode_access_token};
///
/// let token = create_access_token("user@example.com").unwrap();
/// assert_eq!(decode_access_token(&token).as_deref(), Some("user@example.com"));
/// assert_eq!(decode_access_token("not-a-real-token"), None);
/// ```
pub fn decode_access_token(token: &str) -> Option<String> {
    let cfg = settings();

    let mut validation = Validation::new(algorithm());
    // Tolerate a few seconds of clock skew between services.
    validation.leeway = 30;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.secret_key.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims.sub)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let hashed = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hashed));
        assert!(!verify_password("wrong password", &hashed));
    }

    #[test]
    fn jwt_roundtrip() {
        let token = create_access_token("user@example.com").unwrap();
        let subject = decode_access_token(&token);
        assert_eq!(subject.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn jwt_rejects_garbage() {
        assert_eq!(decode_access_token("not-a-real-token"), None);
    }
}
