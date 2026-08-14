# MiniMax.md

- 문서 목적: MiniMax Code(Mavis / 미니맥스 코드) 하네스가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령, 오케스트레이터/워커 운영 원칙
- 대상 독자: MiniMax Code, 저장소 관리자, 멀티 에이전트 운영자
- 상태: active
- 최종 수정일: 2026-08-14
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `docs/PROJECT_PROFILE.md`, `AGENTS.md`, **[docs/CONCEPT.md](./docs/CONCEPT.md)** ← **SSOT (D-135 overlay)**, [DETAILED_DESIGN_OVERLAY.md](./docs/architecture/DETAILED_DESIGN_OVERLAY.md)
  - §5.10 Agent 모드 (orchestrator/single/loop)
  - §5.11 Built-in sub-agents
  - §5.12 `~/.myharness/` 디렉토리 구조
  - §5.13 LLM Wiki memory
  - §5.14 Skill/MCP first-class
  - §11 결정 보류 (TASK-002) · D-135 overlay 확정

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
- **코드 commit + 메모리 동기화는 단일 push 에 포함한다** (2026-06-14 워크플로우 점검). 협업자가 main fetch 시점에 결정 ID + 메모리가 항상 정합하도록:
  1. 로컬: 코드/문서 작업 + 검증 + 메모리 동기화 (`state.json` + `session_handoff.md` + `work_backlog.md` + 신규 `backlog/YYYY-MM-DD.md`)
  2. staging: `git add feat_files + memory_files` (함께)
  3. commit + push:
     - 옵션 A: 1 commit (코드 + 메모리 한 commit)
     - 옵션 B: 2 commit (feat commit → 메모리 commit) + **단일 push** 에 둘 다
  4. **코드 commit message trailer 에 결정 ID 명시**:
     ```
     feat(<scope>): <subject>

     <body>

     Refs: D-NN (TASK-XXXX v2.0 Sub-task N Commit X)
     Tests: <count> pass + <N> ignored
     Clippy: 0 warning
     Binary: <delta>
     ```
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

## 프로젝트 실행 기본값 (도메인별)

이 하네스는 **코드 개발 / 서버 관리 / 환경 셋업** 세 도메인의 작업 진입점이다. 각 도메인별 표준 명령은 **[`docs/CONCEPT.md`](./docs/CONCEPT.md) §5.2 (명령 가이드)** 가 SSOT — 본 섹션은 그 참조 + 점진 채움용.

**컨셉 핵심 (CONCEPT.md §0, D-140)**: 화면 = `surface/` · 엔진 = 숨긴 `grok` · MiniMax. 설계 `docs/architecture/DETAILED_DESIGN_SURFACE.md`.

### 코드 개발

- v1 명령: `myharness code review|implement|test|commit` (CONCEPT.md §5.2 코드 도메인)
- **TASK-005 결정 (D-36, 2026-06-07, `docs/development_log.md` §5)**: 스택 = **Rust 1안**. Cargo workspace (`myharness/Cargo.toml`) + 8 member crates (core / llm / tui / tools / context / cli / auth / compression). 의존성: `rig-core = "0.38"`, `rmcp = "1.7"`, `ratatui`, `keyring`, `cargo-dist`.
- **TASK-006 결정 (D-36 의 TASK-005 Rust 정합 자동 확정)**: TUI = **ratatui** (`myharness/crates/tui/`).
- 설치 (Rust 1안 기반): `cargo build --release --manifest-path myharness/Cargo.toml` — release binary `myharness/target/release/myharness` 산출
- 로컬 실행: `./target/release/myharness --mode=orchestrator` (D-29, 3-모드: orchestrator/single/loop)
- 빠른 테스트: `cargo test --manifest-path myharness/Cargo.toml --workspace --lib`
- 격리 테스트: `cargo test --manifest-path myharness/Cargo.toml -p <crate-name> --lib` (예: `-p myharness-core`, `-p myharness-llm`)
- 실행 확인: `./target/release/myharness --version` 또는 `./target/release/myharness --mode=single "echo hello"` (단일 에이전트 smoke)

### 서버 관리

- v1 명령: `myharness server status|logs|deploy|config` (CONCEPT.md §5.2 서버 도메인)
- 서버 호스트 목록 / SSH 별칭: `TODO` (`~/.ssh/config` 별칭 또는 운영 alias)
- 헬스체크: `TODO` (curl, ping, systemctl 등 도메인별 헬퍼)
- 로그 확인: `TODO` (journalctl, lnav, tail 패턴)
- 설정 변경: `TODO` (dotfiles 저장소 path + 적용 명령)
- 배포: `TODO`

### 환경 셋업

- v1 명령: `myharness env setup|install|shell|diagnose` (CONCEPT.md §5.2 환경 도메인)
- Homebrew 패키지 목록: `TODO` (`brew bundle --file=Brewfile` 패턴)
- 런타임 버전 매니저: `TODO` (asdf / rtx / mise 등)
- dotfiles / 셸 설정: `TODO` (저장소 path + 동기화 명령)
- 신규 머신 부트스트랩: `TODO` (위 셋을 묶은 단일 진입점)

## 문서 작업 기준

- 문서 위키 홈: `README.md`
- 운영 문서 위치: `ai-workflow/memory/`
- backlog 위치: `ai-workflow/memory/backlog/`
- session handoff 위치: `ai-workflow/memory/session_handoff.md`

## MiniMax Code 전용 메모

- MiniMax Code는 `MiniMax.md` 와 `AGENTS.md` 모두를 진입점으로 활용한다. 시스템 정책과 충돌할 경우 MiniMax.md 가 우선하되, 두 문서가 같은 사실을 가리키는 방향으로 동기화한다.
- `minimax_config_example.json` 는 사용자 환경 설정(`~/.MiniMax/config.json` 또는 프로젝트 로컬 `.MiniMax/config.json`)에 복사해 사용한다. 서버 토큰 등은 직접 채워 넣는다.
- 워커 호출 시 위험한 외부 작업(예: 데이터베이스 마이그레이션, 프로덕션 배포, 시크릿 회전)은 사용자 명시적 승인을 먼저 받는다.
- 신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다.
