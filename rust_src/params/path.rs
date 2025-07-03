//! Path parameter extraction and validation for FastAPI-RS.
//!
//! Provides functions to process and validate path parameters according to
//! schema definitions, leveraging core validation logic.

use crate::params::validation::{
    validate_path_params, ParameterSchema, ValidationError, ValidationResult,
};
use serde_json::Value;
use std::collections::HashMap;

/// Validate path parameters against a schema.
///
/// # Arguments
/// * `params` - Key-value pairs from the HTTP path.
/// * `schema` - JSON schema for each parameter.
///
/// # Returns
/// `ValidationResult` with validated data or error details.
pub fn process_path_params(
    params: HashMap<String, String>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult, ValidationError> {
    validate_path_params(params, schema)
}
