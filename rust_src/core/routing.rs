use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use smallvec::SmallVec;
use ahash::AHashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutingError {
    #[error("Invalid path pattern: {0}")]
    InvalidPath(String),
    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Route not found")]
    RouteNotFound,
}

pub type Result<T> = std::result::Result<T, RoutingError>;

static REGEX_CACHE: Lazy<DashMap<String, Arc<Regex>>> = Lazy::new(DashMap::new);
static PATH_PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{([^}:]+)(?::([^}]+))?\}").unwrap()
});

#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: SmallVec<[String; 4]>,
    pub name: Option<String>,
    pub regex: Arc<Regex>,
    pub param_names: SmallVec<[String; 4]>,
    pub path_format: String,
}

impl Route {
    pub fn new(path: &str, methods: Vec<String>, name: Option<String>) -> Result<Self> {
        let (regex_pattern, param_names, path_format) = compile_path_pattern(path)?;
        let regex = get_or_compile_regex(&regex_pattern)?;
        
        Ok(Route {
            path: path.to_string(),
            methods: methods.into(),
            name,
            regex,
            param_names,
            path_format,
        })
    }
}

pub fn create_route(path: &str, methods: Vec<String>, name: Option<String>) -> Result<Route> {
    Route::new(path, methods, name)
}

pub fn match_route(
    path: &str,
    method: &str,
    routes: &[Route],
) -> Option<(usize, HashMap<String, String>)> {
    for (idx, route) in routes.iter().enumerate() {
        if !route.methods.iter().any(|m| m == method) {
            continue;
        }
        
        if let Some(captures) = route.regex.captures(path) {
            let mut params = HashMap::with_capacity(route.param_names.len());
            
            for (i, param_name) in route.param_names.iter().enumerate() {
                if let Some(capture) = captures.get(i + 1) {
                    params.insert(param_name.clone(), capture.as_str().to_string());
                }
            }
            
            return Some((idx, params));
        }
    }
    None
}

pub fn compile_path_regex(path: &str) -> Result<String> {
    let (pattern, _, _) = compile_path_pattern(path)?;
    Ok(pattern)
}

fn compile_path_pattern(path: &str) -> Result<(String, SmallVec<[String; 4]>, String)> {
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
    pattern.push('$');
    
    Ok((pattern, param_names, path_format))
}

fn get_or_compile_regex(pattern: &str) -> Result<Arc<Regex>> {
    if let Some(cached) = REGEX_CACHE.get(pattern) {
        return Ok(cached.clone());
    }
    
    let regex = Arc::new(Regex::new(pattern)?);
    REGEX_CACHE.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

#[derive(Default)]
pub struct RouteTree {}