use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use std::io::{Read, Write};
use flate2::{Compression, write::GzEncoder, read::GzDecoder};
use zstd::stream::{Encoder as ZstdEncoder, Decoder as ZstdDecoder};

/// Compress data with GZIP at specified level (0-9).
#[pyfunction]
pub fn gzip_compress(data: &[u8], level: u32) -> PyResult<Vec<u8>> {
    if level > 9 {
        return Err(PyValueError::new_err("Compression level must be 0-9"));
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    encoder.finish()
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Decompress GZIP-compressed data.
#[pyfunction]
pub fn gzip_decompress(data: &[u8]) -> PyResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(buf)
}

/// Compress data using Zstandard at specified compression level (1-22).
#[pyfunction]
pub fn zstd_compress(data: &[u8], level: i32) -> PyResult<Vec<u8>> {
    if level < 1 || level > 22 {
        return Err(PyValueError::new_err("Zstd level must be between 1 and 22"));
    }
    let mut encoder = ZstdEncoder::new(Vec::new(), level)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    encoder.write_all(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    encoder.finish()
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Decompress Zstandard-compressed data.
#[pyfunction]
pub fn zstd_decompress(data: &[u8]) -> PyResult<Vec<u8>> {
    let mut decoder = ZstdDecoder::new(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(buf)
}

#[pymodule]
pub fn compress(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gzip_compress, m)?)?;
    m.add_function(wrap_pyfunction!(gzip_decompress, m)?)?;
    m.add_function(wrap_pyfunction!(zstd_compress, m)?)?;
    m.add_function(wrap_pyfunction!(zstd_decompress, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_roundtrip() {
        let data = b"Hello, FastAPI Rust!";
        let compressed = gzip_compress(data, 6).unwrap();
        let decompressed = gzip_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_zstd_roundtrip() {
        let data = b"Another test string for Zstd.";
        let compressed = zstd_compress(data, 3).unwrap();
        let decompressed = zstd_decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    #[should_panic]
    fn test_invalid_gzip_level() {
        gzip_compress(b"x", 10).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_zstd_level() {
        zstd_compress(b"x", 0).unwrap();
    }
}
