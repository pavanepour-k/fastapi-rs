import rustlib as rust_mod
import json
from typing import Any, Dict, List


def validate_json_schema(json_data: Dict[str, Any], schema: Dict[str, Any]) -> bool:
    """
    Validate a JSON object against a schema using Rust.

    Args:
        json_data: The JSON data to validate.
        schema: The JSON schema definition.

    Returns:
        True if the data conforms to the schema, False otherwise.
    """
    return rust_mod.validate_json_schema(
        json.dumps(json_data),
        json.dumps(schema),
    )


def calculate_similarity(str1: str, str2: str) -> float:
    """
    Compute a similarity score between two strings using Rust.

    Args:
        str1: First string.
        str2: Second string.

    Returns:
        A float score in the range [0.0, 1.0] indicating similarity.
    """
    return rust_mod.calculate_similarity(str1, str2)


def parse_csv(data: str) -> List[List[str]]:
    """
    Parse CSV-formatted text into rows of fields using Rust.

    Args:
        data: A CSV string where each line is a row and fields are comma-separated.

    Returns:
        A list of rows, each row is a list of string fields.
    """
    return rust_mod.parse_csv(data)
