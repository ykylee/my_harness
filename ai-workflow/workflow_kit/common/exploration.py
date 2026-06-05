"""Repository exploration and stack detection helpers."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

IGNORED_DIRS = {
    ".git", ".hg", ".svn", ".idea", ".vscode", "node_modules", ".next", ".turbo",
    "dist", "build", "coverage", "__pycache__", ".venv", "venv",
}

def iter_repo_files(root: Path, ignore_dirs: set[str] | None = None) -> list[Path]:
    results: list[Path] = []
    combined_ignore = IGNORED_DIRS.copy()
    if ignore_dirs:
        combined_ignore.update(ignore_dirs)

    for entry in sorted(root.iterdir()):
        if entry.name in combined_ignore:
            continue
        if entry.is_file():
            results.append(entry)
        elif entry.is_dir():
            results.extend(iter_repo_files(entry, ignore_dirs=ignore_dirs))
    return results

def detect_package_scripts(root: Path) -> dict[str, str]:
    package_json = root / "package.json"
    if not package_json.exists():
        return {}
    try:
        data = json.loads(package_json.read_text(encoding="utf-8"))
        return data.get("scripts", {})
    except:
        return {}

def guess_run_command(root: Path, package_scripts: dict[str, str]) -> str:
    if "start" in package_scripts:
        return "npm start"
    if "dev" in package_scripts:
        return "npm run dev"

    # Python fallback
    for candidate in ("app/main.py", "main.py", "src/main.py"):
        if (root / candidate).exists():
            return f"python {candidate}"

    return "TODO: 로컬 실행 명령 입력"

def analyze_repo_structure(root: Path, ignore_dirs: set[str] | None = None) -> dict[str, Any]:
    combined_ignore = IGNORED_DIRS.copy()
    if ignore_dirs:
        combined_ignore.update(ignore_dirs)

    top_level_entries = sorted([p.name for p in root.iterdir() if p.name not in combined_ignore])

    docs_dirs = sorted({n for n in ("docs", "doc", "wiki", "handbook") if (root / n).exists() and n not in combined_ignore})
    test_dirs = sorted({n for n in ("tests", "test", "spec", "__tests__") if (root / n).exists() and n not in combined_ignore})
    source_dirs = sorted({n for n in ("src", "app", "apps", "services", "packages", "lib") if (root / n).exists() and n not in combined_ignore})

    stack_labels: list[str] = []
    if (root / "package.json").exists(): stack_labels.append("node")
    if (root / "pyproject.toml").exists() or (root / "requirements.txt").exists(): stack_labels.append("python")
    if (root / "Cargo.toml").exists(): stack_labels.append("rust")
    if (root / "go.mod").exists(): stack_labels.append("go")
    if (root / "Gemfile").exists(): stack_labels.append("ruby")

    primary_stack = stack_labels[0] if stack_labels else "unknown"
    package_scripts = detect_package_scripts(root)

    return {
        "top_level_entries": top_level_entries,
        "docs_dirs": docs_dirs,
        "test_dirs": test_dirs,
        "source_dirs": source_dirs,
        "stack_labels": stack_labels,
        "primary_stack": primary_stack,
        "package_scripts": package_scripts
    }
