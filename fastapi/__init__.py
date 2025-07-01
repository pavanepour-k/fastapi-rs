"""FastAPI framework, high performance, easy to learn, fast to code, ready for production"""

__version__ = "0.115.14"

from starlette import status as status

from ._rust
from .applications import FastAPI as FastAPI
from .routing
from .params
from .encoders
from .utils
