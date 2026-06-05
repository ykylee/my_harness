"""Markdown link parsing helpers for workflow kit scripts."""

from __future__ import annotations

import os
import re
from pathlib import Path


LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def normalize_link_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if "#" in target:
        target = target.split("#", 1)[0]
    return target


def iter_markdown_link_targets(text: str) -> list[str]:
    targets: list[str] = []
    for match in LINK_RE.finditer(text):
        target = normalize_link_target(match.group(1))
        if not target or "://" in target or target.startswith("#"):
            continue
        targets.append(target)
    return targets


def markdown_targets(path: Path) -> list[str]:
    return iter_markdown_link_targets(path.read_text(encoding="utf-8"))


def resolve_relative_target(base: Path, raw_target: str) -> Path:
    return (base.parent / raw_target).resolve()


def rel_link_from_doc(doc_path: Path, target_path: Path) -> str:
    return os.path.relpath(target_path, start=doc_path.parent).replace(os.sep, "/")


def find_broken_links(path: Path) -> list[str]:
    broken: list[str] = []
    for raw_target in markdown_targets(path):
        if not resolve_relative_target(path, raw_target).exists():
            broken.append(raw_target)
    return broken

