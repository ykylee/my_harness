"""Idempotent .gitignore management for the standard AI workflow kit.

Uses marker-based block management so that the workflow section can be
cleanly inserted, updated, or verified without risking duplicate entries.
"""

from __future__ import annotations

from pathlib import Path

from workflow_kit.constants import (
    GITIGNORE_BLOCK_MARKER_END,
    GITIGNORE_BLOCK_MARKER_START,
    GITIGNORE_PATTERNS,
)


def _build_block() -> str:
    """Return the full marker-wrapped block text."""
    lines = [GITIGNORE_BLOCK_MARKER_START] + GITIGNORE_PATTERNS + [GITIGNORE_BLOCK_MARKER_END]
    return "\n".join(lines)


def ensure_gitignore_patterns(target_root: Path, dry_run: bool) -> list[str]:
    """Ensure the workflow .gitignore patterns exist in *target_root*/.gitignore.

    Uses marker comments to manage the block idempotently:
    - If the file does not exist → create it with the full block.
    - If the markers already exist → replace the block content in-place.
    - If the markers do not exist but legacy patterns are detected → migrate to marked block.
    - Otherwise → append the full block.

    Returns a list of human-readable change descriptions (empty = no changes).
    """
    gitignore_path = target_root / ".gitignore"
    block = _build_block()

    # --- Case 1: .gitignore does not exist -----------------------------------
    if not gitignore_path.exists():
        if dry_run:
            return ["Create .gitignore with standard workflow patterns"]
        gitignore_path.write_text(block + "\n", encoding="utf-8")
        return ["Created .gitignore with standard workflow patterns"]

    content = gitignore_path.read_text(encoding="utf-8")
    changes: list[str] = []

    # --- Case 2: Markers already present → replace block in-place -----------
    if GITIGNORE_BLOCK_MARKER_START in content and GITIGNORE_BLOCK_MARKER_END in content:
        start_idx = content.index(GITIGNORE_BLOCK_MARKER_START)
        end_idx = content.index(GITIGNORE_BLOCK_MARKER_END) + len(GITIGNORE_BLOCK_MARKER_END)
        existing_block = content[start_idx:end_idx]
        if existing_block == block:
            return []  # already up to date
        new_content = content[:start_idx] + block + content[end_idx:]
        changes.append("Updated workflow patterns block in .gitignore")
        if not dry_run:
            gitignore_path.write_text(new_content, encoding="utf-8")
        return changes

    # --- Case 3: Legacy patterns without markers → migrate to markers -------
    if "/ai-workflow/scripts/" in content:
        changes.append("Migrated legacy .gitignore patterns to marked block")
        if not dry_run:
            # We append the marked block. Future runs will use Case 2.
            # We leave the legacy patterns as is to avoid accidental deletion,
            # but the new block will take precedence or be redundant.
            with gitignore_path.open("a", encoding="utf-8") as f:
                f.write("\n" + block + "\n")
        return changes

    # --- Case 4: No existing patterns → append full block -------------------
    changes.append("Appended standard workflow patterns to .gitignore")
    if not dry_run:
        with gitignore_path.open("a", encoding="utf-8") as f:
            f.write("\n" + block + "\n")
    return changes
