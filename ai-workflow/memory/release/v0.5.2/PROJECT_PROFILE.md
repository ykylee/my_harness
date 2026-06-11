# Project Workflow Profile

- 문서 목적: Standard AI Workflow 저장소 자체의 운영 규칙과 검증 기준을 정의한다.
- 범위: 저장소 개요, 문서 구조, 기본 명령, 검증 포인트, 예외 규칙
- 대상 독자: 저장소 maintainer, 멀티 에이전트 운영자, AI agent
- 상태: stable
- 최종 수정일: 2026-06-06
- 관련 문서: [공통 표준](../../../../workflow-source/core/global_workflow_standard.md), [Maturity Matrix](../../../../workflow-source/core/maturity_matrix.json), [Bootstrap Script](../../../../workflow-source/scripts/bootstrap_workflow_kit.py)

## 1. 프로젝트 개요

- 프로젝트명: **Standard AI Workflow**
- 프로젝트 슬러그: `standard-ai-workflow`
- 프로젝트 목적: 여러 프로젝트에서 공통으로 사용할 수 있는 표준 AI 협업 워크플로우 문서와 템플릿, skill/MCP/agent 구현 기준을 독립 프로젝트 형태로 제공한다.
- 주요 이해관계자: 저장소 maintainer (`ykylee`), 워크플로우 도입 검토자, 멀티 에이전트 운영자

## 2. 문서 구조 (Path)

- 문서 위키 홈: `README.md`
- 운영 문서 홈: `ai-workflow/memory/<branch>/`
- 백로그 위치: `ai-workflow/memory/<branch>/backlog/YYYY-MM-DD.md`
- 세션 인계 문서: `ai-workflow/memory/<branch>/session_handoff.md`
- 환경 기록 위치: `ai-workflow/memory/repository_assessment.md`
- 글로벌 인덱스: `ai-workflow/memory/work_backlog.md`
- 상태 캐시: `ai-workflow/memory/<branch>/state.json`

## 3. 기본 명령 (Commands)

- 가상환경 생성: `python3.11 -m venv .venv-review`
- 의존성 설치: `.venv-review/bin/pip install -r requirements.txt -r requirements-dev.txt`
- 부트스트랩 (기존 프로젝트 모드):
  ```bash
  python3 workflow-source/scripts/bootstrap_workflow_kit.py \
    --target-root . \
    --project-slug standard-ai-workflow \
    --project-name "Standard AI Workflow" \
    --harness minimax-code \
    --adoption-mode existing \
    --copy-core-docs \
    --force
  ```
- 빠른 테스트 (스모크 전체):
  ```bash
  for t in workflow-source/tests/check_*.py; do
    PYTHONPATH=workflow-source .venv-review/bin/python "$t" >/dev/null
  done
  ```
- 격리 테스트 (부트스트랩): `python3 workflow-source/tests/check_bootstrap.py`
- 상태 동기화:
  ```bash
  python3 workflow-source/scripts/generate_workflow_state.py \
    --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
    --session-handoff-path ai-workflow/memory/<branch>/session_handoff.md \
    --work-backlog-index-path ai-workflow/memory/work_backlog.md \
    --output-path ai-workflow/memory/<branch>/state.json
  ```
- PR: `gh pr create` / `gh pr merge --squash --delete-branch`

## 4. 검증 포인트 (Validation)

- 코드 변경: 43/43 smoke test 통과 (`workflow-source/tests/check_*.py`) + Pydantic 출력 스키마 정합 (`schemas/generated_output_schemas.json` 재생성)
- 문서 변경: `check_docs.py` 메타데이터 + 링크 검증 통과 + `workflow-linter` 통과
- 부트스트랩 변경: `check_bootstrap.py` 의 모든 모드 (new / existing / opencode / gemini-cli / antigravity / **minimax-code**) 통과
- MCP round-trip: `check_bootstrap_mcp_roundtrip.py` 5개 하네스 모두 통과
- PR 머지 전: GitHub Actions smoke test 통과 + CI run id PR 코멘트 확인

## 5. 예외 규칙 (Policy)

- 병합: `ai-workflow/memory/<branch>/state.json` 등 자동 생성 파일은 충돌 시 소스 문서 (backlog, handoff) 를 기준으로 재생성
- 승인: `core/` 문서 (특히 `global_workflow_standard.md`, `maturity_matrix.json`) 변경 시 자체 self-review 필수
- 제약: Python 3.10+ (MCP SDK), `.venv-review/`는 git 추적 금지
- 기타: `ai-workflow/` 는 self-dogfooding 대상. 워크플로우 표준 그 자체를 코드와 함께 갱신해야 함
- 자체 워크플로우 준수: 작업 시작 시 `session-start` 스킬 실행, 일별 `backlog/YYYY-MM-DD.md` 갱신, 종료 시 `state.json` 재생성
- PR 작업 패턴: 별도 fix 브랜치 만들지 말고 기존 PR 브랜치에 직접 커밋 + push (이속도 우선, PR 자동 업데이트)

## 다음에 읽을 문서

- [세션 인계 문서](./session_handoff.md)
- [작업 백로그 인덱스](../../work_backlog.md)
- [Maturity Matrix](../../../../workflow-source/core/maturity_matrix.json)
