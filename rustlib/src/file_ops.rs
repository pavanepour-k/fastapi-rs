use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::{fs, path::Path, io::{Read, BufReader}};
use sha2::{Sha256, Digest};
use crc32fast::Hasher;

/// Read the entire contents of a file into a string.
#[pyfunction]
pub fn read_file(path: &str) -> PyResult<String> {
    fs::read_to_string(path).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// Write a string to a file, creating or truncating it.
#[pyfunction]
pub fn write_file(path: &str, content: &str) -> PyResult<()> {
    fs::write(path, content).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// List names of entries in a directory.
#[pyfunction]
pub fn list_dir(path: &str) -> PyResult<Vec<String>> {
    let entries = fs::read_dir(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| PyIOError::new_err(e.to_string()))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

/// Check whether a path exists.
#[pyfunction]
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Remove a file or empty directory.
#[pyfunction]
pub fn remove(path: &str) -> PyResult<()> {
    let p = Path::new(path);
    if p.is_dir() {
        fs::remove_dir(path).map_err(|e| PyIOError::new_err(e.to_string()))
    } else {
        fs::remove_file(path).map_err(|e| PyIOError::new_err(e.to_string()))
    }
}

/// Rename or move a file or directory.
#[pyfunction]
pub fn rename(path: &str, new_path: &str) -> PyResult<()> {
    fs::rename(path, new_path).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// Create a directory and all parent components if they are missing.
#[pyfunction]
pub fn create_dir(path: &str) -> PyResult<()> {
    fs::create_dir_all(path).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// Get metadata: (size in bytes, is_file, is_dir).
#[pyfunction]
pub fn get_metadata(path: &str) -> PyResult<(u64, bool, bool)> {
    let meta = fs::metadata(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok((meta.len(), meta.is_file(), meta.is_dir()))
}

/// Calculate SHA-256 hash of a file, returning a hex string.
#[pyfunction]
pub fn calculate_file_hash(path: &str) -> PyResult<String> {
    let file = fs::File::open(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer).map_err(|e| PyIOError::new_err(e.to_string()))?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calculate CRC32 checksum of a file.
#[pyfunction]
pub fn calculate_checksum(path: &str) -> PyResult<u32> {
    let file = fs::File::open(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer).map_err(|e| PyIOError::new_err(e.to_string()))?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.finalize())
}

/// Verify file integrity by comparing SHA-256 hash to expected hex string.
#[pyfunction]
pub fn verify_file_integrity(path: &str, expected: &str) -> PyResult<bool> {
    let hash = calculate_file_hash(path)?;
    Ok(hash.eq_ignore_ascii_case(expected))
}

#[pymodule]
pub fn file_ops(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_file, m)?)?;
    m.add_function(wrap_pyfunction!(list_dir, m)?)?;
    m.add_function(wrap_pyfunction!(file_exists, m)?)?;
    m.add_function(wrap_pyfunction!(remove, m)?)?;
    m.add_function(wrap_pyfunction!(rename, m)?)?;
    m.add_function(wrap_pyfunction!(create_dir, m)?)?;
    m.add_function(wrap_pyfunction!(get_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_file_hash, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_checksum, m)?)?;
    m.add_function(wrap_pyfunction!(verify_file_integrity, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_file() {
        let path = "test.txt";
        let content = "Hello, world!";
        write_file(path, content).unwrap();
        let read = read_file(path).unwrap();
        assert_eq!(read, content);
        remove(path).unwrap();
    }

    #[test]
    fn test_list_dir_and_exists() {
        let dir = "test_dir";
        create_dir(dir).unwrap();
        assert!(file_exists(dir));
        let entries = list_dir(".").unwrap();
        assert!(entries.contains(&dir.to_string()));
        remove(dir).unwrap();
    }

    #[test]
    fn test_rename_and_metadata() {
        let src = "src.txt";
        let dst = "dst.txt";
        write_file(src, "data").unwrap();
        rename(src, dst).unwrap();
        let (size, is_file, is_dir) = get_metadata(dst).unwrap();
        assert_eq!(size, 4);
        assert!(is_file);
        assert!(!is_dir);
        remove(dst).unwrap();
    }

    #[test]
    fn test_file_hash_and_verify() {
        let path = "hash.txt";
        let data = "data";
        write_file(path, data).unwrap();
        let hash = calculate_file_hash(path).unwrap();
        assert_eq!(hash, "3a6eb0790f39ac87c94f3856b2dd2c5d110e6811602261a9a923d3bb23adc8b7");
        assert!(verify_file_integrity(path, &hash).unwrap());
        remove(path).unwrap();
    }

    #[test]
    fn test_checksum() {
        let path = "check.txt";
        let data = "data";
        write_file(path, data).unwrap();
        let checksum = calculate_checksum(path).unwrap();
        assert_eq!(checksum, 0xADF3F363);
        remove(path).unwrap();
    }
}
