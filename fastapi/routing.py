"""
FastAPI routing module with Rust acceleration.

This module provides the routing functionality for FastAPI with automatic
fallback to pure Python implementation when Rust extensions are unavailable.
"""
import inspect
import re
from enum import Enum
from typing import (
    Any, Callable, Dict, List, Optional, Sequence, Set, Type, Union, 
    cast, get_type_hints
)

from starlette.routing import BaseRoute
from starlette.routing import Mount as Mount  # noqa
from starlette.routing import Route as StarletteRoute
from starlette.responses import JSONResponse, Response
from starlette.types import ASGIApp

from . import _rust
from .datastructures import Default, DefaultPlaceholder
from .dependencies.models import Dependant
from .dependencies.utils import get_dependant, solve_dependencies
from .exceptions import HTTPException, RequestValidationError, WebSocketRequestValidationError
from .types import DecoratedCallable
from .utils import generate_unique_id


class APIRoute(StarletteRoute):
    """
    API route with Rust acceleration for high-performance routing.
    """
    
    def __init__(
        self,
        path: str,
        endpoint: Callable[..., Any],
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        name: Optional[str] = None,
        methods: Optional[Union[Set[str], List[str]]] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        dependency_overrides_provider: Optional[Any] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> None:
        self.path = path
        self.endpoint = endpoint
        self.response_model = response_model
        self.status_code = status_code
        self.tags = tags or []
        self.dependencies = list(dependencies or [])
        self.summary = summary
        self.description = description
        self.response_description = response_description
        self.responses = responses or {}
        self.deprecated = deprecated
        self.name = name
        self.operation_id = operation_id
        self.response_model_include = response_model_include
        self.response_model_exclude = response_model_exclude
        self.response_model_by_alias = response_model_by_alias
        self.response_model_exclude_unset = response_model_exclude_unset
        self.response_model_exclude_defaults = response_model_exclude_defaults
        self.response_model_exclude_none = response_model_exclude_none
        self.include_in_schema = include_in_schema
        self.response_class = response_class
        self.dependency_overrides_provider = dependency_overrides_provider
        self.callbacks = callbacks or []
        self.openapi_extra = openapi_extra
        self.generate_unique_id_function = generate_unique_id_function
        
        # Set methods
        if methods is None:
            methods = ["GET"]
        self.methods = set([method.upper() for method in methods])
        
        # Generate path format for OpenAPI
        self.path_format, self.path_regex, self.param_names = self._compile_path(path)
        
        # Get dependant
        self.dependant = get_dependant(path=self.path, call=self.endpoint)
        
        # Add dependencies
        for depends in self.dependencies:
            self.dependant.dependencies.append(
                get_dependant(path=self.path, call=depends.dependency, use_cache=depends.use_cache)
            )
        
        # Create Rust route if available
        self._rust_route = None
        if _rust.rust_available():
            try:
                self._rust_route = _rust.RustRouting.create_route(
                    path, list(self.methods), name
                )
            except Exception:
                # Fall back to Python implementation
                pass
        
        # Call parent constructor
        super().__init__(
            path=self.path_regex,
            endpoint=self._handle_request,
            methods=list(self.methods),
            name=self.name,
        )
    
    def _compile_path(self, path: str) -> tuple:
        """Compile path with parameter extraction."""
        # Try Rust implementation first
        if _rust.rust_available():
            try:
                path_regex = _rust.get_rust_function("_fastapi_rust", "compile_path_regex")(path)
                # Extract parameter names
                param_names = re.findall(r"\{(\w+)\}", path)
                path_format = path
                return path_format, path_regex, param_names
            except Exception:
                pass
        
        # Python fallback
        path_regex = "^"
        path_format = path
        param_names = []
        
        for match in re.finditer(r"\{(\w+)(?::([^\}]+))?\}", path):
            param_name = match.group(1)
            param_names.append(param_name)
            param_pattern = match.group(2) or r"[^/]+"
            path_regex = path_regex[:match.start()] + f"(?P<{param_name}>{param_pattern})" + path_regex[match.end():]
        
        path_regex += "$"
        return path_format, path_regex, param_names
    
    async def _handle_request(self, request: Any) -> Response:
        """Handle incoming request with dependency injection."""
        try:
            # Solve dependencies
            values, errors, _, _, _ = await solve_dependencies(
                request=request,
                dependant=self.dependant,
                dependency_overrides_provider=self.dependency_overrides_provider,
            )
            
            if errors:
                raise RequestValidationError(errors)
            
            # Call endpoint
            raw_response = await self.endpoint(**values)
            
            # Process response
            if isinstance(raw_response, Response):
                return raw_response
            
            # Use response_class to create response
            response_class = self.response_class
            if isinstance(response_class, DefaultPlaceholder):
                response_class = JSONResponse
            
            # Use Rust serialization if available
            if _rust.rust_available():
                try:
                    content = _rust.RustSerialization.jsonable_encoder(raw_response)
                    return response_class(content=content, status_code=self.status_code or 200)
                except Exception:
                    pass
            
            # Python fallback
            from .encoders import jsonable_encoder
            content = jsonable_encoder(raw_response)
            return response_class(content=content, status_code=self.status_code or 200)
            
        except HTTPException:
            raise
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))
    
    @property
    def path_format(self) -> str:
        """Get the path format for OpenAPI."""
        return self._path_format
    
    @path_format.setter
    def path_format(self, value: str) -> None:
        """Set the path format."""
        self._path_format = value


class APIWebSocketRoute(BaseRoute):
    """WebSocket route handler."""
    
    def __init__(
        self,
        path: str,
        endpoint: Callable[..., Any],
        *,
        name: Optional[str] = None,
        dependencies: Optional[Sequence[Depends]] = None,
    ) -> None:
        self.path = path
        self.endpoint = endpoint
        self.name = name
        self.dependencies = list(dependencies or [])
        
        # Get dependant
        self.dependant = get_dependant(path=self.path, call=self.endpoint)
        
        # Add dependencies
        for depends in self.dependencies:
            self.dependant.dependencies.append(
                get_dependant(path=self.path, call=depends.dependency, use_cache=depends.use_cache)
            )
        
        # Compile path
        self.path_regex = self._compile_path(path)
        self.path_format = path
    
    def _compile_path(self, path: str) -> str:
        """Compile WebSocket path."""
        # Similar to APIRoute but simpler
        path_regex = "^"
        for match in re.finditer(r"\{(\w+)(?::([^\}]+))?\}", path):
            param_name = match.group(1)
            param_pattern = match.group(2) or r"[^/]+"
            path_regex = path_regex[:match.start()] + f"(?P<{param_name}>{param_pattern})" + path_regex[match.end():]
        path_regex += "$"
        return path_regex
    
    async def __call__(self, scope: Dict[str, Any], receive: Callable, send: Callable) -> None:
        """Handle WebSocket connection."""
        from .websockets import WebSocket
        
        websocket = WebSocket(scope, receive=receive, send=send)
        
        try:
            # Solve dependencies
            values, errors, _, _, _ = await solve_dependencies(
                request=websocket,
                dependant=self.dependant,
                dependency_overrides_provider=None,
            )
            
            if errors:
                raise WebSocketRequestValidationError(errors)
            
            # Call endpoint
            await self.endpoint(**values)
            
        except Exception as e:
            await websocket.close(code=1011)
            raise


class APIRouter:
    """
    API Router with support for Rust-accelerated route matching.
    """
    
    def __init__(
        self,
        *,
        prefix: str = "",
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        default_response_class: Type[Response] = Default(JSONResponse),
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        callbacks: Optional[List[BaseRoute]] = None,
        routes: Optional[List[BaseRoute]] = None,
        redirect_slashes: bool = True,
        default: Optional[ASGIApp] = None,
        dependency_overrides_provider: Optional[Any] = None,
        route_class: Type[APIRoute] = APIRoute,
        on_startup: Optional[Sequence[Callable[[], Any]]] = None,
        on_shutdown: Optional[Sequence[Callable[[], Any]]] = None,
        deprecated: Optional[bool] = None,
        include_in_schema: bool = True,
        generate_unique_id_function: Callable[[APIRoute], str] = Default(generate_unique_id),
    ) -> None:
        self.prefix = prefix
        self.tags = tags or []
        self.dependencies = list(dependencies or [])
        self.default_response_class = default_response_class
        self.responses = responses or {}
        self.callbacks = callbacks or []
        self.routes: List[BaseRoute] = list(routes or [])
        self.redirect_slashes = redirect_slashes
        self.default = default
        self.dependency_overrides_provider = dependency_overrides_provider
        self.route_class = route_class
        self.on_startup = list(on_startup or [])
        self.on_shutdown = list(on_shutdown or [])
        self.deprecated = deprecated
        self.include_in_schema = include_in_schema
        self.generate_unique_id_function = generate_unique_id_function
        
        # Rust route cache for performance
        self._rust_routes: Optional[List[Any]] = None
        if _rust.rust_available():
            self._rust_routes = []
    
    def add_api_route(
        self,
        path: str,
        endpoint: Callable[..., Any],
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        methods: Optional[Union[Set[str], List[str]]] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> None:
        """Add an API route to the router."""
        route = self.route_class(
            self.prefix + path,
            endpoint,
            response_model=response_model,
            status_code=status_code,
            tags=self.tags + (list(tags) if tags else []),
            dependencies=self.dependencies + (list(dependencies) if dependencies else []),
            summary=summary,
            description=description,
            response_description=response_description,
            responses={**self.responses, **(responses or {})},
            deprecated=deprecated or self.deprecated,
            name=name,
            methods=methods,
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def post(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a POST route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["POST"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def put(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a PUT route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["PUT"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def patch(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a PATCH route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["PATCH"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def delete(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a DELETE route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["DELETE"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def head(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a HEAD route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["HEAD"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def options(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add an OPTIONS route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["OPTIONS"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def trace(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a TRACE route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["TRACE"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema,
            response_class=response_class,
            name=name,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function,
        )

    def websocket(
        self,
        path: str,
        name: Optional[str] = None,
        *,
        dependencies: Optional[Sequence[Depends]] = None,
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a WebSocket route."""
        def decorator(func: DecoratedCallable) -> DecoratedCallable:
            self.add_api_websocket_route(
                path,
                func,
                name=name,
                dependencies=dependencies,
            )
            return func
        return decorator

    def add_api_websocket_route(
        self,
        path: str,
        endpoint: Callable[..., Any],
        name: Optional[str] = None,
        *,
        dependencies: Optional[Sequence[Depends]] = None,
    ) -> None:
        """Add a WebSocket route to the router."""
        route = APIWebSocketRoute(
            self.prefix + path,
            endpoint,
            name=name,
            dependencies=self.dependencies + (list(dependencies) if dependencies else []),
        )
        self.routes.append(route)

    def include_router(
        self,
        router: "APIRouter",
        *,
        prefix: str = "",
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
    ) -> None:
        """Include another router."""
        if prefix:
            assert not prefix.endswith("/"), "A route prefix must not end with '/'"
            assert prefix.startswith("/"), "A route prefix must start with '/'"
        
        for route in router.routes:
            if isinstance(route, (APIRoute, APIWebSocketRoute)):
                # Update route path
                route.path = prefix + route.path
                route.path_format = prefix + route.path_format
                
                # Update tags
                if tags:
                    route.tags = list(set(route.tags + tags))
                
                # Update dependencies
                if dependencies:
                    route.dependencies = list(route.dependencies) + list(dependencies)
                
                # Update responses
                if responses and isinstance(route, APIRoute):
                    route.responses = {**responses, **route.responses}
            
            self.routes.append(route)
        
        # Merge event handlers
        self.on_startup.extend(router.on_startup)
        self.on_shutdown.extend(router.on_shutdown)


# Import Depends to avoid circular imports
from .params import Depends=response_model_exclude,
            response_model_by_alias=response_model_by_alias,
            response_model_exclude_unset=response_model_exclude_unset,
            response_model_exclude_defaults=response_model_exclude_defaults,
            response_model_exclude_none=response_model_exclude_none,
            include_in_schema=include_in_schema and self.include_in_schema,
            response_class=response_class or cast(Type[Response], self.default_response_class),
            dependency_overrides_provider=self.dependency_overrides_provider,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function or cast(Callable[["APIRoute"], str], self.generate_unique_id_function),
        self.routes.append(route)
        
        # Add to Rust route cache
        if self._rust_routes is not None and hasattr(route, "_rust_route") and route._rust_route:
            self._rust_routes.append(route._rust_route)
    
    def api_route(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        methods: Optional[List[str]] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Decorator for adding API routes."""
        def decorator(func: DecoratedCallable) -> DecoratedCallable:
            self.add_api_route(
                path,
                func,
                response_model=response_model,
                status_code=status_code,
                tags=tags,
                dependencies=dependencies,
                summary=summary,
                description=description,
                response_description=response_description,
                responses=responses,
                deprecated=deprecated,
                methods=methods,
                operation_id=operation_id,
                response_model_include=response_model_include,
                response_model_exclude=response_model_exclude,
                response_model_by_alias=response_model_by_alias,
                response_model_exclude_unset=response_model_exclude_unset,
                response_model_exclude_defaults=response_model_exclude_defaults,
                response_model_exclude_none=response_model_exclude_none,
                include_in_schema=include_in_schema,
                response_class=response_class,
                name=name,
                callbacks=callbacks,
                openapi_extra=openapi_extra,
                generate_unique_id_function=generate_unique_id_function,
            )
            return func
        return decorator
    
    def get(
        self,
        path: str,
        *,
        response_model: Optional[Type[Any]] = None,
        status_code: Optional[int] = None,
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        summary: Optional[str] = None,
        description: Optional[str] = None,
        response_description: str = "Successful Response",
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
        deprecated: Optional[bool] = None,
        operation_id: Optional[str] = None,
        response_model_include: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_exclude: Optional[Union[Set[int], Set[str], Dict[int, Any], Dict[str, Any]]] = None,
        response_model_by_alias: bool = True,
        response_model_exclude_unset: bool = False,
        response_model_exclude_defaults: bool = False,
        response_model_exclude_none: bool = False,
        include_in_schema: bool = True,
        response_class: Type[Response] = Default(JSONResponse),
        name: Optional[str] = None,
        callbacks: Optional[List["APIRoute"]] = None,
        openapi_extra: Optional[Dict[str, Any]] = None,
        generate_unique_id_function: Callable[["APIRoute"], str] = Default(generate_unique_id),
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        """Add a GET route."""
        return self.api_route(
            path=path,
            response_model=response_model,
            status_code=status_code,
            tags=tags,
            dependencies=dependencies,
            summary=summary,
            description=description,
            response_description=response_description,
            responses=responses,
            deprecated=deprecated,
            methods=["GET"],
            operation_id=operation_id,
            response_model_include=response_model_include,
            response_model_exclude
        )