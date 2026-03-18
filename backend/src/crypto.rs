use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as PHOsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Algorithm, Version,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{AppError, AppResult};

// Vault key size: 256 bits = 32 bytes
pub const VAULT_KEY_SIZE: usize = 32;

/// A zeroize-on-drop wrapper for sensitive key material.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct VaultKey(pub [u8; VAULT_KEY_SIZE]);

impl VaultKey {
    /// Generate a new random vault key.
    pub fn generate() -> Self {
        let mut key = [0u8; VAULT_KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        VaultKey(key)
    }
}

/// Hash a master password using Argon2id with hardened parameters.
///
/// Parameters chosen per OWASP recommendations (2023):
/// - m = 64 MiB, t = 3, p = 4
pub fn hash_master_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut PHOsRng);
    let params = Params::new(65536, 3, 4, None)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Crypto(e.to_string()))?
        .to_string();
    Ok(hash)
}

/// Verify a master password against its stored Argon2id hash.
pub fn verify_master_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}

/// Derive a 256-bit wrapping key from the master password + user id as salt.
///
/// This key is used to encrypt/decrypt the vault key at rest.
pub fn derive_wrapping_key(password: &str, user_id: &str) -> AppResult<[u8; VAULT_KEY_SIZE]> {
    let mut output = [0u8; VAULT_KEY_SIZE];
    let params = Params::new(65536, 3, 4, Some(VAULT_KEY_SIZE))
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    // Use user_id as salt (16-64 bytes required by Argon2)
    // Pad or truncate to 16 bytes
    let mut salt_bytes = [0u8; 16];
    let uid_bytes = user_id.as_bytes();
    let copy_len = uid_bytes.len().min(16);
    salt_bytes[..copy_len].copy_from_slice(&uid_bytes[..copy_len]);
    let salt_str = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    argon2
        .hash_password_into(password.as_bytes(), salt_str.as_str().as_bytes(), &mut output)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    Ok(output)
}

/// Encrypt data with AES-256-GCM, returning base64(nonce || ciphertext).
pub fn encrypt(key: &[u8; VAULT_KEY_SIZE], plaintext: &[u8]) -> AppResult<String> {
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| AppError::Crypto(e.to_string()))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    Ok(B64.encode(combined))
}

/// Decrypt data from base64(nonce || ciphertext) with AES-256-GCM.
pub fn decrypt(key: &[u8; VAULT_KEY_SIZE], encoded: &str) -> AppResult<Vec<u8>> {
    let combined = B64
        .decode(encoded)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    if combined.len() < 12 {
        return Err(AppError::Crypto("Invalid ciphertext".into()));
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::Crypto("Decryption failed".into()))?;
    Ok(plaintext)
}

/// Encrypt a string field, returning base64-encoded ciphertext.
pub fn encrypt_str(key: &[u8; VAULT_KEY_SIZE], s: &str) -> AppResult<String> {
    encrypt(key, s.as_bytes())
}

/// Decrypt a string field from base64-encoded ciphertext.
pub fn decrypt_str(key: &[u8; VAULT_KEY_SIZE], encoded: &str) -> AppResult<String> {
    let bytes = decrypt(key, encoded)?;
    String::from_utf8(bytes).map_err(|e| AppError::Crypto(e.to_string()))
}

/// Generate a secure random password of the given length.
pub fn generate_password(
    length: usize,
    use_uppercase: bool,
    use_digits: bool,
    use_symbols: bool,
) -> String {
    let mut charset: Vec<u8> = b"abcdefghijklmnopqrstuvwxyz".to_vec();
    if use_uppercase {
        charset.extend_from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_digits {
        charset.extend_from_slice(b"0123456789");
    }
    if use_symbols {
        charset.extend_from_slice(b"!@#$%^&*()-_=+[]{}|;:,.<>?");
    }

    let mut password = Vec::with_capacity(length);
    let mut rng = OsRng;
    for _ in 0..length {
        let idx = (rng.next_u32() as usize) % charset.len();
        password.push(charset[idx]);
    }
    String::from_utf8(password).expect("charset is ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_verify() {
        let password = "correct horse battery staple";
        let hash = hash_master_password(password).unwrap();
        assert!(verify_master_password(password, &hash).unwrap());
        assert!(!verify_master_password("wrong password", &hash).unwrap());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = VaultKey::generate();
        let plaintext = "my secret password 123!";
        let encrypted = encrypt_str(&key.0, plaintext).unwrap();
        let decrypted = decrypt_str(&key.0, &encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        // Each encrypt call uses a fresh nonce
        let key = VaultKey::generate();
        let plaintext = "same plaintext";
        let enc1 = encrypt_str(&key.0, plaintext).unwrap();
        let enc2 = encrypt_str(&key.0, plaintext).unwrap();
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_generate_password_length() {
        let pwd = generate_password(24, true, true, true);
        assert_eq!(pwd.len(), 24);
    }

    #[test]
    fn test_derive_wrapping_key_deterministic() {
        let k1 = derive_wrapping_key("password", "user-id-1234").unwrap();
        let k2 = derive_wrapping_key("password", "user-id-1234").unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_wrapping_key_different_passwords() {
        let k1 = derive_wrapping_key("password1", "user-id-1234").unwrap();
        let k2 = derive_wrapping_key("password2", "user-id-1234").unwrap();
        assert_ne!(k1, k2);
    }
}
