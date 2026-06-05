#!/usr/bin/env python3
"""Export harness-specific workflow packages into a dist directory."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as WORKFLOW_KIT_VERSION
from workflow_kit.common.doc_transformer import DocTransformer


SUPPORTED_HARNESSES = ("codex", "opencode", "gemini-cli", "pi-dev", "antigravity")
MINIMAL_CORE_DOCS = (
    "global_workflow_standard.md",
    "workflow_adoption_entrypoints.md",
    "workflow_skill_catalog.md",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export a harness-specific workflow package into dist/."
    )
    parser.add_argument(
        "--harness",
        action="append",
        choices=list(SUPPORTED_HARNESSES),
        dest="harnesses",
        required=True,
        help="Harness package to export.",
    )
    parser.add_argument(
        "--output-dir",
        default=str(REPO_ROOT / "dist"),
        help="Root dist directory for exported packages.",
    )
    parser.add_argument(
        "--version",
        default=WORKFLOW_KIT_VERSION,
        help="Version label to stamp into the exported package.",
    )
    parser.add_argument(
        "--include-source-docs",
        action="store_true",
        help="Include developer-facing source docs in the exported package.",
    )
    parser.add_argument(
        "--include-global-snippets",
        action="store_true",
        help="Include additive global harness snippet examples in the exported package.",
    )
    return parser.parse_args()


def selected_harnesses(args: argparse.Namespace) -> list[str]:
    return sorted(dict.fromkeys(args.harnesses))


def rel(path: Path, base: Path) -> str:
    return path.relative_to(base).as_posix()


def copy_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def copy_tree(source: Path, destination: Path) -> list[Path]:
    copied: list[Path] = []
    for path in sorted(source.rglob("*")):
        if path.is_dir():
            continue
        target = destination / path.relative_to(source)
        copy_file(path, target)
        copied.append(target)
    return copied


def workflow_common_sources() -> list[Path]:
    return [
        SOURCE_ROOT / "core" / "global_workflow_standard.md",
        SOURCE_ROOT / "core" / "workflow_adoption_entrypoints.md",
        SOURCE_ROOT / "core" / "workflow_skill_catalog.md",
        SOURCE_ROOT / "core" / "existing_project_onboarding_contract.md",
        SOURCE_ROOT / "core" / "workflow_harness_distribution.md",
        SOURCE_ROOT / "core" / "workflow_release_spec.md",
        SOURCE_ROOT / "core" / "strategic_threads.md",
        SOURCE_ROOT / "core" / "maturity_matrix.json",
        SOURCE_ROOT / "core" / "automated_repro_scaffold_skill_spec.md",
        SOURCE_ROOT / "core" / "phase5_governance_guide.md",
        SOURCE_ROOT / "examples" / "end_to_end_skill_demo.md",
        SOURCE_ROOT / "mcp_servers" / "read_only_bundle.md",
        SOURCE_ROOT / "schemas" / "read_only_harness_mcp_examples.json",
        SOURCE_ROOT / "schemas" / "read_only_jsonrpc_fixtures.json",
        SOURCE_ROOT / "schemas" / "read_only_transport_descriptors.json",
        SOURCE_ROOT / "templates" / "release_note_template.md",
        SOURCE_ROOT / "templates" / "pilot_candidate_checklist.md",
        SOURCE_ROOT / "templates" / "pilot_adoption_record_template.md",
        SOURCE_ROOT / "harnesses" / "README.md",
    ]


def minimal_core_sources() -> list[Path]:
    return [SOURCE_ROOT / "core" / name for name in MINIMAL_CORE_DOCS]


def global_snippet_source_map() -> dict[str, list[Path]]:
    return {
        "codex": [
            SOURCE_ROOT / "global-snippets" / "codex" / "README.md",
            SOURCE_ROOT / "global-snippets" / "codex" / "config.toml.snippet",
        ],
        "opencode": [
            SOURCE_ROOT / "global-snippets" / "opencode" / "README.md",
            SOURCE_ROOT / "global-snippets" / "opencode" / "opencode.global.jsonc",
        ],
    }


def harness_specific_sources(harness: str) -> list[Path]:
    harness_dir = SOURCE_ROOT / "harnesses" / harness
    sources = [harness_dir / "README.md"]
    apply_guide = harness_dir / "apply_guide.md"
    if apply_guide.exists():
        sources.append(apply_guide)

    if harness == "codex":
        return sources
    if harness == "opencode":
        return sources
    if harness == "gemini-cli":
        return sources
    if harness == "pi-dev":
        return sources
    if harness == "antigravity":
        return sources
    raise ValueError(f"Unsupported harness: {harness}")


def bootstrap_export_sources(harness: str, temp_repo: Path) -> list[Path]:
    args = [
        "python3",
        str(SOURCE_ROOT / "scripts" / "bootstrap_workflow_kit.py"),
        "--target-root",
        str(temp_repo),
        "--project-slug",
        "export_sample",
        "--project-name",
        "Export Sample",
        "--harness",
        harness,
    ]
    completed = shutil.which("python3")
    if completed is None:
        raise RuntimeError("python3 is required to export harness packages.")
    import subprocess

    subprocess.run(args, cwd=REPO_ROOT, check=True, capture_output=True, text=True)

    sources = [
        temp_repo / "ai-workflow",
    ]
    if harness == "codex":
        sources.extend(
            [
                temp_repo / "AGENTS.md",
                temp_repo / ".codex" / "config.toml.example",
            ]
        )
    elif harness == "opencode":
        sources.extend(
            [
                temp_repo / "AGENTS.md",
                temp_repo / "opencode.json",
                temp_repo / ".opencode",
            ]
        )
    elif harness == "gemini-cli":
        sources.extend(
            [
                temp_repo / "GEMINI.md",
            ]
        )
    elif harness == "pi-dev":
        sources.extend(
            [
                temp_repo / "AGENTS.md",
            ]
        )
    elif harness == "antigravity":
        sources.extend(
            [
                temp_repo / "ANTIGRAVITY.md",
            ]
        )
    return sources


def copy_minimal_runtime_docs(bundle_root: Path, package_root: Path) -> list[str]:
    included_files: list[str] = []
    transformer = DocTransformer()
    for source in minimal_core_sources():
        destination = bundle_root / "ai-workflow" / "core" / source.name
        if source.suffix == ".md":
            transformer.transform_file(source, destination)
        else:
            copy_file(source, destination)
        included_files.append(rel(destination, package_root))
    return included_files


def recommended_entrypoints_for(harness: str) -> list[str]:
    common = [
        "bundle/ai-workflow/README.md",
        "bundle/ai-workflow/memory/state.json",
        "bundle/ai-workflow/memory/session_handoff.md",
        "bundle/ai-workflow/memory/work_backlog.md",
        "bundle/ai-workflow/memory/PROJECT_PROFILE.md",
        "bundle/ai-workflow/core/workflow_adoption_entrypoints.md",
        "bundle/ai-workflow/core/workflow_skill_catalog.md",
    ]
    if harness == "codex":
        return ["bundle/AGENTS.md"] + common
    if harness == "opencode":
        return [
            "bundle/AGENTS.md",
            "bundle/opencode.json",
            "bundle/.opencode/skills/standard-ai-workflow/SKILL.md",
            "bundle/.opencode/agents/workflow-orchestrator.md",
        ] + common
    if harness == "gemini-cli":
        return ["bundle/GEMINI.md"] + common
    if harness == "pi-dev":
        return ["bundle/AGENTS.md"] + common
    if harness == "antigravity":
        return ["bundle/ANTIGRAVITY.md"] + common
    raise ValueError(f"Unsupported harness: {harness}")


def package_apply_steps_for(harness: str) -> list[str]:
    if harness == "codex":
        return [
            "압축을 풀고 가능하면 `ai-workflow/scripts/apply_workflow_upgrade.py` 를 사용하여 `bundle/` 내용을 반영한다. 이 스크립트는 버전 비교와 .gitignore 셋업을 자동으로 수행한다.",
            "수동 적용 시 `bundle/AGENTS.md` 와 `bundle/ai-workflow/` 디렉터리를 대상 저장소 루트에 복사한다.",
            "선택적으로 `bundle/.codex/config.toml.example` 내용을 현재 사용자 `~/.codex/config.toml` 에 additive 방식으로 반영할지 검토한다.",
            "`AGENTS.md` 가 `ai-workflow/memory/state.json`, `session_handoff.md`, `work_backlog.md`, `PROJECT_PROFILE.md` 를 먼저 읽도록 유지한다.",
            "첫 세션에서는 `state.json`, `session_handoff.md`, `work_backlog.md`, 오늘 날짜 backlog 를 실제 저장소 상태로 갱신한다.",
        ]
    if harness == "opencode":
        return [
            "압축을 풀고 가능하면 `ai-workflow/scripts/apply_workflow_upgrade.py` 를 사용하여 `bundle/` 내용을 반영한다. 이 스크립트는 버전 비교와 .gitignore 셋업을 자동으로 수행한다.",
            "수동 적용 시 `bundle/AGENTS.md`, `bundle/opencode.json`, `bundle/.opencode/`, `bundle/ai-workflow/` 를 대상 저장소 루트에 복사한다.",
            "`opencode.json` 의 instruction 경로와 `.opencode/agents/` 권한 범위가 현재 저장소 운영 방식과 맞는지 검토한다.",
            "메인 오케스트레이터는 `.opencode/agents/workflow-orchestrator.md` 를 기준으로 두고, 직접 도구 호출 없이 worker agent 위임만 수행하는 패턴을 유지한다.",
            "worker agent 는 bounded scope 안에서 실제 읽기/수정/검증을 맡고, low-risk 실행에서는 `ask` 를 최소화하는 방향으로 운영한다.",
            "첫 세션에서는 `state.json`, `session_handoff.md`, `work_backlog.md`, 오늘 날짜 backlog 를 실제 저장소 상태로 갱신한다.",
        ]
    if harness == "gemini-cli":
        return [
            "압축을 풀고 가능하면 `ai-workflow/scripts/apply_workflow_upgrade.py` 를 사용하여 `bundle/` 내용을 반영한다. 이 스크립트는 버전 비교와 .gitignore 셋업을 자동으로 수행한다.",
            "수동 적용 시 `bundle/GEMINI.md` 와 `bundle/ai-workflow/` 디렉터리를 대상 저장소 루트에 복사한다.",
            "`GEMINI.md` 가 `ai-workflow/memory/state.json`, `session_handoff.md`, `work_backlog.md`, `PROJECT_PROFILE.md` 를 먼저 읽도록 유지한다.",
            "Gemini CLI 에서는 `GEMINI.md` 가 시스템 지침보다 우선하므로, 프로젝트 특화 규칙이 이 문서에 잘 반영됐는지 확인한다.",
            "첫 세션에서는 `state.json`, `session_handoff.md`, `work_backlog.md`, 오늘 날짜 backlog 를 실제 저장소 상태로 갱신한다.",
        ]
    if harness == "pi-dev":
        return [
            "압축을 풀고 가능하면 `ai-workflow/scripts/apply_workflow_upgrade.py` 를 사용하여 `bundle/` 내용을 반영한다. 이 스크립트는 버전 비교와 .gitignore 셋업을 자동으로 수행한다.",
            "수동 적용 시 `bundle/AGENTS.md` 와 `bundle/ai-workflow/` 디렉터리를 대상 저장소 루트에 복사한다.",
            "`AGENTS.md` 가 `ai-workflow/memory/state.json`, `session_handoff.md`, `work_backlog.md`, `PROJECT_PROFILE.md` 를 먼저 읽도록 유지한다.",
            "Pi Coding Agent 는 루트의 `AGENTS.md` 를 자동으로 시스템 지침에 주입합니다.",
            "첫 세션에서는 `state.json`, `session_handoff.md`, `work_backlog.md`, 오늘 날짜 backlog 를 실제 저장소 상태로 갱신한다.",
        ]
    if harness == "antigravity":
        return [
            "압축을 풀고 가능하면 `ai-workflow/scripts/apply_workflow_upgrade.py` 를 사용하여 `bundle/` 내용을 반영한다. 이 스크립트는 버전 비교, .gitignore 셋업, 스테일 파일 정리를 지원한다.",
            "수동 적용 시 `bundle/ANTIGRAVITY.md` 와 `bundle/ai-workflow/` 디렉터리를 대상 저장소 루트에 복사한다.",
            "`ANTIGRAVITY.md` 가 `ai-workflow/memory/state.json`, `session_handoff.md`, `work_backlog.md`, `PROJECT_PROFILE.md` 를 먼저 읽도록 유지한다.",
            "Antigravity 는 루트의 `ANTIGRAVITY.md` 를 시스템 지침에 우선 반영하며, Artifacts 와 Browser sub-agent 를 적극 활용합니다.",
            "첫 세션에서는 `state.json`, `session_handoff.md`, `work_backlog.md`, 오늘 날짜 backlog 를 실제 저장소 상태로 갱신한다.",
        ]
    raise ValueError(f"Unsupported harness: {harness}")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def render_package_contents(
    *,
    harness: str,
    package_name: str,
    version: str,
    include_source_docs: bool,
    include_global_snippets: bool,
) -> str:
    harness_runtime_files = {
        "codex": [
            "- `bundle/AGENTS.md`",
            "- `bundle/.codex/config.toml.example`",
        ],
        "opencode": [
            "- `bundle/AGENTS.md`",
            "- `bundle/opencode.json`",
            "- `bundle/.opencode/skills/standard-ai-workflow/SKILL.md`",
            "- `bundle/.opencode/agents/workflow-orchestrator.md`",
            "- `bundle/.opencode/agents/workflow-worker.md`",
            "- `bundle/.opencode/agents/workflow-doc-worker.md`",
            "- `bundle/.opencode/agents/workflow-code-worker.md`",
            "- `bundle/.opencode/agents/workflow-validation-worker.md`",
        ],
        "gemini-cli": [
            "- `bundle/GEMINI.md`",
        ],
        "pi-dev": [
            "- `bundle/AGENTS.md`",
        ],
        "antigravity": [
            "- `bundle/ANTIGRAVITY.md`",
        ],
    }[harness]
    source_docs_state = "포함됨" if include_source_docs else "기본 제외"
    global_snippets_state = "포함됨" if include_global_snippets else "기본 제외"
    return f"""# Package Contents

- 문서 목적: `{package_name}` 배포 패키지에 무엇이 들어 있는지와 어떤 파일부터 읽어야 하는지 빠르게 안내한다.
- 범위: 패키지 레이어, 포함 파일, 기본 제외 항목, 권장 진입점
- 대상 독자: 배포 패키지를 받는 AI agent 운영자, 저장소 관리자, 하네스 통합 담당자
- 상태: draft
- 최종 수정일: {datetime.now(timezone.utc).date().isoformat()}
- 관련 문서: `./manifest.json`, `./APPLY_GUIDE.md`

## 1. 패키지 식별자

- 패키지명: `{package_name}`
- 하네스: `{harness}`
- 버전: `{version}`
- 배포 프로필: `agent_runtime_minimal`

## 2. 포함 레이어

- 공통 runtime workflow 레이어
- 하네스 runtime overlay 레이어
- 패키지 메타데이터 레이어

## 3. 공통 runtime workflow 파일

- `bundle/ai-workflow/README.md`
- `bundle/ai-workflow/core/global_workflow_standard.md`
- `bundle/ai-workflow/core/workflow_adoption_entrypoints.md`
- `bundle/ai-workflow/core/workflow_skill_catalog.md`
- `bundle/ai-workflow/memory/PROJECT_PROFILE.md`
- `bundle/ai-workflow/memory/state.json`
- `bundle/ai-workflow/memory/session_handoff.md`
- `bundle/ai-workflow/memory/work_backlog.md`
- `bundle/ai-workflow/memory/backlog/2026-04-23.md`

## 4. 하네스 runtime overlay 파일

{chr(10).join(harness_runtime_files)}

## 5. 패키지 메타데이터

- `manifest.json`
- `PACKAGE_CONTENTS.md`
- `APPLY_GUIDE.md`

## 6. 기본 제외 항목

- 개발 참고용 source docs: {source_docs_state}
- 전역 설정 snippet 예시: {global_snippets_state}
- draft MCP descriptor/fixture/reference docs: 기본 제외

기본 프로필은 실제 AI agent 가 읽는 런타임 파일만 남겨 컨텍스트 낭비를 줄이는 것을 목표로 한다.

## 7. 권장 진입점

{chr(10).join(f"- `{entry}`" for entry in recommended_entrypoints_for(harness))}

## 8. 다음 단계

- 먼저 [APPLY_GUIDE.md](./APPLY_GUIDE.md) 를 읽고 대상 저장소에 복사할 파일 범위를 정한다.
- 이후 `manifest.json` 으로 실제 포함 파일과 제외 정책을 다시 확인한다.
"""


def render_apply_guide(
    *,
    harness: str,
    package_name: str,
    version: str,
) -> str:
    runtime_copy_items = {
        "codex": [
            "- `bundle/AGENTS.md -> <repo>/AGENTS.md`",
            "- `bundle/.codex/config.toml.example -> <repo>/.codex/config.toml.example`",
            "- `bundle/ai-workflow -> <repo>/ai-workflow`",
        ],
        "opencode": [
            "- `bundle/AGENTS.md -> <repo>/AGENTS.md`",
            "- `bundle/opencode.json -> <repo>/opencode.json`",
            "- `bundle/.opencode -> <repo>/.opencode`",
            "- `bundle/ai-workflow -> <repo>/ai-workflow`",
        ],
        "gemini-cli": [
            "- `bundle/GEMINI.md -> <repo>/GEMINI.md`",
            "- `bundle/ai-workflow -> <repo>/ai-workflow`",
        ],
        "pi-dev": [
            "- `bundle/AGENTS.md -> <repo>/AGENTS.md`",
            "- `bundle/ai-workflow -> <repo>/ai-workflow`",
        ],
        "antigravity": [
            "- `bundle/ANTIGRAVITY.md -> <repo>/ANTIGRAVITY.md`",
            "- `bundle/ai-workflow -> <repo>/ai-workflow`",
        ],
    }[harness]
    first_session_reads = {
        "codex": [
            "- `AGENTS.md`",
            "- `ai-workflow/memory/state.json`",
            "- `ai-workflow/memory/session_handoff.md`",
            "- `ai-workflow/memory/work_backlog.md`",
            "- `ai-workflow/memory/PROJECT_PROFILE.md`",
        ],
        "opencode": [
            "- `AGENTS.md`",
            "- `opencode.json`",
            "- `.opencode/skills/standard-ai-workflow/SKILL.md`",
            "- `.opencode/agents/workflow-orchestrator.md`",
            "- `ai-workflow/memory/state.json`",
            "- `ai-workflow/memory/session_handoff.md`",
            "- `ai-workflow/memory/work_backlog.md`",
            "- `ai-workflow/memory/PROJECT_PROFILE.md`",
        ],
        "gemini-cli": [
            "- `GEMINI.md`",
            "- `ai-workflow/memory/state.json`",
            "- `ai-workflow/memory/session_handoff.md`",
            "- `ai-workflow/memory/work_backlog.md`",
            "- `ai-workflow/memory/PROJECT_PROFILE.md`",
        ],
        "pi-dev": [
            "- `AGENTS.md`",
            "- `ai-workflow/memory/state.json`",
            "- `ai-workflow/memory/session_handoff.md`",
            "- `ai-workflow/memory/work_backlog.md`",
            "- `ai-workflow/memory/PROJECT_PROFILE.md`",
        ],
        "antigravity": [
            "- `ANTIGRAVITY.md`",
            "- `ai-workflow/memory/state.json`",
            "- `ai-workflow/memory/session_handoff.md`",
            "- `ai-workflow/memory/work_backlog.md`",
            "- `ai-workflow/memory/PROJECT_PROFILE.md`",
        ],
    }[harness]
    return f"""# Apply Guide

- 문서 목적: `{package_name}` `{version}` 패키지를 실제 저장소에 적용하는 최소 절차를 안내한다.
- 범위: 압축 해제, 파일 복사, 하네스별 확인 포인트, 첫 세션 시작 순서
- 대상 독자: 배포 패키지를 다른 환경에서 적용하는 운영자와 개발자
- 상태: draft
- 최종 수정일: {datetime.now(timezone.utc).date().isoformat()}
- 관련 문서: `./PACKAGE_CONTENTS.md`, `./manifest.json`

## 1. 적용 대상

- 기존 저장소에 workflow/skill 기반 온보딩 묶음을 얹고 싶을 때
- `{harness}` 하네스에서 바로 읽을 최소 runtime 파일만 가져가고 싶을 때
- 개발 참고 문서보다 실제 agent 소비 경로를 우선하고 싶을 때

## 2. 적용 절차

1. zip 파일을 풀거나 `bundle/` 디렉터리를 연다.
2. 아래 runtime 파일을 대상 저장소 루트에 복사한다.

{chr(10).join(runtime_copy_items)}

3. 기존 저장소에 같은 경로의 파일이 있으면 덮어쓰기 전에 프로젝트 특화 값이 이미 들어 있는지 확인한다.
4. 복사 후 하네스 진입 파일이 `ai-workflow/memory/` 문서를 먼저 읽는지 확인한다.
5. 첫 세션에서 backlog/handoff/profile 을 실제 저장소 기준으로 갱신한다.

## 3. 하네스별 확인 포인트

{chr(10).join(f"- {line}" for line in package_apply_steps_for(harness))}

## 4. 첫 세션 권장 읽기 순서

{chr(10).join(first_session_reads)}

## 5. 적용 후 바로 수정할 항목

- `ai-workflow/memory/state.json` 의 current_focus 와 next_documents
- `ai-workflow/memory/PROJECT_PROFILE.md` 의 실행/테스트/검증 명령
- `ai-workflow/memory/session_handoff.md` 의 현재 기준선
- `ai-workflow/memory/work_backlog.md` 와 최신 날짜 backlog 의 실제 작업 상태

## 6. 주의 사항

- 이 패키지는 minimal runtime profile 이므로 개발 참고용 source docs 와 전역 snippet 예시는 기본적으로 들어 있지 않다.
- 필요하면 export 원본 저장소에서 `--include-source-docs`, `--include-global-snippets` 옵션으로 다시 패키징한다.
- MCP draft 자료는 이번 릴리즈 기본 적용 경로가 아니므로, runtime 적용 전에 별도 검토 없이는 바로 연결하지 않는다.
"""


def write_package_docs(
    *,
    package_root: Path,
    harness: str,
    package_name: str,
    version: str,
    include_source_docs: bool,
    include_global_snippets: bool,
) -> list[str]:
    contents_path = package_root / "PACKAGE_CONTENTS.md"
    apply_guide_path = package_root / "APPLY_GUIDE.md"
    write_text(
        contents_path,
        render_package_contents(
            harness=harness,
            package_name=package_name,
            version=version,
            include_source_docs=include_source_docs,
            include_global_snippets=include_global_snippets,
        ),
    )
    write_text(
        apply_guide_path,
        render_apply_guide(
            harness=harness,
            package_name=package_name,
            version=version,
        ),
    )
    return [contents_path.name, apply_guide_path.name]


def export_harness(
    harness: str,
    output_root: Path,
    *,
    version: str,
    include_source_docs: bool,
    include_global_snippets: bool,
) -> dict[str, object]:
    package_root = output_root / "harnesses" / harness / version
    bundle_root = package_root / "bundle"
    if package_root.exists():
        shutil.rmtree(package_root)
    bundle_root.mkdir(parents=True, exist_ok=True)

    included_files: list[str] = []

    snippet_files: list[str] = []

    with tempfile.TemporaryDirectory() as tmpdir:
        temp_repo = Path(tmpdir) / "sample-target"
        temp_repo.mkdir(parents=True, exist_ok=True)
        for source in bootstrap_export_sources(harness, temp_repo):
            if source.is_dir():
                copied = copy_tree(source, bundle_root / source.name)
                included_files.extend(rel(path, package_root) for path in copied)
            else:
                destination = bundle_root / source.relative_to(temp_repo)
                copy_file(source, destination)
                included_files.append(rel(destination, package_root))
        profile_source = temp_repo / "docs" / "PROJECT_PROFILE.md"
        if profile_source.exists():
            profile_destination = bundle_root / "ai-workflow" / "memory" / "PROJECT_PROFILE.md"
            copy_file(profile_source, profile_destination)
            included_files.append(rel(profile_destination, package_root))

    included_files.extend(copy_minimal_runtime_docs(bundle_root, package_root))

    if include_source_docs:
        for source in workflow_common_sources() + harness_specific_sources(harness):
            destination = bundle_root / "source-docs" / rel(source, REPO_ROOT)
            copy_file(source, destination)
            included_files.append(rel(destination, package_root))

    if include_global_snippets:
        for source in global_snippet_source_map().get(harness, []):
            destination = bundle_root / "global-snippets" / rel(source, SOURCE_ROOT / "global-snippets")
            copy_file(source, destination)
            rel_dest = rel(destination, package_root)
            included_files.append(rel_dest)
            snippet_files.append(rel_dest)

    version_path = bundle_root / "ai-workflow" / "VERSION"
    version_path.parent.mkdir(parents=True, exist_ok=True)
    version_path.write_text(version, encoding="utf-8")
    included_files.append(rel(version_path, package_root))

    package_name = f"standard-ai-workflow-{harness}"
    included_files.extend(
        write_package_docs(
            package_root=package_root,
            harness=harness,
            package_name=package_name,
            version=version,
            include_source_docs=include_source_docs,
            include_global_snippets=include_global_snippets,
        )
    )
    manifest = {
        "harness": harness,
        "package_name": package_name,
        "package_version": version,
        "release_focus": "workflow_skill_onboarding",
        "optimization_profile": "agent_runtime_minimal",
        "exported_at": datetime.now(timezone.utc).isoformat(),
        "source_root": str(REPO_ROOT),
        "bundle_root": str(bundle_root),
        "included_files": sorted(included_files),
        "global_snippet_files": sorted(snippet_files),
        "recommended_entrypoints": recommended_entrypoints_for(harness),
        "deferred_release_items": [
            "official_mcp_server_default_adoption",
            "harness_mcp_activation",
        ],
        "excluded_by_default": [
            "developer_source_docs",
            "global_snippet_examples",
            "draft_mcp_reference_assets",
        ],
        "notes": [
            "This package is a generated dist artifact.",
            "This export is optimized for AI agents that read and use runtime workflow files directly.",
            "Developer-facing source docs and draft MCP assets are excluded by default to reduce context waste.",
            "Regenerate after updating core docs, harness docs, or bootstrap overlay builders.",
            "Current release focus is workflow/skill onboarding and pilot adoption preparation.",
            "MCP server activation remains deferred to a later release.",
        ],
    }
    manifest_path = package_root / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    archive_base = package_root / f"{package_name}-{version}"
    archive_path = shutil.make_archive(str(archive_base), "zip", root_dir=package_root, base_dir=".")

    return {
        "harness": harness,
        "package_name": package_name,
        "package_version": version,
        "package_root": str(package_root),
        "bundle_root": str(bundle_root),
        "manifest_path": str(manifest_path),
        "archive_path": archive_path,
        "included_files_count": len(included_files),
    }


def main() -> int:
    args = parse_args()
    output_root = Path(args.output_dir).resolve()
    exports = [
        export_harness(
            harness,
            output_root,
            version=args.version,
            include_source_docs=args.include_source_docs,
            include_global_snippets=args.include_global_snippets,
        )
        for harness in selected_harnesses(args)
    ]
    payload = {
        "output_root": str(output_root),
        "package_version": args.version,
        "exports": exports,
    }
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
