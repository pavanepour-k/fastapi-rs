from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
import uvicorn
from typing import Optional, List
import os

from .rust_api import RustCrypto, RustValidation, RustFileOps
from .models import (
    User, UserCreate, UserResponse, 
    LoginRequest, LoginResponse,
    FileUpload, FileResponse,
    ValidationRequest, ValidationResponse
)
from .database import get_db_connection, init_db
from .auth import verify_token, create_access_token

security = HTTPBearer()

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Initialize database
    await init_db()
    yield
    # Cleanup if needed
    pass

app = FastAPI(
    title="FastAPI with Rust Performance",
    description="FastAPI application with Rust-powered crypto and validation",
    version="1.0.0",
    lifespan=lifespan
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.post("/register", response_model=UserResponse)
async def register_user(user_data: UserCreate, db=Depends(get_db_connection)):
    # Check if user exists
    existing_user = await db.fetch_one(
        "SELECT id FROM users WHERE email = ?", (user_data.email,)
    )
    if existing_user:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Email already registered"
        )
    
    # Hash password using Rust module
    password_hash = RustCrypto.hash_password(user_data.password)
    
    # Insert user
    user_id = await db.execute(
        "INSERT INTO users (email, password_hash, full_name) VALUES (?, ?, ?)",
        (user_data.email, password_hash, user_data.full_name)
    )
    
    return UserResponse(
        id=user_id,
        email=user_data.email,
        full_name=user_data.full_name
    )

@app.post("/login", response_model=LoginResponse)
async def login(login_data: LoginRequest, db=Depends(get_db_connection)):
    # Get user from database
    user = await db.fetch_one(
        "SELECT id, email, password_hash, full_name FROM users WHERE email = ?",
        (login_data.email,)
    )
    
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials"
        )
    
    # Verify password using Rust module
    if not RustCrypto.verify_password(login_data.password, user["password_hash"]):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials"
        )
    
    # Create access token
    access_token = create_access_token(data={"sub": str(user["id"])})
    
    return LoginResponse(
        access_token=access_token,
        token_type="bearer",
        user=UserResponse(
            id=user["id"],
            email=user["email"],
            full_name=user["full_name"]
        )
    )

@app.get("/users/me", response_model=UserResponse)
async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security),
    db=Depends(get_db_connection)
):
    user_id = verify_token(credentials.credentials)
    
    user = await db.fetch_one(
        "SELECT id, email, full_name FROM users WHERE id = ?",
        (user_id,)
    )
    
    if not user:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="User not found"
        )
    
    return UserResponse(
        id=user["id"],
        email=user["email"],
        full_name=user["full_name"]
    )

@app.post("/validate", response_model=ValidationResponse)
async def validate_data(
    validation_request: ValidationRequest,
    credentials: HTTPAuthorizationCredentials = Depends(security)
):
    verify_token(credentials.credentials)
    
    # Use Rust validation module
    email_valid = RustValidation.validate_email(validation_request.email) if validation_request.email else True
    phone_valid = RustValidation.validate_phone(validation_request.phone) if validation_request.phone else True
    
    # Pattern matching using Rust regex
    pattern_matches = []
    if validation_request.patterns:
        for pattern_data in validation_request.patterns:
            matches = RustValidation.find_pattern_matches(
                pattern_data.pattern, 
                pattern_data.text
            )
            pattern_matches.append({
                "pattern": pattern_data.pattern,
                "matches": matches
            })
    
    return ValidationResponse(
        email_valid=email_valid,
        phone_valid=phone_valid,
        pattern_matches=pattern_matches
    )

@app.post("/files/upload", response_model=FileResponse)
async def upload_file(
    file_data: FileUpload,
    credentials: HTTPAuthorizationCredentials = Depends(security),
    db=Depends(get_db_connection)
):
    user_id = verify_token(credentials.credentials)
    
    # Calculate file hash using Rust module
    file_hash = RustFileOps.calculate_file_hash(file_data.content)
    
    # Check file integrity
    if file_data.expected_hash and file_hash != file_data.expected_hash:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="File integrity check failed"
        )
    
    # Store file metadata
    file_id = await db.execute(
        "INSERT INTO files (user_id, filename, file_hash, file_size) VALUES (?, ?, ?, ?)",
        (user_id, file_data.filename, file_hash, len(file_data.content))
    )
    
    return FileResponse(
        id=file_id,
        filename=file_data.filename,
        file_hash=file_hash,
        file_size=len(file_data.content),
        upload_status="completed"
    )

@app.get("/files/{file_id}", response_model=FileResponse)
async def get_file_info(
    file_id: int,
    credentials: HTTPAuthorizationCredentials = Depends(security),
    db=Depends(get_db_connection)
):
    user_id = verify_token(credentials.credentials)
    
    file_info = await db.fetch_one(
        "SELECT id, filename, file_hash, file_size FROM files WHERE id = ? AND user_id = ?",
        (file_id, user_id)
    )
    
    if not file_info:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="File not found"
        )
    
    return FileResponse(
        id=file_info["id"],
        filename=file_info["filename"],
        file_hash=file_info["file_hash"],
        file_size=file_info["file_size"],
        upload_status="completed"
    )

@app.get("/health")
async def health_check():
    return {"status": "healthy", "rust_modules": "loaded"}

if __name__ == "__main__":
    uvicorn.run(
        "app.main:app",
        host="0.0.0.0",
        port=8000,
        reload=True
    )