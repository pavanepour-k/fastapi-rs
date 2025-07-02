"""FastAPI framework, high performance, easy to learn, fast to code, ready for production"""

__version__ = "0.115.14"

# Third-party dependencies
from starlette import status as status

# Rust bridge (if enabled)
from . import _rust

# Core application interface
from .applications import FastAPI as FastAPI

# Background & concurrency
from .background import BackgroundTasks as BackgroundTasks
from .concurrency import asynccontextmanager as asynccontextmanager

# Data structures
from .datastructures import (
    Default as Default,
    DefaultPlaceholder as DefaultPlaceholder,
    FormData as FormData,
    Headers as Headers,
    QueryParams as QueryParams,
    State as State,
    UploadFile as UploadFile,
)

# Encoders
from .encoders import jsonable_encoder as jsonable_encoder

# Exception classes
from .exceptions import (
    FastAPIError as FastAPIError,
    HTTPException as HTTPException,
    RequestValidationError as RequestValidationError,
    ResponseValidationError as ResponseValidationError,
    WebSocketException as WebSocketException,
    WebSocketRequestValidationError as WebSocketRequestValidationError,
)

# Dependency/parameter functions
from .param_functions import (
    Body as Body,
    Cookie as Cookie,
    Depends as Depends,
    File as File,
    Form as Form,
    Header as Header,
    Path as Path,
    Query as Query,
    Security as Security,
)

# Params
from .params import Param as Param

# Request/Response objects
from .requests import Request as Request
from .responses import (
    FileResponse as FileResponse,
    HTMLResponse as HTMLResponse,
    JSONResponse as JSONResponse,
    ORJSONResponse as ORJSONResponse,
    PlainTextResponse as PlainTextResponse,
    RedirectResponse as RedirectResponse,
    Response as Response,
    StreamingResponse as StreamingResponse,
    UJSONResponse as UJSONResponse,
)

# Routing
from .routing import (
    APIRoute as APIRoute,
    APIRouter as APIRouter,
    APIWebSocketRoute as APIWebSocketRoute,
)

# Types
from .types import Undefined as Undefined

# WebSocket
from .websockets import (
    WebSocket as WebSocket,
    WebSocketDisconnect as WebSocketDisconnect,
    WebSocketState as WebSocketState,
)
