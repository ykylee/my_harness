#!/usr/bin/env python3
"""Apply and upgrade the standard AI workflow kit in a target repository."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Import shared constants and helpers from workflow_kit
# ---------------------------------------------------------------------------
REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as WORKFLOW_KIT_VERSION
from workflow_kit.constants import PRESERVE_RELATIVE_PATHS
from workflow_kit.gitignore import ensure_gitignore_patterns


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Upgrade the workflow kit in a target repository."
    )
    parser.add_argument(
        "--source-root",
        help="Path to the new version bundle or exported package root.",
    )
    parser.add_argument(
        "--target-root",
        default=".",
        help="Target repository root to upgrade.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show changes without applying them.",
    )
    parser.add_argument(
        "--setup-gitignore-only",
        action="store_true",
        help="Only run the .gitignore setup and exit.",
    )
    parser.add_argument(
        "--skip-gitignore",
        action="store_true",
        help="Do not modify .gitignore during upgrade.",
    )
    parser.add_argument(
        "--force-cleanup",
        action="store_true",
        help="Delete files in the target ai-workflow/ that are not in the new bundle. "
        "Requires --yes to actually delete (otherwise prints a preview).",
    )
    parser.add_argument(
        "--yes",
        "-y",
        action="store_true",
        help="Confirm destructive operations (e.g. --force-cleanup) without prompting.",
    )
    return parser.parse_args()


def get_file_hash(path: Path) -> str:
    hash_sha256 = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_sha256.update(chunk)
    return hash_sha256.hexdigest()


def get_current_version(target_root: Path) -> str:
    version_path = target_root / "ai-workflow" / "VERSION"
    if version_path.exists():
        return version_path.read_text(encoding="utf-8").strip()
    return "unknown"


def _is_preserved(rel_path: Path) -> bool:
    """Return True if *rel_path* is a preserved data path or lives under one."""
    for p in PRESERVE_RELATIVE_PATHS:
        if rel_path == p:
            return True
        # Check if rel_path is under a preserved directory
        try:
            rel_path.relative_to(p)
            return True
        except ValueError:
            continue
    return False


def apply_upgrade(
    source_bundle: Path,
    target_root: Path,
    dry_run: bool,
    force_cleanup: bool,
    confirmed: bool,
) -> dict[str, list[str]]:
    results: dict[str, list[str]] = {
        "created": [],
        "updated": [],
        "deleted": [],
        "preserved": [],
        "ignored": [],
    }

    # 1. Map files in source bundle
    source_files: dict[Path, Path] = {}
    for p in source_bundle.rglob("*"):
        if p.is_file():
            source_files[p.relative_to(source_bundle)] = p

    # 2. Map files in target ai-workflow/
    target_workflow_root = target_root / "ai-workflow"
    target_files: set[Path] = set()
    if target_workflow_root.exists():
        for p in target_workflow_root.rglob("*"):
            if p.is_file():
                target_files.add(p.relative_to(target_root))

    # 3. Handle specific version migrations (reserved for future use)
    # For example: if current_version < "v0.4.0": run_some_migration()

    # 4. Handle Update/Create
    for rel_path, src_path in source_files.items():
        dst_path = target_root / rel_path

        if _is_preserved(rel_path):
            if dst_path.exists():
                results["preserved"].append(str(rel_path))
                continue
            # If preserved path doesn't exist yet, create it from source
            results["created"].append(str(rel_path))
            if not dry_run:
                dst_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_path, dst_path)
            continue

        if dst_path.exists():
            src_hash = get_file_hash(src_path)
            dst_hash = get_file_hash(dst_path)

            if src_hash != dst_hash:
                results["updated"].append(str(rel_path))
                if not dry_run:
                    dst_path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(src_path, dst_path)
            else:
                results["ignored"].append(str(rel_path))
        else:
            results["created"].append(str(rel_path))
            if not dry_run:
                dst_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_path, dst_path)

    # 5. Handle Cleanup (Stale files in ai-workflow/)
    if force_cleanup:
        for rel_path in sorted(target_files):
            if rel_path not in source_files:
                if _is_preserved(rel_path):
                    continue
                results["deleted"].append(str(rel_path))
                if not dry_run and confirmed:
                    (target_root / rel_path).unlink()

    # 6. Update VERSION file (always overwritten with the new version)
    if not dry_run:
        version_path = target_root / "ai-workflow" / "VERSION"
        version_path.parent.mkdir(parents=True, exist_ok=True)
        version_path.write_text(WORKFLOW_KIT_VERSION, encoding="utf-8")

    return results


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    args = parse_args()
    target_root = Path(args.target_root).resolve()

    # --- gitignore-only mode ------------------------------------------------
    if args.setup_gitignore_only:
        print(f"--- Setting up .gitignore in {target_root} ---")
        changes = ensure_gitignore_patterns(target_root, args.dry_run)
        if changes:
            for c in changes:
                print(f"  - {c}")
        else:
            print("  - No changes needed.")
        sys.exit(0)

    # --- full upgrade mode --------------------------------------------------
    if not args.source_root:
        print("Error: --source-root is required for upgrade.")
        sys.exit(1)

    source_root = Path(args.source_root).resolve()

    # Find bundle root
    bundle_root = source_root / "bundle" if (source_root / "bundle").is_dir() else source_root

    if not (bundle_root / "ai-workflow").is_dir():
        print(f"Error: Could not find ai-workflow/ in {bundle_root}")
        sys.exit(1)

    current_version = get_current_version(target_root)
    print(f"--- Upgrading Workflow Kit in {target_root} ---")
    print(f"Current version: {current_version}")
    print(f"Target version:  {WORKFLOW_KIT_VERSION}")

    if args.dry_run:
        print("[DRY RUN] No changes will be written.")

    # Apply file changes
    confirmed = args.yes or args.dry_run
    results = apply_upgrade(bundle_root, target_root, args.dry_run, args.force_cleanup, confirmed)

    # Apply .gitignore changes
    gitignore_changes: list[str] = []
    if not args.skip_gitignore:
        gitignore_changes = ensure_gitignore_patterns(target_root, args.dry_run)

    # Print Summary
    print(f"\nSummary of changes:")
    print(f"- Created:   {len(results['created'])}")
    print(f"- Updated:   {len(results['updated'])}")
    print(f"- Deleted:   {len(results['deleted'])}")
    print(f"- Preserved: {len(results['preserved'])}")
    print(f"- Ignored:   {len(results['ignored'])} (already up to date)")
    print(f"- gitignore: {len(gitignore_changes)} changes")

    if results["updated"]:
        print("\nUpdated files:")
        for p in results["updated"]:
            print(f"  [OVERWRITE] {p}")

    if results["deleted"]:
        print("\nDeleted files (stale):")
        for p in results["deleted"]:
            print(f"  [DELETE]    {p}")

    if gitignore_changes:
        print("\nGitignore updates:")
        for c in gitignore_changes:
            print(f"  - {c}")

    # Safety check: --force-cleanup without --yes and without --dry-run
    if args.force_cleanup and results["deleted"] and not args.dry_run and not args.yes:
        print(f"\n⚠️  {len(results['deleted'])} file(s) would be deleted.")
        print("   Re-run with --yes to actually delete them.")
        print("   (The above is a preview; no files were deleted.)")
        sys.exit(0)

    print("\nUpgrade completed successfully.")


if __name__ == "__main__":
    main()
