pub mod models;

use crate::core::Route;
use crate::params::ValidationResult as RustValidationResult;
use pyo3::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

#[pyclass]
#[derive(Debug, Clone)]
pub struct FastApiRoute {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub methods: Vec<String>,
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub path_format: String,
    pub(crate) inner: Route,
}

#[pymethods]
impl FastApiRoute {
    #[new]
    pub fn new(path: String, methods: Vec<String>, name: Option<String>) -> PyResult<Self> {
        let route = Route::new(&path, methods.clone(), name.clone())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(FastApiRoute {
            path: path.clone(),
            methods,
            name,
            path_format: route.path_format.clone(),
            inner: route,
        })
    }

    pub fn matches(&self, path: &str, method: &str) -> bool {
        if !self.methods.iter().any(|m| m == method) {
            return false;
        }
        self.inner.regex.is_match(path)
    }

    pub fn extract_params(&self, path: &str) -> Option<HashMap<String, String>> {
        if let Some(captures) = self.inner.regex.captures(path) {
            let mut params = HashMap::new();
            for (i, param_name) in self.inner.param_names.iter().enumerate() {
                if let Some(capture) = captures.get(i + 1) {
                    params.insert(param_name.clone(), capture.as_str().to_string());
                }
            }
            Some(params)
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FastApiRoute(path='{}', methods={:?}, name={:?})",
            self.path, self.methods, self.name
        )
    }
}

impl FastApiRoute {
    pub fn from(route: Route) -> Self {
        FastApiRoute {
            path: route.path.clone(),
            methods: route.methods.iter().cloned().collect(),
            name: route.name.clone(),
            path_format: route.path_format.clone(),
            inner: route,
        }
    }

    pub fn to_rust_route(&self) -> Route {
        self.inner.clone()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    #[pyo3(get)]
    pub valid: bool,
    #[pyo3(get)]
    pub errors: Vec<String>,
    #[pyo3(get)]
    pub validated_data: HashMap<String, Value>,
}

#[pymethods]
impl ValidationResult {
    #[new]
    pub fn new(valid: bool, errors: Vec<String>, validated_data: HashMap<String, Value>) -> Self {
        Self {
            valid,
            errors,
            validated_data,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn get_data(&self) -> HashMap<String, Value> {
        self.validated_data.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(valid={}, errors={}, data_keys={:?})",
            self.valid,
            self.errors.len(),
            self.validated_data.keys().collect::<Vec<_>>()
        )
    }
}

impl ValidationResult {
    pub fn from(result: RustValidationResult) -> Self {
        let errors = result.errors.iter().map(|e| e.to_string()).collect();
        ValidationResult {
            valid: result.valid,
            errors,
            validated_data: result.validated_data,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct RequestData {
    #[pyo3(get)]
    pub method: String,
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub headers: HashMap<String, String>,
    #[pyo3(get)]
    pub query_params: HashMap<String, Vec<String>>,
    #[pyo3(get)]
    pub content_type: Option<String>,
    pub body: Option<Vec<u8>>,
}

#[pymethods]
impl RequestData {
    #[new]
    pub fn new(
        method: String,
        path: String,
        headers: HashMap<String, String>,
        query_params: HashMap<String, Vec<String>>,
        body: Option<Vec<u8>>,
    ) -> Self {
        let content_type = headers
            .get("content-type")
            .map(|ct| ct.split(';').next().unwrap_or(ct).trim().to_lowercase());

        Self {
            method,
            path,
            headers,
            query_params,
            content_type,
            body,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    pub fn get_query_param(&self, name: &str) -> Option<String> {
        self.query_params.get(name)?.first().cloned()
    }

    pub fn get_query_params(&self, name: &str) -> Option<Vec<String>> {
        self.query_params.get(name).cloned()
    }

    pub fn has_body(&self) -> bool {
        self.body.as_ref().map_or(false, |b| !b.is_empty())
    }

    pub fn body_size(&self) -> usize {
        self.body.as_ref().map_or(0, |b| b.len())
    }

    pub fn is_json(&self) -> bool {
        self.content_type
            .as_ref()
            .map_or(false, |ct| ct.starts_with("application/json"))
    }

    pub fn is_form_data(&self) -> bool {
        self.content_type.as_ref().map_or(false, |ct| {
            ct.starts_with("application/x-www-form-urlencoded")
        })
    }

    pub fn is_multipart(&self) -> bool {
        self.content_type
            .as_ref()
            .map_or(false, |ct| ct.starts_with("multipart/form-data"))
    }

    #[getter]
    pub fn body_bytes(&self) -> Option<Vec<u8>> {
        self.body.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "RequestData(method='{}', path='{}', content_type={:?}, body_size={})",
            self.method,
            self.path,
            self.content_type,
            self.body_size()
        )
    }
}

pub use models::*;
