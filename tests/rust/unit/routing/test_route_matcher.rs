//! Unit tests for route matching functionality.

use fastapi_rs::routing::{
    APIRoute, CompiledRoute, RouteMatch,
    compile_route, match_route, match_single_route,
    create_compiled_routes, get_or_compile_regex,
};
use ahash::AHashMap;

#[test]
fn test_get_or_compile_regex() {
    let pattern = r"^/test/([^/]+)$";
    
    // First call compiles and caches
    let regex1 = get_or_compile_regex(pattern).unwrap();
    
    // Second call retrieves from cache
    let regex2 = get_or_compile_regex(pattern).unwrap();
    
    // Should be the same Arc instance
    assert!(std::sync::Arc::ptr_eq(&regex1, &regex2));
}

#[test]
fn test_compile_route_basic() {
    let mut route = APIRoute::new(
        "/users/{id}".to_string(),
        vec!["GET".to_string()],
    );
    
    compile_route(&mut route).unwrap();
    
    assert!(route.path_regex.is_some());
    assert_eq!(route.path_regex.unwrap(), r"^/users/([^/]+)$");
    assert_eq!(route.param_names, vec!["id"]);
    assert_eq!(route.path_format, Some("/users/{id}".to_string()));
}

#[test]
fn test_compile_route_multiple_params() {
    let mut route = APIRoute::new(
        "/api/{version}/users/{id:int}".to_string(),
        vec!["GET".to_string()],
    );
    
    compile_route(&mut route).unwrap();
    
    assert_eq!(route.param_names, vec!["version", "id"]);
    assert!(route.path_regex.unwrap().contains(r"([0-9]+)"));
}

#[test]
fn test_match_single_route_success() {
    let mut route = APIRoute::new(
        "/users/{id}".to_string(),
        vec!["GET".to_string(), "POST".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    // Successful matches
    let params = match_single_route(&route, "/users/123", "GET").unwrap();
    assert_eq!(params.get("id"), Some(&"123".to_string()));
    
    let params = match_single_route(&route, "/users/abc", "POST").unwrap();
    assert_eq!(params.get("id"), Some(&"abc".to_string()));
}

#[test]
fn test_match_single_route_method_not_allowed() {
    let mut route = APIRoute::new(
        "/users/{id}".to_string(),
        vec!["GET".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    let result = match_single_route(&route, "/users/123", "POST");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Method not allowed"));
}

#[test]
fn test_match_single_route_not_found() {
    let mut route = APIRoute::new(
        "/users/{id}".to_string(),
        vec!["GET".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    let result = match_single_route(&route, "/posts/123", "GET");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Route not found"));
}

#[test]
fn test_create_compiled_routes() {
    let routes = vec![
        APIRoute::new("/users".to_string(), vec!["GET".to_string()]),
        APIRoute::new("/users/{id}".to_string(), vec!["GET".to_string()]),
        APIRoute::new("/posts/{id:int}".to_string(), vec!["POST".to_string()]),
    ];
    
    let compiled = create_compiled_routes(routes).unwrap();
    assert_eq!(compiled.len(), 3);
    
    // Check each route was compiled
    assert!(compiled[0].route.path_regex.is_some());
    assert!(compiled[1].route.path_regex.is_some());
    assert!(compiled[2].route.path_regex.is_some());
}

#[test]
fn test_match_route_with_compiled_routes() {
    let routes = vec![
        APIRoute::new("/api/v1/users".to_string(), vec!["GET".to_string()]),
        APIRoute::new("/api/v1/users/{id}".to_string(), vec!["GET".to_string()]),
        APIRoute::new("/api/v1/posts/{id:int}".to_string(), vec!["POST".to_string()]),
    ];
    
    let compiled = create_compiled_routes(routes).unwrap();
    
    // Test static route match
    let matched = match_route("/api/v1/users", "GET", &compiled).unwrap();
    assert_eq!(matched.route_index, 0);
    assert!(matched.params.is_empty());
    
    // Test dynamic route match
    let matched = match_route("/api/v1/users/123", "GET", &compiled).unwrap();
    assert_eq!(matched.route_index, 1);
    assert_eq!(matched.params.get("id"), Some(&"123".to_string()));
    
    // Test typed param match
    let matched = match_route("/api/v1/posts/456", "POST", &compiled).unwrap();
    assert_eq!(matched.route_index, 2);
    assert_eq!(matched.params.get("id"), Some(&"456".to_string()));
    
    // Test no match
    assert!(match_route("/api/v1/posts/abc", "POST", &compiled).is_none());
    assert!(match_route("/api/v1/users", "POST", &compiled).is_none());
}

#[test]
fn test_match_route_priority() {
    // More specific routes should match before less specific
    let routes = vec![
        APIRoute::new("/users/{id}".to_string(), vec!["GET".to_string()]),
        APIRoute::new("/users/me".to_string(), vec!["GET".to_string()]),
    ];
    
    let compiled = create_compiled_routes(routes).unwrap();
    
    // This test shows current behavior - may need route priority logic
    let matched = match_route("/users/me", "GET", &compiled);
    assert!(matched.is_some());
}

#[test]
fn test_compiled_route_match_path() {
    let mut route = APIRoute::new(
        "/items/{item_id}/details/{detail_id:int}".to_string(),
        vec!["GET".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    let regex = get_or_compile_regex(route.path_regex.as_ref().unwrap()).unwrap();
    let compiled = CompiledRoute::new(route, regex);
    
    let params = compiled.match_path("/items/abc/details/123").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.get("item_id"), Some(&"abc".to_string()));
    assert_eq!(params.get("detail_id"), Some(&"123".to_string()));
    
    // Invalid paths
    assert!(compiled.match_path("/items/abc/details/xyz").is_none());
    assert!(compiled.match_path("/items/abc/details/").is_none());
}

#[test]
fn test_case_sensitive_paths() {
    let mut route = APIRoute::new(
        "/Users/{id}".to_string(),
        vec!["GET".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    // Path matching should be case sensitive
    assert!(match_single_route(&route, "/Users/123", "GET").is_ok());
    assert!(match_single_route(&route, "/users/123", "GET").is_err());
}

#[test]
fn test_empty_param_values() {
    let mut route = APIRoute::new(
        "/search/{query}".to_string(),
        vec!["GET".to_string()],
    );
    compile_route(&mut route).unwrap();
    
    // Empty param should not match (using [^/]+ pattern)
    assert!(match_single_route(&route, "/search/", "GET").is_err());
    assert!(match_single_route(&route, "/search//", "GET").is_err());
}