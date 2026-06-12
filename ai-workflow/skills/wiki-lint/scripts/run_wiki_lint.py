#!/usr/bin/env python3
"""wiki-lint: 검사 ~/wiki/ vault 무결성 (L01~L10).

stdlib only (tomllib 포함, Python 3.11+). SSOT: ~/wiki/schema/lint_rules.md

v1.5 (D-71) — 단일 wiki/ 하위에서 검사.
D-72 (2026-06-10) — per-project 검사 지원. vault 구조:
    wiki/projects/<project>/<sub>/...
    wiki/cross/<sub>/...
  --project, --project-config CLI flag 추가.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

TOOL_VERSION = "0.2.0"

# === 검사 대상 (wiki/ 하위, per-project + cross) ===
WIKI_SUBDIRS = ("concepts", "entities", "topics", "sources", "comparisons", "query", "meta")
DEFAULT_PROJECTS = ("my-harness", "devhub")

REQUIRED_FRONTMATTER = ("title", "type", "tags", "last_touched", "related", "status")
VALID_TYPES = {"concept", "entity", "topic", "source", "comparison", "query"}
VALID_STATUS = {"draft", "reviewed", "stale"}
STALE_DAYS = 90
LOG_STALE_DAYS = 7
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
WIKI_LINK_RE = re.compile(r"\[\[([^\]\n]+?)\]\]")
FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n(.*)$", re.DOTALL)
INBOUND_RULE_KEYS = {"related"}


@dataclass
class Finding:
    rule: str
    severity: str  # error | warn | info
    path: str
    message: str
    extra: dict[str, Any] = field(default_factory=dict)


@dataclass
class Page:
    rel_path: str
    abs_path: Path
    has_frontmatter: bool
    frontmatter: dict[str, Any]
    body: str
    inbound_from: set[str] = field(default_factory=set)
    project: str = ""  # D-72: per-project 라벨 (e.g. "my-harness", "devhub", "cross")


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
        if line.startswith("  - ") and current_list_key is not None:
            value = line[4:].strip()
            if isinstance(fm.get(current_list_key), list):
                fm[current_list_key].append(_coerce_scalar(value))
            continue
        m2 = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.*)$", line)
        if not m2:
            current_list_key = None
            continue
        key, value = m2.group(1), m2.group(2).strip()
        current_list_key = None
        if value == "":
            fm[key] = []
            current_list_key = key
        elif value == "[]":
            fm[key] = []
        elif value == "none" or value == "null" or value == "~":
            fm[key] = None
        elif value.startswith("[") and value.endswith("]"):
            inner = value[1:-1].strip()
            if not inner:
                fm[key] = []
            else:
                fm[key] = [_coerce_scalar(p.strip().strip('"').strip("'")) for p in inner.split(",")]
        else:
            fm[key] = _coerce_scalar(value.strip('"').strip("'"))
    return fm, body, True


def _coerce_scalar(v: str) -> Any:
    if v.lower() in ("true", "yes"):
        return True
    if v.lower() in ("false", "no"):
        return False
    if re.fullmatch(r"-?\d+", v):
        return int(v)
    return v
def _iter_wiki_dirs(vault: Path, projects: list[str] | None) -> list[tuple[Path, str]]:
    """vault 의 wiki/ 하위 검사 디렉터리 + per-project 라벨 반환.

    projects 가 None 이면 자동 발견 (wiki/projects/*/).
    결과: [(abs_path, project_label), ...]
    """
    out: list[tuple[Path, str]] = []
    wiki_root = vault / "wiki"
    if not wiki_root.is_dir():
        return out

    # 1) wiki/projects/<project>/<sub>/...
    projects_root = wiki_root / "projects"
    if projects_root.is_dir():
        proj_list = projects if projects is not None else [
            p.name for p in sorted(projects_root.iterdir()) if p.is_dir()
        ]
        for proj in proj_list:
            proj_root = projects_root / proj
            if not proj_root.is_dir():
                continue
            for sub in WIKI_SUBDIRS:
                d = proj_root / sub
                if d.is_dir():
                    out.append((d, proj))

    # 2) wiki/cross/<sub>/... (project="cross")
    cross_root = wiki_root / "cross"
    if cross_root.is_dir():
        for sub in WIKI_SUBDIRS:
            d = cross_root / sub
            if d.is_dir():
                out.append((d, "cross"))

    return out


def load_pages(
    vault: Path, projects: list[str] | None = None
) -> list[Page]:
    pages: list[Page] = []
    for d, project in _iter_wiki_dirs(vault, projects):
        for p in sorted(d.glob("*.md")):
            try:
                text = p.read_text(encoding="utf-8")
            except OSError:
                continue
            fm, body, has = parse_frontmatter(text)
            pages.append(
                Page(
                    rel_path=str(p.relative_to(vault)),
                    abs_path=p,
                    has_frontmatter=has,
                    frontmatter=fm,
                    body=body,
                    project=project,
                )
            )
    return pages


def index_pages(pages: list[Page]) -> dict[str, str]:
    """stem (파일명) → rel_path"""
    return {p.abs_path.stem: p.rel_path for p in pages}


def extract_wiki_links(body: str) -> list[str]:
    return WIKI_LINK_RE.findall(body)


# === 규칙 구현 ===
def rule_l01(p: Page) -> list[Finding]:
    if not p.has_frontmatter:
        return [
            Finding("L01", "error", p.rel_path, "frontmatter 누락 (위키 페이지는 frontmatter 필수)")
        ]
    missing = [k for k in REQUIRED_FRONTMATTER if k not in p.frontmatter]
    if not missing:
        return []
    return [
        Finding(
            "L01",
            "error",
            p.rel_path,
            f"frontmatter 필수 필드 누락: {', '.join(missing)}",
            extra={"missing_fields": missing},
        )
    ]


def rule_l02(p: Page, idx: dict[str, str]) -> list[Finding]:
    findings: list[Finding] = []
    fm_related = p.frontmatter.get("related") or []
    if isinstance(fm_related, str):
        fm_related = [fm_related]
    for raw in fm_related:
        target = _strip_link_brackets(raw)
        if not target or target.lower() in ("none", "null", "-"):
            continue
        if target not in idx and not any(s == target or s.startswith(target + "-") for s in idx):
            findings.append(
                Finding(
                    "L02",
                    "error",
                    p.rel_path,
                    f"broken link in frontmatter related: [[{target}]]",
                    extra={"target": target, "source_field": "related"},
                )
            )
    for raw in extract_wiki_links(p.body):
        target = raw.split("|")[0].split("#")[0].strip()
        if target and target not in idx and not any(s == target or s.startswith(target + "-") for s in idx):
            findings.append(
                Finding(
                    "L02",
                    "error",
                    p.rel_path,
                    f"broken wiki link: [[{target}]]",
                    extra={"target": target, "source_field": "body"},
                )
            )
    return findings


def rule_l03(p: Page, idx: dict[str, str]) -> list[Finding]:
    if not p.inbound_from:
        return [
            Finding(
                "L03",
                "warn",
                p.rel_path,
                "고아 페이지 — 어떤 페이지에서도 inbound link 없음",
            )
        ]
    return []


def rule_l04(pages: list[Page], skip_paths: set[str] | None = None) -> list[Finding]:
    """중복 title 감지. L07 의 부분집합이지만, 'mod 진화' 와 'exact 중복' 구분 위해 별도 규칙.

    D-97: skip_paths 의 page 가 all_paths 에 포함된 finding 은 skip
    (cross-project 의도적 mirror 면제, D-72 정책과 정합).
    """
    by_title: dict[str, list[str]] = {}
    for p in pages:
        t = p.frontmatter.get("title")
        if isinstance(t, str) and t.strip():
            by_title.setdefault(t.strip().lower(), []).append(p.rel_path)
    findings: list[Finding] = []
    import fnmatch
    for title, paths in by_title.items():
        if len(paths) > 1:
            if skip_paths and any(fnmatch.fnmatch(p, pat) for p in paths for pat in skip_paths):
                continue
            findings.append(
                Finding(
                    "L04",
                    "warn",
                    paths[0],
                    f"동일 title 의 페이지 {len(paths)}개 (통합 권장): {title}",
                    extra={"all_paths": paths, "title": title},
                )
            )
    return findings


def rule_l05(p: Page, today: datetime) -> list[Finding]:
    raw = p.frontmatter.get("last_touched")
    if not isinstance(raw, str) or not DATE_RE.match(raw):
        return []
    try:
        touched = datetime.strptime(raw, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    except ValueError:
        return []
    age = (today - touched).days
    if age >= STALE_DAYS:
        return [
            Finding(
                "L05",
                "info",
                p.rel_path,
                f"stale — {age}일 미갱신 (>= {STALE_DAYS}일)",
                extra={"days_since_touched": age},
            )
        ]
    return []


def rule_l06(p: Page, vault: Path) -> list[Finding]:
    if not p.has_frontmatter:
        return []
    sources = p.frontmatter.get("sources") or []
    if not isinstance(sources, list):
        return []
    findings: list[Finding] = []
    for raw in sources:
        s = str(raw).strip()
        if not s or s.startswith(("http://", "https://")):
            continue
        candidate = (vault / s).resolve()
        if not candidate.exists():
            findings.append(
                Finding(
                    "L06",
                    "error",
                    p.rel_path,
                    f"sources 경로 부재: {s}",
                    extra={"missing_source": s},
                )
            )
    return findings


def rule_l07(pages: list[Page]) -> list[Finding]:
    """구조적 모순: 같은 title 의 두 페이지가 모두 status=reviewed 면 모순 가능성.

    L04 (warn) 와 차별점 — L04 는 모든 중복, L07 은 reviewed 충돌만.
    """
    by_title: dict[str, list[Page]] = {}
    for p in pages:
        t = p.frontmatter.get("title")
        if isinstance(t, str) and t.strip():
            by_title.setdefault(t.strip().lower(), []).append(p)
    findings: list[Finding] = []
    for title, ps in by_title.items():
        if len(ps) < 2:
            continue
        reviewed = [p for p in ps if p.frontmatter.get("status") == "reviewed"]
        if len(reviewed) >= 2:
            findings.append(
                Finding(
                    "L07",
                    "error",
                    reviewed[0].rel_path,
                    f"동일 title 의 reviewed 페이지 {len(reviewed)}개 (모순 가능성)",
                    extra={"all_paths": [p.rel_path for p in reviewed], "title": title},
                )
            )
    return findings

def rule_l07_one(p: Page, pages: list[Page]) -> list[Finding]:
    """rule_l07 의 single-page 변형. skip_paths 가 적용된 page 의 모순만 반환.

    page 가 reviewed 가 아니면 skip. 같은 title 의 reviewed 다른 page 가 있으면
    그 page 의 L07 finding 만 반환 (이미 다른 page 의 finding 에서 잡혔을 수 있지만,
    skip_paths 가 적용된 page 의 경우 본인이 직접 검출되어야 함).
    """
    if p.frontmatter.get("status") != "reviewed":
        return []
    title = p.frontmatter.get("title")
    if not isinstance(title, str) or not title.strip():
        return []
    title_lower = title.strip().lower()
    reviewed_peers = [
        other for other in pages
        if other is not p
        and other.frontmatter.get("status") == "reviewed"
        and isinstance(other.frontmatter.get("title"), str)
        and other.frontmatter.get("title", "").strip().lower() == title_lower
    ]
    if not reviewed_peers:
        return []
    all_reviewed = [p] + reviewed_peers
    return [
        Finding(
            "L07",
            "error",
            p.rel_path,
            f"동일 title 의 reviewed 페이지 {len(all_reviewed)}개 (모순 가능성)",
            extra={"all_paths": [x.rel_path for x in all_reviewed], "title": title_lower},
        )
    ]


def rule_l08(pages: list[Page], index_path: Path, skip_paths: set[str] | None = None) -> list[Finding]:
    if not index_path.is_file():
        return [
            Finding(
                "L08",
                "warn",
                str(index_path.relative_to(index_path.parent.parent))
                if index_path.parent.parent in index_path.parents
                else str(index_path),
                "index.md 부재 — 모든 wiki 페이지가 미등록",
            )
        ]
    try:
        text = index_path.read_text(encoding="utf-8")
    except OSError:
        return []
    registered: set[str] = set()
    for line in text.splitlines():
        for m in re.findall(r"`?([\w\-./]+\.md)`?", line):
            registered.add(m)
    findings: list[Finding] = []
    import fnmatch
    for p in pages:
        rel = p.rel_path
        if rel not in registered:
            if skip_paths and any(fnmatch.fnmatch(rel, pat) for pat in skip_paths):
                continue
            findings.append(
                Finding(
                    "L08",
                    "warn",
                    rel,
                    "index.md 에 미등록 wiki 페이지 (자동 추가 제안 가능)",
                )
            )
    return findings


def rule_l09(vault: Path, today: datetime) -> list[Finding]:
    log = vault / "log.md"
    if not log.is_file():
        return [
            Finding("L09", "info", "log.md", "log.md 부재 — 시계열 변경 추적 불가")
        ]
    try:
        text = log.read_text(encoding="utf-8")
    except OSError:
        return []
    latest: datetime | None = None
    for m in re.finditer(r"^##\s*\[(\d{4}-\d{2}-\d{2})\]", text, re.MULTILINE):
        d = datetime.strptime(m.group(1), "%Y-%m-%d").replace(tzinfo=timezone.utc)
        if latest is None or d > latest:
            latest = d
    if latest is None:
        return [
            Finding("L09", "info", "log.md", "log.md 에 타임스탬프 항목 없음")
        ]
    age = (today - latest).days
    if age >= LOG_STALE_DAYS:
        return [
            Finding(
                "L09",
                "info",
                "log.md",
                f"vault 유휴 — {age}일 경과 (>= {LOG_STALE_DAYS}일)",
                extra={"days_idle": age},
            )
        ]
    return []


def rule_l10(p: Page, vault: Path) -> list[Finding]:
    if not p.has_frontmatter:
        return []
    ptype = p.frontmatter.get("type")
    if ptype not in ("source", "comparison"):
        return []
    sources = p.frontmatter.get("sources") or []
    if not isinstance(sources, list):
        sources = []
    raw_paths = [
        s for s in sources
        if isinstance(s, str) and not s.startswith(("http://", "https://"))
    ]
    if raw_paths:
        return []
    return [
        Finding(
            "L10",
            "error",
            p.rel_path,
            f"type={ptype} 인데 raw/ source 가 0개 (1차 출처 부재)",
        )
    ]
def load_project_config(vault: Path, project: str) -> dict[str, Any]:
    """`wiki/projects/<project>/.wiki-lint.toml` 자동 로딩.

    형식:
    ```toml
    [rules.L07]
    skip_paths = ["wiki/projects/devhub/sources/ADR-*.md"]
    ```
    """
    config_path = vault / "wiki" / ("cross" if project == "cross" else f"projects/{project}") / ".wiki-lint.toml"
    if not config_path.is_file():
        return {}
    try:
        with open(config_path, "rb") as f:
            return tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        return {"_error": f"config load failed: {exc}"}


def is_skipped(rule_id: str, page_path: str, config: dict[str, Any]) -> bool:
    """config 의 [rules.<ID>].skip_paths glob 패턴 매칭."""
    if not config:
        return False
    rules_cfg = config.get("rules", {})
    rule_cfg = rules_cfg.get(rule_id, {})
    skip = rule_cfg.get("skip_paths", [])
    if not skip:
        return False
    import fnmatch
    return any(fnmatch.fnmatch(page_path, pat) for pat in skip)


def check_permission(vault: Path) -> Finding | None:
    """vault 가 쓰기 가능한 디렉터리이고, wiki 가 있는지 확인."""
    if not vault.is_dir():
        return Finding("PERM", "error", str(vault), f"vault 경로 부재 또는 디렉터리 아님: {vault}")
    if not (vault / "wiki").is_dir():
        return Finding("PERM", "error", str(vault / "wiki"), f"vault 안에 wiki/ 디렉터리 부재")
    return None


# === 메인 ===
def run_lint(
    vault: Path,
    rule_filter: set[str] | None,
    project: str | None = None,
    project_config: dict[str, Any] | None = None,
) -> dict[str, Any]:
    today = datetime.now(timezone.utc)
    perm_err = check_permission(vault)
    if perm_err is not None:
        return {
            "status": "error",
            "tool_version": TOOL_VERSION,
            "error": perm_err.message,
            "error_code": "PERMISSION_DENIED" if perm_err.rule == "PERM" else "VAULT_INVALID",
            "warnings": [],
            "source_context": {"vault_path": str(vault)},
        }
    projects_filter = [project] if project else None
    pages = load_pages(vault, projects=projects_filter)
    # project 별 config 자동 발견
    project_configs: dict[str, dict[str, Any]] = {}
    for p in pages:
        if p.project and p.project not in project_configs:
            cfg = project_config if project_config is not None else load_project_config(vault, p.project)
            project_configs[p.project] = cfg
    idx = index_pages(pages)
    findings: list[Finding] = []
    active = rule_filter or {
        "L01", "L02", "L03", "L04", "L05", "L06", "L07", "L08", "L09", "L10"
    }

    # inbound 계산 (L03)
    for p in pages:
        fm_related = p.frontmatter.get("related") or []
        if isinstance(fm_related, str):
            fm_related = [fm_related]
        for raw in fm_related:
            t = _strip_link_brackets(raw)
            if t in idx and t != p.abs_path.stem:
                target_rel = next(
                    (pp.rel_path for pp in pages if pp.abs_path.stem == t),
                    None,
                )
                if target_rel:
                    for pp in pages:
                        if pp.rel_path == target_rel:
                            pp.inbound_from.add(p.rel_path)
        for raw in extract_wiki_links(p.body):
            t = raw.split("|")[0].split("#")[0].strip()
            if t in idx and t != p.abs_path.stem:
                for pp in pages:
                    if pp.abs_path.stem == t:
                        pp.inbound_from.add(p.rel_path)

    def _is_skipped(rule_id: str, page: Page) -> bool:
        if project_config is not None:
            return is_skipped(rule_id, page.rel_path, project_config)
        cfg = project_configs.get(page.project, {})
        return is_skipped(rule_id, page.rel_path, cfg)

    if "L01" in active:
        for p in pages:
            findings.extend(rule_l01(p))
    if "L02" in active:
        for p in pages:
            findings.extend(rule_l02(p, idx))
    if "L03" in active:
        for p in pages:
            if _is_skipped("L03", p):
                continue
            findings.extend(rule_l03(p, idx))
    if "L04" in active:
        l04_skip_paths: set[str] = set()
        for proj_cfg in [cfg, *project_configs.values()]:
            l04_skip_paths.update(proj_cfg.get("rules", {}).get("L04", {}).get("skip_paths", []))
        findings.extend(rule_l04(pages, l04_skip_paths))
    if "L05" in active:
        for p in pages:
            findings.extend(rule_l05(p, today))
    if "L06" in active:
        for p in pages:
            if _is_skipped("L06", p):
                continue
            findings.extend(rule_l06(p, vault))
    if "L07" in active:
        for p in pages:
            if _is_skipped("L07", p):
                continue
            findings.extend(rule_l07_one(p, pages))
    if "L08" in active:
        l08_skip_paths: set[str] = set()
        for proj_cfg in [cfg, *project_configs.values()]:
            l08_skip_paths.update(proj_cfg.get("rules", {}).get("L08", {}).get("skip_paths", []))
        findings.extend(rule_l08(pages, vault / "index.md", l08_skip_paths))
    if "L09" in active:
        findings.extend(rule_l09(vault, today))
    if "L10" in active:
        for p in pages:
            findings.extend(rule_l10(p, vault))

    sev = {"error": 0, "warn": 0, "info": 0}
    for f in findings:
        sev[f.severity] = sev.get(f.severity, 0) + 1

    return {
        "status": "ok",
        "tool_version": TOOL_VERSION,
        "vault_path": str(vault),
        "examined_at": today.isoformat(timespec="seconds"),
        "summary": {
            "errors": sev["error"],
            "warns": sev["warn"],
            "infos": sev["info"],
            "pages_scanned": len(pages),
            "rules_executed": sorted(active),
        },
        "findings": [
            {
                "rule": f.rule,
                "severity": f.severity,
                "path": f.path,
                "message": f.message,
                **({"extra": f.extra} if f.extra else {}),
            }
            for f in findings
        ],
    }


def render_markdown(result: dict[str, Any]) -> str:
    if result.get("status") != "ok":
        return f"# Lint Report — {datetime.now().strftime('%Y-%m-%d')}\n\nERROR: {result.get('error')} ({result.get('error_code')})\n"
    s = result["summary"]
    lines = [
        f"# Lint Report — {datetime.now().strftime('%Y-%m-%d')}",
        "",
        f"- vault: `{result['vault_path']}`",
        f"- 검사 시각: {result['examined_at']}",
        f"- 검사자: wiki-lint {result['tool_version']}",
        f"- 결과: **{s['errors']} error** / **{s['warns']} warn** / **{s['infos']} info** (pages={s['pages_scanned']})",
        f"- 실행 규칙: {', '.join(s['rules_executed'])}",
        "",
    ]
    findings = result["findings"]
    by_sev: dict[str, list[dict[str, Any]]] = {"error": [], "warn": [], "info": []}
    for f in findings:
        by_sev[f["severity"]].append(f)
    sev_title = {"error": "Error", "warn": "Warn", "info": "Info"}
    for sev, title in sev_title.items():
        if not by_sev[sev]:
            continue
        lines.append(f"## {title}")
        lines.append("")
        for f in by_sev[sev]:
            extra = f.get("extra")
            extra_str = f" — {json.dumps(extra, ensure_ascii=False)}" if extra else ""
            lines.append(f"- [{f['rule']}] `{f['path']}` — {f['message']}{extra_str}")
        lines.append("")
    if not findings:
        lines.append("_위반 없음. vault 깨끗._")
        lines.append("")
    return "\n".join(lines)


def write_report(vault: Path, md: str, project: str | None = None) -> Path:
    """리포트 저장. project 지정 시 `_lint/<project>/report_YYYY-MM-DD.md`."""
    if project:
        report_dir = vault / "_lint" / project
    else:
        report_dir = vault / "_lint"
    report_dir.mkdir(parents=True, exist_ok=True)
    fname = f"report_{datetime.now().strftime('%Y-%m-%d')}.md"
    out = report_dir / fname
    out.write_text(md, encoding="utf-8")
    return out


def _strip_link_brackets(raw: Any) -> str:
    s = str(raw).strip()
    if s.startswith("[[") and s.endswith("]]"):
        s = s[2:-2]
    return s.split("|")[0].split("#")[0].strip()


# === CLI ===
def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        prog="wiki-lint",
        description="LLM Wiki vault 무결성 검사 (L01~L10) — D-72 per-project 지원",
    )
    ap.add_argument("--vault-path", required=True, help="vault 루트 (예: ~/wiki)")
    ap.add_argument(
        "--project",
        default=None,
        help="특정 project 만 검사 (예: my-harness, devhub). 기본: 전체 project 자동 발견",
    )
    ap.add_argument(
        "--project-config",
        default=None,
        help="per-project rule override (TOML). 기본: wiki/projects/<project>/.wiki-lint.toml 자동",
    )
    ap.add_argument(
        "--rules",
        default="L01,L02,L03,L04,L05,L06,L07,L08,L09,L10",
        help="검사할 규칙 ID 콤마 구분 (기본: 전체)",
    )
    ap.add_argument("--output", choices=("json", "markdown", "both"), default="both")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args(argv)

    vault = Path(args.vault_path).expanduser().resolve()
    rule_filter = {r.strip() for r in args.rules.split(",") if r.strip()} or None
    project_config: dict[str, Any] | None = None
    if args.project_config:
        try:
            with open(args.project_config, "rb") as f:
                project_config = tomllib.load(f)
        except (OSError, tomllib.TOMLDecodeError) as exc:
            print(json.dumps({
                "status": "error",
                "tool_version": TOOL_VERSION,
                "error": f"config load failed: {exc}",
                "error_code": "CONFIG_INVALID",
            }, ensure_ascii=False, indent=2))
            return 2
    result = run_lint(
        vault, rule_filter,
        project=args.project,
        project_config=project_config,
    )

    if args.output in ("json", "both"):
        print(json.dumps(result, ensure_ascii=False, indent=2))
    if args.output in ("markdown", "both"):
        md = render_markdown(result)
        out = write_report(vault, md, project=args.project)
        if not args.quiet:
            sys.stderr.write(f"lint report: {out}\n")

    if result.get("status") != "ok":
        return 2
    return 1 if result["summary"]["errors"] > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
