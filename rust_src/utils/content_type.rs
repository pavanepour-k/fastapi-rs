use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContentTypeError {
    #[error("Invalid content type format: {0}")]
    InvalidFormat(String),
    #[error("Missing media type")]
    MissingMediaType,
}

pub type Result<T> = std::result::Result<T, ContentTypeError>;

/// Parse Content-Type header value
pub fn parse_content_type(content_type: &str) -> Result<(String, HashMap<String, String>)> {
    if content_type.trim().is_empty() {
        return Err(ContentTypeError::MissingMediaType);
    }

    let mut parts = content_type.split(';');

    let media_type = parts
        .next()
        .ok_or_else(|| ContentTypeError::MissingMediaType)?
        .trim()
        .to_lowercase();

    if media_type.is_empty() {
        return Err(ContentTypeError::MissingMediaType);
    }

    let mut parameters = HashMap::new();

    for part in parts {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim().trim_matches('"').to_string();
            parameters.insert(key, value);
        }
    }

    Ok((media_type, parameters))
}

/// Get charset from content type parameters
pub fn get_charset(parameters: &HashMap<String, String>) -> Option<&String> {
    parameters.get("charset")
}

/// Get boundary from content type parameters (for multipart)
pub fn get_boundary(parameters: &HashMap<String, String>) -> Option<&String> {
    parameters.get("boundary")
}

/// Check if content type is JSON
pub fn is_json_content_type(media_type: &str) -> bool {
    matches!(
        media_type,
        "application/json" | "application/json-patch+json" | "application/merge-patch+json"
    )
}

/// Check if content type is form data
pub fn is_form_content_type(media_type: &str) -> bool {
    media_type == "application/x-www-form-urlencoded"
}

/// Check if content type is multipart
pub fn is_multipart_content_type(media_type: &str) -> bool {
    media_type.starts_with("multipart/")
}

/// Check if content type is text
pub fn is_text_content_type(media_type: &str) -> bool {
    media_type.starts_with("text/") || is_json_content_type(media_type)
}

/// Normalize media type
pub fn normalize_media_type(media_type: &str) -> String {
    media_type.trim().to_lowercase()
}

/// Build content type string from media type and parameters
pub fn build_content_type(media_type: &str, parameters: &HashMap<String, String>) -> String {
    if parameters.is_empty() {
        media_type.to_string()
    } else {
        let params: Vec<String> = parameters
            .iter()
            .map(|(k, v)| {
                if v.contains(' ') || v.contains(';') || v.contains(',') {
                    format!("{}=\"{}\"", k, v)
                } else {
                    format!("{}={}", k, v)
                }
            })
            .collect();
        format!("{}; {}", media_type, params.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_content_type() {
        let (media_type, params) = parse_content_type("application/json").unwrap();
        assert_eq!(media_type, "application/json");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_content_type_with_charset() {
        let (media_type, params) = parse_content_type("text/html; charset=utf-8").unwrap();
        assert_eq!(media_type, "text/html");
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
    }

    #[test]
    fn test_parse_content_type_with_boundary() {
        let (media_type, params) =
            parse_content_type("multipart/form-data; boundary=something").unwrap();
        assert_eq!(media_type, "multipart/form-data");
        assert_eq!(params.get("boundary"), Some(&"something".to_string()));
    }

    #[test]
    fn test_parse_content_type_with_quoted_params() {
        let (media_type, params) = parse_content_type("text/plain; charset=\"utf-8\"").unwrap();
        assert_eq!(media_type, "text/plain");
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
    }

    #[test]
    fn test_parse_content_type_case_insensitive() {
        let (media_type, params) = parse_content_type("TEXT/HTML; CHARSET=UTF-8").unwrap();
        assert_eq!(media_type, "text/html");
        assert_eq!(params.get("charset"), Some(&"UTF-8".to_string()));
    }

    #[test]
    fn test_parse_empty_content_type() {
        assert!(parse_content_type("").is_err());
        assert!(parse_content_type("   ").is_err());
    }

    #[test]
    fn test_content_type_predicates() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/json-patch+json"));
        assert!(!is_json_content_type("text/plain"));

        assert!(is_form_content_type("application/x-www-form-urlencoded"));
        assert!(!is_form_content_type("application/json"));

        assert!(is_multipart_content_type("multipart/form-data"));
        assert!(is_multipart_content_type("multipart/mixed"));
        assert!(!is_multipart_content_type("application/json"));

        assert!(is_text_content_type("text/plain"));
        assert!(is_text_content_type("text/html"));
        assert!(is_text_content_type("application/json"));
        assert!(!is_text_content_type("image/png"));
    }

    #[test]
    fn test_build_content_type() {
        let mut params = HashMap::new();
        params.insert("charset".to_string(), "utf-8".to_string());

        let content_type = build_content_type("text/html", &params);
        assert_eq!(content_type, "text/html; charset=utf-8");

        let content_type_no_params = build_content_type("application/json", &HashMap::new());
        assert_eq!(content_type_no_params, "application/json");

        let mut params_with_quotes = HashMap::new();
        params_with_quotes.insert("boundary".to_string(), "form boundary".to_string());
        let content_type_quoted = build_content_type("multipart/form-data", &params_with_quotes);
        assert_eq!(
            content_type_quoted,
            "multipart/form-data; boundary=\"form boundary\""
        );
    }
}
