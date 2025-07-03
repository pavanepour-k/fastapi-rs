import inspect
import json
import os
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Union

import yaml
from pydantic.fields import FieldInfo
from typing_extensions import Literal, get_args, get_origin

from . import _rust
from .datastructures import Default
from .exceptions import FastAPIError
from .utils import is_scalar_field, is_scalar_sequence_field

class ParamTypes(str, Enum):
    query = "query"
    header = "header"
    path = "path"
    cookie = "cookie"
    body = "body"
    form = "form"
    file = "file"


_schema_cache: Optional[Dict[str, Any]] = None
_schema_file_mtime: Optional[float] = None


def load_params_schema(path: Optional[str] = None, force_reload: bool = False) -> Dict[str, Any]:
    """Load parameter schema with hot-reload support."""
    global _schema_cache, _schema_file_mtime
    
    if path is None:
        path = os.environ.get("FASTAPI_PARAMS_SCHEMA", "params_schema.yaml")
    
    if not os.path.exists(path):
        # Return empty schema if file doesn't exist
        return {"endpoints": {}, "definitions": {}}
    
    current_mtime = os.path.getmtime(path)
    
    if not force_reload and _schema_cache is not None and _schema_file_mtime == current_mtime:
        return _schema_cache
    
    try:
        if path.endswith((".yaml", ".yml")):
            with open(path, "r", encoding="utf-8") as f:
                _schema_cache = yaml.safe_load(f) or {"endpoints": {}, "definitions": {}}
        elif path.endswith(".json"):
            with open(path, "r", encoding="utf-8") as f:
                _schema_cache = json.load(f)
        else:
            raise ValueError(f"Unsupported schema format: {path}")
        
        _schema_file_mtime = current_mtime
        return _schema_cache
    except Exception as e:
        # Log error and return empty schema
        import logging
        logging.warning(f"Failed to load params schema from {path}: {e}")
        return {"endpoints": {}, "definitions": {}}


def get_param_schema(endpoint: str, param_name: str) -> Optional[Dict[str, Any]]:
    """Get parameter schema with safe fallback."""
    schema = load_params_schema()
    try:
        return schema["endpoints"][endpoint]["params"][param_name]
    except (KeyError, TypeError):
        return None


class Param(FieldInfo):
    in_: ParamTypes

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        media_type: str = "application/json",
        embed: bool = False,
        **extra: Any,
    ):
        self.media_type = media_type
        self.embed = embed
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Form(Param):
    in_ = ParamTypes.form

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        media_type: str = "application/x-www-form-urlencoded",
        **extra: Any,
    ):
        self.media_type = media_type
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class File(Param):
    in_ = ParamTypes.file

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        media_type: str = "multipart/form-data",
        **extra: Any,
    ):
        self.media_type = media_type
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Depends:
    """
    Declare a dependency for a FastAPI endpoint.
    
    This class is used to declare dependencies that should be called
    to provide a value for a parameter in a FastAPI endpoint.
    """
    
    def __init__(
        self, 
        dependency: Optional[Callable[..., Any]] = None, 
        *, 
        use_cache: bool = True
    ):
        self.dependency = dependency
        self.use_cache = use_cache

    def __repr__(self) -> str:
        attr = getattr(self.dependency, "__name__", type(self.dependency).__name__)
        return f"{self.__class__.__name__}({attr})"


class Security(Depends):
    """
    Declare a security dependency for a FastAPI endpoint.
    
    This is a special case of Depends that is used for security dependencies.
    """
    
    def __init__(
        self,
        dependency: Optional[Callable[..., Any]] = None,
        *,
        scopes: Optional[List[str]] = None,
        use_cache: bool = True,
    ):
        super().__init__(dependency=dependency, use_cache=use_cache)
        self.scopes = scopes or []


def param_factory(
    param_type: ParamTypes,
    default: Any = ...,
    *,
    alias: Optional[str] = None,
    title: Optional[str] = None,
    description: Optional[str] = None,
    **kwargs
) -> Param:
    """
    Factory function to create parameter instances based on type.
    
    This provides a dynamic way to create parameters when the type
    is determined at runtime.
    """
    param_classes = {
        ParamTypes.path: Path,
        ParamTypes.query: Query,
        ParamTypes.header: Header,
        ParamTypes.cookie: Cookie,
        ParamTypes.body: Body,
        ParamTypes.form: Form,
        ParamTypes.file: File,
    }
    
    param_class = param_classes.get(param_type)
    if not param_class:
        raise ValueError(f"Unknown parameter type: {param_type}")
    
    return param_class(
        default=default,
        alias=alias,
        title=title,
        description=description,
        **kwargs
    )
 Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        **extra: Any,
    ):
        self.deprecated = deprecated
        self.include_in_schema = include_in_schema
        self.openapi_examples = openapi_examples
        
        kwargs = dict(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern or regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            json_schema_extra=json_schema_extra,
            **extra,
        )
        
        # Remove None values
        kwargs = {k: v for k, v in kwargs.items() if v is not None}
        
        if examples is not None:
            kwargs["examples"] = examples
        elif example is not None:
            kwargs["examples"] = [example]
        
        super().__init__(**kwargs)

    def _validate_with_rust(self, value: Any, field_name: str) -> Any:
        """Validate parameter using Rust if available."""
        if not _rust.rust_available():
            return value
        
        try:
            if self.in_ == ParamTypes.path:
                result = _rust.RustValidation.validate_path_params(
                    {field_name: value},
                    {field_name: self._get_validation_schema()}
                )
            elif self.in_ == ParamTypes.query:
                result = _rust.RustValidation.validate_query_params(
                    {field_name: value},
                    {field_name: self._get_validation_schema()}
                )
            elif self.in_ == ParamTypes.header:
                result = _rust.RustValidation.validate_header_params(
                    {field_name: value},
                    {field_name: self._get_validation_schema()}
                )
            elif self.in_ == ParamTypes.body:
                result = _rust.RustValidation.validate_body_params(
                    value,
                    self._get_validation_schema()
                )
            else:
                return value
            
            if result and hasattr(result, 'validated_data'):
                return result.validated_data.get(field_name, value)
        except Exception:
            # Fall back to Python validation
            pass
        
        return value
    
    def _get_validation_schema(self) -> Dict[str, Any]:
        """Get validation schema for Rust validator."""
        schema = {
            "type": "string",  # Default type
            "in": self.in_.value,
        }
        
        if self.gt is not None:
            schema["exclusiveMinimum"] = self.gt
        if self.ge is not None:
            schema["minimum"] = self.ge
        if self.lt is not None:
            schema["exclusiveMaximum"] = self.lt
        if self.le is not None:
            schema["maximum"] = self.le
        if self.min_length is not None:
            schema["minLength"] = self.min_length
        if self.max_length is not None:
            schema["maxLength"] = self.max_length
        if hasattr(self, "pattern") and self.pattern:
            schema["pattern"] = self.pattern
        
        return schema


class Path(Param):
    in_ = ParamTypes.path

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        **extra: Any,
    ):
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Query(Param):
    in_ = ParamTypes.query

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        convert_underscores: bool = True,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        **extra: Any,
    ):
        self.convert_underscores = convert_underscores
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Header(Param):
    in_ = ParamTypes.header

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        convert_underscores: bool = True,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        **extra: Any,
    ):
        self.convert_underscores = convert_underscores
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Cookie(Param):
    in_ = ParamTypes.cookie

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description: Optional[str] = None,
        gt: Optional[float] = None,
        ge: Optional[float] = None,
        lt: Optional[float] = None,
        le: Optional[float] = None,
        min_length: Optional[int] = None,
        max_length: Optional[int] = None,
        pattern: Optional[str] = None,
        regex: Optional[str] = None,
        discriminator: Union[str, None] = None,
        strict: Union[bool, None] = None,
        multiple_of: Union[float, None] = None,
        allow_inf_nan: Union[bool, None] = None,
        max_digits: Union[int, None] = None,
        decimal_places: Union[int, None] = None,
        examples: Optional[List[Any]] = None,
        example: Optional[Any] = None,
        openapi_examples: Optional[Dict[str, Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        json_schema_extra: Union[Dict[str, Any], None] = None,
        **extra: Any,
    ):
        super().__init__(
            default=default,
            default_factory=default_factory,
            alias=alias,
            alias_priority=alias_priority,
            validation_alias=validation_alias,
            serialization_alias=serialization_alias,
            title=title,
            description=description,
            gt=gt,
            ge=ge,
            lt=lt,
            le=le,
            min_length=min_length,
            max_length=max_length,
            pattern=pattern,
            regex=regex,
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            example=example,
            openapi_examples=openapi_examples,
            deprecated=deprecated,
            include_in_schema=include_in_schema,
            json_schema_extra=json_schema_extra,
            **extra,
        )


class Body(Param):
    in_ = ParamTypes.body

    def __init__(
        self,
        default: Any = ...,
        *,
        default_factory: Union[Callable[[], Any], None] = None,
        alias: Optional[str] = None,
        alias_priority: Union[int, None] = None,
        validation_alias: Union[str, None] = None,
        serialization_alias: Union[str, None] = None,
        title: Optional[str] = None,
        description:
    )