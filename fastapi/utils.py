"""
FastAPI utilities module with Rust acceleration.

Provides utility functions with automatic fallback to pure Python
implementation when Rust extensions are unavailable.
"""
import inspect
import re
from dataclasses import is_dataclass
from enum import Enum
from typing import Any, Dict, List, Optional, Set, Type, Union, cast, get_type_hints

from pydantic import BaseModel
from pydantic.fields import FieldInfo, ModelField
from typing_extensions import Literal, get_args, get_origin

from . import _rust
from .datastructures import DefaultPlaceholder


def is_body_allowed_for_status_code(status_code: Union[int, str, None]) -> bool:
    """Check if a body is allowed for the given status code."""
    if status_code is None:
        return True
    
    # Convert to int if string
    if isinstance(status_code, str):
        try:
            status_code = int(status_code)
        except ValueError:
            return True
    
    # No body allowed for these status codes
    return status_code not in {204, 205, 304}


def generate_unique_id(route: Any) -> str:
    """
    Generate a unique ID for a route.
    
    Uses Rust acceleration when available for improved performance.
    """
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            operation_id = _rust.RustUtils.generate_unique_id(
                route.name or route.endpoint.__name__,
                ",".join(sorted(route.methods)),
                route.path_format,
            )
            return operation_id
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    operation_id = f"{route.name or route.endpoint.__name__}_{route.path_format}"
    operation_id = re.sub(r'[^0-9a-zA-Z_]', '_', operation_id)
    
    # Add method prefix if multiple methods
    if len(route.methods) == 1:
        method = next(iter(route.methods)).lower()
        operation_id = f"{method}_{operation_id}"
    
    return operation_id


def deep_dict_update(original: Dict[Any, Any], update: Dict[Any, Any]) -> Dict[Any, Any]:
    """Recursively update a dictionary."""
    updated = original.copy()
    
    for key, value in update.items():
        if (
            key in updated 
            and isinstance(updated[key], dict) 
            and isinstance(value, dict)
        ):
            updated[key] = deep_dict_update(updated[key], value)
        else:
            updated[key] = value
    
    return updated


def get_value_or_default(
    first_item: Union[DefaultPlaceholder, Any],
    *default_items: Any,
) -> Any:
    """Get the first non-default value."""
    if not isinstance(first_item, DefaultPlaceholder):
        return first_item
    
    for item in default_items:
        if not isinstance(item, DefaultPlaceholder):
            return item
    
    return first_item


def is_scalar_field(field: ModelField) -> bool:
    """Check if a field is a scalar type."""
    field_type = field.type_
    
    # Handle Optional types
    if get_origin(field_type) is Union:
        args = get_args(field_type)
        if type(None) in args:
            # This is Optional[T], check T
            non_none_args = [arg for arg in args if arg is not type(None)]
            if len(non_none_args) == 1:
                field_type = non_none_args[0]
    
    # Check if it's a basic scalar type
    if field_type in (str, int, float, bool, bytes):
        return True
    
    # Check for Enum
    if inspect.isclass(field_type) and issubclass(field_type, Enum):
        return True
    
    # Check for datetime types
    try:
        import datetime
        if field_type in (datetime.datetime, datetime.date, datetime.time, datetime.timedelta):
            return True
    except ImportError:
        pass
    
    # Check for UUID
    try:
        import uuid
        if field_type is uuid.UUID:
            return True
    except ImportError:
        pass
    
    # Check for Path
    try:
        from pathlib import Path
        if field_type is Path:
            return True
    except ImportError:
        pass
    
    return False


def is_scalar_sequence_field(field: ModelField) -> bool:
    """Check if a field is a sequence of scalar types."""
    field_type = field.type_
    origin = get_origin(field_type)
    
    # Check if it's a sequence type
    if origin in (list, List, set, Set, tuple):
        args = get_args(field_type)
        if args:
            # Check if all arguments are scalar
            return all(
                is_scalar_field(type('', (), {'type_': arg})())  # Create a dummy field
                for arg in args
                if arg is not type(None)
            )
    
    return False


def get_path_param_names(path: str) -> List[str]:
    """Extract parameter names from a path string."""
    return re.findall(r"\{(\w+)(?:[^}]*)?\}", path)


def create_response_field(
    name: str,
    type_: Type[Any],
    default: Any = None,
    alias: Optional[str] = None,
    required: bool = True,
    field_info: Optional[FieldInfo] = None,
) -> ModelField:
    """Create a response field for serialization."""
    if field_info is None:
        field_info = FieldInfo(default=default, alias=alias)
    
    return ModelField(
        name=name,
        type_=type_,
        required=required,
        field_info=field_info,
    )


def parse_content_type(content_type: str) -> tuple:
    """
    Parse content type header.
    
    Uses Rust acceleration when available for improved performance.
    """
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            return _rust.RustUtils.parse_content_type(content_type)
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    parts = content_type.split(";")
    media_type = parts[0].strip()
    params = {}
    
    for part in parts[1:]:
        if "=" in part:
            key, value = part.split("=", 1)
            params[key.strip()] = value.strip().strip('"')
    
    return media_type, params


def get_openapi_operation_metadata(
    *,
    method: str,
    path: str,
    response_model: Optional[Type[Any]],
    status_code: Optional[int],
    tags: Optional[List[str]],
    summary: Optional[str],
    description: Optional[str],
    response_description: str,
    deprecated: Optional[bool],
    operation_id: Optional[str],
) -> Dict[str, Any]:
    """Get OpenAPI operation metadata."""
    operation: Dict[str, Any] = {}
    
    if tags:
        operation["tags"] = tags
    if summary:
        operation["summary"] = summary
    if description:
        operation["description"] = description
    if deprecated:
        operation["deprecated"] = deprecated
    if operation_id:
        operation["operationId"] = operation_id
    
    # Add response info
    status_code_str = str(status_code or 200)
    operation["responses"] = {
        status_code_str: {
            "description": response_description,
        }
    }
    
    return operation


def is_pydantic_model(obj: Any) -> bool:
    """Check if an object is a Pydantic model."""
    try:
        return isinstance(obj, type) and issubclass(obj, BaseModel)
    except TypeError:
        return False


def is_dataclass_instance(obj: Any) -> bool:
    """Check if an object is a dataclass instance."""
    return is_dataclass(obj) and not isinstance(obj, type)


def get_flat_models_from_fields(
    fields: List[ModelField],
    known_models: Set[Type[BaseModel]],
) -> Set[Type[BaseModel]]:
    """Get all Pydantic models from a list of fields."""
    models: Set[Type[BaseModel]] = set()
    
    for field in fields:
        field_models = get_flat_models_from_field(field, known_models)
        models.update(field_models)
    
    return models


def get_flat_models_from_field(
    field: ModelField,
    known_models: Set[Type[BaseModel]],
) -> Set[Type[BaseModel]]:
    """Get all Pydantic models from a field."""
    models: Set[Type[BaseModel]] = set()
    
    # Get the field type
    field_type = field.type_
    
    # Handle container types
    origin = get_origin(field_type)
    if origin is not None:
        # Handle Union types (including Optional)
        if origin is Union:
            for arg in get_args(field_type):
                if arg is not type(None) and is_pydantic_model(arg):
                    models.add(arg)
        # Handle List, Set, Tuple, etc.
        elif origin in (list, List, set, Set, tuple):
            for arg in get_args(field_type):
                if is_pydantic_model(arg):
                    models.add(arg)
        # Handle Dict
        elif origin in (dict, Dict):
            args = get_args(field_type)
            if len(args) >= 2 and is_pydantic_model(args[1]):
                models.add(args[1])
    
    # Handle direct Pydantic model
    elif is_pydantic_model(field_type):
        models.add(field_type)
    
    # Recursively get models from sub-models
    new_models = models - known_models
    known_models.update(new_models)
    
    for model in new_models:
        for sub_field in model.__fields__.values():
            sub_models = get_flat_models_from_field(sub_field, known_models)
            models.update(sub_models)
    
    return models


def get_typed_signature(call: Any) -> inspect.Signature:
    """Get typed signature of a callable."""
    signature = inspect.signature(call)
    globalns = getattr(call, "__globals__", {})
    
    typed_params = []
    for param in signature.parameters.values():
        annotation = param.annotation
        if isinstance(annotation, str):
            annotation = eval(annotation, globalns)
        
        typed_params.append(
            inspect.Parameter(
                name=param.name,
                kind=param.kind,
                default=param.default,
                annotation=annotation,
            )
        )
    
    typed_signature = inspect.Signature(typed_params)
    return typed_signature


def get_typed_return_annotation(call: Any) -> Any:
    """Get typed return annotation of a callable."""
    signature = inspect.signature(call)
    annotation = signature.return_annotation
    
    if annotation is inspect.Signature.empty:
        return None
    
    globalns = getattr(call, "__globals__", {})
    if isinstance(annotation, str):
        annotation = eval(annotation, globalns)
    
    return annotation


def create_cloned_field(
    field: ModelField,
    *,
    name: Optional[str] = None,
    default: Any = None,
    required: Optional[bool] = None,
    alias: Optional[str] = None,
    field_info: Optional[FieldInfo] = None,
) -> ModelField:
    """Create a cloned field with modifications."""
    return ModelField(
        name=name or field.name,
        type_=field.type_,
        required=required if required is not None else field.required,
        default=default if default is not None else field.default,
        alias=alias or field.alias,
        field_info=field_info or field.field_info,
    )