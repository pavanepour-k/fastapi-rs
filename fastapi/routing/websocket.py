import re
from typing import Any, Callable, Dict, Optional, Sequence

from starlette.routing import BaseRoute

from ..dependencies.models import Dependant
from ..dependencies.utils import get_dependant, solve_dependencies
from ..exceptions import WebSocketRequestValidationError
from ..params import Depends


class APIWebSocketRoute(BaseRoute):
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
        
        self.dependant = get_dependant(path=self.path, call=self.endpoint)
        
        for depends in self.dependencies:
            self.dependant.dependencies.append(
                get_dependant(path=self.path, call=depends.dependency, use_cache=depends.use_cache)
            )
        
        self.path_regex = self._compile_path(path)
        self.path_format = path
    
    def _compile_path(self, path: str) -> str:
        path_regex = "^"
        for match in re.finditer(r"\{(\w+)(?::([^\}]+))?\}", path):
            param_name = match.group(1)
            param_pattern = match.group(2) or r"[^/]+"
            path_regex = path_regex[:match.start()] + f"(?P<{param_name}>{param_pattern})" + path_regex[match.end():]
        path_regex += "$"
        return path_regex
    
    async def __call__(self, scope: Dict[str, Any], receive: Callable, send: Callable) -> None:
        from ..websockets import WebSocket
        
        websocket = WebSocket(scope, receive=receive, send=send)
        
        try:
            values, errors, _, _, _ = await solve_dependencies(
                request=websocket,
                dependant=self.dependant,
                dependency_overrides_provider=None,
            )
            
            if errors:
                raise WebSocketRequestValidationError(errors)
            
            await self.endpoint(**values)
            
        except Exception as e:
            await websocket.close(code=1011)
            raise