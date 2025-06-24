use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use ammonia::clean;
use regex::Regex;

/// Sanitize HTML by removing harmful tags and attributes.
#[pyfunction]
pub fn sanitize_html(input: &str) -> String {
    clean(input)
}

/// Replace banned words with "[censored]" in a case-insensitive manner.
#[pyfunction]
pub fn filter_bad_words(text: &str, banned_list: Vec<String>) -> String {
    let mut result = text.to_string();
    for word in banned_list {
        let escaped = regex::escape(&word);
        let pattern = format!(r"(?i)\b{}\b", escaped);
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, "[censored]").to_string();
        }
    }
    result
}

/// Python module initializer for text sanitization and filtering.
#[pymodule]
pub fn filter(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sanitize_html, m)?)?;
    m.add_function(wrap_pyfunction!(filter_bad_words, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let html = "<script>alert('XSS')</script><div>Safe</div>";
        let sanitized = sanitize_html(html);
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("<div>Safe</div>"));
    }

    #[test]
    fn test_filter_bad_words() {
        let banned = vec!["bad".to_string(), "evil".to_string()];
        let text = "This is bad and EVIL world.";
        let filtered = filter_bad_words(text, banned);
        assert!(filtered.contains("[censored] and [censored] world."));
    }
}