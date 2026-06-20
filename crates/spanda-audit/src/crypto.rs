//! crypto support for Spanda.
//!
use crate::record::Hash;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

/// Derive a 32-byte Ed25519 seed from arbitrary key material (UTF-8).
fn seed_bytes(material: &str) -> [u8; 32] {
    let digest = Sha256::digest(material.as_bytes());
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&digest);
    seed
}

fn signing_key_from_material(material: &str) -> SigningKey {
    // Signing key from material.
    //
    // Parameters:
    // - `material` — input value
    //
    // Returns:
    // SigningKey.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::signing_key_from_material(material);

    // Produce from bytes as the result.
    SigningKey::from_bytes(&seed_bytes(material))
}

pub(crate) fn is_hex_public_key(key: &str) -> bool {
    //
    // Parameters:
    // - `key` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::is_hex_public_key(key);

    // Produce is ascii hexdigit as the result.
    key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit())
}

/// Derive hex-encoded Ed25519 public key from signing material.
pub fn public_key_from_material(material: &str) -> String {
    // Public key from material.
    //
    // Parameters:
    // - `material` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::public_key_from_material(material);

    // Produce encode as the result.
    hex::encode(
        signing_key_from_material(material)
            .verifying_key()
            .to_bytes(),
    )
}

/// Compute SHA-256 hash of UTF-8 data, returned as hex string.
pub fn sha256(data: &str) -> Hash {
    // Sha256.
    //
    // Parameters:
    // - `data` — input value
    //
    // Returns:
    // Hash.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::sha256(data);

    // Create mutable hasher for accumulating results.
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    Hash(hex::encode(hasher.finalize()))
}

/// Sign data with Ed25519 using signing material or hex-encoded private seed.
pub fn sign(data: &str, key_material: &str) -> String {
    // Sign.
    //
    // Parameters:
    // - `data` — input value
    // - `key_material` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::sign(data, key_material);

    // Compute sk for the following logic.
    let sk = if key_material.len() == 64 && key_material.chars().all(|c| c.is_ascii_hexdigit()) {
        // Handle the success value from decode.
        if let Ok(bytes) = hex::decode(key_material) {
            // Take the branch when len equals 32.
            if bytes.len() == 32 {
                SigningKey::from_bytes(&bytes.try_into().expect("32-byte seed"))
            } else {
                signing_key_from_material(key_material)
            }
        } else {
            signing_key_from_material(key_material)
        }
    } else {
        signing_key_from_material(key_material)
    };
    hex::encode(sk.sign(data.as_bytes()).to_bytes())
}
/// Verify an Ed25519 signature.
///
/// `key` may be a hex-encoded public key (64 hex chars) or signing material (derives public key).
pub fn verify_signature(data: &str, signature: &str, key: &str) -> bool {
    // Verify signature.
    //
    // Parameters:
    // - `data` — input value
    // - `signature` — input value
    // - `key` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_audit::crypto::verify_signature(data, signature, key);

    // Compute sig bytes for the following logic.
    let sig_bytes = match hex::decode(signature) {
        Ok(bytes) if bytes.len() == 64 => bytes,
        _ => return false,
    };
    let sig = Signature::from_bytes(
        &sig_bytes
            .try_into()
            .expect("signature length checked above"),
    );
    let verify_with = |vk: &VerifyingKey| vk.verify_strict(data.as_bytes(), &sig).is_ok();

    // Take this path when is hex public key(key).
    if is_hex_public_key(key) {
        // Handle the success value from decode.
        if let Ok(pk) = hex::decode(key) {
            // Take the branch when len equals 32.
            if pk.len() == 32 {
                // Handle the success value from expect.
                if let Ok(vk) = VerifyingKey::from_bytes(&pk.try_into().expect("32-byte key")) {
                    return verify_with(&vk);
                }
            }
        }
        return false;
    }
    verify_with(&signing_key_from_material(key).verifying_key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_deterministic() {
        // Sha256 is deterministic.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_audit::crypto::sha256_is_deterministic();

        let h1 = sha256("hello");
        let h2 = sha256("hello");
        assert_eq!(h1.0, h2.0);
        assert_eq!(h1.0.len(), 64);
    }

    #[test]
    fn sign_and_verify_roundtrip_with_material() {
        // Sign and verify roundtrip with material.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_audit::crypto::sign_and_verify_roundtrip_with_material();

        let sig = sign("payload", "device-key-001");
        assert_eq!(sig.len(), 128);
        assert!(verify_signature("payload", &sig, "device-key-001"));
        assert!(!verify_signature("payload", &sig, "wrong-key"));
    }

    #[test]
    fn verify_with_derived_public_key() {
        // Verify with derived public key.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_audit::crypto::verify_with_derived_public_key();

        let material = "device-key-001";
        let sig = sign("payload", material);
        let pubkey = public_key_from_material(material);
        assert!(verify_signature("payload", &sig, &pubkey));
    }

    #[test]
    fn signatures_differ_from_legacy_hmac() {
        // Signatures differ from legacy hmac.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_audit::crypto::signatures_differ_from_legacy_hmac();

        let sig = sign("payload", "device-key-001");
        assert_ne!(sig.len(), 64);
    }
}
