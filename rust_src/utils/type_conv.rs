use pyo3::prelude::*;
use pyo3::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeConversionError {
    #[error("Unsupported Python type: {0}")]
    UnsupportedType(String),
    #[error("Type conversion failed: {0}")]
    ConversionFailed(String),
    #[error("Invalid type annotation: {0}")]
    InvalidAnnotation(String),
}

pub type Result<T> = std::result::Result<T, TypeConversionError>;

/// Convert Python object to its type name string
pub fn convert_python_type(py_obj: &Bound<PyAny>) -> Result<String> {
    let type_name = py_obj.get_type().name()
        .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
    
    Ok(match type_name {
        "str" => "string".to_string(),
        "int" => "integer".to_string(),
        "float" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "list" => "array".to_string(),
        "dict" => "object".to_string(),
        "tuple" => "array".to_string(),
        "NoneType" => "null".to_string(),
        "bytes" => "binary".to_string(),
        "datetime" => "string".to_string(), // ISO format
        "date" => "string".to_string(),     // ISO format
        "time" => "string".to_string(),     // ISO format
        "Decimal" => "number".to_string(),
        "UUID" => "string".to_string(),
        _ => {
            // Check for common patterns
            if is_enum_type(py_obj) {
                "string".to_string()
            } else if is_pydantic_model(py_obj) {
                "object".to_string()
            } else if is_dataclass(py_obj) {
                "object".to_string()
            } else {
                format!("unknown:{}", type_name)
            }
        }
    })
}

/// Get JSON Schema type from Python type annotation
pub fn python_type_to_json_schema_type(py_type: &Bound<PyAny>) -> Result<String> {
    let type_name = py_type.get_type().name()
        .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
    
    Ok(match type_name {
        "type" => {
            // It's a type object, get its name
            let name = py_type.getattr("__name__")
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
                .str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
                .to_str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            
            match name {
                "str" => "string",
                "int" => "integer",
                "float" => "number",
                "bool" => "boolean",
                "list" => "array",
                "dict" => "object",
                "bytes" => "string", // Base64 encoded
                _ => "string", // Default fallback
            }.to_string()
        }
        _ => convert_python_type(py_type)?
    })
}

/// Extract type information from typing annotations
pub fn extract_typing_info(annotation: &Bound<PyAny>) -> Result<TypeInfo> {
    Python::with_gil(|py| {
        // Check if it's from typing module
        if let Ok(origin) = annotation.getattr("__origin__") {
            let origin_name = origin.get_type().name()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            
            match origin_name {
                "list" | "List" => {
                    let args = get_type_args(annotation)?;
                    let item_type = if args.is_empty() {
                        "any".to_string()
                    } else {
                        python_type_to_json_schema_type(&args[0])?
                    };
                    Ok(TypeInfo {
                        base_type: "array".to_string(),
                        item_type: Some(item_type),
                        nullable: false,
                        optional: false,
                    })
                }
                "dict" | "Dict" => {
                    Ok(TypeInfo {
                        base_type: "object".to_string(),
                        item_type: None,
                        nullable: false,
                        optional: false,
                    })
                }
                "union" | "Union" => {
                    let args = get_type_args(annotation)?;
                    let (main_type, nullable) = extract_union_info(&args)?;
                    Ok(TypeInfo {
                        base_type: main_type,
                        item_type: None,
                        nullable,
                        optional: false,
                    })
                }
                _ => Ok(TypeInfo {
                    base_type: python_type_to_json_schema_type(annotation)?,
                    item_type: None,
                    nullable: false,
                    optional: false,
                })
            }
        } else {
            // Direct type
            Ok(TypeInfo {
                base_type: python_type_to_json_schema_type(annotation)?,
                item_type: None,
                nullable: false,
                optional: false,
            })
        }
    })
}

/// Type information extracted from Python annotations
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub base_type: String,
    pub item_type: Option<String>,
    pub nullable: bool,
    pub optional: bool,
}

fn get_type_args(annotation: &Bound<PyAny>) -> Result<Vec<Bound<PyAny>>> {
    if let Ok(args) = annotation.getattr("__args__") {
        if let Ok(tuple) = args.downcast::<PyTuple>() {
            return Ok(tuple.iter().collect());
        }
    }
    Ok(Vec::new())
}

fn extract_union_info(args: &[Bound<PyAny>]) -> Result<(String, bool)> {
    let mut types = Vec::new();
    let mut nullable = false;
    
    for arg in args {
        let type_name = python_type_to_json_schema_type(arg)?;
        if type_name == "null" {
            nullable = true;
        } else {
            types.push(type_name);
        }
    }
    
    let main_type = if types.len() == 1 {
        types.into_iter().next().unwrap()
    } else if types.is_empty() {
        "null".to_string()
    } else {
        // Multiple non-null types, use the first one as primary
        types.into_iter().next().unwrap()
    };
    
    Ok((main_type, nullable))
}

fn is_enum_type(py_obj: &Bound<PyAny>) -> bool {
    Python::with_gil(|py| {
        if let Ok(enum_module) = py.import("enum") {
            if let Ok(enum_class) = enum_module.getattr("Enum") {
                return py_obj.is_instance(&enum_class).unwrap_or(false);
            }
        }
        false
    })
}

fn is_pydantic_model(py_obj: &Bound<PyAny>) -> bool {
    py_obj.hasattr("model_dump").unwrap_or(false) || 
    py_obj.hasattr("dict").unwrap_or(false)
}

fn is_dataclass(py_obj: &Bound<PyAny>) -> bool {
    Python::with_gil(|py| {
        if let Ok(dataclasses) = py.import("dataclasses") {
            if let Ok(is_dataclass_fn) = dataclasses.getattr("is_dataclass") {
                return is_dataclass_fn.call1((py_obj,))
                    .and_then(|result| result.extract::<bool>())
                    .unwrap_or(false);
            }
        }
        false
    })
}

/// Convert Python value to Rust native type
pub fn python_to_rust_value(py_obj: &Bound<PyAny>) -> Result<RustValue> {
    if py_obj.is_none() {
        return Ok(RustValue::None);
    }
    
    if let Ok(s) = py_obj.downcast::<PyString>() {
        let text = s.to_str()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
        return Ok(RustValue::String(text.to_string()));
    }
    
    if let Ok(i) = py_obj.downcast::<PyInt>() {
        let num = i.extract::<i64>()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
        return Ok(RustValue::Integer(num));
    }
    
    if let Ok(f) = py_obj.downcast::<PyFloat>() {
        let num = f.extract::<f64>()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
        return Ok(RustValue::Float(num));
    }
    
    if let Ok(b) = py_obj.downcast::<PyBool>() {
        let val = b.extract::<bool>()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
        return Ok(RustValue::Boolean(val));
    }
    
    if let Ok(list) = py_obj.downcast::<PyList>() {
        let mut items = Vec::new();
        for item in list.iter() {
            items.push(python_to_rust_value(&item)?);
        }
        return Ok(RustValue::Array(items));
    }
    
    if let Ok(dict) = py_obj.downcast::<PyDict>() {
        let mut map = std::collections::HashMap::new();
        for (key, value) in dict.iter() {
            let key_str = key.str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
                .to_str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
                .to_string();
            map.insert(key_str, python_to_rust_value(&value)?);
        }
        return Ok(RustValue::Object(map));
    }
    
    // Fallback to string representation
    let str_repr = py_obj.str()
        .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
        .to_str()
        .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
        .to_string();
    Ok(RustValue::String(str_repr))
}

/// Rust representation of Python values
#[derive(Debug, Clone)]
pub enum RustValue {
    None,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<RustValue>),
    Object(std::collections::HashMap<String, RustValue>),
}

impl RustValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RustValue::None => "null",
            RustValue::String(_) => "string",
            RustValue::Integer(_) => "integer",
            RustValue::Float(_) => "number",
            RustValue::Boolean(_) => "boolean",
            RustValue::Array(_) => "array",
            RustValue::Object(_) => "object",
        }
    }
    
    pub fn is_null(&self) -> bool {
        matches!(self, RustValue::None)
    }
    
    pub fn as_string(&self) -> Option<&String> {
        match self {
            RustValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            RustValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    pub fn as_float(&self) -> Option<f64> {
        match self {
            RustValue::Float(f) => Some(*f),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            RustValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Convert RustValue back to Python object
    pub fn to_python(&self, py: Python) -> PyResult<Py<PyAny>> {
        match self {
            RustValue::None => Ok(PyNone::get_bound(py).into_py(py)),
            RustValue::String(s) => Ok(PyString::new_bound(py, s).into_py(py)),
            RustValue::Integer(i) => Ok(PyInt::new_bound(py, *i).into_py(py)),
            RustValue::Float(f) => Ok(PyFloat::new_bound(py, *f).into_py(py)),
            RustValue::Boolean(b) => Ok(PyBool::new_bound(py, *b).into_py(py)),
            RustValue::Array(arr) => {
                let py_list = PyList::empty_bound(py);
                for item in arr {
                    py_list.append(item.to_python(py)?)?;
                }
                Ok(py_list.into_py(py))
            }
            RustValue::Object(obj) => {
                let py_dict = PyDict::new_bound(py);
                for (key, value) in obj {
                    py_dict.set_item(key, value.to_python(py)?)?;
                }
                Ok(py_dict.into_py(py))
            }
        }
    }
}

/// Schema conversion utilities for FastAPI compatibility
pub struct SchemaConverter;

impl SchemaConverter {
    /// Convert Python type annotations to OpenAPI schema
    pub fn python_type_to_openapi_schema(py_type: &Bound<PyAny>) -> Result<serde_json::Value> {
        let type_info = extract_typing_info(py_type)?;
        
        let mut schema = serde_json::json!({
            "type": type_info.base_type
        });
        
        if let Some(item_type) = type_info.item_type {
            schema["items"] = serde_json::json!({
                "type": item_type
            });
        }
        
        if type_info.nullable {
            schema["nullable"] = serde_json::Value::Bool(true);
        }
        
        Ok(schema)
    }
    
    /// Convert Pydantic model to OpenAPI schema
    pub fn pydantic_model_to_schema(model: &Bound<PyAny>) -> Result<serde_json::Value> {
        Python::with_gil(|py| {
            // Try to get the schema from Pydantic model
            if let Ok(schema_method) = model.getattr("model_json_schema") {
                if let Ok(schema_result) = schema_method.call0() {
                    if let Ok(schema_dict) = schema_result.downcast::<PyDict>() {
                        return Self::py_dict_to_json_value(schema_dict);
                    }
                }
            }
            
            // Fallback to basic object schema
            Ok(serde_json::json!({
                "type": "object",
                "properties": {}
            }))
        })
    }
    
    fn py_dict_to_json_value(py_dict: &Bound<PyDict>) -> Result<serde_json::Value> {
        let mut json_obj = serde_json::Map::new();
        
        for (key, value) in py_dict.iter() {
            let key_str = key.str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
                .to_str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            
            let json_value = Self::py_any_to_json_value(&value)?;
            json_obj.insert(key_str.to_string(), json_value);
        }
        
        Ok(serde_json::Value::Object(json_obj))
    }
    
    fn py_any_to_json_value(py_any: &Bound<PyAny>) -> Result<serde_json::Value> {
        if py_any.is_none() {
            return Ok(serde_json::Value::Null);
        }
        
        if let Ok(s) = py_any.downcast::<PyString>() {
            let text = s.to_str()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            return Ok(serde_json::Value::String(text.to_string()));
        }
        
        if let Ok(i) = py_any.downcast::<PyInt>() {
            let num = i.extract::<i64>()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            return Ok(serde_json::Value::Number(num.into()));
        }
        
        if let Ok(f) = py_any.downcast::<PyFloat>() {
            let num = f.extract::<f64>()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            let json_num = serde_json::Number::from_f64(num)
                .ok_or_else(|| TypeConversionError::ConversionFailed("Invalid float".to_string()))?;
            return Ok(serde_json::Value::Number(json_num));
        }
        
        if let Ok(b) = py_any.downcast::<PyBool>() {
            let val = b.extract::<bool>()
                .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
            return Ok(serde_json::Value::Bool(val));
        }
        
        if let Ok(list) = py_any.downcast::<PyList>() {
            let mut json_array = Vec::new();
            for item in list.iter() {
                json_array.push(Self::py_any_to_json_value(&item)?);
            }
            return Ok(serde_json::Value::Array(json_array));
        }
        
        if let Ok(dict) = py_any.downcast::<PyDict>() {
            return Self::py_dict_to_json_value(dict);
        }
        
        // Fallback to string representation
        let str_repr = py_any.str()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?
            .to_str()
            .map_err(|e| TypeConversionError::ConversionFailed(e.to_string()))?;
        Ok(serde_json::Value::String(str_repr.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_basic_python_types() {
        Python::with_gil(|py| {
            let py_str = PyString::new_bound(py, "hello");
            assert_eq!(convert_python_type(&py_str.as_any()).unwrap(), "string");
            
            let py_int = PyInt::new_bound(py, 42);
            assert_eq!(convert_python_type(&py_int.as_any()).unwrap(), "integer");
            
            let py_float = PyFloat::new_bound(py, 3.14);
            assert_eq!(convert_python_type(&py_float.as_any()).unwrap(), "number");
            
            let py_bool = PyBool::new_bound(py, true);
            assert_eq!(convert_python_type(&py_bool.as_any()).unwrap(), "boolean");
            
            let py_none = PyNone::get_bound(py);
            assert_eq!(convert_python_type(&py_none.as_any()).unwrap(), "null");
        });
    }
    
    #[test]
    fn test_python_to_rust_value() {
        Python::with_gil(|py| {
            let py_str = PyString::new_bound(py, "hello");
            let rust_val = python_to_rust_value(&py_str.as_any()).unwrap();
            assert_eq!(rust_val.as_string(), Some(&"hello".to_string()));
            
            let py_int = PyInt::new_bound(py, 42);
            let rust_val = python_to_rust_value(&py_int.as_any()).unwrap();
            assert_eq!(rust_val.as_integer(), Some(42));
            
            let py_none = PyNone::get_bound(py);
            let rust_val = python_to_rust_value(&py_none.as_any()).unwrap();
            assert!(rust_val.is_null());
        });
    }
    
    #[test]
    fn test_rust_value_type_names() {
        assert_eq!(RustValue::None.type_name(), "null");
        assert_eq!(RustValue::String("test".to_string()).type_name(), "string");
        assert_eq!(RustValue::Integer(42).type_name(), "integer");
        assert_eq!(RustValue::Float(3.14).type_name(), "number");
        assert_eq!(RustValue::Boolean(true).type_name(), "boolean");
        assert_eq!(RustValue::Array(vec![]).type_name(), "array");
        assert_eq!(RustValue::Object(std::collections::HashMap::new()).type_name(), "object");
    }
    
    #[test]
    fn test_rust_value_to_python_conversion() {
        Python::with_gil(|py| {
            let rust_val = RustValue::String("hello".to_string());
            let py_val = rust_val.to_python(py).unwrap();
            let py_str = py_val.downcast_bound::<PyString>(py).unwrap();
            assert_eq!(py_str.to_str().unwrap(), "hello");
            
            let rust_val = RustValue::Integer(42);
            let py_val = rust_val.to_python(py).unwrap();
            let py_int = py_val.downcast_bound::<PyInt>(py).unwrap();
            assert_eq!(py_int.extract::<i64>().unwrap(), 42);
        });
    }
    
    #[test]
    fn test_schema_converter() {
        Python::with_gil(|py| {
            let py_str_type = py.get_type::<PyString>();
            let schema = SchemaConverter::python_type_to_openapi_schema(&py_str_type.as_any()).unwrap();
            
            assert_eq!(schema["type"], "string");
        });
    }
}