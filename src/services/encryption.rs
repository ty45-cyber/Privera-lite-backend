use ring::aead::{self, Aad, BoundKey, Nonce, NonceSequence, SealingKey, OpeningKey, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use crate::errors::{AppError, AppResult};

struct CounterNonce([u8; 12]);

impl CounterNonce {
    fn from_bytes(bytes: [u8; 12]) -> Self {
        Self(bytes)
    }
}

impl NonceSequence for CounterNonce {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        Nonce::try_assume_unique_for_key(&self.0)
    }
}

/// Encrypts plaintext with AES-256-GCM.
/// Returns base64(nonce || ciphertext_with_tag).
pub fn encrypt(key_hex: &str, plaintext: &str) -> AppResult<String> {
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| AppError::Encryption(format!("Key decode: {e}")))?;

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| AppError::Encryption("Nonce generation failed".into()))?;

    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| AppError::Encryption("Key init failed".into()))?;
    let nonce_seq = CounterNonce::from_bytes(nonce_bytes);
    let mut sealing = SealingKey::new(unbound, nonce_seq);

    let mut in_out = plaintext.as_bytes().to_vec();
    sealing
        .seal_in_place_append_tag(Aad::empty(), &mut in_out)
        .map_err(|_| AppError::Encryption("Seal failed".into()))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&in_out);
    Ok(B64.encode(&combined))
}

/// Decrypts base64(nonce || ciphertext_with_tag).
pub fn decrypt(key_hex: &str, ciphertext_b64: &str) -> AppResult<String> {
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| AppError::Encryption(format!("Key decode: {e}")))?;

    let combined = B64.decode(ciphertext_b64)
        .map_err(|_| AppError::Encryption("Base64 decode failed".into()))?;

    if combined.len() < 12 {
        return Err(AppError::Encryption("Ciphertext too short".into()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce_arr: [u8; 12] = nonce_bytes.try_into()
        .map_err(|_| AppError::Encryption("Nonce slice failed".into()))?;

    let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| AppError::Encryption("Key init failed".into()))?;
    let nonce_seq = CounterNonce::from_bytes(nonce_arr);
    let mut opening = OpeningKey::new(unbound, nonce_seq);

    let mut buf = ciphertext.to_vec();
    let plaintext = opening
        .open_in_place(Aad::empty(), &mut buf)
        .map_err(|_| AppError::Encryption("Decrypt failed".into()))?;

    std::str::from_utf8(plaintext)
        .map(|s| s.to_string())
        .map_err(|_| AppError::Encryption("UTF-8 decode failed".into()))
}
