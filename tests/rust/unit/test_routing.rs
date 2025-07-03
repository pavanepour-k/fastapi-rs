#[path = "../common/mod.rs"]
mod common;

use fastapi_rs::core::routing::*;
use common::*;

#[test]
fn test_create_api_route() {
    setup();
    let route = create_api_route("/users/{id}", vec!["GET"], None).unwrap();
    assert_eq!(route.path, "/users/{id}");
    assert!(route.methods.contains(&"GET".to_string()));
}

#[test]
fn test_route_compilation() {
    let mut route = APIRoute::new("/users/{id:int}".to_string(), vec!["GET".to_string()]);
    route.compile().unwrap();
    assert!(route.path_regex.is_some());
    assert_eq!(route.param_names, vec!["id"]);
}

#[test]
fn test_route_matching() {
    let mut route = APIRoute::new("/users/{id:int}".to_string(), vec!["GET".to_string()]);
    route.compile().unwrap();
    
    assert!(assert_route_matches(&route, "/users/123", "GET"));
    assert!(!assert_route_matches(&route, "/users/abc", "GET"));
    assert!(!assert_route_matches(&route, "/users/123", "POST"));
}

#[test]
fn test_path_regex_compilation() {
    let regex = compile_path_regex("/users/{id:int}").unwrap();
    assert_eq!(regex, r"^/users/([0-9]+)$");
}

#[test]
fn test_route_tree() {
    let mut tree = RouteTree::new();
    let route1 = create_api_route("/users", vec!["GET"], None).unwrap();
    let route2 = create_api_route("/users/{id}", vec!["GET"], None).unwrap();
    
    tree.add_route(route1).unwrap();
    tree.add_route(route2).unwrap();
    
    let (index, params) = tree.match_route("/users", "GET").unwrap();
    assert_eq!(index, 0);
    assert!(params.is_empty());
    
    let (index, params) = tree.match_route("/users/123", "GET").unwrap();
    assert_eq!(index, 1);
    assert_eq!(params.get("id").unwrap(), "123");
}