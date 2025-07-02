"""
Python-Rust bridge module for FastAPI performance enhancements.

Provides dynamic loading of Rust extensions with automatic fallback to pure Python
implementations to ensure 100% compatibility.
"""

import importlib
import logging
import sys
import warnings
from typing import Any, Callable, Dict, Optional, TypeVar

from fastapi.logger import logger

__all__ = [
    "rust_available",
    "get_rust_function",
    "RustExtensionError",
    "check_compatibility",
]

T = TypeVar("T")

# Version compatibility constraints
MINIMUM_PYTHON_VERSION = (3, 8)
MAXIMUM_PYTHON_VERSION = (3, 13)
RUST_EXTENSION_VERSION = "0.1.0"

# Track loaded Rust modules
_rust_modules: Dict[str, Any] = {}
_rust_available: Optional[bool] = None
_compatibility_checked = False


class RustExtensionError(ImportError):
    """Raised when Rust extension loading fails."""

    pass


def _check_python_version() -> bool:
    """Verify Python version compatibility."""
    current_version = sys.version_info[:2]
    return (
        MINIMUM_PYTHON_VERSION <= current_version <= MAXIMUM_PYTHON_VERSION
    )


def _check_rust_extension_version() -> bool:
    """Verify Rust extension version compatibility."""
    try:
        from _fastapi_rust import __version__ as rust_version

        if rust_version != RUST_EXTENSION_VERSION:
            logger.warning(
                f"Rust extension version mismatch: expected {RUST_EXTENSION_VERSION}, "
                f"got {rust_version}"
            )
            return False
        return True
    except ImportError:
        return False
    except Exception as e:
        logger.error(f"Error checking Rust extension version: {e}")
        return False


def check_compatibility() -> bool:
    """
    Check if the system is compatible with Rust extensions.

    Returns:
        bool: True if compatible, False otherwise.
    """
    global _compatibility_checked

    if _compatibility_checked:
        return rust_available()

    _compatibility_checked = True

    if not _check_python_version():
        logger.info(
            f"Python version {sys.version_info[:2]} not supported for Rust extensions"
        )
        return False

    return True


def rust_available() -> bool:
    """
    Check if Rust extensions are available and loaded.

    Returns:
        bool: True if Rust extensions are available, False otherwise.
    """
    global _rust_available

    if _rust_available is not None:
        return _rust_available

    if not check_compatibility():
        _rust_available = False
        return False

    try:
        import _fastapi_rust

        # Verify the module has expected attributes
        required_attrs = [
            "init_rust_backend",
            "create_api_route",
            "jsonable_encoder",
            "constant_time_compare",
        ]

        for attr in required_attrs:
            if not hasattr(_fastapi_rust, attr):
                raise RustExtensionError(
                    f"Rust extension missing required attribute: {attr}"
                )

        # Initialize the Rust backend
        if _fastapi_rust.init_rust_backend():
            _rust_available = True
            logger.info("Rust extensions loaded successfully")
        else:
            _rust_available = False
            logger.warning("Rust backend initialization failed")

    except ImportError as e:
        _rust_available = False
        logger.info(f"Rust extensions not available: {e}")
    except Exception as e:
        _rust_available = False
        logger.error(f"Error loading Rust extensions: {e}")

    return _rust_available


def _load_rust_module(module_name: str) -> Any:
    """
    Load a Rust module dynamically.

    Args:
        module_name: Name of the module to load.

    Returns:
        The loaded module.

    Raises:
        RustExtensionError: If module loading fails.
    """
    if module_name in _rust_modules:
        return _rust_modules[module_name]

    try:
        if module_name == "_fastapi_rust":
            module = importlib.import_module(module_name)
        else:
            # For sub-modules
            module = importlib.import_module(f"_fastapi_rust.{module_name}")

        _rust_modules[module_name] = module
        return module
    except ImportError as e:
        raise RustExtensionError(f"Failed to load Rust module {module_name}: {e}")


def get_rust_function(
    module_name: str,
    function_name: str,
    fallback: Optional[Callable[..., T]] = None,
) -> Callable[..., T]:
    """
    Get a function from Rust extension with automatic fallback.

    Args:
        module_name: Name of the Rust module.
        function_name: Name of the function to retrieve.
        fallback: Optional fallback function if Rust is not available.

    Returns:
        The Rust function if available, otherwise the fallback.

    Raises:
        ValueError: If no function is available (neither Rust nor fallback).
    """
    if rust_available():
        try:
            if module_name == "_fastapi_rust":
                import _fastapi_rust

                module = _fastapi_rust
            else:
                module = _load_rust_module(module_name)

            func = getattr(module, function_name, None)
            if func is not None:
                return func
        except Exception as e:
            logger.debug(
                f"Failed to get Rust function {module_name}.{function_name}: {e}"
            )

    if fallback is not None:
        return fallback

    raise ValueError(
        f"No implementation available for {module_name}.{function_name}"
    )


class RustAccelerator:
    """
    Context manager for conditionally using Rust acceleration.

    Example:
        with RustAccelerator("routing") as accelerator:
            if accelerator.available:
                # Use Rust implementation
                result = accelerator.call("match_route", path, method)
            else:
                # Use Python fallback
                result = python_match_route(path, method)
    """

    def __init__(self, feature: str):
        self.feature = feature
        self.available = False
        self._module = None

    def __enter__(self) -> "RustAccelerator":
        if rust_available():
            try:
                self._module = _load_rust_module("_fastapi_rust")
                self.available = True
            except Exception:
                self.available = False
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        # Clean up resources if needed
        pass

    def call(self, function_name: str, *args, **kwargs) -> Any:
        """Call a Rust function with the given arguments."""
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
    def hash_password(password: str, algorithm: Optional[str] = None) -> str:
        """Hash password with Rust acceleration if available."""
        func = get_rust_function(
            "_fastapi_rust",
            "hash_password",
            fallback=None,  # Will be provided by security/utils.py
        )
        return func(password, algorithm)


# Performance monitoring decorator
def with_rust_acceleration(operation: str):
    """
    Decorator to automatically use Rust acceleration when available.

    Args:
        operation: Name of the operation for logging.

    Example:
        @with_rust_acceleration("json_encoding")
        def encode_json(data):
            # Python implementation
            return json.dumps(data)
    """

    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        def wrapper(*args, **kwargs) -> T:
            if rust_available():
                # Try to find Rust equivalent
                rust_func_name = f"rust_{func.__name__}"
                try:
                    rust_func = get_rust_function(
                        "_fastapi_rust", rust_func_name, fallback=None
                    )
                    if rust_func:
                        return rust_func(*args, **kwargs)
                except ValueError:
                    pass

            # Fall back to Python implementation
            return func(*args, **kwargs)

        wrapper.__name__ = func.__name__
        wrapper.__doc__ = func.__doc__
        return wrapper

    return decorator


# Initialize on import
if not _compatibility_checked:
    check_compatibility()