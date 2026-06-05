from __future__ import annotations
import sys
from pathlib import Path
from typing import Any

from workflow_kit.common.paths import resolve_existing_path
from workflow_kit.common.patching import apply_robust_patch

def apply_robust_patch_payload(
    *,
    file_path: str,
    patch_content: str,
    tool_version: str,
) -> dict[str, Any]:
    """
    Applies a Search-Replace block patch to a file using the common patching logic.
    """
    path = resolve_existing_path(file_path)
    success, message = apply_robust_patch(path, patch_content)
    
    return {
        "status": "ok" if success else "error",
        "tool_version": tool_version,
        "message": message,
        "file_path": str(path),
        "warnings": [] if success else [message],
    }
