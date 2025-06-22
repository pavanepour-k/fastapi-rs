"""
FastAPI routers module initialization.

This module exposes all API routers for the FastAPI application.
Each router handles specific domain endpoints with proper authentication,
authorization, and integration with Rust-based performance modules.
"""

from .user import router as user_router
from .item import router as item_router
from .rust_routes import router as rust_router

__all__ = [
    "user_router",
    "item_router", 
    "rust_router"
]