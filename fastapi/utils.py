import inspect
import re
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, Sequence, Set, Type, Union
from weakref import WeakKeyDictionary

from pydantic import BaseModel, create_model
from pydantic.fields import FieldInfo, ModelField
from starlette.datastructures import URL
from starlette.routing import get_name

from . import _rust
from .datastructures import Default, DefaultPlaceholder

if TYPE_CHECKING:
    from .routing import APIRoute

__all__ = [
    "generate_unique_id",
    "deep_dict_update",
    "get_value_or_default",
    "match_pydantic_error_url",
    "is_body_allowed_for_status_code",
    "get_path_param_names",
    "create_model_field",
    "create_response_field",
    "is_scalar_field",
    "is_scalar_sequence_field",
    "is_sequence_field",
    "is_bytes_field",
    "is_bytes_sequence_field",
]


# Cached compiled regex
_path_param_regex = re.compile(r"\{(\w+)\}")


def generate_unique_id(route: "APIRoute") -> str:
    """
    Generate a unique ID for a route, using Rust acceleration if available.
    """
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust", "generate_unique_id"
            )(route.name, route.methods[0] if route.methods else "GET", route.path)
        except Exception:
            pass
    
    # Python fallback
    operation_id = f"{route.name}_{route.path}"
    operation_id = re.sub(r"[^a-zA-Z0-9_]", "_", operation_id)
    operation_id = operation_id.lower()
    return operation_id


def deep_dict_update(mapping: Dict[Any, Any], *updating_mappings: Dict[Any, Any]) -> Dict[Any, Any]:
    """Deep update a dict with another dict."""
    updated_mapping = mapping.copy()
    for updating_mapping in updating_mappings:
        for k, v in updating_mapping.items():
            if (
                k in updated_mapping
                and isinstance(updated_mapping[k], dict)
                and isinstance(v, dict)
            ):
                updated_mapping[k] = deep_dict_update(updated_mapping[k], v)
            else:
                updated_mapping[k] = v
    return updated_mapping


def get_value_or_default(
    first_item: Union[DefaultPlaceholder, Any],
    *default_items: Union[DefaultPlaceholder, Any],
) -> Any:
    """Get the first non-default value."""
    for item in (first_item, *default_items):
        if not isinstance(item, DefaultPlaceholder):
            return item
    return first_item


def match_pydantic_error_url(error: Any) -> Optional[str]:
    """Extract URL from Pydantic error."""
    if hasattr(error, "url"):
        return str(error.url)
    return None


def is_body_allowed_for_status_code(status_code: Union[int, str, None]) -> bool:
    """Check if a body is allowed for the given status code."""
    if status_code is None:
        return True
    if isinstance(status_code, str) and status_code == "default":
        return True
    current_status_code = int(status_code)
    return current_status_code not in {204, 304}


def get_path_param_names(path: str) -> Set[str]:
    """Extract parameter names from a path string."""
    return {match.group(1) for match in _path_param_regex.finditer(path)}


def create_model_field(
    name: str,
    type_: Type[Any],
    default: Any = None,
    alias: Optional[str] = None,
    required: bool = True,
    field_info: Optional[FieldInfo] = None,
) -> ModelField:
    """Create a Pydantic ModelField."""
    from pydantic.fields import ModelField
    
    if field_info is None:
        field_info = FieldInfo(default=default)
    
    return ModelField(
        name=name,
        type_=type_,
        default=default,
        alias=alias,
        required=required,
        model_config=BaseModel.model_config,
        field_info=field_info,
    )


def create_response_field(
    name: str,
    type_: Type[Any],
    default: Any = None,
    alias: Optional[str] = None,
    required: bool = True,
    field_info: Optional[FieldInfo] = None,
) -> ModelField:
    """Create a response field."""
    return create_model_field(
        name=name,
        type_=type_,
        default=default,
        alias=alias,
        required=required,
        field_info=field_info,
    )


def is_scalar_field(field: ModelField) -> bool:
    """Check if a field is a scalar type."""
    from . import params
    
    field_info = field.field_info
    if not (
        isinstance(field_info, params.Path)
        or isinstance(field_info, params.Query)
        or isinstance(field_info, params.Header)
        or isinstance(field_info, params.Cookie)
    ):
        return False
    
    if field.shape != 1:  # Not a single value
        return False
    
    return True


def is_scalar_sequence_field(field: ModelField) -> bool:
    """Check if a field is a sequence of scalar types."""
    from . import params
    
    field_info = field.field_info
    if not (
        isinstance(field_info, params.Query)
        or isinstance(field_info, params.Header)
        or isinstance(field_info, params.Cookie)
    ):
        return False
    
    if field.shape == 2:  # List shape
        return True
    
    return False


def is_sequence_field(field: ModelField) -> bool:
    """Check if a field is a sequence type."""
    return field.shape == 2  # List shape


def is_bytes_field(field: ModelField) -> bool:
    """Check if a field is a bytes type."""
    return field.type_ is bytes


def is_bytes_sequence_field(field: ModelField) -> bool:
    """Check if a field is a sequence of bytes."""
    if field.shape != 2:  # Not a list
        return False
    
    # Check if the inner type is bytes
    inner_type = getattr(field.type_, "__args__", [None])[0]
    return inner_type is bytes


def lenient_issubclass(cls: Any, class_or_tuple: Union[Type[Any], tuple[Type[Any], ...]]) -> bool:
    """Check if cls is a subclass of class_or_tuple, handling non-class types gracefully."""
    try:
        return isinstance(cls, type) and issubclass(cls, class_or_tuple)
    except TypeError:
        return False


def get_typed_signature(call: Callable[..., Any]) -> inspect.Signature:
    """Get typed signature of a callable."""
    import inspect
    from typing import get_type_hints
    
    signature = inspect.signature(call)
    try:
        typed_params = get_type_hints(call, include_extras=True)
    except (NameError, AttributeError):
        typed_params = {}
    
    typed_params_list = []
    for param in signature.parameters.values():
        param_type = typed_params.get(param.name, param.annotation)
        typed_params_list.append(
            param.replace(annotation=param_type)
        )
    
    return inspect.Signature(typed_params_list, return_annotation=signature.return_annotation)


def get_typed_annotation(annotation: Any, globalns: Dict[str, Any]) -> Any:
    """Get typed annotation, handling string annotations."""
    if isinstance(annotation, str):
        annotation = eval(annotation, globalns)
    return annotation


def get_typed_return_annotation(call: Callable[..., Any]) -> Any:
    """Get typed return annotation of a callable."""
    import inspect
    from typing import get_type_hints
    
    try:
        hints = get_type_hints(call, include_extras=True)
        return hints.get("return", inspect.Signature.empty)
    except (NameError, AttributeError):
        return inspect.signature(call).return_annotation


def generate_operation_id_for_path(*, name: str, path: str, method: str) -> str:
    """Generate an operation ID for a given path and method."""
    if _rust.rust_available():
        try:
            return _rust.get_rust_function(
                "_fastapi_rust", "generate_unique_id"
            )(name, method, path)
        except Exception:
            pass
    
    # Python fallback
    operation_id = f"{name}_{path}_{method}"
    operation_id = re.sub(r"[^a-zA-Z0-9_]", "_", operation_id)
    return operation_id.lower()