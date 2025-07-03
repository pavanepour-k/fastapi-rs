#[path = "../common/mod.rs"]
mod common;

use fastapi_rs::{core, params, serialization, security};
use common::*;
use std::collections::HashMap;
use serde_json::json;

#[test]
fn test_full_request_pipeline() {
    setup();
    
    let mut tree = core::routing::RouteTree::new();
    let route = core::routing::create_api_route("/users/{id:int}", vec!["GET"], None).unwrap();
    tree.add_route(route).unwrap();
    
    let (index, params) = tree.match_route("/users/123", "GET").unwrap();
    assert_eq!(index, 0);
    assert_eq!(params.get("id").unwrap(), "123");
    
    let schema = mock_validation_schema();
    let result = params::validation::validate_path_params(params.into_iter().collect(), schema).unwrap();
    assert_validation_success(&result);
}

#[test]
fn test_json_serialization_pipeline() {
    let json_data = json!({"name": "John", "age": 30});
    let json_str = serialization::decoders::fast_parse_json_string(&json_data.to_string()).unwrap();
    assert_eq!(json_str["name"], "John");
}

#[test]
fn test_oauth2_full_flow() {
    let mut server = security::oauth2::OAuth2Server::new();
    let client = mock_oauth2_client();
    server.register_client(client);
    
    let code = server.create_authorization_code(
        "test_client",
        "http://localhost:8000/callback",
        vec!["read".to_string()],
        None,
        None,
        Some("user123".to_string())
    ).unwrap();
    
    let token_response = server.exchange_authorization_code(
        &code,
        "test_client",
        Some("secret"),
        "http://localhost:8000/callback",
        None
    ).unwrap();
    
    assert!(!token_response.access_token.is_empty());
    assert_eq!(token_response.token_type, "Bearer");
    
    let access_token = server.validate_access_token(&token_response.access_token).unwrap();
    assert_eq!(access_token.client_id, "test_client");
    assert_eq!(access_token.user_id, Some("user123".to_string()));
}