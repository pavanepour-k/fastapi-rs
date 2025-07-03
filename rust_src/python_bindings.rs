use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyBytes, PyAny};
use std::collections::HashMap;
use crate::{core, params, serialization, security, utils, types};

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
        let route = core::routing::create_api_route(path, &methods.iter().map(|s| s.as_str()).collect::<Vec<_>>(), name.as_deref())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        let py_route = types::FastApiRoute {
            path: route.path.clone(),
            methods: route.methods.clone(),
            name: route.name.clone(),
            path_format: route.path_format.clone().unwrap_or_default(),
            inner: core::Route::from_api_route(route),
        };
        
        Py::new(py, py_route)
    })
}

#[pyfunction]
pub fn match_route(
    path: &str,
    method: &str,
    routes: Vec<Py<types::FastApiRoute>>,
) -> PyResult<Option<(usize, HashMap<String, String>)>> {
    Python::with_gil(|py| {
        let api_routes: PyResult<Vec<_>> = routes.iter()
            .map(|r| {
                let route_ref = r.borrow(py);
                Ok(route_ref.inner.to_api_route())
            })
            .collect();
        let api_routes = api_routes?;
        
        Ok(core::routing::match_route(path, method, &api_routes))
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
        let schema_map = dict_to_value_map(schema)?;
        
        let result = params::validation::validate_path_params(param_map, schema_map)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
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
        let schema_map = dict_to_value_map(schema)?;
        
        let result = params::validation::validate_query_params(param_map, schema_map)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
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
        let schema_map = dict_to_value_map(schema)?;
        
        let result = params::validation::validate_header_params(header_map, schema_map)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
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
                "Body must be bytes or string"
            ));
        };
        
        let schema_map = dict_to_value_map(schema)?;
        let result = params::validation::validate_body_params(body_data, schema_map)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
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
pub fn serialize_response(
    data: &Bound<PyAny>,
    content_type: Option<&str>,
) -> PyResult<Vec<u8>> {
    serialization::encoders::serialize_response(data, content_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pyfunction]
pub fn deserialize_request(
    body: &Bound<PyBytes>,
    content_type: &str,
) -> PyResult<Py<PyAny>> {
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
pub fn hash_password(
    password: &str,
    algorithm: Option<&str>,
) -> PyResult<String> {
    security::utils::hash_password(password, algorithm)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

// Utility functions
#[pyfunction]
pub fn generate_unique_id(
    route_name: &str,
    method: &str,
    path: &str,
) -> PyResult<String> {
    Ok(utils::id_generation::generate_unique_id(route_name, method, path))
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

// OAuth2 functions
#[pyfunction]
pub fn create_oauth2_server() -> PyResult<Py<OAuth2ServerWrapper>> {
    Python::with_gil(|py| {
        let server = security::oauth2::OAuth2Server::new();
        Py::new(py, OAuth2ServerWrapper { inner: server })
    })
}

#[pyclass]
pub struct OAuth2ServerWrapper {
    inner: security::oauth2::OAuth2Server,
}

#[pymethods]
impl OAuth2ServerWrapper {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: security::oauth2::OAuth2Server::new(),
        }
    }
    
    pub fn register_client(
        &mut self,
        client_id: String,
        client_secret: Option<String>,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
    ) -> PyResult<()> {
        let client = security::oauth2::OAuth2Client::new(client_id, client_secret)
            .with_redirect_uris(redirect_uris)
            .with_scopes(scopes);
        
        self.inner.register_client(client);
        Ok(())
    }
    
    pub fn create_authorization_code(
        &mut self,
        client_id: &str,
        redirect_uri: &str,
        scopes: Vec<String>,
        user_id: Option<String>,
    ) -> PyResult<String> {
        self.inner.create_authorization_code(
            client_id,
            redirect_uri,
            scopes,
            None, // code_challenge
            None, // code_challenge_method
            user_id,
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    
    pub fn exchange_authorization_code(
        &mut self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
    ) -> PyResult<HashMap<String, serde_json::Value>> {
        let token_response = self.inner.exchange_authorization_code(
            code,
            client_id,
            client_secret,
            redirect_uri,
            None, // code_verifier
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        let mut result = HashMap::new();
        result.insert("access_token".to_string(), serde_json::Value::String(token_response.access_token));
        result.insert("token_type".to_string(), serde_json::Value::String(token_response.token_type));
        result.insert("expires_in".to_string(), serde_json::Value::Number(token_response.expires_in.into()));
        
        if let Some(refresh_token) = token_response.refresh_token {
            result.insert("refresh_token".to_string(), serde_json::Value::String(refresh_token));
        }
        
        if let Some(scope) = token_response.scope {
            result.insert("scope".to_string(), serde_json::Value::String(scope));
        }
        
        Ok(result)
    }
    
    pub fn validate_access_token(&self, token: &str) -> PyResult<HashMap<String, serde_json::Value>> {
        let access_token = self.inner.validate_access_token(token)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        let mut result = HashMap::new();
        result.insert("client_id".to_string(), serde_json::Value::String(access_token.client_id.clone()));
        result.insert("scopes".to_string(), serde_json::Value::Array(
            access_token.scopes.iter().map(|s| serde_json::Value::String(s.clone())).collect()
        ));
        
        if let Some(user_id) = &access_token.user_id {
            result.insert("user_id".to_string(), serde_json::Value::String(user_id.clone()));
        }
        
        Ok(result)
    }
}

// Async utilities
#[pyfunction]
pub fn run_async_task(py: Python, coro: &PyAny) -> PyResult<Py<PyAny>> {
    utils::async_tools::run_async_python(&coro.as_borrowed())
}

#[pyfunction] 
pub fn create_async_context() -> PyResult<Py<AsyncContextWrapper>> {
    Python::with_gil(|py| {
        let ctx = utils::async_tools::AsyncContext::new();
        Py::new(py, AsyncContextWrapper { inner: ctx })
    })
}

#[pyclass]
pub struct AsyncContextWrapper {
    inner: utils::async_tools::AsyncContext,
}

#[pymethods]
impl AsyncContextWrapper {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: utils::async_tools::AsyncContext::new(),
        }
    }
}

// Rate limiting
#[pyfunction]
pub fn create_rate_limiter(limit: u32, window_seconds: u32) -> PyResult<Py<RateLimiterWrapper>> {
    Python::with_gil(|py| {
        let mut limiter = HashMap::new();
        Py::new(py, RateLimiterWrapper { 
            limiters: limiter,
            limit,
            window_seconds,
        })
    })
}

#[pyclass]
pub struct RateLimiterWrapper {
    limiters: HashMap<String, types::models::RateLimitModel>,
    limit: u32,
    window_seconds: u32,
}

#[pymethods]
impl RateLimiterWrapper {
    #[new]
    pub fn new(limit: u32, window_seconds: u32) -> Self {
        Self {
            limiters: HashMap::new(),
            limit,
            window_seconds,
        }
    }
    
    pub fn is_allowed(&mut self, key: &str) -> PyResult<bool> {
        let rate_limit = self.limiters.entry(key.to_string())
            .or_insert_with(|| types::models::RateLimitModel::new(key.to_string(), self.limit, self.window_seconds));
        
        Ok(rate_limit.increment())
    }
    
    pub fn remaining(&self, key: &str) -> PyResult<u32> {
        match self.limiters.get(key) {
            Some(rate_limit) => Ok(rate_limit.remaining()),
            None => Ok(self.limit),
        }
    }
    
    pub fn reset(&mut self, key: &str) -> PyResult<()> {
        if let Some(rate_limit) = self.limiters.get_mut(key) {
            rate_limit.reset();
        }
        Ok(())
    }
}

// Helper functions
fn dict_to_value_map(dict: &Bound<PyDict>) -> PyResult<HashMap<String, serde_json::Value>> {
    let mut result = HashMap::new();
    
    for (key, value) in dict.iter() {
        let key_str = key.str()?.to_str()?.to_string();
        let json_value = py_any_to_json_value(&value)?;
        result.insert(key_str, json_value);
    }
    
    Ok(result)
}

fn py_any_to_json_value(py_any: &Bound<PyAny>) -> PyResult<serde_json::Value> {
    if py_any.is_none() {
        return Ok(serde_json::Value::Null);
    }
    
    if let Ok(s) = py_any.downcast::<PyString>() {
        return Ok(serde_json::Value::String(s.to_str()?.to_string()));
    }
    
    if let Ok(i) = py_any.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    
    if let Ok(f) = py_any.extract::<f64>() {
        let num = serde_json::Number::from_f64(f)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid float"))?;
        return Ok(serde_json::Value::Number(num));
    }
    
    if let Ok(b) = py_any.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }
    
    if let Ok(list) = py_any.downcast::<PyList>() {
        let mut json_array = Vec::new();
        for item in list.iter() {
            json_array.push(py_any_to_json_value(&item)?);
        }
        return Ok(serde_json::Value::Array(json_array));
    }
    
    if let Ok(dict) = py_any.downcast::<PyDict>() {
        return dict_to_value_map(dict).map(serde_json::Value::Object);
    }
    
    // Fallback to string representation
    let str_repr = py_any.str()?.to_str()?.to_string();
    Ok(serde_json::Value::String(str_repr))
}

// Extension trait to bridge between API route types
pub trait RouteConversion {
    fn from_api_route(api_route: core::routing::APIRoute) -> Self;
    fn to_api_route(&self) -> core::routing::APIRoute;
}

impl RouteConversion for core::Route {
    fn from_api_route(api_route: core::routing::APIRoute) -> Self {
        // This is a simplified conversion
        // In a real implementation, you'd need proper mapping
        core::Route {
            path: api_route.path,
            methods: api_route.methods.into_iter().collect(),
            name: api_route.name,
            regex: std::sync::Arc::new(regex::Regex::new("").unwrap()), // Placeholder
            param_names: api_route.param_names.into(),
            path_format: api_route.path_format.unwrap_or_default(),
        }
    }
    
    fn to_api_route(&self) -> core::routing::APIRoute {
        core::routing::APIRoute {
            path: self.path.clone(),
            methods: self.methods.iter().cloned().collect(),
            name: self.name.clone(),
            path_regex: None,
            path_format: Some(self.path_format.clone()),
            param_names: self.param_names.iter().cloned().collect(),
            include_in_schema: true,
            tags: Vec::new(),
        }
    }
}