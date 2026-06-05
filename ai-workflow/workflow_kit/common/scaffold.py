"""Test scaffolding helpers for generating validation scripts."""

from __future__ import annotations

from pathlib import Path
from datetime import date


def generate_validation_scaffold(
    *,
    project_root: Path,
    task_id: str = "validation",
    commands: list[str] | None = None,
    change_summary: str | None = None,
) -> Path:
    """Generate a Python unittest scaffold for reproducing or validating a task."""
    test_dir = project_root / "tests"
    test_dir.mkdir(parents=True, exist_ok=True)

    file_name = f"repro_{task_id.lower().replace('-', '_')}.py"
    target_path = test_dir / file_name

    command_section = ""
    if commands:
        command_section = "\n".join([f"# - {cmd}" for cmd in commands])

    summary_comment = f"# 작업 요약: {change_summary}" if change_summary else "# 작업 요약: (정보 없음)"

    template = f'''#!/usr/bin/env python3
"""Validation script for {task_id}."""

from __future__ import annotations

import unittest
import sys
from pathlib import Path

# 프로젝트 루트를 path에 추가하여 모듈 임포트 지원
REPO_ROOT = Path(__file__).resolve().parents[1]
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

# {summary_comment}
# 권장 검증 명령:
{command_section}
# 생성일: {date.today().isoformat()}

class Test{task_id.replace('-', '').capitalize()}Validation(unittest.TestCase):
    """{task_id}에 대한 검증 테스트 케이스."""

    def setUp(self):
        """테스트 전 준비 작업."""
        pass

    def test_behavior(self):
        """재현 또는 검증하고자 하는 핵심 동작을 여기에 구현한다."""
        # TODO: 실제 검증 로직 구현
        # self.assertTrue(True)
        pass

    def tearDown(self):
        """테스트 후 정리 작업."""
        pass

if __name__ == "__main__":
    unittest.main()
'''
    target_path.write_text(template, encoding="utf-8")
    return target_path
