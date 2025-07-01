use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required parameter: {0}")]
    MissingRequired(String),
    #[error("Invalid type for parameter {param}: expected {expected}, got {actual}")]
    InvalidType {
        param: String,
        expected: String,
        actual: String,
    },
    #[error("Value out of range for parameter {param}: {value}")]
    OutOfRange { param: String, value: String },
    #[error("Invalid format for parameter {param}: {value}")]
    InvalidFormat { param: String, value: String },
    #[error("Parameter {param} does not match pattern: {pattern}")]
    PatternMismatch { param: String, pattern: String },
    #[error("Parameter {param} is too long: {len} > {max}")]
    TooLong {
        param: String,
        len: usize,
        max: usize,
    },
    #[error("Parameter {param} is too short: {len} < {min}")]
    TooShort {
        param: String,
        len: usize,
        min: usize,
    },
}

pub type Result<T> = std::result::Result<T, ValidationError>;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub validated_data: HashMap<String, Value>,
}

impl ValidationResult {
    pub fn success(data: HashMap<String, Value>) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            validated_data: data,
        }
    }

    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            validated_data: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }
}

#[derive(Debug, Clone)]
pub struct ParameterSchema {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<Value>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<String>>,
}

impl ParameterSchema {
    pub fn new(name: String, param_type: String) -> Self {
        Self {
            name,
            param_type,
            required: false,
            default: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            pattern: None,
            enum_values: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn with_length_range(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }

    pub fn with_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.minimum = min;
        self.maximum = max;
        self
    }

    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    pub fn with_enum(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }
}

static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

pub fn validate_path_params(
    params: HashMap<String, String>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult> {
    let schemas = parse_schema_map(schema)?;
    validate_parameters(
        params.into_iter().map(|(k, v)| (k, vec![v])).collect(),
        schemas,
    )
}

pub fn validate_query_params(
    params: HashMap<String, String>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult> {
    let schemas = parse_schema_map(schema)?;
    validate_parameters(
        params.into_iter().map(|(k, v)| (k, vec![v])).collect(),
        schemas,
    )
}

pub fn validate_header_params(
    headers: HashMap<String, String>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult> {
    let schemas = parse_schema_map(schema)?;
    let normalized_headers: HashMap<String, Vec<String>> = headers
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), vec![v]))
        .collect();
    validate_parameters(normalized_headers, schemas)
}

pub fn validate_body_params(
    body: Vec<u8>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult> {
    let body_str = std::str::from_utf8(&body).map_err(|_| ValidationError::InvalidFormat {
        param: "body".to_string(),
        value: "Invalid UTF-8".to_string(),
    })?;

    let json_value: Value =
        serde_json::from_str(body_str).map_err(|_| ValidationError::InvalidFormat {
            param: "body".to_string(),
            value: "Invalid JSON".to_string(),
        })?;

    validate_json_against_schema(json_value, schema)
}

fn validate_parameters(
    params: HashMap<String, Vec<String>>,
    schemas: Vec<ParameterSchema>,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::success(HashMap::new());

    for schema in schemas {
        match params.get(&schema.name) {
            Some(values) if !values.is_empty() => {
                let value = &values[0]; // Take first value for now
                match validate_single_parameter(value, &schema) {
                    Ok(validated_value) => {
                        result
                            .validated_data
                            .insert(schema.name.clone(), validated_value);
                    }
                    Err(error) => {
                        result.add_error(error);
                    }
                }
            }
            _ => {
                if schema.required {
                    result.add_error(ValidationError::MissingRequired(schema.name.clone()));
                } else if let Some(default) = schema.default {
                    result.validated_data.insert(schema.name.clone(), default);
                }
            }
        }
    }

    Ok(result)
}

fn validate_single_parameter(value: &str, schema: &ParameterSchema) -> Result<Value> {
    // Type validation and conversion
    let converted_value =
        match schema.param_type.as_str() {
            "string" | "str" => Value::String(value.to_string()),
            "integer" | "int" => value.parse::<i64>().map(Value::Number).map_err(|_| {
                ValidationError::InvalidType {
                    param: schema.name.clone(),
                    expected: "integer".to_string(),
                    actual: value.to_string(),
                }
            })?,
            "number" | "float" => value
                .parse::<f64>()
                .map(|f| Value::Number(serde_json::Number::from_f64(f).unwrap()))
                .map_err(|_| ValidationError::InvalidType {
                    param: schema.name.clone(),
                    expected: "number".to_string(),
                    actual: value.to_string(),
                })?,
            "boolean" | "bool" => match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Value::Bool(true),
                "false" | "0" | "no" | "off" => Value::Bool(false),
                _ => {
                    return Err(ValidationError::InvalidType {
                        param: schema.name.clone(),
                        expected: "boolean".to_string(),
                        actual: value.to_string(),
                    })
                }
            },
            "email" => {
                if EMAIL_REGEX.is_match(value) {
                    Value::String(value.to_string())
                } else {
                    return Err(ValidationError::InvalidFormat {
                        param: schema.name.clone(),
                        value: value.to_string(),
                    });
                }
            }
            "uuid" => {
                if UUID_REGEX.is_match(value) {
                    Value::String(value.to_string())
                } else {
                    return Err(ValidationError::InvalidFormat {
                        param: schema.name.clone(),
                        value: value.to_string(),
                    });
                }
            }
            _ => Value::String(value.to_string()),
        };

    // Length validation for strings
    if let Value::String(s) = &converted_value {
        if let Some(min_len) = schema.min_length {
            if s.len() < min_len {
                return Err(ValidationError::TooShort {
                    param: schema.name.clone(),
                    len: s.len(),
                    min: min_len,
                });
            }
        }

        if let Some(max_len) = schema.max_length {
            if s.len() > max_len {
                return Err(ValidationError::TooLong {
                    param: schema.name.clone(),
                    len: s.len(),
                    max: max_len,
                });
            }
        }

        // Pattern validation
        if let Some(pattern) = &schema.pattern {
            let regex = Regex::new(pattern).map_err(|_| ValidationError::InvalidFormat {
                param: schema.name.clone(),
                value: format!("Invalid regex pattern: {}", pattern),
            })?;

            if !regex.is_match(s) {
                return Err(ValidationError::PatternMismatch {
                    param: schema.name.clone(),
                    pattern: pattern.clone(),
                });
            }
        }

        // Enum validation
        if let Some(enum_values) = &schema.enum_values {
            if !enum_values.contains(s) {
                return Err(ValidationError::InvalidFormat {
                    param: schema.name.clone(),
                    value: format!("Value must be one of: {}", enum_values.join(", ")),
                });
            }
        }
    }

    // Range validation for numbers
    if let Value::Number(n) = &converted_value {
        let num_value = n.as_f64().unwrap_or(0.0);

        if let Some(min) = schema.minimum {
            if num_value < min {
                return Err(ValidationError::OutOfRange {
                    param: schema.name.clone(),
                    value: num_value.to_string(),
                });
            }
        }

        if let Some(max) = schema.maximum {
            if num_value > max {
                return Err(ValidationError::OutOfRange {
                    param: schema.name.clone(),
                    value: num_value.to_string(),
                });
            }
        }
    }

    Ok(converted_value)
}

fn validate_json_against_schema(
    value: Value,
    _schema: HashMap<String, Value>,
) -> Result<ValidationResult> {
    // Simplified JSON schema validation
    // In a full implementation, this would use a proper JSON schema validator
    let mut validated_data = HashMap::new();
    validated_data.insert("body".to_string(), value);
    Ok(ValidationResult::success(validated_data))
}

fn parse_schema_map(schema: HashMap<String, Value>) -> Result<Vec<ParameterSchema>> {
    let mut schemas = Vec::new();

    for (name, spec) in schema {
        if let Value::Object(spec_obj) = spec {
            let param_type = spec_obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();

            let mut param_schema = ParameterSchema::new(name, param_type);

            if let Some(Value::Bool(required)) = spec_obj.get("required") {
                param_schema.required = *required;
            }

            if let Some(default) = spec_obj.get("default") {
                param_schema.default = Some(default.clone());
            }

            if let Some(Value::Number(min_len)) = spec_obj.get("minLength") {
                param_schema.min_length = min_len.as_u64().map(|n| n as usize);
            }

            if let Some(Value::Number(max_len)) = spec_obj.get("maxLength") {
                param_schema.max_length = max_len.as_u64().map(|n| n as usize);
            }

            if let Some(Value::Number(min)) = spec_obj.get("minimum") {
                param_schema.minimum = min.as_f64();
            }

            if let Some(Value::Number(max)) = spec_obj.get("maximum") {
                param_schema.maximum = max.as_f64();
            }

            if let Some(Value::String(pattern)) = spec_obj.get("pattern") {
                param_schema.pattern = Some(pattern.clone());
            }

            if let Some(Value::Array(enum_vals)) = spec_obj.get("enum") {
                let enum_strings: Vec<String> = enum_vals
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if !enum_strings.is_empty() {
                    param_schema.enum_values = Some(enum_strings);
                }
            }

            schemas.push(param_schema);
        }
    }

    Ok(schemas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_string_parameter() {
        let schema = ParameterSchema::new("name".to_string(), "string".to_string())
            .required()
            .with_length_range(Some(2), Some(50));

        assert!(validate_single_parameter("John", &schema).is_ok());
        assert!(validate_single_parameter("J", &schema).is_err()); // Too short
        assert!(validate_single_parameter(&"x".repeat(51), &schema).is_err()); // Too long
    }

    #[test]
    fn test_validate_integer_parameter() {
        let schema = ParameterSchema::new("age".to_string(), "integer".to_string())
            .with_range(Some(0.0), Some(150.0));

        assert!(validate_single_parameter("25", &schema).is_ok());
        assert!(validate_single_parameter("-1", &schema).is_err()); // Below minimum
        assert!(validate_single_parameter("200", &schema).is_err()); // Above maximum
        assert!(validate_single_parameter("abc", &schema).is_err()); // Invalid type
    }

    #[test]
    fn test_validate_email_parameter() {
        let schema = ParameterSchema::new("email".to_string(), "email".to_string());

        assert!(validate_single_parameter("user@example.com", &schema).is_ok());
        assert!(validate_single_parameter("invalid-email", &schema).is_err());
    }

    #[test]
    fn test_validate_enum_parameter() {
        let schema =
            ParameterSchema::new("role".to_string(), "string".to_string()).with_enum(vec![
                "admin".to_string(),
                "user".to_string(),
                "guest".to_string(),
            ]);

        assert!(validate_single_parameter("admin", &schema).is_ok());
        assert!(validate_single_parameter("invalid", &schema).is_err());
    }

    #[test]
    fn test_validate_boolean_parameter() {
        let schema = ParameterSchema::new("active".to_string(), "boolean".to_string());

        assert!(validate_single_parameter("true", &schema).is_ok());
        assert!(validate_single_parameter("false", &schema).is_ok());
        assert!(validate_single_parameter("1", &schema).is_ok());
        assert!(validate_single_parameter("0", &schema).is_ok());
        assert!(validate_single_parameter("maybe", &schema).is_err());
    }

    #[test]
    fn test_validate_pattern_parameter() {
        let schema = ParameterSchema::new("code".to_string(), "string".to_string())
            .with_pattern(r"^[A-Z]{3}\d{3}$".to_string());

        assert!(validate_single_parameter("ABC123", &schema).is_ok());
        assert!(validate_single_parameter("abc123", &schema).is_err()); // Wrong case
        assert!(validate_single_parameter("ABCD123", &schema).is_err()); // Too many letters
    }

    #[test]
    fn test_missing_required_parameter() {
        let schema = vec![
            ParameterSchema::new("name".to_string(), "string".to_string()).required(),
            ParameterSchema::new("age".to_string(), "integer".to_string()),
        ];

        let params = HashMap::new(); // Empty params
        let result = validate_parameters(params, schema).unwrap();

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0],
            ValidationError::MissingRequired(_)
        ));
    }

    #[test]
    fn test_default_values() {
        let schema = vec![
            ParameterSchema::new("page".to_string(), "integer".to_string()).with_default(json!(1)),
            ParameterSchema::new("limit".to_string(), "integer".to_string())
                .with_default(json!(10)),
        ];

        let params = HashMap::new(); // Empty params
        let result = validate_parameters(params, schema).unwrap();

        assert!(result.valid);
        assert_eq!(result.validated_data.get("page"), Some(&json!(1)));
        assert_eq!(result.validated_data.get("limit"), Some(&json!(10)));
    }

    #[test]
    fn test_uuid_validation() {
        let schema = ParameterSchema::new("id".to_string(), "uuid".to_string());

        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid = "not-a-uuid";

        assert!(validate_single_parameter(valid_uuid, &schema).is_ok());
        assert!(validate_single_parameter(invalid_uuid, &schema).is_err());
    }
}
