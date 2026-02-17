//! Tests for key generation and verification.

use super::*;

// ── generate_keypair ─────────────────────────────────────────────

#[test]
fn test_generate_keypair_returns_base64_keys() {
    let kp = generate_keypair();
    assert_eq!(kp.private_key.len(), 44);
    assert_eq!(kp.public_key.len(), 44);

    let priv_bytes = BASE64.decode(&kp.private_key).unwrap();
    let pub_bytes = BASE64.decode(&kp.public_key).unwrap();
    assert_eq!(priv_bytes.len(), 32);
    assert_eq!(pub_bytes.len(), 32);
}

#[test]
fn test_generate_keypair_keys_are_distinct() {
    let kp = generate_keypair();
    assert_ne!(kp.private_key, kp.public_key);
}

#[test]
fn test_generate_keypair_is_nondeterministic() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();
    assert_ne!(kp1.private_key, kp2.private_key);
    assert_ne!(kp1.public_key, kp2.public_key);
}

#[test]
fn test_generate_keypair_produces_valid_pair() {
    let kp = generate_keypair();
    let result = verify_keypair(&kp.private_key, &kp.public_key).unwrap();
    assert!(result);
}

// ── generate_preshared_key ───────────────────────────────────────

#[test]
fn test_generate_preshared_key_is_valid_base64() {
    let psk = generate_preshared_key();
    assert_eq!(psk.len(), 44);
    let bytes = BASE64.decode(&psk).unwrap();
    assert_eq!(bytes.len(), 32);
}

#[test]
fn test_generate_preshared_key_is_nondeterministic() {
    let psk1 = generate_preshared_key();
    let psk2 = generate_preshared_key();
    assert_ne!(psk1, psk2);
}

// ── verify_keypair ───────────────────────────────────────────────

#[test]
fn test_verify_keypair_valid_pair() {
    let kp = generate_keypair();
    assert!(verify_keypair(&kp.private_key, &kp.public_key).unwrap());
}

#[test]
fn test_verify_keypair_wrong_public_key() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();
    assert!(!verify_keypair(&kp1.private_key, &kp2.public_key).unwrap());
}

#[test]
fn test_verify_keypair_short_keys() {
    assert!(!verify_keypair("short", "short").unwrap());
}

#[test]
fn test_verify_keypair_invalid_base64() {
    // 44 chars but not valid base64
    let bad = "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!?";
    assert_eq!(bad.len(), 44);
    assert!(verify_keypair(bad, bad).is_err());
}

// ── Additional key tests ─────────────────────────────────────────

#[test]
fn test_public_key_derives_from_private() {
    let kp = generate_keypair();
    let priv_bytes: [u8; 32] = BASE64.decode(&kp.private_key).unwrap().try_into().unwrap();
    let secret = x25519_dalek::StaticSecret::from(priv_bytes);
    let derived = x25519_dalek::PublicKey::from(&secret);
    assert_eq!(BASE64.encode(derived.as_bytes()), kp.public_key);
}

#[test]
fn test_preshared_key_decoded_is_32_bytes() {
    let psk = generate_preshared_key();
    let decoded = BASE64.decode(&psk).unwrap();
    assert_eq!(decoded.len(), 32);
}
