"""LLM Wiki vault 상태 점검 (D-71) — session-start 의 wiki_vault 필드.

stdlib only. workflow_kit 비의존. 별도 테스트 가능.

- vault 경로 입력 → WikiVaultStatus 출력
- log.md / index.md / raw/_manifest.md / wiki/ 페이지 카운트
- 선택: wiki-lint 스킬 subprocess 호출 (--output both 로 리포트까지 확보)
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

WIKI_LINT_SCRIPT = (
    Path(__file__).resolve().parent.parent.parent
    / "wiki-lint"
    / "scripts"
    / "run_wiki_lint.py"
)
WIKI_DIRS = ("concepts", "entities", "topics", "sources", "comparisons", "query", "meta")
LOG_ENTRY_RE = re.compile(r"^##\s*\[(\d{4}-\d{2}-\d{2})\][^\n]*$", re.MULTILINE)


class WikiLintSummary:
    """Subset of wiki-lint result. 의존성 없는 plain class — schema 와 호환되도록 attribute 만 노출."""

    def __init__(
        self,
        errors: int = 0,
        warns: int = 0,
        infos: int = 0,
        pages_scanned: int = 0,
        last_report: str | None = None,
        extra: dict[str, Any] | None = None,
    ) -> None:
        self.errors = errors
        self.warns = warns
        self.infos = infos
        self.pages_scanned = pages_scanned
        self.last_report = last_report
        self.extra = extra or {}

    def to_dict(self) -> dict[str, Any]:
        return {
            "errors": self.errors,
            "warns": self.warns,
            "infos": self.infos,
            "pages_scanned": self.pages_scanned,
            "last_report": self.last_report,
            "extra": self.extra,
        }


class WikiVaultStatus:
    """Vault 상태. dict-like — model_dump 호환."""

    def __init__(self, path: str) -> None:
        self.path = path
        self.exists = False
        self.log_md_exists = False
        self.last_log_entry: str | None = None
        self.wiki_page_count = 0
        self.raw_entry_count = 0
        self.lint: WikiLintSummary | None = None
        self.warnings: list[str] = []
        self.notes: list[str] = []

    def to_dict(self) -> dict[str, Any]:
        return {
            "path": self.path,
            "exists": self.exists,
            "log_md_exists": self.log_md_exists,
            "last_log_entry": self.last_log_entry,
            "wiki_page_count": self.wiki_page_count,
            "raw_entry_count": self.raw_entry_count,
            "lint": self.lint.to_dict() if self.lint else None,
            "warnings": list(self.warnings),
            "notes": list(self.notes),
        }


def discover_wiki_vault(
    vault_path: Path,
    *,
    run_lint: bool = True,
    timeout: float = 15.0,
    lint_script: Path | None = None,
) -> WikiVaultStatus:
    """Build a WikiVaultStatus for the given vault path.

    - Path 부재 시 exists=False 로 끝남
    - run_lint=True 면 wiki-lint 를 subprocess 로 호출 (vault 가 있을 때만)
    - subprocess 실패는 lint=None 으로 처리, warnings 에 사유 기록
    - lint_script 인자는 테스트용 (기본은 WIKI_LINT_SCRIPT)
    """
    status = WikiVaultStatus(path=str(vault_path))
    status.exists = vault_path.is_dir()
    if not status.exists:
        return status

    log_md = vault_path / "log.md"
    if log_md.is_file():
        status.log_md_exists = True
        try:
            text = log_md.read_text(encoding="utf-8")
            matches = LOG_ENTRY_RE.findall(text)
            if matches:
                last_date = matches[-1]
                pat = re.compile(rf"^##\s*\[{re.escape(last_date)}\][^\n]*$", re.MULTILINE)
                line_match = pat.search(text)
                if line_match:
                    status.last_log_entry = line_match.group(0).strip()
        except OSError as exc:
            status.warnings.append(f"log.md 읽기 실패: {exc}")

    wiki_root = vault_path / "wiki"
    if wiki_root.is_dir():
        for sub in WIKI_DIRS:
            d = wiki_root / sub
            if d.is_dir():
                status.wiki_page_count += sum(1 for p in d.glob("*.md") if p.is_file())

    manifest = vault_path / "raw" / "_manifest.md"
    if manifest.is_file():
        try:
            text = manifest.read_text(encoding="utf-8")
            status.raw_entry_count = len(LOG_ENTRY_RE.findall(text))
        except OSError:
            pass

    if not run_lint:
        return status

    lint_path = lint_script if lint_script is not None else WIKI_LINT_SCRIPT
    if not lint_path.is_file():
        status.warnings.append("wiki-lint 스킬 미설치 (wiki-lint/SKILL.md 없음)")
        return status

    # --output both 으로 호출해 vault/_lint/report_YYYY-MM-DD.md 까지 materialize
    try:
        proc = subprocess.run(
            [
                sys.executable,
                str(lint_path),
                "--vault-path",
                str(vault_path),
                "--output",
                "both",
                "--quiet",
            ],
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        status.warnings.append(f"wiki-lint 타임아웃 ({timeout}s)")
        return status
    except FileNotFoundError:
        status.warnings.append("wiki-lint 스크립트 부재")
        return status

    if proc.returncode == 2 or not proc.stdout.strip():
        status.warnings.append(f"wiki-lint 실행 실패 (exit={proc.returncode})")
        return status

    try:
        data = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        status.warnings.append(f"wiki-lint 출력 파싱 실패: {exc}")
        return status

    s = data.get("summary", {})
    today_str = datetime.now().strftime("%Y-%m-%d")
    report_path = vault_path / "_lint" / f"report_{today_str}.md"
    status.lint = WikiLintSummary(
        errors=s.get("errors", 0),
        warns=s.get("warns", 0),
        infos=s.get("infos", 0),
        pages_scanned=s.get("pages_scanned", 0),
        last_report=str(report_path) if report_path.is_file() else None,
        extra={"rules_executed": s.get("rules_executed", [])},
    )
    return status


def resolve_wiki_vault_path(
    arg: str | None,
    env_value: str | None = None,
    home: Path | None = None,
) -> Path | None:
    """Resolve wiki vault path: arg > env > default ~/wiki. None 이면 미설정.

    default 는 디렉터리가 실제로 존재할 때만 반환 (의도치 않은 자동 발견 방지).
    `env_value`/`home` 인자는 테스트용 오버라이드.
    """
    if arg:
        return Path(arg).expanduser()
    if env_value:
        return Path(env_value).expanduser()
    default = (home or Path.home()) / "wiki"
    return default if default.is_dir() else None
