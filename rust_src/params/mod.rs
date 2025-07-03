//! Module entry point for parameter processing in FastAPI-RS.
//!
//! Exposes parameter handling submodules: body, path, query, validation.

pub mod body;
pub mod path;
pub mod query;
pub mod validation;

pub use body::*;
pub use path::*;
pub use query::*;
pub use validation::*;
