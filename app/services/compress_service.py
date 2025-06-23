import rustlib


def gzip_compress(data: bytes, level: int) -> bytes:
    """
    Compress data with GZIP at specified level (0-9) using Rust.
    """
    return rustlib.gzip_compress(data, level)


def gzip_decompress(data: bytes) -> bytes:
    """
    Decompress GZIP-compressed data using Rust.
    """
    return rustlib.gzip_decompress(data)


def zstd_compress(data: bytes, level: int) -> bytes:
    """
    Compress data with Zstandard at specified level (> 0) using Rust.
    """
    return rustlib.zstd_compress(data, level)


def zstd_decompress(data: bytes) -> bytes:
    """
    Decompress Zstandard-compressed data using Rust.
    """
    return rustlib.zstd_decompress(data)
