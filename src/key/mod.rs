//! `WireGuard` key generation and verification using `x25519-dalek`.

use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

/// A Curve25519 key pair for `WireGuard`, encoded as base64 strings.
pub struct KeyPair {
    /// Base64-encoded private key (32 bytes → 44 characters).
    pub private_key: String,
    /// Base64-encoded public key derived from the private key.
    pub public_key: String,
}

/// Generate a random Curve25519 key pair suitable for `WireGuard`.
pub fn generate_keypair() -> KeyPair {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    KeyPair {
        private_key: BASE64.encode(secret.to_bytes()),
        public_key: BASE64.encode(public.to_bytes()),
    }
}

/// Generate a random 256-bit pre-shared key, returned as a base64 string.
pub fn generate_preshared_key() -> String {
    let mut key = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut key);
    BASE64.encode(key)
}

/// Verify that the given private key derives the given public key.
///
/// Returns `Ok(false)` if the keys are valid base64 but do not match,
/// or if either key has an unexpected length.
#[allow(dead_code)]
pub fn verify_keypair(private_key_str: &str, public_key_str: &str) -> Result<bool> {
    if private_key_str.len() != 44 || public_key_str.len() != 44 {
        return Ok(false);
    }

    let private_bytes = BASE64.decode(private_key_str).map_err(|e| anyhow!("{e}"))?;
    let public_bytes = BASE64.decode(public_key_str).map_err(|e| anyhow!("{e}"))?;

    if private_bytes.len() != 32 || public_bytes.len() != 32 {
        return Ok(false);
    }

    let secret_bytes: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| anyhow!("Private key must be exactly 32 bytes"))?;
    let secret = StaticSecret::from(secret_bytes);
    let derived_public = PublicKey::from(&secret);

    Ok(derived_public.as_bytes() == public_bytes.as_slice())
}

#[cfg(test)]
mod tests;
