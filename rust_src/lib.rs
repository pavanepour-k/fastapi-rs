use pyo3::prelude::*;

pub mod core;
pub mod params;
pub mod security;
pub mod serialization;
pub mod types;
pub mod utils;

mod python_bindings;

use python_bindings::*;

#[cfg(jemalloc_enabled)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

    // Type system
    m.add_class::<types::FastApiRoute>()?;
    m.add_class::<types::ValidationResult>()?;
    m.add_class::<types::RequestData>()?;

    Ok(())
}
