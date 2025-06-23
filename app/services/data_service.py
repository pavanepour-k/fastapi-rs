import rustlib
from typing import Any, Dict, List


def parse_json(text: str) -> Any:
    """
    Parse a JSON string into a Python object using Rust.

    Args:
        text (str): A valid JSON string.

    Returns:
        Any: The corresponding Python object (dict, list, etc.).
    """
    return rustlib.parse_json(text)


def to_json_string(value: Any) -> str:
    """
    Serialize a Python object into a JSON string using Rust.

    Args:
        value (Any): A JSON-serializable Python object.

    Returns:
        str: A formatted JSON string.
    """
    return rustlib.to_json_string(value)


def read_csv(path: str, has_headers: bool = True) -> List[Dict[str, str]]:
    """
    Read a CSV file into a list of dictionaries using Rust.

    Args:
        path (str): Path to the CSV file.
        has_headers (bool): Whether the CSV includes headers as the first row.

    Returns:
        List[Dict[str, str]]: List of rows as dictionaries (column name or index -> value).
    """
    return rustlib.read_csv(path, has_headers)


def write_csv(path: str, data: List[Dict[str, str]], has_headers: bool = True) -> None:
    """
    Write a list of dictionaries to a CSV file using Rust.

    Args:
        path (str): Path to save the CSV file.
        data (List[Dict[str, str]]): List of dictionaries representing rows.
        has_headers (bool): Whether to write headers as the first row.

    Returns:
        None
    """
    rustlib.write_csv(path, data, has_headers)


def filter_rows(data: List[Dict[str, str]], column: str, pattern: str) -> List[Dict[str, str]]:
    """
    Filter rows where a column matches a regex pattern using Rust.

    Args:
        data (List[Dict[str, str]]): List of row dictionaries.
        column (str): Column name to apply the pattern on.
        pattern (str): Regular expression pattern to match.

    Returns:
        List[Dict[str, str]]: Filtered list of dictionaries.
    """
    return rustlib.filter_rows(data, column, pattern)