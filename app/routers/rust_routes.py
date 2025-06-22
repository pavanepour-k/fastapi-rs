from fastapi import APIRouter, Depends, HTTPException, status, UploadFile, File
from fastapi.security import HTTPBearer
from typing import List, Dict, Any, Optional
from pydantic import BaseModel

from ..models.user import User
from ..auth import get_current_user
from ..rust_api import (
    hash_password,
    verify_password,
    generate_secure_token,
    validate_email,
    validate_text_content,
    filter_content_by_regex,
    calculate_file_hash,
    verify_file_integrity,
    compress_data,
    decompress_data,
    encrypt_data,
    decrypt_data,
    generate_random_bytes,
    hash_data_sha256,
    hash_data_blake3,
    validate_json_schema,
    sanitize_html_content
)

router = APIRouter(
    prefix="/rust",
    tags=["rust-utils"],
    responses={404: {"description": "Not found"}},
)

security = HTTPBearer()


class HashRequest(BaseModel):
    password: str


class VerifyPasswordRequest(BaseModel):
    password: str
    hashed_password: str


class EmailValidationRequest(BaseModel):
    email: str


class TextValidationRequest(BaseModel):
    content: str


class RegexFilterRequest(BaseModel):
    content: str
    pattern: str


class FileHashRequest(BaseModel):
    file_path: str


class FileIntegrityRequest(BaseModel):
    file_path: str
    expected_hash: str


class CompressionRequest(BaseModel):
    data: bytes


class EncryptionRequest(BaseModel):
    data: str
    key: Optional[str] = None


class DecryptionRequest(BaseModel):
    encrypted_data: str
    key: str


class RandomBytesRequest(BaseModel):
    length: int


class HashDataRequest(BaseModel):
    data: str


class JsonSchemaRequest(BaseModel):
    json_data: Dict[str, Any]
    schema: Dict[str, Any]


class HtmlSanitizeRequest(BaseModel):
    html_content: str


@router.post("/hash/password")
async def hash_password_endpoint(
    request: HashRequest,
    current_user: User = Depends(get_current_user)
):
    """Hash a password using Rust bcrypt implementation"""
    try:
        hashed = hash_password(request.password)
        return {"hashed_password": hashed}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to hash password: {str(e)}"
        )


@router.post("/verify/password")
async def verify_password_endpoint(
    request: VerifyPasswordRequest,
    current_user: User = Depends(get_current_user)
):
    """Verify a password against its hash using Rust implementation"""
    try:
        is_valid = verify_password(request.password, request.hashed_password)
        return {"is_valid": is_valid}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to verify password: {str(e)}"
        )


@router.post("/token/generate")
async def generate_token_endpoint(
    length: int = 32,
    current_user: User = Depends(get_current_user)
):
    """Generate a secure random token using Rust implementation"""
    try:
        token = generate_secure_token(length)
        return {"token": token}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to generate token: {str(e)}"
        )


@router.post("/validate/email")
async def validate_email_endpoint(request: EmailValidationRequest):
    """Validate email format using Rust regex implementation"""
    try:
        is_valid = validate_email(request.email)
        return {"is_valid": is_valid, "email": request.email}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to validate email: {str(e)}"
        )


@router.post("/validate/text")
async def validate_text_endpoint(request: TextValidationRequest):
    """Validate text content using Rust implementation"""
    try:
        is_valid = validate_text_content(request.content)
        return {"is_valid": is_valid}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to validate text: {str(e)}"
        )


@router.post("/filter/regex")
async def filter_content_endpoint(request: RegexFilterRequest):
    """Filter content using regex pattern with Rust implementation"""
    try:
        matches = filter_content_by_regex(request.content, request.pattern)
        return {"matches": matches, "pattern": request.pattern}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to filter content: {str(e)}"
        )


@router.post("/file/hash")
async def calculate_file_hash_endpoint(
    file: UploadFile = File(...),
    current_user: User = Depends(get_current_user)
):
    """Calculate file hash using Rust implementation"""
    try:
        file_content = await file.read()
        file_hash = calculate_file_hash(file_content)
        
        return {
            "filename": file.filename,
            "size": len(file_content),
            "hash": file_hash,
            "content_type": file.content_type
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to calculate file hash: {str(e)}"
        )


@router.post("/file/verify")
async def verify_file_integrity_endpoint(
    file: UploadFile = File(...),
    expected_hash: str = None,
    current_user: User = Depends(get_current_user)
):
    """Verify file integrity using Rust implementation"""
    if not expected_hash:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Expected hash is required"
        )
    
    try:
        file_content = await file.read()
        is_valid = verify_file_integrity(file_content, expected_hash)
        
        return {
            "filename": file.filename,
            "expected_hash": expected_hash,
            "is_valid": is_valid
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to verify file integrity: {str(e)}"
        )


@router.post("/compress")
async def compress_data_endpoint(
    data: str,
    current_user: User = Depends(get_current_user)
):
    """Compress data using Rust implementation"""
    try:
        compressed = compress_data(data.encode('utf-8'))
        return {
            "original_size": len(data.encode('utf-8')),
            "compressed_size": len(compressed),
            "compression_ratio": len(compressed) / len(data.encode('utf-8')),
            "compressed_data": compressed.hex()
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to compress data: {str(e)}"
        )


@router.post("/decompress")
async def decompress_data_endpoint(
    compressed_data_hex: str,
    current_user: User = Depends(get_current_user)
):
    """Decompress data using Rust implementation"""
    try:
        compressed_data = bytes.fromhex(compressed_data_hex)
        decompressed = decompress_data(compressed_data)
        
        return {
            "compressed_size": len(compressed_data),
            "decompressed_size": len(decompressed),
            "decompressed_data": decompressed.decode('utf-8')
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to decompress data: {str(e)}"
        )


@router.post("/encrypt")
async def encrypt_data_endpoint(
    request: EncryptionRequest,
    current_user: User = Depends(get_current_user)
):
    """Encrypt data using Rust implementation"""
    try:
        encrypted = encrypt_data(request.data, request.key)
        return {"encrypted_data": encrypted}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to encrypt data: {str(e)}"
        )


@router.post("/decrypt")
async def decrypt_data_endpoint(
    request: DecryptionRequest,
    current_user: User = Depends(get_current_user)
):
    """Decrypt data using Rust implementation"""
    try:
        decrypted = decrypt_data(request.encrypted_data, request.key)
        return {"decrypted_data": decrypted}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to decrypt data: {str(e)}"
        )


@router.post("/random/bytes")
async def generate_random_bytes_endpoint(
    request: RandomBytesRequest,
    current_user: User = Depends(get_current_user)
):
    """Generate secure random bytes using Rust implementation"""
    if request.length <= 0 or request.length > 1024:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Length must be between 1 and 1024"
        )
    
    try:
        random_bytes = generate_random_bytes(request.length)
        return {
            "length": request.length,
            "random_bytes": random_bytes.hex(),
            "base64": random_bytes.hex()  # You could use base64 encoding here
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to generate random bytes: {str(e)}"
        )


@router.post("/hash/sha256")
async def hash_sha256_endpoint(request: HashDataRequest):
    """Hash data using SHA256 with Rust implementation"""
    try:
        hash_result = hash_data_sha256(request.data)
        return {"hash": hash_result, "algorithm": "SHA256"}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to hash data: {str(e)}"
        )


@router.post("/hash/blake3")
async def hash_blake3_endpoint(request: HashDataRequest):
    """Hash data using BLAKE3 with Rust implementation"""
    try:
        hash_result = hash_data_blake3(request.data)
        return {"hash": hash_result, "algorithm": "BLAKE3"}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to hash data: {str(e)}"
        )


@router.post("/validate/json-schema")
async def validate_json_schema_endpoint(
    request: JsonSchemaRequest,
    current_user: User = Depends(get_current_user)
):
    """Validate JSON data against schema using Rust implementation"""
    try:
        is_valid = validate_json_schema(request.json_data, request.schema)
        return {"is_valid": is_valid}
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to validate JSON schema: {str(e)}"
        )


@router.post("/sanitize/html")
async def sanitize_html_endpoint(request: HtmlSanitizeRequest):
    """Sanitize HTML content using Rust implementation"""
    try:
        sanitized = sanitize_html_content(request.html_content)
        return {
            "original_content": request.html_content,
            "sanitized_content": sanitized
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to sanitize HTML: {str(e)}"
        )


@router.get("/benchmark/hash")
async def benchmark_hash_performance(
    iterations: int = 1000,
    current_user: User = Depends(get_current_user)
):
    """Benchmark hash performance between different algorithms"""
    if iterations <= 0 or iterations > 10000:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Iterations must be between 1 and 10000"
        )
    
    try:
        import time
        test_data = "test_data_for_benchmarking"
        
        # Benchmark SHA256
        start_time = time.time()
        for _ in range(iterations):
            hash_data_sha256(test_data)
        sha256_time = time.time() - start_time
        
        # Benchmark BLAKE3
        start_time = time.time()
        for _ in range(iterations):
            hash_data_blake3(test_data)
        blake3_time = time.time() - start_time
        
        return {
            "iterations": iterations,
            "sha256_time": sha256_time,
            "blake3_time": blake3_time,
            "sha256_ops_per_sec": iterations / sha256_time,
            "blake3_ops_per_sec": iterations / blake3_time,
            "blake3_speedup": sha256_time / blake3_time
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to run benchmark: {str(e)}"
        )


@router.get("/health")
async def rust_health_check():
    """Health check for Rust module functionality"""
    try:
        # Test basic functionality
        test_email = validate_email("test@example.com")
        test_hash = hash_data_sha256("test")
        test_token = generate_secure_token(16)
        
        return {
            "status": "healthy",
            "rust_module": "operational",
            "tests_passed": {
                "email_validation": test_email,
                "hash_generation": bool(test_hash),
                "token_generation": bool(test_token)
            }
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Rust module health check failed: {str(e)}"
        )