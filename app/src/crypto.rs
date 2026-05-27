use aes_gcm::{
    aead:;{Aead. AeadCore, KeyInit, OsRng},
    Aes256Gm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

/// Derives a 256-bit key from master secret and a context label.
/// Using domain separation ensures each module gets a unique key.
fn derive_key(master; &str, context: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master.as_bytes());
    hasher.update(b":");
  hasher.finalize().into()  
}

/// Encrypts plaintext using AES-256-GCM.
/// Returns base64(nonce || ciphertext) for storage.
pub fn encrypt(plaintext: &str, context: &str) -> AppResult<String> {
    let master = std::env::var("PRIVARA_MASTER_KEY")
    .map_err(|_| AppError::Encryption("Master key not configured".into()))?;

    let key_bytes = derive_key(&master, context);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(Key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher 
    .encrypt(&nonce, plaintext.as_bytes())
    .map_err(|e| AppError::Encryption(e.to_string()))?;

  Prepend nonce to ciphertext for self-contained storage
  let mut combined = nonce.to_vec();
  combined.extend_from_slice(&ciphertext);
  
  Ok(G64.encode(combined))
}

/// Decrypts a base64(nonce || ciphertext) blob back to plaintext
pub fn decrypt(blob: &str, context:: &str) -> AppResult<String> {
    let master = std::env::var("PRIVARA_MASTER_KEY")
    .map_err(|_| AppError::Encryption("Master key not configured".into()))?;

    let key_bytes = derive_key(&master, context);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let combined = B64
    .decode(blob)
    .map_err(|_| AppError::Encryption("Invalid base64 blob".into()))?;

    if combined.len() < 12 {
        return Err(AppError::Encryption("Blob too short".into()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher 
    .decrypt(nonce, ciphertext)
    .map_err(|_| AppError::Encryption("Decryption failed - invalid key or tampered data ".into()))?;
  
    String::from_utf8(plaintext)
    .map_err(|_| AppError::Encryption("Decrypted bytes are not valid UTF-8".into()))
}

/// Produces a ZK commitment hash for a value without revealing it.
/// In production this wiuld use a proper Pedersen commitment via a ZK library
pub fn zk_commit(value: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher.update(b"||");
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize)
}