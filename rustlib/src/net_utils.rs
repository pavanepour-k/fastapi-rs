use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::net::{IpAddr, ToSocketAddrs};

/// Validate an IP address (IPv4 or IPv6).
#[pyfunction]
pub fn validate_ip(address: &str) -> bool {
    address.parse::<IpAddr>().is_ok()
}

/// Resolve a hostname to IP addresses (may include IPv4 and IPv6).
#[pyfunction]
pub fn resolve_hostname(hostname: &str) -> PyResult<Vec<String>> {
    let addrs = (hostname, 0)
        .to_socket_addrs()
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    let ips: Vec<String> = addrs.map(|sa| sa.ip().to_string()).collect();
    Ok(ips)
}

/// Perform an HTTP GET request and return the response body as a string.
#[pyfunction]
pub fn http_get(url: &str) -> PyResult<String> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    let text = resp.text().map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(text)
}

/// Perform an HTTP POST request with given body and return the response body as a string.
#[pyfunction]
pub fn http_post(url: &str, body: &str) -> PyResult<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .body(body.to_string())
        .send()
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    let text = resp.text().map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(text)
}

/// Get the HTTP status code of a GET request to the given URL.
#[pyfunction]
pub fn get_status_code(url: &str) -> PyResult<u16> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(resp.status().as_u16())
}

#[pymodule]
pub fn net_utils(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_ip, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_hostname, m)?)?;
    m.add_function(wrap_pyfunction!(http_get, m)?)?;
    m.add_function(wrap_pyfunction!(http_post, m)?)?;
    m.add_function(wrap_pyfunction!(get_status_code, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ip() {
        assert!(validate_ip("127.0.0.1"));
        assert!(validate_ip("::1"));
        assert!(!validate_ip("999.999.999.999"));
    }

    #[test]
    fn test_resolve_hostname() {
        let ips = resolve_hostname("localhost").unwrap();
        assert!(ips.iter().any(|ip| ip == "127.0.0.1"));
    }

    #[test]
    #[ignore]
    fn test_http_get() {
        let body = http_get("https://httpbin.org/get").unwrap();
        assert!(body.contains("\"url\": \"https://httpbin.org/get\""));
    }

    #[test]
    #[ignore]
    fn test_http_post() {
        let body = http_post("https://httpbin.org/post", "data=1").unwrap();
        assert!(body.contains("\"data=1\""));
    }

    #[test]
    #[ignore]
    fn test_get_status_code() {
        let code = get_status_code("https://httpbin.org/status/418").unwrap();
        assert_eq!(code, 418);
    }
}
