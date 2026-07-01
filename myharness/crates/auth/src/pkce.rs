//! PKCE (Proof Key for Code Exchange, RFC 7636) + state generator.
//!
//! `code_verifier`: 43-128 char [A-Z][a-z][0-9]-._~
//! `code_challenge`: `S256(code_verifier)` base64url-encoded

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
    pub method: PkceMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkceMethod {
    S256,
}

impl PkceMethod {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PkceMethod::S256 => "S256",
        }
    }
}

#[must_use]
pub fn generate_pkce() -> PkcePair {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    PkcePair {
        verifier,
        challenge,
        method: PkceMethod::S256,
    }
}

/// random state (CSRF 방지) — URL-safe 16 byte
#[must_use]
pub fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_is_url_safe() {
        let p = generate_pkce();
        assert!(
            p.verifier
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!p.verifier.is_empty());
        assert!(!p.challenge.is_empty());
        assert_eq!(p.method, PkceMethod::S256);
    }

    #[test]
    fn pkce_unique_each_call() {
        let a = generate_pkce();
        let b = generate_pkce();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
    }

    #[test]
    fn pkce_challenge_matches_sha256_of_verifier() {
        let p = generate_pkce();
        let mut h = Sha256::new();
        h.update(p.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(h.finalize());
        assert_eq!(p.challenge, expected);
    }

    #[test]
    fn state_is_22_chars_url_safe() {
        let s = generate_state();
        // 16 byte → ~22 char base64url
        assert_eq!(s.len(), 22);
        assert!(
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn state_unique_each_call() {
        let a = generate_state();
        let b = generate_state();
        assert_ne!(a, b);
    }
}
