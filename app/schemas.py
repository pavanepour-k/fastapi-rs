from datetime import datetime
from typing import Optional, List, Union, Dict, Any
from pydantic import BaseModel, EmailStr, Field, validator, root_validator
from enum import Enum
import re


class UserRole(str, Enum):
    """User role enumeration."""
    ADMIN = "admin"
    USER = "user"
    MODERATOR = "moderator"
    GUEST = "guest"


class UserStatus(str, Enum):
    """User status enumeration."""
    ACTIVE = "active"
    INACTIVE = "inactive"
    SUSPENDED = "suspended"
    PENDING = "pending"


class FileType(str, Enum):
    """Allowed file type enumeration."""
    IMAGE = "image"
    DOCUMENT = "document"
    VIDEO = "video"
    AUDIO = "audio"


# Base schemas
class BaseSchema(BaseModel):
    """Base schema with common configuration."""
    
    class Config:
        orm_mode = True
        validate_assignment = True
        use_enum_values = True
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }


# User schemas
class UserBase(BaseSchema):
    """Base user schema."""
    username: str = Field(..., min_length=3, max_length=50, regex=r"^[a-zA-Z0-9_]+$")
    email: EmailStr
    full_name: Optional[str] = Field(None, max_length=100)
    role: UserRole = UserRole.USER
    is_active: bool = True
    
    @validator("username")
    def validate_username(cls, v):
        if v.lower() in ["admin", "root", "system", "null", "undefined"]:
            raise ValueError("Username not allowed")
        return v


class UserCreate(UserBase):
    """User creation schema."""
    password: str = Field(..., min_length=8, max_length=128)
    confirm_password: str = Field(..., min_length=8, max_length=128)
    
    @validator("password")
    def validate_password(cls, v):
        if not re.search(r"[A-Z]", v):
            raise ValueError("Password must contain at least one uppercase letter")
        if not re.search(r"[a-z]", v):
            raise ValueError("Password must contain at least one lowercase letter")
        if not re.search(r"\d", v):
            raise ValueError("Password must contain at least one digit")
        if not re.search(r"[!@#$%^&*(),.?\":{}|<>]", v):
            raise ValueError("Password must contain at least one special character")
        return v
    
    @root_validator
    def validate_passwords_match(cls, values):
        password = values.get("password")
        confirm_password = values.get("confirm_password")
        if password != confirm_password:
            raise ValueError("Passwords do not match")
        return values


class UserUpdate(BaseSchema):
    """User update schema."""
    username: Optional[str] = Field(None, min_length=3, max_length=50, regex=r"^[a-zA-Z0-9_]+$")
    email: Optional[EmailStr] = None
    full_name: Optional[str] = Field(None, max_length=100)
    role: Optional[UserRole] = None
    is_active: Optional[bool] = None


class UserPasswordChange(BaseSchema):
    """User password change schema."""
    current_password: str = Field(..., min_length=1)
    new_password: str = Field(..., min_length=8, max_length=128)
    confirm_new_password: str = Field(..., min_length=8, max_length=128)
    
    @validator("new_password")
    def validate_new_password(cls, v):
        if not re.search(r"[A-Z]", v):
            raise ValueError("Password must contain at least one uppercase letter")
        if not re.search(r"[a-z]", v):
            raise ValueError("Password must contain at least one lowercase letter")
        if not re.search(r"\d", v):
            raise ValueError("Password must contain at least one digit")
        if not re.search(r"[!@#$%^&*(),.?\":{}|<>]", v):
            raise ValueError("Password must contain at least one special character")
        return v
    
    @root_validator
    def validate_passwords_match(cls, values):
        new_password = values.get("new_password")
        confirm_new_password = values.get("confirm_new_password")
        if new_password != confirm_new_password:
            raise ValueError("New passwords do not match")
        return values


class UserResponse(UserBase):
    """User response schema."""
    id: int
    status: UserStatus = UserStatus.ACTIVE
    created_at: datetime
    updated_at: datetime
    last_login: Optional[datetime] = None
    login_count: int = 0


class UserListResponse(BaseSchema):
    """User list response schema."""
    users: List[UserResponse]
    total: int
    page: int
    size: int
    pages: int


# Authentication schemas
class Token(BaseSchema):
    """Token response schema."""
    access_token: str
    refresh_token: Optional[str] = None
    token_type: str = "bearer"
    expires_in: int
    expires_at: datetime
    scope: Optional[str] = None


class TokenData(BaseSchema):
    """Token data schema."""
    sub: Optional[str] = None
    scopes: List[str] = []
    exp: Optional[datetime] = None
    iat: Optional[datetime] = None


class LoginRequest(BaseSchema):
    """Login request schema."""
    username: str = Field(..., min_length=1)
    password: str = Field(..., min_length=1)
    remember_me: bool = False
    scope: Optional[str] = None


class RefreshTokenRequest(BaseSchema):
    """Refresh token request schema."""
    refresh_token: str = Field(..., min_length=1)


class PasswordResetRequest(BaseSchema):
    """Password reset request schema."""
    email: EmailStr


class PasswordResetConfirm(BaseSchema):
    """Password reset confirmation schema."""
    token: str = Field(..., min_length=1)
    new_password: str = Field(..., min_length=8, max_length=128)
    confirm_password: str = Field(..., min_length=8, max_length=128)
    
    @validator("new_password")
    def validate_new_password(cls, v):
        if not re.search(r"[A-Z]", v):
            raise ValueError("Password must contain at least one uppercase letter")
        if not re.search(r"[a-z]", v):
            raise ValueError("Password must contain at least one lowercase letter")
        if not re.search(r"\d", v):
            raise ValueError("Password must contain at least one digit")
        if not re.search(r"[!@#$%^&*(),.?\":{}|<>]", v):
            raise ValueError("Password must contain at least one special character")
        return v
    
    @root_validator
    def validate_passwords_match(cls, values):
        new_password = values.get("new_password")
        confirm_password = values.get("confirm_password")
        if new_password != confirm_password:
            raise ValueError("Passwords do not match")
        return values


# File schemas
class FileBase(BaseSchema):
    """Base file schema."""
    filename: str = Field(..., max_length=255)
    content_type: str = Field(..., max_length=100)
    file_type: FileType
    description: Optional[str] = Field(None, max_length=500)


class FileCreate(FileBase):
    """File creation schema."""
    pass


class FileResponse(FileBase):
    """File response schema."""
    id: int
    file_path: str
    file_size: int
    checksum: str
    upload_date: datetime
    uploaded_by: int
    is_public: bool = False
    download_count: int = 0


class FileListResponse(BaseSchema):
    """File list response schema."""
    files: List[FileResponse]
    total: int
    page: int
    size: int
    pages: int


# API Response schemas
class MessageResponse(BaseSchema):
    """Generic message response schema."""
    message: str
    success: bool = True
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class ErrorResponse(BaseSchema):
    """Error response schema."""
    error: str
    detail: Optional[str] = None
    code: Optional[str] = None
    success: bool = False
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class ValidationErrorResponse(BaseSchema):
    """Validation error response schema."""
    error: str = "Validation error"
    detail: List[Dict[str, Any]]
    success: bool = False
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class HealthCheckResponse(BaseSchema):
    """Health check response schema."""
    status: str = "healthy"
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    version: str
    uptime: float
    database: bool = True
    redis: bool = True
    rust_module: bool = True


class MetricsResponse(BaseSchema):
    """Metrics response schema."""
    active_users: int
    total_requests: int
    average_response_time: float
    error_rate: float
    uptime: float
    memory_usage: float
    cpu_usage: float
    database_connections: int


# Search and filtering schemas
class SortOrder(str, Enum):
    """Sort order enumeration."""
    ASC = "asc"
    DESC = "desc"


class BaseFilter(BaseSchema):
    """Base filter schema."""
    page: int = Field(default=1, ge=1)
    size: int = Field(default=20, ge=1, le=100)
    sort_by: Optional[str] = None
    sort_order: SortOrder = SortOrder.ASC
    search: Optional[str] = Field(None, max_length=100)


class UserFilter(BaseFilter):
    """User filter schema."""
    role: Optional[UserRole] = None
    status: Optional[UserStatus] = None
    is_active: Optional[bool] = None
    created_after: Optional[datetime] = None
    created_before: Optional[datetime] = None


class FileFilter(BaseFilter):
    """File filter schema."""
    file_type: Optional[FileType] = None
    content_type: Optional[str] = None
    uploaded_by: Optional[int] = None
    is_public: Optional[bool] = None
    min_size: Optional[int] = Field(None, ge=0)
    max_size: Optional[int] = Field(None, ge=0)
    uploaded_after: Optional[datetime] = None
    uploaded_before: Optional[datetime] = None


# Rust integration schemas
class HashRequest(BaseSchema):
    """Hash request schema for Rust module."""
    data: str = Field(..., min_length=1)
    algorithm: str = Field(default="sha256", regex=r"^(sha256|sha512|blake3)$")


class HashResponse(BaseSchema):
    """Hash response schema from Rust module."""
    hash: str
    algorithm: str
    processing_time_ms: float


class RegexValidationRequest(BaseSchema):
    """Regex validation request schema for Rust module."""
    pattern: str = Field(..., min_length=1, max_length=1000)
    text: str = Field(..., max_length=10000)
    flags: Optional[List[str]] = Field(default=[], description="Regex flags like 'i', 'm', 's'")


class RegexValidationResponse(BaseSchema):
    """Regex validation response schema from Rust module."""
    is_match: bool
    matches: List[str] = []
    processing_time_ms: float


class FileIntegrityRequest(BaseSchema):
    """File integrity check request schema for Rust module."""
    file_path: str = Field(..., min_length=1)
    expected_checksum: Optional[str] = None
    algorithm: str = Field(default="sha256", regex=r"^(sha256|sha512|blake3)$")


class FileIntegrityResponse(BaseSchema):
    """File integrity check response schema from Rust module."""
    file_path: str
    checksum: str
    algorithm: str
    file_size: int
    is_valid: bool
    processing_time_ms: float


# Batch operation schemas
class BatchUserCreate(BaseSchema):
    """Batch user creation schema."""
    users: List[UserCreate] = Field(..., min_items=1, max_items=100)


class BatchOperationResponse(BaseSchema):
    """Batch operation response schema."""
    total: int
    successful: int
    failed: int
    errors: List[Dict[str, Any]] = []
    results: List[Dict[str, Any]] = []


# Configuration schemas
class SystemConfigResponse(BaseSchema):
    """System configuration response schema."""
    app_name: str
    version: str
    environment: str
    features: Dict[str, bool]
    limits: Dict[str, int]
    rust_module_enabled: bool