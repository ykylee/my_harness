"""File-write helpers and the manifest builder.

The bootstrap script needs three write primitives:

- :func:`write_text` writes a single file. **Smart update** is the
  default behaviour since v0.5.10.1: when the destination already
  exists, the file is compared by VERSION marker (priority) and
  content hash (fallback), and only overwritten when the comparison
  says the source is newer or the content has actually changed. The
  legacy ``FileExistsError`` behaviour is gone; use ``--force`` (passed
  via ``force=True``) to bypass the comparison.
- :func:`copy_core_docs` copies the kit's standard core docs and support
  paths into the generated kit. Same smart-update policy.
- :func:`build_manifest` collects everything the bootstrap script wrote
  into a single JSON-serialisable summary, so the entry-point can echo
  it back to the operator.

These helpers are pure functions on ``Paths`` and :class:`argparse.Namespace`;
they don't need to know about harnesses or MCP.

The smart-update helpers (VERSION marker parsing, content hash,
``decide_action``) live in :mod:`workflow_kit.upgrade_diff` and are
imported here. See ``workflow-source/core/upgrade_policy.md`` for the
full policy.
"""

from __future__ import annotations

import json
import shutil
from dataclasses import asdict
from pathlib import Path
from typing import Any

from bootstrap_lib.paths import Paths
from workflow_kit.upgrade_diff import (
    Action,
    Decision,
    decide_action,
    is_path_preserved,
    stamp_marker,
    suffix_marker_supported,
)


# Resolved kit version used as the per-file marker's version. Set
# lazily so a unit test can monkey-patch ``workflow_kit.__version__``.
_KIT_VERSION: str | None = None

# Module-global accumulator of file actions so harness renderers
# (which have no return-value channel) can record their actions.
# Bootstrap is single-threaded; ``__main__.main()`` drains it into
# the manifest and resets it.
_file_action_log: list[dict[str, str]] = []


def drain_file_actions() -> list[dict[str, str]]:
    """Return and clear the accumulated file actions."""
    out = list(_file_action_log)
    _file_action_log.clear()
    return out


def _kit_version() -> str:
    """Return the kit version string, with the leading 'v' stripped."""
    global _KIT_VERSION
    if _KIT_VERSION is None:
        from workflow_kit import __version__ as v  # local import for testability

        s = v.strip()
        if s.startswith("v") or s.startswith("V"):
            s = s[1:]
        _KIT_VERSION = s
    return _KIT_VERSION


def rel(path: Path, base: Path) -> str:
    """Return a relative path string for ``path`` against ``base``.

    Falls back to the absolute string when ``path`` is not under ``base``.
    """
    try:
        return str(path.resolve().relative_to(base.resolve()))
    except ValueError:
        return str(path.resolve())


# Paths that the smart update must NEVER overwrite, even with force.
# These mirror ``workflow_kit.constants.PRESERVE_RELATIVE_PATHS`` so
# the bootstrap layer can decide without an extra import. Keep in sync
# if PRESERVE_RELATIVE_PATHS is updated.
_PRESERVE_RELATIVE_PATHS: list[Path] = [
    Path("ai-workflow/memory"),
    Path("ai-workflow/WORKFLOW_INDEX.md"),
    Path("ai-workflow/README.md"),
]


def _is_preserved(rel_path: Path) -> bool:
    return is_path_preserved(rel_path, _PRESERVE_RELATIVE_PATHS)


def _read_text_or_none(path: Path) -> str | None:
    """Read a file as UTF-8 text; return None on binary or read failure."""
    try:
        return path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return None


def _copy_binary(
    source: Path,
    destination: Path,
    *,
    is_preserved: bool,
    force: bool,
    rel_to: Path | None = None,
) -> dict[str, str]:
    """Copy a binary file using a content-hash-only decision."""
    if is_preserved and destination.exists() and not force:
        decision = Decision(
            action=Action.PRESERVED,
            reason="binary destination exists under PRESERVE_RELATIVE_PATHS",
        )
    elif destination.exists():
        src_hash = content_hash(source.read_bytes())
        dst_hash = content_hash(destination.read_bytes())
        if src_hash == dst_hash and not force:
            decision = Decision(
                action=Action.IGNORED,
                reason=f"binary file, content hash matches ({src_hash})",
            )
        else:
            decision = Decision(
                action=Action.UPDATED,
                reason=(
                    f"binary file, content hash differs "
                    f"({src_hash} vs {dst_hash})" + (" (force=True)" if force else "")
                ),
            )
    else:
        decision = decide_action(
            src_text=None,
            dst_text=None,
            is_preserved_path=is_preserved,
            force=force,
        )
    if decision.action in (Action.CREATE, Action.UPDATED):
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    action = _make_file_action(destination, decision, rel_to)
    _file_action_log.append(action)
    return action


# ---------------------------------------------------------------------------
# File-action result — what ``write_text`` and ``copy_core_docs`` return
# so the caller can populate the manifest.
# ---------------------------------------------------------------------------


def _make_file_action(
    path: Path,
    decision: Decision,
    rel_to: Path | None = None,
) -> dict[str, str]:
    """Build a manifest-friendly dict for a single file decision."""
    out: dict[str, str] = {
        "path": str(path),
        "action": decision.action.value,
        "reason": decision.reason,
    }
    if rel_to is not None:
        out["rel"] = rel(path, rel_to)
    if decision.src_marker is not None:
        out["src_marker"] = decision.src_marker
    if decision.dst_marker is not None:
        out["dst_marker"] = decision.dst_marker
    return out


def write_text(
    path: Path,
    content: str,
    *,
    force: bool = False,
    src_version: str | None = None,
    rel_to: Path | None = None,
) -> dict[str, str]:
    """Write a single text file using the smart-update policy.

    The caller may pass ``src_version`` (the kit version that's the
    source of ``content``); when omitted, the live ``workflow_kit``
    version is used. The function stamps a VERSION marker on the
    content when (a) ``src_version`` is set or (b) the file suffix
    supports markers and the live kit version is known.

    Returns a manifest dict describing the action taken.
    """
    version = src_version if src_version is not None else _kit_version()
    suffix = path.suffix.lower()

    # Stamp the marker on the source content if the suffix supports it.
    if version and suffix_marker_supported(suffix):
        stamped = stamp_marker(content, version, suffix)
    else:
        stamped = content

    # Preserve check: if the relative path under target_root is under
    # PRESERVE_RELATIVE_PATHS, we never overwrite.
    if rel_to is not None:
        rel_path = Path(rel(path, rel_to))
    else:
        rel_path = path
    is_preserved = _is_preserved(rel_path)

    if path.exists():
        existing = path.read_text(encoding="utf-8")
        decision = decide_action(
            src_text=stamped,
            dst_text=existing,
            is_preserved_path=is_preserved,
            force=force,
        )
    else:
        decision = decide_action(
            src_text=stamped,
            dst_text=None,
            is_preserved_path=is_preserved,
            force=force,
        )

    if decision.action in (Action.CREATE, Action.UPDATED):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(stamped, encoding="utf-8")

    action = _make_file_action(path, decision, rel_to)
    _file_action_log.append(action)
    return action


def copy_core_docs(
    paths: Paths,
    *,
    force: bool = False,
    default_core_docs: list[str],
    default_core_support_paths: list[str],
    source_root: Path,
    src_version: str | None = None,
) -> list[dict[str, str]]:
    """Copy the kit's standard core docs + support paths into the generated kit.

    Returns a list of file-action dicts (one per file processed) suitable
    for inclusion in the bootstrap manifest.
    """
    version = src_version if src_version is not None else _kit_version()
    actions: list[dict[str, str]] = []
    rel_to = paths.target_root

    paths.core_dir.mkdir(parents=True, exist_ok=True)
    for name in default_core_docs:
        source = source_root / "core" / name
        destination = paths.core_dir / name
        suffix = destination.suffix.lower()
        raw_text = _read_text_or_none(source)
        if raw_text is None:
            # Binary file: copy as-is, no marker stamping.
            _copy_binary(source, destination, is_preserved=_is_preserved(Path(rel(destination, rel_to))), force=force, rel_to=rel_to)
            continue
        if version and suffix_marker_supported(suffix):
            stamped = stamp_marker(raw_text, version, suffix)
        else:
            stamped = raw_text

        rel_path = Path(rel(destination, rel_to))
        is_preserved = _is_preserved(rel_path)

        existing = _read_text_or_none(destination) if destination.exists() else None
        decision = decide_action(
            src_text=stamped,
            dst_text=existing,
            is_preserved_path=is_preserved,
            force=force,
        )

        if decision.action in (Action.CREATE, Action.UPDATED):
            shutil.copyfile(source, destination)
            destination.write_text(stamped, encoding="utf-8")

        actions.append(_make_file_action(destination, decision, rel_to))
        _file_action_log.append(actions[-1])

    for raw_relative_path in default_core_support_paths:
        relative_path = Path(raw_relative_path)
        source = source_root / relative_path
        destination = paths.kit_root / relative_path
        if source.is_dir():
            for file_path in sorted(source.rglob("*")):
                if not file_path.is_file():
                    continue
                nested_relative = file_path.relative_to(source_root)
                nested_destination = paths.kit_root / nested_relative
                nested_suffix = nested_destination.suffix.lower()
                nested_raw = _read_text_or_none(file_path)
                if nested_raw is None:
                    _copy_binary(
                        file_path,
                        nested_destination,
                        is_preserved=_is_preserved(Path(rel(nested_destination, rel_to))),
                        force=force,
                        rel_to=rel_to,
                    )
                    continue
                if version and suffix_marker_supported(nested_suffix):
                    nested_stamped = stamp_marker(nested_raw, version, nested_suffix)
                else:
                    nested_stamped = nested_raw

                nested_rel = Path(rel(nested_destination, rel_to))
                nested_preserved = _is_preserved(nested_rel)

                nested_existing = _read_text_or_none(nested_destination) if nested_destination.exists() else None
                nested_decision = decide_action(
                    src_text=nested_stamped,
                    dst_text=nested_existing,
                    is_preserved_path=nested_preserved,
                    force=force,
                )

                if nested_decision.action in (Action.CREATE, Action.UPDATED):
                    nested_destination.parent.mkdir(parents=True, exist_ok=True)
                    nested_destination.write_text(nested_stamped, encoding="utf-8")

                actions.append(_make_file_action(nested_destination, nested_decision, rel_to))
                _file_action_log.append(actions[-1])
            continue

        suffix = destination.suffix.lower()
        raw_text = _read_text_or_none(source)
        if raw_text is None:
            _copy_binary(
                source,
                destination,
                is_preserved=_is_preserved(Path(rel(destination, rel_to))),
                force=force,
                rel_to=rel_to,
            )
            continue
        if version and suffix_marker_supported(suffix):
            stamped = stamp_marker(raw_text, version, suffix)
        else:
            stamped = raw_text

        rel_path = Path(rel(destination, rel_to))
        is_preserved = _is_preserved(rel_path)

        existing = _read_text_or_none(destination) if destination.exists() else None
        decision = decide_action(
            src_text=stamped,
            dst_text=existing,
            is_preserved_path=is_preserved,
            force=force,
        )

        if decision.action in (Action.CREATE, Action.UPDATED):
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(stamped, encoding="utf-8")

        actions.append(_make_file_action(destination, decision, rel_to))
        _file_action_log.append(actions[-1])

    return actions


def build_manifest(
    args: Any,
    paths: Paths,
    core_docs: list[str],
    context: dict[str, object],
    harness_files: dict[str, str],
    dependency_files: list[str] | None,
    selected_harnesses: list[str],
    global_snippet_sources_fn: Any,
    file_actions: list[dict[str, str]] | None = None,
) -> dict[str, object]:
    """Collect everything the bootstrap wrote into a JSON-serialisable summary."""
    selected_snippets = {
        harness: global_snippet_sources_fn()[harness]
        for harness in selected_harnesses
        if harness in global_snippet_sources_fn()
    }
    generated_files: dict[str, str] = {
        "readme": str(paths.readme_path),
        "project_profile": str(paths.profile_path),
        "workflow_state": str(paths.state_path),
        "session_handoff": str(paths.handoff_path),
        "work_backlog": str(paths.backlog_index_path),
        "daily_backlog": str(paths.daily_backlog_path),
        "project_status_assessment": str(paths.status_assessment_path),
    }
    if getattr(args, "adoption_mode", "new") == "existing":
        generated_files["repository_assessment"] = str(paths.assessment_path)
    next_steps = [
        f"Open {rel(paths.profile_path, paths.target_root)} and replace TODO placeholders.",
        f"Refresh {rel(paths.state_path, paths.target_root)} after updating workflow docs.",
        f"Update {rel(paths.handoff_path, paths.target_root)} with the current session baseline.",
        f"Register the next real task in {rel(paths.daily_backlog_path, paths.target_root)}.",
    ]
    install_command = context.get("install_command")
    if dependency_files:
        next_steps.append(
            f"Run `{install_command}` to install recommended dependencies."
        )
    if getattr(args, "adoption_mode", "new") == "existing":
        next_steps.append(
            f"Review {rel(paths.assessment_path, paths.target_root)} and confirm inferred commands and doc paths."
        )
    if selected_snippets:
        next_steps.append(
            "Review the recommended global snippet before merging it into your harness-wide config."
        )

    # Surface the full stack detection so multi-language projects (e.g. a
    # Python backend + Go service + Node frontend) don't get reduced to
    # ``primary_stack`` alone. ``multi_stack`` is a convenience boolean for
    # downstream tools.
    stack_labels: list[str] = list(context.get("stack_labels", []) or [])
    multi_stack = len(stack_labels) > 1

    # Bucket the file actions for an at-a-glance summary.
    file_actions_summary: dict[str, list[dict[str, str]]] = {
        "created": [],
        "updated": [],
        "ignored": [],
        "preserved": [],
    }
    for fa in file_actions or []:
        action = fa.get("action")
        if action == "create":
            file_actions_summary["created"].append(fa)
        elif action == "updated":
            file_actions_summary["updated"].append(fa)
        elif action == "ignored":
            file_actions_summary["ignored"].append(fa)
        elif action == "preserved":
            file_actions_summary["preserved"].append(fa)

    return {
        "target_root": str(paths.target_root),
        "kit_root": str(paths.kit_root),
        "project_slug": args.project_slug,
        "project_name": args.project_name,
        "adoption_mode": args.adoption_mode,
        "harnesses": selected_harnesses,
        "primary_stack": context.get("primary_stack", "unknown"),
        "stack_labels": stack_labels,
        "multi_stack": multi_stack,
        "analysis_summary": context.get("analysis_summary", []),
        "warnings": list(context.get("warnings", []) or []),
        "generated_files": generated_files,
        "generated_harness_files": harness_files,
        "updated_dependency_files": dependency_files or [],
        "global_snippet_candidates": selected_snippets,
        "copied_core_docs": core_docs,
        "next_steps": next_steps,
        "file_actions": file_actions_summary,
    }


__all__ = [
    "build_manifest",
    "copy_core_docs",
    "rel",
    "write_text",
]
