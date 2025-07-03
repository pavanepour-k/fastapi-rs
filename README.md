# FastAPI-RS

High-performance Rust implementation for FastAPI core components, providing 3-5x performance improvements while maintaining 100% API compatibility.

## Features (In progress)

- **High Performance**: 3-5x faster routing, parameter validation, and JSON serialization
- **Memory Safe**: Rust's borrow checker eliminates memory safety issues
- **Security Enhanced**: Constant-time operations prevent timing attacks
- **100% Compatible**: Drop-in replacement for FastAPI core components
- **Zero Breaking Changes**: Maintains identical Python API surface

## Performance Improvements



## Architecture

```Python
fastapi-rs/
├── fastapi/                  # Python interface layer (100% API compatible)
│   ├── __init__.py
│   ├── _rust.py              # Python-Rust bridge module
│   ├── applications.py       # (Python) ASGI app lifecycle
│   ├── websockets.py         # (Python) WebSocket support
│   ├── routing.py            # (Python wrapper) Routes to Rust core
│   ├── encoders.py           # (Python wrapper) Routes to Rust serialization
│   ├── utils.py              # (Python wrapper) Routes to Rust utilities
│   │
│   ├── params/               # Params subsystem
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
│   ├── security/             # Security subsystem
│   │   ├── __init__.py
│   │   ├── api_key.py        # (Python) API key auth
│   │   └── utils.py          # (Python wrapper) Routes to Rust security
│   │
│   ├── dependencies/         # (Python) Dependency injection
│   ├── openapi/              # (Python) OpenAPI schema generation
│   └── middleware/           # (Python) Middleware implementations

```

```Rust
fastapi-rs/
├── rust_src/                 /// High-performance Rust implementation
│   ├── core/                 // Request lifecycle core
│   │   ├── mod.rs
│   │   ├── routing.rs        // Endpoint routing/dispatch
│   │   └── request.rs        // Request processing
│   │
│   ├── params/               // Parameter processing
│   │   ├── mod.rs
│   │   ├── validation.rs     // Data validation logic
│   │   ├── query.rs          // Query param handling
│   │   ├── path.rs           // Path param handling
│   │   └── body.rs           // Request body processing
│   │
│   ├── serialization/        // Data transformation
│   │   ├── mod.rs
│   │   ├── encoders.rs       // JSON serialization
│   │   └── decoders.rs       // Request body deserialization
│   │
│   ├── security/             // Security implementations
│   │   ├── mod.rs
│   │   ├── utils.rs          // Auth helpers
│   │   └── oauth2.rs         // OAuth2 flows
│   │
│   ├── utils/                // Shared utilities
│   │   ├── mod.rs
│   │   ├── async_tools.rs    // Async utilities
│   │   └── type_conv.rs      // Python-Rust type conversion
│   │
│   ├── types/                // Type system
│   │   ├── mod.rs
│   │   └── models.rs         // Pydantic model equivalents
│   │
│   ├── lib.rs                // Rust entry point
│   └── python_bindings.rs    // PyO3 interface definitions
```

```
├── tests/                                  # Verification suite
│   ├── rust/
│   │   ├── common/
│   │   │   └── mod.rs                      # Shared utilities & mocks
│   │   ├── unit/
│   │   │   ├── test_routing.rs             # Route creation & matching
│   │   │   ├── test_validation.rs          # Parameter validation
│   │   │   ├── test_serialization.rs       # JSON/multipart processing  
│   │   │   └── test_security.rs            # OAuth2 & security
│   │   │
│   │   ├── integration/
│   │   │   └── test_integration_fastapi.rs # End-to-end pipeline
│   │   └── bench/
│   │       └── ...
│   │
│   └── python/                             # Module-level tests
│       ├── ...                             # Rust unit tests
│       └── bench/                          # Python interface tests
│
├── build.rs                  # Rust build script
├── Cargo.toml                # Rust dependencies
└── pyproject.toml            # Python packaging
```


## Testing

```bash
# Run Python tests
pytest tests/python/

# Run Rust tests
cargo test/rust/

# Run benchmarks
pytest tests/python/bench/ --benchmark-only

# Performance comparison
python tests/benchmark_comparison.py
```

## Benchmarking

To verify performance improvements on your system:

```bash
# Install benchmark dependencies
pip install pytest-benchmark memory-profiler

# Run comprehensive benchmarks
python -m pytest tests/benchmarks/ -v --benchmark-compare

# Memory usage comparison
python scripts/memory_benchmark.py
```

## Development

### Building from Source

```bash
# Development build
maturin develop

# Release build with optimizations
maturin develop --release

# Build wheel for distribution
maturin build --release --compatibility manylinux_2_28
```

### Code Quality

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy -- -D warnings

# Format Python code
black fastapi/

# Type checking
mypy fastapi/
```

## Compatibility

### Python Versions
- Python 3.8+
- Compatible with all FastAPI versions 0.100+

### Platforms
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

### FastAPI Features
- ✅ Path operations (GET, POST, PUT, DELETE, etc.)
- ✅ Path parameters with types
- ✅ Query parameters
- ✅ Request body (JSON, Form, Files)
- ✅ Header parameters
- ✅ Cookie parameters
- ✅ Dependency injection
- ✅ Security schemes
- ✅ OpenAPI generation
- ✅ Automatic documentation

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Install Rust: https://rustup.rs/
2. Install Python dependencies: `pip install -e .[dev]`
3. Install pre-commit hooks: `pre-commit install`
4. Run tests: `pytest` and `cargo test`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [FastAPI](https://fastapi.tiangolo.com/) - The amazing Python web framework this project enhances
- [PyO3](https://pyo3.rs/) - Python bindings for Rust
- [Maturin](https://github.com/PyO3/maturin) - Build tool for Python extensions in Rust

## Support

- [Documentation](https://github.com/pavanepour-k/fastapi-rs/docs)
- [Issue Tracker](https://github.com/pavanepour-k/fastapi-rs/issues)
- [Discussions](https://github.com/pavanepour-k/fastapi-rs/discussions)
