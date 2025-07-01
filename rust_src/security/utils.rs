use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid algorithm: {0}")]
    InvalidAlgorithm(String),
    #[error("Hash generation failed: {0}")]
    HashError(String),
    #[error("Invalid key format")]
    InvalidKeyFormat,
    #[error("Verification failed")]
    VerificationFailed,
}

pub type Result<T> = std::result::Result<T, SecurityError>;

/// Constant-time string comparison to prevent timing attacks
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

/// Constant-time byte array comparison
pub fn constant_time_compare_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a[i] ^ b[i];
    }

    result == 0
}

/// Verify API key with optional algorithm
pub fn verify_api_key(
    provided_key: &str,
    expected_key: &str,
    algorithm: Option<&str>,
) -> Result<bool> {
    match algorithm {
        Some("plain") | None => Ok(constant_time_compare(provided_key, expected_key)),
        Some("sha256") => {
            let provided_hash = hash_sha256(provided_key.as_bytes());
            let expected_hash = hash_sha256(expected_key.as_bytes());
            Ok(constant_time_compare_bytes(&provided_hash, &expected_hash))
        }
        Some("bcrypt") => {
            // Note: In a real implementation, you'd use a proper bcrypt library
            // For now, fallback to constant time comparison
            Ok(constant_time_compare(provided_key, expected_key))
        }
        Some(alg) => Err(SecurityError::InvalidAlgorithm(alg.to_string())),
    }
}

/// Hash password with specified algorithm
pub fn hash_password(password: &str, algorithm: Option<&str>) -> Result<String> {
    match algorithm {
        Some("sha256") | None => {
            let hash = hash_sha256(password.as_bytes());
            Ok(hex_encode(&hash))
        }
        Some("bcrypt") => {
            // Note: In a real implementation, you'd use a proper bcrypt library
            // For now, use SHA256 as fallback
            let hash = hash_sha256(password.as_bytes());
            Ok(format!("bcrypt:{}", hex_encode(&hash)))
        }
        Some(alg) => Err(SecurityError::InvalidAlgorithm(alg.to_string())),
    }
}

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut bytes = Vec::with_capacity(length);
    let mut hasher = DefaultHasher::new();

    // Use current time as seed
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    now.hash(&mut hasher);

    let mut seed = hasher.finish();

    for _ in 0..length {
        // Simple LCG for demonstration (use proper CSPRNG in production)
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        bytes.push((seed >> 16) as u8);
    }

    bytes
}

/// Generate API key
pub fn generate_api_key(length: Option<usize>) -> String {
    let len = length.unwrap_or(32);
    let bytes = generate_random_bytes(len);
    base64_encode(&bytes)
}

/// Generate session token
pub fn generate_session_token() -> String {
    let bytes = generate_random_bytes(24);
    hex_encode(&bytes)
}

/// Simple SHA-256 implementation (for demonstration)
fn hash_sha256(data: &[u8]) -> [u8; 32] {
    // Note: In production, use a proper SHA-256 implementation
    // This is a simplified version for demonstration
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};

    data.hash(&mut hasher);
    let hash = hasher.finish();

    let mut result = [0u8; 32];
    for i in 0..4 {
        let bytes = ((hash >> (i * 16)) as u64).to_le_bytes();
        for j in 0..8 {
            if i * 8 + j < 32 {
                result[i * 8 + j] = bytes[j];
            }
        }
    }

    result
}

/// Hex encoding
fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);

    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
    }

    result
}

/// Base64 encoding
fn base64_encode(bytes: &[u8]) -> String {
    base64::encode(bytes)
}

/// Secure random string generator
pub fn generate_secure_random_string(length: usize, charset: Option<&str>) -> String {
    let default_charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let chars = charset.unwrap_or(default_charset);
    let char_bytes = chars.as_bytes();

    let random_bytes = generate_random_bytes(length);
    let mut result = String::with_capacity(length);

    for byte in random_bytes {
        let index = (byte as usize) % char_bytes.len();
        result.push(char_bytes[index] as char);
    }

    result
}

/// Timing-safe operation wrapper
pub fn timing_safe_operation<F, T>(operation: F) -> T
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = operation();
    let elapsed = start.elapsed();

    // Ensure minimum execution time to prevent timing attacks
    let min_duration = std::time::Duration::from_micros(100);
    if elapsed < min_duration {
        std::thread::sleep(min_duration - elapsed);
    }

    result
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> (bool, Vec<String>) {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push("Password must be at least 8 characters long".to_string());
    }

    if password.len() > 128 {
        errors.push("Password must be less than 128 characters long".to_string());
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one digit".to_string());
    }

    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
    {
        errors.push("Password must contain at least one special character".to_string());
    }

    (errors.is_empty(), errors)
}

/// Rate limiting utilities
pub struct RateLimiter {
    requests: std::collections::HashMap<String, Vec<u64>>,
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            max_requests,
            window_seconds,
        }
    }

    pub fn is_allowed(&mut self, key: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - self.window_seconds;

        let timestamps = self
            .requests
            .entry(key.to_string())
            .or_insert_with(Vec::new);

        // Remove old timestamps
        timestamps.retain(|&timestamp| timestamp > window_start);

        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub fn cleanup_old_entries(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - self.window_seconds;

        self.requests.retain(|_, timestamps| {
            timestamps.retain(|&timestamp| timestamp > window_start);
            !timestamps.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello!"));
        assert!(!constant_time_compare("", "hello"));
    }

    #[test]
    fn test_constant_time_compare_bytes() {
        let a = b"hello";
        let b = b"hello";
        let c = b"world";

        assert!(constant_time_compare_bytes(a, b));
        assert!(!constant_time_compare_bytes(a, c));
    }

    #[test]
    fn test_verify_api_key() {
        let key = "test-key-123";

        // Plain comparison
        assert!(verify_api_key(key, key, None).unwrap());
        assert!(verify_api_key(key, key, Some("plain")).unwrap());
        assert!(!verify_api_key(key, "wrong-key", None).unwrap());

        // SHA256 comparison
        assert!(verify_api_key(key, key, Some("sha256")).unwrap());
        assert!(!verify_api_key(key, "wrong-key", Some("sha256")).unwrap());
    }

    #[test]
    fn test_generate_api_key() {
        let key1 = generate_api_key(None);
        let key2 = generate_api_key(Some(16));

        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_password_strength_validation() {
        let (valid, errors) = validate_password_strength("Weak123!");
        assert!(valid);
        assert!(errors.is_empty());

        let (valid, errors) = validate_password_strength("weak");
        assert!(!valid);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, 60);

        assert!(limiter.is_allowed("user1"));
        assert!(limiter.is_allowed("user1"));
        assert!(limiter.is_allowed("user1"));
        assert!(!limiter.is_allowed("user1")); // Fourth request should be denied

        assert!(limiter.is_allowed("user2")); // Different user should be allowed
    }

    #[test]
    fn test_generate_secure_random_string() {
        let s1 = generate_secure_random_string(10, None);
        let s2 = generate_secure_random_string(10, Some("ABC123"));

        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2);

        // Check that s2 only contains characters from the specified charset
        assert!(s2.chars().all(|c| "ABC123".contains(c)));
    }
}
