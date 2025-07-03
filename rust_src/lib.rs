use pyo3::prelude::*;

pub mod core;
pub mod params;
pub mod serialization;
pub mod security;
pub mod utils;
pub mod types;

mod python_bindings;

use python_bindings::*;

#[cfg(jemalloc_enabled)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// Core Route struct used throughout the library
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: smallvec::SmallVec<[String; 4]>,
    pub name: Option<String>,
    pub regex: std::sync::Arc<regex::Regex>,
    pub param_names: smallvec::SmallVec<[String; 4]>,
    pub path_format: String,
}

impl Route {
    pub fn new(path: &str, methods: Vec<String>, name: Option<String>) -> Result<Self, core::routing::RoutingError> {
        let (regex_pattern, param_names, path_format) = core::routing::compile_path_pattern(path)?;
        let regex = core::routing::get_or_compile_regex(&regex_pattern)?;
        
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

#[pymodule]
fn _fastapi_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_rust_backend, m)?)?;
    
    // Core routing functions
    m.add_function(wrap_pyfunction!(create_api_route, m)?)?;
    m.add_function(wrap_pyfunction!(match_route, m)?)?;
    m.add_function(wrap_pyfunction!(compile_path_regex, m)?)?;
    
    // Parameter validation functions
    m.add_function(wrap_pyfunction!(validate_path_params, m)?)?;
    m.add_function(wrap_pyfunction!(validate_query_params, m)?)?;
    m.add_function(wrap_pyfunction!(validate_header_params, m)?)?;
    m.add_function(wrap_pyfunction!(validate_body_params, m)?)?;
    
    // Serialization functions
    m.add_function(wrap_pyfunction!(jsonable_encoder, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_response, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_request, m)?)?;
    
    // Security functions
    m.add_function(wrap_pyfunction!(constant_time_compare, m)?)?;
    m.add_function(wrap_pyfunction!(verify_api_key, m)?)?;
    m.add_function(wrap_pyfunction!(hash_password, m)?)?;
    
    // Utility functions
    m.add_function(wrap_pyfunction!(generate_unique_id, m)?)?;
    m.add_function(wrap_pyfunction!(parse_content_type, m)?)?;
    m.add_function(wrap_pyfunction!(convert_python_type, m)?)?;
    
    // OAuth2 functions
    m.add_function(wrap_pyfunction!(create_oauth2_server, m)?)?;
    
    // Async utilities
    m.add_function(wrap_pyfunction!(run_async_task, m)?)?;
    m.add_function(wrap_pyfunction!(create_async_context, m)?)?;
    
    // Rate limiting
    m.add_function(wrap_pyfunction!(create_rate_limiter, m)?)?;
    
    // Type system
    m.add_class::<types::FastApiRoute>()?;
    m.add_class::<types::ValidationResult>()?;
    m.add_class::<types::RequestData>()?;
    m.add_class::<OAuth2ServerWrapper>()?;
    m.add_class::<AsyncContextWrapper>()?;
    m.add_class::<RateLimiterWrapper>()?;
    
    Ok(())
}