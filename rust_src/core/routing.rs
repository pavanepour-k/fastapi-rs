//! High-performance routing module for FastAPI-RS.

use ahash::AHashMap;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("Invalid path pattern: {0}")]
    InvalidPath(String),
    #[error("Route not found: {0}")]
    NotFound(String),
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    #[error("Duplicate route: {0}")]
    DuplicateRoute(String),
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

type RoutingResult<T> = std::result::Result<T, RoutingError>;

static PATH_PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)(?::([a-zA-Z_]+))?\}").unwrap()
});

static REGEX_CACHE: Lazy<RwLock<AHashMap<String, Arc<Regex>>>> =
    Lazy::new(|| RwLock::new(AHashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIRoute {
    pub path: String,
    pub methods: Vec<String>,
    pub name: Option<String>,
    pub path_regex: Option<String>,
    pub path_format: Option<String>,
    pub param_names: Vec<String>,
    pub include_in_schema: bool,
    pub tags: Vec<String>,
}

impl APIRoute {
    pub fn new(path: String, methods: Vec<String>) -> Self {
        Self {
            path,
            methods,
            name: None,
            path_regex: None,
            path_format: None,
            param_names: Vec::new(),
            include_in_schema: true,
            tags: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn compile(&mut self) -> RoutingResult<()> {
        let (pattern, param_names, path_format) = compile_path_pattern(&self.path)?;
        self.path_regex = Some(pattern);
        self.param_names = param_names.into_vec();
        self.path_format = Some(path_format);
        Ok(())
    }

    pub fn matches(&self, path: &str, method: &str) -> Option<AHashMap<String, String>> {
        if !self.methods.contains(&method.to_uppercase()) {
            return None;
        }

        if let Some(regex_pattern) = &self.path_regex {
            let regex = get_or_compile_regex(regex_pattern).ok()?;
            if let Some(captures) = regex.captures(path) {
                let mut params = AHashMap::new();
                for (i, name) in self.param_names.iter().enumerate() {
                    if let Some(value) = captures.get(i + 1) {
                        params.insert(name.clone(), value.as_str().to_string());
                    }
                }
                return Some(params);
            }
        }

        None
    }
}

#[derive(Default)]
pub struct RouteTree {
    routes: Vec<APIRoute>,
    static_routes: AHashMap<String, Vec<usize>>,
    dynamic_routes: Vec<usize>,
}

impl RouteTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(&mut self, mut route: APIRoute) -> RoutingResult<()> {
        route.compile()?;
        
        let index = self.routes.len();
        
        if route.param_names.is_empty() {
            for method in &route.methods {
                let key = format!("{}:{}", route.path, method);
                if self.static_routes.contains_key(&key) {
                    return Err(RoutingError::DuplicateRoute(key));
                }
                self.static_routes.entry(key).or_default().push(index);
            }
        } else {
            self.dynamic_routes.push(index);
        }
        
        self.routes.push(route);
        Ok(())
    }

    pub fn match_route(&self, path: &str, method: &str) -> Option<(usize, AHashMap<String, String>)> {
        let key = format!("{}:{}", path, method.to_uppercase());
        if let Some(indices) = self.static_routes.get(&key) {
            if let Some(&index) = indices.first() {
                return Some((index, AHashMap::new()));
            }
        }
        
        for &index in &self.dynamic_routes {
            if let Some(params) = self.routes[index].matches(path, method) {
                return Some((index, params));
            }
        }
        
        None
    }

    pub fn get_route(&self, index: usize) -> Option<&APIRoute> {
        self.routes.get(index)
    }

    pub fn all_routes(&self) -> &[APIRoute] {
        &self.routes
    }
}

pub fn create_api_route(path: &str, methods: Vec<&str>, name: Option<&str>) -> RoutingResult<APIRoute> {
    let mut route = APIRoute::new(
        path.to_string(),
        methods.iter().map(|m| m.to_uppercase()).collect(),
    );

    if let Some(n) = name {
        route = route.with_name(n.to_string());
    }

    route.compile()?;
    Ok(route)
}

pub fn match_route<'a>(
    path: &str,
    method: &str,
    routes: &'a [APIRoute],
) -> Option<(usize, AHashMap<String, String>)> {
    for (index, route) in routes.iter().enumerate() {
        if let Some(params) = route.matches(path, method) {
            return Some((index, params));
        }
    }
    None
}

pub fn compile_path_regex(path: &str) -> RoutingResult<String> {
    let (pattern, _, _) = compile_path_pattern(path)?;
    Ok(pattern)
}

fn compile_path_pattern(path: &str) -> RoutingResult<(String, SmallVec<[String; 4]>, String)> {
    if !path.starts_with('/') {
        return Err(RoutingError::InvalidPath("Path must start with '/'".to_string()));
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
        
        pattern.push_str(&regex::escape(&path[last_end..full_match.start()]));
        path_format.push_str(&path[last_end..full_match.start()]);
        
        let regex_part = match param_type {
            "int" => r"([0-9]+)",
            "float" => r"([0-9]*\.?[0-9]+)",
            "uuid" => r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})",
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
    pattern.push('

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pattern_compilation() {
        let cases = vec![
            ("/users", r"^/users$", vec![], "/users"),
            ("/users/{id}", r"^/users/([^/]+)$", vec!["id"], "/users/{id}"),
            ("/users/{id:int}", r"^/users/([0-9]+)$", vec!["id"], "/users/{id}"),
            ("/files/{path:path}", r"^/files/(.+)$", vec!["path"], "/files/{path}"),
        ];

        for (path, expected_pattern, expected_params, expected_format) in cases {
            let (pattern, params, format) = compile_path_pattern(path).unwrap();
            assert_eq!(pattern, expected_pattern);
            assert_eq!(params.into_vec(), expected_params);
            assert_eq!(format, expected_format);
        }
    }

    #[test]
    fn test_route_matching() {
        let mut route = APIRoute::new("/users/{id:int}".to_string(), vec!["GET".to_string()]);
        route.compile().unwrap();

        assert!(route.matches("/users/123", "GET").is_some());
        assert!(route.matches("/users/abc", "GET").is_none());
        assert!(route.matches("/users/123", "POST").is_none());
    }

    #[test]
    fn test_route_tree() {
        let mut tree = RouteTree::new();
        
        let route1 = create_api_route("/users", vec!["GET"], None).unwrap();
        let route2 = create_api_route("/users/{id:int}", vec!["GET", "PUT"], None).unwrap();
        
        tree.add_route(route1).unwrap();
        tree.add_route(route2).unwrap();
        
        let (index, params) = tree.match_route("/users", "GET").unwrap();
        assert_eq!(index, 0);
        assert!(params.is_empty());
        
        let (index, params) = tree.match_route("/users/123", "GET").unwrap();
        assert_eq!(index, 1);
        assert_eq!(params.get("id").unwrap(), "123");
    }
});
    
    Ok((pattern, param_names, path_format))
}

pub fn get_or_compile_regex(pattern: &str) -> RoutingResult<Arc<Regex>> {
    if let Some(cached) = REGEX_CACHE.read().get(pattern) {
        return Ok(cached.clone());
    }
    
    let mut cache = REGEX_CACHE.write();
    if let Some(cached) = cache.get(pattern) {
        return Ok(cached.clone());
    }
    
    let regex = Arc::new(Regex::new(pattern)?);
    cache.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pattern_compilation() {
        let cases = vec![
            ("/users", r"^/users$", vec![], "/users"),
            ("/users/{id}", r"^/users/([^/]+)$", vec!["id"], "/users/{id}"),
            ("/users/{id:int}", r"^/users/([0-9]+)$", vec!["id"], "/users/{id}"),
            ("/files/{path:path}", r"^/files/(.+)$", vec!["path"], "/files/{path}"),
        ];

        for (path, expected_pattern, expected_params, expected_format) in cases {
            let (pattern, params, format) = compile_path_pattern(path).unwrap();
            assert_eq!(pattern, expected_pattern);
            assert_eq!(params.into_vec(), expected_params);
            assert_eq!(format, expected_format);
        }
    }

    #[test]
    fn test_route_matching() {
        let mut route = APIRoute::new("/users/{id:int}".to_string(), vec!["GET".to_string()]);
        route.compile().unwrap();

        assert!(route.matches("/users/123", "GET").is_some());
        assert!(route.matches("/users/abc", "GET").is_none());
        assert!(route.matches("/users/123", "POST").is_none());
    }

    #[test]
    fn test_route_tree() {
        let mut tree = RouteTree::new();
        
        let route1 = create_api_route("/users", vec!["GET"], None).unwrap();
        let route2 = create_api_route("/users/{id:int}", vec!["GET", "PUT"], None).unwrap();
        
        tree.add_route(route1).unwrap();
        tree.add_route(route2).unwrap();
        
        let (index, params) = tree.match_route("/users", "GET").unwrap();
        assert_eq!(index, 0);
        assert!(params.is_empty());
        
        let (index, params) = tree.match_route("/users/123", "GET").unwrap();
        assert_eq!(index, 1);
        assert_eq!(params.get("id").unwrap(), "123");
    }
}