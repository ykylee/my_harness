"""Render functions for the generated kit's doc artefacts.

Every ``render_*`` function takes the parsed CLI args and the inferred
project context, and returns the full text of the corresponding
generated artefact. The functions are pure (no side effects), so the
caller decides when and where to write the output.

Templates live in ``workflow-source/templates/`` and are loaded with
:func:`load_template`. The renderers substitute placeholders like
``YYYY-MM-DD`` or ``<Project Name>``.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path

#: Filesystem root of the bootstrap kit (the directory that ships ``templates/`` etc.).
#: Resolved at import time so renderers can load templates without threading it
#: through every call site. Tests can monkey-patch this if they need to swap roots.
SOURCE_ROOT = Path(os.environ.get("STANDARD_AI_WORKFLOW_ROOT", Path(__file__).resolve().parents[2])).resolve()


def _selected_harnesses(args: argparse.Namespace) -> list[str]:
    """Helper: extract the selected harness list from args (imported lazily to avoid cycles)."""
    # Import here to avoid circular import (__main__ imports renderers, renderers
    # only needs to know the result, not selected_harnesses itself).
    from bootstrap_lib.__main__ import selected_harnesses

    return selected_harnesses(args)


def _value_or_inferred(explicit: str | None, fallback: str) -> str:
    """Helper: prefer explicit CLI value unless it is empty or a ``TODO:`` placeholder."""
    from bootstrap_lib.discovery import value_or_inferred

    return value_or_inferred(explicit, fallback)


def render_readme(
    args: argparse.Namespace,
    context: dict[str, object],
    *,
    default_core_docs: list[str],
) -> str:
    if args.copy_core_docs:
        core_docs = "\n".join(
            f"- [core/{name}](./core/{name})" for name in default_core_docs
        )
    else:
        core_docs = "- core 문서는 `--copy-core-docs` 옵션을 사용하면 함께 복사할 수 있다."
    generated_assessment = ""
    mode_summary = "신규 프로젝트용 기본 문서 세트를 생성했다."
    harness_lines = "\n".join(
        f"- `{name}` 하네스용 오버레이 파일 생성" for name in _selected_harnesses(args)
    ) or "- 선택한 하네스 없음"
    if args.adoption_mode == "existing":
        generated_assessment = (
            "- [ai-workflow/memory/repository_assessment.md]"
            "(./ai-workflow/memory/repository_assessment.md)"
        )
        mode_summary = "기존 프로젝트 분석 결과를 반영한 문서 초안과 평가 문서를 생성했다."
    return f"""# Standard AI Workflow Kit

- 문서 목적: `{args.project_name}` 저장소에 표준 AI 워크플로우 기본 문서 세트를 도입할 수 있도록 bootstrap 결과를 안내한다.
- 범위: 공통 코어 문서 위치, 프로젝트 상태 문서 세트, 도입 모드별 후속 작업
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `docs/PROJECT_PROFILE.md`, `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`

## 1. 도입 모드

- 선택한 도입 모드: `{args.adoption_mode}`
- 요약:
- {mode_summary}

## 2. 생성된 파일

- [docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- [ai-workflow/memory/state.json](./memory/state.json)
- [ai-workflow/memory/session_handoff.md](./memory/session_handoff.md)
- [ai-workflow/memory/work_backlog.md](./memory/work_backlog.md)
- [ai-workflow/memory/backlog/{args.today}.md](./memory/backlog/{args.today}.md)
{generated_assessment}

## 3. 코어 문서

{core_docs}

## 4. 하네스 오버레이

{harness_lines}

## 5. 도입 직후 해야 할 일

1. `PROJECT_PROFILE.md` 에 프로젝트 목적, 명령, 검증 규칙을 실제 값으로 채운다.
2. `state.json`, `session_handoff.md`, 오늘 날짜 backlog 를 현재 진행 작업 기준으로 갱신한다.
3. 기존 프로젝트 모드였다면 `repository_assessment.md` 의 추정값을 실제 저장소 규칙과 대조해 수정한다.
4. 선택한 하네스가 있으면 생성된 overlay 파일을 각 하네스 실행 경로에 맞게 검토한다.
5. 이후 표준 skill/MCP 도입 범위는 `core/` 문서를 기준으로 결정한다.

## 6. 언어와 컨텍스트 운영 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, handoff/backlog 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 중간 분류는 모델이 가장 효율적인 형태로 처리하고, 사용자에게는 필요한 결론만 짧게 전달한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 7. 프로젝트 실제 문서 경로 설정값

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- 백로그 위치: `{context['backlog_dir']}`
- 세션 인계 문서 위치: `{context['session_doc_path']}`
- 환경 기록 위치: `{context['environment_dir']}`

## 다음에 읽을 문서

- 프로젝트 프로파일: [../docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- 빠른 상태 요약: [./memory/state.json](./memory/state.json)
- 세션 인계 문서: [./memory/session_handoff.md](./memory/session_handoff.md)
- 작업 백로그 인덱스: [./memory/work_backlog.md](./memory/work_backlog.md)
"""


def load_template(name: str) -> str:
    template_path = SOURCE_ROOT / "templates" / name
    if not template_path.exists():
        return f"MISSING TEMPLATE: {name}"
    return template_path.read_text(encoding="utf-8")


def render_project_profile(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("project_workflow_profile_template.md")
    install_command = _value_or_inferred(args.install_command, str(context["install_command"]))
    run_command = _value_or_inferred(args.run_command, str(context["run_command"]))
    quick_test_command = _value_or_inferred(
        args.quick_test_command, str(context["quick_test_command"])
    )
    isolated_test_command = _value_or_inferred(
        args.isolated_test_command, str(context["isolated_test_command"])
    )
    smoke_check_command = _value_or_inferred(
        args.smoke_check_command, str(context["smoke_check_command"])
    )

    replacements = {
        "<Project Name>": args.project_name,
        "<핵심 사용자 가치 및 목표>": _value_or_inferred(
            args.project_purpose, "TODO: 프로젝트 목적 정리"
        ),
        "<협업 부서 및 담당자>": _value_or_inferred(
            args.stakeholders, "TODO: 주요 이해관계자 정리"
        ),
        "<README.md>": str(context["doc_home"]),
        "<docs/operations/>": str(context["operations_dir"]),
        "<ai-workflow/memory/backlog/>": str(context["backlog_dir"]),
        "<ai-workflow/memory/session_handoff.md>": str(context["session_doc_path"]),
        "<ai-workflow/memory/repository_assessment.md>": str(context["environment_dir"]),
        "<설치 및 가상환경 구성 명령>": install_command,
        "<어플리케이션 실행 명령>": run_command,
        "<단위 테스트 및 Lint 명령>": quick_test_command,
        "<Docker 또는 독립 환경 테스트 명령>": isolated_test_command,
        "<상태 체크 및 E2E 확인 명령>": smoke_check_command,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_session_handoff(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("session_handoff_template.md")

    current_focus = "TODO: Summarize the current session focus."
    in_progress = f"{args.initial_task_id} {args.initial_task_name}"
    blocked = "N/A"
    completed = "N/A"
    key_change = "Initial workflow docs generated."
    next_action = "Review and refine generated workflow docs."
    risk_or_blocker = "N/A"

    if args.adoption_mode == "existing":
        stack_labels = context.get("stack_labels") or []
        if len(stack_labels) > 1:
            stack_summary = (
                f"inferred primary stack: {context['primary_stack']}; "
                f"all detected stacks: {', '.join(stack_labels)}"
            )
        else:
            stack_summary = f"inferred primary stack: {context['primary_stack']}"
        current_focus = f"Existing codebase onboarding completed; {stack_summary}."
        completed = "Repository scan completed"
        key_change = "Generated initial workflow docs from the existing repository scan."
        next_action = "Validate generated profile, handoff, and backlog against the repository."

    replacements = {
        "<CURRENT_FOCUS>": current_focus,
        "<IN_PROGRESS_ITEM>": in_progress,
        "<BLOCKED_ITEM>": blocked,
        "<DONE_ITEM>": completed,
        "<KEY_CHANGE>": key_change,
        "<NEXT_ACTION>": next_action,
        "<RISK_OR_BLOCKER>": risk_or_blocker,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_backlog_index(args: argparse.Namespace) -> str:
    content = load_template("work_backlog_template.md")
    replacements = {
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_daily_backlog(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("daily_backlog_template.md")

    task_goal = "TODO: 작업 목표"
    done_criteria = "TODO: 완료 기준"
    progress = f"`{args.today} 09:00` bootstrap 초기 생성"

    if args.adoption_mode == "existing":
        task_goal = "기존 프로젝트 분석 및 워크플로우 도입"
        done_criteria = "profile/handoff/backlog 초안 생성 및 검토 완료"
        progress = f"`{args.today} 09:00` 기존 저장소 분석 및 문서 생성 완료"

    replacements = {
        "TASK-XXX": args.initial_task_id,
        "<작업명>": args.initial_task_name,
        "planned | in_progress | done | blocked": args.initial_task_status,
        "high | medium | low": args.initial_priority,
        "<name>": args.owner,
        "<file_paths>": f"{context['session_doc_path']}, {context['backlog_dir']}",
        "TODO: 작업 목표": task_goal,
        "TODO: 완료 기준": done_criteria,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    content = content.replace("- 진행 현황:", f"- 진행 현황: {progress}")
    return content


def render_project_status_assessment(args: argparse.Namespace) -> str:
    content = load_template("project_status_assessment_template.md")
    return content.replace("<Project Name>", args.project_name).replace(
        "<YYYY-MM-DD>", args.today
    )


def render_assessment(args: argparse.Namespace, context: dict[str, object]) -> str:
    if args.adoption_mode != "existing":
        return ""
    top_entries = ", ".join(context["top_level_entries"]) or "없음"
    docs_dirs = ", ".join(context["docs_dirs"]) or "없음"
    test_dirs = ", ".join(context["test_dirs"]) or "없음"
    source_dirs = ", ".join(context["source_dirs"]) or "없음"
    stack_labels = ", ".join(context["stack_labels"]) or "없음"
    scripts = ", ".join(sorted(context["package_scripts"])) or "없음"
    sample_paths = "\n".join(f"- `{item}`" for item in context.get("sample_paths", []))
    return f"""# Repository Assessment

- 문서 목적: 기존 프로젝트에 표준 AI 워크플로우를 도입하기 전에 현재 코드베이스와 문서 구조를 빠르게 진단한다.
- 범위: 저장소 구조, 추정 기술 스택, 문서 위치, 테스트 흔적, 초기 워크플로우 도입 포인트
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `./PROJECT_PROFILE.md`, `./session_handoff.md`, `../core/workflow_adoption_entrypoints.md`

## 1. 요약

- 분석 대상 프로젝트:
- `{args.project_name}`
- 분석 모드:
- `existing`
- 추정 기본 스택:
- `{context['primary_stack']}`
- 감지된 스택 라벨:
- `{stack_labels}`

## 2. 저장소 구조 관찰

- 상위 디렉터리 항목:
- `{top_entries}`
- 소스 디렉터리 후보:
- `{source_dirs}`
- 문서 디렉터리 후보:
- `{docs_dirs}`
- 테스트 디렉터리 후보:
- `{test_dirs}`

## 3. 추정 명령

- 설치:
- `{context['install_command']}`
- 로컬 실행:
- `{context['run_command']}`
- 빠른 테스트:
- `{context['quick_test_command']}`
- 격리 테스트:
- `{context['isolated_test_command']}`
- 실행 확인:
- `{context['smoke_check_command']}`

## 4. package script 및 경로 샘플

- package script 목록:
- `{scripts}`
- 분석 중 확인한 경로 샘플:
{sample_paths or '- 없음'}

## 5. 워크플로우 도입 초안

- 추천 문서 위키 홈:
- `{context['doc_home']}`
- 추천 운영 문서 위치:
- `{context['operations_dir']}`
- 추천 backlog 위치:
- `{context['backlog_dir']}`
- 추천 session handoff 위치:
- `{context['session_doc_path']}`

## 6. 자동 분석 기반 다음 작업

- 현재 추정 명령과 실제 운영 명령이 일치하는지 확인한다.
- 기존 문서 체계가 있으면 운영 문서 위치를 그대로 따를지, 별도 워크플로우 디렉터리로 분리할지 결정한다.
- 빠른 테스트와 실행 확인 기준이 약하면 우선 profile 문서에서 검증 규칙을 먼저 보강한다.

## 다음에 읽을 문서

- 프로젝트 프로파일: [./PROJECT_PROFILE.md](./PROJECT_PROFILE.md)
- 세션 인계 문서: [./session_handoff.md](./session_handoff.md)
- 도입 분기 가이드: [../core/workflow_adoption_entrypoints.md](../core/workflow_adoption_entrypoints.md)
"""


__all__ = [
    "SOURCE_ROOT",
    "load_template",
    "render_assessment",
    "render_backlog_index",
    "render_daily_backlog",
    "render_project_profile",
    "render_project_status_assessment",
    "render_readme",
    "render_session_handoff",
]
