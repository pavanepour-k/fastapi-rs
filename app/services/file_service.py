import rustlib
from typing import List, Tuple


def read_file(path: str) -> str:
    """
    Read the entire contents of a file into a string using Rust.
    """
    return rustlib.read_file(path)


def write_file(path: str, content: str) -> None:
    """
    Write a string to a file, creating or truncating it using Rust.
    """
    rustlib.write_file(path, content)


def list_dir(path: str) -> List[str]:
    """
    List names of entries in a directory using Rust.
    """
    return rustlib.list_dir(path)


def file_exists(path: str) -> bool:
    """
    Check whether a path exists using Rust.
    """
    return rustlib.file_exists(path)


def remove(path: str) -> None:
    """
    Remove a file or empty directory using Rust.
    """
    rustlib.remove(path)


def rename(path: str, new_path: str) -> None:
    """
    Rename or move a file or directory using Rust.
    """
    rustlib.rename(path, new_path)


def create_dir(path: str) -> None:
    """
    Create a directory and all parent components if missing using Rust.
    """
    rustlib.create_dir(path)


def get_metadata(path: str) -> Tuple[int, bool, bool]:
    """
    Get metadata: (size in bytes, is_file, is_dir) using Rust.
    """
    return rustlib.get_metadata(path)


def calculate_file_hash(path: str) -> str:
    """
    Calculate the BLAKE3 hash of the file at the given path using Rust.
    """
    return rustlib.calculate_file_hash(path)


def calculate_checksum(path: str) -> int:
    """
    Calculate the CRC32 checksum of the file at the given path using Rust.
    """
    return rustlib.calculate_checksum(path)


def verify_file_integrity(path: str, expected_hash: str) -> bool:
    """
    Verify that the file at the given path matches the expected hash using Rust.
    """
    return rustlib.verify_file_integrity(path, expected_hash)
