#[path = "../common/mod.rs"]
mod common;

use fastapi_rs::params::validation::*;
use common::*;
use std::collections::HashMap;
use serde_json::json;

#[test]
fn test_path_param_validation_success() {
    setup();
    let mut params = HashMap::new();
    params.insert("id".to_string(), "123".to_string());
    
    let result = validate_path_params(params, mock_validation_schema()).unwrap();
    assert_validation_success(&result);
    assert_eq!(result.validated_data.get("id"), Some(&json!(123)));
}

#[test]
fn test_path_param_validation_failure() {
    let mut params = HashMap::new();
    params.insert("id".to_string(), "abc".to_string());
    
    let result = validate_path_params(params, mock_validation_schema()).unwrap();
    assert_validation_failure(&result);
}

#[test]
fn test_parameter_schema_creation() {
    let schema = ParameterSchema::new("test".to_string(), "string".to_string())
        .required()
        .with_length_range(Some(1), Some(50));
    
    assert_eq!(schema.name, "test");
    assert_eq!(schema.param_type, "string");
    assert!(schema.required);
    assert_eq!(schema.min_length, Some(1));
    assert_eq!(schema.max_length, Some(50));
}

#[test]
fn test_email_validation() {
    let schema = ParameterSchema::new("email".to_string(), "email".to_string());
    let result = validate_single_parameter("test@example.com", &schema);
    assert!(result.is_ok());
    
    let result = validate_single_parameter("invalid-email", &schema);
    assert!(result.is_err());
}

#[test]
fn test_integer_validation() {
    let schema = ParameterSchema::new("age".to_string(), "integer".to_string())
        .with_range(Some(0.0), Some(120.0));
    
    let result = validate_single_parameter("25", &schema);
    assert!(result.is_ok());
    
    let result = validate_single_parameter("150", &schema);
    assert!(result.is_err());
}