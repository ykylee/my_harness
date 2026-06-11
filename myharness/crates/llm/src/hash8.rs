//! SHA-256 8-char content hash utility (W21, D-64 F-2).
//!
//! backup filename 에 content fingerprint 로 사용. 64^8 = 2.8×10^14 space →
//! collision 사실상 0. 동일 content → 동일 hash 보장 (deterministic).

/// content 의 SHA-256 hex digest 앞 8 char.
///
/// `empty` 입력은 `e3b0c442` (SHA-256("") 의 hex prefix). 다른 content 와
/// 절대 충돌하지 않음.
#[must_use] 
pub fn content_hash_8(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(content)
        .iter()
        .take(4)
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash8_identical_content_produces_identical_hash() {
        let a = content_hash_8(b"llama3.1:8b");
        let b = content_hash_8(b"llama3.1:8b");
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
    }

    #[test]
    fn hash8_different_content_produces_different_hash() {
        let a = content_hash_8(b"llama3.1:8b");
        let b = content_hash_8(b"qwen2.5:14b");
        assert_ne!(a, b);
    }

    #[test]
    fn hash8_empty_input_returns_known_sha256_prefix() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = content_hash_8(b"");
        assert_eq!(h, "e3b0c442");
    }
}
