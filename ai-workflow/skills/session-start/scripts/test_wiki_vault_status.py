#!/usr/bin/env python3
"""wiki_vault_status 단위 테스트 (D-71).

stdlib only. workflow_kit 비의존. fixture 는 /tmp 에 격리.
"""

from __future__ import annotations

import importlib.util
import shutil
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

# wiki_vault_status 모듈을 importlib 로 로드 (workflow_kit 비의존 확인)
SCRIPT = Path(__file__).resolve().parent / "wiki_vault_status.py"
spec = importlib.util.spec_from_file_location("wiki_vault_status", SCRIPT)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
discover_wiki_vault = mod.discover_wiki_vault
resolve_wiki_vault_path = mod.resolve_wiki_vault_path
WikiVaultStatus = mod.WikiVaultStatus
WikiLintSummary = mod.WikiLintSummary

FIXTURE = Path(tempfile.gettempdir()) / "wiki_vault_status_fixture"


def make_fixture() -> Path:
    if FIXTURE.exists():
        shutil.rmtree(FIXTURE)
    for sub in mod.WIKI_DIRS:
        (FIXTURE / "wiki" / sub).mkdir(parents=True)
    (FIXTURE / "raw" / "ai-workflow").mkdir(parents=True)
    (FIXTURE / "schema").mkdir(parents=True)
    return FIXTURE


def write(p: Path, content: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


class WikiVaultStatusTest(unittest.TestCase):
    def setUp(self) -> None:
        make_fixture()

    # === resolve_wiki_vault_path ===
    def test_resolve_arg_wins(self) -> None:
        result = resolve_wiki_vault_path("/tmp/forced", env_value="/tmp/env")
        self.assertEqual(result, Path("/tmp/forced"))

    def test_resolve_env_when_no_arg(self) -> None:
        result = resolve_wiki_vault_path(None, env_value=str(FIXTURE))
        self.assertEqual(result, FIXTURE)

    def test_resolve_default_only_if_exists(self) -> None:
        # FIXTURE 가 만들어졌으므로 default 발견
        result = resolve_wiki_vault_path(None, home=FIXTURE.parent)
        # FIXTURE 가 wiki 의 자식이 아니라 /wiki 가 아니라서 default 로 안 잡힘
        # → default /wiki 부재 시 None 반환 확인
        # (FIXTURE 자체가 아니라 FIXTURE/wiki 가 있어야 default 가 됨)
        # 일단 None 반환이 정상 (default 디렉터리 부재)
        self.assertIsNone(result)

    def test_resolve_default_when_wiki_exists(self) -> None:
        # FIXTURE 자체가 ~/wiki 처럼 동작하도록 임시 root 에 wiki 이름으로 셋업
        root = FIXTURE.parent / "fake_home"
        wiki = root / "wiki"
        wiki.mkdir(parents=True, exist_ok=True)
        try:
            result = resolve_wiki_vault_path(None, home=root)
            self.assertEqual(result, wiki)
        finally:
            shutil.rmtree(root, ignore_errors=True)

    # === discover_wiki_vault (lint 없이) ===
    def test_discover_existing_vault_basic(self) -> None:
        # log.md / index.md / raw/_manifest.md / wiki pages 채우기
        write(FIXTURE / "log.md", f"# Log\n\n## [{datetime.now().strftime('%Y-%m-%d')}] init | fixture\n")
        write(FIXTURE / "index.md", "# Index\n")
        write(FIXTURE / "raw" / "_manifest.md", "## [2026-06-10] test | path\n")
        write(FIXTURE / "wiki" / "concepts" / "x.md", "---\ntitle: X\n---\n")
        write(FIXTURE / "wiki" / "entities" / "y.md", "---\ntitle: Y\n---\n")
        s = discover_wiki_vault(FIXTURE, run_lint=False)
        self.assertTrue(s.exists)
        self.assertTrue(s.log_md_exists)
        self.assertIsNotNone(s.last_log_entry)
        self.assertIn("init", s.last_log_entry)
        self.assertEqual(s.wiki_page_count, 2)
        self.assertEqual(s.raw_entry_count, 1)
        self.assertIsNone(s.lint)

    def test_discover_vault_without_log(self) -> None:
        s = discover_wiki_vault(FIXTURE, run_lint=False)
        self.assertTrue(s.exists)
        self.assertFalse(s.log_md_exists)
        self.assertIsNone(s.last_log_entry)
        self.assertEqual(s.wiki_page_count, 0)
        self.assertEqual(s.raw_entry_count, 0)

    def test_discover_vault_last_log_picks_recent(self) -> None:
        old = (datetime.now() - timedelta(days=5)).strftime("%Y-%m-%d")
        new = datetime.now().strftime("%Y-%m-%d")
        write(
            FIXTURE / "log.md",
            f"# Log\n\n## [{old}] old | first\n\n## [{new}] new | second\n",
        )
        s = discover_wiki_vault(FIXTURE, run_lint=False)
        self.assertIsNotNone(s.last_log_entry)
        self.assertIn("new", s.last_log_entry)

    def test_discover_nonexistent_vault(self) -> None:
        s = discover_wiki_vault(Path("/tmp/this_does_not_exist_xyz123"), run_lint=False)
        self.assertFalse(s.exists)
        self.assertFalse(s.log_md_exists)
        self.assertIsNone(s.last_log_entry)
        self.assertEqual(s.wiki_page_count, 0)
        self.assertIsNone(s.lint)

    def test_discover_with_lint_clean(self) -> None:
        # 깨끗한 vault: index.md 시드해서 L08 (index missing) 회피,
        # wiki pages 0 → 0/0/0 기대
        write(FIXTURE / "log.md", f"# Log\n\n## [{datetime.now().strftime('%Y-%m-%d')}] init | fixture\n")
        write(FIXTURE / "index.md", "# Index\n")
        write(FIXTURE / "raw" / "_manifest.md", "")
        s = discover_wiki_vault(FIXTURE, run_lint=True, timeout=10.0)
        if "wiki-lint 스킬 미설치" in " ".join(s.warnings):
            self.skipTest("wiki-lint skill not co-located in expected path")
        self.assertIsNotNone(s.lint)
        self.assertEqual(s.lint.errors, 0)
        self.assertEqual(s.lint.warns, 0)
        self.assertEqual(s.lint.infos, 0)
        self.assertIn("L01", s.lint.extra.get("rules_executed", []))

    def test_discover_with_lint_dirty(self) -> None:
        # 위반 1건 만들기 (frontmatter 누락 → L01)
        write(FIXTURE / "wiki" / "concepts" / "bad.md", "# No frontmatter\nbody\n")
        s = discover_wiki_vault(FIXTURE, run_lint=True, timeout=10.0)
        if "wiki-lint 스킬 미설치" in " ".join(s.warnings):
            self.skipTest("wiki-lint skill not co-located in expected path")
        self.assertIsNotNone(s.lint)
        self.assertGreater(s.lint.errors, 0)
        self.assertEqual(s.lint.pages_scanned, 1)
        # 리포트 파일도 _lint/ 에 있어야 함
        self.assertIsNotNone(s.lint.last_report)
        self.assertTrue(Path(s.lint.last_report).is_file())

    def test_discover_lint_script_missing_falls_back(self) -> None:
        # 존재하지 않는 lint_script 경로 주입
        s = discover_wiki_vault(
            FIXTURE, run_lint=True, lint_script=Path("/tmp/no_such_lint_xyz.py")
        )
        self.assertTrue(s.exists)
        self.assertIsNone(s.lint)
        self.assertTrue(any("wiki-lint" in w for w in s.warnings))

    def test_discover_lint_skip(self) -> None:
        s = discover_wiki_vault(FIXTURE, run_lint=False)
        self.assertIsNone(s.lint)
        # lint skip 했으니 warnings 도 비어있음
        self.assertEqual(s.warnings, [])

    # === to_dict 형식 ===
    def test_to_dict_shape(self) -> None:
        s = WikiVaultStatus(path="/tmp/x")
        s.exists = True
        s.wiki_page_count = 3
        d = s.to_dict()
        self.assertIn("path", d)
        self.assertIn("exists", d)
        self.assertIn("lint", d)
        self.assertIn("warnings", d)
        self.assertIn("notes", d)


if __name__ == "__main__":
    unittest.main(verbosity=2)
