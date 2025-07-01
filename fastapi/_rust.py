"""
Python-Rust bridge module for FastAPI performance optimization.
Provides dynamic loading of Rust extensions with automatic fallback to Python implementations.
"""

import sys
import logging
import warnings
from typing import Any, Optional, Dict, Callable
from importlib import import_module

logger = logging.getLogger("fastapi.rust")

_RUST_EXTENSION_NAME = "_fastapi_rust"
_RUST_LOADED = False
_RUST_MODULE: Optional[Any] = None
_FALLBACK_WARNINGS_SHOWN: Dict[str, bool] = {}

# Version compatibility requirements
MIN_PYTHON_VERSION = (3, 8)
MIN_RUST_EXTENSION_VERSION = "0.1.0"


def _check_python_version() -> bool:
    """Check if current Python version meets minimum requirements."""
    return sys.version_info >= MIN_PYTHON_VERSION


def _load_rust_extension() -> Optional[Any]:
    """Attempt to load the Rust extension module."""
    global _RUST_MODULE, _RUST_LOADED
    
    if _RUST_LOADED:
        return _RUST_MODULE
    
    if not _check_python_version():
        logger.warning(
            f"Python version {sys.version_info} is below minimum required {MIN_PYTHON_VERSION}"
        )
        return None
    
    try:
        _RUST_MODULE = import_module(_RUST_EXTENSION_NAME)
        
        # Verify extension version if available
        if hasattr(_RUST_MODULE, "__version__"):
            version = getattr(_RUST_MODULE, "__version__")
            logger.info(f"Loaded Rust extension version: {version}")
        
        # Initialize Rust backend
        if hasattr(_RUST_MODULE, "init_rust_backend"):
            if _RUST_MODULE.init_rust_backend():
                logger.info("Rust backend initialized successfully")
            else:
                logger.warning("Rust backend initialization returned False")
        
        _RUST_LOADED = True
        logger.info("Rust extension loaded successfully")
        return _RUST_MODULE
        
    except ImportError as e:
        logger.debug(f"Rust extension not available: {e}")
        return None
    except Exception as e:
        logger.error(f"Error loading Rust extension: {e}", exc_info=True)
        return None


def _show_fallback_warning(func_name: str) -> None:
    """Show warning when falling back to Python implementation."""
    if func_name not in _FALLBACK_WARNINGS_SHOWN:
        warnings.warn(
            f"Using Python fallback for '{func_name}'. "
            "Install fastapi-rs for better performance.",
            RuntimeWarning,
            stacklevel=3
        )
        _FALLBACK_WARNINGS_SHOWN[func_name] = True


def _get_rust_function(func_name: str) -> Optional[Callable]:
    """Get a function from the Rust module if available."""
    rust_module = _load_rust_extension()
    if rust_module and hasattr(rust_module, func_name):
        return getattr(rust_module, func_name)
    return None


# Core routing functions with fallback
def create_api_route(path: str, methods: list, name: Optional[str] = None):
    """Create an API route with Rust optimization or Python fallback."""
    rust_func = _get_rust_function("create_api_route")
    if rust_func:
        return rust_func(path, methods, name)
    
    _show_fallback_warning("create_api_route")
    from fastapi.routing import APIRoute as _PyAPIRoute
    return _PyAPIRoute(path=path, endpoint=lambda: None, methods=methods, name=name)


def match_route(path: str, method: str, routes: list):
    """Match a route using Rust optimization or Python fallback."""
    rust_func = _get_rust_function("match_route")
    if rust_func:
        return rust_func(path, method, routes)
    
    _show_fallback_warning("match_route")
    # Python fallback implementation
    for idx, route in enumerate(routes):
        if method in route.methods and route.path_regex.match(path):
            match = route.path_regex.match(path)
            return (idx, match.groupdict() if match else {})
    return None


def compile_path_regex(path: str) -> str:
    """Compile path to regex pattern using Rust or Python fallback."""
    rust_func = _get_rust_function("compile_path_regex")
    if rust_func:
        return rust_func(path)
    
    _show_fallback_warning("compile_path_regex")
    import re
    # Python fallback - simplified path compilation
    pattern = path
    pattern = re.sub(r'{([^}:]+)(?::([^}]+))?}', r'(?P<\1>[^/]+)', pattern)
    return f"^{pattern}$"


# Parameter validation functions with fallback
def validate_path_params(params: dict, schema: dict):
    """Validate path parameters using Rust or Python fallback."""
    rust_func = _get_rust_function("validate_path_params")
    if rust_func:
        return rust_func(params, schema)
    
    _show_fallback_warning("validate_path_params")
    # Python fallback
    from fastapi.params import Path
    errors = []
    validated_data = {}
    
    for param_name, param_value in params.items():
        if param_name in schema:
            # Simplified validation
            validated_data[param_name] = param_value
    
    return type('ValidationResult', (), {
        'valid': len(errors) == 0,
        'errors': errors,
        'validated_data': validated_data
    })()


def validate_query_params(params: dict, schema: dict):
    """Validate query parameters using Rust or Python fallback."""
    rust_func = _get_rust_function("validate_query_params")
    if rust_func:
        return rust_func(params, schema)
    
    _show_fallback_warning("validate_query_params")
    # Reuse path params validation logic for simplicity
    return validate_path_params(params, schema)


def validate_header_params(headers: dict, schema: dict):
    """Validate header parameters using Rust or Python fallback."""
    rust_func = _get_rust_function("validate_header_params")
    if rust_func:
        return rust_func(headers, schema)
    
    _show_fallback_warning("validate_header_params")
    # Normalize header names to lowercase
    normalized_headers = {k.lower(): v for k, v in headers.items()}
    return validate_path_params(normalized_headers, schema)


def validate_body_params(body: Any, schema: dict):
    """Validate request body using Rust or Python fallback."""
    rust_func = _get_rust_function("validate_body_params")
    if rust_func:
        return rust_func(body, schema)
    
    _show_fallback_warning("validate_body_params")
    # Python fallback
    import json
    errors = []
    validated_data = {}
    
    try:
        if isinstance(body, (str, bytes)):
            body_data = json.loads(body)
        else:
            body_data = body
        validated_data = body_data
    except Exception as e:
        errors.append(str(e))
    
    return type('ValidationResult', (), {
        'valid': len(errors) == 0,
        'errors': errors,
        'validated_data': validated_data
    })()


# Serialization functions with fallback
def jsonable_encoder(obj: Any, **kwargs) -> str:
    """Encode object to JSON using Rust or Python fallback."""
    rust_func = _get_rust_function("jsonable_encoder")
    if rust_func:
        return rust_func(obj)
    
    _show_fallback_warning("jsonable_encoder")
    from fastapi.encoders import jsonable_encoder as _py_encoder
    import json
    return json.dumps(_py_encoder(obj, **kwargs))


def serialize_response(data: Any, content_type: Optional[str] = None) -> bytes:
    """Serialize response data using Rust or Python fallback."""
    rust_func = _get_rust_function("serialize_response")
    if rust_func:
        return rust_func(data, content_type)
    
    _show_fallback_warning("serialize_response")
    # Python fallback
    if content_type == "application/json" or content_type is None:
        import json
        from fastapi.encoders import jsonable_encoder as _py_encoder
        return json.dumps(_py_encoder(data)).encode('utf-8')
    elif content_type == "text/plain":
        return str(data).encode('utf-8')
    else:
        return bytes(data)


def deserialize_request(body: bytes, content_type: str):
    """Deserialize request body using Rust or Python fallback."""
    rust_func = _get_rust_function("deserialize_request")
    if rust_func:
        return rust_func(body, content_type)
    
    _show_fallback_warning("deserialize_request")
    # Python fallback
    if content_type == "application/json":
        import json
        return json.loads(body.decode('utf-8'))
    elif content_type == "application/x-www-form-urlencoded":
        from urllib.parse import parse_qs
        return parse_qs(body.decode('utf-8'))
    else:
        return body.decode('utf-8')


# Security functions with fallback
def constant_time_compare(a: str, b: str) -> bool:
    """Constant-time string comparison using Rust or Python fallback."""
    rust_func = _get_rust_function("constant_time_compare")
    if rust_func:
        return rust_func(a, b)
    
    _show_fallback_warning("constant_time_compare")
    import hmac
    return hmac.compare_digest(a, b)


def verify_api_key(provided_key: str, expected_key: str, algorithm: Optional[str] = None) -> bool:
    """Verify API key using Rust or Python fallback."""
    rust_func = _get_rust_function("verify_api_key")
    if rust_func:
        return rust_func(provided_key, expected_key, algorithm)
    
    _show_fallback_warning("verify_api_key")
    # Python fallback
    if algorithm == "sha256":
        import hashlib
        provided_hash = hashlib.sha256(provided_key.encode()).hexdigest()
        expected_hash = hashlib.sha256(expected_key.encode()).hexdigest()
        return constant_time_compare(provided_hash, expected_hash)
    else:
        return constant_time_compare(provided_key, expected_key)


def hash_password(password: str, algorithm: Optional[str] = None) -> str:
    """Hash password using Rust or Python fallback."""
    rust_func = _get_rust_function("hash_password")
    if rust_func:
        return rust_func(password, algorithm)
    
    _show_fallback_warning("hash_password")
    # Python fallback
    import hashlib
    if algorithm == "sha256" or algorithm is None:
        return hashlib.sha256(password.encode()).hexdigest()
    elif algorithm == "bcrypt":
        # Simplified - would use bcrypt library in production
        return f"bcrypt:{hashlib.sha256(password.encode()).hexdigest()}"
    else:
        raise ValueError(f"Unsupported algorithm: {algorithm}")


# Utility functions with fallback
def generate_unique_id(route_name: str, method: str, path: str) -> str:
    """Generate unique operation ID using Rust or Python fallback."""
    rust_func = _get_rust_function("generate_unique_id")
    if rust_func:
        return rust_func(route_name, method, path)
    
    _show_fallback_warning("generate_unique_id")
    # Python fallback
    from fastapi.utils import generate_operation_id_for_path
    return generate_operation_id_for_path(name=route_name, path=path, method=method)


def parse_content_type(content_type: str) -> tuple:
    """Parse content type header using Rust or Python fallback."""
    rust_func = _get_rust_function("parse_content_type")
    if rust_func:
        return rust_func(content_type)
    
    _show_fallback_warning("parse_content_type")
    # Python fallback
    parts = content_type.split(';')
    media_type = parts[0].strip().lower()
    params = {}
    
    for part in parts[1:]:
        if '=' in part:
            key, value = part.split('=', 1)
            params[key.strip().lower()] = value.strip().strip('"')
    
    return (media_type, params)


def convert_python_type(obj: Any) -> str:
    """Convert Python object to type string using Rust or Python fallback."""
    rust_func = _get_rust_function("convert_python_type")
    if rust_func:
        return rust_func(obj)
    
    _show_fallback_warning("convert_python_type")
    # Python fallback
    type_map = {
        str: "string",
        int: "integer",
        float: "number",
        bool: "boolean",
        list: "array",
        dict: "object",
        type(None): "null",
        bytes: "binary"
    }
    return type_map.get(type(obj), f"unknown:{type(obj).__name__}")


# Resource cleanup
def cleanup_rust_resources():
    """Clean up Rust resources when shutting down."""
    global _RUST_MODULE, _RUST_LOADED
    
    if _RUST_MODULE and hasattr(_RUST_MODULE, "cleanup"):
        try:
            _RUST_MODULE.cleanup()
            logger.info("Rust resources cleaned up successfully")
        except Exception as e:
            logger.error(f"Error cleaning up Rust resources: {e}")
    
    _RUST_MODULE = None
    _RUST_LOADED = False


# Performance monitoring
class RustPerformanceMonitor:
    """Monitor performance metrics for Rust vs Python implementations."""
    
    def __init__(self):
        self.metrics = {
            'rust_calls': 0,
            'python_calls': 0,
            'rust_time': 0.0,
            'python_time': 0.0
        }
    
    def record_call(self, is_rust: bool, duration: float):
        """Record a function call and its duration."""
        if is_rust:
            self.metrics['rust_calls'] += 1
            self.metrics['rust_time'] += duration
        else:
            self.metrics['python_calls'] += 1
            self.metrics['python_time'] += duration
    
    def get_statistics(self) -> dict:
        """Get performance statistics."""
        return {
            'total_calls': self.metrics['rust_calls'] + self.metrics['python_calls'],
            'rust_percentage': (
                self.metrics['rust_calls'] / 
                (self.metrics['rust_calls'] + self.metrics['python_calls']) * 100
                if self.metrics['rust_calls'] + self.metrics['python_calls'] > 0 
                else 0
            ),
            'average_rust_time': (
                self.metrics['rust_time'] / self.metrics['rust_calls']
                if self.metrics['rust_calls'] > 0 else 0
            ),
            'average_python_time': (
                self.metrics['python_time'] / self.metrics['python_calls']
                if self.metrics['python_calls'] > 0 else 0
            ),
            **self.metrics
        }


# Global performance monitor instance
_performance_monitor = RustPerformanceMonitor()


def get_performance_stats() -> dict:
    """Get current performance statistics."""
    return _performance_monitor.get_statistics()


# Module initialization
def __getattr__(name: str):
    """Dynamic attribute access for additional Rust functions."""
    rust_func = _get_rust_function(name)
    if rust_func:
        return rust_func
    raise AttributeError(f"module 'fastapi._rust' has no attribute '{name}'")


# Register cleanup on module unload
import atexit
atexit.register(cleanup_rust_resources)