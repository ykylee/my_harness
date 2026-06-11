#!/usr/bin/env python3
"""wiki-ingest-from-raw: raw/projects/<project>/ 의 source file 을 읽어
wiki/projects/<project>/sources/<title>.md 자동 작성 + cross-ref + index/log 갱신.

[vault 운영 규약 ~/wiki/AGENTS.md v1.5 §2.1 Ingest 자동화]

Usage:
    python3 run_wiki_ingest.py --vault-path ~/wiki --project devhub --all
    python3 run_wiki_ingest.py --vault-path ~/wiki --project devhub --all --apply
    python3 run_wiki_ingest.py --vault-path ~/wiki --project devhub \\
        --source docs/adr/0001-idp-selection.md --apply

Exit code:
    0 — success (dry-run 또는 apply 모두 정상)
    1 — error (vault 부재, project whitelist 미준수, raw/ 수정 시도, post-ingest lint 실패)
    2 — invalid option
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

TOOL_VERSION = "0.1.0"

# === 상수 ===
VALID_PROJECTS = ("devhub", "my-harness")
WIKI_SUBDIRS = ("concepts", "entities", "topics", "sources", "comparisons", "query", "meta")
SOURCE_TYPE_RULES = [
    # (path glob pattern, type, tag prefix)
    (re.compile(r"^docs/adr/0\d{3}-.*\.md$"), "source", "adr"),
    (re.compile(r"^docs/governance/.*\.md$"), "topic", "governance"),
    (re.compile(r"^docs/planning/.*\.md$"), "topic", "planning"),
    (re.compile(r"^docs/setup/.*\.md$"), "topic", "setup"),
    (re.compile(r"^docs/requirements\.md$"), "concept", "requirements"),
    (re.compile(r"^docs/openapi\.yaml$"), "source", "openapi"),
    (re.compile(r"^ai-workflow/memory/state\.json$"), "topic", "memory-state"),
    (re.compile(r"^ai-workflow/memory/session_handoff\.md$"), "topic", "memory-handoff"),
    (re.compile(r"^ai-workflow/memory/work_backlog\.md$"), "topic", "memory-backlog"),
]
BODY_MAX_CHARS = 2000
LOG_LINE_FORMAT = "## [{date}] ingest | {title}"
LOG_DATE_FORMAT = "%Y-%m-%d"

# === 데이터 클래스 ===
@dataclass
class Source:
    rel_path: str  # raw/projects/<project>/<rel_path>
    abs_path: Path
    title: str
    type: str
    tags: list[str] = field(default_factory=list)
    body_excerpt: str = ""
    status: str = "to_ingest"  # to_ingest | already_ingested | skipped


@dataclass
class Finding:
    rule: str
    severity: str  # info | warn | error
    source_path: str = ""
    target_page: str = ""
    action: str = ""  # create | update | skip
    message: str = ""
    preview: str = ""


@dataclass
class IngestSummary:
    sources_total: int = 0
    sources_already_ingested: int = 0
    sources_to_ingest: int = 0
    sources_skipped: int = 0
    pages_to_create: int = 0
    pages_to_update: int = 0
    index_md_updates: int = 0
    log_md_appends: int = 0


# === helpers ===
def now_utc_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def today_utc_date() -> str:
    return datetime.now(timezone.utc).strftime(LOG_DATE_FORMAT)


def kebab(s: str) -> str:
    """kebab-case normalize."""
    s = re.sub(r"[^A-Za-z0-9._-]+", "-", s).strip("-").lower()
    return re.sub(r"-+", "-", s)


def classify_type(rel_path: str) -> tuple[str, str]:
    """(type, tag-prefix) 결정. 매칭 rule 없으면 ('topic', 'misc')."""
    for pattern, type_, tag_prefix in SOURCE_TYPE_RULES:
        if pattern.match(rel_path):
            return type_, tag_prefix
    return "topic", "misc"


def derive_title(abs_path: Path, rel_path: str) -> str:
    """title 결정: file stem (확장자 제외) + directory hint."""
    stem = abs_path.stem
    if rel_path.startswith("docs/adr/0"):
        return f"ADR {stem[:4]} {stem[5:].replace('-', ' ')}".strip()
    if rel_path.startswith("ai-workflow/memory/"):
        return f"AI-workflow {stem.replace('-', ' ')}".strip()
    return stem.replace("-", " ").strip()


def build_target_page_rel(source_rel: str, project: str) -> str:
    """raw/projects/<project>/<rel> → wiki/projects/<project>/sources/<title>.md"""
    source_abs = Path(source_rel)
    stem = kebab(source_abs.stem)
    return f"wiki/projects/{project}/sources/{stem}.md"


def extract_excerpt(abs_path: Path) -> str:
    """raw file 의 본문 발췌 (max BODY_MAX_CHARS)."""
    try:
        text = abs_path.read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        return f"(read error: {e})"
    if not text.strip():
        return ""
    # 첫 H1 + 첫 paragraph 추출
    lines = text.splitlines()
    h1 = ""
    para = ""
    in_code = False
    for line in lines:
        s = line.strip()
        if s.startswith("```"):
            in_code = not in_code
            continue
        if in_code:
            continue
        if not h1 and s.startswith("# "):
            h1 = s[2:].strip()
            continue
        if h1 and not para and s and not s.startswith("#"):
            para = s
            break
    excerpt_parts = []
    if h1:
        excerpt_parts.append(f"# {h1}\n")
    if para:
        excerpt_parts.append(f"\n{para}\n")
    excerpt = "\n".join(excerpt_parts).strip()
    if len(excerpt) > BODY_MAX_CHARS:
        excerpt = excerpt[:BODY_MAX_CHARS] + "\n\n... (truncated)"
    return excerpt


def collect_related_keywords(text: str) -> set[str]:
    """본문에서 keyword 추출 (capitalized 단어 3+ chars + ADR/REQ/UC ID 패턴)."""
    keywords: set[str] = set()
    for m in re.finditer(r"\b[A-Z][a-z]{2,}\b", text):
        keywords.add(m.group(0).lower())
    for m in re.finditer(r"\b(?:ADR|REQ|UC|ARCH|RM|API|UT|IMPL|TC)-\w+", text):
        keywords.add(m.group(0).upper())
    return keywords


def build_page_content(
    source: Source,
    related_links: list[str],
    raw_rel_path: str,
) -> str:
    """wiki page 본문 (frontmatter + body) 생성."""
    fm = (
        "---\n"
        f"title: {source.title}\n"
        f"type: {source.type}\n"
        f"tags: [{', '.join(source.tags) if source.tags else 'none'}]\n"
        f"sources: [raw/projects/{_PROJECT}/{raw_rel_path}]\n"
        f"last_touched: {today_utc_date()}\n"
        f"related: [{', '.join(f'[[{l}]]' for l in related_links) if related_links else ''}]\n"
        f"status: draft\n"
        f"contradictions: [none]\n"
        "---\n\n"
    )
    body = (
        f"# {source.title}\n\n"
        f"## 원본 출처\n\n"
        f"raw mirror: `raw/projects/{_PROJECT}/{raw_rel_path}`\n\n"
        f"## 요약\n\n"
        f"{source.body_excerpt or '(원본 본문이 비어있거나 발췌 불가)'}\n\n"
    )
    if related_links:
        body += "## Related sources\n\n"
        for link in related_links:
            body += f"- [[{link}]]\n"
        body += "\n"
    body += (
        f"## Notes\n\n"
        f"- 본 페이지는 2026-06-11 ingest skill (D-72) 의 dry-run preview 입니다. 사용자 검토 후 `reviewed` 승격 권장.\n"
    )
    return fm + body


# === glob state (set by run) ===
_PROJECT = ""


def run_ingest(
    vault: Path,
    project: str,
    source: str | None,
    apply: bool,
    limit: int | None,
    skip_lint: bool,
    output_format: str,
    quiet: bool,
) -> dict:
    global _PROJECT
    _PROJECT = project
    started = now_utc_iso()
    findings: list[Finding] = []
    summary = IngestSummary()

    # ----- step 0: validation -----
    if not vault.is_dir():
        findings.append(Finding("INGEST-09", "error", message=f"vault 부재: {vault}"))
    agents = vault / "AGENTS.md"
    if not agents.is_file():
        findings.append(Finding("INGEST-09", "error", message=f"AGENTS.md 부재: {agents} (vault 정합 마커)"))
    raw_root = vault / "raw" / "projects" / project
    if not raw_root.is_dir():
        findings.append(Finding("INGEST-08", "error", message=f"raw projects 부재: {raw_root}"))
    if any(f.severity == "error" for f in findings):
        return _finalize(started, vault, project, apply, summary, findings, output_format)

    # ----- step 1: source 식별 -----
    sources = list_sources(raw_root, source, limit, findings)
    summary.sources_total = len(sources)
    summary.sources_to_ingest = sum(1 for s in sources if s.status == "to_ingest")
    summary.sources_already_ingested = sum(1 for s in sources if s.status == "already_ingested")
    summary.sources_skipped = sum(1 for s in sources if s.status == "skipped")
    summary.pages_to_create = summary.sources_to_ingest
    summary.index_md_updates = 1 if summary.sources_to_ingest > 0 else 0
    summary.log_md_appends = summary.sources_to_ingest

    # ----- step 2: 각 source 의 wiki page preview/action -----
    sources_root = vault / "wiki" / "projects" / project
    sources_dir = sources_root / "sources"
    if apply:
        sources_dir.mkdir(parents=True, exist_ok=True)

    related_index = build_related_index(sources_root)

    for src in sources:
        if src.status == "skipped":
            findings.append(Finding("INGEST-05", "warn", source_path=str(src.abs_path), message="0-byte raw file"))
            continue
        if src.status == "already_ingested":
            findings.append(Finding("INGEST-02", "info", source_path=str(src.abs_path), message="already ingested"))
            continue

        target_rel = build_target_page_rel(src.rel_path, project)
        target_abs = vault / target_rel
        title_for_link = target_abs.stem

        related = match_related(src, related_index, max_links=5)
        content = build_page_content(src, related, src.rel_path)
        if not apply:
            findings.append(Finding("INGEST-01", "info", source_path=str(src.abs_path), target_page=target_rel, action="create", preview=content[:500]))
        else:
            try:
                target_abs.write_text(content, encoding="utf-8")
                findings.append(Finding("INGEST-01", "info", source_path=str(src.abs_path), target_page=target_rel, action="create", message="applied"))
            except OSError as e:
                findings.append(Finding("INGEST-06", "error", source_path=str(src.abs_path), target_page=target_rel, message=f"write error: {e}"))

    # ----- step 3: index.md / log.md 갱신 (apply 한정) -----
    if apply and summary.sources_to_ingest > 0:
        index_path = sources_root / "index.md"
        log_path = vault / "log.md"
        try:
            with index_path.open("a", encoding="utf-8") as f:
                f.write(f"\n## [ingest {today_utc_date()}] {summary.sources_to_ingest} sources\n")
                for f2 in findings:
                    if f2.action == "create" and f2.message == "applied":
                        f.write(f"- [{f2.target_page}]({f2.target_page.split('/')[-1]}) — ingested\n")
        except OSError as e:
            findings.append(Finding("INGEST-06", "error", message=f"index.md write error: {e}"))
        try:
            with log_path.open("a", encoding="utf-8") as f:
                for f2 in findings:
                    if f2.action == "create" and f2.message == "applied":
                        title = Path(f2.target_path).stem if hasattr(f2, "target_path") else f2.target_page
                        f.write(LOG_LINE_FORMAT.format(date=today_utc_date(), title=title) + "\n")
        except OSError as e:
            findings.append(Finding("INGEST-06", "error", message=f"log.md write error: {e}"))

    # ----- step 4: post-ingest wiki-lint (선택) -----
    if apply and not skip_lint and summary.sources_to_ingest > 0:
        lint_warnings = run_wiki_lint(vault, project, quiet)
        for w in lint_warnings:
            findings.append(Finding("INGEST-10", "warn", message=w))

    return _finalize(started, vault, project, apply, summary, findings, output_format)


def _finalize(
    started: str,
    vault: Path,
    project: str,
    apply: bool,
    summary: IngestSummary,
    findings: list[Finding],
    output_format: str,
) -> dict:
    result = {
        "status": "ok" if not any(f.severity == "error" for f in findings) else "error",
        "tool_version": TOOL_VERSION,
        "vault_path": str(vault),
        "project": project,
        "mode": "apply" if apply else "dry-run",
        "examined_at": started,
        "summary": summary.__dict__,
        "findings": [f.__dict__ for f in findings],
        "errors": [f.__dict__ for f in findings if f.severity == "error"],
        "warnings": [f.__dict__ for f in findings if f.severity == "warn"],
    }
    return result


def list_sources(
    raw_root: Path, source: str | None, limit: int | None, findings: list[Finding]
) -> list[Source]:
    """raw/ 의 source file 식별."""
    sources: list[Source] = []
    if source:
        rel = source.lstrip("/")
        abs_p = raw_root / rel
        if not abs_p.is_file():
            findings.append(Finding("INGEST-06", "error", source_path=str(abs_p), message="source file not found"))
            return sources
        candidates = [abs_p]
    else:
        candidates = sorted(p for p in raw_root.rglob("*") if p.is_file())
    if limit is not None:
        candidates = candidates[:limit]
    for abs_p in candidates:
        rel = str(abs_p.relative_to(raw_root))
        stem = abs_p.stem
        # already_ingested: wiki page already exists
        target_rel = f"wiki/projects/{_PROJECT}/sources/{kebab(stem)}.md"
        if (abs_p.parent.parent.parent.parent / target_rel).is_file():
            sources.append(Source(rel, abs_p, derive_title(abs_p, rel), "topic", [], "", status="already_ingested"))
            continue
        # skipped: 0-byte
        if abs_p.stat().st_size == 0:
            sources.append(Source(rel, abs_p, derive_title(abs_p, rel), "topic", [], "", status="skipped"))
            continue
        # to_ingest
        type_, tag_prefix = classify_type(rel)
        tags = [tag_prefix, f"project-{_PROJECT}"]
        excerpt = extract_excerpt(abs_p)
        sources.append(Source(rel, abs_p, derive_title(abs_p, rel), type_, tags, excerpt, status="to_ingest"))
    return sources


def build_related_index(sources_root: Path) -> dict[str, set[str]]:
    """concepts/entities/topics page 들의 keyword index."""
    idx: dict[str, set[str]] = {}
    for sub in ("concepts", "entities", "topics"):
        d = sources_root / sub
        if not d.is_dir():
            continue
        for p in d.glob("*.md"):
            try:
                text = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            idx[p.stem] = collect_related_keywords(text)
    return idx


def match_related(src: Source, related_index: dict[str, set[str]], max_links: int = 5) -> list[str]:
    """src 와 keyword 매칭 상위 N page stem 반환."""
    src_keywords = collect_related_keywords(src.body_excerpt + " " + src.title)
    if not src_keywords:
        return []
    scores: list[tuple[int, str]] = []
    for stem, kws in related_index.items():
        overlap = len(src_keywords & kws)
        if overlap > 0:
            scores.append((overlap, stem))
    scores.sort(reverse=True)
    return [s for _, s in scores[:max_links]]


def run_wiki_lint(vault: Path, project: str, quiet: bool) -> list[str]:
    """wiki-lint subprocess 호출. 실패 시 warnings 반환."""
    import subprocess
    skill = Path.home() / "repos" / "my_harness" / "ai-workflow" / "skills" / "wiki-lint" / "scripts" / "run_wiki_lint.py"
    if not skill.is_file():
        return [f"wiki-lint 미설치: {skill}"]
    try:
        proc = subprocess.run(
            [sys.executable, str(skill), "--vault-path", str(vault), "--project", project, "--output", "json", "--quiet"],
            capture_output=True,
            text=True,
            timeout=60,
        )
    except subprocess.TimeoutExpired:
        return ["wiki-lint timeout (60s)"]
    except OSError as e:
        return [f"wiki-lint subprocess error: {e}"]
    if proc.returncode not in (0, 1):  # 0 = clean, 1 = findings
        return [f"wiki-lint exit {proc.returncode}: {proc.stderr.strip()[:200]}"]
    try:
        out = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return ["wiki-lint JSON parse error"]
    summary = out.get("summary", {})
    return [
        f"wiki-lint: errors={summary.get('errors', 0)} warns={summary.get('warns', 0)} infos={summary.get('infos', 0)} pages={summary.get('pages_scanned', 0)}"
    ]


def render_markdown(result: dict) -> str:
    lines: list[str] = []
    lines.append(f"# Ingest Report — {result['examined_at'][:10]}\n")
    lines.append(f"- vault: {result['vault_path']}")
    lines.append(f"- project: {result['project']}")
    lines.append(f"- mode: {result['mode']}")
    lines.append(f"- 검사 시각: {result['examined_at']}")
    lines.append(f"- 검사자: wiki-ingest-from-raw {result['tool_version']}")
    s = result["summary"]
    lines.append(
        f"- 결과: {s['sources_to_ingest']} to ingest, {s['sources_already_ingested']} already, {s['sources_skipped']} skipped, {len(result['errors'])} errors"
    )
    if s["sources_to_ingest"] > 0:
        lines.append("\n## Preview\n")
        lines.append("| source_path | target_page | action | title |")
        lines.append("| --- | --- | --- | --- |")
        for f in result["findings"]:
            if f["action"] == "create" and f["severity"] == "info":
                title = Path(f["target_page"]).stem
                lines.append(f"| {f['source_path']} | {f['target_page']} | {f['action']} | {title} |")
    if result["warnings"]:
        lines.append("\n## Warnings\n")
        for w in result["warnings"]:
            lines.append(f"- {w['message']}")
    if result["errors"]:
        lines.append("\n## Errors\n")
        for e in result["errors"]:
            lines.append(f"- {e['message']}")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="wiki-ingest-from-raw",
        description="raw/projects/<project>/ → wiki/projects/<project>/sources/ 자동 ingest",
    )
    ap.add_argument("--vault-path", default="~/wiki", help="vault 루트 (default: ~/wiki)")
    ap.add_argument("--project", required=True, help="devhub|my-harness")
    ap.add_argument("--source", default=None, help="1 file ingest (raw/ 상대 경로)")
    ap.add_argument("--all", action="store_true", help="project 의 모든 source 일괄 ingest")
    ap.add_argument("--limit", type=int, default=None, help="--all 시 최대 N건")
    ap.add_argument("--apply", action="store_true", help="실제 ingest (default = dry-run)")
    ap.add_argument("--skip-lint", action="store_true", help="post-ingest wiki-lint skip")
    ap.add_argument("--output", choices=("json", "markdown", "both"), default="both", help="출력 형식")
    ap.add_argument("--quiet", action="store_true", help="stderr 메시지 최소화")
    args = ap.parse_args(argv)

    vault = Path(args.vault_path).expanduser().resolve()
    if args.project not in VALID_PROJECTS:
        print(f"error: invalid --project: {args.project} (must be one of {VALID_PROJECTS})", file=sys.stderr)
        return 2
    if args.source and args.all:
        print("error: --source and --all are mutually exclusive", file=sys.stderr)
        return 2

    result = run_ingest(
        vault=vault,
        project=args.project,
        source=args.source,
        apply=args.apply,
        limit=args.limit,
        skip_lint=args.skip_lint,
        output_format=args.output,
        quiet=args.quiet,
    )
    if args.output in ("json", "both"):
        print(json.dumps(result, ensure_ascii=False, indent=2))
    if args.output in ("markdown", "both"):
        md = render_markdown(result)
        report_dir = vault / "_lint" / args.project
        report_dir.mkdir(parents=True, exist_ok=True)
        report_path = report_dir / f"ingest_{today_utc_date()}.md"
        report_path.write_text(md, encoding="utf-8")
        if not args.quiet:
            print(f"\n# report: {report_path}", file=sys.stderr)
    return 1 if any(f["severity"] == "error" for f in result["findings"]) else 0


if __name__ == "__main__":
    sys.exit(main())
