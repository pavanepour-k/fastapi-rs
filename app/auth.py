from datetime import datetime, timedelta, timezone
from typing import Optional, Union
from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from jose import JWTError, jwt
from passlib.context import CryptContext
from pydantic import BaseModel
import secrets
import os

from .rust_api import hash_password, verify_password_hash


class Token(BaseModel):
    access_token: str
    token_type: str
    expires_in: int


class TokenData(BaseModel):
    username: Optional[str] = None
    scopes: list[str] = []


class User(BaseModel):
    id: int
    username: str
    email: str
    is_active: bool = True
    is_superuser: bool = False
    created_at: datetime
    updated_at: datetime


class UserInDB(User):
    hashed_password: str


class UserCreate(BaseModel):
    username: str
    email: str
    password: str


class UserUpdate(BaseModel):
    username: Optional[str] = None
    email: Optional[str] = None
    password: Optional[str] = None
    is_active: Optional[bool] = None


# JWT configuration
SECRET_KEY = os.getenv("SECRET_KEY", secrets.token_urlsafe(32))
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

# Security schemes
security = HTTPBearer()
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None) -> str:
    """Create JWT access token with optional expiration time."""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


def verify_token(token: str) -> TokenData:
    """Verify JWT token and extract token data."""
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )
    
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        username: str = payload.get("sub")
        if username is None:
            raise credentials_exception
        
        scopes = payload.get("scopes", [])
        token_data = TokenData(username=username, scopes=scopes)
    except JWTError:
        raise credentials_exception
    
    return token_data


def get_password_hash(password: str) -> str:
    """Hash password using Rust implementation."""
    return hash_password(password)


def verify_password(plain_password: str, hashed_password: str) -> bool:
    """Verify password against hash using Rust implementation."""
    return verify_password_hash(plain_password, hashed_password)


async def get_current_user(credentials: HTTPAuthorizationCredentials = Depends(security)) -> User:
    """Get current authenticated user from JWT token."""
    token_data = verify_token(credentials.credentials)
    
    # Mock user retrieval - replace with actual database query
    user = await get_user_by_username(token_data.username)
    if user is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="User not found",
            headers={"WWW-Authenticate": "Bearer"},
        )
    
    return user


async def get_current_active_user(current_user: User = Depends(get_current_user)) -> User:
    """Get current active user."""
    if not current_user.is_active:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Inactive user"
        )
    return current_user


async def get_current_superuser(current_user: User = Depends(get_current_user)) -> User:
    """Get current superuser."""
    if not current_user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Not enough permissions"
        )
    return current_user


def require_scopes(*required_scopes: str):
    """Decorator to require specific scopes for endpoint access."""
    def scope_checker(credentials: HTTPAuthorizationCredentials = Depends(security)):
        token_data = verify_token(credentials.credentials)
        
        for scope in required_scopes:
            if scope not in token_data.scopes:
                raise HTTPException(
                    status_code=status.HTTP_403_FORBIDDEN,
                    detail=f"Not enough permissions. Required scope: {scope}"
                )
        
        return token_data
    
    return scope_checker


async def authenticate_user(username: str, password: str) -> Union[UserInDB, bool]:
    """Authenticate user with username and password."""
    user = await get_user_by_username(username)
    if not user:
        return False
    
    if not verify_password(password, user.hashed_password):
        return False
    
    return user


async def get_user_by_username(username: str) -> Optional[UserInDB]:
    """Get user by username from database."""
    # Mock implementation - replace with actual database query
    # This should query your actual database
    mock_users = {
        "testuser": UserInDB(
            id=1,
            username="testuser",
            email="test@example.com",
            is_active=True,
            is_superuser=False,
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            hashed_password=get_password_hash("testpassword")
        )
    }
    
    return mock_users.get(username)


async def create_user(user_create: UserCreate) -> UserInDB:
    """Create new user."""
    hashed_password = get_password_hash(user_create.password)
    
    # Mock implementation - replace with actual database insertion
    user_in_db = UserInDB(
        id=1,  # This should come from database auto-increment
        username=user_create.username,
        email=user_create.email,
        is_active=True,
        is_superuser=False,
        created_at=datetime.now(timezone.utc),
        updated_at=datetime.now(timezone.utc),
        hashed_password=hashed_password
    )
    
    return user_in_db


def generate_reset_token(email: str) -> str:
    """Generate password reset token."""
    expire = datetime.now(timezone.utc) + timedelta(hours=1)
    to_encode = {"sub": email, "exp": expire, "type": "reset"}
    token = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return token


def verify_reset_token(token: str) -> Optional[str]:
    """Verify password reset token and return email."""
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        email: str = payload.get("sub")
        token_type: str = payload.get("type")
        
        if email is None or token_type != "reset":
            return None
        
        return email
    except JWTError:
        return None