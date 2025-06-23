import rustlib
from typing import Any, Dict, List, Tuple


class RustCrypto:
    """Cryptographic hash functions implemented in Rust."""

    @staticmethod
    def hash_sha256(data: bytes) -> str:
        """Compute SHA-256 hash, returning a lowercase hex string."""
        return rustlib.hash_sha256(data)

    @staticmethod
    def hash_sha1(data: bytes) -> str:
        """Compute SHA-1 hash, returning a lowercase hex string."""
        return rustlib.hash_sha1(data)

    @staticmethod
    def hash_md5(data: bytes) -> str:
        """Compute MD5 hash, returning a lowercase hex string."""
        return rustlib.hash_md5(data)

    @staticmethod
    def hmac_sha256(data: bytes, key: bytes) -> str:
        """Compute HMAC-SHA256 with the given key, returning a lowercase hex string."""
        return rustlib.hmac_sha256(data, key)


class RustValidation:
    """Validation utilities using Rust regex patterns."""

    @staticmethod
    def validate_email(email: str) -> bool:
        """Validate email address format."""
        return rustlib.validate_email(email)

    @staticmethod
    def validate_phone(number: str) -> bool:
        """Validate phone number (digits, spaces, hyphens, optional '+')."""
        return rustlib.validate_phone(number)

    @staticmethod
    def validate_url(url: str) -> bool:
        """Validate URL format (http/https, domain, optional path)."""
        return rustlib.validate_url(url)

    @staticmethod
    def find_pattern_matches(pattern: str, text: str) -> List[str]:
        """Find all non-overlapping regex matches in text."""
        return rustlib.find_pattern_matches(pattern, text)


class RustFilter:
    """Filtering utilities for HTML sanitization and text replacement."""

    @staticmethod
    def sanitize_html(input_text: str) -> str:
        """Remove harmful tags and attributes from HTML."""
        return rustlib.sanitize_html(input_text)

    @staticmethod
    def filter_bad_words(text: str, banned_list: List[str]) -> str:
        """Replace banned words with '[censored]' (case-insensitive)."""
        return rustlib.filter_bad_words(text, banned_list)

    @staticmethod
    def find_and_replace(pattern: str, text: str, replacement: str) -> str:
        """Replace all regex matches with replacement."""
        return rustlib.find_and_replace(pattern, text, replacement)


class RustDataProcessing:
    """Data processing utilities for JSON and CSV operations."""

    @staticmethod
    def parse_json(text: str) -> Any:
        """Parse JSON string into Python object."""
        return rustlib.parse_json(text)

    @staticmethod
    def to_json_string(value: Any) -> str:
        """Serialize Python object into JSON string."""
        return rustlib.to_json_string(value)

    @staticmethod
    def read_csv(path: str, has_headers: bool = True) -> List[Dict[str, str]]:
        """Read CSV file into list of row dictionaries."""
        return rustlib.read_csv(path, has_headers)

    @staticmethod
    def write_csv(path: str, data: List[Dict[str, str]], has_headers: bool = True) -> None:
        """Write list of row dictionaries to CSV file."""
        rustlib.write_csv(path, data, has_headers)

    @staticmethod
    def filter_rows(data: List[Dict[str, str]], column: str, pattern: str) -> List[Dict[str, str]]:
        """Filter rows where column matches regex pattern."""
        return rustlib.filter_rows(data, column, pattern)


class RustFileOps:
    """File operations implemented in Rust for performance."""

    @staticmethod
    def read_file(path: str) -> str:
        """Read entire file contents as string."""
        return rustlib.read_file(path)

    @staticmethod
    def write_file(path: str, content: str) -> None:
        """Write string content to file."""
        rustlib.write_file(path, content)

    @staticmethod
    def list_dir(path: str) -> List[str]:
        """List directory entries."""
        return rustlib.list_dir(path)

    @staticmethod
    def file_exists(path: str) -> bool:
        """Check if path exists."""
        return rustlib.file_exists(path)

    @staticmethod
    def remove(path: str) -> None:
        """Remove file or empty directory."""
        rustlib.remove(path)

    @staticmethod
    def rename(path: str, new_path: str) -> None:
        """Rename or move file/directory."""
        rustlib.rename(path, new_path)

    @staticmethod
    def create_dir(path: str) -> None:
        """Create directory and parents if missing."""
        rustlib.create_dir(path)

    @staticmethod
    def get_metadata(path: str) -> Tuple[int, bool, bool]:
        """Get metadata: size, is_file, is_dir."""
        return rustlib.get_metadata(path)

    @staticmethod
    def calculate_file_hash(path: str) -> str:
        """Calculate BLAKE3 hash of file at path."""
        return rustlib.calculate_file_hash(path)

    @staticmethod
    def calculate_checksum(path: str) -> int:
        """Calculate CRC32 checksum of file at path."""
        return rustlib.calculate_checksum(path)

    @staticmethod
    def verify_file_integrity(path: str, expected_hash: str) -> bool:
        """Verify file integrity against expected hash."""
        return rustlib.verify_file_integrity(path, expected_hash)


class RustCompression:
    """Compression and decompression utilities."""

    @staticmethod
    def gzip_compress(data: bytes, level: int) -> bytes:
        """Compress data with GZIP at specified level."""
        return rustlib.gzip_compress(data, level)

    @staticmethod
    def gzip_decompress(data: bytes) -> bytes:
        """Decompress GZIP-compressed data."""
        return rustlib.gzip_decompress(data)

    @staticmethod
    def zstd_compress(data: bytes, level: int) -> bytes:
        """Compress data with Zstandard at specified level."""
        return rustlib.zstd_compress(data, level)

    @staticmethod
    def zstd_decompress(data: bytes) -> bytes:
        """Decompress Zstandard-compressed data."""
        return rustlib.zstd_decompress(data)


class RustNetwork:
    """Network utilities implemented in Rust."""

    @staticmethod
    def validate_ip(address: str) -> bool:
        """Validate IPv4 or IPv6 address."""
        return rustlib.validate_ip(address)

    @staticmethod
    def resolve_hostname(hostname: str) -> List[str]:
        """Resolve hostname to IP addresses."""
        return rustlib.resolve_hostname(hostname)

    @staticmethod
    def http_get(url: str) -> str:
        """Perform HTTP GET and return response body."""
        return rustlib.http_get(url)

    @staticmethod
    def http_post(url: str, body: str) -> str:
        """Perform HTTP POST and return response body."""
        return rustlib.http_post(url, body)

    @staticmethod
    def get_status_code(url: str) -> int:
        """Get HTTP status code from GET request."""
        return rustlib.get_status_code(url)


class RustRandom:
    """Random number and data generation using Rust."""

    @staticmethod
    def random_u32(min_value: int, max_value: int) -> int:
        """Generate random u32 in range [min, max]."""
        return rustlib.random_u32(min_value, max_value)

    @staticmethod
    def random_f64() -> float:
        """Generate random float in [0.0, 1.0)."""
        return rustlib.random_f64()

    @staticmethod
    def random_bool() -> bool:
        """Generate random boolean."""
        return rustlib.random_bool()

    @staticmethod
    def random_string(length: int) -> str:
        """Generate random alphanumeric string of given length."""
        return rustlib.random_string(length)

    @staticmethod
    def random_bytes(length: int) -> bytes:
        """Generate random bytes of given length."""
        return rustlib.random_bytes(length)

    @staticmethod
    def uuid_v4() -> str:
        """Generate UUID v4 string."""
        return rustlib.uuid_v4()


# Convenience module-level functions for backward compatibility
hash_sha256 = RustCrypto.hash_sha256
hash_sha1 = RustCrypto.hash_sha1
hash_md5 = RustCrypto.hash_md5
hmac_sha256 = RustCrypto.hmac_sha256

validate_email = RustValidation.validate_email
validate_phone = RustValidation.validate_phone
validate_url = RustValidation.validate_url
find_pattern_matches = RustValidation.find_pattern_matches

sanitize_html = RustFilter.sanitize_html
filter_bad_words = RustFilter.filter_bad_words
find_and_replace = RustFilter.find_and_replace

parse_json = RustDataProcessing.parse_json
to_json_string = RustDataProcessing.to_json_string
read_csv = RustDataProcessing.read_csv
write_csv = RustDataProcessing.write_csv
filter_rows = RustDataProcessing.filter_rows

read_file = RustFileOps.read_file
write_file = RustFileOps.write_file
list_dir = RustFileOps.list_dir
file_exists = RustFileOps.file_exists
remove = RustFileOps.remove
rename = RustFileOps.rename
create_dir = RustFileOps.create_dir
get_metadata = RustFileOps.get_metadata
calculate_file_hash = RustFileOps.calculate_file_hash
calculate_checksum = RustFileOps.calculate_checksum
verify_file_integrity = RustFileOps.verify_file_integrity

gzip_compress = RustCompression.gzip_compress
gzip_decompress = RustCompression.gzip_decompress
zstd_compress = RustCompression.zstd_compress
zstd_decompress = RustCompression.zstd_decompress

validate_ip = RustNetwork.validate_ip
resolve_hostname = RustNetwork.resolve_hostname
http_get = RustNetwork.http_get
http_post = RustNetwork.http_post
get_status_code = RustNetwork.get_status_code

random_u32 = RustRandom.random_u32
random_f64 = RustRandom.random_f64
random_bool = RustRandom.random_bool
random_string = RustRandom.random_string
random_bytes = RustRandom.random_bytes
uuid_v4 = RustRandom.uuid_v4
