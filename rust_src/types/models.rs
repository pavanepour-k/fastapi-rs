use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareModel {
    pub name: String,
    pub enabled: bool,
    pub order: i32,
    pub config: HashMap<String, serde_json::Value>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitModel {
    pub key: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub current_count: u32,
    pub reset_time: DateTime<Utc>,
}

impl RateLimitModel {
    pub fn new(key: String, limit: u32, window_seconds: u32) -> Self {
        Self {
            key,
            limit,
            window_seconds,
            current_count: 0,
            reset_time: Utc::now() + chrono::Duration::seconds(window_seconds as i64),
        }
    }
    
    pub fn is_exceeded(&self) -> bool {
        self.current_count >= self.limit
    }
    
    pub fn increment(&mut self) -> bool {
        if self.is_expired() {
            self.reset();
        }
        
        if self.current_count < self.limit {
            self.current_count += 1;
            true
        } else {
            false
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.reset_time
    }
    
    pub fn reset(&mut self) {
        self.current_count = 0;
        self.reset_time = Utc::now() + chrono::Duration::seconds(self.window_seconds as i64);
    }
    
    pub fn remaining(&self) -> u32 {
        if self.is_expired() {
            self.limit
        } else {
            self.limit.saturating_sub(self.current_count)
        }
    }
}

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
    
    pub fn time_to_live(&self) -> Option<i64> {
        self.expires_at.map(|exp| (exp - Utc::now()).num_seconds().max(0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorModel {
    pub error_type: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
    pub status_code: Option<u16>,
}

impl ErrorModel {
    pub fn new(error_type: String, message: String) -> Self {
        Self {
            error_type,
            message,
            details: None,
            timestamp: Utc::now(),
            request_id: None,
            status_code: None,
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
    
    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }
    
    pub fn validation_error(field: String, message: String) -> Self {
        let mut details = HashMap::new();
        details.insert("field".to_string(), serde_json::Value::String(field));
        
        Self::new("ValidationError".to_string(), message)
            .with_details(details)
            .with_status_code(422)
    }
    
    pub fn authentication_error(message: String) -> Self {
        Self::new("AuthenticationError".to_string(), message)
            .with_status_code(401)
    }
    
    pub fn authorization_error(message: String) -> Self {
        Self::new("AuthorizationError".to_string(), message)
            .with_status_code(403)
    }
    
    pub fn not_found_error(resource: String) -> Self {
        Self::new("NotFoundError".to_string(), format!("{} not found", resource))
            .with_status_code(404)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConnectionModel {
    pub connection_id: String,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub message_count: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WebSocketConnectionModel {
    pub fn new(connection_id: String, client_ip: String) -> Self {
        let now = Utc::now();
        Self {
            connection_id,
            client_ip,
            user_agent: None,
            connected_at: now,
            last_activity: now,
            message_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
            metadata: HashMap::new(),
        }
    }
    
    pub fn update_activity(&mut self, message_size: u64, is_outgoing: bool) {
        self.last_activity = Utc::now();
        self.message_count += 1;
        
        if is_outgoing {
            self.bytes_sent += message_size;
        } else {
            self.bytes_received += message_size;
        }
    }
    
    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.connected_at
    }
    
    pub fn idle_time(&self) -> chrono::Duration {
        Utc::now() - self.last_activity
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
    fn test_rate_limit_model() {
        let mut rate_limit = RateLimitModel::new("user123".to_string(), 5, 60);
        
        assert!(rate_limit.increment());
        assert_eq!(rate_limit.current_count, 1);
        assert_eq!(rate_limit.remaining(), 4);
        
        for _ in 0..4 {
            assert!(rate_limit.increment());
        }
        
        assert!(!rate_limit.increment());
        assert!(rate_limit.is_exceeded());
        assert_eq!(rate_limit.remaining(), 0);
    }
    
    #[test]
    fn test_cache_entry_model() {
        let mut entry = CacheEntryModel::new(
            "test_key".to_string(), 
            serde_json::json!("test_value")
        ).with_ttl(3600);
        
        assert!(!entry.is_expired());
        assert!(entry.time_to_live().unwrap() > 3500);
        
        entry.touch();
        assert_eq!(entry.access_count, 1);
    }
    
    #[test]
    fn test_error_model_builders() {
        let validation_error = ErrorModel::validation_error(
            "email".to_string(), 
            "Invalid email format".to_string()
        );
        
        assert_eq!(validation_error.error_type, "ValidationError");
        assert_eq!(validation_error.status_code, Some(422));
        assert!(validation_error.details.is_some());
        
        let auth_error = ErrorModel::authentication_error("Token expired".to_string());
        assert_eq!(auth_error.status_code, Some(401));
        
        let not_found = ErrorModel::not_found_error("User".to_string());
        assert_eq!(not_found.message, "User not found");
        assert_eq!(not_found.status_code, Some(404));
    }
    
    #[test]
    fn test_websocket_connection_model() {
        let mut conn = WebSocketConnectionModel::new(
            "conn123".to_string(),
            "192.168.1.1".to_string()
        );
        
        conn.update_activity(100, true);
        assert_eq!(conn.message_count, 1);
        assert_eq!(conn.bytes_sent, 100);
        assert_eq!(conn.bytes_received, 0);
        
        conn.update_activity(50, false);
        assert_eq!(conn.message_count, 2);
        assert_eq!(conn.bytes_received, 50);
    }
}