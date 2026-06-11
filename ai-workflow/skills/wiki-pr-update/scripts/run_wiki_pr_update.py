#!/usr/bin/env python3
"""wiki-pr-update skill impl (D-80, 2026-06-11).

GitHub PR 1건의 metadata + touched file 을 LLM Wiki vault 의
``wiki/projects/<project>/prs/<num>.md`` 페이지로 자동 갱신한다.

- idempotency key ``pr-<num>-<head.sha>`` — 기존 page 의 frontmatter
  ``last_touched`` 와 비교해 skip / re-write 결정.
- ``--apply`` 시점에 ``prs/<num>.md`` 신규 작성 + cross-ref idempotent
  append + ``index.md`` / ``log.md`` 갱신.
- ``--reingest`` 는 mirror-list 7 patterns 매칭 source path list 만
  stdout 의 ``reingest_dispatch`` 에 추가 (실제 ``wiki-ingest-from-raw``
  호출은 wrapper 가 담당, D-72 §11.1 thin-wrapper 정공법).
- gh CLI 호출은 ``--gh-fetch`` 옵션 시점에만. 기본은 wrapper 가
  ``--pr-metadata <file>`` 로 전달한 JSON 을 읽는다.

stdlib only (Python 3.10+).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

TOOL_VERSION = "0.1.0"
VALID_PROJECTS = ("devhub", "my-harness")

# mirror-list 7 patterns (D-72 §11.1 정합, handoff §3.3)
MIRROR_PATTERNS = [
    r"^docs/adr/0\d{3}-.*\.md$",
    r"^docs/governance/.*\.md$",
    r"^docs/planning/.*\.md$",
    r"^docs/setup/.*\.md$",
    r"^docs/requirements\.md$",
    r"^docs/openapi\.yaml$",
    r"^ai-workflow/memory/(state\.json|session_handoff\.md|work_backlog\.md)$",
]
MIRROR_RE = [re.compile(p) for p in MIRROR_PATTERNS]

# GH pr view --json field (D-80 handoff §3.5)
GH_JSON_FIELDS = "number,title,author,state,mergedAt,headRefOid,files"


# ---------- data classes ----------

@dataclass
class PrMetadata:
    number: int
    title: str
    author: str
    state: str
    merged_at: str | None
    head_sha: str
    files: list[str] = field(default_factory=list)


# ---------- helpers ----------

def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")


def err(code: str, msg: str) -> dict:
    return {"error_code": code, "message": msg}


def fail(out: dict, code: str, msg: str) -> None:
    """Set ok=False and append an error."""
    out["ok"] = False
    out["errors"].append(f"{code}: {msg}")


def warn(out: dict, msg: str) -> None:
    out["warnings"].append(msg)


def resolve_vault(path_str: str) -> Path:
    return Path(path_str).expanduser().resolve()


def validate_vault(vault: Path, project: str, out: dict) -> bool:
    if not vault.is_dir():
        fail(out, "VAULT_NOT_FOUND", f"--vault-path {vault} is not a directory")
        return False
    agents = vault / "AGENTS.md"
    if not agents.is_file():
        fail(out, "AGENTS_MD_MISSING", f"vault marker {agents} not found")
        return False
    proj_dir = vault / "wiki" / "projects" / project
    if not proj_dir.is_dir():
        fail(out, "PROJECT_DIR_MISSING", f"project dir {proj_dir} not found")
        return False
    prs_dir = proj_dir / "prs"
    if not prs_dir.is_dir():
        # dry-run 도 directory 가 없으면 preview 단계에서 알리기 위해 warn 만
        warn(out, f"prs dir {prs_dir} not found (--apply 시 생성 시도)")
    index = proj_dir / "index.md"
    if not index.is_file():
        fail(out, "INDEX_MD_MISSING", f"index.md {index} not found")
        return False
    log = vault / "log.md"
    if not log.is_file():
        fail(out, "LOG_MD_MISSING", f"log.md {log} not found")
        return False
    return True


def load_metadata_file(path: Path, out: dict) -> PrMetadata | None:
    if not path.is_file():
        fail(out, "METADATA_READ_FAIL", f"--pr-metadata {path} not found")
        return None
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(out, "METADATA_READ_FAIL", f"{path}: {exc}")
        return None
    return parse_metadata(raw, out)


def parse_metadata(raw: dict, out: dict) -> PrMetadata | None:
    for key in ("number", "title", "state", "headRefOid"):
        if key not in raw:
            fail(out, "METADATA_READ_FAIL", f"missing required field '{key}'")
            return None
    files = raw.get("files") or []
    file_paths: list[str] = []
    for entry in files:
        if isinstance(entry, dict):
            p = entry.get("path")
            if p:
                file_paths.append(p)
        elif isinstance(entry, str):
            file_paths.append(entry)
    author = raw.get("author") or {}
    if isinstance(author, dict):
        author_login = author.get("login") or "unknown"
    else:
        author_login = "unknown"
    merged_at = raw.get("mergedAt") or None
    return PrMetadata(
        number=int(raw["number"]),
        title=str(raw["title"]),
        author=author_login,
        state=str(raw["state"]),
        merged_at=str(merged_at) if merged_at else None,
        head_sha=str(raw["headRefOid"]),
        files=sorted(set(file_paths)),
    )


def fetch_metadata_via_gh(pr: int, out: dict) -> PrMetadata | None:
    """--gh-fetch 모드: gh CLI 직접 호출."""
    try:
        proc = subprocess.run(
            ["gh", "pr", "view", str(pr), "--json", GH_JSON_FIELDS],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except FileNotFoundError:
        fail(out, "GH_NOT_FOUND", "gh CLI not found in PATH (wrapper fallback 사용 권장)")
        return None
    except subprocess.TimeoutExpired:
        fail(out, "GH_TIMEOUT", "gh pr view timeout (30s)")
        return None
    if proc.returncode != 0:
        fail(out, "GH_FAIL", f"gh pr view exit={proc.returncode}: {proc.stderr.strip()}")
        return None
    try:
        raw = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        fail(out, "METADATA_READ_FAIL", f"gh output parse: {exc}")
        return None
    return parse_metadata(raw, out)


def touched_to_titles(files: list[str]) -> list[str]:
    """repo-relative path → vault page title (kebab-case stem)."""
    titles: list[str] = []
    for f in files:
        stem = Path(f).stem
        title = re.sub(r"[^A-Za-z0-9._-]+", "-", stem).strip("-").lower()
        if title:
            titles.append(title)
    return titles


def classify_files(vault: Path, project: str, files: list[str]) -> dict:
    proj_dir = vault / "wiki" / "projects" / project
    src_dir = proj_dir / "sources"
    prs_dir = proj_dir / "prs"
    vault_source_existing: list[str] = []
    vault_pr_existing: list[str] = []
    raw_unmapped: list[str] = []
    other: list[str] = []
    for f in files:
        title = re.sub(r"[^A-Za-z0-9._-]+", "-", Path(f).stem).strip("-").lower()
        if (src_dir / f"{title}.md").is_file():
            vault_source_existing.append(title)
        elif (prs_dir / f"{title}.md").is_file():
            vault_pr_existing.append(title)
        elif any(rx.match(f) for rx in MIRROR_RE):
            raw_unmapped.append(f)
        else:
            other.append(f)
    return {
        "vault_source_existing": vault_source_existing,
        "vault_pr_existing": vault_pr_existing,
        "raw_unmapped": raw_unmapped,
        "other": other,
    }


def compute_idempotency_key(pr_number: int, head_sha: str) -> str:
    return f"pr-{pr_number}-{head_sha[:12]}"


def parse_frontmatter_last_touched(text: str) -> str | None:
    m = re.match(r"^---\r?\n([\s\S]*?)\r?\n---\r?\n?", text)
    if not m:
        return None
    fm = m.group(1)
    m2 = re.search(r"^last_touched:\s*['\"]?(\d{4}-\d{2}-\d{2})", fm, re.M)
    return m2.group(1) if m2 else None


def render_frontmatter(meta: PrMetadata, sources: list[str], related: list[str], today: str) -> str:
    tags = f"[pr, project-{meta.state.lower()}]"
    author = meta.author or "unknown"
    merged = meta.merged_at or "null"
    sources_yaml = json.dumps(sources)
    related_yaml = json.dumps(related)
    return (
        "---\n"
        f"title: \"PR #{meta.number}: {meta.title}\"\n"
        "type: pr\n"
        f"tags: {tags}\n"
        f"pr_number: {meta.number}\n"
        f"author: {author}\n"
        f"state: {meta.state}\n"
        f"merged_at: {merged}\n"
        f"head_sha: {meta.head_sha}\n"
        f"sources: {sources_yaml}\n"
        f"last_touched: {today}\n"
        f"related: {related_yaml}\n"
        "status: draft\n"
        "contradictions: [none]\n"
        "---\n"
    )


def render_body(meta: PrMetadata, classification: dict) -> str:
    parts: list[str] = []
    parts.append(f"# PR #{meta.number}: {meta.title}\n")
    parts.append(f"PR #{meta.number} 의 LLM Wiki vault 페이지 (D-80 wiki-pr-update 자동 생성).\n")
    parts.append("## State / Author / Head SHA\n")
    parts.append(f"- author: `{meta.author}`")
    parts.append(f"- state: `{meta.state}`")
    parts.append(f"- merged_at: `{meta.merged_at or 'none'}`")
    parts.append(f"- head_sha: `{meta.head_sha}`")
    parts.append("")
    if meta.files:
        parts.append("## Touched files")
        for f in meta.files:
            parts.append(f"- `{f}`")
        parts.append("")
    rel = classification.get("vault_source_existing", [])
    if rel:
        parts.append("## Related sources")
        for t in rel:
            parts.append(f"- [[{t}]]")
        parts.append("")
    parts.append("## Notes")
    parts.append("- 자동 생성 페이지 — 사용자 검토 후 status 를 `reviewed` 로 승격 가능.")
    parts.append("- idempotency: frontmatter `last_touched` + `head_sha` 가 같으면 재실행 시 skip.")
    parts.append("- 자동 Gitea remote push 없음 — 사용자 수동 push (AGENTS.md §6.5).")
    parts.append("")
    return "\n".join(parts)


def render_page(meta: PrMetadata, classification: dict) -> str:
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    sources = list(meta.files)
    related = [f"[[{t}]]" for t in classification.get("vault_source_existing", [])]
    fm = render_frontmatter(meta, sources, related, today)
    body = render_body(meta, classification)
    return fm + "\n" + body


def append_cross_ref_if_missing(src_page: Path, pr_num: int, out: dict) -> bool:
    """vault source page 에 `## Related prs` 섹션 idempotent append."""
    if not src_page.is_file():
        return False
    text = src_page.read_text(encoding="utf-8")
    marker = f"[[pr-{pr_num}]]"
    if marker in text:
        return False
    section_header = "## Related prs"
    if section_header in text:
        new_text = re.sub(
            r"(## Related prs\s*\n)([^\n]*\n)?",
            lambda m: m.group(0) + f"- {marker}\n",
            text,
            count=1,
        )
    else:
        new_text = text.rstrip("\n") + f"\n\n{section_header}\n\n- {marker}\n"
    src_page.write_text(new_text, encoding="utf-8")
    return True


def append_index_line(index_path: Path, pr_num: int, title: str, today: str, out: dict) -> bool:
    """`PRs` 섹션에 idempotent 1줄 append (없으면 섹션 생성)."""
    if not index_path.is_file():
        return False
    text = index_path.read_text(encoding="utf-8")
    marker = f"[PR #{pr_num}]"
    if marker in text:
        return False
    line = f"- {marker}(prs/{pr_num}.md) — {title} ({today})"
    section_header = "## PRs"
    if section_header in text:
        new_text = re.sub(
            r"(## PRs\s*\n)([^\n]*\n)?",
            lambda m: m.group(0) + line + "\n",
            text,
            count=1,
        )
    else:
        new_text = text.rstrip("\n") + f"\n\n{section_header}\n\n{line}\n"
    index_path.write_text(new_text, encoding="utf-8")
    return True


def append_log_line(log_path: Path, pr_num: int, title: str, project: str, today: str, out: dict) -> bool:
    text = log_path.read_text(encoding="utf-8") if log_path.is_file() else ""
    marker = f"pr-update | PR #{pr_num}"
    if marker in text:
        return False
    line = f"## [{today}] pr-update | PR #{pr_num} | {title} | project={project}"
    new_text = (text.rstrip("\n") + "\n" + line + "\n") if text else (line + "\n")
    log_path.write_text(new_text, encoding="utf-8")
    return True


def write_lint_report(lint_dir: Path, project: str, meta: PrMetadata, mode: str, summary: dict, out: dict) -> None:
    lint_dir.mkdir(parents=True, exist_ok=True)
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    report = lint_dir / f"pr_update_{today}.md"
    lines = [
        f"# PR Update Report — {today}",
        "",
        f"- vault: {out['vault_path']}",
        f"- project: {project}",
        f"- PR: #{meta.number} — {meta.title}",
        f"- head.sha: {meta.head_sha}",
        f"- mode: {mode}",
        f"- 검사 시각: {now_iso()}",
        f"- 검사자: wiki-pr-update {TOOL_VERSION}",
        f"- 결과: {summary['pages_created']} page created, {len(out['errors'])} errors",
        "",
        "## Preview",
        "| pr_number | title | head_sha | action | target_page |",
        "| --- | --- | --- | --- | --- |",
        f"| {meta.number} | {meta.title} | {meta.head_sha} | create | wiki/projects/{project}/prs/{meta.number}.md |",
        "",
        "## Touched files",
    ]
    for f in meta.files:
        lines.append(f"- {f}")
    lines.append("")
    report.write_text("\n".join(lines), encoding="utf-8")


# ---------- main ----------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="run_wiki_pr_update",
        description="wiki-pr-update skill impl (D-80)",
    )
    parser.add_argument("--pr", type=int, required=True, help="GitHub PR number")
    parser.add_argument("--vault-path", default="~/wiki", help="vault root (default: ~/wiki)")
    parser.add_argument("--project", default="devhub", choices=VALID_PROJECTS, help="target project")
    parser.add_argument("--pr-metadata", type=Path, help="gh pr view --json output file (wrapper 가 전달)")
    parser.add_argument("--touched-files", type=Path, help="gh pr diff --name-only output file (optional)")
    parser.add_argument("--apply", action="store_true", help="실제 vault 갱신 (default = dry-run)")
    parser.add_argument("--reingest", action="store_true", help="mirror-list 7 patterns 매칭 시 wiki-ingest-from-raw dispatch list 생성")
    parser.add_argument("--gh-fetch", action="store_true", help="gh CLI 로 metadata 자동 fetch (default: --pr-metadata file 사용)")
    parser.add_argument("--quiet", action="store_true", help="stderr 메시지 최소화")
    parser.add_argument("--output", choices=("json", "markdown", "both"), default="json")
    args = parser.parse_args(argv)

    out: dict = {
        "ok": True,
        "pr_number": args.pr,
        "pr_title": "",
        "head_sha": "",
        "vault_path": "",
        "project": args.project,
        "mode": "apply" if args.apply else "dry-run",
        "tool_version": TOOL_VERSION,
        "examined_at": now_iso(),
        "summary": {
            "touched_files": 0,
            "vault_source_files": 0,
            "pages_created": 0,
            "index_md_updates": 0,
            "log_md_appends": 0,
            "idempotent_skip": False,
        },
        "created": [],
        "appended": [],
        "warnings": [],
        "errors": [],
    }

    # ---- pr number validation ----
    if args.pr <= 0:
        fail(out, "INVALID_PR", f"--pr must be > 0 (got {args.pr})")
        return _emit(out, args)

    # ---- vault resolution + validation ----
    vault = resolve_vault(args.vault_path)
    out["vault_path"] = str(vault)
    if not validate_vault(vault, args.project, out):
        return _emit(out, args)

    # ---- metadata load (file or gh-fetch) ----
    meta: PrMetadata | None = None
    if args.gh_fetch:
        meta = fetch_metadata_via_gh(args.pr, out)
    elif args.pr_metadata is not None:
        meta = load_metadata_file(args.pr_metadata, out)
    else:
        fail(out, "METADATA_MISSING", "--pr-metadata <file> or --gh-fetch required")
    if meta is None:
        return _emit(out, args)

    # cross-check: pr number
    if meta.number != args.pr:
        warn(out, f"pr number mismatch: arg={args.pr}, metadata={meta.number}")
        meta.number = args.pr  # CLI 값을 SSOT 로

    out["pr_title"] = meta.title
    out["head_sha"] = meta.head_sha
    out["summary"]["touched_files"] = len(meta.files)

    # ---- touched-files cross-check (optional) ----
    if args.touched_files is not None and args.touched_files.is_file():
        try:
            tf_lines = [
                ln.strip() for ln in args.touched_files.read_text(encoding="utf-8").splitlines() if ln.strip()
            ]
            tf_set = set(tf_lines)
            meta_set = set(meta.files)
            if tf_set != meta_set:
                only_meta = sorted(meta_set - tf_set)
                only_tf = sorted(tf_set - meta_set)
                warn(
                    out,
                    f"touched-files mismatch: only-in-metadata={only_meta}, only-in-touched-files={only_tf}",
                )
        except OSError as exc:
            warn(out, f"--touched-files read fail: {exc}")

    # ---- classify touched files ----
    classification = classify_files(vault, args.project, meta.files)
    out["summary"]["vault_source_files"] = len(classification["vault_source_existing"])

    # ---- idempotency check ----
    prs_dir = vault / "wiki" / "projects" / args.project / "prs"
    target_page = prs_dir / f"{meta.number}.md"
    idem_key = compute_idempotency_key(meta.number, meta.head_sha)
    out["idempotency_key"] = idem_key  # extra field (디버깅용)
    if target_page.is_file():
        existing = target_page.read_text(encoding="utf-8")
        last_touched = parse_frontmatter_last_touched(existing)
        existing_sha_m = re.search(r"^head_sha:\s*['\"]?([0-9a-f]+)", existing, re.M)
        existing_sha = existing_sha_m.group(1) if existing_sha_m else None
        if last_touched is not None and existing_sha == meta.head_sha:
            # 같은 head.sha — idempotent skip
            out["summary"]["idempotent_skip"] = True
            if not args.quiet:
                print(f"[wiki-pr-update] PR #{meta.number} already updated (idem={idem_key}); skip", file=sys.stderr)
            return _emit(out, args)
        # head.sha 가 다르면 re-write (force-push / rebase 케이스)
        warn(out, f"head.sha changed: existing={existing_sha}, new={meta.head_sha}; re-writing")

    # ---- --reingest dispatch list (no actual call) ----
    reingest_dispatch: list[str] = []
    if args.reingest:
        reingest_dispatch = list(classification.get("raw_unmapped", []))
        if not reingest_dispatch:
            warn(out, "--reingest enabled but no mirror-list match in touched files")
    out["reingest_dispatch"] = reingest_dispatch  # wrapper 가 이 list 로 wiki-ingest-from-raw 호출

    # ---- dry-run exit ----
    if not args.apply:
        out["dry_run_target"] = str(target_page)
        return _emit(out, args)

    # ---- apply: write page + cross-ref + index/log ----
    try:
        prs_dir.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        fail(out, "PERMISSION_DENIED", f"mkdir {prs_dir}: {exc}")
        return _emit(out, args)

    page_text = render_page(meta, classification)
    target_page.write_text(page_text, encoding="utf-8")
    out["created"].append(str(target_page.relative_to(vault)))
    out["summary"]["pages_created"] = 1

    # cross-ref to vault source pages
    cross_ref_appends = 0
    for title in classification.get("vault_source_existing", []):
        src = vault / "wiki" / "projects" / args.project / "sources" / f"{title}.md"
        if append_cross_ref_if_missing(src, meta.number, out):
            cross_ref_appends += 1
    out["summary"]["cross_ref_appends"] = cross_ref_appends

    # index.md + log.md
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    index_path = vault / "wiki" / "projects" / args.project / "index.md"
    if append_index_line(index_path, meta.number, meta.title, today, out):
        out["summary"]["index_md_updates"] = 1
        out["appended"].append("index.md (1 line)")

    log_path = vault / "log.md"
    if append_log_line(log_path, meta.number, meta.title, args.project, today, out):
        out["summary"]["log_md_appends"] = 1
        out["appended"].append("log.md (1 line)")

    # lint report (markdown output)
    if args.output in ("markdown", "both"):
        lint_dir = vault / "_lint" / args.project
        try:
            write_lint_report(lint_dir, args.project, meta, out["mode"], out["summary"], out)
        except OSError as exc:
            warn(out, f"lint report write fail: {exc}")

    return _emit(out, args)


def _emit(out: dict, args: argparse.Namespace) -> int:
    """stdout 으로 JSON dump + exit code."""
    if not args.quiet and out.get("warnings"):
        for w in out["warnings"]:
            print(f"[wiki-pr-update] warn: {w}", file=sys.stderr)
    if not args.quiet and out.get("errors"):
        for e in out["errors"]:
            print(f"[wiki-pr-update] error: {e}", file=sys.stderr)
    print(json.dumps(out, ensure_ascii=False, indent=2))
    return 0 if out.get("ok") else 1


if __name__ == "__main__":
    sys.exit(main())
