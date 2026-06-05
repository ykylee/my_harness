#!/usr/bin/env python3
"""Smoke checks for markdown docs in the standard AI workflow repository."""

from __future__ import annotations

import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
REQUIRED_METADATA = [
    "문서 목적",
    "범위",
    "대상 독자",
    "상태",
    "최종 수정일",
    "관련 문서",
]
ENGLISH_METADATA = [
    "Purpose",
    "Scope",
    "Audience",
    "Status",
    "Updated",
    "Related docs",
]
LINK_PATTERN = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
SKIP_PREFIXES = ("http://", "https://", "mailto:", "#")
IGNORED_PARTS = {
    ".git",
    ".codex",
    ".opencode",
    ".venv",
    "venv",
    "env",
    ".venv-review",
    ".venv-copilot",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    ".ai-workflow-backups",
    ".tmp-ai-workflow-runtime-hidden",
    "tmp",
    "tmp-test",
    "scratch",
    "templates",
    "releases",
    "archive",
    "tests",
}
IGNORED_PREFIX_PARTS = (".venv",)
IGNORED_AI_WORKFLOW_SUBTREES = {
    ("ai-workflow", "core"),
    ("ai-workflow", "examples"),
    ("ai-workflow", "global-snippets"),
    ("ai-workflow", "harnesses"),
    ("ai-workflow", "mcp_servers"),
    ("ai-workflow", "schemas"),
    ("ai-workflow", "scripts"),
    ("ai-workflow", "skills"),
    ("ai-workflow", "templates"),
    ("ai-workflow", "workflow_kit"),
}
IGNORED_WORKFLOW_SOURCE_SUBTREES = {
    ("workflow-source", "core"),
    ("workflow-source", "examples"),
    ("workflow-source", "global-snippets"),
    ("workflow-source", "harnesses"),
    ("workflow-source", "mcp_servers"),
    ("workflow-source", "schemas"),
    ("workflow-source", "scripts"),
    ("workflow-source", "skills"),
    ("workflow-source", "templates"),
    ("workflow-source", "workflow_kit"),
}


def iter_markdown_files() -> list[Path]:
    markdown_files: list[Path] = []
    for path in REPO_ROOT.rglob("*.md"):
        if set(path.parts).intersection(IGNORED_PARTS):
            continue
        rel_parts = path.relative_to(REPO_ROOT).parts
        if any(
            part == "node_modules"
            or part in IGNORED_PARTS
            or any(part.startswith(prefix) for prefix in IGNORED_PREFIX_PARTS)
            for part in rel_parts
        ):
            continue
        if len(rel_parts) >= 2 and tuple(rel_parts[:2]) in IGNORED_AI_WORKFLOW_SUBTREES:
            continue
        if len(rel_parts) >= 2 and tuple(rel_parts[:2]) in IGNORED_WORKFLOW_SOURCE_SUBTREES:
            continue
        markdown_files.append(path)
    return sorted(markdown_files)


def normalize_link_target(raw_target: str) -> str:
    target = raw_target.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if "#" in target:
        target = target.split("#", 1)[0]
    return target


def check_metadata(path: Path) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    header_lines = lines[:20]
    required_metadata = REQUIRED_METADATA
    if has_metadata_set(header_lines, ENGLISH_METADATA):
        required_metadata = ENGLISH_METADATA
    missing = []
    for field in required_metadata:
        prefix = f"- {field}:"
        if not any(line.startswith(prefix) for line in header_lines):
            missing.append(field)
    if not lines or not lines[0].startswith("# "):
        missing.append("제목 헤더")
    return missing


def has_metadata_set(lines: list[str], fields: list[str]) -> bool:
    return all(any(line.startswith(f"- {field}:") for line in lines) for field in fields)


def check_links(path: Path) -> list[str]:
    errors: list[str] = []
    text = path.read_text(encoding="utf-8")
    for match in LINK_PATTERN.finditer(text):
        raw_target = normalize_link_target(match.group(1))
        if not raw_target or raw_target.startswith(SKIP_PREFIXES):
            continue
        if "://" in raw_target:
            continue
        if raw_target == "ai-workflow" or raw_target.startswith("ai-workflow/") or raw_target.startswith("../ai-workflow/"):
            continue
        target_path = (path.parent / raw_target).resolve()
        if not target_path.exists():
            errors.append(f"broken link `{raw_target}`")
    return errors


def main() -> int:
    failures: list[str] = []
    for path in iter_markdown_files():
        rel_path = path.relative_to(REPO_ROOT)
        missing_metadata = check_metadata(path)
        if missing_metadata:
            failures.append(
                f"{rel_path}: missing metadata fields: {', '.join(missing_metadata)}"
            )
        link_errors = check_links(path)
        for error in link_errors:
            failures.append(f"{rel_path}: {error}")

    targeted_phrases = {
        Path("workflow-source/core/global_workflow_standard.md"): [
            "task delegation 과 결과 통합에 집중하는 구성을 기본값으로 둔다.",
            "ask 는 genuinely blocking decision 이나 위험한 외부 작업으로만 좁히는 편을 기본 원칙으로 둔다.",
        ],
        Path("workflow-source/core/workflow_agent_topology.md"): [
            "메인 오케스트레이터가 직접 도구 호출을 수행하는 패턴은 기본값이 아니며",
            "권장 툴 성격: task-only delegation",
        ],
    }
    for rel_path, snippets in targeted_phrases.items():
        text = (REPO_ROOT / rel_path).read_text(encoding="utf-8")
        for snippet in snippets:
            if snippet not in text:
                failures.append(f"{rel_path}: missing required phrase `{snippet}`")

    if failures:
        print("Document smoke check failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(f"Document smoke check passed for {len(iter_markdown_files())} markdown files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
