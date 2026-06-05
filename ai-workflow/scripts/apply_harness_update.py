#!/usr/bin/env python3
"""Safely apply a workflow harness bundle into a target repository."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path


MANAGED_RELATIVE_PATHS = [
    Path("AGENTS.md"),
    Path("ai-workflow"),
    Path(".codex"),
    Path("opencode.json"),
    Path(".opencode"),
]

PRESERVE_RELATIVE_PATHS = [
    Path("ai-workflow/memory"),
    Path("ai-workflow/WORKFLOW_INDEX.md"),
    Path("ai-workflow/memory/PROJECT_PROFILE.md"),
    Path("ai-workflow/memory/session_handoff.md"),
    Path("ai-workflow/memory/state.json"),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Apply a workflow harness bundle into a target repository with backup-first replacement."
    )
    parser.add_argument(
        "--source-root",
        required=True,
        help="Path to an exported package root or directly to a bundle-like directory.",
    )
    parser.add_argument(
        "--target-root",
        required=True,
        help="Repository root that should receive the updated workflow files.",
    )
    parser.add_argument(
        "--backup-root",
        help="Optional root directory for backups. Defaults to <target-root>/.ai-workflow-backups.",
    )
    parser.add_argument(
        "--timestamp",
        help="Stable timestamp label for the backup directory. Defaults to current UTC time.",
    )
    parser.add_argument(
        "--manifest-out",
        help="Optional path to also write the JSON result manifest.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be backed up and replaced without writing files.",
    )
    parser.add_argument(
        "--preserve-data",
        action="store_true",
        help="Preserve user data (profile/handoff/state.json/backlog) during update.",
    )
    return parser.parse_args()


def file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tree_digest(path: Path) -> str:
    digest = hashlib.sha256()
    for child in sorted(path.rglob("*")):
        if child.is_dir():
            continue
        digest.update(child.relative_to(path).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(file_digest(child).encode("ascii"))
        digest.update(b"\0")
    return digest.hexdigest()


def path_signature(path: Path) -> dict[str, str] | None:
    if not path.exists():
        return None
    if path.is_file():
        return {"kind": "file", "digest": file_digest(path)}
    if path.is_dir():
        return {"kind": "dir", "digest": tree_digest(path)}
    return {"kind": "other", "digest": ""}


def detect_bundle_root(source_root: Path) -> Path:
    if (source_root / "bundle").is_dir():
        return source_root / "bundle"
    for rel_path in MANAGED_RELATIVE_PATHS:
        if (source_root / rel_path).exists():
            return source_root
    raise FileNotFoundError(
        "No bundle-like workflow payload found under source-root. Expected `bundle/` or managed workflow files."
    )


def collect_source_entries(bundle_root: Path) -> list[Path]:
    entries = [rel_path for rel_path in MANAGED_RELATIVE_PATHS if (bundle_root / rel_path).exists()]
    if not entries:
        raise FileNotFoundError("Bundle does not contain any managed workflow paths.")
    return entries


def remove_existing(path: Path) -> None:
    if not path.exists() and not path.is_symlink():
        return
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
        return
    path.unlink()


def copy_entry(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir():
        shutil.copytree(source, destination)
        return
    shutil.copy2(source, destination)


def is_preserved_path(path: Path, preserve_paths: set[Path]) -> bool:
    for preserved in preserve_paths:
        if path == preserved or preserved in path.parents:
            return True
    return False


def copy_entry_overlay(
    source: Path,
    destination: Path,
    *,
    target_root: Path,
    preserve_paths: set[Path],
) -> None:
    if not source.is_dir():
        copy_entry(source, destination)
        return

    for child in sorted(source.rglob("*")):
        rel_from_source = child.relative_to(source)
        rel_from_root = destination.relative_to(target_root) / rel_from_source
        if is_preserved_path(rel_from_root, preserve_paths):
            continue
        target_child = destination / rel_from_source
        if child.is_dir():
            target_child.mkdir(parents=True, exist_ok=True)
            continue
        target_child.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(child, target_child)


def backup_entry(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir() and not source.is_symlink():
        shutil.copytree(source, destination)
        return
    shutil.copy2(source, destination)


def build_manifest(
    *,
    source_root: Path,
    bundle_root: Path,
    target_root: Path,
    backup_dir: Path | None,
    managed_paths: list[Path],
    created_paths: list[str],
    updated_paths: list[str],
    skipped_paths: list[str],
    backed_up_paths: list[str],
    dry_run: bool,
) -> dict[str, object]:
    return {
        "status": "dry_run" if dry_run else "applied",
        "source_root": str(source_root),
        "bundle_root": str(bundle_root),
        "target_root": str(target_root),
        "backup_dir": str(backup_dir) if backup_dir is not None else None,
        "managed_paths": [path.as_posix() for path in managed_paths],
        "created_paths": created_paths,
        "updated_paths": updated_paths,
        "skipped_paths": skipped_paths,
        "backed_up_paths": backed_up_paths,
    }


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    target_root = Path(args.target_root).resolve()
    bundle_root = detect_bundle_root(source_root)
    managed_paths = collect_source_entries(bundle_root)

    timestamp = args.timestamp or datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    backup_root = Path(args.backup_root).resolve() if args.backup_root else target_root / ".ai-workflow-backups"

    created_paths: list[str] = []
    updated_paths: list[str] = []
    skipped_paths: list[str] = []
    backed_up_paths: list[str] = []

    changes_to_apply: list[tuple[Path, Path, bool]] = []
    preserve_path_strings = {p.as_posix() for p in PRESERVE_RELATIVE_PATHS}
    preserve_paths = {Path(path) for path in PRESERVE_RELATIVE_PATHS}
    for rel_path in managed_paths:
        if args.preserve_data and rel_path.as_posix() in preserve_path_strings:
            skipped_paths.append(rel_path.as_posix())
            continue
        source_path = bundle_root / rel_path
        target_path = target_root / rel_path
        source_signature = path_signature(source_path)
        target_signature = path_signature(target_path)
        if target_signature is None:
            created_paths.append(rel_path.as_posix())
            changes_to_apply.append((source_path, target_path, False))
            continue
        if source_signature == target_signature:
            skipped_paths.append(rel_path.as_posix())
            continue
        updated_paths.append(rel_path.as_posix())
        backed_up_paths.append(rel_path.as_posix())
        changes_to_apply.append((source_path, target_path, True))

    backup_dir = backup_root / timestamp if backed_up_paths else None

    if not args.dry_run:
        for source_path, target_path, needs_backup in changes_to_apply:
            rel_path = target_path.relative_to(target_root)
            if needs_backup and backup_dir is not None:
                backup_entry(target_path, backup_dir / rel_path)
            if args.preserve_data and source_path.is_dir():
                target_path.mkdir(parents=True, exist_ok=True)
                copy_entry_overlay(
                    source_path,
                    target_path,
                    target_root=target_root,
                    preserve_paths=preserve_paths,
                )
            else:
                remove_existing(target_path)
                copy_entry(source_path, target_path)

    manifest = build_manifest(
        source_root=source_root,
        bundle_root=bundle_root,
        target_root=target_root,
        backup_dir=backup_dir,
        managed_paths=managed_paths,
        created_paths=created_paths,
        updated_paths=updated_paths,
        skipped_paths=skipped_paths,
        backed_up_paths=backed_up_paths,
        dry_run=args.dry_run,
    )
    payload = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    if args.manifest_out:
        manifest_path = Path(args.manifest_out).resolve()
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(payload, encoding="utf-8")
    sys.stdout.write(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
