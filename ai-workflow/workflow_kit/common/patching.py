"""Core logic for robust SEARCH/REPLACE patching with fuzzy matching."""

from __future__ import annotations

import difflib
from pathlib import Path
from typing import Any, TypedDict


class PatchBlock(TypedDict):
    search: list[str]
    replace: list[str]


def normalize_lines(lines: list[str]) -> list[str]:
    """Return lines stripped of leading/trailing whitespace, ignoring empty lines."""
    return [line.strip() for line in lines if line.strip()]


def fuzzy_find_block(
    source_lines: list[str], 
    search_lines: list[str], 
    threshold: float = 0.8
) -> tuple[int, int]:
    """
    Find the most similar block in source_lines matching search_lines.
    Returns (start_index, length) in source_lines, or (-1, 0) if no match.
    """
    norm_search = normalize_lines(search_lines)
    if not norm_search:
        return -1, 0

    search_len = len(norm_search)

    valid_source_indices = []
    norm_source = []
    for i, line in enumerate(source_lines):
        if line.strip():
            norm_source.append(line.strip())
            valid_source_indices.append(i)

    if not norm_source:
        return -1, 0

    best_match_idx = -1
    best_length = 0
    best_ratio = 0.0

    # Sliding window search
    for i in range(len(norm_source)):
        # Allow slight variations in window size (+/- 2 lines)
        for size_diff in range(-2, 3):
            w_size = search_len + size_diff
            if w_size <= 0 or i + w_size > len(norm_source):
                continue

            window = norm_source[i : i + w_size]
            ratio = difflib.SequenceMatcher(None, window, norm_search).ratio()

            if ratio > best_ratio:
                best_ratio = ratio
                best_match_idx = valid_source_indices[i]
                end_idx = valid_source_indices[i + w_size - 1]
                best_length = end_idx - best_match_idx + 1

            if ratio == 1.0:
                break
        if best_ratio == 1.0:
            break

    if best_ratio >= threshold:
        return best_match_idx, best_length
    return -1, 0


def parse_patch_blocks(patch_content: str) -> list[PatchBlock]:
    """Parse SEARCH/REPLACE blocks from patch string."""
    blocks: list[PatchBlock] = []
    search_block: list[str] = []
    replace_block: list[str] = []
    state = "NONE"

    for line in patch_content.splitlines(keepends=True):
        if line.startswith("<<<<<<< SEARCH"):
            state = "SEARCH"
            search_block = []
            replace_block = []
            continue
        elif line.startswith("======="):
            if state == "SEARCH":
                state = "REPLACE"
            continue
        elif line.startswith(">>>>>>> REPLACE"):
            if state == "REPLACE":
                blocks.append({"search": search_block, "replace": replace_block})
            state = "NONE"
            continue

        if state == "SEARCH":
            search_block.append(line)
        elif state == "REPLACE":
            replace_block.append(line)

    return blocks


def apply_robust_patch(
    file_path: Path, 
    patch_content: str, 
    *, 
    dry_run: bool = False
) -> tuple[bool, str]:
    """
    Parse and apply SEARCH/REPLACE blocks to a file.
    Returns (success, message).
    """
    if not file_path.exists():
        return False, f"File not found: {file_path}"

    original_text = file_path.read_text(encoding="utf-8")
    original_lines = original_text.splitlines(keepends=True)
    lines = list(original_lines)

    blocks = parse_patch_blocks(patch_content)
    if not blocks:
        return False, "No valid SEARCH/REPLACE block found in patch content."

    # Apply blocks one by one
    for i, block in enumerate(blocks):
        search_lines = block["search"]
        replace_lines = block["replace"]

        start_idx, length = fuzzy_find_block(lines, search_lines)
        if start_idx == -1:
            return False, f"Could not find a reliable match for SEARCH block #{i+1}."

        lines = lines[:start_idx] + replace_lines + lines[start_idx + length :]

    new_content = "".join(lines)

    # Syntax check for Python files
    if file_path.suffix == ".py":
        try:
            compile(new_content, str(file_path), "exec")
        except SyntaxError as e:
            return False, f"Patch would result in SyntaxError: {e}"

    if not dry_run:
        file_path.write_text(new_content, encoding="utf-8")

    return True, f"Successfully applied {len(blocks)} patch block(s)."
