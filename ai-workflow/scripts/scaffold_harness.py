#!/usr/bin/env python3
"""Scaffold a new harness stub inside the standard AI workflow repository."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a new harness stub directory and starter docs."
    )
    parser.add_argument("--harness-name", required=True)
    parser.add_argument(
        "--display-name",
        help="Human-friendly name for the harness. Defaults to a title-cased harness name.",
    )
    parser.add_argument(
        "--root-entrypoint",
        default="TODO: 하네스가 실제로 읽는 루트 진입 파일",
    )
    parser.add_argument(
        "--config-file",
        default="TODO: 하네스 전용 설정 파일 경로",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite generated harness stub files if they already exist.",
    )
    return parser.parse_args()


def slugify(raw: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", raw.strip().lower()).strip("-")
    if not slug:
        raise ValueError("Harness name must contain at least one alphanumeric character.")
    return slug


def titleize(slug: str) -> str:
    return " ".join(part.capitalize() for part in slug.split("-"))


def write_text(path: Path, content: str, *, force: bool) -> None:
    if path.exists() and not force:
        raise FileExistsError(f"Destination already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def render_harness_readme(display_name: str, slug: str, args: argparse.Namespace) -> str:
    return f"""# {display_name} Harness Package

- 문서 목적: 표준 AI 워크플로우를 `{display_name}` 하네스에 맞춰 배포할 때 생성할 파일과 검토 포인트를 정리한다.
- 범위: 루트 진입 파일, 설정 파일, 공통 workflow 문서 연결 방식
- 대상 독자: {display_name} 사용자, 저장소 관리자, AI workflow 설계자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `../../core/workflow_harness_distribution.md`, `../../scripts/bootstrap_workflow_kit.py`, `../_template/README.md`

## 생성 대상

- `{args.root_entrypoint}`
- `{args.config_file}`
- TODO: 하네스가 요구하는 추가 overlay 파일

## 구성 원칙

- `{args.root_entrypoint}` 는 하네스의 실제 진입 파일로 사용한다.
- 상세 정책은 `ai-workflow/memory/` 문서를 먼저 읽도록 연결한다.
- 설정 파일은 가능한 한 최소 예시만 두고, 긴 정책 본문은 공통 문서로 위임한다.
- export bundle 의 `bundle/source-docs/schemas/read_only_transport_descriptors.json` 는 read-only MCP 연결 검토용 draft descriptor 로 취급한다.
- descriptor 의 `transport_ready` 값이 `false` 인 동안에는 실제 MCP 연결보다 참고 산출물로 두는 편이 안전하다.

## bootstrap 확장 TODO

- `scripts/bootstrap_workflow_kit.py` 에 `{slug}` 하네스 생성 함수를 추가한다.
- `SUPPORTED_HARNESSES`, `HARNESS_DEFINITIONS`, `HARNESS_FILE_BUILDERS` 에 `{slug}` 를 등록한다.
- `tests/check_bootstrap.py` 에 `{slug}` 생성 검증을 추가한다.
- descriptor export 위치와 draft 사용 범위를 문서화한다.

## 다음에 읽을 문서

- 하네스 허브: [../README.md](../README.md)
- 하네스 템플릿: [../_template/README.md](../_template/README.md)
- 배포 전략: [../../core/workflow_harness_distribution.md](../../core/workflow_harness_distribution.md)
"""


def render_overlay_spec(display_name: str, slug: str, args: argparse.Namespace) -> str:
    return f"""# {display_name} Overlay Spec

- 문서 목적: `{display_name}` 하네스 오버레이를 구현하기 전에 필요한 파일 목록과 연결 전략을 정리한다.
- 범위: 진입 파일, 설정 파일, 공통 workflow 참조 경로, 권한 정책 초안
- 대상 독자: 저장소 관리자, AI workflow 설계자, 하네스 통합 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./README.md`, `../../core/workflow_harness_distribution.md`, `../../scripts/bootstrap_workflow_kit.py`

## 1. 하네스 식별자

- slug:
- `{slug}`
- display name:
- `{display_name}`

## 2. 예상 진입 파일

- 루트 진입 파일:
- `{args.root_entrypoint}`
- 설정 파일:
- `{args.config_file}`
- 추가 overlay 파일:
- TODO

## 3. 공통 workflow 연결 규칙

- 항상 `ai-workflow/memory/session_handoff.md` 를 우선 읽게 연결한다.
- 항상 `ai-workflow/memory/work_backlog.md` 를 참조하게 연결한다.
- 항상 `ai-workflow/memory/PROJECT_PROFILE.md` 를 참조하게 연결한다.
- 필요하면 `ai-workflow/memory/repository_assessment.md` 를 adoption 단계 보조 문서로 사용한다.
- read-only MCP draft descriptor 는 `bundle/source-docs/schemas/read_only_transport_descriptors.json` 위치를 기준으로 검토한다.

## 4. 구현 체크리스트

- bootstrap 생성 함수 추가
- 레지스트리 등록
- smoke test 추가
- README 문서 갱신
- descriptor export 산출물 포함 여부 확인

## 다음에 읽을 문서

- 하네스 패키지 안내: [./README.md](./README.md)
- 하네스 템플릿: [../_template/README.md](../_template/README.md)
"""


def build_manifest(harness_dir: Path, files: dict[str, str], slug: str, display_name: str) -> dict[str, object]:
    return {
        "harness_name": slug,
        "display_name": display_name,
        "harness_dir": str(harness_dir),
        "generated_files": files,
        "next_steps": [
            "Fill in the root entrypoint and config file placeholders.",
            "Add a harness file builder to scripts/bootstrap_workflow_kit.py.",
            "Register the new harness in SUPPORTED_HARNESSES, HARNESS_DEFINITIONS, and HARNESS_FILE_BUILDERS.",
            "Extend tests/check_bootstrap.py for the new harness.",
        ],
    }


def main() -> int:
    args = parse_args()
    try:
        slug = slugify(args.harness_name)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    display_name = args.display_name or titleize(slug)
    harness_dir = SOURCE_ROOT / "harnesses" / slug
    readme_path = harness_dir / "README.md"
    overlay_spec_path = harness_dir / "overlay_spec.md"

    try:
        write_text(
            readme_path,
            render_harness_readme(display_name, slug, args),
            force=args.force,
        )
        write_text(
            overlay_spec_path,
            render_overlay_spec(display_name, slug, args),
            force=args.force,
        )
    except FileExistsError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    manifest = build_manifest(
        harness_dir,
        {
            "readme": str(readme_path),
            "overlay_spec": str(overlay_spec_path),
        },
        slug,
        display_name,
    )
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
