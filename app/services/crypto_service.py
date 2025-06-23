import rustlib


def hash_sha256(data: bytes) -> str:
    """
    Compute SHA-256 hash of input data using Rust.

    Args:
        data (bytes): The input byte sequence to hash.

    Returns:
        str: Lowercase hexadecimal representation of the SHA-256 hash.
    """
    return rustlib.hash_sha256(data)


def hash_sha1(data: bytes) -> str:
    """
    Compute SHA-1 hash of input data using Rust.

    Args:
        data (bytes): The input byte sequence to hash.

    Returns:
        str: Lowercase hexadecimal representation of the SHA-1 hash.
    """
    return rustlib.hash_sha1(data)


def hash_md5(data: bytes) -> str:
    """
    Compute MD5 hash of input data using Rust.

    Args:
        data (bytes): The input byte sequence to hash.

    Returns:
        str: Lowercase hexadecimal representation of the MD5 hash.
    """
    return rustlib.hash_md5(data)


def hmac_sha256(data: bytes, key: bytes) -> str:
    """
    Compute HMAC-SHA256 of input data using the provided key, via Rust.

    Args:
        data (bytes): The input byte sequence to authenticate.
        key (bytes): The secret key used for HMAC.

    Returns:
        str: Lowercase hexadecimal representation of the HMAC-SHA256.
    """
    return rustlib.hmac_sha256(data, key)