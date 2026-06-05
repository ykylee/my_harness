"""Git utilities for workflow automation."""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
from pathlib import Path

@dataclass
class CommitEntry:
    subject: str
    hash: str
    author: str
    date: str
    category: str

def get_git_log(repo_path: str | Path, commit_range: str) -> List[str]:
    try:
        cmd = ["git", "-C", str(repo_path), "log", "--pretty=format:%s|%h|%an|%ad", "--date=iso", commit_range]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        if not result.stdout.strip():
            return []
        return result.stdout.strip().split("\n")
    except subprocess.CalledProcessError:
        return []

def categorize_message(message: str) -> str:
    m = message.lower()
    if any(kw in m for kw in ["feat", "add", "implement"]): return "Feature"
    if any(kw in m for kw in ["fix", "bug", "patch"]): return "Bug Fix"
    if any(kw in m for kw in ["docs", "readme", "markdown"]): return "Docs"
    if any(kw in m for kw in ["refactor", "clean", "simplify"]): return "Refactor"
    if any(kw in m for kw in ["test", "spec", "check"]): return "Test"
    if any(kw in m for kw in ["chore", "config", "build", "ci", "deps"]): return "Chore"
    return "Other"

def process_logs(logs: List[str]) -> List[CommitEntry]:
    entries = []
    for line in logs:
        parts = line.split("|")
        if len(parts) < 4: continue
        subject, h, author, date = parts[0], parts[1], parts[2], parts[3]
        category = categorize_message(subject)
        entries.append(CommitEntry(subject, h, author, date, category))
    return entries

def generate_git_markdown_summary(entries: List[CommitEntry], commit_range: str) -> str:
    if not entries:
        return f"No commits found in range `{commit_range}`."

    cat_map: Dict[str, List[str]] = {}
    for e in entries:
        if e.category not in cat_map: cat_map[e.category] = []
        cat_map[e.category].append(f"- {e.subject} (`{e.hash}`)")

    md = [f"### Git 작업 요약 ({commit_range})\n"]
    for cat in ["Feature", "Bug Fix", "Refactor", "Docs", "Test", "Chore", "Other"]:
        if cat in cat_map:
            md.append(f"#### {cat}")
            md.extend(cat_map[cat])
            md.append("")
    return "\n".join(md)

def summarize_git_history(repo_path: str | Path, commit_range: str) -> dict:
    logs = get_git_log(repo_path, commit_range)
    entries = process_logs(logs)
    return {
        "entries": entries,
        "markdown": generate_git_markdown_summary(entries, commit_range),
        "commit_count": len(entries)
    }
