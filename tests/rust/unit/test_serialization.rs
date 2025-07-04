#[path = "../common/mod.rs"]
mod common;

use rust_src::serialization::*;
use common::*;
use serde_json::json;

#[test]
fn test_json_parsing() {
    setup();
    let json_str = r#"{"name": "John", "age": 30}"#;
    let result = decoders::fast_parse_json_string(json_str).unwrap();
    assert_eq!(result["name"], "John");
    assert_eq!(result["age"], 30);
}

#[test]
fn test_multipart_field_parsing() {
    let headers = "Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain";
    let content = "Hello, World!";
    
    let field = decoders::parse_multipart_field(headers, content).unwrap();
    assert_eq!(field.name, Some("file".to_string()));
    assert_eq!(field.filename, Some("test.txt".to_string()));
    assert_eq!(field.content_type, Some("text/plain".to_string()));
}

#[test]
fn test_percent_decode() {
    assert_eq!(decoders::percent_decode("Hello%20World"), "Hello World");
    assert_eq!(decoders::percent_decode("test%21%40%23"), "test!@#");
}

#[test]
fn test_json_context_parsing() {
    let body = b"invalid json";
    let result = decoders::parse_json_with_context(body, "test");
    assert!(result.is_err());
}