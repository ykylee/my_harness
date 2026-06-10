#!/usr/bin/env python3
"""D-72 per-project tests for wiki-lint (subset).

Separate file to keep the original test file focused on D-71 rules.
"""

from __future__ import annotations

import sys
import unittest
from datetime import datetime
from pathlib import Path

TOOL = Path(__file__).resolve().parent / "run_wiki_lint.py"
FIXTURE = Path("/tmp") / "wiki_fixture_lint_d72"


def make_fixture() -> Path:
    if FIXTURE.exists():
        import shutil
        shutil.rmtree(FIXTURE)
    for sub in ("concepts", "entities", "topics", "sources", "comparisons", "meta"):
        (FIXTURE / "wiki" / "projects" / "my-harness" / sub).mkdir(parents=True)
        (FIXTURE / "wiki" / "projects" / "devhub" / sub).mkdir(parents=True)
    (FIXTURE / "raw" / "projects" / "my-harness").mkdir(parents=True)
    (FIXTURE / "raw" / "projects" / "devhub").mkdir(parents=True)
    (FIXTURE / "_lint").mkdir(parents=True)
    (FIXTURE / "raw" / "projects" / "devhub" / "docs" / "adr").mkdir(parents=True)
    (FIXTURE / "raw" / "projects" / "devhub" / "docs" / "adr" / "0019-keycloak-only-idp.md").write_text("# 0019\n")
    return FIXTURE


def write(p: str, content: str) -> None:
    full = FIXTURE / p
    full.parent.mkdir(parents=True, exist_ok=True)
    full.write_text(content, encoding="utf-8")


def frontmatter(**kw) -> str:
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
    import json
    import subprocess
    proc = subprocess.run(
        [
            sys.executable, str(TOOL),
            "--vault-path", str(FIXTURE),
            "--rules", rules,
            "--output", "json",
        ],
        capture_output=True, text=True,
    )
    if proc.returncode == 2 or not proc.stdout.strip():
        return {"status": "error", "_stderr": proc.stderr, "_stdout": proc.stdout}
    return json.loads(proc.stdout)


def findings_by_rule(result: dict) -> dict:
    out = {}
    for f in result.get("findings", []):
        out.setdefault(f["rule"], []).append(f)
    return out


class D72Test(unittest.TestCase):
    def setUp(self) -> None:
        make_fixture()

    def test_per_project_discovery(self) -> None:
        write(
            "wiki/projects/devhub/sources/sample.md",
            frontmatter(
                title="Sample DevHub",
                type="source",
                tags=[],
                sources=["raw/projects/devhub/docs/adr/0019-keycloak-only-idp.md"],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint()
        self.assertEqual(r["summary"]["pages_scanned"], 1)

    def test_per_project_cross_not_in_projects(self) -> None:
        write(
            "wiki/cross/topics/test.md",
            frontmatter(
                title="Cross Test",
                type="topic",
                tags=[],
                sources=[],
                last_touched="2026-06-10",
                related=[],
                status="draft",
            ) + "body\n",
        )
        r = run_lint()
        # cross/ 도 스캔됨
        self.assertEqual(r["summary"]["pages_scanned"], 1)

    def test_per_project_l07_skip(self) -> None:
        for name in ("a", "b"):
            write(
                f"wiki/projects/devhub/sources/adr-test-{name}.md",
                frontmatter(
                    title="ADR-Test",
                    type="source",
                    tags=[],
                    sources=["raw/projects/devhub/docs/adr/0019-keycloak-only-idp.md"],
                    last_touched="2026-06-10",
                    related=[],
                    status="reviewed",
                ) + "body\n",
            )
        toml_path = FIXTURE / "wiki" / "projects" / "devhub" / ".wiki-lint.toml"
        toml_path.write_text(
            '[rules.L07]\n'
            'skip_paths = ["wiki/projects/devhub/sources/adr-test-*.md"]\n',
        )
        try:
            r = run_lint("L07")
            self.assertEqual(findings_by_rule(r).get("L07", []), [])
        finally:
            toml_path.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main(verbosity=2)
