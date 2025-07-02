import dataclasses
import datetime
from collections import defaultdict, deque
from collections.abc import Iterable
from decimal import Decimal
from enum import Enum
from ipaddress import (
    IPv4Address,
    IPv4Interface,
    IPv4Network,
    IPv6Address,
    IPv6Interface,
    IPv6Network,
)
from pathlib import Path, PurePath
from types import GeneratorType
from typing import Any, Callable, Dict, List, Optional, Set, Tuple, Union
from uuid import UUID

from pydantic import BaseModel
from pydantic.json import pydantic_encoder

from . import _rust

__all__ = ["jsonable_encoder"]


# Custom encoder types
ENCODERS_BY_TYPE: Dict[type, Callable[[Any], Any]] = {
    bytes: lambda o: o.decode("utf-8"),
    datetime.date: lambda o: o.isoformat(),
    datetime.datetime: lambda o: o.isoformat(),
    datetime.time: lambda o: o.isoformat(),
    datetime.timedelta: lambda o: o.total_seconds(),
    Decimal: float,
    Enum: lambda o: o.value,
    frozenset: list,
    deque: list,
    GeneratorType: list,
    UUID: str,
    IPv4Address: str,
    IPv4Interface: str,
    IPv4Network: str,
    IPv6Address: str,
    IPv6Interface: str,
    IPv6Network: str,
    Path: str,
    PurePath: str,
}


def jsonable_encoder(
    obj: Any,
    include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
    exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
    by_alias: bool = True,
    exclude_unset: bool = False,
    exclude_defaults: bool = False,
    exclude_none: bool = False,
    custom_encoder: Optional[Dict[type, Callable[[Any], Any]]] = None,
    sqlalchemy_safe: bool = True,
) -> Any:
    """
    Convert a Python object to a JSON-serializable format.
    
    Uses Rust acceleration when available for improved performance.
    """
    # Try Rust implementation first
    if _rust.rust_available() and custom_encoder is None:
        try:
            # For simple cases without complex options, use Rust
            if (
                include is None 
                and exclude is None 
                and not exclude_unset 
                and not exclude_defaults
                and not exclude_none
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
        if "__root__" in obj_dict:
            obj_dict = obj_dict["__root__"]
        return jsonable_encoder(
            obj_dict,
            exclude_none=exclude_none,
            custom_encoder=custom_encoder,
            sqlalchemy_safe=sqlalchemy_safe,
        )
    if dataclasses.is_dataclass(obj):
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
    if isinstance(obj, Enum):
        return obj.value
    if isinstance(obj, PurePath):
        return str(obj)
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
            encoded_key = jsonable_encoder(
                key,
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_none=exclude_none,
                custom_encoder=custom_encoder,
                sqlalchemy_safe=sqlalchemy_safe,
            )
            encoded_value = jsonable_encoder(
                obj[key],
                by_alias=by_alias,
                exclude_unset=exclude_unset,
                exclude_none=exclude_none,
                custom_encoder=custom_encoder,
                sqlalchemy_safe=sqlalchemy_safe,
            )
            if not (exclude_none and encoded_value is None):
                encoded_dict[encoded_key] = encoded_value
        return encoded_dict
    if isinstance(obj, (list, set, frozenset, GeneratorType, tuple, deque)):
        encoded_list = []
        for item in obj:
            encoded_list.append(
                jsonable_encoder(
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
            )
        return encoded_list

    if type(obj) in ENCODERS_BY_TYPE:
        return ENCODERS_BY_TYPE[type(obj)](obj)
    for encoder_type, encoder_func in ENCODERS_BY_TYPE.items():
        if isinstance(obj, encoder_type):
            return encoder_func(obj)

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

    errors = []
    try:
        return pydantic_encoder(obj)
    except Exception as e:
        errors.append(e)
        raise ValueError(errors) from e