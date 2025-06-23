use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use sha2::{Sha256, Digest as Sha2Digest};
use sha1::Sha1;
use md5::Md5;
use hmac::{Hmac, Mac, NewMac};

/// Compute SHA-256 hash of input data, returning hex string.
#[pyfunction]
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute SHA-1 hash of input data, returning hex string.
#[pyfunction]
pub fn hash_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute MD5 hash of input data, returning hex string.
#[pyfunction]
pub fn hash_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute HMAC-SHA256 of input data using the provided key, returning hex string.
#[pyfunction]
pub fn hmac_sha256(data: &[u8], key: &[u8]) -> PyResult<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    mac.update(data);
    let result = mac.finalize().into_bytes();
    Ok(format!("{:x}", result))
}

#[pymodule]
pub fn hashing(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hash_sha256, m)?)?;
    m.add_function(wrap_pyfunction!(hash_sha1, m)?)?;
    m.add_function(wrap_pyfunction!(hash_md5, m)?)?;
    m.add_function(wrap_pyfunction!(hmac_sha256, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let data = b"hello";
        let h = hash_sha256(data);
        assert_eq!(h, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_hash_sha1() {
        let data = b"hello";
        let h = hash_sha1(data);
        assert_eq!(h, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_hash_md5() {
        let data = b"hello";
        let h = hash_md5(data);
        assert_eq!(h, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_hmac_sha256() {
        let data = b"data";
        let key = b"key";
        let h = hmac_sha256(data, key).unwrap();
        assert_eq!(h, "f7bc83f430538424b13298e6aa6fb143ef4d59a1494618cbfbbd6f0a8b7f3f43");
    }
}