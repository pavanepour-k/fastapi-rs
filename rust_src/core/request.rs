use std::collections::HashMap;
use bytes::Bytes;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
    #[error("Body parsing failed: {0}")]
    BodyParsingError(String),
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),
}

pub type Result<T> = std::result::Result<T, RequestError>;

#[derive(Debug, Clone)]
pub struct RequestData {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, Vec<String>>,
    pub body: Option<Bytes>,
    pub content_type: Option<String>,
}

impl RequestData {
    pub fn new(
        method: String,
        path: String,
        headers: HashMap<String, String>,
        query_params: HashMap<String, Vec<String>>,
        body: Option<Bytes>,
    ) -> Self {
        let content_type = headers.get("content-type")
            .map(|ct| ct.split(';').next().unwrap_or(ct).trim().to_lowercase());
            
        Self {
            method,
            path,
            headers,
            query_params,
            body,
            content_type,
        }
    }
    
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
    
    pub fn get_query_param(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)?.first()
    }
    
    pub fn get_query_params(&self, name: &str) -> Option<&Vec<String>> {
        self.query_params.get(name)
    }
    
    pub fn has_body(&self) -> bool {
        self.body.as_ref().map_or(false, |b| !b.is_empty())
    }
    
    pub fn body_size(&self) -> usize {
        self.body.as_ref().map_or(0, |b| b.len())
    }
    
    pub fn is_json(&self) -> bool {
        self.content_type.as_ref()
            .map_or(false, |ct| ct.starts_with("application/json"))
    }
    
    pub fn is_form_data(&self) -> bool {
        self.content_type.as_ref()
            .map_or(false, |ct| ct.starts_with("application/x-www-form-urlencoded"))
    }
    
    pub fn is_multipart(&self) -> bool {
        self.content_type.as_ref()
            .map_or(false, |ct| ct.starts_with("multipart/form-data"))
    }
    
    pub fn parse_json_body(&self) -> Result<Value> {
        match &self.body {
            Some(body) if !body.is_empty() => {
                serde_json::from_slice(body)
                    .map_err(|e| RequestError::BodyParsingError(e.to_string()))
            }
            _ => Ok(Value::Null),
        }
    }
    
    pub fn parse_form_body(&self) -> Result<HashMap<String, Vec<String>>> {
        match &self.body {
            Some(body) if !body.is_empty() => {
                let body_str = std::str::from_utf8(body)
                    .map_err(|e| RequestError::BodyParsingError(e.to_string()))?;
                
                Ok(parse_form_data(body_str))
            }
            _ => Ok(HashMap::new()),
        }
    }
}

pub fn parse_query_string(query: &str) -> HashMap<String, Vec<String>> {
    let mut params = HashMap::new();
    
    if query.is_empty() {
        return params;
    }
    
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let key = percent_decode(key);
            let value = percent_decode(value);
            
            params.entry(key).or_insert_with(Vec::new).push(value);
        } else if !pair.is_empty() {
            let key = percent_decode(pair);
            params.entry(key).or_insert_with(Vec::new).push(String::new());
        }
    }
    
    params
}

pub fn parse_form_data(data: &str) -> HashMap<String, Vec<String>> {
    parse_query_string(data)
}

pub fn percent_decode(input: &str) -> String {
    percent_encoding::percent_decode_str(input)
        .decode_utf8_lossy()
        .into_owned()
}

pub fn normalize_header_name(name: &str) -> String {
    name.to_lowercase()
}

pub fn parse_content_type(content_type: &str) -> (String, HashMap<String, String>) {
    let mut parts = content_type.split(';');
    let media_type = parts.next().unwrap_or("").trim().to_lowercase();
    
    let mut params = HashMap::new();
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim().trim_matches('"').to_string();
            params.insert(key, value);
        }
    }
    
    (media_type, params)
}

#[derive(Debug, Clone)]
pub struct ParsedRequest {
    pub data: RequestData,
    pub path_params: HashMap<String, String>,
    pub route_index: Option<usize>,
}

impl ParsedRequest {
    pub fn new(data: RequestData) -> Self {
        Self {
            data,
            path_params: HashMap::new(),
            route_index: None,
        }
    }
    
    pub fn with_route_match(
        mut self, 
        route_index: usize, 
        path_params: HashMap<String, String>
    ) -> Self {
        self.route_index = Some(route_index);
        self.path_params = path_params;
        self
    }
    
    pub fn get_path_param(&self, name: &str) -> Option<&String> {
        self.path_params.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let params = parse_query_string("name=John&age=30&tags=rust&tags=web");
        
        assert_eq!(params.get("name"), Some(&vec!["John".to_string()]));
        assert_eq!(params.get("age"), Some(&vec!["30".to_string()]));
        assert_eq!(params.get("tags"), Some(&vec!["rust".to_string(), "web".to_string()]));
    }
    
    #[test]
    fn test_parse_query_string_encoded() {
        let params = parse_query_string("message=Hello%20World&special=%21%40%23");
        
        assert_eq!(params.get("message"), Some(&vec!["Hello World".to_string()]));
        assert_eq!(params.get("special"), Some(&vec!["!@#".to_string()]));
    }
    
    #[test]
    fn test_parse_content_type() {
        let (media_type, params) = parse_content_type("application/json; charset=utf-8");
        
        assert_eq!(media_type, "application/json");
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
    }
    
    #[test]
    fn test_request_data_json_detection() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let request = RequestData::new(
            "POST".to_string(),
            "/api/users".to_string(),
            headers,
            HashMap::new(),
            Some(Bytes::from(r#"{"name": "John"}"#)),
        );
        
        assert!(request.is_json());
        assert!(!request.is_form_data());
        assert!(request.has_body());
    }
    
    #[test]
    fn test_json_body_parsing() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let request = RequestData::new(
            "POST".to_string(),
            "/api/users".to_string(),
            headers,
            HashMap::new(),
            Some(Bytes::from(r#"{"name": "John", "age": 30}"#)),
        );
        
        let json = request.parse_json_body().unwrap();
        assert_eq!(json["name"], "John");
        assert_eq!(json["age"], 30);
    }
    
    #[test]
    fn test_form_body_parsing() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/x-www-form-urlencoded".to_string());
        
        let request = RequestData::new(
            "POST".to_string(),
            "/api/users".to_string(),
            headers,
            HashMap::new(),
            Some(Bytes::from("name=John&age=30&tags=rust&tags=web")),
        );
        
        let form_data = request.parse_form_body().unwrap();
        assert_eq!(form_data.get("name"), Some(&vec!["John".to_string()]));
        assert_eq!(form_data.get("age"), Some(&vec!["30".to_string()]));
        assert_eq!(form_data.get("tags"), Some(&vec!["rust".to_string(), "web".to_string()]));
    }
    
    #[test]
    fn test_multipart_detection() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "multipart/form-data; boundary=something".to_string());
        
        let request = RequestData::new(
            "POST".to_string(),
            "/upload".to_string(),
            headers,
            HashMap::new(),
            Some(Bytes::from("multipart data")),
        );
        
        assert!(request.is_multipart());
        assert!(!request.is_json());
        assert!(!request.is_form_data());
    }
    
    #[test]
    fn test_header_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        
        let request = RequestData::new(
            "GET".to_string(),
            "/api/protected".to_string(),
            headers,
            HashMap::new(),
            None,
        );
        
        assert_eq!(request.get_header("content-type"), Some(&"application/json".to_string()));
        assert_eq!(request.get_header("AUTHORIZATION"), Some(&"Bearer token123".to_string()));
        assert_eq!(request.get_header("X-Missing"), None);
    }
    
    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("Hello%20World"), "Hello World");
        assert_eq!(percent_decode("test%21%40%23"), "test!@#");
        assert_eq!(percent_decode("no_encoding"), "no_encoding");
        assert_eq!(percent_decode("caf%C3%A9"), "café");
    }
    
    #[test]
    fn test_empty_body_handling() {
        let request = RequestData::new(
            "GET".to_string(),
            "/api/test".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
        );
        
        assert!(!request.has_body());
        assert_eq!(request.body_size(), 0);
        assert_eq!(request.parse_json_body().unwrap(), Value::Null);
        assert!(request.parse_form_body().unwrap().is_empty());
    }
}