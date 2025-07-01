use crate::{core, params, security, serialization, types, utils};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyDict, PyList, PyString};
use std::collections::HashMap;

#[pyfunction]
pub fn init_rust_backend() -> PyResult<bool> {
    Ok(true)
}

// Core routing functions
#[pyfunction]
pub fn create_api_route(
    path: &str,
    methods: Vec<String>,
    name: Option<String>,
) -> PyResult<Py<types::FastApiRoute>> {
    Python::with_gil(|py| {
        let route = core::routing::create_route(path, methods, name)?;
        Py::new(py, types::FastApiRoute::from(route))
    })
}

#[pyfunction]
pub fn match_route(
    path: &str,
    method: &str,
    routes: Vec<Py<types::FastApiRoute>>,
) -> PyResult<Option<(usize, HashMap<String, String>)>> {
    Python::with_gil(|py| {
        let rust_routes: PyResult<Vec<_>> = routes
            .iter()
            .map(|r| r.borrow(py).to_rust_route())
            .collect();
        let rust_routes = rust_routes?;

        Ok(core::routing::match_route(path, method, &rust_routes))
    })
}

#[pyfunction]
pub fn compile_path_regex(path: &str) -> PyResult<String> {
    core::routing::compile_path_regex(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Parameter validation functions
#[pyfunction]
pub fn validate_path_params(
    params: &Bound<PyDict>,
    schema: &Bound<PyDict>,
) -> PyResult<Py<types::ValidationResult>> {
    Python::with_gil(|py| {
        let param_map = utils::py_dict_to_hashmap(params)?;
        let schema_map = utils::py_dict_to_hashmap(schema)?;

        let result = params::validation::validate_path_params(param_map, schema_map)?;
        Py::new(py, types::ValidationResult::from(result))
    })
}

#[pyfunction]
pub fn validate_query_params(
    params: &Bound<PyDict>,
    schema: &Bound<PyDict>,
) -> PyResult<Py<types::ValidationResult>> {
    Python::with_gil(|py| {
        let param_map = utils::py_dict_to_hashmap(params)?;
        let schema_map = utils::py_dict_to_hashmap(schema)?;

        let result = params::validation::validate_query_params(param_map, schema_map)?;
        Py::new(py, types::ValidationResult::from(result))
    })
}

#[pyfunction]
pub fn validate_header_params(
    headers: &Bound<PyDict>,
    schema: &Bound<PyDict>,
) -> PyResult<Py<types::ValidationResult>> {
    Python::with_gil(|py| {
        let header_map = utils::py_dict_to_hashmap(headers)?;
        let schema_map = utils::py_dict_to_hashmap(schema)?;

        let result = params::validation::validate_header_params(header_map, schema_map)?;
        Py::new(py, types::ValidationResult::from(result))
    })
}

#[pyfunction]
pub fn validate_body_params(
    body: &Bound<PyAny>,
    schema: &Bound<PyDict>,
) -> PyResult<Py<types::ValidationResult>> {
    Python::with_gil(|py| {
        let body_data = if let Ok(bytes) = body.downcast::<PyBytes>() {
            bytes.as_bytes().to_vec()
        } else if let Ok(string) = body.downcast::<PyString>() {
            string.to_str()?.as_bytes().to_vec()
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Body must be bytes or string",
            ));
        };

        let schema_map = utils::py_dict_to_hashmap(schema)?;
        let result = params::validation::validate_body_params(body_data, schema_map)?;
        Py::new(py, types::ValidationResult::from(result))
    })
}

// Serialization functions
#[pyfunction]
pub fn jsonable_encoder(obj: &Bound<PyAny>) -> PyResult<String> {
    serialization::encoders::jsonable_encoder(obj)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
pub fn serialize_response(data: &Bound<PyAny>, content_type: Option<&str>) -> PyResult<Vec<u8>> {
    serialization::encoders::serialize_response(data, content_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
pub fn deserialize_request(body: &Bound<PyBytes>, content_type: &str) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        serialization::decoders::deserialize_request(body.as_bytes(), content_type)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    })
}

// Security functions
#[pyfunction]
pub fn constant_time_compare(a: &str, b: &str) -> PyResult<bool> {
    Ok(security::utils::constant_time_compare(a, b))
}

#[pyfunction]
pub fn verify_api_key(
    provided_key: &str,
    expected_key: &str,
    algorithm: Option<&str>,
) -> PyResult<bool> {
    security::utils::verify_api_key(provided_key, expected_key, algorithm)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
pub fn hash_password(password: &str, algorithm: Option<&str>) -> PyResult<String> {
    security::utils::hash_password(password, algorithm)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Utility functions
#[pyfunction]
pub fn generate_unique_id(route_name: &str, method: &str, path: &str) -> PyResult<String> {
    Ok(utils::id_generation::generate_unique_id(
        route_name, method, path,
    ))
}

#[pyfunction]
pub fn parse_content_type(content_type: &str) -> PyResult<(String, HashMap<String, String>)> {
    utils::content_type::parse_content_type(content_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
pub fn convert_python_type(py_obj: &Bound<PyAny>) -> PyResult<String> {
    utils::type_conv::convert_python_type(py_obj)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}
