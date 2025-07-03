//! Request body extraction and validation for FastAPI-RS.
//!
//! Provides functions to process and validate JSON request bodies according to
//! schema definitions, leveraging core validation logic.

use crate::params::validation::{
    validate_body_params, ParameterSchema, ValidationError, ValidationResult,
};
use serde_json::Value;
use std::collections::HashMap;

/// Validate a JSON body against a schema.
///
/// # Arguments
/// * `body` - Raw request body bytes.
/// * `schema` - JSON schema for body fields.
///
/// # Returns
/// `ValidationResult` with validated data or error details.
pub fn process_body(
    body: Vec<u8>,
    schema: HashMap<String, Value>,
) -> Result<ValidationResult, ValidationError> {
    validate_body_params(body, schema)
}
