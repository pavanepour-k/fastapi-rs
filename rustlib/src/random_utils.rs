use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::wrap_pyfunction;
use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, Standard};
use uuid::Uuid;

/// Generate a random u32 in the range [min, max].
#[pyfunction]
pub fn random_u32(min: u32, max: u32) -> PyResult<u32> {
    if min > max {
        return Err(PyValueError::new_err("min must be <= max"));
    }
    Ok(thread_rng().gen_range(min..=max))
}

/// Generate a random f64 in the range [0.0, 1.0).
#[pyfunction]
pub fn random_f64() -> f64 {
    thread_rng().gen()
}

/// Generate a random boolean.
#[pyfunction]
pub fn random_bool() -> bool {
    thread_rng().gen()
}

/// Generate a random string of given length containing alphanumeric characters.
#[pyfunction]
pub fn random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate random bytes of given length.
#[pyfunction]
pub fn random_bytes(length: usize) -> Vec<u8> {
    thread_rng()
        .sample_iter(Standard)
        .take(length)
        .collect()
}

/// Generate a new UUID v4.
#[pyfunction]
pub fn uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

/// Python module initializer for random utilities.
#[pymodule]
pub fn random_utils(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(random_u32, m)?)?;
    m.add_function(wrap_pyfunction!(random_f64, m)?)?;
    m.add_function(wrap_pyfunction!(random_bool, m)?)?;
    m.add_function(wrap_pyfunction!(random_string, m)?)?;
    m.add_function(wrap_pyfunction!(random_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(uuid_v4, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_random_u32_range() {
        let v = random_u32(1, 10).unwrap();
        assert!(v >= 1 && v <= 10);
    }

    #[test]
    #[should_panic]
    fn test_random_u32_invalid() {
        random_u32(10, 1).unwrap();
    }

    #[test]
    fn test_random_string_length() {
        let s = random_string(16);
        assert_eq!(s.len(), 16);
    }

    #[test]
    fn test_random_bytes_length() {
        let b = random_bytes(8);
        assert_eq!(b.len(), 8);
    }

    #[test]
    fn test_uuid_v4_format() {
        let id = uuid_v4();
        assert!(Uuid::parse_str(&id).is_ok());
    }
}
