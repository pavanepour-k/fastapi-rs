import hmac
import secrets
from typing import Optional

from .. import _rust

__all__ = [
    "constant_time_compare",
    "get_authorization_scheme_param",
    "verify_api_key",
    "hash_password",
    "generate_token",
]


def constant_time_compare(a: str, b: str) -> bool:
    """
    Constant-time string comparison to prevent timing attacks.
    Uses Rust implementation when available for better performance.
    """
    if _rust.rust_available():
        try:
            return _rust.RustSecurity.constant_time_compare(a, b)
        except Exception:
            pass
    
    # Python fallback using hmac.compare_digest
    return hmac.compare_digest(a.encode("utf-8"), b.encode("utf-8"))


def get_authorization_scheme_param(authorization_header_value: Optional[str]) -> tuple[str, str]:
    """
    Get the authorization scheme and parameter from the Authorization header.
    
    Args:
        authorization_header_value: The value of the Authorization header.
        
    Returns:
        A tuple of (scheme, param) or ("", "") if invalid.
    """
    if not authorization_header_value:
        return "", ""
    
    parts = authorization_header_value.split(" ", 1)
    if len(parts) == 1:
        return "", ""
    
    return parts[0], parts[1]


def verify_api_key(
    provided_key: str,
    expected_key: str,
    algorithm: Optional[str] = None
) -> bool:
    """
    Verify an API key using constant-time comparison.
    Uses Rust implementation when available.
    
    Args:
        provided_key: The API key provided by the client.
        expected_key: The expected API key.
        algorithm: Optional hashing algorithm (e.g., "sha256", "bcrypt").
        
    Returns:
        True if the keys match, False otherwise.
    """
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust", "verify_api_key"
            )(provided_key, expected_key, algorithm)
        except Exception:
            pass
    
    # Python fallback
    if algorithm is None or algorithm == "plain":
        return constant_time_compare(provided_key, expected_key)
    else:
        # For now, just use plain comparison
        # In production, you'd implement proper hashing
        return constant_time_compare(provided_key, expected_key)


def hash_password(password: str, algorithm: Optional[str] = None) -> str:
    """
    Hash a password using the specified algorithm.
    Uses Rust implementation when available.
    
    Args:
        password: The password to hash.
        algorithm: The hashing algorithm (e.g., "sha256", "bcrypt").
        
    Returns:
        The hashed password.
    """
    if _rust.rust_available():
        try:
            return _rust.RustSecurity.hash_password(password, algorithm)
        except Exception:
            pass
    
    # Python fallback - simple example, use proper hashing in production
    if algorithm is None or algorithm == "sha256":
        import hashlib
        return hashlib.sha256(password.encode()).hexdigest()
    else:
        # For now, just return a simple hash
        import hashlib
        return f"{algorithm}:{hashlib.sha256(password.encode()).hexdigest()}"


def generate_token(length: int = 32) -> str:
    """
    Generate a cryptographically secure random token.
    
    Args:
        length: The length of the token in bytes (default: 32).
        
    Returns:
        A hex-encoded random token.
    """
    return secrets.token_hex(length)


def generate_session_token() -> str:
    """
    Generate a session token.
    Uses Rust implementation when available.
    
    Returns:
        A secure session token.
    """
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust", "generate_session_token"
            )()
        except Exception:
            pass
    
    # Python fallback
    return secrets.token_urlsafe(24)


def validate_password_strength(password: str) -> tuple[bool, list[str]]:
    """
    Validate password strength.
    Uses Rust implementation when available.
    
    Args:
        password: The password to validate.
        
    Returns:
        A tuple of (is_valid, error_messages).
    """
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust", "validate_password_strength"
            )(password)
        except Exception:
            pass
    
    # Python fallback
    errors = []
    
    if len(password) < 8:
        errors.append("Password must be at least 8 characters long")
    
    if len(password) > 128:
        errors.append("Password must be less than 128 characters long")
    
    if not any(c.islower() for c in password):
        errors.append("Password must contain at least one lowercase letter")
    
    if not any(c.isupper() for c in password):
        errors.append("Password must contain at least one uppercase letter")
    
    if not any(c.isdigit() for c in password):
        errors.append("Password must contain at least one digit")
    
    special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?"
    if not any(c in special_chars for c in password):
        errors.append("Password must contain at least one special character")
    
    return (len(errors) == 0, errors)