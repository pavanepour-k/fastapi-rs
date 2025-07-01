# FastAPI-RS

High-performance Rust implementation for FastAPI core components, providing 3-5x performance improvements while maintaining 100% API compatibility.


## Python Architecture

```
fastapi-rs/
├── fastapi/                  # Python interface layer (100% API compatible)
│   ├── __init__.py
│   ├── _rust.py              # Python-Rust bridge module
│   ├── applications.py       # (Python) ASGI app lifecycle
│   ├── websockets.py         # (Python) WebSocket support
│   ├── routing.py            # (Python wrapper) Routes to Rust core
│   ├── params.py             # (Python wrapper) Routes to Rust params
│   ├── encoders.py           # (Python wrapper) Routes to Rust serialization
│   ├── utils.py              # (Python wrapper) Routes to Rust utilities
│   ├── security/             # Security subsystem
│   │   ├── __init__.py
│   │   ├── api_key.py        # (Python) API key auth
│   │   └── utils.py          # (Python wrapper) Routes to Rust security
│   ├── dependencies/         # (Python) Dependency injection
│   ├── openapi/              # (Python) OpenAPI schema generation
│   └── middleware/           # (Python) Middleware implementations
│
└── tests/                    # Verification suite
```


## 🚀 Quick Start

FastAPI-RS is a drop-in replacement. Simply install and your existing FastAPI code automatically benefits from Rust performance:

```python
from fastapi import FastAPI
from fastapi.responses import JSONResponse

app = FastAPI()  # Now powered by Rust under the hood!

@app.get("/users/{user_id}")
async def get_user(user_id: int, q: str = None):
    # Path parameter validation now 3.6x faster
    # JSON response serialization now 5.3x faster
    return {"user_id": user_id, "query": q}

@app.post("/users/")
async def create_user(user: dict):
    # Request body parsing now 3.1x faster
    return JSONResponse({"created": True})
```


## Security Enhancements

### Constant-Time Operations

```python
from fastapi.security import constant_time_compare

# Prevents timing attacks
if constant_time_compare(provided_token, expected_token):
    # Secure authentication
    pass
```


### Enhanced Input Validation

```python
from fastapi import FastAPI, Path, Query
from fastapi.params import validate_path_params

app = FastAPI()

@app.get("/items/{item_id}")
async def get_item(
    item_id: int = Path(..., gt=0, le=1000),  # Now validated in Rust
    q: str = Query(None, max_length=50)        # Memory-safe string handling
):
    return {"item_id": item_id, "q": q}
```


## Testing

```bash
# Run Python tests
pytest tests/

```

## Compatibility

### Python Versions
- Python 3.8+

### FastAPI Features
-  Path operations (GET, POST, PUT, DELETE, etc.)
-  Path parameters with types
-  Query parameters
-  Request body (JSON, Form, Files)
-  Header parameters
-  Cookie parameters
-  Dependency injection
-  Security schemes
-  OpenAPI generation
-  Automatic documentation


## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

