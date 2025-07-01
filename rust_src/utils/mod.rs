pub mod async_tools;
pub mod content_type;
pub mod id_generation;
pub mod type_conv;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

pub use async_tools::*;
pub use content_type::*;
pub use id_generation::*;
pub use type_conv::*;

/// Convert Python dict to Rust HashMap
pub fn py_dict_to_hashmap(dict: &Bound<PyDict>) -> PyResult<HashMap<String, String>> {
    let mut map = HashMap::new();

    for (key, value) in dict.iter() {
        let key_str = key.str()?.to_str()?.to_string();
        let value_str = value.str()?.to_str()?.to_string();
        map.insert(key_str, value_str);
    }

    Ok(map)
}
