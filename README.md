# Rust Architecture

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
    └── rust/
        ├── common/mod.rs           # Shared utilities & mocks
        ├── unit/
        │   ├── test_routing.rs     # Route creation & matching
        │   ├── test_validation.rs  # Parameter validation
        │   ├── test_serialization.rs # JSON/multipart processing  
        │   └── test_security.rs    # OAuth2 & security
        ├── integration/
        │   └── test_integration_fastapi.rs # End-to-end pipeline
        └── bench/...
```

