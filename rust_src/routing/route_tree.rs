use ahash::AHashMap;
use thiserror::Error;

use crate::routing::route_model::{APIRoute, CompiledRoute, RouteMatch};
use crate::routing::route_matcher::{
    compile_route, create_compiled_routes, match_route as match_route_impl,
    get_or_compile_regex, MatchError,
};

#[derive(Error, Debug)]
pub enum RouteTreeError {
    #[error("Duplicate route: {0}")]
    DuplicateRoute(String),
    #[error("Route not found")]
    NotFound,
    #[error("Match error: {0}")]
    Match(#[from] MatchError),
}

type Result<T> = std::result::Result<T, RouteTreeError>;

#[derive(Default)]
pub struct RouteTree {
    routes: Vec<APIRoute>,
    compiled_routes: Vec<CompiledRoute>,
    static_routes: AHashMap<String, Vec<usize>>,
    dynamic_routes: Vec<usize>,
}

impl RouteTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_route(&mut self, mut route: APIRoute) -> Result<()> {
        compile_route(&mut route)?;

        let index = self.routes.len();
        let key = format!("{}:{}", route.path, route.methods.join(","));

        if route.param_names.is_empty() {
            if self.static_routes.contains_key(&key) {
                return Err(RouteTreeError::DuplicateRoute(key));
            }
            self.static_routes.entry(key).or_default().push(index);
        } else {
            self.dynamic_routes.push(index);
        }

        let regex = get_or_compile_regex(
            route.path_regex.as_ref().unwrap()
        )?;
        self.compiled_routes.push(CompiledRoute::new(
            route.clone(),
            regex,
        ));
        self.routes.push(route);
        Ok(())
    }

    pub fn match_route(
        &self,
        path: &str,
        method: &str,
    ) -> Option<RouteMatch> {
        let static_key = self.static_routes.iter()
            .find(|(key, _)| {
                let parts: Vec<&str> = key.split(':').collect();
                if parts.len() == 2 {
                    let route_path = parts[0];
                    let methods = parts[1];
                    route_path == path && 
                    methods.split(',').any(|m| m == method.to_uppercase())
                } else {
                    false
                }
            });

        if let Some((_, indices)) = static_key {
            if let Some(&index) = indices.first() {
                return Some(RouteMatch::new(index, AHashMap::new()));
            }
        }

        let dynamic_compiled: Vec<&CompiledRoute> = self.dynamic_routes
            .iter()
            .filter_map(|&idx| self.compiled_routes.get(idx))
            .collect();

        match_route_impl(path, method, &dynamic_compiled)
            .map(|mut m| {
                m.route_index = self.dynamic_routes[m.route_index];
                m
            })
    }

    pub fn get_route(&self, index: usize) -> Option<&APIRoute> {
        self.routes.get(index)
    }

    pub fn all_routes(&self) -> &[APIRoute] {
        &self.routes
    }

    pub fn clear(&mut self) {
        self.routes.clear();
        self.compiled_routes.clear();
        self.static_routes.clear();
        self.dynamic_routes.clear();
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

pub fn create_route_tree(routes: Vec<APIRoute>) -> Result<RouteTree> {
    let mut tree = RouteTree::new();
    for route in routes {
        tree.add_route(route)?;
    }
    Ok(tree)
}