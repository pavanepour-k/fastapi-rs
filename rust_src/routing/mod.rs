pub mod route_model;
pub mod path_compiler;
pub mod route_matcher;
pub mod route_tree;

pub use route_model::*;
pub use path_compiler::{compile_path_pattern, CompilationError};
pub use route_matcher::{
    get_or_compile_regex, compile_route, match_route, 
    match_single_route, create_compiled_routes, MatchError,
};
pub use route_tree::{RouteTree, create_route_tree, RouteTreeError};

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
    #[error("Compilation error: {0}")]
    Compilation(#[from] CompilationError),
    #[error("Match error: {0}")]
    Match(#[from] MatchError),
    #[error("Route tree error: {0}")]
    Tree(#[from] RouteTreeError),
}

type RoutingResult<T> = std::result::Result<T, RoutingError>;

pub fn create_api_route(
    path: &str, 
    methods: Vec<&str>, 
    name: Option<&str>
) -> RoutingResult<APIRoute> {
    let mut route = APIRoute::new(
        path.to_string(),
        methods.iter().map(|m| m.to_uppercase()).collect(),
    );

    if let Some(n) = name {
        route = route.with_name(n.to_string());
    }

    compile_route(&mut route)?;
    Ok(route)
}

pub fn compile_path_regex(path: &str) -> RoutingResult<String> {
    let (pattern, _, _) = compile_path_pattern(path)?;
    Ok(pattern)
}