import asyncio
import aiosqlite
from typing import Optional, Dict, Any, List
from contextlib import asynccontextmanager
import os
from pathlib import Path

DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./app.db")
DATABASE_PATH = DATABASE_URL.replace("sqlite:///", "")

class DatabaseConnection:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self._connection: Optional[aiosqlite.Connection] = None
    
    async def connect(self):
        if self._connection is None:
            self._connection = await aiosqlite.connect(self.db_path)
            self._connection.row_factory = aiosqlite.Row
        return self._connection
    
    async def disconnect(self):
        if self._connection:
            await self._connection.close()
            self._connection = None
    
    async def execute(self, query: str, params: tuple = ()) -> int:
        conn = await self.connect()
        cursor = await conn.execute(query, params)
        await conn.commit()
        return cursor.lastrowid
    
    async def fetch_one(self, query: str, params: tuple = ()) -> Optional[Dict[str, Any]]:
        conn = await self.connect()
        cursor = await conn.execute(query, params)
        row = await cursor.fetchone()
        return dict(row) if row else None
    
    async def fetch_all(self, query: str, params: tuple = ()) -> List[Dict[str, Any]]:
        conn = await self.connect()
        cursor = await conn.execute(query, params)
        rows = await cursor.fetchall()
        return [dict(row) for row in rows]
    
    async def execute_many(self, query: str, params_list: List[tuple]) -> None:
        conn = await self.connect()
        await conn.executemany(query, params_list)
        await conn.commit()

# Global database connection instance
_db_connection: Optional[DatabaseConnection] = None

async def get_db_connection() -> DatabaseConnection:
    global _db_connection
    if _db_connection is None:
        _db_connection = DatabaseConnection(DATABASE_PATH)
    return _db_connection

@asynccontextmanager
async def get_db_transaction():
    db = await get_db_connection()
    conn = await db.connect()
    try:
        yield db
        await conn.commit()
    except Exception:
        await conn.rollback()
        raise
    finally:
        pass

async def init_db():
    """Initialize database with required tables."""
    db = await get_db_connection()
    
    # Ensure database directory exists
    Path(DATABASE_PATH).parent.mkdir(parents=True, exist_ok=True)
    
    # Create users table
    await db.execute("""
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            full_name TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            is_verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    # Create files table
    await db.execute("""
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            file_hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            content_type TEXT,
            checksum TEXT,
            compression_type TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
    """)
    
    # Create sessions table for token management
    await db.execute("""
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token_hash TEXT NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            is_revoked BOOLEAN DEFAULT FALSE,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
    """)
    
    # Create validation_logs table
    await db.execute("""
        CREATE TABLE IF NOT EXISTS validation_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            validation_type TEXT NOT NULL,
            input_data TEXT,
            validation_result BOOLEAN,
            error_message TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
    """)
    
    # Create indexes for better performance
    await db.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
    await db.execute("CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id)")
    await db.execute("CREATE INDEX IF NOT EXISTS idx_files_hash ON files(file_hash)")
    await db.execute("CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)")
    await db.execute("CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash)")
    await db.execute("CREATE INDEX IF NOT EXISTS idx_validation_logs_user_id ON validation_logs(user_id)")

async def close_db():
    """Close database connection."""
    global _db_connection
    if _db_connection:
        await _db_connection.disconnect()
        _db_connection = None

# Database utilities
async def create_user(email: str, password_hash: str, full_name: str) -> int:
    """Create a new user and return user ID."""
    db = await get_db_connection()
    return await db.execute(
        "INSERT INTO users (email, password_hash, full_name) VALUES (?, ?, ?)",
        (email, password_hash, full_name)
    )

async def get_user_by_email(email: str) -> Optional[Dict[str, Any]]:
    """Get user by email address."""
    db = await get_db_connection()
    return await db.fetch_one(
        "SELECT * FROM users WHERE email = ?",
        (email,)
    )

async def get_user_by_id(user_id: int) -> Optional[Dict[str, Any]]:
    """Get user by ID."""
    db = await get_db_connection()
    return await db.fetch_one(
        "SELECT * FROM users WHERE id = ?",
        (user_id,)
    )

async def update_user_verification(user_id: int, is_verified: bool = True) -> None:
    """Update user verification status."""
    db = await get_db_connection()
    await db.execute(
        "UPDATE users SET is_verified = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        (is_verified, user_id)
    )

async def create_file_record(
    user_id: int, 
    filename: str, 
    file_hash: str, 
    file_size: int,
    content_type: Optional[str] = None,
    checksum: Optional[str] = None
) -> int:
    """Create file record and return file ID."""
    db = await get_db_connection()
    return await db.execute(
        "INSERT INTO files (user_id, filename, file_hash, file_size, content_type, checksum) VALUES (?, ?, ?, ?, ?, ?)",
        (user_id, filename, file_hash, file_size, content_type, checksum)
    )

async def get_user_files(user_id: int) -> List[Dict[str, Any]]:
    """Get all files for a user."""
    db = await get_db_connection()
    return await db.fetch_all(
        "SELECT * FROM files WHERE user_id = ? ORDER BY created_at DESC",
        (user_id,)
    )

async def log_validation(
    user_id: Optional[int],
    validation_type: str,
    input_data: str,
    validation_result: bool,
    error_message: Optional[str] = None
) -> None:
    """Log validation attempt."""
    db = await get_db_connection()
    await db.execute(
        "INSERT INTO validation_logs (user_id, validation_type, input_data, validation_result, error_message) VALUES (?, ?, ?, ?, ?)",
        (user_id, validation_type, input_data, validation_result, error_message)
    )