import os
import yaml
import json
from typing import Any, Dict, Optional, List, Union
from pydantic.fields import FieldInfo
from starlette.datastructures import Headers, QueryParams
from . import _rust
from .datastructures import Default
from .utils import is_scalar_field, is_scalar_sequence_field
import logging

__all__ = [
    "Path", "Query", "Header", "Cookie", "Body", "Form", "File",
    "Param", "ParamTypes", "Depends", "Security",
    "load_params_schema", "get_param_schema", "param_factory"
]

PARAMS_SCHEMA_FILE = os.environ.get("FASTAPI_PARAMS_SCHEMA", "params_schema.yaml")
_loaded_schema: Optional[Dict[str, Any]] = None

def load_params_schema(path: str = PARAMS_SCHEMA_FILE) -> Dict[str, Any]:
    global _loaded_schema
    if _loaded_schema is not None:
        return _loaded_schema
    if path.endswith(".yaml") or path.endswith(".yml"):
        with open(path, "r", encoding="utf-8") as f:
            _loaded_schema = yaml.safe_load(f)
    elif path.endswith(".json"):
        with open(path, "r", encoding="utf-8") as f:
            _loaded_schema = json.load(f)
    else:
        raise RuntimeError("Unsupported schema format: must be .yaml, .yml, or .json")
    return _loaded_schema

def get_param_schema(endpoint: str, param: str) -> Dict[str, Any]:
    schema = load_params_schema()
    try:
        return schema["endpoints"][endpoint]["params"][param]
    except KeyError:
        raise RuntimeError(f"Schema for endpoint={endpoint} param={param} not found.")

class ParamTypes:
    query = "query"
    header = "header"
    path = "path"
    cookie = "cookie"

class RustSchemaConverter:
    FIELD_MAPPING = {
        "min_length": "minLength",
        "max_length": "maxLength",
        "pattern": "pattern",
        "required": "required",
        "enum": "enum",
        "gt": "gt",
        "ge": "ge",
        "lt": "lt",
        "le": "le",
        "multiple_of": "multipleOf",
        "allow_inf_nan": "allowInfNan",
        "max_digits": "maxDigits",
        "decimal_places": "decimalPlaces",
        "strict": "strict",
        "discriminator": "discriminator",
        "examples": "examples",
        "example": "example",
        "deprecated": "deprecated",
        "include_in_schema": "includeInSchema",
        "json_schema_extra": "jsonSchemaExtra"
    }

    @classmethod
    def convert(cls, schema: Dict[str, Any]) -> Dict[str, Any]:
        rust_schema = {}
        for key, value in schema.items():
            mapped_key = cls.FIELD_MAPPING.get(key, key)
            rust_schema[mapped_key] = value
        return rust_schema

def verify_rust_schema_conversion(py_schema: Dict[str, Any], rust_schema: Dict[str, Any]) -> bool:
    converted = RustSchemaConverter.convert(py_schema)
    return converted == rust_schema

def _basic_python_validation(params: Dict[str, Any], schema: Dict[str, Any]) -> Dict[str, Any]:
    validated = {}
    for key, field in schema.items():
        required = field.get("required", False)
        if required and key not in params:
            raise ValueError(f"Missing required field: {key}")
        if key in params:
            value = params[key]
        else:
            if "default" in field:
                value = field["default"]
            else:
                value = None
        if value is None and required:
            raise ValueError(f"Field '{key}' is required and not provided.")
        if "type" in field:
            expected_type = field["type"]
            if expected_type == "string":
                if not isinstance(value, str):
                    raise TypeError(f"Field '{key}' must be string.")
            elif expected_type == "integer":
                if not isinstance(value, int):
                    raise TypeError(f"Field '{key}' must be integer.")
            elif expected_type == "number":
                if not isinstance(value, (int, float)):
                    raise TypeError(f"Field '{key}' must be a number.")
            elif expected_type == "boolean":
                if not isinstance(value, bool):
                    raise TypeError(f"Field '{key}' must be boolean.")
        validated[key] = value
    return validated

def _python_to_rust_value(value: Any) -> Any:
    if value is None:
        return None
    if isinstance(value, bool):
        return bool(value)
    if isinstance(value, int):
        return int(value)
    if isinstance(value, float):
        return float(value)
    if isinstance(value, str):
        return value
    if isinstance(value, (list, tuple)):
        return [_python_to_rust_value(v) for v in value]
    if isinstance(value, dict):
        return {k: _python_to_rust_value(v) for k, v in value.items()}
    return value

def _marshal_dict_for_rust(data: Dict[str, Any]) -> Dict[str, Any]:
    return {k: _python_to_rust_value(v) for k, v in data.items()}

class RustValidationError(Exception):
    def __init__(self, detail: Any, code: str = None, field: str = None, errors: Any = None):
        self.detail = detail
        self.code = code
        self.field = field
        self.errors = errors or []
        super().__init__(self.detail)

    def as_dict(self):
        return {
            "error": "validation_error",
            "detail": self.detail,
            "code": self.code,
            "field": self.field,
            "errors": self.errors
        }

    def as_json(self):
        import json
        return json.dumps(self.as_dict())

def rust_error_to_python_exception(rust_error: Any, context: str = "") -> RustValidationError:
    if isinstance(rust_error, dict):
        detail = rust_error.get("message") or rust_error.get("detail") or str(rust_error)
        code = rust_error.get("code")
        field = rust_error.get("field")
        errors = rust_error.get("errors", [])
    else:
        detail = str(rust_error)
        code = None
        field = None
        errors = []
    detail_with_ctx = f"{context}: {detail}" if context else detail
    return RustValidationError(detail_with_ctx, code=code, field=field, errors=errors)

def rust_exception_json_response(exc: RustValidationError) -> str:
    return exc.as_json()

class Param(FieldInfo):
    in_: ParamTypes
    convert_underscores: bool = True

    def __init__(
        self,
        default: Any = ...,
        *,
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
        self.deprecated = deprecated
        self.include_in_schema = include_in_schema
        self.openapi_examples = openapi_examples
        super().__init__(
            default=default,
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
            discriminator=discriminator,
            strict=strict,
            multiple_of=multiple_of,
            allow_inf_nan=allow_inf_nan,
            max_digits=max_digits,
            decimal_places=decimal_places,
            examples=examples,
            json_schema_extra=json_schema_extra,
            **extra,
        )

    def to_rust_schema(self) -> Dict[str, Any]:
        python_schema = self.__dict__.copy()
        rust_schema = RustSchemaConverter.convert(python_schema)
        return rust_schema

class Form(Param):
    def __init__(
        self,
        default: Any = Default(None),
        *,
        media_type: str = "application/x-www-form-urlencoded",
        **kwargs: Any,
    ):
        super().__init__(default=default, **kwargs)
        self.media_type = media_type

class File(Form):
    def __init__(
        self,
        default: Any = Default(None),
        *,
        media_type: str = "multipart/form-data",
        **kwargs: Any,
    ):
        super().__init__(default=default, media_type=media_type, **kwargs)

class Depends:
    def __init__(
        self,
        dependency: Optional[Any] = None,
        *,
        use_cache: bool = True,
    ):
        self.dependency = dependency
        self.use_cache = use_cache

    def __repr__(self) -> str:
        attr = getattr(self.dependency, "__name__", type(self.dependency).__name__)
        cache = "" if self.use_cache else ", use_cache=False"
        return f"{self.__class__.__name__}({attr}{cache})"

class Security(Depends):
    def __init__(
        self,
        dependency: Optional[Any] = None,
        *,
        scopes: Optional[List[str]] = None,
        use_cache: bool = True,
    ):
        super().__init__(dependency=dependency, use_cache=use_cache)
        self.scopes = scopes or []

def param_factory(
    endpoint: str,
    param: str,
    param_type: Optional[str] = None,
    override: Optional[Dict[str, Any]] = None,
) -> Param:
    """
    Factory to generate Param (or subclass) instances from the central schema.
    Optionally override any value using the override dict.
    """
    schema = get_param_schema(endpoint, param).copy()
    if override:
        schema.update(override)
    default = schema.pop("default", ...)
    type_map = {
        "path": Path,
        "query": Query,
        "header": Header,
        "cookie": Cookie,
        "form": Form,
        "file": File
    }
    cls = type_map.get(param_type or schema.get("in"), Param)
    # Remove unknown keys
    param_init_keys = cls.__init__.__code__.co_varnames
    filtered_schema = {k: v for k, v in schema.items() if k in param_init_keys}
    return cls(default=default, **filtered_schema)

def validate_path_params(params: Dict[str, str], schema: Dict[str, Any]) -> Dict[str, Any]:
    rust_schema = RustSchemaConverter.convert(schema)
    function_name = "validate_path_params"
    rust_params = _marshal_dict_for_rust(params)
    if _rust.rust_available():
        try:
            result = _rust.get_rust_function(
                "_fastapi_rust", function_name
            )(rust_params, rust_schema)
            if result.valid:
                return result.validated_data
            else:
                logging.error(
                    "[%s] Rust validation failed: %s | input=%r schema=%r",
                    function_name, result.errors, rust_params, rust_schema
                )
                raise rust_error_to_python_exception(result.errors, context=function_name)
        except Exception as exc:
            logging.error(
                "[%s] Exception during Rust validation: %r | input=%r schema=%r",
                function_name, exc, rust_params, rust_schema
            )
            raise rust_error_to_python_exception(str(exc), context=function_name) from exc
    logging.error(
        "[%s] Rust extension is not available | input=%r schema=%r",
        function_name, rust_params, rust_schema
    )
    raise RuntimeError(f"Critical: Rust extension for parameter validation is not available ({function_name})")

def validate_query_params(params: QueryParams, schema: Dict[str, Any]) -> Dict[str, Any]:
    rust_schema = RustSchemaConverter.convert(schema)
    param_dict = dict(params)
    rust_params = _marshal_dict_for_rust(param_dict)
    function_name = "validate_query_params"
    if _rust.rust_available():
        try:
            result = _rust.get_rust_function(
                "_fastapi_rust", function_name
            )(rust_params, rust_schema)
            if result.valid:
                return result.validated_data
            else:
                logging.error(
                    "[%s] Rust validation failed: %s | input=%r schema=%r",
                    function_name, result.errors, rust_params, rust_schema
                )
                raise rust_error_to_python_exception(result.errors, context=function_name)
        except Exception as exc:
            logging.error(
                "[%s] Exception during Rust validation: %r | input=%r schema=%r",
                function_name, exc, rust_params, rust_schema
            )
            raise rust_error_to_python_exception(str(exc), context=function_name) from exc
    logging.error(
        "[%s] Rust extension is not available | input=%r schema=%r",
        function_name, rust_params, rust_schema
    )
    raise RuntimeError(f"Critical: Rust extension for parameter validation is not available ({function_name})")

def validate_header_params(headers: Headers, schema: Dict[str, Any]) -> Dict[str, Any]:
    rust_schema = RustSchemaConverter.convert(schema)
    header_dict = dict(headers)
    rust_params = _marshal_dict_for_rust(header_dict)
    function_name = "validate_header_params"
    if _rust.rust_available():
        try:
            result = _rust.get_rust_function(
                "_fastapi_rust", function_name
            )(rust_params, rust_schema)
            if result.valid:
                return result.validated_data
            else:
                logging.error(
                    "[%s] Rust validation failed: %s | input=%r schema=%r",
                    function_name, result.errors, rust_params, rust_schema
                )
                raise rust_error_to_python_exception(result.errors, context=function_name)
        except Exception as exc:
            logging.error(
                "[%s] Exception during Rust validation: %r | input=%r schema=%r",
                function_name, exc, rust_params, rust_schema
            )
            raise rust_error_to_python_exception(str(exc), context=function_name) from exc
    logging.error(
        "[%s] Rust extension is not available | input=%r schema=%r",
        function_name, rust_params, rust_schema
    )
    raise RuntimeError(f"Critical: Rust extension for parameter validation is not available ({function_name})")

class Path(Param):
    in_ = ParamTypes.path

    def __init__(self, default: Any = ..., **kwargs: Any):
        super().__init__(default=default, **kwargs)

class Query(Param):
    in_ = ParamTypes.query

    def __init__(self, default: Any = Default(None), **kwargs: Any):
        super().__init__(default=default, **kwargs)

class Header(Param):
    in_ = ParamTypes.header
    convert_underscores = True

    def __init__(self, default: Any = Default(None), *, convert_underscores: bool = True, **kwargs: Any):
        super().__init__(default=default, **kwargs)
        self.convert_underscores = convert_underscores

class Cookie(Param):
    in_ = ParamTypes.cookie

    def __init__(self, default: Any = Default(None), **kwargs: Any):
        super().__init__(default=default, **kwargs)
