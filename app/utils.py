import logging
import logging.handlers
import sys
import traceback
import uuid
import hashlib
import hmac
import secrets
import time
import json
from datetime import datetime, timezone
from typing import Optional, Dict, Any, List, Union
from pathlib import Path
from functools import wraps
from contextlib import contextmanager

from fastapi import HTTPException, Request, Response, status
from fastapi.responses import JSONResponse
from pydantic import ValidationError
import structlog

from .config import get_settings


settings = get_settings()


# Logging configuration
def setup_logging():
    """Setup application logging configuration."""
    
    # Configure structlog
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            structlog.processors.StackInfoRenderer(),
            structlog.dev.set_exc_info,
            structlog.processors.TimeStamper(fmt="ISO"),
            structlog.dev.ConsoleRenderer() if settings.is_development else structlog.processors.JSONRenderer()
        ],
        wrapper_class=structlog.make_filtering_bound_logger(getattr(logging, settings.log_level)),
        logger_factory=structlog.WriteLoggerFactory(),
        cache_logger_on_first_use=True,
    )
    
    # Configure standard logging
    logger = logging.getLogger()
    logger.setLevel(getattr(logging, settings.log_level))
    
    # Remove existing handlers
    for handler in logger.handlers[:]:
        logger.removeHandler(handler)
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(getattr(logging, settings.log_level))
    
    if settings.is_development:
        formatter = logging.Formatter(
            fmt='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            datefmt='%Y-%m-%d %H:%M:%S'
        )
    else:
        formatter = logging.Formatter(
            fmt='{"timestamp": "%(asctime)s", "name": "%(name)s", "level": "%(levelname)s", "message": "%(message)s"}',
            datefmt='%Y-%m-%dT%H:%M:%S'
        )
    
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)
    
    # File handler if log file is specified
    if settings.log_file:
        log_path = Path(settings.log_file)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        
        file_handler = logging.handlers.RotatingFileHandler(
            filename=settings.log_file,
            maxBytes=10*1024*1024,  # 10MB
            backupCount=5,
            encoding='utf-8'
        )
        file_handler.setLevel(getattr(logging, settings.log_level))
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)
    
    # Suppress noisy loggers in production
    if settings.is_production:
        logging.getLogger("uvicorn.access").setLevel(logging.WARNING)
        logging.getLogger("sqlalchemy.engine").setLevel(logging.WARNING)


# Get structured logger
def get_logger(name: str = None) -> structlog.BoundLogger:
    """Get structured logger instance."""
    return structlog.get_logger(name or __name__)


logger = get_logger(__name__)


# Exception handling utilities
class AppException(Exception):
    """Base application exception."""
    
    def __init__(
        self,
        message: str,
        code: str = None,
        status_code: int = status.HTTP_500_INTERNAL_SERVER_ERROR,
        details: Dict[str, Any] = None
    ):
        self.message = message
        self.code = code or self.__class__.__name__
        self.status_code = status_code
        self.details = details or {}
        super().__init__(self.message)


class ValidationException(AppException):
    """Validation error exception."""
    
    def __init__(self, message: str, field_errors: List[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            code="VALIDATION_ERROR",
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            details={"field_errors": field_errors or []}
        )


class AuthenticationException(AppException):
    """Authentication error exception."""
    
    def __init__(self, message: str = "Authentication failed"):
        super().__init__(
            message=message,
            code="AUTHENTICATION_ERROR",
            status_code=status.HTTP_401_UNAUTHORIZED
        )


class AuthorizationException(AppException):
    """Authorization error exception."""
    
    def __init__(self, message: str = "Access denied"):
        super().__init__(
            message=message,
            code="AUTHORIZATION_ERROR",
            status_code=status.HTTP_403_FORBIDDEN
        )


class ResourceNotFoundException(AppException):
    """Resource not found exception."""
    
    def __init__(self, resource: str, identifier: Union[str, int] = None):
        message = f"{resource} not found"
        if identifier:
            message += f" with identifier: {identifier}"
        
        super().__init__(
            message=message,
            code="RESOURCE_NOT_FOUND",
            status_code=status.HTTP_404_NOT_FOUND,
            details={"resource": resource, "identifier": str(identifier) if identifier else None}
        )


class ConflictException(AppException):
    """Resource conflict exception."""
    
    def __init__(self, message: str, resource: str = None):
        super().__init__(
            message=message,
            code="RESOURCE_CONFLICT",
            status_code=status.HTTP_409_CONFLICT,
            details={"resource": resource} if resource else {}
        )


class RateLimitException(AppException):
    """Rate limit exceeded exception."""
    
    def __init__(self, message: str = "Rate limit exceeded", retry_after: int = None):
        super().__init__(
            message=message,
            code="RATE_LIMIT_EXCEEDED",
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            details={"retry_after": retry_after} if retry_after else {}
        )


class ExternalServiceException(AppException):
    """External service error exception."""
    
    def __init__(self, service: str, message: str = None):
        super().__init__(
            message=message or f"External service '{service}' is unavailable",
            code="EXTERNAL_SERVICE_ERROR",
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            details={"service": service}
        )


# Exception handlers
async def app_exception_handler(request: Request, exc: AppException) -> JSONResponse:
    """Handle application exceptions."""
    
    logger.error(
        "Application exception occurred",
        exception=exc.__class__.__name__,
        message=exc.message,
        code=exc.code,
        status_code=exc.status_code,
        details=exc.details,
        path=request.url.path,
        method=request.method
    )
    
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "error": exc.message,
            "code": exc.code,
            "details": exc.details,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "path": request.url.path
        }
    )


async def validation_exception_handler(request: Request, exc: ValidationError) -> JSONResponse:
    """Handle Pydantic validation exceptions."""
    
    field_errors = []
    for error in exc.errors():
        field_errors.append({
            "field": ".".join(str(loc) for loc in error["loc"]),
            "message": error["msg"],
            "type": error["type"]
        })
    
    logger.warning(
        "Validation error occurred",
        errors=field_errors,
        path=request.url.path,
        method=request.method
    )
    
    return JSONResponse(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        content={
            "error": "Validation error",
            "code": "VALIDATION_ERROR",
            "details": {"field_errors": field_errors},
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "path": request.url.path
        }
    )


async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    """Handle FastAPI HTTP exceptions."""
    
    logger.warning(
        "HTTP exception occurred",
        status_code=exc.status_code,
        detail=exc.detail,
        path=request.url.path,
        method=request.method
    )
    
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "error": exc.detail,
            "code": f"HTTP_{exc.status_code}",
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "path": request.url.path
        }
    )


async def general_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """Handle general exceptions."""
    
    error_id = str(uuid.uuid4())
    
    logger.error(
        "Unhandled exception occurred",
        error_id=error_id,
        exception=exc.__class__.__name__,
        message=str(exc),
        traceback=traceback.format_exc(),
        path=request.url.path,
        method=request.method
    )
    
    # Don't expose internal error details in production
    if settings.is_production:
        return JSONResponse(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            content={
                "error": "Internal server error",
                "code": "INTERNAL_SERVER_ERROR",
                "error_id": error_id,
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "path": request.url.path
            }
        )
    else:
        return JSONResponse(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            content={
                "error": str(exc),
                "code": "INTERNAL_SERVER_ERROR",
                "error_id": error_id,
                "traceback": traceback.format_exc().split('\n'),
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "path": request.url.path
            }
        )


# Utility functions
def generate_correlation_id() -> str:
    """Generate unique correlation ID for request tracking."""
    return str(uuid.uuid4())


def generate_secure_token(length: int = 32) -> str:
    """Generate cryptographically secure token."""
    return secrets.token_urlsafe(length)


def generate_api_key() -> str:
    """Generate API key."""
    return f"ak_{secrets.token_urlsafe(32)}"


def hash_string(data: str, salt: str = None) -> str:
    """Hash string with optional salt."""
    if salt:
        return hashlib.pbkdf2_hex(data.encode(), salt.encode(), 100000)
    return hashlib.sha256(data.encode()).hexdigest()


def verify_signature(data: str, signature: str, secret: str) -> bool:
    """Verify HMAC signature."""
    expected_signature = hmac.new(
        secret.encode(),
        data.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected_signature)


def create_signature(data: str, secret: str) -> str:
    """Create HMAC signature."""
    return hmac.new(
        secret.encode(),
        data.encode(),
        hashlib.sha256
    ).hexdigest()


def sanitize_filename(filename: str) -> str:
    """Sanitize filename for safe storage."""
    import re
    
    # Remove path components
    filename = Path(filename).name
    
    # Replace unsafe characters
    filename = re.sub(r'[<>:"/\\|?*]', '_', filename)
    
    # Remove control characters
    filename = ''.join(char for char in filename if ord(char) >= 32)
    
    # Limit length
    if len(filename) > 255:
        name, ext = Path(filename).stem, Path(filename).suffix
        filename = name[:255-len(ext)] + ext
    
    return filename or 'unnamed_file'


def format_file_size(size_bytes: int) -> str:
    """Format file size in human readable format."""
    if size_bytes == 0:
        return "0 B"
    
    size_names = ["B", "KB", "MB", "GB", "TB"]
    i = 0
    while size_bytes >= 1024 and i < len(size_names) - 1:
        size_bytes /= 1024.0
        i += 1
    
    return f"{size_bytes:.1f} {size_names[i]}"


def parse_duration(duration_str: str) -> int:
    """Parse duration string to seconds."""
    import re
    
    duration_regex = re.compile(
        r'(?:(\d+)\s*d[ays]*)?'
        r'(?:(\d+)\s*h[ours]*)?'
        r'(?:(\d+)\s*m[inutes]*)?'
        r'(?:(\d+)\s*s[econds]*)?',
        re.IGNORECASE
    )
    
    match = duration_regex.match(duration_str.strip())
    if not match:
        raise ValueError(f"Invalid duration format: {duration_str}")
    
    days, hours, minutes, seconds = match.groups()
    
    total_seconds = 0
    if days:
        total_seconds += int(days) * 24 * 3600
    if hours:
        total_seconds += int(hours) * 3600
    if minutes:
        total_seconds += int(minutes) * 60
    if seconds:
        total_seconds += int(seconds)
    
    return total_seconds


# Decorators
def timing_decorator(func):
    """Decorator to measure function execution time."""
    @wraps(func)
    async def async_wrapper(*args, **kwargs):
        start_time = time.time()
        try:
            result = await func(*args, **kwargs)
            execution_time = time.time() - start_time
            logger.info(
                f"Function {func.__name__} executed",
                execution_time=f"{execution_time:.3f}s"
            )
            return result
        except Exception as e:
            execution_time = time.time() - start_time
            logger.error(
                f"Function {func.__name__} failed",
                execution_time=f"{execution_time:.3f}s",
                error=str(e)
            )
            raise
    
    @wraps(func)
    def sync_wrapper(*args, **kwargs):
        start_time = time.time()
        try:
            result = func(*args, **kwargs)
            execution_time = time.time() - start_time
            logger.info(
                f"Function {func.__name__} executed",
                execution_time=f"{execution_time:.3f}s"
            )
            return result
        except Exception as e:
            execution_time = time.time() - start_time
            logger.error(
                f"Function {func.__name__} failed",
                execution_time=f"{execution_time:.3f}s",
                error=str(e)
            )
            raise
    
    if hasattr(func, '__code__') and 'await' in func.__code__.co_names:
        return async_wrapper
    else:
        return sync_wrapper


def retry_decorator(max_retries: int = 3, delay: float = 1.0):
    """Decorator to retry function on failure."""
    def decorator(func):
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            last_exception = None
            
            for attempt in range(max_retries + 1):
                try:
                    return await func(*args, **kwargs)
                except Exception as e:
                    last_exception = e
                    if attempt < max_retries:
                        logger.warning(
                            f"Function {func.__name__} failed, retrying",
                            attempt=attempt + 1,
                            max_retries=max_retries,
                            error=str(e)
                        )
                        await asyncio.sleep(delay * (2 ** attempt))  # Exponential backoff
                    else:
                        logger.error(
                            f"Function {func.__name__} failed after all retries",
                            attempts=max_retries + 1,
                            error=str(e)
                        )
            
            raise last_exception
        
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            last_exception = None
            
            for attempt in range(max_retries + 1):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    last_exception = e
                    if attempt < max_retries:
                        logger.warning(
                            f"Function {func.__name__} failed, retrying",
                            attempt=attempt + 1,
                            max_retries=max_retries,
                            error=str(e)
                        )
                        time.sleep(delay * (2 ** attempt))  # Exponential backoff
                    else:
                        logger.error(
                            f"Function {func.__name__} failed after all retries",
                            attempts=max_retries + 1,
                            error=str(e)
                        )
            
            raise last_exception
        
        if hasattr(func, '__code__') and 'await' in func.__code__.co_names:
            return async_wrapper
        else:
            return sync_wrapper
    
    return decorator


# Context managers
@contextmanager
def performance_monitor(operation_name: str):
    """Context manager to monitor operation performance."""
    start_time = time.time()
    start_memory = None
    
    try:
        import psutil
        process = psutil.Process()
        start_memory = process.memory_info().rss
    except ImportError:
        pass
    
    logger.info(f"Starting operation: {operation_name}")
    
    try:
        yield
        success = True
    except Exception as e:
        success = False
        logger.error(f"Operation failed: {operation_name}", error=str(e))
        raise
    finally:
        execution_time = time.time() - start_time
        
        log_data = {
            "operation": operation_name,
            "execution_time": f"{execution_time:.3f}s",
            "success": success
        }
        
        if start_memory:
            try:
                end_memory = process.memory_info().rss
                memory_diff = end_memory - start_memory
                log_data["memory_delta"] = format_file_size(memory_diff)
            except:
                pass
        
        logger.info(f"Operation completed: {operation_name}", **log_data)


# JSON utilities
def json_serializer(obj):
    """Custom JSON serializer for complex objects."""
    if isinstance(obj, datetime):
        return obj.isoformat()
    elif isinstance(obj, uuid.UUID):
        return str(obj)
    elif hasattr(obj, '__dict__'):
        return obj.__dict__
    else:
        return str(obj)


def safe_json_loads(data: str, default=None):
    """Safely load JSON data with fallback."""
    try:
        return json.loads(data)
    except (json.JSONDecodeError, TypeError):
        return default


def safe_json_dumps(data: Any, default=json_serializer) -> str:
    """Safely dump JSON data with custom serializer."""
    try:
        return json.dumps(data, default=default, ensure_ascii=False)
    except (TypeError, ValueError):
        return json.dumps({"error": "Unable to serialize data"})


# Initialize logging on module import
setup_logging()