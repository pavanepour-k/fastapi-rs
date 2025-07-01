use bytes::Bytes;
use chrono::{DateTime, NaiveDateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{
    PyAny, PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyNone, PyString, PyTuple,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Unsupported type for JSON encoding: {0}")]
    UnsupportedType(String),
    #[error("Serialization failed: {0}")]
    SerializationError(String),
    #[error("Invalid datetime format: {0}")]
    InvalidDatetime(String),
    #[error("Circular reference detected")]
    CircularReference,
}

pub type Result<T> = std::result::Result<T, EncodingError>;

pub fn jsonable_encoder(obj: &Bound<PyAny>) -> Result<String> {
    let value = python_to_json_value(obj, &mut std::collections::HashSet::new())?;
    serde_json::to_string(&value).map_err(|e| EncodingError::SerializationError(e.to_string()))
}

pub fn serialize_response(data: &Bound<PyAny>, content_type: Option<&str>) -> Result<Vec<u8>> {
    match content_type {
        Some("application/json") | None => {
            let json_str = jsonable_encoder(data)?;
            Ok(json_str.into_bytes())
        }
        Some("text/plain") => {
            let text = data
                .str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?
                .to_str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
            Ok(text.as_bytes().to_vec())
        }
        Some("application/octet-stream") => {
            if let Ok(bytes) = data.downcast::<PyBytes>() {
                Ok(bytes.as_bytes().to_vec())
            } else {
                Err(EncodingError::UnsupportedType(
                    "Expected bytes for octet-stream".to_string(),
                ))
            }
        }
        Some(ct) => Err(EncodingError::UnsupportedType(format!(
            "Unsupported content type: {}",
            ct
        ))),
    }
}

fn python_to_json_value(
    obj: &Bound<PyAny>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    // Prevent infinite recursion
    let obj_id = obj.as_ptr() as usize;
    if visited.contains(&obj_id) {
        return Err(EncodingError::CircularReference);
    }

    // Handle None
    if obj.is_none() {
        return Ok(Value::Null);
    }

    // Handle basic types first for performance
    if let Ok(s) = obj.downcast::<PyString>() {
        let text = s
            .to_str()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        return Ok(Value::String(text.to_string()));
    }

    if let Ok(i) = obj.downcast::<PyInt>() {
        let num = i
            .extract::<i64>()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        return Ok(Value::Number(num.into()));
    }

    if let Ok(f) = obj.downcast::<PyFloat>() {
        let num = f
            .extract::<f64>()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        let json_num = serde_json::Number::from_f64(num)
            .ok_or_else(|| EncodingError::SerializationError("Invalid float value".to_string()))?;
        return Ok(Value::Number(json_num));
    }

    if let Ok(b) = obj.downcast::<PyBool>() {
        let val = b
            .extract::<bool>()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        return Ok(Value::Bool(val));
    }

    visited.insert(obj_id);

    let result = if let Ok(dict) = obj.downcast::<PyDict>() {
        encode_dict(dict, visited)
    } else if let Ok(list) = obj.downcast::<PyList>() {
        encode_list(list, visited)
    } else if let Ok(tuple) = obj.downcast::<PyTuple>() {
        encode_tuple(tuple, visited)
    } else if let Ok(bytes) = obj.downcast::<PyBytes>() {
        encode_bytes(bytes)
    } else if is_datetime(obj) {
        encode_datetime(obj)
    } else if has_dict_method(obj) {
        encode_object_with_dict(obj, visited)
    } else if is_enum(obj) {
        encode_enum(obj)
    } else if is_pydantic_model(obj) {
        encode_pydantic_model(obj, visited)
    } else {
        // Fallback to string representation
        let str_repr = obj
            .str()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?
            .to_str()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        Ok(Value::String(str_repr.to_string()))
    };

    visited.remove(&obj_id);
    result
}

fn encode_dict(
    dict: &Bound<PyDict>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    let mut map = Map::new();

    for (key, value) in dict.iter() {
        let key_str = if let Ok(s) = key.downcast::<PyString>() {
            s.to_str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?
                .to_string()
        } else {
            key.str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?
                .to_str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?
                .to_string()
        };

        let json_value = python_to_json_value(&value, visited)?;
        map.insert(key_str, json_value);
    }

    Ok(Value::Object(map))
}

fn encode_list(
    list: &Bound<PyList>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    let mut vec = Vec::with_capacity(list.len());

    for item in list.iter() {
        let json_value = python_to_json_value(&item, visited)?;
        vec.push(json_value);
    }

    Ok(Value::Array(vec))
}

fn encode_tuple(
    tuple: &Bound<PyTuple>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    let mut vec = Vec::with_capacity(tuple.len());

    for item in tuple.iter() {
        let json_value = python_to_json_value(&item, visited)?;
        vec.push(json_value);
    }

    Ok(Value::Array(vec))
}

fn encode_bytes(bytes: &Bound<PyBytes>) -> Result<Value> {
    let b64 = base64::encode(bytes.as_bytes());
    Ok(Value::String(b64))
}

fn encode_datetime(obj: &Bound<PyAny>) -> Result<Value> {
    Python::with_gil(|py| {
        // Try to get ISO format string
        if let Ok(isoformat) = obj.call_method0("isoformat") {
            let iso_str = isoformat
                .str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?
                .to_str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
            return Ok(Value::String(iso_str.to_string()));
        }

        // Fallback to string representation
        let str_repr = obj
            .str()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?
            .to_str()
            .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
        Ok(Value::String(str_repr.to_string()))
    })
}

fn encode_object_with_dict(
    obj: &Bound<PyAny>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    if let Ok(dict) = obj.getattr("__dict__") {
        if let Ok(py_dict) = dict.downcast::<PyDict>() {
            return encode_dict(py_dict, visited);
        }
    }

    // Fallback to string representation
    let str_repr = obj
        .str()
        .map_err(|e| EncodingError::SerializationError(e.to_string()))?
        .to_str()
        .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
    Ok(Value::String(str_repr.to_string()))
}

fn encode_enum(obj: &Bound<PyAny>) -> Result<Value> {
    if let Ok(value) = obj.getattr("value") {
        return python_to_json_value(&value, &mut std::collections::HashSet::new());
    }

    // Fallback to name
    if let Ok(name) = obj.getattr("name") {
        if let Ok(name_str) = name.downcast::<PyString>() {
            let text = name_str
                .to_str()
                .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
            return Ok(Value::String(text.to_string()));
        }
    }

    // Final fallback to string representation
    let str_repr = obj
        .str()
        .map_err(|e| EncodingError::SerializationError(e.to_string()))?
        .to_str()
        .map_err(|e| EncodingError::SerializationError(e.to_string()))?;
    Ok(Value::String(str_repr.to_string()))
}

fn encode_pydantic_model(
    obj: &Bound<PyAny>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Value> {
    // Try model_dump() first (Pydantic v2)
    if let Ok(dump_method) = obj.getattr("model_dump") {
        if let Ok(result) = dump_method.call0() {
            if let Ok(dict) = result.downcast::<PyDict>() {
                return encode_dict(dict, visited);
            }
        }
    }

    // Try dict() method (Pydantic v1)
    if let Ok(dict_method) = obj.getattr("dict") {
        if let Ok(result) = dict_method.call0() {
            if let Ok(dict) = result.downcast::<PyDict>() {
                return encode_dict(dict, visited);
            }
        }
    }

    // Fallback to __dict__
    encode_object_with_dict(obj, visited)
}

fn is_datetime(obj: &Bound<PyAny>) -> bool {
    Python::with_gil(|py| {
        if let Ok(datetime_module) = py.import("datetime") {
            if let Ok(datetime_class) = datetime_module.getattr("datetime") {
                return obj.is_instance(&datetime_class).unwrap_or(false);
            }
        }
        false
    })
}

fn has_dict_method(obj: &Bound<PyAny>) -> bool {
    obj.hasattr("__dict__").unwrap_or(false)
}

fn is_enum(obj: &Bound<PyAny>) -> bool {
    Python::with_gil(|py| {
        if let Ok(enum_module) = py.import("enum") {
            if let Ok(enum_class) = enum_module.getattr("Enum") {
                return obj.is_instance(&enum_class).unwrap_or(false);
            }
        }
        false
    })
}

fn is_pydantic_model(obj: &Bound<PyAny>) -> bool {
    obj.hasattr("model_dump").unwrap_or(false) || obj.hasattr("dict").unwrap_or(false)
}

// Fast path encoders for common types
pub fn encode_string_fast(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
}

pub fn encode_number_fast(n: f64) -> String {
    if n.is_finite() {
        n.to_string()
    } else {
        "null".to_string()
    }
}

pub fn encode_bool_fast(b: bool) -> String {
    if b { "true" } else { "false" }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyDict;

    #[test]
    fn test_encode_basic_types() {
        Python::with_gil(|py| {
            // String
            let py_str = PyString::new_bound(py, "hello");
            let result =
                python_to_json_value(&py_str.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();
            assert_eq!(result, Value::String("hello".to_string()));

            // Integer
            let py_int = PyInt::new_bound(py, 42);
            let result =
                python_to_json_value(&py_int.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();
            assert_eq!(result, Value::Number(42.into()));

            // Boolean
            let py_bool = PyBool::new_bound(py, true);
            let result =
                python_to_json_value(&py_bool.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();
            assert_eq!(result, Value::Bool(true));

            // None
            let py_none = PyNone::get_bound(py);
            let result =
                python_to_json_value(&py_none.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();
            assert_eq!(result, Value::Null);
        });
    }

    #[test]
    fn test_encode_dict() {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("name", "John").unwrap();
            dict.set_item("age", 30).unwrap();

            let result =
                python_to_json_value(&dict.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();

            if let Value::Object(map) = result {
                assert_eq!(map.get("name"), Some(&Value::String("John".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Number(30.into())));
            } else {
                panic!("Expected object");
            }
        });
    }

    #[test]
    fn test_encode_list() {
        Python::with_gil(|py| {
            let list = PyList::new_bound(py, &[1, 2, 3]);

            let result =
                python_to_json_value(&list.as_any(), &mut std::collections::HashSet::new())
                    .unwrap();

            if let Value::Array(arr) = result {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Number(1.into()));
                assert_eq!(arr[1], Value::Number(2.into()));
                assert_eq!(arr[2], Value::Number(3.into()));
            } else {
                panic!("Expected array");
            }
        });
    }
}
