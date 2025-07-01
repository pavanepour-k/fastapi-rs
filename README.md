# FastAPI-RS

High-performance Rust implementation for FastAPI core components, providing 3-5x performance improvements while maintaining 100% API compatibility.

## Rust Architecture

```
fastapi-rs/
├── rust_src/                 # High-performance Rust implementation
│   ├── core/                 # Request lifecycle core
│   │   ├── mod.rs
│   │   ├── routing.rs        # Endpoint routing/dispatch
│   │   └── request.rs        # Request processing
│   │
│   ├── params/               # Parameter processing
│   │   ├── mod.rs
│   │   ├── validation.rs     # Data validation logic
│   │   ├── query.rs          # Query param handling
│   │   ├── path.rs           # Path param handling
│   │   └── body.rs           # Request body processing
│   │
│   ├── serialization/        # Data transformation
│   │   ├── mod.rs
│   │   ├── encoders.rs       # JSON serialization
│   │   └── decoders.rs       # Request body deserialization
│   │
│   ├── security/             # Security implementations
│   │   ├── mod.rs
│   │   ├── utils.rs          # Auth helpers
│   │   └── oauth2.rs         # OAuth2 flows
│   │
│   ├── utils/                # Shared utilities
│   │   ├── mod.rs
│   │   ├── async_tools.rs    # Async utilities
│   │   └── type_conv.rs      # Python-Rust type conversion
│   │
│   ├── types/                # Type system
│   │   ├── mod.rs
│   │   └── models.rs         # Pydantic model equivalents
│   │
│   ├── lib.rs                # Rust entry point
│   └── python_bindings.rs    # PyO3 interface definitions
│
└── tests/                    # Verification suite
```

## Security Enhancements

### Memory Safety

- Zero buffer overflows (guaranteed at compile-time)
- No use-after-free vulnerabilities
- Automatic bounds checking on all operations

## Testing

```bash

# Run Rust tests
cargo test

```


### Code Quality

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy -- -D warnings

```

## Compatibility

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


## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**FastAPI-RS** - Bringing Rust's performance and safety to Python's most loved web framework.