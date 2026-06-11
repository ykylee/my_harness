"""Smart update helpers for bootstrap and upgrade operations.

The standard AI workflow kit can be re-applied to a target repository
that already has files from a previous install. The previous behaviour
was to raise ``FileExistsError`` for any colliding destination unless
the caller passed ``--force``, which made bootstrap unusable in CI /
fresh-env scenarios.

This module implements the **smart update** policy (see
``workflow-source/core/upgrade_policy.md``):

1. Read the per-file VERSION marker (``<!-- standard-ai-workflow-kit: v0.5.10.1-beta -->``
   or the equivalent comment prefix for other file types) from both
   the source payload and the destination file, if present.
2. If neither side has a marker, fall back to SHA-256 content hash
   comparison.
3. Apply a 7-rule precedence to decide whether the destination should
   be ``CREATED``, ``UPDATED``, ``IGNORED``, or ``PRESERVED``.
4. Always treat ``PRESERVE_RELATIVE_PATHS`` (user data) as ``PRESERVED``,
   regardless of force flag.

The module also exposes a ``read_kit_version`` helper that reads the
top-level ``ai-workflow/VERSION`` file when present, used to short-circuit
per-file comparison when the target is already at or above the source
kit version.
"""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# VERSION marker constants
# ---------------------------------------------------------------------------

MARKER_KIT_ID = "standard-ai-workflow-kit"
# Matches: <optional comment prefix> standard-ai-workflow-kit: v<semver>
# Examples that match:
#   <!-- standard-ai-workflow-kit: v0.5.10.1-beta -->
#   # standard-ai-workflow-kit: v0.5.10.1-beta
#   // standard-ai-workflow-kit: v0.5.10.1-beta
# Examples that don't match:
#   standard-ai-workflow-kit: v0.5.10.1-beta  (no comment prefix)
#   <!-- standard-ai-workflow-kit: 0.5.10.1-beta -->  (no leading 'v')
MARKER_REGEX = re.compile(
    r"^\s*(?P<prefix>[\S ]{0,12})"
    r"standard-ai-workflow-kit:"
    r"\s+v(?P<version>\d+\.\d+\.\d+(?:\.\d+)?(?:-[A-Za-z0-9.]+)?)"
    r"\s*(?:-->\s*)?$"
)

# File-suffix → comment prefix used when stamping markers.
# For suffixes not listed, no marker is stamped (hash-only comparison).
COMMENT_PREFIX_BY_SUFFIX: dict[str, str] = {
    ".md": "<!-- ",
    ".markdown": "<!-- ",
    ".py": "# ",
    ".sh": "# ",
    ".bash": "# ",
    ".toml": "# ",
    ".yaml": "# ",
    ".yml": "# ",
    ".txt": "# ",
    ".cfg": "# ",
    ".ini": "# ",
    ".js": "// ",
    ".ts": "// ",
    ".tsx": "// ",
    ".jsx": "// ",
    ".go": "// ",
    ".rs": "// ",
    ".java": "// ",
    ".css": "/* ",
    ".html": "<!-- ",
}

# Suffixes that get NO marker (json has no inline-comment convention;
# binary files have no use). Hash-only fallback applies.
NO_MARKER_SUFFIXES: frozenset[str] = frozenset(
    {".json", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".zip", ".tar", ".gz"}
)

# Known pre-release ordering for the ``pre`` segment of a semver-like
# version. Anything not in this list is sorted lexically AFTER all
# known pre-releases (e.g. "dev" > "rc1").
KNOWN_PRE_RELEASE_ORDER: dict[str, int] = {
    "alpha": 1,
    "a": 1,
    "beta": 2,
    "b": 2,
    "rc": 3,
    "pre": 3,
    "preview": 3,
    "final": 99,
    "stable": 99,
    "": 100,  # no pre-release segment
}


# ---------------------------------------------------------------------------
# Action enum
# ---------------------------------------------------------------------------


class Action(str, Enum):
    """Outcome of a smart-update comparison.

    Used by both :func:`decide_action` and the manifest builders.
    """

    CREATE = "create"
    UPDATED = "updated"
    IGNORED = "ignored"
    PRESERVED = "preserved"


# ---------------------------------------------------------------------------
# Version comparison
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ParsedVersion:
    """A parsed semver-like version, e.g. ``v0.5.10.1-beta``.

    ``pre`` is the pre-release segment WITHOUT the leading dash, lowercased.
    """

    major: int
    minor: int
    patch: int
    pre: str

    @classmethod
    def parse(cls, raw: str) -> "ParsedVersion":
        """Parse ``v0.5.10.1-beta`` (4-part) or ``v0.5.10-beta`` (3-part) style.

        Leading 'v' is optional. 4-part versions are treated as
        ``major.minor.patch.hotfix`` (CalVer / PEP 440 style).
        """
        s = raw.strip()
        if s.startswith("v") or s.startswith("V"):
            s = s[1:]
        # Split off pre-release
        if "-" in s:
            base, pre = s.split("-", 1)
            pre = pre.lower()
        else:
            base = s
            pre = ""
        parts = base.split(".")
        if len(parts) not in (3, 4):
            raise ValueError(
                f"Version must have 3 or 4 numeric segments, got {raw!r}"
            )
        try:
            major = int(parts[0])
            minor = int(parts[1])
            patch = int(parts[2])
        except ValueError as exc:
            raise ValueError(
                f"Version segments must be integers, got {raw!r}"
            ) from exc
        return cls(major=major, minor=minor, patch=patch, pre=pre)

    def __str__(self) -> str:
        base = f"v{self.major}.{self.minor}.{self.patch}"
        if self.pre:
            return f"{base}-{self.pre}"
        return base

    def _pre_rank(self) -> int:
        return KNOWN_PRE_RELEASE_ORDER.get(self.pre, 50)

    def _key(self) -> tuple[int, int, int, int, int]:
        return (self.major, self.minor, self.patch, self._pre_rank(), 0)

    def __lt__(self, other: "ParsedVersion") -> bool:  # type: ignore[override]
        return self._key() < other._key()

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ParsedVersion):
            return NotImplemented
        return self._key() == other._key()


def compare_marker(src_version: str, dst_version: str) -> int:
    """Compare two version strings.

    Returns -1 if ``src < dst``, 0 if equal, 1 if ``src > dst``.
    """
    s = ParsedVersion.parse(src_version)
    d = ParsedVersion.parse(dst_version)
    if s < d:
        return -1
    if s > d:
        return 1
    return 0


# ---------------------------------------------------------------------------
# Marker parsing / formatting
# ---------------------------------------------------------------------------


def parse_version_marker(text: str) -> Optional[str]:
    """Extract the kit version from a file's first few lines.

    Returns the version string (without leading 'v') if a marker is
    found, else ``None``. The marker may be on the first line of the
    file (most common) or within the first 5 lines (to accommodate
    files with a leading blank line or a separate comment block).
    """
    lines = text.splitlines()[:5]
    for line in lines:
        match = MARKER_REGEX.match(line)
        if match:
            return match.group("version")
    return None


def suffix_marker_supported(suffix: str) -> bool:
    """Return True if a file with the given suffix can have a stamped marker."""
    return suffix.lower() in COMMENT_PREFIX_BY_SUFFIX


def format_version_marker(version: str, suffix: str) -> str:
    """Build a marker line for stamping on a new file.

    ``suffix`` includes the leading dot (e.g. ``.md``).
    Returns the full marker line, ending with the appropriate comment
    closer for HTML-style prefixes.
    """
    prefix = COMMENT_PREFIX_BY_SUFFIX[suffix.lower()]
    closer = " -->" if prefix.strip() == "<!--" else ""
    return f"{prefix}standard-ai-workflow-kit: v{version}{closer}".rstrip()


def stamp_marker(content: str, version: str, suffix: str) -> str:
    """Prepend a version marker to a file's content, preserving the rest.

    The marker is placed on the very first line, followed by a blank
    line (if the content is non-empty and doesn't already start with a
    blank line).
    """
    if not suffix_marker_supported(suffix):
        return content
    marker = format_version_marker(version, suffix)
    # Avoid double-stamping
    if parse_version_marker(content) is not None:
        return content
    if not content:
        return f"{marker}\n"
    if content.startswith("\n"):
        return f"{marker}{content}"
    return f"{marker}\n\n{content}"


# ---------------------------------------------------------------------------
# Content hashing
# ---------------------------------------------------------------------------


def content_hash(data: bytes | str) -> str:
    """Compute a short SHA-256 content hash, hex-encoded, 16 chars."""
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.sha256(data).hexdigest()[:16]


def file_hash(path: Path) -> str:
    """Compute the short SHA-256 of a file's content."""
    return content_hash(path.read_bytes())


# ---------------------------------------------------------------------------
# Top-level VERSION file
# ---------------------------------------------------------------------------


def read_kit_version(target_root: Path) -> Optional[str]:
    """Read ``<target_root>/ai-workflow/VERSION`` if it exists.

    Returns the version string (without leading 'v', with pre-release
    segment if present), or ``None`` if the file doesn't exist.
    """
    version_path = target_root / "ai-workflow" / "VERSION"
    if not version_path.exists():
        return None
    raw = version_path.read_text(encoding="utf-8").strip()
    if not raw:
        return None
    s = raw
    if s.startswith("v") or s.startswith("V"):
        s = s[1:]
    return s


# ---------------------------------------------------------------------------
# The 7-rule decision
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Decision:
    """Result of a smart-update comparison for a single file."""

    action: Action
    src_marker: Optional[str] = None
    dst_marker: Optional[str] = None
    reason: str = ""

    def to_dict(self) -> dict[str, str]:
        out: dict[str, str] = {"action": self.action.value, "reason": self.reason}
        if self.src_marker is not None:
            out["src_marker"] = self.src_marker
        if self.dst_marker is not None:
            out["dst_marker"] = self.dst_marker
        return out


def decide_action(
    *,
    src_text: Optional[str],
    dst_text: Optional[str],
    src_path: Optional[Path] = None,
    dst_path: Optional[Path] = None,
    is_preserved_path: bool = False,
    force: bool = False,
) -> Decision:
    """Decide what to do with a single file.

    Parameters
    ----------
    src_text, dst_text
        The file content of the source (new) and destination (existing)
        sides. Pass ``None`` if the side doesn't exist.
    src_path, dst_path
        Optional paths, used only for marker extraction when the text
        is large and the marker is known to be at the top. Either
        ``*_text`` or ``*_path`` may be provided; text wins.
    is_preserved_path
        True if the destination path is under ``PRESERVE_RELATIVE_PATHS``.
        In that case the result is always ``PRESERVED`` regardless of
        ``force``.
    force
        If True, skips rules 3~6 (markers / hash comparison) and
        always returns ``UPDATED`` (or ``CREATE`` if destination
        doesn't exist). PRESERVED paths are still respected.
    """
    # PRESERVE only applies to an existing destination; a new file under
    # a preserved path (e.g. ai-workflow/memory/ on first bootstrap) is
    # still CREATEd since the user hasn't had a chance to put data there.
    src_exists = src_text is not None
    dst_exists = dst_text is not None

    if is_preserved_path and dst_exists:
        return Decision(
            action=Action.PRESERVED,
            reason="destination exists under PRESERVE_RELATIVE_PATHS (user data)",
        )

    if not src_exists:
        return Decision(action=Action.IGNORED, reason="no source content provided")

    if not dst_exists:
        if force:
            return Decision(
                action=Action.CREATE,
                reason="destination missing, force=True",
            )
        return Decision(
            action=Action.CREATE,
            reason="destination does not exist",
        )

    src_marker = parse_version_marker(src_text) if src_text else None
    dst_marker = parse_version_marker(dst_text) if dst_text else None
    src_hash = content_hash(src_text) if src_text is not None else None
    dst_hash = content_hash(dst_text) if dst_text is not None else None

    # force=True overrides marker/hash comparison; PRESERVE on existing
    # destinations was already handled at the top.
    if force:
        return Decision(
            action=Action.UPDATED,
            src_marker=src_marker,
            dst_marker=dst_marker,
            reason="force=True",
        )

    # Rules 3-6: marker / hash comparison.
    if src_marker and dst_marker:
        cmp = compare_marker(src_marker, dst_marker)
        if cmp > 0:
            # Source is newer → update.
            return Decision(
                action=Action.UPDATED,
                src_marker=src_marker,
                dst_marker=dst_marker,
                reason=f"source marker v{src_marker} newer than destination v{dst_marker}",
            )
        if cmp < 0:
            # Destination is newer → ignore (downgrade would be wrong).
            return Decision(
                action=Action.IGNORED,
                src_marker=src_marker,
                dst_marker=dst_marker,
                reason=(
                    f"destination marker v{dst_marker} is newer than source v{src_marker}; "
                    "refusing to downgrade"
                ),
            )
        # Markers equal → hash comparison.
        assert src_hash is not None and dst_hash is not None
        if src_hash == dst_hash:
            return Decision(
                action=Action.IGNORED,
                src_marker=src_marker,
                dst_marker=dst_marker,
                reason="markers match and content hash matches",
            )
        return Decision(
            action=Action.UPDATED,
            src_marker=src_marker,
            dst_marker=dst_marker,
            reason=(
                f"markers match (v{src_marker}) but content hash differs "
                f"({src_hash} vs {dst_hash}); user edit detected, overwriting"
            ),
        )

    # Rule 6: legacy file (no marker on either side or just one).
    assert src_hash is not None and dst_hash is not None
    if src_hash == dst_hash:
        return Decision(
            action=Action.IGNORED,
            src_marker=src_marker,
            dst_marker=dst_marker,
            reason="no markers, content hash matches (legacy file already up to date)",
        )
    return Decision(
        action=Action.UPDATED,
        src_marker=src_marker,
        dst_marker=dst_marker,
        reason=(
            f"no markers, content hash differs ({src_hash} vs {dst_hash}); "
            "treating as out of date"
        ),
    )


def is_path_preserved(rel_path: Path, preserved_paths: list[Path]) -> bool:
    """Return True if ``rel_path`` is under any of ``preserved_paths``."""
    for p in preserved_paths:
        if rel_path == p:
            return True
        try:
            rel_path.relative_to(p)
            return True
        except ValueError:
            continue
    return False


__all__ = [
    "Action",
    "Decision",
    "ParsedVersion",
    "compare_marker",
    "content_hash",
    "decide_action",
    "file_hash",
    "format_version_marker",
    "is_path_preserved",
    "parse_version_marker",
    "read_kit_version",
    "stamp_marker",
    "suffix_marker_supported",
    "COMMENT_PREFIX_BY_SUFFIX",
    "MARKER_KIT_ID",
    "MARKER_REGEX",
    "NO_MARKER_SUFFIXES",
]
