from typing import Any, Optional

from .base import Param, ParamTypes
from .body import Body
from .cookie import Cookie
from .file import File
from .form import Form
from .header import Header
from .path import Path
from .query import Query


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