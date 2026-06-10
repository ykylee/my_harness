#!/usr/bin/env python3
"""wiki-lint 검증 테스트.

unittest stdlib 기반. 각 L01~L10 규칙이 fixture 에서 정확히 fire 하는지 확인.
fixture 위치: /tmp/wiki_fixture_lint/ (없으면 자동 생성)
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

TOOL = Path(__file__).resolve().parent / "run_wiki_lint.py"
FIXTURE = Path(tempfile.gettempdir()) / "wiki_fixture_lint"


# === fixture 빌더 ===
def make_fresh_fixture() -> Path:
    """fixture 디렉터리를 매 테스트 시작 시 깨끗하게 재생성."""
    if FIXTURE.exists():
        shutil.rmtree(FIXTURE)
    (FIXTURE / "wiki" / "concepts").mkdir(parents=True)
    (FIXTURE / "wiki" / "entities").mkdir(parents=True)
    (FIXTURE / "wiki" / "topics").mkdir(parents=True)
    (FIXTURE / "wiki" / "sources").mkdir(parents=True)
    (FIXTURE / "wiki" / "comparisons").mkdir(parents=True)
    (FIXTURE / "wiki" / "query").mkdir(parents=True)
    (FIXTURE / "wiki" / "meta").mkdir(parents=True)
    (FIXTURE / "raw" / "ai-workflow").mkdir(parents=True)
    (FIXTURE / "raw" / "clippings").mkdir(parents=True)
    (FIXTURE / "schema").mkdir(parents=True)
    (FIXTURE / "_lint").mkdir(parents=True)
    (FIXTURE / "raw" / "ai-workflow" / "state.json").write_text('{"schema_version": "1"}')
    (FIXTURE / "raw" / "clippings" / "2026-04-01_karpathy.md").write_text("# karpathy clip\n")
    (FIXTURE / "log.md").write_text(
        f"# Log\n\n## [{datetime.now().strftime('%Y-%m-%d')}] init | fixture\n"
    )
    (FIXTURE / "index.md").write_text("# Index\n")
    return FIXTURE


def write_page(rel: str, content: str) -> None:
    p = FIXTURE / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


def frontmatter(**kw) -> str:
    """간단한 frontmatter 빌더. None 은 항목 누락."""
    lines = ["---"]
    for k, v in kw.items():
        if v is None:
            continue
        if isinstance(v, list):
            if not v:
                lines.append(f"{k}: []")
            else:
                lines.append(f"{k}:")
                for item in v:
                    lines.append(f"  - {item}")
        else:
            lines.append(f"{k}: {v}")
    lines.append("---")
    lines.append("")
    return "\n".join(lines)


def run_lint(rules: str = "L01,L02,L03,L04,L05,L06,L07,L08,L09,L10") -> dict:
    proc = subprocess.run(
        [
            sys.executable,
            str(TOOL),
            "--vault-path",
            str(FIXTURE),
            "--rules",
            rules,
            "--output",
            "json",
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode == 2:
        # 권한/vault 에러
        return {"status": "error", "_stderr": proc.stderr, "_stdout": proc.stdout}
    import json
    return json.loads(proc.stdout)


def findings_by_rule(result: dict) -> dict[str, list[dict]]:
    out: dict[str, list[dict]] = {}
    for f in result.get("findings", []):
        out.setdefault(f["rule"], []).append(f)
    return out


# === 테스트 케이스 ===
class WikiLintTest(unittest.TestCase):
    def setUp(self) -> None:
        make_fresh_fixture()

    # L01
    def test_l01_no_frontmatter(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/no-frontmatter.md",
            "# No frontmatter\nbody\n",
        )
        r = run_lint("L01")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L01", [])), 1)
        self.assertIn("no-frontmatter.md", rules["L01"][0]["path"])

    def test_l01_missing_fields(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/partial.md",
            frontmatter(title="Partial", type="concept", last_touched="2026-06-10") + "body\n",
        )
        r = run_lint("L01")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L01", [])), 1)
        self.assertIn("tags", rules["L01"][0]["extra"]["missing_fields"])

    def test_l01_clean(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/clean.md",
            frontmatter(
                title="Clean",
                type="concept",
                tags=["x"],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L01")
        self.assertEqual(findings_by_rule(r).get("L01", []), [])

    # L02
    def test_l02_broken_related_and_body(self) -> None:
        write_page(
            "wiki/projects/my-harness/entities/broken.md",
            frontmatter(
                title="Broken",
                type="entity",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=["[[ghost-page]]"],
                status="draft",
            ) + "Body with [[also-ghost]].\n",
        )
        r = run_lint("L02")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L02", [])), 2)

    def test_l02_resolved_link(self) -> None:
        # 대상 페이지도 함께 생성 → broken 이면 안 됨
        write_page(
            "wiki/projects/my-harness/entities/target.md",
            frontmatter(
                title="Target",
                type="entity",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        write_page(
            "wiki/projects/my-harness/entities/source.md",
            frontmatter(
                title="Source",
                type="entity",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=["[[target]]"],
                status="draft",
            ) + "Body with [[target]].\n",
        )
        r = run_lint("L02")
        self.assertEqual(findings_by_rule(r).get("L02", []), [])

    # L03
    def test_l03_orphan(self) -> None:
        write_page(
            "wiki/projects/my-harness/topics/orphan.md",
            frontmatter(
                title="Orphan",
                type="topic",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L03")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L03", [])), 1)

    def test_l03_not_orphan_with_inbound(self) -> None:
        # 두 페이지: hub, leaf
        write_page(
            "wiki/projects/my-harness/concepts/hub.md",
            frontmatter(
                title="Hub",
                type="concept",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        write_page(
            "wiki/projects/my-harness/concepts/leaf.md",
            frontmatter(
                title="Leaf",
                type="concept",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        # hub 에서 leaf 로 inbound link
        (FIXTURE / "wiki" / "projects" / "my-harness" / "concepts" / "hub.md").write_text(
            frontmatter(
                title="Hub",
                type="concept",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            )
            + "body refers [[leaf]]\n",
        )
        r = run_lint("L03")
        rules = findings_by_rule(r)
        paths = {f["path"] for f in rules.get("L03", [])}
        self.assertNotIn("wiki/projects/my-harness/concepts/leaf.md", paths)
        self.assertIn("wiki/projects/my-harness/concepts/hub.md", paths)  # hub 도 inbound 없음

    # L04
    def test_l04_duplicate_title(self) -> None:
        for name in ("a", "b"):
            write_page(
                f"wiki/projects/my-harness/entities/dup-{name}.md",
                frontmatter(
                    title="Duplicate",
                    type="entity",
                    tags=[],
                    sources=["raw/ai-workflow/state.json"],
                    last_touched="2026-06-10",
                    related=[],
                    status="draft",
                ) + "body\n",
            )
        r = run_lint("L04")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L04", [])), 1)
        self.assertEqual(len(rules["L04"][0]["extra"]["all_paths"]), 2)

    def test_l04_unique(self) -> None:
        write_page(
            "wiki/projects/my-harness/entities/single.md",
            frontmatter(
                title="Single",
                type="entity",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L04")
        self.assertEqual(findings_by_rule(r).get("L04", []), [])

    # L05
    def test_l05_stale(self) -> None:
        old = (datetime.now() - timedelta(days=91)).strftime("%Y-%m-%d")
        write_page(
            "wiki/projects/my-harness/concepts/old.md",
            frontmatter(
                title="Old",
                type="concept",
                tags=[],
                sources=[],
                last_touched=old,
                related=[],
                status="reviewed",
            ) + "body\n",
        )
        r = run_lint("L05")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L05", [])), 1)
        self.assertGreaterEqual(rules["L05"][0]["extra"]["days_since_touched"], 90)

    def test_l05_fresh(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/fresh.md",
            frontmatter(
                title="Fresh",
                type="concept",
                tags=[],
                sources=[],
                last_touched=datetime.now().strftime("%Y-%m-%d"),
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L05")
        self.assertEqual(findings_by_rule(r).get("L05", []), [])

    # L06
    def test_l06_missing_source_path(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/no-file.md",
            frontmatter(
                title="NoFile",
                type="concept",
                tags=[],
                sources=["raw/does-not-exist.md"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L06")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L06", [])), 1)
        self.assertEqual(rules["L06"][0]["extra"]["missing_source"], "raw/does-not-exist.md")

    def test_l06_existing_source(self) -> None:
        write_page(
            "wiki/projects/my-harness/concepts/ok.md",
            frontmatter(
                title="OK",
                type="concept",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L06")
        self.assertEqual(findings_by_rule(r).get("L06", []), [])

    def test_l06_url_source_skipped(self) -> None:
        # URL 은 filesystem 검사를 skip — L06 발화 X
        write_page(
            "wiki/projects/my-harness/concepts/url.md",
            frontmatter(
                title="URLSrc",
                type="concept",
                tags=[],
                sources=["https://example.com/article.md"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L06")
        self.assertEqual(findings_by_rule(r).get("L06", []), [])

    # L07
    def test_l07_reviewed_duplicates(self) -> None:
        for name in ("a", "b"):
            write_page(
                f"wiki/projects/my-harness/entities/dup-{name}.md",
                frontmatter(
                    title="Conflict",
                    type="entity",
                    tags=[],
                    sources=["raw/ai-workflow/state.json"],
                    last_touched="2026-06-10",
                    related=[],
                    status="reviewed",
                ) + "body\n",
            )
        r = run_lint("L07")
        rules = findings_by_rule(r)
        # rule_l07_one fires once per reviewed peer (per-page variant, 2 pages -> 2 findings)
        self.assertEqual(len(rules.get("L07", [])), 2)

    def test_l07_draft_duplicates_no_fire(self) -> None:
        for name in ("a", "b"):
            write_page(
                f"wiki/projects/my-harness/entities/dup-{name}.md",
                frontmatter(
                    title="Drafts",
                    type="entity",
                    tags=[],
                    sources=["raw/ai-workflow/state.json"],
                    last_touched="2026-06-10",
                    related=[],
                    status="draft",
                ) + "body\n",
            )
        r = run_lint("L07")
        # draft 만 있으면 L04 가 warn, L07 은 발화 X
        self.assertEqual(findings_by_rule(r).get("L07", []), [])

    # L08
    def test_l08_unindexed(self) -> None:
        write_page(
            "wiki/projects/my-harness/meta/unindexed.md",
            frontmatter(
                title="Unindexed",
                type="meta",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L08")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L08", [])), 1)

    def test_l08_indexed(self) -> None:
        write_page(
            "wiki/projects/my-harness/meta/indexed.md",
            frontmatter(
                title="Indexed",
                type="meta",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        (FIXTURE / "index.md").write_text(
            "# Index\n\n- Indexed — `wiki/projects/my-harness/meta/indexed.md`\n"
        )
        r = run_lint("L08")
        self.assertEqual(findings_by_rule(r).get("L08", []), [])

    def test_l08_index_missing(self) -> None:
        (FIXTURE / "index.md").unlink()
        r = run_lint("L08")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L08", [])), 1)
        self.assertIn("index.md 부재", rules["L08"][0]["message"])

    # L09
    def test_l09_fresh_log(self) -> None:
        # log.md 가 오늘 날짜 — 유휴 아님
        r = run_lint("L09")
        self.assertEqual(findings_by_rule(r).get("L09", []), [])

    def test_l09_stale_log(self) -> None:
        old = (datetime.now() - timedelta(days=10)).strftime("%Y-%m-%d")
        (FIXTURE / "log.md").write_text(f"# Log\n\n## [{old}] old | fixture\n")
        r = run_lint("L09")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L09", [])), 1)
        self.assertGreaterEqual(rules["L09"][0]["extra"]["days_idle"], 7)

    def test_l09_no_log_file(self) -> None:
        (FIXTURE / "log.md").unlink()
        r = run_lint("L09")
        self.assertEqual(len(findings_by_rule(r).get("L09", [])), 1)

    # L10
    def test_l10_source_no_raw(self) -> None:
        write_page(
            "wiki/projects/my-harness/sources/bare.md",
            frontmatter(
                title="Bare",
                type="source",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L10")
        rules = findings_by_rule(r)
        self.assertEqual(len(rules.get("L10", [])), 1)

    def test_l10_source_with_raw_ok(self) -> None:
        write_page(
            "wiki/projects/my-harness/sources/good.md",
            frontmatter(
                title="Good",
                type="source",
                tags=[],
                sources=["raw/ai-workflow/state.json"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L10")
        self.assertEqual(findings_by_rule(r).get("L10", []), [])

    def test_l10_concept_no_raw_ok(self) -> None:
        # type=concept 는 L10 면제
        write_page(
            "wiki/projects/my-harness/concepts/c.md",
            frontmatter(
                title="C",
                type="concept",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint("L10")
        self.assertEqual(findings_by_rule(r).get("L10", []), [])

    # 종합
    def test_full_lint_returns_known_shape(self) -> None:
        # 페이지 1개만 — 깨끗한 케이스
        write_page(
            "wiki/projects/my-harness/concepts/only.md",
            frontmatter(
                title="Only",
                type="concept",
                tags=["x"],
                sources=["raw/ai-workflow/state.json"],
                last_touched=datetime.now().strftime("%Y-%m-%d"),
                related=[],
                status="draft",
            ) + "body\n",
        )
        (FIXTURE / "index.md").write_text("# Index\n\n- Only — `wiki/projects/my-harness/concepts/only.md`\n")
        r = run_lint()
        self.assertEqual(r["status"], "ok")
        self.assertEqual(r["summary"]["pages_scanned"], 1)
        # 깨끗하므로 error 0
        self.assertEqual(r["summary"]["errors"], 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
