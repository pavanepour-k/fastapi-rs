import json
import os
from typing import Any, Dict, Optional

import yaml

_schema_cache: Optional[Dict[str, Any]] = None
_schema_file_mtime: Optional[float] = None


def load_params_schema(path: Optional[str] = None, force_reload: bool = False) -> Dict[str, Any]:
    """Load parameter schema with hot-reload support."""
    global _schema_cache, _schema_file_mtime
    
    if path is None:
        path = os.environ.get("FASTAPI_PARAMS_SCHEMA", "params_schema.yaml")
    
    if not os.path.exists(path):
        return {"endpoints": {}, "definitions": {}}
    
    current_mtime = os.path.getmtime(path)
    
    if not force_reload and _schema_cache is not None and _schema_file_mtime == current_mtime:
        return _schema_cache
    
    try:
        if path.endswith((".yaml", ".yml")):
            with open(path, "r", encoding="utf-8") as f:
                _schema_cache = yaml.safe_load(f) or {"endpoints": {}, "definitions": {}}
        elif path.endswith(".json"):
            with open(path, "r", encoding="utf-8") as f:
                _schema_cache = json.load(f)
        else:
            raise ValueError(f"Unsupported schema format: {path}")
        
        _schema_file_mtime = current_mtime
        return _schema_cache
    except Exception as e:
        import logging
        logging.warning(f"Failed to load params schema from {path}: {e}")
        return {"endpoints": {}, "definitions": {}}


def get_param_schema(endpoint: str, param_name: str) -> Optional[Dict[str, Any]]:
    """Get parameter schema with safe fallback."""
    schema = load_params_schema()
    try:
        return schema["endpoints"][endpoint]["params"][param_name]
    except (KeyError, TypeError):
        return None