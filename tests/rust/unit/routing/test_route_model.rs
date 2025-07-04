//! Unit tests for route model structures.

use ahash::AHashMap;
use rust_src::routing::route_model::{APIRoute, CompiledRoute, RouteMatch};
use regex::Regex;
use std::sync::Arc;

#[test]
fn test_api_route_new() {
    let route = APIRoute::new(
        "/api/v1/users".to_string(),
        vec!["GET".to_string(), "POST".to_string()],
    );

    assert_eq!(route.path, "/api/v1/users");
    assert_eq!(route.methods, vec!["GET", "POST"]);
    assert!(route.name.is_none());
    assert!(route.path_regex.is_none());
    assert!(route.path_format.is_none());
    assert!(route.param_names.is_empty());
    assert!(route.include_in_schema);
    assert!(route.tags.is_empty());
}

#[test]
fn test_api_route_builder_pattern() {
    let route = APIRoute::new("/users/{id}".to_string(), vec!["GET".to_string()])
        .with_name("get_user".to_string())
        .with_tags(vec!["users".to_string(), "api".to_string()])
        .include_in_schema(false);

    assert_eq!(route.name, Some("get_user".to_string()));
    assert_eq!(route.tags, vec!["users", "api"]);
    assert!(!route.include_in_schema);
}

#[test]
fn test_route_supports_method() {
    let route = APIRoute::new(
        "/test".to_string(),
        vec!["GET".to_string(), "PUT".to_string()],
    );

    // Case insensitive method checking
    assert!(route.supports_method("GET"));
    assert!(route.supports_method("get"));
    assert!(route.supports_method("Put"));
    assert!(route.supports_method("PUT"));

    // Methods not supported
    assert!(!route.supports_method("POST"));
    assert!(!route.supports_method("DELETE"));
    assert!(!route.supports_method("PATCH"));
}

#[test]
fn test_route_get_name_auto_generation() {
    let test_cases = vec![
        ("/users", "users"),
        ("/api/v1/users", "api_v1_users"),
        ("/users/{id}", "users_id"),
        (
            "/users/{user_id}/posts/{post_id}",
            "users_user_id_posts_post_id",
        ),
        ("/{id}", "id"),
    ];

    for (path, expected_name) in test_cases {
        let route = APIRoute::new(path.to_string(), vec!["GET".to_string()]);
        assert_eq!(route.get_name(), expected_name);
    }
}

#[test]
fn test_route_get_name_with_explicit_name() {
    let route = APIRoute::new(
        "/some/complex/{path}/route".to_string(),
        vec!["GET".to_string()],
    )
    .with_name("my_custom_name".to_string());

    assert_eq!(route.get_name(), "my_custom_name");
}

#[test]
fn test_compiled_route_creation() {
    let mut route = APIRoute::new(
        "/users/{id}/posts/{post_id}".to_string(),
        vec!["GET".to_string()],
    );
    route.param_names = vec!["id".to_string(), "post_id".to_string()];

    let regex = Arc::new(Regex::new(r"^/users/([^/]+)/posts/([^/]+)$").unwrap());

    let compiled = CompiledRoute::new(route.clone(), regex);

    assert_eq!(compiled.route.path, "/users/{id}/posts/{post_id}");
    assert_eq!(compiled.param_indices.len(), 2);
    assert_eq!(compiled.param_indices[0], ("id".to_string(), 1));
    assert_eq!(compiled.param_indices[1], ("post_id".to_string(), 2));
}

#[test]
fn test_compiled_route_match_path() {
    let mut route = APIRoute::new("/users/{id}".to_string(), vec!["GET".to_string()]);
    route.param_names = vec!["id".to_string()];

    let regex = Arc::new(Regex::new(r"^/users/([^/]+)$").unwrap());
    let compiled = CompiledRoute::new(route, regex);

    // Successful match
    let params = compiled.match_path("/users/123").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params.get("id"), Some(&"123".to_string()));

    // Failed match
    assert!(compiled.match_path("/users/").is_none());
    assert!(compiled.match_path("/posts/123").is_none());
    assert!(compiled.match_path("/users/123/posts").is_none());
}

#[test]
fn test_compiled_route_multiple_params() {
    let mut route = APIRoute::new(
        "/api/{version}/users/{id}".to_string(),
        vec!["GET".to_string()],
    );
    route.param_names = vec!["version".to_string(), "id".to_string()];

    let regex = Arc::new(Regex::new(r"^/api/([^/]+)/users/([^/]+)$").unwrap());
    let compiled = CompiledRoute::new(route, regex);

    let params = compiled.match_path("/api/v1/users/42").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.get("version"), Some(&"v1".to_string()));
    assert_eq!(params.get("id"), Some(&"42".to_string()));
}

#[test]
fn test_route_match_creation() {
    let mut params = AHashMap::new();
    params.insert("id".to_string(), "123".to_string());
    params.insert("name".to_string(), "test".to_string());

    let route_match = RouteMatch::new(5, params.clone());

    assert_eq!(route_match.route_index, 5);
    assert_eq!(route_match.params, params);
}

#[test]
fn test_route_with_empty_methods() {
    let route = APIRoute::new("/test".to_string(), vec![]);

    assert!(route.methods.is_empty());
    assert!(!route.supports_method("GET"));
    assert!(!route.supports_method("POST"));
}

#[test]
fn test_route_serialization_fields() {
    let mut route = APIRoute::new("/test".to_string(), vec!["GET".to_string()]);

    // Set optional fields
    route.path_regex = Some(r"^/test$".to_string());
    route.path_format = Some("/test".to_string());
    route.param_names = vec!["param1".to_string()];

    // Verify all fields are accessible
    assert_eq!(route.path_regex, Some(r"^/test$".to_string()));
    assert_eq!(route.path_format, Some("/test".to_string()));
    assert_eq!(route.param_names, vec!["param1"]);
}
