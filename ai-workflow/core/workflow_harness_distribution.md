# Workflow Harness Distribution

- 문서 목적: 표준 AI 워크플로우를 개별 AI 하네스에 맞는 배포 단위로 변환할 때 필요한 공통 원칙과 산출물 구성을 정리한다.
- 범위: 공통 코어와 하네스 오버레이의 관계, 타겟별 생성 파일, 배포 전략, 확장 포인트
- 대상 독자: 저장소 관리자, AI workflow 설계자, 하네스 통합 담당자
- 상태: draft
- 최종 수정일: 2026-06-05
- 관련 문서: `./global_workflow_standard.md`, `./workflow_adoption_entrypoints.md`, `./workflow_configuration_layers.md`, `./workflow_global_injection_policy.md`, `../scripts/bootstrap_workflow_kit.py`, `../scripts/bootstrap_harnesses/__init__.py`

## 1. 기본 원칙

- 워크플로우 정책 원문은 `core/` 와 `ai-workflow/memory/` 문서에 둔다.
- 하네스별 파일은 정책을 복제하기보다, 공통 문서를 읽도록 연결하는 오버레이로 유지한다.
- 하네스가 요구하는 루트 파일과 설정 파일만 타겟별로 생성한다.
- 공통 규칙 변경 시 하네스 오버레이는 가능한 한 참조 경로만 유지해 중복을 줄인다.
- 이번 릴리즈의 배포 축은 MCP 활성화가 아니라 workflow/skill 기반 온보딩과 파일럿 적용 준비다.
- 배포 export 는 개발 편의보다 배포 대상 AI agent 의 컨텍스트 효율을 우선한다.

## 2. 배포 레이어

표준 배포는 아래 두 레이어로 나눈다.

1. 공통 워크플로우 레이어
2. 하네스 오버레이 레이어

공통 워크플로우 레이어에는 아래가 포함된다.

- `ai-workflow/README.md`
- `ai-workflow/core/global_workflow_standard.md`
- `ai-workflow/core/workflow_adoption_entrypoints.md`
- `ai-workflow/core/workflow_skill_catalog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/backlog/YYYY-MM-DD.md`

개발 참고용 source docs, 파일럿 템플릿, draft MCP 자료는 기본 배포 묶음에서 제외하고 필요할 때만 opt-in 포함한다.

하네스 오버레이 레이어에는 아래가 포함된다.

- 하네스가 실제로 읽는 루트 문서
- 하네스 설정 파일
- 하네스 전용 skill 또는 agent 정의 파일

## 3. Codex 타겟

Codex 타겟은 프로젝트 루트의 `AGENTS.md` 를 핵심 진입점으로 본다.

권장 산출물:

- `AGENTS.md`
- `.codex/config.toml.example`

구성 원칙:

- `AGENTS.md` 는 `ai-workflow/memory/` 문서를 먼저 읽도록 안내한다.
- `.codex/config.toml.example` 는 전역 Codex 설정에 병합할 수 있는 예시 스니펫 형태로 둔다.

## 4. OpenCode 타겟

OpenCode 타겟은 공통 상위 진입점 `AGENTS.md` 와 `opencode.json`, project-local `.opencode/` 디렉터리 구성을 함께 사용한다.

권장 산출물:

- `AGENTS.md`
- `opencode.json`
- `.opencode/skills/standard-ai-workflow/SKILL.md`
- `.opencode/agents/workflow-orchestrator.md`
- `.opencode/agents/workflow-worker.md`
- `.opencode/agents/workflow-doc-worker.md`
- `.opencode/agents/workflow-code-worker.md`
- `.opencode/agents/workflow-validation-worker.md`

구성 원칙:

- `AGENTS.md` 는 OpenCode 패키지에서도 공통 상위 지침 진입점으로 유지한다.
- `opencode.json` 의 `instructions` 는 `AGENTS.md` 와 공통 workflow 문서를 직접 가리킨다.
- skill 과 agent 파일은 공통 workflow 문서를 읽고 세션 시작, backlog 갱신, handoff 갱신을 우선 수행하도록 만든다.
- 메인 오케스트레이터는 task-only coordinator 로 두고, 직접 도구 호출 없이 실제 수정/검증/초안 작성은 worker agent 로 분리하는 구성을 기본값으로 둔다.
- worker 는 generic 하나만 두기보다 문서, 구현/빌드, 검증 성격으로 나눌수록 컨텍스트 오염과 권한 범위를 더 잘 제어할 수 있다.
- worker 는 bounded scope 안에서 실제 실행을 맡고, low-risk 범위에서는 `ask` 를 과도하게 발생시키지 않는 방향을 기본 운영값으로 둔다.

## 5. Antigravity 타겟

Antigravity 타겟은 프로젝트 루트의 `ANTIGRAVITY.md` 를 핵심 진입점으로 본다.

권장 산출물:

- `ANTIGRAVITY.md`

구성 원칙:

- `ANTIGRAVITY.md` 는 `ai-workflow/memory/` 문서를 먼저 읽도록 안내한다.
- 브라우저 서브 에이전트 등 적절한 서브 에이전트로 작업을 분리하는 패턴을 권장한다.

## 6. MiniMax Code 타겟

MiniMax Code(미니맥스 코드) 타겟은 메인 orchestrator + doc/code/validation worker 분화 패턴을 그대로 가져간다.

권장 산출물:

- `AGENTS.md` (Codex/OpenCode 와 공통)
- `MiniMax.md` (MiniMax Code 전용 진입점)
- `MiniMax_config.example.json` (사용자 환경 `.MiniMax/config.json` 으로 복사해 사용)
- `.MiniMax/agents/workflow-orchestrator.md`
- `.MiniMax/agents/workflow-worker.md`
- `.MiniMax/agents/workflow-doc-worker.md`
- `.MiniMax/agents/workflow-code-worker.md`
- `.MiniMax/agents/workflow-validation-worker.md`

구성 원칙:

- `MiniMax.md` 는 메인 orchestrator의 한국어 보고, `WorkerTask` / `WorkerResponse` 형식, 워커 책임 경계를 명시한다.
- `MiniMax_config.example.json` 의 `mcp_servers` 항목은 read-only JSON-RPC draft bridge 만 참조하고, 사용자 토큰/시크릿은 환경 변수로 분리한다.
- 워커 페르소나는 OpenCode 와 동일한 4종 (orchestrator + 3 worker) 으로 시작하고, 도메인 추가 시 동일 패턴으로 `.MiniMax/agents/` 에 누적한다.
- 새 하네스 추가는 `bootstrap_harnesses/__init__.py` 의 `HARNESS_SPECS` 한 줄과 `bootstrap_workflow_kit.py` 의 `register_harness_builder` 한 줄로 끝낸다.

## 7. 유지보수 원칙

- 하네스별 파일 안에 긴 정책 본문을 중복해서 넣지 않는다.
- 공통 문서 경로가 바뀌면 Codex/OpenCode 오버레이도 함께 갱신한다.
- bootstrap 스크립트는 하네스 선택 옵션으로 같은 공통 문서 세트 위에 오버레이만 추가하도록 유지한다.
- 하네스 전역 설정은 기본값만 제공하고, 프로젝트별 규칙은 local 문서가 우선하도록 유지한다.
- MCP 관련 descriptor 와 예시는 패키지에 포함할 수 있지만, 이번 릴리즈 기본 소비 경로는 `workflow_adoption_entrypoints` 와 `workflow_skill_catalog` 이어야 한다.
- 배포 패키지는 하네스별 개별 버전 디렉터리와 버전이 포함된 zip 이름으로 생성한다.

## 7. 확장 포인트

다른 하네스를 추가할 때는 아래 순서를 권장한다.

1. 하네스가 실제로 읽는 진입 파일이 무엇인지 먼저 고정한다.
2. 그 하네스 전용 파일 안에서는 공통 workflow 문서를 읽도록 연결한다.
3. `bootstrap_workflow_kit.py` 에 파일 생성 함수를 추가한다.
4. 하네스 레지스트리에 이름, 설명, 생성 함수를 등록한다.
5. smoke test와 하네스 문서를 함께 추가한다.

현재 bootstrap 스크립트는 하네스 레지스트리 기반으로 동작하므로, 다음 타겟은 `SUPPORTED_HARNESSES`, `HARNESS_DEFINITIONS`, `HARNESS_FILE_BUILDERS` 에 등록하는 방식으로 확장한다.

## 다음에 읽을 문서

- 도입 분기 가이드: [./workflow_adoption_entrypoints.md](./workflow_adoption_entrypoints.md)
- 릴리즈 규격: [./workflow_release_spec.md](./workflow_release_spec.md)
- 설정 계층 가이드: [./workflow_configuration_layers.md](./workflow_configuration_layers.md)
- 비침투적 주입 정책: [./workflow_global_injection_policy.md](./workflow_global_injection_policy.md)
- 하네스 템플릿: [../harnesses/_template/README.md](../harnesses/_template/README.md)
- 스크립트 안내: [../scripts/README.md](../scripts/README.md)
