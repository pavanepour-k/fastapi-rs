import rustlib as rust_mod
from typing import Optional


def hash_sha256(data: bytes) -> str:
    """
    Compute SHA-256 hash of input data, returning a lowercase hex string.
    """
    return rust_mod.hash_sha256(data)


def hash_sha1(data: bytes) -> str:
    """
    Compute SHA-1 hash of input data, returning a lowercase hex string.
    """
    return rust_mod.hash_sha1(data)


def hash_md5(data: bytes) -> str:
    """
    Compute MD5 hash of input data, returning a lowercase hex string.
    """
    return rust_mod.hash_md5(data)


def hmac_sha256(data: bytes, key: bytes) -> str:
    """
    Compute HMAC-SHA256 of input data with the given key, returning a lowercase hex string.
    Raises ValueError if the key length is invalid.
    """
    return rust_mod.hmac_sha256(data, key)
