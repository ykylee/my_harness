"""Per-harness renderers and write_*_harness_files dispatchers."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from bootstrap_lib.harnesses import register_harness_builder
from bootstrap_lib.paths import (
    Paths,
    antigravity_agents_path,
    codex_agents_path,
    codex_config_example_path,
    gemini_cli_agents_path,
    minimax_agents_path,
    opencode_agent_path,
    opencode_code_worker_agent_path,
    opencode_config_path,
    opencode_doc_worker_agent_path,
    opencode_skill_path,
    opencode_validation_worker_agent_path,
    opencode_worker_agent_path,
)
from bootstrap_lib.writes import rel, write_text


def render_gemini_cli_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# GEMINI.md

- 문서 목적: Gemini CLI 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Gemini CLI, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Gemini CLI 전용 메모

- Gemini CLI 는 프로젝트 루트의 `GEMINI.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- `GEMINI.md` 에 기재된 지침은 시스템 프롬프트보다 우선하는 강력한 지침으로 취급한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 서브 에이전트(`invoke_agent`)로 분리하는 패턴을 권장한다.
- 서브 에이전트에게는 책임 범위와 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- {harness_note}
"""


def antigravity_agents_path(paths: Paths) -> Path:
    return paths.target_root / "ANTIGRAVITY.md"


def minimax_agents_path(paths: Paths) -> Path:
    """Return the MiniMax Code entry file path (project root ``MiniMax.md``)."""
    return paths.target_root / "MiniMax.md"




def render_antigravity_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# ANTIGRAVITY.md

- 문서 목적: Antigravity 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Antigravity, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## Antigravity 전용 작업 원칙

### 1. Artifacts (작업 증빙) 활용
Antigravity 에이전트는 모든 주요 의사결정과 작업 결과를 Artifacts 로 관리한다.
- **Implementation Plan**: 복잡한 수정 전에는 반드시 계획 문서를 작성하여 의도를 공유한다.
- **Task List**: 작업 단위를 쪼개어 실시간 진행 상황을 기록한다.
- **Walkthrough**: 작업 완료 후 변경 사항과 검증 결과를 요약하여 제출한다.

### 2. 브라우저 통합 및 서브 에이전트
UI 검증이나 외부 환경 조작이 필요한 경우, 직접 도구를 사용하는 대신 전용 **브라우저 서브 에이전트**를 활용하여 스크린샷과 녹화본을 증빙으로 확보한다.

### 3. 워크플로우 Skills 연동
`ai-workflow/skills/` 및 `scripts/` 아래의 도구들은 Antigravity 의 **Specialized Skills** 로 간주한다. 복잡한 상태 갱신이나 백로그 동기화는 직접 파일을 수정하기보다 이 도구들을 호출하여 수행하는 것을 권장한다.

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Antigravity 전용 메모

- Antigravity 는 프로젝트 루트의 `ANTIGRAVITY.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- `ANTIGRAVITY.md` 에 기재된 지침은 시스템 프롬프트보다 우선하는 강력한 지침으로 취급한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 브라우저 서브 에이전트 등 적절한 서브 에이전트로 분리하는 패턴을 권장한다.
- 서브 에이전트에게는 책임 범위와 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- {harness_note}
"""


def render_minimax_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    """Render ``MiniMax.md`` — the MiniMax Code harness entry file."""
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    smoke_check = context['smoke_check_command']
    if "TODO" in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# MiniMax.md

- 문서 목적: MiniMax Code(Mavis / 미니맥스 코드) 하네스가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령, 오케스트레이터/워커 운영 원칙
- 대상 독자: MiniMax Code, 저장소 관리자, 멀티 에이전트 운영자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`, `AGENTS.md`

## 목적

이 저장소에서는 **Standard AI Workflow**를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다. MiniMax Code는 메인 orchestrator로 동작하고, doc/code/validation worker에 bounded scope 작업을 위임해 컨텍스트를 절약한다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `AGENTS.md` (워크플로우 규칙 요약)

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.
- 가능한 한 메인 orchestrator는 조정과 통합에 집중하고, 도구 호출/탐색/수정은 `.MiniMax/agents/workflow-*.md` 워커에 위임한다.

## 오케스트레이터 / 워커 운영 원칙 (Multi-Agent Topology)

- **Orchestrator (Mavis / 미니맥스 코드 메인 에이전트)**: 사용자 직접 소통, 작업 분해, 워커 호출/통합, `state.json`/`session_handoff`/`work_backlog` 동기화 전담. 도구 호출을 직접 떠안지 않는다.
- **doc-worker**: 문서 링크/메타데이터/카탈로그 정합성 작업. `ai-workflow/skills/doc-sync`, `merge-doc-reconcile`, `workflow-linter` 호출.
- **code-worker**: 코드 수정/리팩토링 작업. `ai-workflow/skills/code-index-update`, `robust-patcher` 호출. 출력 파일 범위는 `output_files` 명시.
- **validation-worker**: 테스트/스모크 실행 및 결과 기록. `ai-workflow/skills/validation-plan`, `ai-workflow/tests/check_*.py` 호출.

워커에 작업을 위임할 때는 `WorkerTask` (worker_id, task_description, input_files, output_files, constraints, context_summary) 형식으로 의도와 책임 경계를 명확히 적는다. 결과는 `WorkerResponse` (status, summary, produced_artifacts, risks_identified, suggested_follow_up) 형식으로 받는다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## MiniMax Code 전용 메모

- MiniMax Code는 `MiniMax.md` 와 `AGENTS.md` 모두를 진입점으로 활용한다. 시스템 정책과 충돌할 경우 MiniMax.md 가 우선하되, 두 문서가 같은 사실을 가리키는 방향으로 동기화한다.
- `minimax_config_example.json` 는 사용자 환경 설정(`~/.MiniMax/config.json` 또는 프로젝트 로컬 `.MiniMax/config.json`)에 복사해 사용한다. 서버 토큰 등은 직접 채워 넣는다.
- 워커 호출 시 위험한 외부 작업(예: 데이터베이스 마이그레이션, 프로덕션 배포, 시크릿 회전)은 사용자 명시적 승인을 먼저 받는다.
- {harness_note}
"""


def render_minimax_config_example() -> str:
    """Render a ``MiniMax_config.example.json`` snippet for the user to copy.

    The values are intentionally placeholders; users fill in their own MCP
    server tokens, project name, and harness-specific options.
    """
    return """{
  "$schema": "https://MiniMax.dev/schema/config.json",
  "project_name": "Standard AI Workflow Project",
  "language": "ko-KR",
  "agents": {
    "workflow-orchestrator": {
      "file": ".MiniMax/agents/workflow-orchestrator.md",
      "role": "orchestrator"
    },
    "workflow-worker": {
      "file": ".MiniMax/agents/workflow-worker.md",
      "role": "worker"
    },
    "workflow-doc-worker": {
      "file": ".MiniMax/agents/workflow-doc-worker.md",
      "role": "doc-worker"
    },
    "workflow-code-worker": {
      "file": ".MiniMax/agents/workflow-code-worker.md",
      "role": "code-worker"
    },
    "workflow-validation-worker": {
      "file": ".MiniMax/agents/workflow-validation-worker.md",
      "role": "validation-worker"
    }
  },
  "mcp_servers": {
    "standard-ai-workflow-readonly": {
      "command": "python3",
      "args": ["-m", "workflow_kit.server.read_only_jsonrpc", "--stdio-lines"],
      "env": {
        "PYTHONPATH": "./workflow-source"
      },
      "transport_ready": false,
      "description": "Read-only MCP draft fixture for the standard workflow kit. See workflow-source/schemas/read_only_transport_descriptors.json."
    }
  },
  "workflow": {
    "memory_dir": "ai-workflow/memory",
    "session_handoff_path": "ai-workflow/memory/session_handoff.md",
    "work_backlog_index_path": "ai-workflow/memory/work_backlog.md",
    "project_profile_path": "docs/PROJECT_PROFILE.md",
    "state_json_path": "ai-workflow/memory/state.json"
  },
  "session_protocol": {
    "language": "ko-KR",
    "auto_refresh_state": true,
    "require_handoff_before_done": true
  }
}
"""


def render_minimax_orchestrator(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the orchestrator overlay describing the orchestrator's role."""
    return f"""# workflow-orchestrator

- 문서 목적: MiniMax Code 메인 orchestrator 페르소나의 책임/경계/산출물을 정의한다.
- 범위: 작업 분해, 워커 위임, handoff/state 동기화, 사용자 보고
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `../../../MiniMax.md`, `../../../AGENTS.md`, `workflow-worker.md`

## 책임

1. 사용자 요청을 받아 bounded-scope 작업 단위로 분해한다.
2. 각 작업을 `WorkerTask` 형식으로 워커(doc/code/validation)에 위임한다.
3. 워커의 `WorkerResponse` 를 모아서 `state.json` / `session_handoff.md` / 최신 `backlog` 를 갱신한다.
4. 사용자에게 한국어로 짧은 진행 보고와 다음 행동을 안내한다.

## 절대 하지 말 것

- 직접 `read_file` / `edit_file` 로 프로젝트 코드를 수정하지 않는다. (code-worker에 위임)
- 직접 `bash` 로 테스트/스모크를 실행하지 않는다. (validation-worker에 위임)
- 워커가 보고한 사실 외에 추측성 결론을 추가하지 않는다.

## 종료 조건

- 모든 위임 작업이 `WorkerResponse.status == "ok"` 또는 명시적 blocked 사유와 함께 반환됨
- `state.json` 의 `session.last_orchestrator_action` 이 이번 세션의 최종 행동으로 갱신됨
- `session_handoff.md` 의 "다음 세션 시작 포인트" 가 한 문장으로 갱신됨
"""


def render_minimax_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the generic worker overlay (sub-agent operating contract)."""
    return f"""# workflow-worker

- 문서 목적: MiniMax Code sub-worker 의 공통 운영 계약을 정의한다.
- 범위: 입력, 책임, 산출물, 통신 형식
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `../../../workflow-source/core/workflow_agent_topology.md`, `../../../workflow-source/prompts/code_worker_prompt.md`, `../../../workflow-source/prompts/doc_worker_prompt.md`, `../../../workflow-source/prompts/validation_worker_prompt.md`

## 입력

- orchestrator 가 위임한 `WorkerTask` (worker_id, task_description, input_files, output_files, constraints, context_summary)

## 책임

1. `output_files` 명시 범위 내에서만 변경한다.
2. 변경 후 `produced_artifacts`, `risks_identified`, `suggested_follow_up` 을 함께 보고한다.
3. 정적 검증 실패나 외부 시스템 호출이 필요하면 validation-worker에 협업 위임한다.

## 절대 하지 말 것

- 다른 워커의 `output_files` 를 수정하지 않는다.
- 명시되지 않은 의존성 추가/제거를 하지 않는다.

## 산출물

- `WorkerResponse` (status, summary, produced_artifacts, risks_identified, suggested_follow_up, raw_worker_output)
"""


def render_minimax_doc_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the doc-worker overlay."""
    return f"""# workflow-doc-worker

- 문서 목적: MiniMax Code doc-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 문서 정합성, 메타데이터, 링크, 카탈로그 동기화
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/doc_worker_prompt.md`

## 책임

1. `doc-sync` 스킬로 변경된 코드/문서가 영향 받는 문서를 식별하고 recommended review order 를 만든다.
2. `merge-doc-reconcile` 스킬로 충돌한 handoff/state/backlog 를 정리한다.
3. `workflow-linter` 스킬로 메타데이터/링크/카탈로그 정합성을 검사하고 복구한다.
4. 결과는 `output_files` 안의 문서들에 한정해 직접 수정한다.

## 금지

- 코드를 수정하지 않는다 (code-worker 영역)
- `backlog-update` 로 상태를 갱신할 때는 orchestrator 에게 명시적 위임을 요청한다
"""


def render_minimax_code_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the code-worker overlay."""
    return f"""# workflow-code-worker

- 문서 목적: MiniMax Code code-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 코드 구현, 정밀 리팩토링, 회귀 수정
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/code_worker_prompt.md`

## 책임

1. orchestrator 가 위임한 bounded scope 안에서만 코드를 수정한다.
2. `code-index-update` 스킬로 코드 인덱스/카탈로그를 동기화한다.
3. `robust_patcher` 스킬로 정밀 패치를 적용한다.
4. 변경 후 `produced_artifacts` 에 실제 변경한 파일 목록을 남긴다.

## 금지

- 명시되지 않은 파일을 수정하지 않는다.
- 의존성 추가/제거는 orchestrator 의 명시적 승인 없이 하지 않는다.
"""


def render_minimax_validation_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the validation-worker overlay."""
    return f"""# workflow-validation-worker

- 문서 목적: MiniMax Code validation-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 테스트/스모크 실행, 결과 기록
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/validation_worker_prompt.md`

## 책임

1. `validation-plan` 스킬로 변경 사항에 적합한 검증 단계를 결정한다.
2. `ai-workflow/tests/check_*.py` 와 같은 스모크/테스트 스크립트를 실행한다.
3. 실행 결과를 `passed`, `failed`, `skipped` 로 명확히 분류하고, 실패 시 raw stderr 를 `risks_identified` 에 첨부한다.

## 금지

- 코드/문서를 직접 수정하지 않는다.
- 외부 시스템 호출은 orchestrator 의 명시적 승인을 받는다.
"""


def render_codex_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# AGENTS.md

- 문서 목적: Codex 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Codex, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Codex 전용 메모

- Codex 는 프로젝트 루트의 `AGENTS.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- OpenAI 관련 질문이 나오면 OpenAI 문서 MCP 를 우선 사용하는 구성을 권장한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 worker 성격의 서브 에이전트로 분리하는 패턴을 권장한다.
- worker 에게는 책임 파일과 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- `main`/`small` 모델을 함께 운영한다면, 메인 에이전트는 난도 높은 판단과 통합에, worker 는 bounded scope 탐색/초안/검증에 우선 배치하는 편이 효율적이다.
- {harness_note}
"""


def render_codex_config_example() -> str:
    return """# Merge this into ~/.codex/config.toml if you want Codex-wide defaults.

[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"
default_tools_approval_mode = "approve"
supports_parallel_tool_calls = true
"""


def render_opencode_config(args: argparse.Namespace, paths: Paths) -> str:
    instructions = [
        "AGENTS.md",
        f"{rel(paths.state_path, paths.target_root)}",
        f"{rel(paths.profile_path, paths.target_root)}",
        f"{rel(paths.handoff_path, paths.target_root)}",
        f"{rel(paths.backlog_index_path, paths.target_root)}",
    ]
    return json.dumps(
        {
            "$schema": "https://opencode.ai/config.json",
            "instructions": instructions,
            "agent": {
                "workflow-orchestrator": {
                    "mode": "primary",
                    "description": "Standard AI workflow orchestrator for this project",
                    "prompt": "{file:.opencode/agents/workflow-orchestrator.md}",
                    "permission": {
                        "task": {
                            "*": "deny",
                            "workflow-*": "allow",
                        }
                    },
                },
                "workflow-worker": {
                    "description": "Scoped worker for implementation, draft writing, and verification tasks",
                    "prompt": "{file:.opencode/agents/workflow-worker.md}",
                },
                "workflow-doc-worker": {
                    "description": "Scoped worker for document reading, comparison, and draft updates",
                    "prompt": "{file:.opencode/agents/workflow-doc-worker.md}",
                },
                "workflow-code-worker": {
                    "description": "Scoped worker for bounded code edits and implementation tasks",
                    "prompt": "{file:.opencode/agents/workflow-code-worker.md}",
                },
                "workflow-validation-worker": {
                    "description": "Scoped worker for checks, logs, and validation evidence collection",
                    "prompt": "{file:.opencode/agents/workflow-validation-worker.md}",
                }
            },
            "mcp_servers": {
                "openaiDeveloperDocs": {
                    "type": "remote",
                    "url": "https://developers.openai.com/mcp",
                }
            },
        },
        ensure_ascii=False,
        indent=2,
    ) + "\n"


def render_opencode_skill() -> str:
    return """---
name: standard-ai-workflow
description: Load the project workflow docs before starting or updating work in this repository.
---

# Standard AI Workflow

Use this skill when you need to start a session, update backlog state, sync documents, or prepare a handoff.

Always read:

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

If the repository is still in adoption, also read:

- `ai-workflow/memory/repository_assessment.md`

Follow these rules:

- Write user-facing status updates, work reports, and document drafts in Korean by default.
- Keep code, commands, file paths, config keys, and external product names in their original form when needed.
- Brief the task before editing files.
- Keep task status aligned with backlog records.
- Do not mark work done without validation evidence.
- Update `state.json`, the handoff, and the latest backlog before ending a session.
- Keep internal reasoning and intermediate classification compact, and avoid long repeated explanations to the user.
- Leave only essential facts in handoff/backlog so session context stays lean.
- Treat `ai-workflow/` as workflow metadata only. Ignore it during normal project document exploration unless the task is explicitly about workflow docs or session state.
"""


def render_opencode_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Orchestrates the standard AI workflow for this repository
mode: primary
permission:
  edit: deny
  bash: deny
  webfetch: deny
---

You are the workflow orchestrator for this repository.

Start each substantial task by reading:

- `AGENTS.md`
- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

Treat `ai-workflow/` as a workflow metadata layer, not part of the normal project work scope. After session restoration, ignore it during project code or project document exploration unless the task explicitly asks for workflow doc maintenance.

You may directly read only the minimum session-restoration set and tiny triage inputs:

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- one clearly bounded file or path for tiny triage

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

When the repo is in adoption mode, review `ai-workflow/memory/repository_assessment.md` before trusting inferred commands.

User-facing workflow rules:

- Write visible work reports, summaries, and document drafts in Korean by default.
- Keep code, commands, file paths, config keys, and external system names in their original form when useful.
- Use concise progress updates and avoid long repeated reasoning in user-visible messages.
- Keep internal processing compact and preserve only the facts needed for the next step or next session.
- Do not call direct tools yourself. Use only task delegation for repository exploration, comparisons, implementation, checks, and draft generation.
- Use sub-agents aggressively for file exploration, comparisons, log inspection, and draft generation when that helps reduce context pollution.
- Keep the main orchestrator focused on coordination, prioritization, integration, and the final user-facing report.
- Separate broad read-heavy exploration from write tasks when possible so one stream of work does not pollute another stream's context.
- Treat this agent as a read-mostly coordinator with task-only execution: delegate edits, scans, log review, and validation to sub-agents instead of making exceptions for direct tool use.
- Keep direct read narrow: after the session-restoration set, only tiny single-file or single-path triage reads stay local; broader reading goes to workers.
- Ask the user only when a missing decision is genuinely blocking or a risky external action needs confirmation; otherwise make the smallest reasonable assumption and continue through a worker.
- When delegating, give each worker a bounded scope, clear output, and a concise completion contract.
- Prefer `workflow-doc-worker` for large document reads and draft updates, `workflow-code-worker` for bounded implementation, config edits, and build-oriented tasks, and `workflow-validation-worker` for checks and evidence collection.
- If your harness supports per-agent model selection, prefer the main model for this orchestrator and a smaller model for the worker agents by default.
- Do not treat `ai-workflow/` as part of normal project document discovery. Use it only for workflow-state restoration or explicit workflow-maintenance tasks.
"""


def render_opencode_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a workflow worker for this repository.

You are not the main orchestrator. Your role is to execute a tightly scoped task and return only the essential result.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the specific `ai-workflow/memory/` document or file paths that match your assigned scope

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned file or task scope.
- Prefer doing the actual bounded work instead of producing long plans.
- Summarize only the key facts, edits, risks, and follow-up items needed by the orchestrator.
- Avoid pasting large raw outputs when a short summary is enough.
- If you edit files, keep changes narrow and do not expand into unrelated cleanup.
- If you run checks, report only the command intent and the result that matters.
- Write user-facing drafts in Korean by default unless the assigned task clearly requires another language.
- Minimize asks during execution. Proceed with the smallest reasonable assumption unless the orchestrator explicitly requested a decision point.
- Ignore `ai-workflow/` during normal project document or source exploration unless the assigned task explicitly targets workflow docs or session-state updates.
"""


def render_opencode_doc_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    return f"""---
description: Executes bounded document-focused workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a document-focused workflow worker for this repository.

Your role is to read, compare, summarize, and update a tightly scoped set of documents without pulling unrelated context into the main orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the assigned `ai-workflow/memory/` documents or directly named doc paths

Worker rules:

- Stay within the assigned document scope.
- Prefer concise comparisons, change notes, and draft text over long quotations.
- Return only the facts, inconsistencies, draft wording, and follow-up items needed by the orchestrator.
- Keep user-facing drafts in Korean by default.
- Minimize asks during execution and resolve obvious document-structure choices locally when risk is low.
- If your harness supports per-agent model selection, this worker is a good default target for a smaller model.
- Ignore `ai-workflow/` when looking for project documentation unless the assigned task is explicitly about workflow docs or session-state maintenance.
"""


def render_opencode_code_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded implementation and build-focused workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are an implementation and build-focused workflow worker for this repository.

Your role is to implement a tightly scoped code or config change, run the minimum relevant build-oriented checks when needed, and report only the essential result back to the orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the specific source files, tests, and workflow docs tied to your assigned scope

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned write scope.
- Prefer shipping the bounded change over expanding into adjacent cleanup.
- Treat build, compile, package, or asset-generation commands as part of your default scope when they are the shortest path to proving the implementation still holds.
- If you run checks, report what matters: pass/fail, key regression risk, build impact, and any deferred follow-up.
- Avoid broad repository exploration unless explicitly assigned.
- Minimize asks during execution. Make bounded implementation choices locally unless the change would alter product behavior or ownership boundaries.
- If your harness supports per-agent model selection, use a smaller model for routine edits and reserve the main model for unusually risky or architectural code tasks.
- Ignore `ai-workflow/` during normal implementation-context discovery unless the assigned task explicitly targets workflow docs or workflow automation.
"""


def render_opencode_validation_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded validation and evidence-collection tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a validation-focused workflow worker for this repository.

Your role is to run bounded checks, inspect logs, gather evidence, and return a compact validation summary to the orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the assigned validation scope, commands, and relevant backlog or handoff notes

Project defaults:

- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned validation scope and command set.
- Report only the result that matters: what ran, what failed or passed, and what evidence should be recorded.
- Avoid flooding the orchestrator with raw logs when a short summary is enough.
- Minimize asks during execution and complete the assigned checks unless the environment is genuinely blocked.
- If your harness supports per-agent model selection, this worker is usually a strong candidate for a smaller model.
- Ignore `ai-workflow/` during normal validation-context discovery unless the assigned task explicitly targets workflow docs or session-state verification.
"""


def write_codex_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    codex_config = codex_config_example_path(paths)
    write_text(codex_config, render_codex_config_example(), force=args.force, rel_to=paths.target_root)
    return {
        "codex_config_example": str(codex_config),
    }


def write_opencode_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    opencode_config = opencode_config_path(paths)
    opencode_skill = opencode_skill_path(paths)
    opencode_agent = opencode_agent_path(paths)
    opencode_worker_agent = opencode_worker_agent_path(paths)
    opencode_doc_worker_agent = opencode_doc_worker_agent_path(paths)
    opencode_code_worker_agent = opencode_code_worker_agent_path(paths)
    opencode_validation_worker_agent = opencode_validation_worker_agent_path(paths)
    write_text(opencode_config, render_opencode_config(args, paths), force=args.force, rel_to=paths.target_root)
    write_text(opencode_skill, render_opencode_skill(), force=args.force, rel_to=paths.target_root)
    write_text(opencode_agent, render_opencode_agent(args, context), force=args.force, rel_to=paths.target_root)
    write_text(opencode_worker_agent, render_opencode_worker_agent(args, context), force=args.force, rel_to=paths.target_root)
    write_text(opencode_doc_worker_agent, render_opencode_doc_worker_agent(args, context), force=args.force, rel_to=paths.target_root)
    write_text(opencode_code_worker_agent, render_opencode_code_worker_agent(args, context), force=args.force, rel_to=paths.target_root)
    write_text(
        opencode_validation_worker_agent,
        render_opencode_validation_worker_agent(args, context),
        force=args.force,
        rel_to=paths.target_root,
    )
    return {
        "opencode_config": str(opencode_config),
        "opencode_skill": str(opencode_skill),
        "opencode_agent": str(opencode_agent),
        "opencode_worker_agent": str(opencode_worker_agent),
        "opencode_doc_worker_agent": str(opencode_doc_worker_agent),
        "opencode_code_worker_agent": str(opencode_code_worker_agent),
        "opencode_validation_worker_agent": str(opencode_validation_worker_agent),
    }


def write_gemini_cli_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    # GEMINI.md is written in write_harness_files if selected,
    # but we can also do it here if we want to be explicit or if we change write_harness_files.
    # Currently write_harness_files writes it.
    return {}


def render_pi_dev_agents(args: argparse.Namespace, context: dict[str, object]) -> str:
    return f"""# AGENTS.md (Pi Coding Agent Profile)

- **Mandate**: 본 저장소는 'Standard AI Workflow'를 따릅니다. 모든 행동은 아래 문서의 상태를 기준으로 결정하십시오.
- **Priority Docs**:
    1. `ai-workflow/memory/state.json` (현재 세션의 진실의 원천)
    2. `ai-workflow/memory/session_handoff.md` (이전 세션 인계 사항)
    3. `ai-workflow/memory/work_backlog.md` (작업 목록)

## 1. 세션 시작 루틴 (Mandatory)
세션이 시작되면 가장 먼저 `ai-workflow/memory/state.json`을 읽고 `current_focus`와 `next_documents`를 파악하십시오. 이후 `session_handoff.md`를 읽어 중단된 지점부터 작업을 재개하십시오.

## 2. 작업 원칙 (Research -> Strategy -> Execution)
- **Research**: `grep_search`와 `read_file`을 사용하여 현재 코드와 문서 상태를 객관적으로 확인하십시오.
- **Strategy**: 변경 계획을 세우고, 작업 전후에 어떤 문서를 갱신할지 결정하십시오.
- **Execution**: `edit`, `write`, `bash` 도구를 사용하여 변경을 수행하십시오.

## 3. 워크플로우 상태 관리
- 작업 상태가 변경되면 반드시 `ai-workflow/memory/backlog/`의 해당 날짜 문서를 업데이트하십시오.
- 세션 종료 전에는 `ai-workflow/memory/state.json`과 `session_handoff.md`를 갱신하여 다음 에이전트를 위한 맥락을 보존하십시오.

## 4. 도구 사용 가이드
- 복잡한 워크플로우 제어(상태 자동 갱신 등)가 필요할 때 `python3 ai-workflow/scripts/` 아래의 도구들을 활용할 수 있습니다.
- 모든 도구 호출 결과는 구조화된 JSON으로 처리하는 것을 선호합니다.

## 5. 언어 가이드
- 사용자에게 보고하거나 문서를 작성할 때는 한국어를 사용하십시오.
- 코드와 기술적 명칭은 원문을 유지하십시오.
"""


def write_pi_dev_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    # Pi Coding Agent primarily uses AGENTS.md at the root
    # We will also create a pi-dev specific apply guide if possible
    return {}


def write_antigravity_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    return {}


#: Register each harness's ``write_*_harness_files`` implementation. The
#: :data:`HARNESS_FILE_BUILDERS` registry lives in
#: :mod:`bootstrap_lib.harnesses`; we just populate it from here.
register_harness_builder("codex", write_codex_harness_files)
register_harness_builder("opencode", write_opencode_harness_files)
register_harness_builder("gemini-cli", write_gemini_cli_harness_files)
register_harness_builder("pi-dev", write_pi_dev_harness_files)
register_harness_builder("antigravity", write_antigravity_harness_files)


def write_minimax_code_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    """Generate MiniMax Code harness overlay files.

    The overlay mirrors the OpenCode orchestrator + worker split but pins the
    entry files at ``AGENTS.md`` and ``MiniMax.md`` (the latter is the
    MiniMax Code-specific entry point) and emits a JSON config example so
    the user can drop the snippet into their ``~/.MiniMax/config.json`` or
    a project-local ``.MiniMax/config.json`` without further editing.
    """
    generated: dict[str, str] = {}

    minimax_root = paths.target_root / ".MiniMax"
    minimax_config = paths.target_root / "MiniMax_config.example.json"
    minimax_orchestrator = minimax_root / "agents" / "workflow-orchestrator.md"
    minimax_worker = minimax_root / "agents" / "workflow-worker.md"
    minimax_doc_worker = minimax_root / "agents" / "workflow-doc-worker.md"
    minimax_code_worker = minimax_root / "agents" / "workflow-code-worker.md"
    minimax_validation_worker = minimax_root / "agents" / "workflow-validation-worker.md"

    write_text(minimax_config, render_minimax_config_example(), force=args.force, rel_to=paths.target_root)
    generated["minimax_config_example"] = str(minimax_config)
    write_text(minimax_orchestrator, render_minimax_orchestrator(args, context), force=args.force, rel_to=paths.target_root)
    generated["minimax_orchestrator"] = str(minimax_orchestrator)
    write_text(minimax_worker, render_minimax_worker(args, context), force=args.force, rel_to=paths.target_root)
    generated["minimax_worker"] = str(minimax_worker)
    write_text(minimax_doc_worker, render_minimax_doc_worker(args, context), force=args.force, rel_to=paths.target_root)
    generated["minimax_doc_worker"] = str(minimax_doc_worker)
    write_text(minimax_code_worker, render_minimax_code_worker(args, context), force=args.force, rel_to=paths.target_root)
    generated["minimax_code_worker"] = str(minimax_code_worker)
    write_text(minimax_validation_worker, render_minimax_validation_worker(args, context), force=args.force, rel_to=paths.target_root)
    generated["minimax_validation_worker"] = str(minimax_validation_worker)
    return generated


register_harness_builder("minimax-code", write_minimax_code_harness_files)


# ---------------------------------------------------------------------------
# Self-check: HARNESS_SPECS (single source of truth for harness metadata) must
# agree with HARNESS_FILE_BUILDERS (the actual renderer registry). If they
# drift, the CLI --harness choices will not match the renderers, which is the
# same class of bug we hit in v0.5.7.1 (sub-packages missing) and in v0.5.7
# (HARNESS_DEFINITIONS missing pi-dev).
# ---------------------------------------------------------------------------
def _verify_harness_registry_consistency() -> None:
    from bootstrap_lib.harnesses import HARNESS_FILE_BUILDERS, HARNESS_SPECS, SUPPORTED_HARNESSES

    spec_keys = set(HARNESS_SPECS)
    builder_keys = set(HARNESS_FILE_BUILDERS)
    supported = set(SUPPORTED_HARNESSES)
    missing_builder = sorted(spec_keys - builder_keys)
    missing_spec = sorted(builder_keys - spec_keys)
    missing_supported = sorted(supported - spec_keys)
    if missing_builder or missing_spec or missing_supported:
        problems = []
        if missing_builder:
            problems.append(
                "HARNESS_SPECS has entries without a renderer: " + ", ".join(missing_builder)
            )
        if missing_spec:
            problems.append(
                "HARNESS_FILE_BUILDERS has entries without a spec: " + ", ".join(missing_spec)
            )
        if missing_supported:
            problems.append(
                "SUPPORTED_HARNESSES has entries without a spec: "
                + ", ".join(missing_supported)
            )
        raise RuntimeError(
            "Harness registry drift detected. The single source of truth is "
            "HARNESS_SPECS; fix the following before releasing:\n  - "
            + "\n  - ".join(problems)
        )


_verify_harness_registry_consistency()

