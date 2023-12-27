use once_cell::sync::Lazy;

use crate::errors::ApiError;


pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16)));

const _SALT: &[u8] = b"supersecuresalt";

// PLEASE NOTE THIS IS ONLY FOR DEMO PLEASE DO MORE RESEARCH FOR PRODUCTION USE
pub fn _hash_password(password: &str) -> Result<String, ApiError> {
    let config = argon2::Config {
        secret: SECRET_KEY.as_bytes(),
        ..argon2::Config::rfc9106_low_mem()
    };
    argon2::hash_encoded(password.as_bytes(), _SALT, &config).map_err(|err| {
        dbg!(err);
        ApiError::InternalServerError("InternalServerError hash_encoded".to_string())
    })
}

pub fn _verify(hash: &str, password: &str) -> Result<bool, ApiError> {
    argon2::verify_encoded_ext(hash, password.as_bytes(), SECRET_KEY.as_bytes(), &[]).map_err(
        |err| {
            dbg!(err);
            ApiError::Unauthorized("Unauthorized".to_string())
        },
    )
}

#[cfg(test)]
mod tests {
    use std::env;

    use actix_web::cookie::Key;

    use super::SECRET_KEY;

    #[test]
    fn secret_key_default() {
        env::remove_var("SECRET_KEY");

        let key_vec = &SECRET_KEY.as_bytes().to_vec();

        Key::from(key_vec);
    }
}