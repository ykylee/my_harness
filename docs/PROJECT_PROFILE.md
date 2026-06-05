# My Harness — Project Workflow Profile

- 문서 목적: yklee 개인 코딩 에이전트 하네스의 특화 규칙과 실행/검증 기준을 정의한다.
- 범위: 하네스 개요, 문서 구조, 기본 명령, 검증 포인트, 예외 규칙
- 대상 독자: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- 상태: active
- 최종 수정일: 2026-06-05
- 관련 문서: [공통 표준](../ai-workflow/core/global_workflow_standard.md), [MiniMax 진입점](../MiniMax.md)

## 1. 프로젝트 개요
- 프로젝트명: My Harness
- 프로젝트 슬러그: my-harness
- 프로젝트 목적: yklee가 운영하는 개인 코딩 에이전트 하네스. `standard_ai_workflow` (ykylee/standard_ai_workflow)의 `minimax-code` 하네스 오버레이를 기반으로 Mavis/MiniMax Code 환경에서 동작. Mavis 메인 orchestrator + doc/code/validation 워커 분화 패턴을 채택해 컨텍스트 절약과 작업 추적성을 확보한다.
- 주요 이해관계자: yklee (오너/유지보수), Mavis orchestrator, doc/code/validation 워커
- 적용 환경: macOS (M-series), Python 3.11+, Mavis 데몬

## 2. 문서 구조 (Path)
- 문서 위키 홈: `README.md`
- 하네스 진입점: `MiniMax.md` (Mavis 메인 에이전트가 먼저 읽음)
- 워커 정의: `.MiniMax/agents/workflow-*.md`
- 워크플로우 코어: `ai-workflow/core/`
- 운영 문서 홈: `ai-workflow/memory/`
- 백로그 위치: `ai-workflow/memory/backlog/`
- 세션 인계 문서: `ai-workflow/memory/session_handoff.md`
- 환경 기록 위치: `ai-workflow/memory/environments/`
- 하네스 설정 예시: `MiniMax_config.example.json` → `.MiniMax/config.json` 으로 복사 후 사용

## 3. 기본 명령 (Commands)
- 워크플로우 상태 동기화:
  ```bash
  PYTHONPATH=./ai-workflow python3 ./ai-workflow/scripts/generate_workflow_state.py \
    --project-profile-path docs/PROJECT_PROFILE.md \
    --session-handoff-path ai-workflow/memory/session_handoff.md \
    --work-backlog-index-path ai-workflow/memory/work_backlog.md \
    --output-path ai-workflow/memory/state.json
  ```
- 워크플로우 재적용/업그레이드:
  ```bash
  python3 ./ai-workflow/scripts/bootstrap_workflow_kit.py \
    --target-root . \
    --project-slug my-harness \
    --project-name "My Harness" \
    --harness minimax-code \
    --adoption-mode new \
    --copy-core-docs \
    --force
  ```
- 백로그 갱신: `ai-workflow/skills/backlog-update` 또는 일별 `ai-workflow/memory/backlog/YYYY-MM-DD.md` 직접 편집
- 빠른 테스트 (스모크): `for t in ai-workflow/tests/check_*.py; do python3 "$t" || exit 1; done`
  - 주의: 소스 프레임워크(`workflow-source/` at root) 레이아웃 가정이라 현재 레이아웃에서는 일부 실패할 수 있음. 컨슈머 환경 전용 스모크가 필요하면 별도 추가한다.

## 4. 검증 포인트 (Validation)
- 워크플로우 변경: `state.json` 재생성 결과 `status: ok`, `MiniMax.md` / `AGENTS.md`(해당 시) / `state.json` 링크 무결성
- 문서 변경: `ai-workflow/core/global_workflow_standard.md` 규약(메타데이터, 한국어 기본, 컨텍스트 절약) 준수
- 하네스 진입점 변경: `MiniMax.md` 가 항상 `state.json` → `session_handoff.md` → `work_backlog.md` → `PROJECT_PROFILE.md` 순서로 안내하는지 확인
- 워커 변경: `.MiniMax/agents/workflow-*.md` 가 `WorkerTask` / `WorkerResponse` 스키마를 따르는지 확인
- 배포/운영: 워크플로우 업그레이드 시 `ai-workflow/scripts/apply_workflow_upgrade.py` 사용 검토

## 5. 예외 규칙 (Policy)
- 병합: `ai-workflow/memory/state.json` 등 자동 생성 파일은 충돌 시 소스 문서(backlog, handoff) 기준으로 재생성
- 승인: 하네스 오버레이(`MiniMax.md`, `.MiniMax/agents/`) 변경 시 yklee 본인이 직접 결정
- 제약:
  - `ai-workflow/` 경로는 코드베이스 시맨틱 검색/탐색 범위에서 제외
  - 워커는 메인 orchestrator의 명시적 위임 없이는 사용자 호출 받지 않음
  - 위험한 외부 작업(DB 마이그레이션, 프로덕션 배포, 시크릿 회전)은 사용자 명시적 승인 후에만 실행
- 언어: 사용자 보고/상태 요약/handoff/backlog 문안은 한국어 기본. 코드/명령/경로/설정 key는 원문 유지
- 컨텍스트: 메인 orchestrator는 가능한 한 도구 호출을 직접 떠안지 않고 워커에 위임. 사용자에게는 결론과 다음 행동만 짧게 보고

## 다음에 읽을 문서
- [Mavis 진입 규칙](../MiniMax.md)
- [세션 인계 문서](../ai-workflow/memory/session_handoff.md)
- [작업 백로그 인덱스](../ai-workflow/memory/work_backlog.md)
- [워크플로우 코어 표준](../ai-workflow/core/global_workflow_standard.md)
