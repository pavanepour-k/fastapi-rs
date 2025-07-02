"""
FastAPI Rust Bridge Module

Provides seamless integration between Python and Rust implementations
with automatic fallback to pure Python when Rust extensions are unavailable.
"""
import importlib
import logging
import warnings
from typing import Any, Callable, Dict, Optional, TypeVar, Union

logger = logging.getLogger("fastapi.rust")

# Global state for Rust extension availability
_rust_available: Optional[bool] = None
_rust_modules: Dict[str, Any] = {}

T = TypeVar("T")


def rust_available() -> bool:
    """Check if Rust extensions are available."""
    global _rust_available
    
    if _rust_available is not None:
        return _rust_available
    
    try:
        import _fastapi_rust
        _rust_available = True
        logger.info("Rust acceleration enabled")
    except ImportError:
        _rust_available = False
        logger.info("Rust acceleration not available, using pure Python")
    
    return _rust_available


def get_rust_module(module_name: str) -> Optional[Any]:
    """Get a Rust module with caching."""
    if module_name in _rust_modules:
        return _rust_modules[module_name]
    
    if not rust_available():
        return None
    
    try:
        module = importlib.import_module(module_name)
        _rust_modules[module_name] = module
        return module
    except ImportError:
        logger.debug(f"Rust module {module_name} not found")
        return None


def get_rust_function(
    module_name: str, 
    function_name: str, 
    fallback: Optional[Callable] = None
) -> Callable:
    """Get a Rust function with automatic fallback."""
    module = get_rust_module(module_name)
    
    if module and hasattr(module, function_name):
        return getattr(module, function_name)
    
    if fallback:
        return fallback
    
    raise RuntimeError(
        f"Rust function {module_name}.{function_name} not available and no fallback provided"
    )


class RustExtension:
    """Base class for Rust extension wrappers."""
    
    def __init__(self, feature: str):
        self.feature = feature
        self.available = rust_available()
        self._module = None
        
        if self.available:
            try:
                self._module = importlib.import_module("_fastapi_rust")
            except ImportError:
                self.available = False
                warnings.warn(
                    f"Rust extension for {feature} not available, using Python fallback",
                    RuntimeWarning,
                    stacklevel=2
                )
    
    def call_rust(self, function_name: str, *args, **kwargs):
        """Call a Rust function with error handling."""
        if not self.available or self._module is None:
            raise RuntimeError(
                f"Rust acceleration not available for {self.feature}"
            )
        
        func = getattr(self._module, function_name)
        return func(*args, **kwargs)


def log_performance_metric(operation: str, rust_time: float, python_time: float) -> None:
    """
    Log performance comparison between Rust and Python implementations.
    
    Args:
        operation: Name of the operation.
        rust_time: Time taken by Rust implementation.
        python_time: Time taken by Python implementation.
    """
    if rust_time > 0:
        speedup = python_time / rust_time
        logger.debug(
            f"{operation}: Rust {rust_time:.4f}s, Python {python_time:.4f}s, "
            f"Speedup: {speedup:.2f}x"
        )


def cleanup_rust_resources() -> None:
    """Clean up Rust extension resources."""
    global _rust_modules, _rust_available
    
    _rust_modules.clear()
    _rust_available = None
    
    # Force garbage collection of Rust objects
    import gc
    gc.collect()


# Compatibility layer for specific FastAPI components
class RustRouting:
    """Rust-accelerated routing functionality."""
    
    @staticmethod
    def create_route(path: str, methods: list, name: Optional[str] = None):
        """Create a route with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "create_api_route",
            fallback=None,  # Will be provided by routing.py
        )
        return func(path, methods, name)
    
    @staticmethod
    def match_route(path: str, method: str, routes: list):
        """Match a route with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "match_route",
            fallback=None,  # Will be provided by routing.py
        )
        return func(path, method, routes)


class RustSerialization:
    """Rust-accelerated serialization functionality."""
    
    @staticmethod
    def jsonable_encoder(obj: Any) -> str:
        """Encode object to JSON with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "jsonable_encoder",
            fallback=None,  # Will be provided by encoders.py
        )
        return func(obj)
    
    @staticmethod
    def serialize_response(data: Any, content_type: Optional[str] = None) -> bytes:
        """Serialize response with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "serialize_response",
            fallback=None,  # Will be provided by encoders.py
        )
        return func(data, content_type)


class RustSecurity:
    """Rust-accelerated security functionality."""
    
    @staticmethod
    def constant_time_compare(a: str, b: str) -> bool:
        """Constant-time string comparison with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "constant_time_compare",
            fallback=None,  # Will be provided by security/utils.py
        )
        return func(a, b)
    
    @staticmethod
    def verify_api_key(provided_key: str, expected_key: str, algorithm: Optional[str] = None) -> bool:
        """Verify API key with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "verify_api_key",
            fallback=None,  # Will be provided by security/utils.py
        )
        return func(provided_key, expected_key, algorithm)


class RustValidation:
    """Rust-accelerated parameter validation."""
    
    @staticmethod
    def validate_path_params(params: dict, schema: dict):
        """Validate path parameters with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "validate_path_params",
            fallback=None,  # Will be provided by params.py
        )
        return func(params, schema)
    
    @staticmethod
    def validate_query_params(params: dict, schema: dict):
        """Validate query parameters with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "validate_query_params",
            fallback=None,  # Will be provided by params.py
        )
        return func(params, schema)
    
    @staticmethod
    def validate_header_params(headers: dict, schema: dict):
        """Validate header parameters with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "validate_header_params",
            fallback=None,  # Will be provided by params.py
        )
        return func(headers, schema)
    
    @staticmethod
    def validate_body_params(body: Any, schema: dict):
        """Validate body parameters with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "validate_body_params",
            fallback=None,  # Will be provided by params.py
        )
        return func(body, schema)


class RustUtils:
    """Rust-accelerated utility functions."""
    
    @staticmethod
    def generate_unique_id(route_name: str, method: str, path: str) -> str:
        """Generate unique ID with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "generate_unique_id",
            fallback=None,  # Will be provided by utils.py
        )
        return func(route_name, method, path)
    
    @staticmethod
    def parse_content_type(content_type: str) -> tuple:
        """Parse content type with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "parse_content_type",
            fallback=None,  # Will be provided by utils.py
        )
        return func(content_type)


# Initialize Rust availability check on module import
rust_available()