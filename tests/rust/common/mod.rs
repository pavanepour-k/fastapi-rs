#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use rust_src::{core, params, types, security};

#[cfg(test)]
pub fn setup() {
    std::env::set_var("RUST_LOG", "debug");
}

#[cfg(test)]
pub fn mock_validation_schema() -> HashMap<String, Value> {
    let mut schema = HashMap::new();
    schema.insert("id".to_string(), serde_json::json!({
        "type": "integer",
        "required": true,
        "minimum": 1
    }));
    schema
}

#[cfg(test)]
pub fn mock_request_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("authorization".to_string(), "Bearer token123".to_string());
    headers
}

#[cfg(test)]
pub fn mock_oauth2_client() -> security::oauth2::OAuth2Client {
    security::oauth2::OAuth2Client::new("test_client".to_string(), Some("secret".to_string()))
        .with_redirect_uris(vec!["http://localhost:8000/callback".to_string()])
        .with_scopes(vec!["read".to_string(), "write".to_string()])
}

#[cfg(test)]
pub fn assert_validation_success(result: &params::validation::ValidationResult) {
    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[cfg(test)]
pub fn assert_validation_failure(result: &params::validation::ValidationResult) {
    assert!(!result.valid);
    assert!(!result.errors.is_empty());
}
