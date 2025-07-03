from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Union

from pydantic.fields import FieldInfo

from .. import _rust


class ParamTypes(str, Enum):
    query = "query"
    header = "header"
    path = "path"
    cookie = "cookie"
    body = "body"
    form = "form"
    file = "file"


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
        
        kwargs = {k: v for k, v in kwargs.items() if v is not None}
        
        if examples is not None:
            kwargs["examples"] = examples
        elif example is not None:
            kwargs["examples"] = [example]
        
        super().__init__(**kwargs)

    def _validate_with_rust(self, value: Any, field_name: str) -> Any:
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
            pass
        
        return value
    
    def _get_validation_schema(self) -> Dict[str, Any]:
        schema = {
            "type": "string",
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