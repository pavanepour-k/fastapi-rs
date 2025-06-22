"""
PyO3 Rust module bindings for FastAPI application.
Provides high-performance cryptographic, validation, and file operations.
"""

from typing import List, Optional
import rust_mod

class RustCrypto:
    """Cryptographic operations using Rust implementations."""
    
    @staticmethod
    def hash_password(password: str) -> str:
        """Hash password using Argon2 via Rust module."""
        return rust_mod.hash_password(password)
    
    @staticmethod
    def verify_password(password: str, hash: str) -> bool:
        """Verify password against hash using Rust module."""
        return rust_mod.verify_password(password, hash)
    
    @staticmethod
    def sha256_hash(data: str) -> str:
        """Calculate SHA256 hash using Rust module."""
        return rust_mod.sha256_hash(data)
    
    @staticmethod
    def bcrypt_hash(password: str, cost: int = 12) -> str:
        """Hash password using bcrypt via Rust module."""
        return rust_mod.bcrypt_hash(password, cost)
    
    @staticmethod
    def bcrypt_verify(password: str, hash: str) -> bool:
        """Verify password against bcrypt hash using Rust module."""
        return rust_mod.bcrypt_verify(password, hash)

class RustValidation:
    """Validation operations using Rust regex and pattern matching."""
    
    @staticmethod
    def validate_email(email: str) -> bool:
        """Validate email format using Rust regex."""
        return rust_mod.validate_email(email)
    
    @staticmethod
    def validate_phone(phone: str) -> bool:
        """Validate phone number format using Rust regex."""
        return rust_mod.validate_phone(phone)
    
    @staticmethod
    def validate_url(url: str) -> bool:
        """Validate URL format using Rust regex."""
        return rust_mod.validate_url(url)
    
    @staticmethod
    def find_pattern_matches(pattern: str, text: str) -> List[str]:
        """Find all matches of regex pattern in text using Rust regex."""
        return rust_mod.find_pattern_matches(pattern, text)
    
    @staticmethod
    def sanitize_input(input_text: str) -> str:
        """Sanitize input text using Rust string operations."""
        return rust_mod.sanitize_input(input_text)
    
    @staticmethod
    def validate_json_schema(json_data: str, schema: str) -> bool:
        """Validate JSON against schema using Rust validation."""
        return rust_mod.validate_json_schema(json_data, schema)

class RustFileOps:
    """File operations using Rust implementations for performance."""
    
    @staticmethod
    def calculate_file_hash(content: bytes) -> str:
        """Calculate BLAKE3 hash of file content using Rust module."""
        return rust_mod.calculate_file_hash(content)
    
    @staticmethod
    def verify_file_integrity(content: bytes, expected_hash: str) -> bool:
        """Verify file integrity using BLAKE3 hash via Rust module."""
        return rust_mod.verify_file_integrity(content, expected_hash)
    
    @staticmethod
    def compress_data(data: bytes) -> bytes:
        """Compress data using Rust compression algorithms."""
        return rust_mod.compress_data(data)
    
    @staticmethod
    def decompress_data(compressed_data: bytes) -> bytes:
        """Decompress data using Rust decompression algorithms."""
        return rust_mod.decompress_data(compressed_data)
    
    @staticmethod
    def calculate_checksum(data: bytes, algorithm: str = "blake3") -> str:
        """Calculate checksum using specified algorithm via Rust module."""
        return rust_mod.calculate_checksum(data, algorithm)

class RustSecurity:
    """Security-related operations using Rust implementations."""
    
    @staticmethod
    def generate_secure_token(length: int = 32) -> str:
        """Generate cryptographically secure random token using Rust."""
        return rust_mod.generate_secure_token(length)
    
    @staticmethod
    def constant_time_compare(a: str, b: str) -> bool:
        """Perform constant-time string comparison using Rust."""
        return rust_mod.constant_time_compare(a, b)
    
    @staticmethod
    def derive_key(password: str, salt: bytes, iterations: int = 100000) -> bytes:
        """Derive key from password using PBKDF2 via Rust module."""
        return rust_mod.derive_key(password, salt, iterations)
    
    @staticmethod
    def encrypt_aes(data: bytes, key: bytes) -> bytes:
        """Encrypt data using AES via Rust module."""
        return rust_mod.encrypt_aes(data, key)
    
    @staticmethod
    def decrypt_aes(encrypted_data: bytes, key: bytes) -> bytes:
        """Decrypt data using AES via Rust module."""
        return rust_mod.decrypt_aes(encrypted_data, key)

# Module-level functions for backward compatibility
def hash_password(password: str) -> str:
    """Convenience function for password hashing."""
    return RustCrypto.hash_password(password)

def verify_password(password: str, hash: str) -> bool:
    """Convenience function for password verification."""
    return RustCrypto.verify_password(password, hash)

def validate_email(email: str) -> bool:
    """Convenience function for email validation."""
    return RustValidation.validate_email(email)

def calculate_file_hash(content: bytes) -> str:
    """Convenience function for file hash calculation."""
    return RustFileOps.calculate_file_hash(content)