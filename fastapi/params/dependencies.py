from typing import Any, Callable, List, Optional


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