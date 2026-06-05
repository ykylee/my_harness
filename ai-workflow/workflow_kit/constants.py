"""Shared constants for the standard AI workflow kit."""

from __future__ import annotations

from pathlib import Path

# ---------------------------------------------------------------------------
# .gitignore block markers – used to idempotently manage the workflow section
# ---------------------------------------------------------------------------
GITIGNORE_BLOCK_MARKER_START = "# === WORKFLOW KIT START ==="
GITIGNORE_BLOCK_MARKER_END = "# === WORKFLOW KIT END ==="

# ---------------------------------------------------------------------------
# Patterns that should appear between the markers in .gitignore
# ---------------------------------------------------------------------------
GITIGNORE_PATTERNS: list[str] = [
    "# Workflow layer (selective tracking)",
    "/ANTIGRAVITY.md",
    "/AGENTS.md",
    "/GEMINI.md",
    "",
    "# Exclude duplicated engine/tool copies",
    "/ai-workflow/scripts/",
    "/ai-workflow/skills/",
    "/ai-workflow/workflow_kit/",
    "/ai-workflow/mcp_servers/",
    "/ai-workflow/schemas/",
    "/ai-workflow/templates/",
    "/ai-workflow/examples/",
    "/ai-workflow/global-snippets/",
    "/ai-workflow/harnesses/",
    "/ai-workflow/tests/",
    "",
    "# Keep the data (memory) and core guides",
    "!/ai-workflow/README.md",
    "!/ai-workflow/WORKFLOW_INDEX.md",
    "!/ai-workflow/core/",
    "!/ai-workflow/memory/",
]

# ---------------------------------------------------------------------------
# Paths that are considered user data and should be preserved during upgrade.
# These are relative to the target repository root.
# NOTE: ai-workflow/VERSION is intentionally *not* listed here because the
#       upgrade script always overwrites it with the new version at the end.
# ---------------------------------------------------------------------------
PRESERVE_RELATIVE_PATHS: list[Path] = [
    Path("ai-workflow/memory"),
    Path("ai-workflow/WORKFLOW_INDEX.md"),
    Path("ai-workflow/README.md"),
]
