from pydantic import BaseModel, EmailStr, Field, validator
from typing import Optional, List, Dict, Any
from datetime import datetime

class UserBase(BaseModel):
    email: EmailStr
    full_name: str = Field(..., min_length=1, max_length=100)

class UserCreate(UserBase):
    password: str = Field(..., min_length=8, max_length=128)
    
    @validator('password')
    def validate_password(cls, v):
        if not any(c.isupper() for c in v):
            raise ValueError('Password must contain at least one uppercase letter')
        if not any(c.islower() for c in v):
            raise ValueError('Password must contain at least one lowercase letter')
        if not any(c.isdigit() for c in v):
            raise ValueError('Password must contain at least one digit')
        return v

class UserResponse(UserBase):
    id: int
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    
    class Config:
        from_attributes = True

class User(UserResponse):
    password_hash: str
    is_active: bool = True
    is_verified: bool = False

class LoginRequest(BaseModel):
    email: EmailStr
    password: str = Field(..., min_length=1)

class LoginResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
    expires_in: int = 3600
    user: UserResponse

class TokenData(BaseModel):
    user_id: Optional[int] = None
    email: Optional[str] = None

class PatternMatch(BaseModel):
    pattern: str = Field(..., min_length=1)
    text: str = Field(..., min_length=1)

class ValidationRequest(BaseModel):
    email: Optional[EmailStr] = None
    phone: Optional[str] = None
    url: Optional[str] = None
    patterns: Optional[List[PatternMatch]] = None
    json_data: Optional[str] = None
    json_schema: Optional[str] = None

class ValidationResponse(BaseModel):
    email_valid: bool = True
    phone_valid: bool = True
    url_valid: bool = True
    pattern_matches: List[Dict[str, Any]] = []
    json_valid: bool = True
    validation_errors: List[str] = []

class FileUpload(BaseModel):
    filename: str = Field(..., min_length=1, max_length=255)
    content: bytes
    content_type: Optional[str] = None
    expected_hash: Optional[str] = None
    
    class Config:
        arbitrary_types_allowed = True

class FileResponse(BaseModel):
    id: int
    filename: str
    file_hash: str
    file_size: int
    content_type: Optional[str] = None
    upload_status: str = "pending"
    uploaded_at: Optional[datetime] = None
    
    class Config:
        from_attributes = True

class FileMetadata(BaseModel):
    id: int
    user_id: int
    filename: str
    file_hash: str
    file_size: int
    content_type: Optional[str] = None
    checksum: Optional[str] = None
    compression_type: Optional[str] = None
    created_at: datetime
    updated_at: Optional[datetime] = None

class HashRequest(BaseModel):
    data: str = Field(..., min_length=1)
    algorithm: str = Field(default="sha256", regex="^(sha256|blake3|md5)$")

class HashResponse(BaseModel):
    data: str
    algorithm: str
    hash: str
    calculated_at: datetime

class EncryptionRequest(BaseModel):
    data: bytes
    key: Optional[bytes] = None
    algorithm: str = Field(default="aes", regex="^(aes|chacha20)$")
    
    class Config:
        arbitrary_types_allowed = True

class EncryptionResponse(BaseModel):
    encrypted_data: bytes
    key: bytes
    algorithm: str
    iv: Optional[bytes] = None
    
    class Config:
        arbitrary_types_allowed = True

class SecurityTokenRequest(BaseModel):
    length: int = Field(default=32, ge=16, le=128)
    include_special_chars: bool = True
    numeric_only: bool = False

class SecurityTokenResponse(BaseModel):
    token: str
    length: int
    generated_at: datetime
    expires_at: Optional[datetime] = None

class CompressionRequest(BaseModel):
    data: bytes
    algorithm: str = Field(default="gzip", regex="^(gzip|brotli|zstd)$")
    level: int = Field(default=6, ge=1, le=9)
    
    class Config:
        arbitrary_types_allowed = True

class CompressionResponse(BaseModel):
    compressed_data: bytes
    original_size: int
    compressed_size: int
    compression_ratio: float
    algorithm: str
    
    class Config:
        arbitrary_types_allowed = True

class ErrorResponse(BaseModel):
    error: str
    detail: str
    error_code: Optional[str] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)

class HealthResponse(BaseModel):
    status: str = "healthy"
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    version: str = "1.0.0"
    rust_modules: str = "loaded"
    database_status: str = "connected"