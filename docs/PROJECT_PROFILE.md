# My Harness — Project Workflow Profile

- 문서 목적: yklee 개인 코딩 에이전트 하네스의 특화 규칙과 실행/검증 기준을 정의한다.
- 범위: 하네스 개요, 문서 구조, 기본 명령, 검증 포인트, 예외 규칙
- 대상 독자: yklee (single user / 프로젝트 오너), my_harness 개발 시 사용하는 Mavis / Mavis 워커 (이 저장소 개발 workflow 한정)
- 상태: active
- 최종 수정일: 2026-07-01 (D-100~D-104 결정 log 후속 — development_log.md 정정 + D-104 oh-my-pi Hybrid Read v2 반영)
- 관련 문서: [공통 표준](../ai-workflow/core/global_workflow_standard.md) (Mavis 워크플로우 표준 — 이 저장소 개발 workflow 한정), [MiniMax 진입점](../MiniMax.md) (Mavis 진입점 — 이 저장소 개발 workflow 한정), **[CONCEPT.md](./CONCEPT.md) ← my_harness v1 컨셉 SSOT (Mavis zero coupling)**, [development_log.md](./development_log.md), [REFERENCES.md](./REFERENCES.md)
  - CONCEPT.md §5.10 Agent 모드 (orchestrator/single/loop)
  - CONCEPT.md §5.11 Built-in sub-agents (15개, 3-도메인)
  - CONCEPT.md §5.12 `~/.myharness/` 디렉토리 구조
  - CONCEPT.md §5.13 LLM Wiki memory (v2+)
  - CONCEPT.md §5.14 Skill/MCP first-class
  - CONCEPT.md §11 결정 보류 (TASK-002/005/006/007/008)

## 1. 프로젝트 개요
- 프로젝트명: My Harness
- 프로젝트 슬러그: my-harness
- 프로젝트 산출물: **my_harness** — yklee 의 **standalone CLI/TUI coding agent**. terminal 에서 `myharness <command>` 로 직접 실행, LLM provider 와 **직접 통신**, 3-도메인 (코드/서버/환경) 작업. 자세한 컨셉은 **[CONCEPT.md](./CONCEPT.md) (SSOT)** 참조.
- **산출물의 적용 도메인** (my_harness 가 작업하는 범위):
  - **코드 개발 전반** — 새 기능 구현, 리팩토링, 버그 수정, 리뷰, 테스트, PR 작업
  - **기본 서버 관리** — 프로세스/서비스 상태 점검, 로그 확인, 설정 변경, 배포 헬퍼
  - **환경 셋업** — 로컬/원격 개발 환경 부트스트랩, 의존성 설치, 셸/도구 설정
- **이 저장소(my_harness 개발 repo) 의 workflow 표준** (D-25: my_harness 산출물 자체와 무관, **개발 시**만 사용):
  - `ai-workflow/core/global_workflow_standard.md` 의 한국어 보고 / 컨텍스트 절약 / 이벤트 소싱 / 비참조 원칙 / 상태값(`planned|in_progress|blocked|done`)
  - Mavis 가 메인 orchestrator, .MiniMax/agents/ 워커에 bounded scope 작업 위임
  - 이 workflow 는 **yklee 가 my_harness 를 개발할 때** 사용. my_harness 가 동작할 때는 무관 (CONCEPT.md §5.8 참조)
- 주요 이해관계자: yklee (오너/유지보수), Mavis / Mavis 워커 (개발 workflow 한정, 산출물 무관)
- 적용 환경 (개발 시): macOS (M-series), Python 3.11+, Mavis 데몬, gh CLI, 필요 시 원격 서버 SSH
- 적용 환경 (산출물 my_harness): **사용자 terminal** (macOS / Linux / Windows), **Rust 1안** (D-36 — ratatui + rig-core + rmcp + keyring + cargo-dist), LLM provider API (Anthropic/OpenAI/Google/DeepSeek/local Ollama), (선택) headroom built-in 알고리즘

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
- **Second brain vault (D-71)**: `~/wiki/` (Obsidian 직접 open, out-of-repo, ai-workflow consumer). 디자인: [./architecture/DETAILED_DESIGN_LLM_WIKI.md](./architecture/DETAILED_DESIGN_LLM_WIKI.md). 운영 규약: `~/wiki/AGENTS.md`

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

## 3.1 도메인별 작업 명령 (Domain-specific)

**v1 명령 구조는 [`CONCEPT.md` §5.2](./CONCEPT.md) 가 SSOT** (단일 진실 공급원). 본 섹션은 v1 명령 + 점진 채움 항목 (yklee 인프라 정보 의존) 통합.

- **코드 개발** (CONCEPT.md §5.2): `myharness code review|implement|test|commit` — 각 명령 = 1 sub-agent (mini_coder_max / fullstack-dev) 위임. **TASK-005 / TASK-006 결정 (D-36, 2026-06-07, `docs/development_log.md` §5) — Rust 1안 + ratatui**. 표준 명령은 Cargo workspace 기준: `cargo build --release --manifest-path myharness/Cargo.toml`, `./target/release/myharness --mode=orchestrator`, `cargo test --manifest-path myharness/Cargo.toml --workspace --lib`, `cargo test --manifest-path myharness/Cargo.toml -p <crate-name> --lib`, `./target/release/myharness --version` smoke. 서버 관리 / 환경 셋업 의 TODO 는 TASK-002 후속 (yklee 인프라 정보 필요).
- **서버 관리** (CONCEPT.md §5.2): `myharness server status|logs|deploy|config` — 원격 서버 호스트 목록 / SSH 별칭 / 헬스체크 명령 — yklee 개인 인프라 정보 필요. 초기값은 본 문서 하단 "## 부록" 섹션에 TODO 로 적는다.
- **환경 셋업** (CONCEPT.md §5.2): `myharness env setup|install|shell|diagnose` — Homebrew 패키지 목록, asdf/rtx 런타임 버전, dotfiles 저장소 경로 — yklee의 macOS 셋업에 맞춰 점진적으로 채운다.

## 4. 검증 포인트 (Validation)
- 워크플로우 변경: `state.json` 재생성 결과 `status: ok`, `MiniMax.md` / `AGENTS.md`(해당 시) / `state.json` 링크 무결성
- 문서 변경: `ai-workflow/core/global_workflow_standard.md` 규약(메타데이터, 한국어 기본, 컨텍스트 절약) 준수
- 하네스 진입점 변경: `MiniMax.md` 가 항상 `state.json` → `session_handoff.md` → `work_backlog.md` → `PROJECT_PROFILE.md` 순서로 안내하는지 확인
- 워커 변경: `.MiniMax/agents/workflow-*.md` 가 `WorkerTask` / `WorkerResponse` 스키마를 따르는지 확인
- 코드 개발: 변경 PR 의 CI 통과, 관련 테스트 실행, 리뷰 코멘트 응답
- 서버 관리: 작업 전 상태 스냅샷, 작업 후 상태 비교 로그, 영향 서비스 헬스체크
- 환경 셋업: idempotent 확인(재실행해도 결과 동일), 설치 직후 smoke test
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
