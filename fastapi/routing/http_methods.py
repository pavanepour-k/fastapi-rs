from typing import Any, Callable, Dict, List, Optional, Sequence, Set, Type, Union, cast

from starlette.responses import JSONResponse, Response
from starlette.routing import BaseRoute
from starlette.types import ASGIApp

from .. import _rust
from ..datastructures import Default
from ..params import Depends
from ..types import DecoratedCallable
from ..utils import generate_unique_id
from .api_route import APIRoute
from .http_methods import HTTPMethodsMixin
from .websocket import APIWebSocketRoute


class APIRouter(HTTPMethodsMixin):
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
            include_in_schema=include_in_schema and self.include_in_schema,
            response_class=response_class or cast(Type[Response], self.default_response_class),
            dependency_overrides_provider=self.dependency_overrides_provider,
            callbacks=callbacks,
            openapi_extra=openapi_extra,
            generate_unique_id_function=generate_unique_id_function or cast(Callable[["APIRoute"], str], self.generate_unique_id_function),
        )
        self.routes.append(route)
        
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
    
    def include_router(
        self,
        router: "APIRouter",
        *,
        prefix: str = "",
        tags: Optional[List[str]] = None,
        dependencies: Optional[Sequence[Depends]] = None,
        responses: Optional[Dict[Union[int, str], Dict[str, Any]]] = None,
    ) -> None:
        if prefix:
            assert not prefix.endswith("/"), "A route prefix must not end with '/'"
            assert prefix.startswith("/"), "A route prefix must start with '/'"
        
        for route in router.routes:
            if isinstance(route, (APIRoute, APIWebSocketRoute)):
                route.path = prefix + route.path
                route.path_format = prefix + route.path_format
                
                if tags:
                    route.tags = list(set(route.tags + tags))
                
                if dependencies:
                    route.dependencies = list(route.dependencies) + list(dependencies)
                
                if responses and isinstance(route, APIRoute):
                    route.responses = {**responses, **route.responses}
            
            self.routes.append(route)
        
        self.on_startup.extend(router.on_startup)
        self.on_shutdown.extend(router.on_shutdown)
    
    def add_api_websocket_route(
        self,
        path: str,
        endpoint: Callable[..., Any],
        name: Optional[str] = None,
        *,
        dependencies: Optional[Sequence[Depends]] = None,
    ) -> None:
        route = APIWebSocketRoute(
            self.prefix + path,
            endpoint,
            name=name,
            dependencies=self.dependencies + (list(dependencies) if dependencies else []),
        )
        self.routes.append(route)
    
    def websocket(
        self,
        path: str,
        name: Optional[str] = None,
        *,
        dependencies: Optional[Sequence[Depends]] = None,
    ) -> Callable[[DecoratedCallable], DecoratedCallable]:
        def decorator(func: DecoratedCallable) -> DecoratedCallable:
            self.add_api_websocket_route(
                path,
                func,
                name=name,
                dependencies=dependencies,
            )
            return func
        return decorator