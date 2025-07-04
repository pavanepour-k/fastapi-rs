//! Path pattern compilation for route matching.

use once_cell::sync::Lazy;
use regex::Regex;
use smallvec::SmallVec;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilationError {
    #[error("Invalid path pattern: {0}")]
    InvalidPath(String),
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

type CompilationResult<T> = std::result::Result<T, CompilationError>;

static PATH_PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)(?::([a-zA-Z_]+))?\}")
        .unwrap()
});

/// Compile path pattern to regex and extract parameters.
pub fn compile_path_pattern(
    path: &str,
) -> CompilationResult<(String, SmallVec<[String; 4]>, String)> {
    if !path.starts_with('/') {
        return Err(CompilationError::InvalidPath(
            "Path must start with '/'".to_string(),
        ));
    }

    let mut pattern = String::with_capacity(path.len() * 2);
    let mut param_names = SmallVec::new();
    let mut path_format = String::with_capacity(path.len());
    let mut last_end = 0;

    pattern.push('^');

    for cap in PATH_PARAM_REGEX.captures_iter(path) {
        let full_match = cap.get(0).unwrap();
        let param_name = cap.get(1).unwrap().as_str();
        let param_type = cap.get(2).map(|m| m.as_str()).unwrap_or("str");

        pattern.push_str(&regex::escape(
            &path[last_end..full_match.start()],
        ));
        path_format.push_str(&path[last_end..full_match.start()]);

        let regex_part = match param_type {
            "int" => r"([0-9]+)",
            "float" => r"([0-9]*\.?[0-9]+)",
            "uuid" => {
                r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
            }
            "path" => r"(.+)",
            "str" | _ => r"([^/]+)",
        };

        pattern.push_str(regex_part);
        path_format.push('{');
        path_format.push_str(param_name);
        path_format.push('}');
        param_names.push(param_name.to_string());

        last_end = full_match.end();
    }

    pattern.push_str(&regex::escape(&path[last_end..]));
    path_format.push_str(&path[last_end..]);
    pattern.push('$');

    validate_compiled_pattern(&pattern)?;

    Ok((pattern, param_names, path_format))
}

/// Extract parameter type from path segment.
pub fn extract_param_type(segment: &str) -> Option<(&str, &str)> {
    PATH_PARAM_REGEX.captures(segment).and_then(|cap| {
        let name = cap.get(1)?.as_str();
        let param_type = cap.get(2).map(|m| m.as_str()).unwrap_or("str");
        Some((name, param_type))
    })
}

/// Validate compiled regex pattern.
fn validate_compiled_pattern(pattern: &str) -> CompilationResult<()> {
    Regex::new(pattern)?;
    Ok(())
}

/// Check if path contains parameters.
pub fn has_path_params(path: &str) -> bool {
    PATH_PARAM_REGEX.is_match(path)
}

/// Count number of parameters in path.
pub fn count_path_params(path: &str) -> usize {
    PATH_PARAM_REGEX.captures_iter(path).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let (pattern, params, format) =
            compile_path_pattern("/users").unwrap();
        assert_eq!(pattern, r"^/users$");
        assert!(params.is_empty());
        assert_eq!(format, "/users");
    }

    #[test]
    fn test_path_with_params() {
        let (pattern, params, format) =
            compile_path_pattern("/users/{id}").unwrap();
        assert_eq!(pattern, r"^/users/([^/]+)$");
        assert_eq!(params.as_slice(), &["id"]);
        assert_eq!(format, "/users/{id}");
    }

    #[test]
    fn test_typed_params() {
        let cases = vec![
            (
                "/users/{id:int}",
                r"^/users/([0-9]+)$",
                vec!["id"],
            ),
            (
                "/items/{price:float}",
                r"^/items/([0-9]*\.?[0-9]+)$",
                vec!["price"],
            ),
            (
                "/files/{path:path}",
                r"^/files/(.+)$",
                vec!["path"],
            ),
        ];

        for (path, expected_pattern, expected_params) in cases {
            let (pattern, params, _) =
                compile_path_pattern(path).unwrap();
            assert_eq!(pattern, expected_pattern);
            assert_eq!(params.into_vec(), expected_params);
        }
    }

    #[test]
    fn test_multiple_params() {
        let (pattern, params, format) = compile_path_pattern(
            "/users/{user_id}/posts/{post_id:int}",
        )
        .unwrap();
        assert_eq!(pattern, r"^/users/([^/]+)/posts/([0-9]+)$");
        assert_eq!(params.as_slice(), &["user_id", "post_id"]);
        assert_eq!(format, "/users/{user_id}/posts/{post_id}");
    }

    #[test]
    fn test_invalid_path() {
        assert!(compile_path_pattern("users").is_err());
        assert!(compile_path_pattern("").is_err());
    }

    #[test]
    fn test_has_path_params() {
        assert!(has_path_params("/users/{id}"));
        assert!(has_path_params("/users/{id:int}/posts"));
        assert!(!has_path_params("/users"));
        assert!(!has_path_params("/api/v1/health"));
    }
}