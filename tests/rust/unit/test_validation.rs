#[path = "../common/mod.rs"]
mod common;

use common::*;
use rust_src::params::validation::*;
use serde_json::json;
use std::collections::HashMap;

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
fn test_string_validation() {
    let schema = ParameterSchema::new("test".to_string(), "string".to_string())
        .with_length_range(Some(3), Some(10));

    assert!(validate_single_parameter("hello", &schema).is_ok());
    assert!(validate_single_parameter("hi", &schema).is_err());
    assert!(validate_single_parameter("hello world!", &schema).is_err());
}

#[test]
fn test_integer_validation() {
    let schema = ParameterSchema::new("age".to_string(), "integer".to_string())
        .with_range(Some(0.0), Some(120.0));

    let result = validate_single_parameter("25", &schema);
    assert!(result.is_ok());

    let result = validate_single_parameter("150", &schema);
    assert!(result.is_err());

    let result = validate_single_parameter("-5", &schema);
    assert!(result.is_err());

    let result = validate_single_parameter("abc", &schema);
    assert!(result.is_err());
}

#[test]
fn test_integer_validation_with_bounds() {
    let schema = ParameterSchema::new("age".to_string(), "integer".to_string()).with_bounds(
        Some(0.0),
        Some(0.0),
        None,
        Some(150.0),
    ); // gt: 0, le: 150

    assert!(validate_single_parameter("25", &schema).is_ok());
    assert!(validate_single_parameter("150", &schema).is_ok());
    assert!(validate_single_parameter("0", &schema).is_err()); // Not greater than 0
    assert!(validate_single_parameter("151", &schema).is_err()); // Greater than 150
}

#[test]
fn test_float_validation() {
    let schema = ParameterSchema::new("price".to_string(), "float".to_string())
        .with_range(Some(0.0), Some(1000.0));

    assert!(validate_single_parameter("99.99", &schema).is_ok());
    assert!(validate_single_parameter("0", &schema).is_ok());
    assert!(validate_single_parameter("-10.5", &schema).is_err());
    assert!(validate_single_parameter("not_a_number", &schema).is_err());
}

#[test]
fn test_float_multiple_of() {
    let mut schema = ParameterSchema::new("price".to_string(), "float".to_string());
    schema.multiple_of = Some(0.25);
    schema.minimum = Some(0.0);

    assert!(validate_single_parameter("1.25", &schema).is_ok());
    assert!(validate_single_parameter("2.50", &schema).is_ok());
    assert!(validate_single_parameter("1.33", &schema).is_err()); // Not multiple of 0.25
}

#[test]
fn test_boolean_validation() {
    let schema = ParameterSchema::new("active".to_string(), "boolean".to_string());

    // Valid true values
    assert_eq!(
        validate_single_parameter("true", &schema).unwrap(),
        json!(true)
    );
    assert_eq!(
        validate_single_parameter("1", &schema).unwrap(),
        json!(true)
    );
    assert_eq!(
        validate_single_parameter("yes", &schema).unwrap(),
        json!(true)
    );
    assert_eq!(
        validate_single_parameter("on", &schema).unwrap(),
        json!(true)
    );

    // Valid false values
    assert_eq!(
        validate_single_parameter("false", &schema).unwrap(),
        json!(false)
    );
    assert_eq!(
        validate_single_parameter("0", &schema).unwrap(),
        json!(false)
    );
    assert_eq!(
        validate_single_parameter("no", &schema).unwrap(),
        json!(false)
    );
    assert_eq!(
        validate_single_parameter("off", &schema).unwrap(),
        json!(false)
    );

    // Invalid values
    assert!(validate_single_parameter("maybe", &schema).is_err());
}

#[test]
fn test_email_validation() {
    let schema = ParameterSchema::new("email".to_string(), "email".to_string());

    // Valid emails
    assert!(validate_single_parameter("test@example.com", &schema).is_ok());
    assert!(validate_single_parameter("user.name+tag@domain.co.uk", &schema).is_ok());
    assert!(validate_single_parameter("test_email@sub.domain.com", &schema).is_ok());

    // Invalid emails
    assert!(validate_single_parameter("invalid.email", &schema).is_err());
    assert!(validate_single_parameter("@example.com", &schema).is_err());
    assert!(validate_single_parameter("user@", &schema).is_err());
    assert!(validate_single_parameter("user@@example.com", &schema).is_err());
}

#[test]
fn test_uuid_validation() {
    let schema = ParameterSchema::new("id".to_string(), "uuid".to_string());

    // Valid UUIDs
    assert!(validate_single_parameter("550e8400-e29b-41d4-a716-446655440000", &schema).is_ok());
    assert!(validate_single_parameter("6ba7b810-9dad-11d1-80b4-00c04fd430c8", &schema).is_ok());

    // Case insensitive
    let result =
        validate_single_parameter("550E8400-E29B-41D4-A716-446655440000", &schema).unwrap();
    assert_eq!(result, json!("550e8400-e29b-41d4-a716-446655440000")); // Normalized to lowercase

    // Invalid UUIDs
    assert!(validate_single_parameter("550e8400-e29b-41d4-a716", &schema).is_err());
    assert!(validate_single_parameter("not-a-uuid", &schema).is_err());
    assert!(validate_single_parameter("550e8400-e29b-41d4-a716-44665544000g", &schema).is_err());
    // 'g' is invalid
}

#[test]
fn test_url_validation() {
    let schema = ParameterSchema::new("website".to_string(), "url".to_string());

    // Valid URLs
    assert!(validate_single_parameter("https://example.com", &schema).is_ok());
    assert!(
        validate_single_parameter("http://subdomain.example.com/path?query=1", &schema).is_ok()
    );
    assert!(validate_single_parameter("https://example.com:8080/path#anchor", &schema).is_ok());

    // Invalid URLs
    assert!(validate_single_parameter("not a url", &schema).is_err());
    assert!(validate_single_parameter("ftp://example.com", &schema).is_err()); // Only http/https
    assert!(validate_single_parameter("https://", &schema).is_err());
}

#[test]
fn test_ipv4_validation() {
    let schema = ParameterSchema::new("ip".to_string(), "ipv4".to_string());

    // Valid IPv4
    assert!(validate_single_parameter("192.168.1.1", &schema).is_ok());
    assert!(validate_single_parameter("0.0.0.0", &schema).is_ok());
    assert!(validate_single_parameter("255.255.255.255", &schema).is_ok());

    // Invalid IPv4
    assert!(validate_single_parameter("256.1.1.1", &schema).is_err());
    assert!(validate_single_parameter("192.168.1", &schema).is_err());
    assert!(validate_single_parameter("192.168.1.1.1", &schema).is_err());
}

#[test]
fn test_ipv6_validation() {
    let schema = ParameterSchema::new("ip".to_string(), "ipv6".to_string());

    // Valid IPv6
    assert!(validate_single_parameter("2001:0db8:85a3:0000:0000:8a2e:0370:7334", &schema).is_ok());
    assert!(validate_single_parameter("2001:db8:85a3::8a2e:370:7334", &schema).is_ok());
    assert!(validate_single_parameter("::1", &schema).is_ok());

    // Invalid IPv6
    assert!(validate_single_parameter("not_an_ipv6", &schema).is_err());
    assert!(validate_single_parameter("192.168.1.1", &schema).is_err()); // IPv4, not IPv6
}

#[test]
fn test_datetime_validation() {
    let schema = ParameterSchema::new("timestamp".to_string(), "datetime".to_string());

    // Valid datetimes
    assert!(validate_single_parameter("2023-12-25T10:30:00Z", &schema).is_ok());
    assert!(validate_single_parameter("2023-12-25T10:30:00+05:30", &schema).is_ok());
    assert!(validate_single_parameter("2023-12-25T10:30:00.123456Z", &schema).is_ok());

    // Invalid datetimes
    assert!(validate_single_parameter("2023-12-25", &schema).is_err());
    assert!(validate_single_parameter("not a date", &schema).is_err());
    assert!(validate_single_parameter("2023-12-25 10:30:00", &schema).is_err());
    // Wrong format
}

#[test]
fn test_date_validation() {
    let schema = ParameterSchema::new("date".to_string(), "date".to_string());

    // Valid dates
    assert!(validate_single_parameter("2023-12-25", &schema).is_ok());
    assert!(validate_single_parameter("2000-01-01", &schema).is_ok());

    // Invalid dates
    assert!(validate_single_parameter("25-12-2023", &schema).is_err()); // Wrong format
    assert!(validate_single_parameter("2023/12/25", &schema).is_err()); // Wrong separator
    assert!(validate_single_parameter("not a date", &schema).is_err());
}

#[test]
fn test_time_validation() {
    let schema = ParameterSchema::new("time".to_string(), "time".to_string());

    // Valid times
    assert!(validate_single_parameter("10:30:00", &schema).is_ok());
    assert!(validate_single_parameter("23:59:59", &schema).is_ok());
    assert!(validate_single_parameter("00:00:00", &schema).is_ok());
    assert!(validate_single_parameter("14:30", &schema).is_ok()); // Without seconds

    // Invalid times
    assert!(validate_single_parameter("25:00:00", &schema).is_err()); // Invalid hour
    assert!(validate_single_parameter("10:60:00", &schema).is_err()); // Invalid minute
    assert!(validate_single_parameter("10:30:60", &schema).is_err()); // Invalid second
}

#[test]
fn test_enum_validation() {
    let mut schema = ParameterSchema::new("color".to_string(), "string".to_string());
    schema.enum_values = Some(vec![
        "red".to_string(),
        "green".to_string(),
        "blue".to_string(),
    ]);

    assert!(validate_single_parameter("red", &schema).is_ok());
    assert!(validate_single_parameter("green", &schema).is_ok());
    assert!(validate_single_parameter("yellow", &schema).is_err());
}

#[test]
fn test_case_insensitive_enum() {
    let mut schema = ParameterSchema::new("method".to_string(), "string".to_string());
    schema.enum_values = Some(vec!["GET".to_string(), "POST".to_string()]);
    schema.case_sensitive = false;

    assert!(validate_single_parameter("get", &schema).is_ok());
    assert!(validate_single_parameter("Post", &schema).is_ok());
    assert!(validate_single_parameter("DELETE", &schema).is_err());
}

#[test]
fn test_pattern_validation() {
    let mut schema = ParameterSchema::new("code".to_string(), "string".to_string());
    schema.pattern = Some(r"^[A-Z]{3}-\d{3}$".to_string());

    assert!(validate_single_parameter("ABC-123", &schema).is_ok());
    assert!(validate_single_parameter("XYZ-999", &schema).is_ok());
    assert!(validate_single_parameter("abc-123", &schema).is_err()); // Lowercase
    assert!(validate_single_parameter("ABC-1234", &schema).is_err()); // Too many digits
}

#[test]
fn test_whitespace_handling() {
    let schema = ParameterSchema::new("name".to_string(), "string".to_string())
        .with_length_range(Some(3), Some(10));

    // With strip_whitespace = true (default)
    let result = validate_single_parameter("  John  ", &schema).unwrap();
    assert_eq!(result, json!("John"));

    // Test that validation happens after stripping
    assert!(validate_single_parameter("  ab  ", &schema).is_err()); // Too short after strip
}

#[test]
fn test_no_whitespace_stripping() {
    let mut schema = ParameterSchema::new("name".to_string(), "string".to_string());
    schema.strip_whitespace = false;

    let result = validate_single_parameter("  John  ", &schema).unwrap();
    assert_eq!(result, json!("  John  "));
}

#[test]
fn test_numeric_bounds() {
    let schema = ParameterSchema::new("score".to_string(), "float".to_string()).with_bounds(
        Some(0.0),
        None,
        None,
        Some(100.0),
    );

    assert!(validate_single_parameter("50.5", &schema).is_ok());
    assert!(validate_single_parameter("0", &schema).is_err()); // gt 0
    assert!(validate_single_parameter("100.1", &schema).is_err()); // le 100
}

#[test]
fn test_validation_with_defaults() {
    let mut schemas = Vec::new();

    let mut schema1 = ParameterSchema::new("page".to_string(), "integer".to_string());
    schema1.default = Some(json!(1));
    schemas.push(schema1);

    let mut schema2 = ParameterSchema::new("limit".to_string(), "integer".to_string());
    schema2.default = Some(json!(10));
    schemas.push(schema2);

    // Empty params - should use defaults
    let params = HashMap::new();
    let result = validate_parameters(params, schemas).unwrap();

    assert!(result.valid);
    assert_eq!(result.validated_data.get("page"), Some(&json!(1)));
    assert_eq!(result.validated_data.get("limit"), Some(&json!(10)));
}

#[test]
fn test_required_parameter_missing() {
    let schema = ParameterSchema::new("name".to_string(), "string".to_string()).required();

    let schemas = vec![schema];
    let params = HashMap::new();

    let result = validate_parameters(params, schemas).unwrap();
    assert!(!result.valid);
    assert!(!result.errors.is_empty());

    match &result.errors[0] {
        ValidationError::MissingRequired(param) => assert_eq!(param, "name"),
        _ => panic!("Expected MissingRequired error"),
    }
}

#[test]
fn test_multiple_values() {
    let mut schema = ParameterSchema::new("tags".to_string(), "string".to_string());
    schema.allow_multiple = true;

    let values = vec!["rust".to_string(), "python".to_string(), "go".to_string()];
    let result = validate_parameter_values(&values, &schema).unwrap();

    if let serde_json::Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "rust");
        assert_eq!(arr[1], "python");
        assert_eq!(arr[2], "go");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_multiple_values_not_allowed() {
    let schema = ParameterSchema::new("name".to_string(), "string".to_string());

    let values = vec!["John".to_string(), "Jane".to_string()];
    let result = validate_parameter_values(&values, &schema);

    assert!(result.is_err());
    match result {
        Err(ValidationError::MultipleValuesNotAllowed(param)) => assert_eq!(param, "name"),
        _ => panic!("Expected MultipleValuesNotAllowed error"),
    }
}

#[test]
fn test_security_integer_overflow() {
    let schema = ParameterSchema::new("count".to_string(), "integer".to_string());

    // Test for potential overflow
    assert!(validate_single_parameter("9223372036854775807", &schema).is_ok()); // i64::MAX
    assert!(validate_single_parameter("9223372036854775808", &schema).is_err());
    // Overflow
}

#[test]
fn test_validation_error_messages() {
    let schema = ParameterSchema::new("age".to_string(), "integer".to_string())
        .with_range(Some(0.0), Some(120.0))
        .required();

    let result = validate_single_parameter("150", &schema);
    if let Err(ValidationError::OutOfRange { param, value }) = result {
        assert_eq!(param, "age");
        assert!(value.contains("150"));
    } else {
        panic!("Expected OutOfRange error");
    }
}

#[test]
fn test_body_validation_json() {
    let body = r#"{"name": "John", "age": 30, "active": true}"#.as_bytes().to_vec();
    let schema = HashMap::new();

    let result = validate_body_params(body, schema).unwrap();
    assert!(result.valid);

    let body_value = result.validated_data.get("body").unwrap();
    assert_eq!(body_value["name"], "John");
    assert_eq!(body_value["age"], 30);
    assert_eq!(body_value["active"], true);
}

#[test]
fn test_body_validation_empty() {
    let body = Vec::new();
    let schema = HashMap::new();

    let result = validate_body_params(body, schema).unwrap();
    assert!(result.valid);
    assert!(result.validated_data.is_empty());
}

#[test]
fn test_body_validation_invalid_utf8() {
    let body = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8
    let schema = HashMap::new();

    let result = validate_body_params(body, schema);
    assert!(result.is_err());

    match result {
        Err(ValidationError::InvalidFormat { param, .. }) => assert_eq!(param, "body"),
        _ => panic!("Expected InvalidFormat error"),
    }
}

#[test]
fn test_header_param_validation() {
    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "secret123".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let schema = HashMap::new();
    let result = validate_header_params(headers, schema).unwrap();

    assert!(result.valid);
    // Headers should be normalized to lowercase
    assert!(result.validated_data.contains_key("x-api-key"));
    assert!(result.validated_data.contains_key("content-type"));
}

#[test]
fn test_query_params_with_arrays() {
    let mut params = HashMap::new();
    params.insert("items[0]".to_string(), "first".to_string());
    params.insert("items[1]".to_string(), "second".to_string());
    params.insert("name".to_string(), "test".to_string());

    let schema = HashMap::new();
    let result = validate_query_params(params, schema).unwrap();

    assert!(result.valid);
    assert!(result.validated_data.contains_key("items"));
    assert!(result.validated_data.contains_key("name"));
}

#[test]
fn test_concurrent_validation() {
    use std::sync::Arc;
    use std::thread;

    let schema = Arc::new(ParameterSchema::new(
        "value".to_string(),
        "integer".to_string(),
    ));
    let mut handles = vec![];

    for i in 0..10 {
        let schema_clone = Arc::clone(&schema);
        let handle = thread::spawn(move || {
            let value = i.to_string();
            validate_single_parameter(&value, &schema_clone)
        });
        handles.push(handle);
    }

    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }
}
