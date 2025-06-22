import os
import secrets
from typing import Optional, List, Union
from pydantic import BaseSettings, validator, Field
from pydantic.networks import PostgresDsn, HttpUrl
from functools import lru_cache


class Settings(BaseSettings):
    """Application settings with environment variable support."""
    
    # Application
    app_name: str = Field(default="FastAPI Rust Integration", env="APP_NAME")
    debug: bool = Field(default=False, env="DEBUG")
    version: str = Field(default="1.0.0", env="VERSION")
    environment: str = Field(default="development", env="ENVIRONMENT")
    
    # Server
    host: str = Field(default="127.0.0.1", env="HOST")
    port: int = Field(default=8000, env="PORT")
    reload: bool = Field(default=True, env="RELOAD")
    workers: int = Field(default=1, env="WORKERS")
    
    # Security
    secret_key: str = Field(default_factory=lambda: secrets.token_urlsafe(32), env="SECRET_KEY")
    access_token_expire_minutes: int = Field(default=30, env="ACCESS_TOKEN_EXPIRE_MINUTES")
    refresh_token_expire_days: int = Field(default=7, env="REFRESH_TOKEN_EXPIRE_DAYS")
    password_reset_expire_hours: int = Field(default=1, env="PASSWORD_RESET_EXPIRE_HOURS")
    algorithm: str = Field(default="HS256", env="JWT_ALGORITHM")
    
    # CORS
    backend_cors_origins: List[str] = Field(
        default=["http://localhost:3000", "http://localhost:8080"],
        env="BACKEND_CORS_ORIGINS"
    )
    
    @validator("backend_cors_origins", pre=True)
    def assemble_cors_origins(cls, v: Union[str, List[str]]) -> Union[List[str], str]:
        if isinstance(v, str) and not v.startswith("["):
            return [i.strip() for i in v.split(",")]
        elif isinstance(v, (list, str)):
            return v
        raise ValueError(v)
    
    # Database
    database_url: Optional[PostgresDsn] = Field(default=None, env="DATABASE_URL")
    database_host: str = Field(default="localhost", env="DATABASE_HOST")
    database_port: int = Field(default=5432, env="DATABASE_PORT")
    database_name: str = Field(default="fastapi_db", env="DATABASE_NAME")
    database_user: str = Field(default="postgres", env="DATABASE_USER")
    database_password: str = Field(default="password", env="DATABASE_PASSWORD")
    database_pool_size: int = Field(default=10, env="DATABASE_POOL_SIZE")
    database_max_overflow: int = Field(default=20, env="DATABASE_MAX_OVERFLOW")
    database_echo: bool = Field(default=False, env="DATABASE_ECHO")
    
    @validator("database_url", pre=True)
    def assemble_db_connection(cls, v: Optional[str], values: dict) -> str:
        if isinstance(v, str):
            return v
        return PostgresDsn.build(
            scheme="postgresql",
            user=values.get("database_user"),
            password=values.get("database_password"),
            host=values.get("database_host"),
            port=str(values.get("database_port")),
            path=f"/{values.get('database_name') or ''}",
        )
    
    # Redis
    redis_url: str = Field(default="redis://localhost:6379/0", env="REDIS_URL")
    redis_host: str = Field(default="localhost", env="REDIS_HOST")
    redis_port: int = Field(default=6379, env="REDIS_PORT")
    redis_db: int = Field(default=0, env="REDIS_DB")
    redis_password: Optional[str] = Field(default=None, env="REDIS_PASSWORD")
    redis_pool_size: int = Field(default=10, env="REDIS_POOL_SIZE")
    
    # Email
    smtp_server: Optional[str] = Field(default=None, env="SMTP_SERVER")
    smtp_port: int = Field(default=587, env="SMTP_PORT")
    smtp_username: Optional[str] = Field(default=None, env="SMTP_USERNAME")
    smtp_password: Optional[str] = Field(default=None, env="SMTP_PASSWORD")
    smtp_use_tls: bool = Field(default=True, env="SMTP_USE_TLS")
    smtp_use_ssl: bool = Field(default=False, env="SMTP_USE_SSL")
    email_from: Optional[str] = Field(default=None, env="EMAIL_FROM")
    email_from_name: Optional[str] = Field(default=None, env="EMAIL_FROM_NAME")
    
    # File Upload
    max_file_size: int = Field(default=10 * 1024 * 1024, env="MAX_FILE_SIZE")  # 10MB
    allowed_file_types: List[str] = Field(
        default=["image/jpeg", "image/png", "image/gif", "application/pdf"],
        env="ALLOWED_FILE_TYPES"
    )
    upload_directory: str = Field(default="uploads", env="UPLOAD_DIRECTORY")
    
    @validator("allowed_file_types", pre=True)
    def assemble_file_types(cls, v: Union[str, List[str]]) -> Union[List[str], str]:
        if isinstance(v, str) and not v.startswith("["):
            return [i.strip() for i in v.split(",")]
        elif isinstance(v, (list, str)):
            return v
        raise ValueError(v)
    
    # Logging
    log_level: str = Field(default="INFO", env="LOG_LEVEL")
    log_format: str = Field(
        default="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        env="LOG_FORMAT"
    )
    log_file: Optional[str] = Field(default=None, env="LOG_FILE")
    log_rotation: str = Field(default="1 day", env="LOG_ROTATION")
    log_retention: str = Field(default="30 days", env="LOG_RETENTION")
    
    # Rate Limiting
    rate_limit_enabled: bool = Field(default=True, env="RATE_LIMIT_ENABLED")
    rate_limit_requests: int = Field(default=100, env="RATE_LIMIT_REQUESTS")
    rate_limit_period: int = Field(default=60, env="RATE_LIMIT_PERIOD")  # seconds
    
    # Monitoring
    enable_metrics: bool = Field(default=True, env="ENABLE_METRICS")
    metrics_endpoint: str = Field(default="/metrics", env="METRICS_ENDPOINT")
    health_check_endpoint: str = Field(default="/health", env="HEALTH_CHECK_ENDPOINT")
    
    # External APIs
    external_api_timeout: int = Field(default=30, env="EXTERNAL_API_TIMEOUT")
    external_api_retries: int = Field(default=3, env="EXTERNAL_API_RETRIES")
    
    # Rust Module Configuration
    rust_module_enabled: bool = Field(default=True, env="RUST_MODULE_ENABLED")
    rust_hash_algorithm: str = Field(default="argon2", env="RUST_HASH_ALGORITHM")
    rust_hash_rounds: int = Field(default=12, env="RUST_HASH_ROUNDS")
    rust_regex_cache_size: int = Field(default=1000, env="RUST_REGEX_CACHE_SIZE")
    rust_file_chunk_size: int = Field(default=8192, env="RUST_FILE_CHUNK_SIZE")
    
    # Testing
    testing: bool = Field(default=False, env="TESTING")
    test_database_url: Optional[str] = Field(default=None, env="TEST_DATABASE_URL")
    
    # Feature Flags
    feature_user_registration: bool = Field(default=True, env="FEATURE_USER_REGISTRATION")
    feature_password_reset: bool = Field(default=True, env="FEATURE_PASSWORD_RESET")
    feature_email_verification: bool = Field(default=False, env="FEATURE_EMAIL_VERIFICATION")
    feature_two_factor_auth: bool = Field(default=False, env="FEATURE_TWO_FACTOR_AUTH")
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False
        
    @validator("environment")
    def validate_environment(cls, v):
        allowed_envs = ["development", "staging", "production", "testing"]
        if v not in allowed_envs:
            raise ValueError(f"Environment must be one of: {allowed_envs}")
        return v
    
    @validator("log_level")
    def validate_log_level(cls, v):
        allowed_levels = ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]
        if v.upper() not in allowed_levels:
            raise ValueError(f"Log level must be one of: {allowed_levels}")
        return v.upper()
    
    @validator("rust_hash_algorithm")
    def validate_hash_algorithm(cls, v):
        allowed_algorithms = ["argon2", "bcrypt", "scrypt"]
        if v not in allowed_algorithms:
            raise ValueError(f"Hash algorithm must be one of: {allowed_algorithms}")
        return v
    
    @property
    def is_development(self) -> bool:
        return self.environment == "development"
    
    @property
    def is_production(self) -> bool:
        return self.environment == "production"
    
    @property
    def is_testing(self) -> bool:
        return self.testing or self.environment == "testing"
    
    @property
    def database_url_sync(self) -> str:
        """Synchronous database URL for SQLAlchemy."""
        return str(self.database_url).replace("postgresql://", "postgresql+psycopg2://")
    
    @property
    def database_url_async(self) -> str:
        """Asynchronous database URL for SQLAlchemy."""
        return str(self.database_url).replace("postgresql://", "postgresql+asyncpg://")


class DevelopmentSettings(Settings):
    """Development environment settings."""
    debug: bool = True
    reload: bool = True
    log_level: str = "DEBUG"
    database_echo: bool = True


class ProductionSettings(Settings):
    """Production environment settings."""
    debug: bool = False
    reload: bool = False
    log_level: str = "INFO"
    database_echo: bool = False
    workers: int = 4


class TestingSettings(Settings):
    """Testing environment settings."""
    testing: bool = True
    debug: bool = True
    database_name: str = "test_fastapi_db"
    redis_db: int = 1
    
    @validator("database_url", pre=True)
    def assemble_test_db_connection(cls, v: Optional[str], values: dict) -> str:
        if v and "test" in v:
            return v
        return PostgresDsn.build(
            scheme="postgresql",
            user=values.get("database_user"),
            password=values.get("database_password"),
            host=values.get("database_host"),
            port=str(values.get("database_port")),
            path=f"/test_{values.get('database_name') or 'fastapi_db'}",
        )


@lru_cache()
def get_settings() -> Settings:
    """Get application settings with caching."""
    environment = os.getenv("ENVIRONMENT", "development").lower()
    
    if environment == "production":
        return ProductionSettings()
    elif environment == "testing":
        return TestingSettings()
    else:
        return DevelopmentSettings()


# Global settings instance
settings = get_settings()