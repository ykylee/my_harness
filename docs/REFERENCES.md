# Reference Harness Analysis

- 문서 목적: TASK-004 산출물. 5개 오픈소스 코딩 에이전트 하네스를 8축으로 비교 분석해 `my_harness` CLI/TUI 툴의 아키텍처 결정 입력을 만든다.
- 범위: 5개 레퍼런스 × 8축 비교표 + 1-페이지 프로필 + 교차 인사이트 + 우리 방향성 초안
- 대상 독자: yklee, Mavis orchestrator, 다음 디자인 리뷰 참여자
- 상태: draft (1차)
- 최종 수정일: 2026-06-06
- 관련 문서: [PROJECT_PROFILE](../../docs/PROJECT_PROFILE.md), [TASK-005 (my_harness CLI/TUI 전환)](../../ai-workflow/memory/backlog/2026-06-05.md), [harness-refs/.upstream-urls](../../../../harness-refs/.upstream-urls)

## 1. 어떻게 읽을 문서

1. **§2 비교표** 먼저 훑기 — 한 줄 단위 인사이트.
2. **§3 1-페이지 프로필** — 우리 툴과 가장 닮은 1~2개 골라 집중.
3. **§4 교차 인사이트** — 5개 다 답한 추세.
4. **§5 우리 방향성 초안** — TASK-005 시작 전 합의용.

## 2. 8축 비교표

| 축 | opencode | aider | codex | goose | gemini-cli |
| --- | --- | --- | --- | --- | --- |
| **언어 / 빌드** | TS (Bun) + Go SDK | Python (>=3.10) | Rust workspace + Bazel + Nix | Rust workspace (1.91+) | TS (Node 20+) |
| **TUI 라이브러리** | `@opentui/solid` (Solid.js TUI) | rich (Markdown 렌더링만, 풀스크린 TUI 아님) | ratatui + crossterm | ❌ TUI 없음 — cliclack (프롬프트) + clap | ink (React for CLI) |
| **Cross-platform** | macOS/Linux/Win 명시 (`win32.ts` FFI) | Python 어디서나 | Win(mac sandbox) / Linux(bwrap) / macOS(Seatbelt) | Win / Linux / macOS + Electron desktop | Win / Linux / macOS (sandbox: docker/podman/none) |
| **에이전트 토폴로지** | **Client-Server** (TUI는 worker, server와 RPC) | Monolithic single-loop | Workspace crates, app-server + TUI 분리 | CLI + server (goosed) + Electron desktop | CLI (ink) + core 분리, A2A-server 별도 |
| **컨텍스트 관리** | journal.json + Effect schema, OpenTUI attach/server | `repo.py` git index + `repomap.py` graph | `codex-message-history` crate, **10K 토큰/item 캡**, **"no history rewrite"** 규칙 | `keyring` 크레이트로 시크릿, `chatrecall` 컨텍스트 | settings.json + Gemini API context, `context/` 모듈 |
| **세션 / 영속화** | session schema (SessionID), attach/validate | `history.py` (token-based summarization), 디스크 저장 | rollout 형식, resume 지원 | sqlite + chatrecall | session JSON, memory tests, perf tests |
| **확장 포인트** | **풍부**: plugin/ theme/ command/ tool/ skill/ agent/ | 없음 (closed) | `core-plugins/`, `core-skills/`, `ext/extension-api/`, hooks | `goose-mcp` (rmcp 1.4) | **MCP 1급** + Hooks (Registry/Runner/Aggregator/Planner/EventHandler/Translator) + A2A |
| **워크플로우 호환성** | `AGENTS.md` 보유 (code convention 중심) | 없음 | `AGENTS.md` 보유 (**매우 상세**, "model visible context" 규칙 6개, 800줄 변경 가이드) | `AGENTS.md` 보유 (DCO sign-off, 빌드 명령) | `GEMINI.md` 보유 (전체 프로젝트 컨텍스트) |

## 3. 1-페이지 프로필

### 3.1 opencode (sst/opencode, Go+TS, dev 브랜치)

- **라이선스**: MIT
- **철학**: 클라이언트-서버 분리, TUI는 thin worker. 한 서버에 여러 TUI 동시 attach 가능 (`attach <url>` 명령).
- **TUI 핵심**: `@opentui/solid` — Solid.js reactivity + 커스텀 렌더러. TUI 자체가 npm 패키지(`@opentui/core`, `@opentui/keymap`)로 분리 → **TUI 라이브러리 자체가 재사용 가능**.
- **확장 모델**: `.opencode/` 한 디렉토리에 `plugin/`, `command/`, `theme/`, `tool/`, `skill/`, `agent/`, `glossary/` 모두 JSON으로 정의. 워크플로우 표준화에 가까운 형태.
- **우리한테 시사점**: TUI/CLI 와 무관하게 **세션/도구/플러그인을 JSON 스키마로 선언**하는 패턴이 우리 `MiniMax.md` 기반 운영 정책과 잘 맞음. standard_ai_workflow 의 skills/ 와 직접 매핑 가능.
- **취약점**: dev 브랜치 default — 안정성 트랙 vs 기능 트랙 분리. 1인/소규모에 적합한지 의문.

### 3.2 aider (Aider-AI/aider, Python)

- **라이선스**: Apache 2.0
- **철학**: 가장 단순한 구조. `aider` 단일 entry point, Python 단일 프로세스. 채팅-with-코드베이스에 집중.
- **TUI 핵심**: rich 라이브러리로 markdown/diff 렌더링. 풀스크린 TUI가 아니라 REPL-style. 사용자가 터미널 자연스럽게 사용.
- **컨텍스트 관리의 교과서**: `aider/repo.py` 가 git index 를 직접 색인, `repomap.py` 가 그래프 기반 토큰 최적화. `history.py` 의 token-based summarization (max_tokens=1024).
- **우리한테 시사점**: 1) 풀스크린 TUI가 꼭 필요한 건 아니다 — REPL/프롬프트 + 좋은 출력 렌더링이면 충분. 2) 토큰 예산/요약은 모든 레퍼런스 공통 관심사.
- **취약점**: 확장성 없음 (closed), MCP/plugin 미지원, 멀티 에이전트 없음.

### 3.3 codex (openai/codex, Rust+TS)

- **라이선스**: Apache 2.0
- **철학**: OpenAI 의 "production coding agent". 명확한 모듈 경계, 무수한 작은 crate.
- **TUI 핵심**: ratatui + crossterm (Rust TUI 표준). `codex-tui` crate 가 TUI 책임지고, `codex-core` 는 agent 로직만.
- **워크플로우 표준의 모범**: `codex/AGENTS.md` 200+ 줄 — "model visible context" 6개 규칙(10K 토큰 캡, 1K 토큰 경고 등), 800줄 변경 가이드, sandbox 변수 규칙까지. **이건 우리 `MiniMax.md` 의 모범 답안**.
- **확장**: `core-plugins/`, `core-skills/`, `ext/extension-api/` — extensions 와 skills 가 분리되어 있음.
- **우리한테 시사점**: 1) `core` 가 비대해지지 않게 별도 crate 로 빼는 규율 (OpenCode 도 비슷한 효과). 2) sandbox 추상화 (`shell-escalation` crate) 가 Linux/macOS/Windows 별로 잘 되어 있음 — 우리 `bwrap` / `Seatbelt` / WindowsJob 패턴 참고.
- **취약점**: 너무 큰 workspace — 진입장벽 높음, 읽기 부담.

### 3.4 goose (block/goose → aaif.io, Rust+Python)

- **라이선스**: Apache 2.0
- **철학**: **멀티 인터페이스** (CLI + Electron desktop + server `goosed`). TUI 없음 — 프롬프트 기반 CLI.
- **TUI 핵심**: 없음. `cliclack` 으로 interactive prompts, `console` crate 로 터미널 출력. 풀스크린 TUI의 대안.
- **시크릿 관리의 모범**: `keyring` crate (3.6.3, vendored) 로 OS 키체인 직접 사용 — GitHub/GitLab/OpenAI 토큰 등. 우리도 똑같이 해야 함.
- **MCP 1급**: `goose-mcp` crate + `rmcp` 1.4. **provider 등록 → 도구 자동 노출** 패턴.
- **우리한테 시사점**: 1) 모든 기능을 단일 TUI에 우겨넣지 말고, **CLI = core logic + 프롬프트 UX, server = HTTP API, desktop = Electron** 분리도 옵션. 2) 시크릿은 무조건 OS keychain (macOS Keychain / Windows Credential Manager / Linux Secret Service).
- **취약점**: TUI 없음 — 우리가 원하던 것과 거리.

### 3.5 gemini-cli (google-gemini/gemini-cli, TS/Node)

- **라이선스**: Apache 2.0
- **철학**: **확장성의 모범**. CLI 본체 + A2A server + VS Code companion + SDK + devtools.
- **TUI 핵심**: **ink (React for CLIs)**. 가장 큰 생태계, React 개발자에게 진입장벽 낮음.
- **Hooks 시스템의 교과서**: `hookRegistry` → `hookPlanner` → `hookEventHandler` → `hookRunner` → `hookAggregator` → `hookTranslator` — 이벤트 → 등록 → 평가 → 실행 흐름이 매우 정교. 우리 `MiniMax.md` 의 워커 토폴로지와 직접 매핑 가능.
- **Sandbox 추상화**: `docker` / `podman` / `none` 3가지 backend 를 같은 인터페이스로 — 우리 3-도메인(코드/서버/환경) 작업 시 동일 패턴.
- **A2A (Agent-to-Agent)**: 별도 패키지로 server-side 에이전트 — 멀티 에이전트 미래의 힌트.
- **우리한테 시사점**: 1) **hook 시스템은 우리 오버레이의 핵심**. Worker delegation 의 trigger 조건을 hooks 로 표현 가능. 2) MCP 1급 + 공식 OAuth 지원 — 우리도 최소 MCP 호환 + 시크릿은 keychain. 3) `settings.schema.json` 처럼 설정은 JSON Schema 로 검증.
- **취약점**: TypeScript/Node 의 single binary 어려움 (Node 번들링 vs SEA vs 외부 의존). `build:binary` 명령 있지만 복잡.

## 4. 교차 인사이트 (5개 다 답한 추세)

### 4.1 TUI 선택 = 사실상 3개 진영

- **Rust 진영**: `ratatui + crossterm` (codex)
- **TypeScript 진영**: `ink` (gemini-cli) **또는** `@opentui/solid` (opencode — 직접 만듦)
- **Python 진영**: `rich` / `textual` (aider 는 rich 만)

→ 우리 선택지는 **Rust+ratatui** vs **TypeScript+ink** vs **TypeScript+@opentui** 셋. Python 은 cross-platform 패키징 어려움으로 사실상 제외.

### 4.2 확장성 = 4가지 패턴

- **JSON-declared config** (opencode): 파일 기반 선언, 가벼움
- **Plugin/Skill 분리** (codex): 두 레이어로 나눠 의도 명확화
- **MCP 1급** (goose, gemini-cli): 외부 도구 표준 인터페이스
- **Hooks** (gemini-cli): 이벤트 기반 자동화

→ 우리는 **MiniMax.md 운영 정책 + JSON-declared skills/agents (opencode 스타일) + MCP 호환 + hooks (gemini-cli 스타일)** 의 조합이 가장 부합. standard_ai_workflow 의 skills/ 메타 레이어와 자연스럽게 정렬.

### 4.3 시크릿 관리 = 전원 keychain

- goose: `keyring` crate (vendored)
- gemini-cli: `oauth-token-storage` (keychain)
- codex: `secrets` crate
- opencode: server-side auth (likely env-based)
- aider: 환경변수 위주

→ 우리도 **OS keychain 1급** + `MavisConfig` 같은 추상화 레이어. mavis 가 keychain 자격증명도 관리 가능.

### 4.4 컨텍스트 = 모두 토큰 예산 + 요약 + 색인

- codex: 10K 토큰/item 캡, "no history rewrite"
- aider: max_tokens=1024 summarization
- gemini-cli: memory tests, perf tests
- opencode: journal.json + Effect schema
- goose: chatrecall

→ 우리도 **컨텍스트 절약이 1차 설계 목표**. standard_ai_workflow 의 1.2 "컨텍스트 절약 원칙" 이 이미 우리 표준.

### 4.5 워크플로우 표준 = AGENTS.md 가 de facto

- opencode, codex, goose = `AGENTS.md` 보유
- gemini-cli = `GEMINI.md` (같은 역할, 벤더별 이름)
- aider = 없음 (하지만 Pythonic 한 docstring 으로 대체)

→ 우리 `MiniMax.md` + `AGENTS.md` 조합은 **이미 산업 표준 패턴**. TASK-005 에서 `MiniMax.md` 의 내용을 더 풍성하게 채우면 됨.

## 5. 우리 방향성 초안 (TASK-005 시작 전 합의용)

### 5.1 추천 스택 (1안: Rust)

- **언어**: Rust
- **TUI**: `ratatui + crossterm` (codex 검증)
- **빌드**: cargo + `cargo-dist` (cross-platform 빌드/릴리스)
- **MCP**: `rmcp` SDK (goose 와 같은 crate)
- **시크릿**: `keyring` crate (OS keychain)
- **워크플로우**: `MiniMax.md` + `AGENTS.md` + `ai-workflow/core/` 그대로 흡수

**장점**: 단일 바이너리 배포 용이, codex 와 같은 검증된 TUI 스택, keychain/시크릿/MCP 생태계 Rust 가 더 성숙.

### 5.2 추천 스택 (2안: TypeScript)

- **언어**: TypeScript
- **TUI**: `ink` (gemini-cli 검증) 또는 `@opentui/solid` (opencode, 더 정교)
- **빌드**: esbuild + `sea` (Node SEA, 단일 바이너리) 또는 Bun 번들
- **MCP**: 공식 `@modelcontextprotocol/sdk` (TS 가 표준)
- **시크릿**: `keytar` (deprecated) 또는 OS keychain 직접 호출
- **워크플로우**: standard_ai_workflow 그대로 흡수 (TS 환경 친화적)

**장점**: gemini-cli 의 풍부한 hooks 시스템, MCP SDK 표준, 개발 속도 빠름.
**단점**: 단일 바이너리 어려움 (Node 런타임 의존), `keytar` deprecated.

### 5.3 차이점 요약

| 결정 | Rust 1안 | TS 2안 |
| --- | --- | --- |
| TUI 진영 | ratatui/crossterm (검증) | ink (생태계) / @opentui (혁신) |
| 단일 바이너리 | cargo-dist 쉬움 | Node SEA/Bun 가능하나 까다로움 |
| MCP SDK | rmcp 1.4 성숙 | @modelcontextprotocol/sdk 표준 |
| 시크릿 | keyring 안정 | keytar deprecated 이슈 |
| 진입장벽 | 중 (Rust 학습) | 낮음 (TS/JS 친숙) |
| 우리 `MiniMax.md` 흡수 | 추가 작업 필요 | JSON/TS 친화적 |

### 5.4 미결

- **언어 선택**: 1안 (Rust) vs 2안 (TypeScript) — TASK-005 시작 전 너 결정.
- **이름**: 현재 `my_harness` 임시. CLI/TUI 정식 명칭 미정.
- **핵심 컨셉 한 줄**: "what is it" 을 한 문장으로 — 아직 미정. (다음 세션 TASK-005 의 첫 단계)

## 6. 다음 단계

- TASK-004 (이 문서) — **draft 상태로 끝**. 너 리뷰 후 1~2개 골라 deep-dive 가능.
- TASK-005 — 본 문서 §5 의 추천 중 너 선택 후 분해. 첫 단계는 **스택 결정 → MVP 범위 → 컨셉 한 줄**.
- 미해결: Gitea PAT 발급 (1번 옵션 진행 중단) — 너가 web UI 에서 발급 후 토큰 값 알려주면 keychain 셋업 마무리.

## 7. 부록: 원본 위치

- `/Users/yklee/repos/harness-refs/opencode/` → `AGENTS.md`, `packages/opencode/src/cli/cmd/tui/`
- `/Users/yklee/repos/harness-refs/aider/` → `aider/io.py`, `aider/repo.py`, `aider/history.py`
- `/Users/yklee/repos/harness-refs/codex/` → `AGENTS.md`, `codex-rs/tui/Cargo.toml`, `codex-rs/message-history/`
- `/Users/yklee/repos/harness-refs/goose/` → `AGENTS.md`, `crates/goose-cli/`, `crates/goose-mcp/`
- `/Users/yklee/repos/harness-refs/gemini-cli/` → `GEMINI.md`, `packages/core/src/hooks/`, `packages/a2a-server/`
