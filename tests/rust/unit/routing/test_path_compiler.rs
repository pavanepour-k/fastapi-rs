//! Unit tests for path compilation functionality.

use rust_src::routing::path_compiler::{
    compile_path_pattern, extract_param_type, has_path_params,
    count_path_params, CompilationError,
};

#[test]
fn test_compile_simple_paths() {
    let test_cases = vec![
        ("/", r"^/$", vec![], "/"),
        ("/users", r"^/users$", vec![], "/users"),
        ("/api/v1/items", r"^/api/v1/items$", vec![], "/api/v1/items"),
    ];

    for (path, expected_pattern, expected_params, expected_format) in test_cases {
        let (pattern, params, format) = compile_path_pattern(path).unwrap();
        assert_eq!(pattern, expected_pattern);
        assert_eq!(params.into_vec(), expected_params);
        assert_eq!(format, expected_format);
    }
}

#[test]
fn test_compile_paths_with_params() {
    let test_cases = vec![
        (
            "/users/{id}",
            r"^/users/([^/]+)$",
            vec!["id"],
            "/users/{id}",
        ),
        (
            "/users/{user_id}/posts/{post_id}",
            r"^/users/([^/]+)/posts/([^/]+)$",
            vec!["user_id", "post_id"],
            "/users/{user_id}/posts/{post_id}",
        ),
        (
            "/{category}/{item_id}",
            r"^/([^/]+)/([^/]+)$",
            vec!["category", "item_id"],
            "/{category}/{item_id}",
        ),
    ];

    for (path, expected_pattern, expected_params, expected_format) in test_cases {
        let (pattern, params, format) = compile_path_pattern(path).unwrap();
        assert_eq!(pattern, expected_pattern);
        assert_eq!(params.into_vec(), expected_params);
        assert_eq!(format, expected_format);
    }
}

#[test]
fn test_compile_typed_params() {
    let test_cases = vec![
        (
            "/users/{id:int}",
            r"^/users/([0-9]+)$",
            vec!["id"],
        ),
        (
            "/prices/{amount:float}",
            r"^/prices/([0-9]*\.?[0-9]+)$",
            vec!["amount"],
        ),
        (
            "/docs/{id:uuid}",
            r"^/docs/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$",
            vec!["id"],
        ),
        (
            "/files/{filepath:path}",
            r"^/files/(.+)$",
            vec!["filepath"],
        ),
        (
            "/items/{name:str}",
            r"^/items/([^/]+)$",
            vec!["name"],
        ),
    ];

    for (path, expected_pattern, expected_params) in test_cases {
        let (pattern, params, _) = compile_path_pattern(path).unwrap();
        assert_eq!(pattern, expected_pattern);
        assert_eq!(params.into_vec(), expected_params);
    }
}

#[test]
fn test_compile_mixed_typed_params() {
    let (pattern, params, format) = compile_path_pattern(
        "/api/v{version:int}/users/{user_id}/items/{price:float}"
    ).unwrap();
    
    assert_eq!(
        pattern,
        r"^/api/v([0-9]+)/users/([^/]+)/items/([0-9]*\.?[0-9]+)$"
    );
    assert_eq!(params.into_vec(), vec!["version", "user_id", "price"]);
    assert_eq!(format, "/api/v{version}/users/{user_id}/items/{price}");
}

#[test]
fn test_compile_path_with_special_chars() {
    let test_cases = vec![
        "/api/v1.0/users", 
        "/files/test.txt",
        "/path/with-dashes",
        "/path_with_underscores",
    ];

    for path in test_cases {
        let (pattern, params, format) = compile_path_pattern(path).unwrap();
        assert!(params.is_empty());
        assert_eq!(format, path);
        // Check regex escaping worked
        assert!(pattern.contains(r"\.") || !path.contains('.'));
    }
}

#[test]
fn test_invalid_paths() {
    let invalid_paths = vec![
        "users",  // No leading slash
        "",       // Empty path
        "//users", // Double slash (though might be valid in some contexts)
    ];

    for path in invalid_paths {
        let result = compile_path_pattern(path);
        assert!(result.is_err());
        if let Err(CompilationError::InvalidPath(msg)) = result {
            assert!(msg.contains("Path must start with '/'"));
        }
    }
}

#[test]
fn test_extract_param_type() {
    let test_cases = vec![
        ("{id}", Some(("id", "str"))),
        ("{id:int}", Some(("id", "int"))),
        ("{user_id:str}", Some(("user_id", "str"))),
        ("{price:float}", Some(("price", "float"))),
        ("{path:path}", Some(("path", "path"))),
        ("static", None),
        ("{}", None), // Invalid param
    ];

    for (segment, expected) in test_cases {
        let result = extract_param_type(segment);
        assert_eq!(result, expected);
    }
}

#[test]
fn test_has_path_params() {
    assert!(has_path_params("/users/{id}"));
    assert!(has_path_params("/api/{version}/items"));
    assert!(has_path_params("/{param:int}"));
    
    assert!(!has_path_params("/users"));
    assert!(!has_path_params("/api/v1/items"));
    assert!(!has_path_params("/"));
}

#[test]
fn test_count_path_params() {
    assert_eq!(count_path_params("/users"), 0);
    assert_eq!(count_path_params("/users/{id}"), 1);
    assert_eq!(count_path_params("/users/{id}/posts/{post_id}"), 2);
    assert_eq!(count_path_params("/{a}/{b}/{c}/{d}"), 4);
    assert_eq!(count_path_params("/{id:int}/{name:str}"), 2);
}

#[test]
fn test_compile_path_param_names_validation() {
    // Valid param names
    let valid_names = vec![
        "/users/{id}",
        "/users/{user_id}",
        "/users/{userId}",
        "/users/{id123}",
        "/users/{_id}",
    ];

    for path in valid_names {
        assert!(compile_path_pattern(path).is_ok());
    }
}

#[test]
fn test_path_format_preservation() {
    let test_cases = vec![
        "/users/{id}/edit",
        "/api/v{version:int}/docs",
        "/files/{path:path}",
    ];

    for original_path in test_cases {
        let (_, _, format) = compile_path_pattern(original_path).unwrap();
        // Format should preserve parameter names but not types
        assert!(format.contains("{"));
        assert!(format.contains("}"));
        assert!(!format.contains(":"));
    }
}