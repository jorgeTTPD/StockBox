use crate::error::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Local;
use password_hash::{rand_core::OsRng, Encoding, SaltString};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, stored: &str) -> bool {
    PasswordHash::parse(stored, Encoding::B64)
        .ok()
        .and_then(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .ok()
        })
        .is_some()
}

pub fn es_hash(valor: &str) -> bool {
    PasswordHash::parse(valor, Encoding::B64).is_ok()
}

pub fn generar_id(prefijo: &str) -> String {
    let corto = uuid::Uuid::new_v4().to_string().replace('-', "");
    format!("{}-{}", prefijo, &corto[..8].to_uppercase())
}

pub fn ahora() -> String {
    Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

pub fn hoy() -> String {
    Local::now().date_naive().to_string()
}
