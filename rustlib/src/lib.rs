use pyo3::prelude::*;
use pyo3::wrap_pymodule;

mod hashing;
mod regex_utils;
mod file_ops;
mod filter;
mod random_utils;
mod compress;
mod net_utils;
mod data_proc;

/// Main Python module initializer
#[pymodule]
pub fn rustlib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapping(wrap_pymodule!(hashing, m)?)?;
    m.add_wrapping(wrap_pymodule!(regex_utils, m)?)?;
    m.add_wrapping(wrap_pymodule!(file_ops, m)?)?;
    m.add_wrapping(wrap_pymodule!(filter, m)?)?;
    m.add_wrapping(wrap_pymodule!(random_utils, m)?)?;
    m.add_wrapping(wrap_pymodule!(compress, m)?)?;
    m.add_wrapping(wrap_pymodule!(net_utils, m)?)?;
    m.add_wrapping(wrap_pymodule!(data_proc, m)?)?;
    Ok(())
}
