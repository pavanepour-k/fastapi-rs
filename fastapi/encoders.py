"""
FastAPI encoders module with Rust acceleration.

Provides high-performance JSON encoding with automatic fallback
to pure Python implementation when Rust extensions are unavailable.
"""
import dataclasses
import datetime
import decimal
import enum
import ipaddress
import pathlib
import re
import types
import uuid
from collections import defaultdict, deque
from typing import Any, Callable, Dict, List, Optional, Set, Tuple, Union

from pydantic import BaseModel
from pydantic.json import pydantic_encoder

from . import _rust


def jsonable_encoder(
    obj: Any,
    *,
    include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
    exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
    by_alias: bool = True,
    exclude_unset: bool = False,
    exclude_defaults: bool = False,
    exclude_none: bool = False,
    custom_encoder: Optional[Dict[Any, Callable[[Any], Any]]] = None,
    sqlalchemy_safe: bool = True,
) -> Any:
    """
    Convert any object to a JSON-serializable object.
    
    Uses Rust acceleration when available for improved performance.
    """
    # Try Rust implementation first for simple cases
    if _rust.rust_available() and custom_encoder is None:
        try:
            # For simple cases without complex options, use Rust
            if (
                include is None 
                and exclude is None 
                and not exclude_unset 
                and not exclude_defaults
                and not exclude_none
                and sqlalchemy_safe
            ):
                return _rust.RustSerialization.jsonable_encoder(obj)
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    custom_encoder = custom_encoder or {}
    if custom_encoder:
        if type(obj) in custom_encoder:
            return custom_encoder[type(obj)](obj)
        for encoder_type, encoder_func in custom_encoder.items():
            if isinstance(obj, encoder_type):
                return encoder_func(obj)

    if include is not None and not isinstance(include, (set, dict)):
        include = set(include)
    if exclude is not None and not isinstance(exclude, (set, dict)):
        exclude = set(exclude)

    if isinstance(obj, BaseModel):
        # Handle Pydantic v1 and v2
        if hasattr(obj, "model_dump"):  # Pydantic v2
            obj_dict = obj.model_dump(
                mode="json",
                include=include,
                exclude=exclude,
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_defaults=exclude_defaults,
                exclude_none=exclude_none,
            )
        else:  # Pydantic v1
            obj_dict = obj.dict(
                include=include,
                exclude=exclude,
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_defaults=exclude_defaults,
                exclude_none=exclude_none,
            )
        if custom_encoder:
            obj_dict = {k: jsonable_encoder(v, custom_encoder=custom_encoder) for k, v in obj_dict.items()}
        return obj_dict
    
    if dataclasses.is_dataclass(obj) and not isinstance(obj, type):
        obj_dict = dataclasses.asdict(obj)
        return jsonable_encoder(
            obj_dict,
            include=include,
            exclude=exclude,
            by_alias=by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            custom_encoder=custom_encoder,
            sqlalchemy_safe=sqlalchemy_safe,
        )
    
    if isinstance(obj, enum.Enum):
        return obj.value
    
    if isinstance(obj, (str, int, float, type(None))):
        return obj
    
    if isinstance(obj, dict):
        encoded_dict = {}
        allowed_keys = set(obj.keys())
        if include is not None:
            allowed_keys &= set(include)
        if exclude is not None:
            allowed_keys -= set(exclude)
        
        for key in allowed_keys:
            if key in obj:
                encoded_value = jsonable_encoder(
                    obj[key],
                    include=include.get(key) if isinstance(include, dict) else None,
                    exclude=exclude.get(key) if isinstance(exclude, dict) else None,
                    by_alias=by_alias,
                    exclude_unset=exclude_unset,
                    exclude_defaults=exclude_defaults,
                    exclude_none=exclude_none,
                    custom_encoder=custom_encoder,
                    sqlalchemy_safe=sqlalchemy_safe,
                )
                if encoded_value is not None or not exclude_none:
                    encoded_dict[key] = encoded_value
        return encoded_dict
    
    if isinstance(obj, (list, set, frozenset, deque, tuple)):
        encoded_list = []
        for item in obj:
            encoded_item = jsonable_encoder(
                item,
                include=include,
                exclude=exclude,
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_defaults=exclude_defaults,
                exclude_none=exclude_none,
                custom_encoder=custom_encoder,
                sqlalchemy_safe=sqlalchemy_safe,
            )
            encoded_list.append(encoded_item)
        return encoded_list
    
    # Handle datetime types
    if isinstance(obj, datetime.datetime):
        return obj.isoformat()
    if isinstance(obj, datetime.date):
        return obj.isoformat()
    if isinstance(obj, datetime.time):
        return obj.isoformat()
    if isinstance(obj, datetime.timedelta):
        return obj.total_seconds()
    
    # Handle other common types
    if isinstance(obj, decimal.Decimal):
        return float(obj)
    if isinstance(obj, uuid.UUID):
        return str(obj)
    if isinstance(obj, bytes):
        return obj.decode("utf-8")
    if isinstance(obj, (pathlib.Path, pathlib.PurePath)):
        return str(obj)
    if isinstance(obj, (ipaddress.IPv4Address, ipaddress.IPv4Interface, ipaddress.IPv4Network,
                       ipaddress.IPv6Address, ipaddress.IPv6Interface, ipaddress.IPv6Network)):
        return str(obj)
    if isinstance(obj, re.Pattern):
        return obj.pattern
    if isinstance(obj, types.GeneratorType):
        return list(obj)
    
    # SQLAlchemy support
    if sqlalchemy_safe:
        try:
            from sqlalchemy.orm import Query
            if isinstance(obj, Query):
                return list(obj)
        except ImportError:
            pass
        
        # Check for SQLAlchemy models
        if hasattr(obj, "__table__"):
            return jsonable_encoder(
                {key: getattr(obj, key) for key in obj.__table__.columns.keys()},
                include=include,
                exclude=exclude,
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_defaults=exclude_defaults,
                exclude_none=exclude_none,
                custom_encoder=custom_encoder,
                sqlalchemy_safe=False,
            )
    
    # Try to get all attributes
    if hasattr(obj, "__dict__"):
        return jsonable_encoder(
            obj.__dict__,
            include=include,
            exclude=exclude,
            by_alias=by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            custom_encoder=custom_encoder,
            sqlalchemy_safe=sqlalchemy_safe,
        )
    
    # Default to string representation
    try:
        return pydantic_encoder(obj)
    except Exception:
        return str(obj)


def serialize_response(
    content: Any,
    *,
    content_type: Optional[str] = None,
    exclude_unset: bool = False,
    exclude_defaults: bool = False,
    exclude_none: bool = False,
) -> bytes:
    """
    Serialize response content to bytes.
    
    Uses Rust acceleration when available for improved performance.
    """
    # Try Rust implementation first
    if _rust.rust_available():
        try:
            return _rust.RustSerialization.serialize_response(content, content_type)
        except Exception:
            # Fall back to Python implementation
            pass
    
    # Python implementation
    import json
    
    if content_type and "json" in content_type:
        json_content = jsonable_encoder(
            content,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
        )
        return json.dumps(json_content, ensure_ascii=False).encode("utf-8")
    
    # For non-JSON content, convert to string and encode
    if isinstance(content, bytes):
        return content
    if isinstance(content, str):
        return content.encode("utf-8")
    
    # Default to JSON encoding
    json_content = jsonable_encoder(
        content,
        exclude_unset=exclude_unset,
        exclude_defaults=exclude_defaults,
        exclude_none=exclude_none,
    )
    return json.dumps(json_content, ensure_ascii=False).encode("utf-8")