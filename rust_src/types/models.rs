use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rust equivalent of FastAPI route model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteModel {
    pub path: String,
    pub methods: Vec<String>,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub include_in_schema: bool,
}

impl Default for RouteModel {
    fn default() -> Self {
        Self {
            path: String::new(),
            methods: vec!["GET".to_string()],
            name: None,
            summary: None,
            description: None,
            tags: Vec::new(),
            deprecated: false,
            include_in_schema: true,
        }
    }
}

/// Request validation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationModel {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub validated_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_type: String,
    pub input_value: Option<String>,
}

/// Parameter schema model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterModel {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    pub example: Option<serde_json::Value>,
    pub constraints: ParameterConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<String>>,
    pub multiple_of: Option<f64>,
}

impl Default for ParameterConstraints {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            pattern: None,
            enum_values: None,
            multiple_of: None,
        }
    }
}

/// Security scheme model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySchemeModel {
    pub scheme_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub flows: Option<OAuthFlowsModel>,
    pub openid_connect_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowsModel {
    pub implicit: Option<OAuthFlowModel>,
    pub password: Option<OAuthFlowModel>,
    pub client_credentials: Option<OAuthFlowModel>,
    pub authorization_code: Option<OAuthFlowModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowModel {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

/// Request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestModel {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, Vec<String>>,
    pub path_params: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub content_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl RequestModel {
    pub fn new(method: String, url: String) -> Self {
        Self {
            method,
            url,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            body: None,
            content_type: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_body(mut self, body: Vec<u8>, content_type: String) -> Self {
        self.body = Some(body);
        self.content_type = Some(content_type);
        self
    }
}

/// Response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseModel {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub content_type: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl ResponseModel {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: None,
            content_type: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_json_body(mut self, body: serde_json::Value) -> Result<Self, serde_json::Error> {
        let json_bytes = serde_json::to_vec(&body)?;
        self.body = Some(json_bytes);
        self.content_type = Some("application/json".to_string());
        Ok(self)
    }

    pub fn with_text_body(mut self, text: String) -> Self {
        self.body = Some(text.into_bytes());
        self.content_type = Some("text/plain".to_string());
        self
    }
}

/// Middleware model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareModel {
    pub name: String,
    pub enabled: bool,
    pub order: i32,
    pub config: HashMap<String, serde_json::Value>,
}

/// Application configuration model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigModel {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub debug: bool,
    pub docs_url: Option<String>,
    pub redoc_url: Option<String>,
    pub openapi_url: Option<String>,
    pub middleware: Vec<MiddlewareModel>,
    pub cors_config: Option<CorsConfigModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfigModel {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u32>,
}

impl Default for CorsConfigModel {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age: None,
        }
    }
}

/// Performance metrics model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub request_count: u64,
    pub average_response_time: f64,
    pub min_response_time: f64,
    pub max_response_time: f64,
    pub error_count: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub timestamp: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            average_response_time: 0.0,
            min_response_time: f64::MAX,
            max_response_time: 0.0,
            error_count: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
            timestamp: Utc::now(),
        }
    }
}

/// Rate limiting model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitModel {
    pub key: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub current_count: u32,
    pub reset_time: DateTime<Utc>,
}

/// Cache entry model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryModel {
    pub key: String,
    pub value: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
}

impl CacheEntryModel {
    pub fn new(key: String, value: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            key,
            value,
            expires_at: None,
            created_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self {
        self.expires_at = Some(Utc::now() + chrono::Duration::seconds(ttl_seconds));
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

/// Error model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorModel {
    pub error_type: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}

impl ErrorModel {
    pub fn new(error_type: String, message: String) -> Self {
        Self {
            error_type,
            message,
            details: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_model_default() {
        let route = RouteModel::default();
        assert_eq!(route.path, "");
        assert_eq!(route.methods, vec!["GET"]);
        assert!(!route.deprecated);
        assert!(route.include_in_schema);
    }

    #[test]
    fn test_request_model_builder() {
        let request = RequestModel::new("POST".to_string(), "/api/users".to_string())
            .with_body(b"test body".to_vec(), "text/plain".to_string());

        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "/api/users");
        assert_eq!(request.body, Some(b"test body".to_vec()));
        assert_eq!(request.content_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_response_model_json() {
        let response = ResponseModel::new(200)
            .with_json_body(serde_json::json!({"message": "success"}))
            .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_type, Some("application/json".to_string()));
        assert!(response.body.is_some());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let mut entry =
            CacheEntryModel::new("test_key".to_string(), serde_json::json!("test_value"))
                .with_ttl(-1); // Already expired

        assert!(entry.is_expired());

        entry.touch();
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_error_model_builder() {
        let mut details = HashMap::new();
        details.insert("field".to_string(), serde_json::json!("username"));

        let error = ErrorModel::new("ValidationError".to_string(), "Invalid input".to_string())
            .with_details(details)
            .with_request_id("req_123".to_string());

        assert_eq!(error.error_type, "ValidationError");
        assert_eq!(error.message, "Invalid input");
        assert!(error.details.is_some());
        assert_eq!(error.request_id, Some("req_123".to_string()));
    }
}
