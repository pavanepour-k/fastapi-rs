import rustlib as rust_mod
from typing import List


def validate_ip(address: str) -> bool:
    """
    Validate an IP address (IPv4 or IPv6) using Rust.
    """
    return rust_mod.validate_ip(address)


def resolve_hostname(hostname: str) -> List[str]:
    """
    Resolve a hostname to IP addresses (may include IPv4 and IPv6) using Rust.
    """
    return rust_mod.resolve_hostname(hostname)


def http_get(url: str) -> str:
    """
    Perform an HTTP GET request and return the response body as a string using Rust.
    """
    return rust_mod.http_get(url)


def http_post(url: str, body: str) -> str:
    """
    Perform an HTTP POST request with the given body and return the response body as a string using Rust.
    """
    return rust_mod.http_post(url, body)


def get_status_code(url: str) -> int:
    """
    Get the HTTP status code of a GET request to the given URL using Rust.
    """
    return rust_mod.get_status_code(url)
