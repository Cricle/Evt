use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use paopao_domain::{AppError, JwtClaims, UserIdentity};
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    issuer: String,
    expire_seconds: u64,
}

impl JwtService {
    pub fn new(secret: String, issuer: String, expire_seconds: u64) -> Self {
        Self {
            secret,
            issuer,
            expire_seconds,
        }
    }

    pub fn issue(&self, user: UserIdentity) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| AppError::Internal(format!("system clock failure: {err}")))?;
        let claims = JwtClaims {
            sub: user.id.to_string(),
            uid: user.id,
            username: user.username,
            iss: self.issuer.clone(),
            exp: (now.as_secs() + self.expire_seconds) as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|err| AppError::Internal(format!("token signing failed: {err}")))
    }

    pub fn verify(&self, token: &str) -> Result<UserIdentity, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(std::slice::from_ref(&self.issuer));
        let claims = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|err| AppError::Unauthorized(format!("invalid token: {err}")))?
        .claims;

        Ok(UserIdentity {
            id: claims.uid,
            username: claims.username,
        })
    }
}

#[derive(Default, Clone)]
pub struct PasswordService;

impl PasswordService {
    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|err| AppError::Internal(format!("password hashing failed: {err}")))
    }

    pub fn verify(&self, password: &str, password_hash: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(|err| {
            AppError::Unauthorized(format!("invalid stored password hash: {err}"))
        })?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("invalid username or password".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::{JwtService, PasswordService};
    use paopao_domain::{AppError, UserIdentity};

    #[test]
    fn jwt_round_trip_preserves_identity() {
        let jwt = JwtService::new("secret".into(), "paopao".into(), 3600);
        let token = jwt
            .issue(UserIdentity {
                id: 42,
                username: "alice".into(),
            })
            .expect("issue token");

        let identity = jwt.verify(&token).expect("verify token");

        assert_eq!(identity.id, 42);
        assert_eq!(identity.username, "alice");
    }

    #[test]
    fn jwt_rejects_token_signed_by_another_secret() {
        let issuer = "paopao".to_string();
        let token = JwtService::new("secret-a".into(), issuer.clone(), 3600)
            .issue(UserIdentity {
                id: 7,
                username: "bob".into(),
            })
            .expect("issue token");

        let err = JwtService::new("secret-b".into(), issuer, 3600)
            .verify(&token)
            .expect_err("token should be rejected");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[test]
    fn password_hash_verifies_original_password() {
        let password = PasswordService::default();
        let hash = password.hash("hunter2").expect("hash password");

        password
            .verify("hunter2", &hash)
            .expect("password should verify");
    }

    #[test]
    fn password_verify_rejects_incorrect_password() {
        let password = PasswordService::default();
        let hash = password.hash("correct-horse").expect("hash password");

        let err = password
            .verify("wrong-battery", &hash)
            .expect_err("password should be rejected");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
