//! Query parameter extraction and validation for FastAPI-RS.
//!
//! Provides functions to process and validate query parameters according to
//! schema definitions, leveraging core validation logic.

use crate::params::validation::{
    validate_query_params, ParameterSchema, ValidationError, ValidationResult,
};
use serde_json::Value;
use std::collections::HashMap;

/// Validate query parameters against a schema.
///
/// # Arguments
/// * `params` - Key-value pairs from the query string.
/// * `schema` - JSON schema for each parameter.
///
/// # Returns
/// `ValidationResult` with validated data or error details.
pub fn process_query_params(
    params: HashMap<String, String>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult, ValidationError> {
    validate_query_params(params, schema)
}
