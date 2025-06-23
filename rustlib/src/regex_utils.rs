use regex::Regex;
use lazy_static::lazy_static;
use pyo3::prelude::*;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"(?i)^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^\+?[0-9\-\s]{7,20}$").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"(?i)^(https?://)?([A-Z0-9.-]+\.[A-Z]{2,})(/.*)?$").unwrap();
}

/// Validate an email address against a standard regex pattern.
#[pyfunction]
pub fn validate_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Validate a phone number (digits, spaces, hyphens, optional leading '+').
#[pyfunction]
pub fn validate_phone(number: &str) -> bool {
    PHONE_REGEX.is_match(number)
}

/// Validate a URL (http/https, domain, optional path).
#[pyfunction]
pub fn validate_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}

/// Find all non-overlapping matches of a regex pattern in the given text.
#[pyfunction]
pub fn find_pattern_matches(pattern: &str, text: &str) -> Vec<String> {
    let re = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_) => return Vec::new(),
    };
    re.find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
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
}
