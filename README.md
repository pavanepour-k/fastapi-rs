## Python Architecture

```
fastapi-rs/
├── fastapi/                  # Python interface layer (Goal: 100% API compatible)
│   ├── __init__.py
│   ├── _rust.py              # Python-Rust bridge module
│   ├── applications.py       # (Python) ASGI app lifecycle
│   ├── websockets.py         # (Python) WebSocket support
│   ├── routing.py            # (Python wrapper) Routes to Rust core
│   ├── params.py             # Params subsystem
│   │   ├── __init__.py
│   │   ├── base.py
│   │   ├── cookie.py         # (Python)
│   │   ├── file.py           # (Python)
│   │   ├── form.py           # (Python)
│   │   ├── header.py         # (Python)
│   │   ├── body.py           # (Python wrapper) Routes to Rust params: Request body processing
│   │   ├── path.py           # (Python wrapper) Routes to Rust params: Path param handling
│   │   └── query.py          # (Python wrapper) Routes to Rust params: Query param handling
│   │
│   ├── encoders.py           # (Python wrapper) Routes to Rust serialization
│   ├── utils.py              # (Python wrapper) Routes to Rust utilities
│   ├── security/             # Security subsystem
│   │   ├── __init__.py
│   │   ├── api_key.py        # (Python) API key auth
│   │   └── utils.py          # (Python wrapper) Routes to Rust security: Security implementations
│   │
│   ├── dependencies/         # (Python) Dependency injection
│   ├── openapi/              # (Python) OpenAPI schema generation
│   └── middleware/           # (Python) Middleware implementations
│
└── tests/                    # Verification suite
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

---

## Todo

### fastapi/params.py

> After confirming that the "Dynamic Schema Expansion and Synchronization (Python ↔ Rust)" feature works without any issues, the files/features will be merged into "(Python) API key auth" and "(Python wrapper) Routes to Rust params".