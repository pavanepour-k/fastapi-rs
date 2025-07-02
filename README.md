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

### params.py

- Schema Dependency

        params_schema.yaml or params_schema.json must exist for correct operation (pay attention to path/format).

        If an endpoint or parameter is missing in the schema, a runtime exception will occur.

- Schema/Class Argument Mapping

        If schema field names differ from the constructor argument names of the Param (and Path/Query, etc.) classes, an exception may be raised or fields may be silently ignored during declaration.

        Fields not present in the schema will be automatically filtered by the factory (which could cause bugs to go unnoticed).

- Parameter Type

        The "in" field must be accurately specified as "query", "path", "header", "cookie", "form", or "file".

- Caching/Reload

        The schema file is read only once at initial load (_loaded_schema caching).

        Changes to the file will not be reflected at runtime; take caution in production/development environments (add hot-reload code if needed).

- Dependency Modules

        Internal modules such as .datastructures, .utils, etc. must exist within the project.

### _rust.py

- Rust Extension File Requirement

        Rust extension files (e.g., _fastapi_rust) must be properly built, distributed, and loadable.

        If missing, a runtime exception will occur (identifiable via exception messages and logging).

- Rust API/Function Name Mismatch

        If Rust function names, argument structures, or return types differ from what Python expects, runtime errors will occur.

        Always synchronize the Python side when updating the Rust extension.

### __init__.py

- Maintaining Public API Paths

        The actual import paths for public modules (such as Param, param_factory, etc.) must exist.

        If internal structure is refactored and import paths change, unintended ImportError may occur.

- Dependency Module Import Errors

        In relative imports (e.g., from .params import ...), errors may occur if the location or name of params.py changes.

        Always verify when packaging or refactoring the directory structure.

### Common/General

- Environment Variable Usage

        If the path is specified using the FASTAPI_PARAMS_SCHEMA environment variable, ensure that the path matches in each deployment environment.

- Differences Between Test and Production

        The schema file, Rust extension, and dependency module paths may differ between production and test environments, so check functionality for each environment.

- Error Handling Policy

        If frequent exceptions occur (such as schema mismatches), make log messages and error details specific to allow clear identification of the cause.

### Other

- Consistency When Extending/Updating the Schema

        When adding new fields, types, or constraints to the schema, you must also update the Python/Rust parser/mapping code.

        If the parsers in the two languages are not kept in sync, real data and validation logic may become inconsistent.

- Caution with Param Constructor Changes

        When upgrading pydantic/FastAPI, constructor arguments for Param/Path/Query, etc. may change, so version-specific management of the schema-to-class mapping code is necessary.
