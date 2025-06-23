from typing import List
import rustlib as rust_mod

def sanitize_html(input: str) -> str:
    """
    Sanitize HTML by removing harmful tags and attributes using Rust.
    """
    return rust_mod.sanitize_html(input)


def filter_bad_words(text: str, banned_list: List[str]) -> str:
    """
    Replace banned words with "[censored]" in a case-insensitive manner using Rust.
    """
    return rust_mod.filter_bad_words(text, banned_list)


def find_and_replace(pattern: str, text: str, replacement: str) -> str:
    """
    Find all occurrences matching a pattern and replace them with replacement using Rust.
    """
    return rust_mod.find_and_replace(pattern, text, replacement)
