//! Route model definitions and core structures.

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;

/// Core route representation for API endpoints.
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
    /// Create a new route with given path and methods.
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

    /// Add a name to the route.
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Add tags to the route.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set schema inclusion flag.
    pub fn include_in_schema(mut self, include: bool) -> Self {
        self.include_in_schema = include;
        self
    }

    /// Get the route name or generate from path.
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.path
                .trim_start_matches('/')
                .replace('/', "_")
                .replace('{', "")
                .replace('}', "")
        })
    }

    /// Check if route supports given method.
    pub fn supports_method(&self, method: &str) -> bool {
        self.methods.contains(&method.to_uppercase())
    }
}

/// Compiled route data for fast matching.
#[derive(Debug, Clone)]
pub struct CompiledRoute {
    pub route: APIRoute,
    pub regex: Arc<regex::Regex>,
    pub param_indices: SmallVec<[(String, usize); 4]>,
}

impl CompiledRoute {
    /// Create compiled route from APIRoute and regex.
    pub fn new(
        route: APIRoute,
        regex: Arc<regex::Regex>,
    ) -> Self {
        let param_indices = route
            .param_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i + 1))
            .collect();

        Self {
            route,
            regex,
            param_indices,
        }
    }

    /// Match path and extract parameters.
    pub fn match_path(
        &self,
        path: &str,
    ) -> Option<AHashMap<String, String>> {
        self.regex.captures(path).map(|captures| {
            let mut params = AHashMap::new();
            for (name, index) in &self.param_indices {
                if let Some(value) = captures.get(*index) {
                    params.insert(
                        name.clone(),
                        value.as_str().to_string(),
                    );
                }
            }
            params
        })
    }
}

/// Route match result with parameters.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    pub route_index: usize,
    pub params: AHashMap<String, String>,
}

impl RouteMatch {
    /// Create new route match.
    pub fn new(
        route_index: usize,
        params: AHashMap<String, String>,
    ) -> Self {
        Self { route_index, params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_route_creation() {
        let route = APIRoute::new(
            "/users/{id}".to_string(),
            vec!["GET".to_string(), "POST".to_string()],
        );

        assert_eq!(route.path, "/users/{id}");
        assert_eq!(route.methods, vec!["GET", "POST"]);
        assert!(route.name.is_none());
        assert!(route.include_in_schema);
    }

    #[test]
    fn test_route_builder() {
        let route = APIRoute::new(
            "/api/v1/users".to_string(),
            vec!["GET".to_string()],
        )
        .with_name("get_users".to_string())
        .with_tags(vec!["users".to_string()])
        .include_in_schema(false);

        assert_eq!(route.name, Some("get_users".to_string()));
        assert_eq!(route.tags, vec!["users"]);
        assert!(!route.include_in_schema);
    }

    #[test]
    fn test_supports_method() {
        let route = APIRoute::new(
            "/test".to_string(),
            vec!["GET".to_string(), "POST".to_string()],
        );

        assert!(route.supports_method("GET"));
        assert!(route.supports_method("get"));
        assert!(route.supports_method("POST"));
        assert!(!route.supports_method("PUT"));
        assert!(!route.supports_method("DELETE"));
    }

    #[test]
    fn test_get_name() {
        let route1 = APIRoute::new(
            "/users/{id}/posts".to_string(),
            vec!["GET".to_string()],
        );
        assert_eq!(route1.get_name(), "users_id_posts");

        let route2 = APIRoute::new(
            "/api/v1/items".to_string(),
            vec!["GET".to_string()],
        )
        .with_name("list_items".to_string());
        assert_eq!(route2.get_name(), "list_items");
    }
}