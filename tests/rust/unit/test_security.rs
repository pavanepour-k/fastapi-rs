#[path = "../common/mod.rs"]
mod common;

use fastapi_rs::security::*;
use common::*;

#[test]
fn test_constant_time_compare() {
    setup();
    assert!(utils::constant_time_compare("hello", "hello"));
    assert!(!utils::constant_time_compare("hello", "world"));
    assert!(!utils::constant_time_compare("hello", "hello!"));
}

#[test]
fn test_api_key_verification() {
    let key = "test-key-123";
    assert!(utils::verify_api_key(key, key, None).unwrap());
    assert!(!utils::verify_api_key(key, "wrong-key", None).unwrap());
}

#[test]
fn test_password_hashing() {
    let password = "test-password";
    let hash = utils::hash_password(password, None).unwrap();
    assert!(!hash.is_empty());
    assert_ne!(hash, password);
}

#[test]
fn test_oauth2_client() {
    let client = mock_oauth2_client();
    assert_eq!(client.client_id, "test_client");
    assert!(client.verify_secret("secret"));
    assert!(!client.verify_secret("wrong"));
    assert!(client.is_redirect_uri_valid("http://localhost:8000/callback"));
}

#[test]
fn test_oauth2_server() {
    let mut server = oauth2::OAuth2Server::new();
    let client = mock_oauth2_client();
    server.register_client(client);
    
    let code = server.create_authorization_code(
        "test_client",
        "http://localhost:8000/callback",
        vec!["read".to_string()],
        None,
        None,
        None
    ).unwrap();
    
    assert!(!code.is_empty());
}