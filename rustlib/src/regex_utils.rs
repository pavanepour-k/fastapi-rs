use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use regex::Regex;
use lazy_static::lazy_static;

/// Validate an email address against a standard regex pattern.
#[pyfunction]
pub fn validate_email(email: &str) -> bool {
    lazy_static! {
        static ref EMAIL_REGEX: Regex =
            Regex::new(r"(?i)^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$").unwrap();
    }
    EMAIL_REGEX.is_match(email)
}

/// Validate a phone number (digits, spaces, hyphens, optional leading '+').
#[pyfunction]
pub fn validate_phone(number: &str) -> bool {
    lazy_static! {
        static ref PHONE_REGEX: Regex =
            Regex::new(r"^\+?[0-9\-\s]{7,20}$").unwrap();
    }
    PHONE_REGEX.is_match(number)
}

/// Validate a URL (http/https, domain, optional path).
#[pyfunction]
pub fn validate_url(url: &str) -> bool {
    lazy_static! {
        static ref URL_REGEX: Regex =
            Regex::new(r"(?i)^(https?://)?([A-Z0-9.-]+\.[A-Z]{2,})(/.*)?$").unwrap();
    }
    URL_REGEX.is_match(url)
}

/// Find all non-overlapping matches of a regex pattern in the given text.
#[pyfunction]
pub fn find_pattern_matches(pattern: &str, text: &str) -> Vec<String> {
    match Regex::new(pattern) {
        Ok(re) => re.find_iter(text).map(|m| m.as_str().to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Find and replace all occurrences matching a pattern with the replacement.
#[pyfunction]
pub fn find_and_replace(pattern: &str, text: &str, replacement: &str) -> String {
    match Regex::new(pattern) {
        Ok(re) => re.replace_all(text, replacement).to_string(),
        Err(_) => text.to_string(),
    }
}

/// Python module initializer for format-validation and regex utilities.
#[pymodule]
pub fn regex_utils(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_email, m)?)?;
    m.add_function(wrap_pyfunction!(validate_phone, m)?)?;
    m.add_function(wrap_pyfunction!(validate_url, m)?)?;
    m.add_function(wrap_pyfunction!(find_pattern_matches, m)?)?;
    m.add_function(wrap_pyfunction!(find_and_replace, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("+1234567890"));
        assert!(validate_phone("123-456-7890"));
        assert!(!validate_phone("phone123"));
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("http://example.com"));
        assert!(validate_url("https://example.com/path?query=1"));
        assert!(!validate_url("not a url"));
    }

    #[test]
    fn test_find_pattern_matches() {
        let text = "foo 123 bar 456 baz";
        let matches = find_pattern_matches(r"\d+", text);
        assert_eq!(matches, vec!["123".to_string(), "456".to_string()]);
    }

    #[test]
    fn test_find_and_replace() {
        let text = "foo foo bar";
        let replaced = find_and_replace("foo", text, "baz");
        assert_eq!(replaced, "baz baz bar");
    }
}