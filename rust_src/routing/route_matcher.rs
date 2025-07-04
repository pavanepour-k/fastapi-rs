use ahash::AHashMap;
use parking_lot::RwLock;
use regex::Regex;
use std::sync::Arc;
use thiserror::Error;

use crate::routing::route_model::{APIRoute, CompiledRoute, RouteMatch};
use crate::routing::path_compiler::{compile_path_pattern, CompilationError};

#[derive(Error, Debug)]
pub enum MatchError {
    #[error("Route not found: {0}")]
    NotFound(String),
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    #[error("Compilation error: {0}")]
    Compilation(#[from] CompilationError),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

type MatchResult<T> = std::result::Result<T, MatchError>;

static REGEX_CACHE: once_cell::sync::Lazy<RwLock<AHashMap<String, Arc<Regex>>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(AHashMap::new()));

pub fn get_or_compile_regex(pattern: &str) -> MatchResult<Arc<Regex>> {
    if let Some(regex) = REGEX_CACHE.read().get(pattern) {
        return Ok(Arc::clone(regex));
    }

    let regex = Arc::new(Regex::new(pattern)?);
    REGEX_CACHE.write().insert(pattern.to_string(), Arc::clone(&regex));
    Ok(regex)
}

pub fn compile_route(route: &mut APIRoute) -> MatchResult<()> {
    let (pattern, param_names, path_format) = 
        compile_path_pattern(&route.path)?;
    route.path_regex = Some(pattern);
    route.param_names = param_names.into_vec();
    route.path_format = Some(path_format);
    Ok(())
}

pub fn match_route(
    path: &str,
    method: &str,
    routes: &[CompiledRoute],
) -> Option<RouteMatch> {
    let method_upper = method.to_uppercase();
    
    for (index, compiled) in routes.iter().enumerate() {
        if !compiled.route.supports_method(&method_upper) {
            continue;
        }
        
        if let Some(params) = compiled.match_path(path) {
            return Some(RouteMatch::new(index, params));
        }
    }
    
    None
}

pub fn match_single_route(
    route: &APIRoute,
    path: &str,
    method: &str,
) -> MatchResult<AHashMap<String, String>> {
    if !route.supports_method(method) {
        return Err(MatchError::MethodNotAllowed(method.to_string()));
    }

    if let Some(regex_pattern) = &route.path_regex {
        let regex = get_or_compile_regex(regex_pattern)?;
        if let Some(captures) = regex.captures(path) {
            let mut params = AHashMap::new();
            for (i, name) in route.param_names.iter().enumerate() {
                if let Some(value) = captures.get(i + 1) {
                    params.insert(name.clone(), value.as_str().to_string());
                }
            }
            return Ok(params);
        }
    }

    Err(MatchError::NotFound(path.to_string()))
}

pub fn create_compiled_routes(
    routes: Vec<APIRoute>,
) -> MatchResult<Vec<CompiledRoute>> {
    routes
        .into_iter()
        .map(|mut route| {
            compile_route(&mut route)?;
            let regex = get_or_compile_regex(
                route.path_regex.as_ref().unwrap()
            )?;
            Ok(CompiledRoute::new(route, regex))
        })
        .collect()
}