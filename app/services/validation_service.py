from typing import List
import rustlib

def validate_email(email: str) -> bool:
    """
    Validate an email address using a compiled regex in Rust.
    """
    return rust_mod.validate_email(email)


def validate_phone(number: str) -> bool:
    """
    Validate a phone number (digits, spaces, hyphens, optional leading '+') using Rust.
    """
    return rust_mod.validate_phone(number)


def validate_url(url: str) -> bool:
    """
    Validate a URL (http/https, domain, optional path) using Rust.
    """
    return rust_mod.validate_url(url)


def find_pattern_matches(pattern: str, text: str) -> List[str]:
    """
    Find all non-overlapping matches of a regex pattern in the given text using Rust.
    """
    return rust_mod.find_pattern_matches(pattern, text)
