use crate::config::CONFIG;
use crate::errors::ApiError;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use argon2rs::argon2i_simple;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PrivateClaim {
    pub user_id: Uuid,
    pub email: String,
    exp: i64,
}

impl PrivateClaim {
    pub fn new(user_id: Uuid, email: String) -> Self {
        Self {
            user_id,
            email,
            exp: (Utc::now() + Duration::hours(CONFIG.jwt_expiration)).timestamp(),
        }
    }
}

/// Create a json web token (JWT)
pub fn create_jwt(private_claim: PrivateClaim) -> Result<String, ApiError> {
    let encoding_key = EncodingKey::from_secret(&CONFIG.jwt_key.as_ref());
    encode(
        &Header::default(),
        &private_claim,
        &encoding_key,
    )
    .map_err(|e| ApiError::CannotEncodeJwtToken(e.to_string()))
}

/// Decode a json web token (JWT)
pub fn decode_jwt(token: &str) -> Result<PrivateClaim, ApiError> {
    let decoding_key = DecodingKey::from_secret(&CONFIG.jwt_key.as_ref());
    decode::<PrivateClaim>(token, &decoding_key, &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| ApiError::CannotDecodeJwtToken(e.to_string()))
}

/// Encrypt a password
///
/// Uses the argon2i algorithm.
/// auth_salt is environment-configured.
pub fn hash(password: &str, salt: &String) -> String {
    let masked = mask_str(&salt, &CONFIG.auth_salt);
    argon2i_simple(&password, &masked)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Gets the identidy service for injection into an Actix app
pub fn get_identity_service() -> IdentityService<CookieIdentityPolicy> {
    IdentityService::new(
        CookieIdentityPolicy::new(&CONFIG.session_key.as_ref())
            .name(&CONFIG.session_name)
            .max_age_time(chrono::Duration::minutes(CONFIG.session_timeout))
            .secure(CONFIG.session_secure),
    )
}


fn mask_str(str: &String, mask : &String) -> String{
    let mut strb = str.clone().into_bytes();
    let maskb = mask.clone().into_bytes();
    let str_len = strb.len();
    let mask_len = maskb.len();
    let mut i = 0;
    let mut m = 0;
    while i < str_len{
        if m >= mask_len {
            m = 0;
        }
        strb[i] = (strb[i].wrapping_add(maskb[m])) % 128;
        i += 1;
        m+= 1;
    }
    return String::from_utf8(strb).unwrap();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    static EMAIL: &str = "test@test.com";

    #[test]
    fn it_hashes_a_password() {
        let password = "password";
        let salt = thread_rng().sample_iter(&Alphanumeric).take(32).collect::<String>();
        let hashed = hash(password, &salt);
        assert_ne!(password, hashed);
    }

    #[test]
    fn it_matches_2_hashed_passwords() {
        let password = "password";
        let salt = thread_rng().sample_iter(&Alphanumeric).take(32).collect::<String>();
        let hashed = hash(password, &salt);
        let hashed_again = hash(password, &salt);
        assert_eq!(hashed, hashed_again);
    }

    #[test]
    fn it_creates_a_jwt() {
        let private_claim = PrivateClaim::new(Uuid::new_v4(), EMAIL.into());
        let jwt = create_jwt(private_claim);
        assert!(jwt.is_ok());
    }

    #[test]
    fn it_decodes_a_jwt() {
        let private_claim = PrivateClaim::new(Uuid::new_v4(), EMAIL.into());
        let jwt = create_jwt(private_claim.clone()).unwrap();
        let decoded = decode_jwt(&jwt).unwrap();
        assert_eq!(private_claim, decoded);
    }


    #[test]
    fn it_masks_a_string() {
        let salt = "salt1salt2salt3".to_string();
        let mask = "mask52632".to_string();
        let masked = mask_str(&salt, &mask);
        assert_ne!(masked, "".to_string());
    }
}
