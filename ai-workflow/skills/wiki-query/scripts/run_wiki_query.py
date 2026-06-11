#!/usr/bin/env python3
r"""wiki-query: ~/wiki/ vault 의 query 처리 (read-only + --file 시 query/ 페이지 file).

[vault 운영 규약 ~/wiki/AGENTS.md v1.5 §2.2 Query 자동화, D-79]

4 query primitive — ripgrep via subprocess (preferred) + Python stdlib regex fallback:
  1) Tag list       — `rg '\#[a-zA-Z0-9_-]+' --only-matching`
  2) Full-text      — `rg -w '<query>' --line-number --context 1 --json`
  3) Wikilink       — `rg '\[\[([^\]|]+)(?:\|[^\]]+)?\]\]' --only-matching`
  4) Frontmatter    — Python regex 8 key 파싱 (title/type/tags/sources/last_touched/related/status/contradictions)

Usage:
    # read-only (default)
    python3 run_wiki_query.py --vault-path ~/wiki --project devhub --query "Keycloak RBAC"
    # tag + type + limit filter
    python3 run_wiki_query.py --vault-path ~/wiki --project devhub --query "ADR-0020" --tag rbac --type concept --limit 5
    # JSON output
    python3 run_wiki_query.py --vault-path ~/wiki --project devhub --query "keycloak" --format json --output json
    # --file mode (AGENTS.md §2.2 6 step 자동)
    python3 run_wiki_query.py --vault-path ~/wiki --project devhub --query "ADR-0020 결정 사항" --file

Exit code:
    0 — success (read 또는 read+write 모두 성공, 0 results 도 success)
    1 — error (vault 부재, AGENTS.md 부재, project whitelist 미준수, --file write 실패, PERMISSION_DENIED)
    2 — invalid option (--query 부재, --project/--format/--type/--output enum 미준수)
"""
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

TOOL_VERSION = "0.1.0"

# === 상수 ===
VALID_PROJECTS = ("devhub", "my-harness")
VALID_FORMATS = ("md", "json", "plain")
VALID_OUTPUTS = ("json", "markdown", "both")
VALID_TYPES = ("concept", "entity", "topic", "source", "comparison", "query")
WIKI_SUBDIRS = ("concepts", "entities", "topics", "sources", "comparisons", "query", "meta")
REQUIRED_FRONTMATTER_KEYS = (
    "title", "type", "tags", "sources", "last_touched", "related", "status", "contradictions",
)
WIKILINK_RE = re.compile(r"\[\[([^\]|\n]+?)(?:\|[^\]]+)?\]\]")
FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n(.*)$", re.DOTALL)
H1_RE = re.compile(r"^# (.+)$", re.MULTILINE)
KEY_VALUE_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.*)$")
LIST_ITEM_RE = re.compile(r"^\s+-\s+(.*)$")
TAG_RE = re.compile(r"#([a-zA-Z0-9_-]+)")
WORD_BOUNDARY_RE_CACHE: dict[str, re.Pattern[str]] = {}

LOG_DATE_FORMAT = "%Y-%m-%d"
EXCERPT_MAX_CHARS = 2000


# === 데이터 클래스 ===
@dataclass
class Page:
    rel_path: str
    abs_path: Path
    project: str
    has_frontmatter: bool
    frontmatter: dict[str, Any]
    body: str
    body_excerpt: str = ""


@dataclass
class Hit:
    title: str
    type: str
    tags: list[str]
    path: str
    sources: list[str]
    last_touched: str
    excerpt: str
    links: list[str] = field(default_factory=list)
    backlinks: list[str] = field(default_factory=list)


# === helpers ===
def now_utc_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def today_utc_date() -> str:
    return datetime.now(timezone.utc).strftime(LOG_DATE_FORMAT)


def kebab(s: str) -> str:
    s = re.sub(r"[^A-Za-z0-9._-]+", "-", s).strip("-").lower()
    return re.sub(r"-+", "-", s)


def _rg_available() -> bool:
    return shutil.which("rg") is not None


def _rg_run(pattern: str, root: Path, glob: str = "*.md") -> list[dict[str, Any]]:
    """ripgrep 으로 pattern 매칭. 매치마다 {path, line, text, line_number, submatches} dict 반환.

    rg 부재 시 pure Python regex fallback 으로 동일 shape 결과 보장.
    """
    matches: list[dict[str, Any]] = []
    if _rg_available():
        try:
            proc = subprocess.run(
                ["rg", "--json", "--line-number", "--no-heading", "-g", glob, pattern, str(root)],
                capture_output=True, text=True, timeout=30, check=False,
            )
        except (subprocess.TimeoutExpired, OSError):
            return _python_run(pattern, root, glob)
        for line in proc.stdout.splitlines():
            try:
                evt = json.loads(line)
            except json.JSONDecodeError:
                continue
            if evt.get("type") == "match":
                data = evt.get("data", {})
                path = data.get("path", {}).get("text", "")
                submatches = data.get("submatches", [])
                sub_match = submatches[0]["match"]["text"] if submatches else ""
                matches.append({
                    "path": path,
                    "line_number": data.get("line_number", 0),
                    "text": data.get("lines", {}).get("text", "").rstrip("\n"),
                    "submatch": sub_match,
                })
    else:
        matches = _python_run(pattern, root, glob)
    return matches


def _python_run(pattern: str, root: Path, glob: str = "*.md") -> list[dict[str, Any]]:
    """rg 부재 시 pure Python regex fallback. _rg_run 과 동일 shape 반환."""
    matches: list[dict[str, Any]] = []
    try:
        regex = re.compile(pattern)
    except re.error:
        return matches
    for p in root.rglob(glob):
        if not p.is_file():
            continue
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for lineno, line in enumerate(text.splitlines(), start=1):
            m = regex.search(line)
            if not m:
                continue
            sub_match = m.group(0)
            if "only-matching" in pattern or "\\#[" in pattern or "[#]" in pattern:
                # tag/wikilink extract: only-matching
                sub_match = m.group(1) if m.lastindex else m.group(0)
            matches.append({
                "path": str(p),
                "line_number": lineno,
                "text": line,
                "submatch": sub_match,
            })
    return matches


def _coerce_scalar(v: str) -> Any:
    if v.lower() in ("true", "yes"):
        return True
    if v.lower() in ("false", "no"):
        return False
    if re.fullmatch(r"-?\d+", v):
        return int(v)
    return v


# === frontmatter 파서 (YAML subset, 안전) ===
def parse_frontmatter(text: str) -> tuple[dict[str, Any], str, bool]:
    m = FRONTMATTER_RE.match(text)
    if not m:
        return {}, text, False
    raw, body = m.group(1), m.group(2)
    fm: dict[str, Any] = {}
    current_list_key: str | None = None
    for line in raw.split("\n"):
        if not line.strip():
            current_list_key = None
            continue
        m_list = LIST_ITEM_RE.match(line)
        if m_list and current_list_key is not None:
            value = m_list.group(1).strip()
            if isinstance(fm.get(current_list_key), list):
                fm[current_list_key].append(_coerce_scalar(value))
            continue
        m_kv = KEY_VALUE_RE.match(line)
        if not m_kv:
            current_list_key = None
            continue
        key, value = m_kv.group(1), m_kv.group(2).strip()
        current_list_key = None
        if value == "":
            fm[key] = []
            current_list_key = key
        elif value in ("[]", "none", "null", "~"):
            fm[key] = [] if value == "[]" else None
        elif value.startswith("[") and value.endswith("]"):
            inner = value[1:-1].strip()
            if not inner:
                fm[key] = []
            else:
                fm[key] = [_coerce_scalar(p.strip().strip('"').strip("'")) for p in inner.split(",")]
        else:
            fm[key] = _coerce_scalar(value.strip('"').strip("'"))
    return fm, body, True


def _strip_link_brackets(raw: Any) -> str:
    s = str(raw).strip()
    if s.startswith("[[") and s.endswith("]]"):
        s = s[2:-2]
    return s.split("|")[0].split("#")[0].strip()


# === page discovery ===
def load_pages(vault: Path, project: str) -> list[Page]:
    pages: list[Page] = []
    for sub in WIKI_SUBDIRS:
        d = vault / "wiki" / "projects" / project / sub
        if not d.is_dir():
            continue
        for p in sorted(d.glob("*.md")):
            try:
                text = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            fm, body, has = parse_frontmatter(text)
            body_excerpt = _make_excerpt(body)
            pages.append(Page(
                rel_path=str(p.relative_to(vault)),
                abs_path=p,
                project=project,
                has_frontmatter=has,
                frontmatter=fm,
                body=body,
                body_excerpt=body_excerpt,
            ))
    return pages


def _make_excerpt(body: str) -> str:
    h1 = ""
    para = ""
    for line in body.splitlines():
        s = line.strip()
        if not h1 and s.startswith("# "):
            h1 = s[2:].strip()
            continue
        if h1 and not para and s and not s.startswith("#"):
            para = s
            break
    parts: list[str] = []
    if h1:
        parts.append(f"# {h1}")
    if para:
        parts.append(para)
    excerpt = "\n".join(parts).strip()
    if len(excerpt) > EXCERPT_MAX_CHARS:
        excerpt = excerpt[:EXCERPT_MAX_CHARS] + "\n\n... (truncated)"
    return excerpt


# === 4 query primitive ===
def page_matches_query(page: Page, query: str) -> bool:
    """3 primitive 매칭 (full-text + wikilink + frontmatter key). tag/type 은 filter 단계."""
    q = query.strip()
    if not q:
        return False
    q_lower = q.lower()

    # full-text: body + frontmatter values
    if re.search(rf"\b{re.escape(q_lower)}\b", page.body.lower()):
        return True
    for v in page.frontmatter.values():
        if isinstance(v, str) and q_lower in v.lower():
            return True
        if isinstance(v, list):
            for item in v:
                if isinstance(item, str) and q_lower in item.lower():
                    return True

    # wikilink: body 의 wikilink target 에 query 포함
    for m in WIKILINK_RE.finditer(page.body):
        target = m.group(1).split("|")[0].split("#")[0].strip()
        if target and (q_lower in target.lower() or target.lower() == q_lower):
            return True

    return False


def compute_backlinks(pages: list[Page], title: str) -> list[str]:
    """다른 page 의 wikilink 가 본 page 의 stem 을 가리키는지 검사."""
    title_lower = title.lower()
    backlinks: list[str] = []
    for other in pages:
        if other.abs_path.stem.lower() == title_lower:
            continue
        for m in WIKILINK_RE.finditer(other.body):
            target = m.group(1).split("|")[0].split("#")[0].strip()
            if target.lower() == title_lower:
                backlinks.append(other.abs_path.stem)
                break
        for raw in other.frontmatter.get("related") or []:
            target = _strip_link_brackets(raw)
            if target.lower() == title_lower:
                backlinks.append(other.abs_path.stem)
                break
    return sorted(set(backlinks))


# === hit building ===
def build_hit(page: Page, query: str) -> Hit:
    title = str(page.frontmatter.get("title") or page.abs_path.stem)
    type_ = str(page.frontmatter.get("type") or "topic")
    tags_raw = page.frontmatter.get("tags") or []
    tags = [str(t) for t in tags_raw] if isinstance(tags_raw, list) else []
    sources_raw = page.frontmatter.get("sources") or []
    sources = [str(s) for s in sources_raw] if isinstance(sources_raw, list) else []
    last_touched = str(page.frontmatter.get("last_touched") or "")

    links: list[str] = []
    for m in WIKILINK_RE.finditer(page.body):
        target = m.group(1).split("|")[0].split("#")[0].strip()
        if target and target not in links:
            links.append(target)
    for raw in page.frontmatter.get("related") or []:
        target = _strip_link_brackets(raw)
        if target and target not in links:
            links.append(target)

    backlinks = compute_backlinks([page], page.abs_path.stem)

    excerpt = _make_query_excerpt(page.body, query, page.body_excerpt)

    return Hit(
        title=title,
        type=type_,
        tags=tags,
        path=page.rel_path,
        sources=sources,
        last_touched=last_touched,
        excerpt=excerpt,
        links=links,
        backlinks=backlinks,
    )


def _make_query_excerpt(body: str, query: str, fallback: str) -> str:
    """query 와 매칭되는 본문 line 주변 (context 1) 발췌. 매칭 없으면 fallback."""
    q_lower = query.lower()
    for i, line in enumerate(body.splitlines()):
        if q_lower in line.lower():
            start = max(0, i - 1)
            end = min(len(body.splitlines()), i + 2)
            snippet = "\n".join(body.splitlines()[start:end]).strip()
            if len(snippet) > EXCERPT_MAX_CHARS:
                snippet = snippet[:EXCERPT_MAX_CHARS] + "\n\n... (truncated)"
            return snippet
    return fallback


# === 결과 출력 렌더링 ===
def render_json(result: dict[str, Any]) -> str:
    return json.dumps(result, ensure_ascii=False, indent=2)


def render_markdown(result: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append(f'# Query: "{result["query"]}"')
    lines.append(f"- vault: {result.get('vault_path', '')}")
    lines.append(f"- project: {result['project']}")
    lines.append(f"- mode: {result['mode']}")
    lines.append(f"- results: {result['hit_count']}")
    if result.get("warnings"):
        lines.append("- warnings:")
        for w in result["warnings"]:
            lines.append(f"  - {w}")
    lines.append("")
    lines.append("## Hits")
    lines.append("")
    for r in result.get("results", []):
        lines.append(f"### [[{r['title']}]] (type: {r['type']}, tags: [{', '.join(r['tags'])}])")
        lines.append(f"- path: {r['path']}")
        lines.append(f"- sources: [{', '.join(r['sources'])}]")
        lines.append(f"- last_touched: {r['last_touched']}")
        if r.get("backlinks"):
            lines.append(f"- backlinks: [{', '.join(r['backlinks'])}]")
        lines.append(f"- excerpt: {r['excerpt']}")
        lines.append("")
    return "\n".join(lines)


def render_plain(result: dict[str, Any]) -> str:
    lines: list[str] = []
    for r in result.get("results", []):
        lines.append(f"[{r['type']}] {r['path']} — {r['excerpt']}")
    return "\n".join(lines)


# === --file mode (6 step) ===
def build_query_page(
    hit: Hit,
    query: str,
    project: str,
    hit_count: int,
    date: str,
    topic: str,
    related_links: list[str],
) -> str:
    fm_lines: list[str] = ["---"]
    fm_lines.append(f'title: "{query} ({date})"')
    fm_lines.append("type: query")
    fm_tags = [kebab(query), f"project-{project}"]
    fm_lines.append(f"tags: [{', '.join(fm_tags)}]")
    fm_lines.append("sources: [none]")
    fm_lines.append(f"last_touched: {date}")
    if related_links:
        fm_lines.append(f"related: [{', '.join(f'[[{l}]]' for l in related_links)}]")
    else:
        fm_lines.append("related: [none]")
    fm_lines.append("status: draft")
    fm_lines.append("contradictions: [none]")
    fm_lines.append("---")
    fm = "\n".join(fm_lines) + "\n\n"

    body = (
        f"# {query}\n\n"
        f"## 질문\n\n{query}\n\n"
        f"## 사용 컨텍스트\n\n- 실행 시각: {date}\n- mode: file\n- project: {project}\n- hit_count: {hit_count}\n\n"
        f"## 답변\n\n"
    )
    if hit_count == 0:
        body += "(no hits — vault 에 query 와 매칭되는 page 없음)\n\n"
    else:
        for i, h in enumerate(related_links[:5], start=1):
            body += f"{i}. [[{h}]]\n"
        body += "\n상세 excerpt:\n\n"
        body += f"> (참고) 본 query 의 1차 후보 (max 5) — 본문은 wiki/ 참조. 총 {hit_count} hits.\n\n"
    body += f"## 후속 액션\n\n"
    if hit_count == 0:
        body += "- 후속 query 권장 (vault 의 다른 키워드 / 다른 project)\n"
        body += "- 또는 ingest source 추가 (raw/ 에 새 file → wiki-ingest-from-raw)\n"
    else:
        body += "- 후보 5건 본문 read 후 종합 답변 (Obsidian graph view)\n"
        body += "- 필요 시 `wiki-ingest-from-raw` 로 새 source file wiki page 화\n"
    body += f"\nFiled as [[query/{date}-{topic}]]\n"
    return fm + body


def apply_query_page(
    vault: Path,
    project: str,
    query: str,
    hits: list[Hit],
) -> dict[str, Any]:
    """AGENTS.md §2.2 step 3-5: query/ 페이지 file + log.md append + index.md 갱신.

    idempotent: 같은 date+topic 이미 존재 시 skip.
    """
    date = today_utc_date()
    topic = kebab(query)
    query_dir = vault / "wiki" / "projects" / project / "query"
    query_dir.mkdir(parents=True, exist_ok=True)
    query_page = query_dir / f"{date}-{topic}.md"

    side_effects: dict[str, Any] = {
        "query_page": "skipped",
        "log_md": "skipped",
        "index_md": "skipped",
    }
    warnings: list[str] = []

    if query_page.exists():
        warnings.append(f"query/ 페이지 이미 존재: {query_page.name} (idempotent skip)")
    else:
        related_links = [h.title for h in hits[:5]]
        content = build_query_page(
            hit=hits[0] if hits else Hit(title="", type="", tags=[], path="", sources=[], last_touched="", excerpt=""),
            query=query,
            project=project,
            hit_count=len(hits),
            date=date,
            topic=topic,
            related_links=related_links,
        )
        try:
            query_page.write_text(content, encoding="utf-8")
            side_effects["query_page"] = "created"
        except OSError as e:
            return {"side_effects": side_effects, "warnings": warnings, "error": f"query/ 페이지 write 실패: {e}"}

    # log.md append (idempotent)
    log_path = vault / "log.md"
    log_line = f"## [{date}] query | {topic}"
    if log_path.is_file():
        try:
            existing = log_path.read_text(encoding="utf-8")
        except OSError:
            existing = ""
    else:
        existing = ""
    if log_line in existing:
        warnings.append(f"log.md 에 같은 line 이미 존재 (idempotent skip)")
    else:
        try:
            with log_path.open("a", encoding="utf-8") as f:
                f.write(log_line + "\n")
            side_effects["log_md"] = "appended"
        except OSError as e:
            warnings.append(f"log.md write 실패: {e}")

    # index.md append (idempotent)
    index_path = vault / "wiki" / "projects" / project / "index.md"
    # `[[<date>-<topic>]]` (wikilink) 또는 `[<date>-<topic>](query/...)` (markdown) 양쪽 모두 매칭
    index_marker_wikilink = f"[[{date}-{topic}]]"
    index_marker_md = f"[{date}-{topic}](query/"
    if index_path.is_file():
        try:
            idx_existing = index_path.read_text(encoding="utf-8")
        except OSError:
            idx_existing = ""
        if index_marker_wikilink in idx_existing or index_marker_md in idx_existing:
            warnings.append(f"index.md 에 같은 link 이미 존재 (idempotent skip)")
        else:
            try:
                with index_path.open("a", encoding="utf-8") as f:
                    f.write(
                        f"\n- [{date}-{topic}](query/{date}-{topic}.md) — "
                        f"{query} ({len(hits)} hits, {date})\n"
                    )
                side_effects["index_md"] = "appended"
            except OSError as e:
                warnings.append(f"index.md write 실패: {e}")
    else:
        warnings.append(f"index.md 부재: {index_path} (수동 생성 필요)")

    return {"side_effects": side_effects, "warnings": warnings, "error": None}


# === 메인 query flow ===
def run_query(
    vault: Path,
    project: str,
    query: str,
    tag: str | None,
    type_: str | None,
    limit: int,
    file_mode: bool,
    output_format: str,
    quiet: bool,
) -> dict[str, Any]:
    started = now_utc_iso()
    result: dict[str, Any] = {
        "ok": False,
        "query": query,
        "project": project,
        "mode": "file" if file_mode else "no-file",
        "tool_version": TOOL_VERSION,
        "examined_at": started,
        "hit_count": 0,
        "results": [],
        "warnings": [],
        "errors": [],
    }

    # 5.1 사전 검증
    if not vault.is_dir():
        result["errors"].append(f"vault 부재: {vault}")
        return result
    agents = vault / "AGENTS.md"
    if not agents.is_file():
        result["errors"].append(f"AGENTS.md 부재: {agents} (vault 정합 마커)")
        return result
    index_md = vault / "index.md"
    if not index_md.is_file():
        result["errors"].append(f"index.md 부재: {index_md} (LLM query 의 첫 reading)")
        return result
    project_dir = vault / "wiki" / "projects" / project
    if not project_dir.is_dir():
        result["errors"].append(f"project 디렉터리 부재: {project_dir}")
        return result

    # 5.2 source 식별 — page scan + 4 query primitive
    pages = load_pages(vault, project)
    if not quiet:
        sys.stderr.write(f"[wiki-query] scanned {len(pages)} pages in {project_dir}\n")

    hits: list[Hit] = []
    truncated = False
    for page in pages:
        if not page_matches_query(page, query):
            continue
        if tag:
            tags_raw = page.frontmatter.get("tags") or []
            tags = [str(t) for t in tags_raw] if isinstance(tags_raw, list) else []
            if tag not in tags:
                continue
        if type_:
            type_raw = page.frontmatter.get("type")
            if type_raw != type_:
                continue
        hits.append(build_hit(page, query))

    # backlinks 는 vault 전체 scan 필요 → 후보 page 들에 대해서만
    if hits:
        all_pages = pages
        for h in hits:
            target_stem = Path(h.path).stem
            h.backlinks = compute_backlinks(all_pages, target_stem)

    if limit > 0 and len(hits) > limit:
        truncated = True
        hits = hits[:limit]

    result["hit_count"] = len(hits)
    result["results"] = [h.__dict__ for h in hits]
    if truncated:
        result["warnings"].append(f"hit_count > limit ({limit}), truncated")
    if not _rg_available():
        result["warnings"].append("rg (ripgrep) 부재 — pure Python regex fallback 사용")
    result["ok"] = True
    result["vault_path"] = str(vault)

    # 5.3-5.5 --file mode
    if file_mode:
        side = apply_query_page(vault, project, query, hits)
        if side.get("error"):
            result["errors"].append(side["error"])
            result["ok"] = False
        result["side_effects"] = side.get("side_effects", {})
        result["warnings"].extend(side.get("warnings", []))

    return result


# === CLI ===
def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="wiki-query",
        description="LLM Wiki vault query (D-79) — read-only + (--file 시) query/ 페이지 file + log.md/index.md 갱신",
    )
    ap.add_argument("--query", required=True, help="검색어 (full-text / wikilink / frontmatter key 매칭)")
    ap.add_argument("--vault-path", default="~/wiki", help="vault 루트 (기본: ~/wiki)")
    ap.add_argument("--project", default="devhub", choices=VALID_PROJECTS, help="대상 project (devhub|my-harness, 기본: devhub)")
    ap.add_argument("--tag", default=None, help="frontmatter tags: 필터 (AND, 단일)")
    ap.add_argument("--type", default=None, choices=VALID_TYPES, help="frontmatter type: 필터")
    ap.add_argument("--limit", type=int, default=20, help="최대 결과 수 (기본: 20, 0 이하 = 무제한)")
    ap.add_argument("--format", default="md", choices=VALID_FORMATS, help="출력 형식 (기본: md)")
    ap.add_argument("--file", dest="file_mode", action="store_true", help="query/ 페이지 file + log.md 1 line + index.md 1 line (AGENTS.md §2.2 step 3-5)")
    ap.add_argument("--no-file", dest="file_mode", action="store_false", help="read-only (default)")
    ap.add_argument("--quiet", action="store_true", help="stderr 메시지 최소화")
    ap.add_argument("--output", default="json", choices=VALID_OUTPUTS, help="output format (json|markdown|both, 기본: json)")
    ap.set_defaults(file_mode=False)
    args = ap.parse_args(argv)

    if not args.query.strip():
        sys.stderr.write("[wiki-query] error: --query required (empty not allowed)\n")
        return 2
    if args.limit < 0:
        sys.stderr.write("[wiki-query] error: --limit must be >= 0 (0 = 무제한)\n")
        return 2

    vault = Path(args.vault_path).expanduser().resolve()
    result = run_query(
        vault=vault,
        project=args.project,
        query=args.query,
        tag=args.tag,
        type_=args.type,
        limit=args.limit,
        file_mode=args.file_mode,
        output_format=args.format,
        quiet=args.quiet,
    )

    if not result["ok"]:
        sys.stderr.write(f"[wiki-query] error: {result['errors']}\n")
        if args.output in ("json", "both"):
            print(render_json(result))
        return 1

    if args.output in ("json", "both"):
        print(render_json(result))
    if args.output in ("markdown", "both"):
        md = render_markdown(result)
        if args.format == "json":
            sys.stderr.write(md + "\n")
        else:
            print(md)
    if args.format == "plain":
        print(render_plain(result))
    elif args.format == "md" and args.output == "json":
        # --format md 이지만 --output json 일 때: json 만 stdout
        pass

    if not args.quiet:
        sys.stderr.write(f"[wiki-query] DONE: {result['hit_count']} hits (mode={result['mode']})\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
