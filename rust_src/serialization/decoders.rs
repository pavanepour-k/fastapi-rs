use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error("Unsupported content type: {0}")]
    UnsupportedContentType(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, DecodingError>;

/// Deserialize request body based on content type
pub fn deserialize_request(body: &[u8], content_type: &str) -> Result<Py<PyAny>> {
    Python::with_gil(|py| match content_type {
        "application/json" => deserialize_json(body, py),
        "application/x-www-form-urlencoded" => deserialize_form_data(body, py),
        "text/plain" => deserialize_text(body, py),
        _ => Err(DecodingError::UnsupportedContentType(
            content_type.to_string(),
        )),
    })
}

/// Deserialize JSON body to Python object
fn deserialize_json(body: &[u8], py: Python) -> Result<Py<PyAny>> {
    let body_str =
        std::str::from_utf8(body).map_err(|e| DecodingError::EncodingError(e.to_string()))?;

    let json_value: Value =
        serde_json::from_str(body_str).map_err(|e| DecodingError::InvalidJson(e.to_string()))?;

    json_to_python(&json_value, py)
}

/// Deserialize form data to Python dict
fn deserialize_form_data(body: &[u8], py: Python) -> Result<Py<PyAny>> {
    let body_str =
        std::str::from_utf8(body).map_err(|e| DecodingError::EncodingError(e.to_string()))?;

    let dict = PyDict::new_bound(py);

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = percent_decode(key);
            let decoded_value = percent_decode(value);

            // Handle multiple values for the same key
            if let Ok(existing) = dict.get_item(&decoded_key) {
                if let Some(existing_val) = existing {
                    // Convert to list if not already
                    if let Ok(list) = existing_val.downcast::<PyList>() {
                        list.append(decoded_value)?;
                    } else {
                        let list = PyList::new_bound(
                            py,
                            &[existing_val, PyString::new_bound(py, &decoded_value)],
                        );
                        dict.set_item(decoded_key, list)?;
                    }
                } else {
                    dict.set_item(decoded_key, decoded_value)?;
                }
            } else {
                dict.set_item(decoded_key, decoded_value)?;
            }
        } else if !pair.is_empty() {
            let decoded_key = percent_decode(pair);
            dict.set_item(decoded_key, "")?;
        }
    }

    Ok(dict.into_py(py))
}

/// Deserialize plain text
fn deserialize_text(body: &[u8], py: Python) -> Result<Py<PyAny>> {
    let text =
        std::str::from_utf8(body).map_err(|e| DecodingError::EncodingError(e.to_string()))?;

    Ok(PyString::new_bound(py, text).into_py(py))
}

/// Convert JSON value to Python object
fn json_to_python(value: &Value, py: Python) -> Result<Py<PyAny>> {
    match value {
        Value::Null => Ok(PyNone::get_bound(py).into_py(py)),
        Value::Bool(b) => Ok(PyBool::new_bound(py, *b).into_py(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(PyInt::new_bound(py, i).into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new_bound(py, f).into_py(py))
            } else {
                Err(DecodingError::ParseError("Invalid number".to_string()))
            }
        }
        Value::String(s) => Ok(PyString::new_bound(py, s).into_py(py)),
        Value::Array(arr) => {
            let py_list = PyList::empty_bound(py);
            for item in arr {
                let py_item = json_to_python(item, py)?;
                py_list.append(py_item)?;
            }
            Ok(py_list.into_py(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new_bound(py);
            for (key, val) in obj {
                let py_val = json_to_python(val, py)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_py(py))
        }
    }
}

/// URL percent decoding
fn percent_decode(input: &str) -> String {
    percent_encoding::percent_decode_str(input)
        .decode_utf8_lossy()
        .into_owned()
}

/// Fast JSON parsing for common cases
pub fn fast_parse_json_string(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str).map_err(|e| DecodingError::InvalidJson(e.to_string()))
}

/// Parse JSON with custom error handling
pub fn parse_json_with_context(body: &[u8], context: &str) -> Result<Value> {
    let body_str = std::str::from_utf8(body).map_err(|e| {
        DecodingError::EncodingError(format!("Invalid UTF-8 in {}: {}", context, e))
    })?;

    serde_json::from_str(body_str)
        .map_err(|e| DecodingError::InvalidJson(format!("JSON parse error in {}: {}", context, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_json() {
        Python::with_gil(|py| {
            let json_body = br#"{"name": "John", "age": 30, "active": true}"#;
            let result = deserialize_json(json_body, py).unwrap();

            let dict = result.downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(
                dict.get_item("name")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "John"
            );
            assert_eq!(
                dict.get_item("age")
                    .unwrap()
                    .unwrap()
                    .extract::<i64>()
                    .unwrap(),
                30
            );
            assert_eq!(
                dict.get_item("active")
                    .unwrap()
                    .unwrap()
                    .extract::<bool>()
                    .unwrap(),
                true
            );
        });
    }

    #[test]
    fn test_deserialize_form_data() {
        Python::with_gil(|py| {
            let form_body = b"name=John&age=30&active=true";
            let result = deserialize_form_data(form_body, py).unwrap();

            let dict = result.downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(
                dict.get_item("name")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "John"
            );
            assert_eq!(
                dict.get_item("age")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "30"
            );
        });
    }

    #[test]
    fn test_deserialize_form_data_multiple_values() {
        Python::with_gil(|py| {
            let form_body = b"tags=rust&tags=web&tags=api";
            let result = deserialize_form_data(form_body, py).unwrap();

            let dict = result.downcast_bound::<PyDict>(py).unwrap();
            let tags = dict.get_item("tags").unwrap().unwrap();
            let tags_list = tags.downcast::<PyList>().unwrap();

            assert_eq!(tags_list.len(), 3);
            assert_eq!(
                tags_list.get_item(0).unwrap().extract::<String>().unwrap(),
                "rust"
            );
            assert_eq!(
                tags_list.get_item(1).unwrap().extract::<String>().unwrap(),
                "web"
            );
            assert_eq!(
                tags_list.get_item(2).unwrap().extract::<String>().unwrap(),
                "api"
            );
        });
    }

    #[test]
    fn test_deserialize_text() {
        Python::with_gil(|py| {
            let text_body = b"Hello, World!";
            let result = deserialize_text(text_body, py).unwrap();

            let text = result.downcast_bound::<PyString>(py).unwrap();
            assert_eq!(text.to_str().unwrap(), "Hello, World!");
        });
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("Hello%20World"), "Hello World");
        assert_eq!(percent_decode("test%21%40%23"), "test!@#");
        assert_eq!(percent_decode("no_encoding"), "no_encoding");
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = b"invalid json";
        Python::with_gil(|py| {
            assert!(deserialize_json(invalid_json, py).is_err());
        });
    }
}
