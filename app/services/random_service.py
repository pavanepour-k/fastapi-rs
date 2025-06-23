import rustlib
from typing import List, Union


def random_u32(min_value: int, max_value: int) -> int:
    """
    Generate a random u32 in the range [min_value, max_value] using Rust.
    Raises ValueError if min_value > max_value.
    """
    return rust_mod.random_u32(min_value, max_value)


def random_f64() -> float:
    """
    Generate a random float in the range [0.0, 1.0) using Rust.
    """
    return rust_mod.random_f64()


def random_bool() -> bool:
    """
    Generate a random boolean using Rust.
    """
    return rust_mod.random_bool()


def random_string(length: int) -> str:
    """
    Generate a random alphanumeric string of given length using Rust.
    """
    return rust_mod.random_string(length)


def random_bytes(length: int) -> bytes:
    """
    Generate random bytes of given length using Rust.
    """
    # rust_mod.random_bytes returns a Python bytes-like object
    return rust_mod.random_bytes(length)


def uuid_v4() -> str:
    """
    Generate a new UUID v4 string using Rust.
    """
    return rust_mod.uuid_v4()
