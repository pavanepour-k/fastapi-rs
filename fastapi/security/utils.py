"""
FastAPI security utilities module with Rust acceleration.

Provides security utility functions with automatic fallback to pure Python
implementation when Rust extensions are unavailable.
"""
import hashlib
import hmac
import secrets
from typing import Optional, Union

from .. import _rust


def constant_time_compare(a: Union[str, bytes], b: Union[str, bytes]) -> bool:
    """
    Perform constant-time string comparison to prevent timing attacks.
    
    Uses Rust acceleration when available for improved security.
    """
    # Ensure both are the same type
    if isinstance(a, str):
        a = a.encode("utf-8")
    if isinstance(b, str):
        b = b.encode("utf-8")
    
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            return _rust.RustSecurity.constant_time_compare(
                a.decode("utf-8"), 
                b.decode("utf-8")
            )
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation using hmac.compare_digest
    return hmac.compare_digest(a, b)


def verify_password(plain_password: str, hashed_password: str) -> bool:
    """
    Verify a password against a hashed password.
    
    Uses constant-time comparison to prevent timing attacks.
    """
    # This is a simplified version - in production, you would use
    # a proper password hashing library like bcrypt or argon2
    return constant_time_compare(
        hash_password(plain_password), 
        hashed_password
    )


def hash_password(password: str, algorithm: Optional[str] = None) -> str:
    """
    Hash a password using the specified algorithm.
    
    Uses Rust acceleration when available for improved performance.
    """
    if algorithm is None:
        algorithm = "sha256"
    
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust",
                "hash_password",
                fallback=None
            )(password, algorithm)
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    if algorithm == "sha256":
        return hashlib.sha256(password.encode()).hexdigest()
    elif algorithm == "sha512":
        return hashlib.sha512(password.encode()).hexdigest()
    elif algorithm == "md5":
        # MD5 is not secure but included for compatibility
        return hashlib.md5(password.encode()).hexdigest()
    else:
        raise ValueError(f"Unsupported hash algorithm: {algorithm}")


def verify_api_key(
    provided_key: str, 
    expected_key: str, 
    algorithm: Optional[str] = None
) -> bool:
    """
    Verify an API key using constant-time comparison.
    
    Uses Rust acceleration when available for improved security.
    """
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            return _rust.RustSecurity.verify_api_key(
                provided_key, 
                expected_key, 
                algorithm
            )
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    if algorithm:
        # If algorithm is specified, compare hashed versions
        provided_hash = hash_password(provided_key, algorithm)
        expected_hash = hash_password(expected_key, algorithm)
        return constant_time_compare(provided_hash, expected_hash)
    else:
        # Direct comparison
        return constant_time_compare(provided_key, expected_key)


def generate_token(length: int = 32) -> str:
    """
    Generate a cryptographically secure random token.
    
    Args:
        length: Number of bytes for the token (default: 32)
    
    Returns:
        A hex-encoded string token
    """
    return secrets.token_hex(length)


def generate_csrf_token() -> str:
    """Generate a CSRF protection token."""
    return generate_token(16)


def verify_token(provided_token: str, expected_token: str) -> bool:
    """
    Verify a token using constant-time comparison.
    
    Args:
        provided_token: The token provided by the client
        expected_token: The expected token
    
    Returns:
        True if tokens match, False otherwise
    """
    return constant_time_compare(provided_token, expected_token)


def get_authorization_header(authorization: Optional[str]) -> Optional[str]:
    """
    Extract the token from an Authorization header.
    
    Args:
        authorization: The Authorization header value
    
    Returns:
        The extracted token or None
    """
    if not authorization:
        return None
    
    parts = authorization.split()
    if len(parts) == 2 and parts[0].lower() == "bearer":
        return parts[1]
    
    return None


def create_access_token(data: dict, expires_delta: Optional[int] = None) -> str:
    """
    Create a simple access token (for demonstration).
    
    In production, use proper JWT libraries.
    """
    import json
    import time
    import base64
    
    to_encode = data.copy()
    if expires_delta:
        expire = time.time() + expires_delta
        to_encode.update({"exp": expire})
    
    # Simple encoding - in production use JWT
    encoded = base64.urlsafe_b64encode(
        json.dumps(to_encode).encode()
    ).decode()
    
    return encoded


def decode_access_token(token: str) -> Optional[dict]:
    """
    Decode a simple access token (for demonstration).
    
    In production, use proper JWT libraries.
    """
    import json
    import time
    import base64
    
    try:
        decoded = base64.urlsafe_b64decode(token.encode()).decode()
        data = json.loads(decoded)
        
        # Check expiration
        if "exp" in data and data["exp"] < time.time():
            return None
        
        return data
    except Exception:
        return None


class SecurityError(Exception):
    """Base exception for security-related errors."""
    pass


class AuthenticationError(SecurityError):
    """Raised when authentication fails."""
    pass


class AuthorizationError(SecurityError):
    """Raised when authorization fails."""
    pass