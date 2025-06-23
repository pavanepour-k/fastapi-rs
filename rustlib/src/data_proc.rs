use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use serde_json::Value;
use csv::{ReaderBuilder, WriterBuilder};
use regex::Regex;
use std::{collections::HashMap, fs::File, io::Write};

/// Parse a JSON string into a serde_json::Value.
#[pyfunction]
pub fn parse_json(text: &str) -> PyResult<Value> {
    serde_json::from_str(text).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Serialize a serde_json::Value into a pretty JSON string.
#[pyfunction]
pub fn to_json_string(value: &Value) -> PyResult<String> {
    serde_json::to_string_pretty(value).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Read a CSV file into a vector of hashmaps (column -> value).
#[pyfunction]
pub fn read_csv(path: &str, has_headers: bool) -> PyResult<Vec<HashMap<String, String>>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(has_headers)
        .from_path(path)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let headers = if has_headers {
        rdr.headers()
            .map_err(|e| PyValueError::new_err(e.to_string()))?
            .clone()
    } else {
        csv::StringRecord::new()
    };
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| PyValueError::new_err(e.to_string()))?;
        let mut map = HashMap::new();
        if has_headers {
            for (h, v) in headers.iter().zip(record.iter()) {
                map.insert(h.to_string(), v.to_string());
            }
        } else {
            for (i, v) in record.iter().enumerate() {
                map.insert(i.to_string(), v.to_string());
            }
        }
        records.push(map);
    }
    Ok(records)
}

/// Write a vector of hashmaps (column -> value) to a CSV file.
#[pyfunction]
pub fn write_csv(path: &str, data: Vec<HashMap<String, String>>, has_headers: bool) -> PyResult<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(has_headers)
        .from_path(path)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    if has_headers {
        if let Some(first) = data.get(0) {
            let headers: Vec<&String> = first.keys().collect();
            wtr.write_record(headers.iter().map(|s| s.as_str()))
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
        }
    }
    for row in data {
        let values: Vec<&String> = row.values().collect();
        wtr.write_record(values.iter().map(|s| s.as_str()))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
    }
    wtr.flush().map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(())
}

/// Filter rows where a given column matches a regex pattern.
#[pyfunction]
pub fn filter_rows(
    data: Vec<HashMap<String, String>>, column: &str, pattern: &str
) -> PyResult<Vec<HashMap<String, String>>> {
    let re = Regex::new(pattern).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let filtered = data.into_iter()
        .filter(|row| {
            row.get(column)
               .map(|v| re.is_match(v))
               .unwrap_or(false)
        })
        .collect();
    Ok(filtered)
}

#[pymodule]
pub fn data_proc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(to_json_string, m)?)?;
    m.add_function(wrap_pyfunction!(read_csv, m)?)?;
    m.add_function(wrap_pyfunction!(write_csv, m)?)?;
    m.add_function(wrap_pyfunction!(filter_rows, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_json_roundtrip() {
        let v = parse_json(r#"{\"key\":123}"#).unwrap();
        let s = to_json_string(&v).unwrap();
        assert!(s.contains("123"));
    }

    #[test]
    fn test_csv_read_write(tmp_path: &std::path::Path) {
        let file = tmp_path.join("test.csv");
        let mut row = HashMap::new();
        row.insert("a".to_string(), "1".to_string());
        write_csv(file.to_str().unwrap(), vec![row.clone()], true).unwrap();
        let data = read_csv(file.to_str().unwrap(), true).unwrap();
        assert_eq!(data[0].get("a"), Some(&"1".to_string()));
    }

    #[test]
    fn test_filter_rows() {
        let mut r1 = HashMap::new(); r1.insert("col".to_string(), "foo".to_string());
        let mut r2 = HashMap::new(); r2.insert("col".to_string(), "bar".to_string());
        let filtered = filter_rows(vec![r1, r2], "col", "^f").unwrap();
        assert_eq!(filtered.len(), 1);
    }
}
