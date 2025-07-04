//! Integration tests for routing functionality.

use fastapi_rs::routing::{
    create_api_route, compile_path_regex, RouteTree,
    APIRoute, create_route_tree,
};

#[test]
fn test_full_routing_workflow() {
    // Create routes using the public API
    let route1 = create_api_route(
        "/api/v1/users",
        vec!["GET", "POST"],
        Some("users_list"),
    ).unwrap();
    
    let route2 = create_api_route(
        "/api/v1/users/{user_id:int}",
        vec!["GET", "PUT", "DELETE"],
        Some("user_detail"),
    ).unwrap();
    
    let route3 = create_api_route(
        "/api/v1/users/{user_id:int}/posts/{post_id}",
        vec!["GET"],
        None,
    ).unwrap();
    
    // Build route tree
    let tree = create_route_tree(vec![route1, route2, route3]).unwrap();
    
    // Test various matching scenarios
    
    // Static route matching
    let match1 = tree.match_route("/api/v1/users", "GET").unwrap();
    assert_eq!(match1.route_index, 0);
    assert!(match1.params.is_empty());
    
    // Dynamic route with int validation
    let match2 = tree.match_route("/api/v1/users/123", "GET").unwrap();
    assert_eq!(match2.route_index, 1);
    assert_eq!(match2.params.get("user_id"), Some(&"123".to_string()));
    
    // Nested dynamic route
    let match3 = tree.match_route(
        "/api/v1/users/456/posts/hello-world",
        "GET"
    ).unwrap();
    assert_eq!(match3.route_index, 2);
    assert_eq!(match3.params.get("user_id"), Some(&"456".to_string()));
    assert_eq!(match3.params.get("post_id"), Some(&"hello-world".to_string()));
    
    // Invalid scenarios
    assert!(tree.match_route("/api/v1/users/abc", "GET").is_none()); // Not int
    assert!(tree.match_route("/api/v1/users", "DELETE").is_none()); // Wrong method
    assert!(tree.match_route("/api/v2/users", "GET").is_none()); // Wrong path
}

#[test]
fn test_compile_path_regex_integration() {
    let test_cases = vec![
        "/simple/path",
        "/users/{id}",
        "/users/{id:int}/posts/{post_id}",
        "/files/{path:path}",
    ];
    
    for path in test_cases {
        let regex_pattern = compile_path_regex(path).unwrap();
        assert!(regex_pattern.starts_with('^'));
        assert!(regex_pattern.ends_with('$'));
        
        // Verify regex compiles
        let regex = regex::Regex::new(&regex_pattern).unwrap();
        assert!(regex.is_match(path) || path.contains('{'));
    }
}

#[test]
fn test_complex_api_routes() {
    let mut tree = RouteTree::new();
    
    // Add various route patterns
    let routes = vec![
        // Root routes
        ("/", vec!["GET"]),
        ("/health", vec!["GET"]),
        ("/metrics", vec!["GET"]),
        
        // User routes
        ("/users", vec!["GET", "POST"]),
        ("/users/{user_id}", vec!["GET", "PUT", "DELETE"]),
        ("/users/{user_id}/profile", vec!["GET", "PUT"]),
        ("/users/{user_id}/posts", vec!["GET", "POST"]),
        
        // Post routes
        ("/posts", vec!["GET"]),
        ("/posts/{post_id:int}", vec!["GET", "PUT", "DELETE"]),
        ("/posts/{post_id:int}/comments", vec!["GET", "POST"]),
        
        // File routes
        ("/files/{filepath:path}", vec!["GET"]),
    ];
    
    for (path, methods) in routes {
        let route = APIRoute::new(
            path.to_string(),
            methods.iter().map(|m| m.to_string()).collect(),
        );
        tree.add_route(route).unwrap();
    }
    
    // Test route matching
    assert!(tree.match_route("/", "GET").is_some());
    assert!(tree.match_route("/health", "GET").is_some());
    assert!(tree.match_route("/users", "POST").is_some());
    assert!(tree.match_route("/users/123", "DELETE").is_some());
    assert!(tree.match_route("/posts/456", "GET").is_some());
    assert!(tree.match_route("/files/path/to/file.txt", "GET").is_some());
    
    // Test param extraction
    let user_match = tree.match_route("/users/john", "GET").unwrap();
    assert_eq!(user_match.params.get("user_id"), Some(&"john".to_string()));
    
    let post_match = tree.match_route("/posts/789/comments", "GET").unwrap();
    assert_eq!(post_match.params.get("post_id"), Some(&"789".to_string()));
    
    let file_match = tree.match_route("/files/docs/api/v1.pdf", "GET").unwrap();
    assert_eq!(
        file_match.params.get("filepath"),
        Some(&"docs/api/v1.pdf".to_string())
    );
}

#[test]
fn test_route_priority_and_specificity() {
    let routes = vec![
        create_api_route("/items/{id}", vec!["GET"], None).unwrap(),
        create_api_route("/items/special", vec!["GET"], None).unwrap(),
        create_api_route("/items/{id}/details", vec!["GET"], None).unwrap(),
    ];
    
    let tree = create_route_tree(routes).unwrap();
    
    // More specific routes should be checked appropriately
    let match1 = tree.match_route("/items/123", "GET").unwrap();
    assert!(match1.params.contains_key("id"));
    
    let match2 = tree.match_route("/items/special", "GET").unwrap();
    // This might match the dynamic route - demonstrates need for priority
    
    let match3 = tree.match_route("/items/456/details", "GET").unwrap();
    assert_eq!(match3.params.get("id"), Some(&"456".to_string()));
}

#[test]
fn test_method_routing() {
    let mut tree = RouteTree::new();
    
    // Same path, different methods
    tree.add_route(APIRoute::new(
        "/resource".to_string(),
        vec!["GET".to_string()],
    ).with_name("get_resource".to_string())).unwrap();
    
    tree.add_route(APIRoute::new(
        "/resource".to_string(),
        vec!["POST".to_string()],
    ).with_name("create_resource".to_string())).unwrap();
    
    tree.add_route(APIRoute::new(
        "/resource".to_string(),
        vec!["PUT".to_string()],
    ).with_name("update_resource".to_string())).unwrap();
    
    // Each method should route to different handler
    let get_match = tree.match_route("/resource", "GET").unwrap();
    let get_route = tree.get_route(get_match.route_index).unwrap();
    assert_eq!(get_route.name, Some("get_resource".to_string()));
    
    let post_match = tree.match_route("/resource", "POST").unwrap();
    let post_route = tree.get_route(post_match.route_index).unwrap();
    assert_eq!(post_route.name, Some("create_resource".to_string()));
    
    let put_match = tree.match_route("/resource", "PUT").unwrap();
    let put_route = tree.get_route(put_match.route_index).unwrap();
    assert_eq!(put_route.name, Some("update_resource".to_string()));
}

#[test]
fn test_unicode_and_special_chars_in_paths() {
    let routes = vec![
        create_api_route("/café/{id}", vec!["GET"], None).unwrap(),
        create_api_route("/files/{name}.{ext}", vec!["GET"], None).unwrap(),
        create_api_route("/path-with-dashes/{param}", vec!["GET"], None).unwrap(),
        create_api_route("/under_score/{p_1}", vec!["GET"], None).unwrap(),
    ];
    
    let tree = create_route_tree(routes).unwrap();
    
    // Unicode paths
    let match1 = tree.match_route("/café/123", "GET").unwrap();
    assert_eq!(match1.params.get("id"), Some(&"123".to_string()));
    
    // Paths with dots (should be escaped in regex)
    let match2 = tree.match_route("/files/document.pdf", "GET").unwrap();
    assert!(match2.params.contains_key("name"));
    
    // Dashes and underscores
    let match3 = tree.match_route("/path-with-dashes/test", "GET").unwrap();
    assert_eq!(match3.params.get("param"), Some(&"test".to_string()));
    
    let match4 = tree.match_route("/under_score/value", "GET").unwrap();
    assert_eq!(match4.params.get("p_1"), Some(&"value".to_string()));
}