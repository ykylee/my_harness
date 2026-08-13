# Gemini CLI (google-gemini/gemini-cli) — 심층 분석

- 문서 목적: `google-gemini/gemini-cli` 레퍼런스의 실제 코드를 14섹션 표준 템플릿으로 분석. 1차 분석 `docs/REFERENCES.md` §3.5 의 1-페이지를 깊이 10배로 확장.
- 범위: gemini-cli 전체 (TypeScript monorepo, ink, hooks, MCP, A2A)
- 대상 독자: yklee, Mavis, TASK-005 디자인 리뷰 참여자
- 상태: final (1차 draft, 2차 확장)
- 최종 수정일: 2026-06-07
- 관련 문서: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [REFERENCES.md §3.5](../REFERENCES.md)

## §1 개요 (Overview)

- **프로젝트명**: `@google/gemini-cli`
- **라이선스**: Apache 2.0
- **언어**: TypeScript (Node 20+)
- **메인 binary**: `gemini` (`packages/cli`)
- **버전**: `0.47.0-nightly.20260602` (시점 기준)
- **코드 규모 (추정)**: monorepo, 7+ packages (cli, core, a2a-server, sdk, devtools, test-utils, vscode-ide-companion)
- **타겟 사용자**: Gemini API 사용자, terminal-first 개발자, MCP 호환 필요한 tool builder
- **1줄 설명**: "Google 의 공식 terminal AI 에이전트. ink + React, 7-component hook 시스템, MCP 1급, A2A-server 별도 패키지"

## §2 아키텍처 (Architecture)

### 2.1 프로세스 모델: 모노레포 + npm workspaces

gemini-cli 는 **명확한 모노레포 구조**. 단일 프로세스 (TUI = main process), 여러 packages 가 `npm workspaces` 로 연결.

```
gemini-cli/
├── packages/
│   ├── cli/            # 사용자 대면 terminal UI (ink, React)
│   ├── core/           # 백엔드 로직, Gemini API, prompt construction, tool execution
│   ├── a2a-server/     # 실험적 Agent-to-Agent server
│   ├── sdk/            # 프로그래매틱 SDK (다른 앱에 임베드)
│   ├── devtools/       # 통합 developer tools (Network/Console inspector)
│   ├── test-utils/     # 공유 test utilities
│   └── vscode-ide-companion/  # VS Code extension (CLI 와 pair)
├── schemas/            # JSON Schema (settings.schema.json)
├── integration-tests/  # E2E (sandbox: none/docker/podman)
├── evals/              # LLM 평가
└── memory-tests/       # 메모리 회귀 테스트
```

### 2.2 핵심 모듈 (packages/core/src/)

```
core/src/
├── agent/            # agent 추상화
├── agents/           # 구체적 agent 구현 (재귀 에이전트, 서브에이전트)
├── commands/         # slash command
├── config/           # 설정 로딩 / 검증
├── confirmation-bus/ # 사용자 컨펌 흐름
├── context/          # 컨텍스트 빌드 (model 입력용)
├── core/             # 메인 entry, tools 등록
├── hooks/            # 7-모듈 hook 시스템 ⭐
├── ide/              # VS Code / JetBrains IDE 통합
├── mcp/              # MCP + OAuth ⭐
├── output/           # 출력 포맷팅
├── policy/           # 정책 (allowlist, blocklist)
├── agents/           # sub-agent
├── tools/            # tool registry
├── fallback/         # 모델 fallback (rate limit 등)
└── availability/     # 가용성 체크
```

### 2.3 핵심 추상화

- **Agent**: 1개의 root agent + 다중 sub-agent (sub-agent 위임)
- **Tool**: 표준 tool (`read_file`, `write_file`, `bash_run` 등) + MCP servers
- **Hook**: 7-모듈 이벤트 시스템 (Registry → Planner → Handler → Runner → Aggregator → Translator)
- **Confirmation Bus**: 위험 명령 사용자 컨펌 흐름
- **Context**: model 입력 컨텍스트 빌드
- **Policy**: 도구/명령 allowlist
- **Settings**: JSON Schema 검증된 설정

### 2.4 데이터 흐름

```
[User Input (ink)]
   ↓
[CLI: gemini.tsx] → render
   ↓ (input)
[Confirmation Bus: 위험 체크]
   ↓
[Agent: 도구 호출 결정]
   ↓
[Tool Registry / MCP Client]
   ↓
[Tool execution] → result
   ↓
[Context: 메시지 추가]
   ↓
[Gemini API 호출]
   ↓ (streaming)
[CLI: ink render]
   ↓
[User Output]
```

## §3 진입점 & CLI

### 3.1 메인 진입점

`packages/cli/src/gemini.tsx` — ink 기반 React 컴포넌트가 main entry.

### 3.2 CLI 명령 트리 (추정)

```
gemini
├── (default)        # 인터랙티브 TUI
├── --model <name>   # 모델 선택 (gemini-2.5-pro, gemini-2.5-flash, ...)
├── --sandbox <type> # sandbox (none/docker/podman)
├── --mcp <config>   # MCP server 추가
├── --policy <file>  # 정책 파일
└── --help
```

### 3.3 인자 파싱

`commander` (likely) 또는 `yargs`. POSIX 스타일.

### 3.4 package.json scripts

```json
{
  "start": "cross-env NODE_ENV=development node scripts/start.js",
  "start:prod": "cross-env NODE_ENV=production node scripts/start.js",
  "build:binary": "node scripts/build_binary.js",
  "test:integration:sandbox:none": "...",
  "test:integration:sandbox:docker": "...",
  "test:integration:sandbox:podman": "..."
}
```

`build:binary` — Node SEA (Single Executable Application) 기반 단일 binary 빌드.

## §4 TUI/UI 구현

### 4.1 TUI 라이브러리: ink (React for CLIs)

`ink` (vadimdemedes/ink) — React for CLIs. gemini-cli 는 **forked version** 사용:
```json
// package.json
"ink": "npm:@jrichman/ink@6.6.9"
```

forked 이유 추정:
- **보안 패치 / CVE 대응** 빠른 적용
- **Google 사내 수정** (e.g., Gemini API 통합, OAuth, telemetry)
- **업스트림 머지 부담** 회피

### 4.2 React 19 / Concurrent rendering

ink 6.x 는 React 19 기반. Concurrent rendering 활용. 큰 terminal grid 에서 부드러운 업데이트.

### 4.3 컴포넌트 구조

```
packages/cli/src/
├── gemini.tsx        # root 컴포넌트
├── interactiveCli.tsx
├── nonInteractiveCli.tsx
├── nonInteractiveCliAgentSession.ts
├── services/         # 클라이언트 측 서비스
├── output-redirection.test.ts
└── ...
```

### 4.4 ink 의 한계 + 대응

ink 의 한계 (React/HTML 기반, 텍스트 중심):
- 이미지: `<Image>` (Sixel / iTerm protocol)
- Syntax highlight: `ink-syntax-highlight` 또는 manual ANSI
- 24-bit color: 지원

### 4.5 Rendering 최적화

- `useMemo` / `useCallback` — React re-render 최소화
- Streaming output — `useStream` hook 패턴
- Virtualized list — 긴 로그/결과 처리

## §5 LLM 통합

### 5.1 Gemini API 추상화

`packages/core/src/code_assist/` — Gemini Code Assist API (Vertex AI 포함). OAuth 기반.

### 5.2 모델 라우팅

`packages/core/src/agents/` — model routing (gemini-2.5-pro, gemini-2.5-flash, gemini-1.5-pro 등). 자동 fallback (rate limit, model unavailability).

### 5.3 Streaming

Server-Sent Events (SSE) 또는 chunked response. ink 의 streaming pattern 으로 실시간 출력.

### 5.4 Function Calling

OpenAI-compatible function calling format. Tool schema (JSON Schema) 자동 변환.

### 5.5 Token 추적 + Quota

`packages/core/src/availability/` — quota check, rate limit handling. 자동 fallback.

## §6 도구/스킬 시스템

### 6.1 내장 도구

`packages/core/src/tools/` — 표준 도구 (read_file, write_file, bash_run, list_files, search_files 등).

### 6.2 도구 등록

JSON Schema 기반. tool name, description, parameters schema, handler.

### 6.3 Tool Policy

`packages/core/src/policy/` — allowlist / blocklist. `--policy <file>` CLI 옵션으로 외부 정책 로드.

### 6.4 Confirmation Bus

`packages/core/src/confirmation-bus/` — 위험 도구 (`bash_run` 등) 실행 전 사용자 컨펌. **inline 프롬프트** (TUI 안에서 yes/no).

## §7 컨텍스트 관리

### 7.1 컨텍스트 빌드

`packages/core/src/context/` — model 입력용 컨텍스트. 시스템 프롬프트 + 도구 정의 + 대화 이력 + 현재 입력.

### 7.2 토큰 예산

모델 context window (1M tokens for gemini-2.5-pro) 기반 자동 추정. Compaction:

### 7.3 메모리 (실험)

`memory-tests/` 디렉토리 + nightly test. **메모리 회귀** 자동 감지.

### 7.4 File 읽기 전략

- 페이지 단위 chunked read
- AST-aware 검색 (의심, 미확인)

## §8 세션 영속화

### 8.1 Storage

`packages/core/src/session/` (추정) — 세션 metadata. **JSON Lines** 형식 추정.

### 8.2 Resume

CLI 재시작 시 `--continue` 또는 `--session <id>` (의심, 미확인).

### 8.3 Settings 영속화

`~/.gemini/settings.json` — JSON Schema 검증. **policy 경로, MCP server, model 등** override.

## §9 확장 시스템 ⭐

### 9.1 Hook 시스템 (7-모듈)

`packages/core/src/hooks/` — 가장 진보된 hook 시스템:

| 모듈 | 역할 |
| --- | --- |
| `hookRegistry.ts` | hook 등록/관리 |
| `hookPlanner.ts` | 어떤 hook 실행할지 계획 |
| `hookEventHandler.ts` | 이벤트 → hook 매칭 |
| `hookRunner.ts` | hook 실제 실행 |
| `hookAggregator.ts` | 다중 hook 결과 집계 |
| `hookTranslator.ts` | hook 결과를 다음 단계로 변환 |
| `trustedHooks.ts` | trusted vs untrusted hook 분리 |

**이벤트 흐름**:
```
Event 발생
  → hookEventHandler: 매칭되는 hook 조회
  → hookPlanner: 실행 순서 / 우선순위 결정
  → hookRunner: 각 hook 실행
  → hookAggregator: 결과 병합
  → hookTranslator: 다음 단계 (block / modify / pass-through)
```

### 9.2 MCP 1급 + OAuth

`packages/core/src/mcp/`:

| 파일 | 역할 |
| --- | --- |
| `oauth-provider.ts` | OAuth 인증 흐름 |
| `oauth-token-storage.ts` | 토큰 저장 (keychain? 추정) |
| `oauth-utils.ts` | OAuth 유틸 |
| `stored-token-provider.ts` | 저장된 토큰 사용 |
| `mcp-oauth-provider.ts` | MCP-specific OAuth |
| `sa-impersonation-provider.ts` | Service Account impersonation (Google Cloud) |
| `google-auth-provider.ts` | Google Auth (ADC) |
| `token-storage/` | 토큰 저장 모듈 |

**OAuth 표준 지원** — Gemini API 인증 (Google Auth), MCP servers 인증 모두.

### 9.3 A2A-server (Agent-to-Agent)

`packages/a2a-server/` — **별도 패키지**로 분리. A2A (Agent-to-Agent) protocol 구현:
- HTTP server (`CODER_AGENT_PORT=41242`)
- 다른 agent 와 통신 (standard protocol)
- SDK 제공

### 9.4 VS Code IDE 통합

`packages/vscode-ide-companion/` — VS Code extension. **CLI 와 pair** 되어 동작 (e.g., IDE 에서 선택한 코드를 CLI context 로).

### 9.5 SDK

`packages/sdk/` — 다른 app 에 임베드하기 위한 API. `import { Gemini } from '@google/gemini-sdk'`.

### 9.6 DevTools

`packages/devtools/` — 통합 developer tools (Network/Console inspector). 디버깅 UI.

## §10 빌드 & 배포

### 10.1 빌드 시스템

- **esbuild** — TypeScript 번들러
- **npm workspaces** — 모노레포
- **Node SEA (Single Executable Application)** — `node --experimental-sea-config` 로 단일 binary
- `build:binary` script — SEA 기반 binary 빌드

### 10.2 단일 바이너리

`build:binary`:
- `node scripts/build_binary.js`
- `sea-config.json` 으로 entry + assets 지정
- `node --experimental-sea-config` → `node --build-sea`
- 결과: 단일 실행 파일 (~50MB Node runtime 포함)

### 10.3 Cross-platform

Node SEA 는 **3 OS** (Linux / macOS / Windows) + arch (x64 / arm64) 지원. **CI matrix** 6+ 조합.

### 10.4 Sandbox

```json
"test:integration:sandbox:none":   "...",
"test:integration:sandbox:docker": "...",
"test:integration:sandbox:podman": "..."
```

3개 sandbox backend:
- **none** — 호스트 직접 실행
- **docker** — Docker 컨테이너
- **podman** — Podman 컨테이너

각 sandbox 는 `GEMINI_SANDBOX` 환경변수로 선택.

### 10.5 Distribution

- **npm**: `npx @google/gemini-cli`
- **GitHub Releases**: platform 별 binary
- **Homebrew / apt** (의심, 미확인)

## §11 테스트 & 품질

### 11.1 테스트 구조

```
gemini-cli/
├── integration-tests/   # E2E (sandbox 별)
├── evals/                # LLM 평가
├── memory-tests/         # 메모리 회귀
├── perf-tests/           # 성능 회귀
└── packages/*/test/      # per-package unit
```

### 11.2 테스트 도구

- **Vitest** — `vitest run` (primary)
- **TypeScript** — `tsc --noEmit`
- **ESLint** — `eslint . --max-warnings 0`
- **Prettier** — formatting

### 11.3 Pre-flight

```bash
npm run preflight
```

`clean + install + build + lint + type check + test` 통합. **PR 제출 전 필수**. (다만 시간 오래 걸림 — 간단 변경은 skip)

### 11.4 Eval

`evals/vitest.config.ts` — `RUN_EVALS=1` 환경변수로 enable. LLM 평가 (실제 task 수행 능력).

### 11.5 Memory test (nightly)

`memory-tests/` — `UPDATE_MEMORY_BASELINES=true` 로 베이스라인 갱신. **메모리 사용량** 자동 검사.

### 11.6 Performance test (nightly)

`perf-tests/` — CPU 성능 회귀. **응답 시간** 자동 검사.

## §12 보안

### 12.1 OAuth 토큰 저장

`packages/core/src/mcp/oauth-token-storage.ts` — 토큰 저장. **keychain 추정** (macOS Keychain, Windows Credential Manager) + file fallback. 정확한 구현은 코드 추가 검증 필요.

### 12.2 Sandbox 추상화

3개 backend (none/docker/podman). **podman 선택** 가능 (Docker daemon 없이 rootless 컨테이너).

### 12.3 정책 시스템

`packages/core/src/policy/` — allowlist / blocklist. 사용자 정의 정책 파일 (YAML/JSON).

### 12.4 Confirmation Bus

위험 도구 실행 전 inline 사용자 컨펌. 자동 yes/no.

### 12.5 Audit log

- 세션 JSON (메시지 + tool call)
- Settings (모든 변경 추적)
- CLI `--verbose` 로그

## §13 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 Hook 시스템 (7-모듈) ⭐

Registry → Planner → Handler → Runner → Aggregator → Translator — typed event + multi-hook aggregation. **우리 my_harness 의 오버레이 worker 토폴로지** 와 직접 매핑. **MiniMax.md 의 워커 명령 = hook 으로 표현 가능**. 우리 TASK-005 시 hook 시스템 도입 강력 권장.

#### 13.2 MCP 1급 + OAuth

`oauth-provider.ts`, `oauth-token-storage.ts` — **OAuth 표준 통합** + MCP 1급. 우리 my_harness 도 MCP host + keychain 시크릿 + OAuth (필요 시).

#### 13.3 Settings JSON Schema

`schemas/settings.schema.json` — **JSON Schema** 로 설정 검증. IDE 자동완성 + 검증. 우리 my_harness 도 `MiniMax.md` 의 `~/.myharness/config.yaml` 에 적용.

#### 13.4 Sandbox 추상화 (3 backend)

`docker` / `podman` / `none` 3개 backend. **podman** 포함이 차별점 (rootless, 데몬 불필요). 우리 my_harness 도 **3 backend** (Seatbelt / bwrap / Windows Job) 검토.

#### 13.5 Confirmation Bus

`packages/core/src/confirmation-bus/` — **inline 컨펌 흐름** (TUI 안에서 yes/no). bash 위험 명령 실행 전 필수. 우리 my_harness 의 정책.

#### 13.6 npm workspaces + packages 분리

`cli`, `core`, `sdk`, `a2a-server`, `vscode-ide-companion` 등 — **명확한 책임 분리**. 우리 my_harness 도 `packages/cli`, `packages/core` 등 분리 검토.

#### 13.7 A2A (Agent-to-Agent) protocol

`packages/a2a-server/` — **Agent 간 통신 표준**. 멀티에이전트 미래의 힌트. 우리 v2+ 검토.

#### 13.8 VS Code IDE companion

`packages/vscode-ide-companion/` — IDE 와 pair. **context 공유** (선택 영역 → CLI). 우리 v2+ 검토.

#### 13.9 Memory + Performance 회귀 테스트

`memory-tests/`, `perf-tests/` — nightly CI. **장기 품질 보증**. 우리도 동일.

#### 13.10 Sandbox 통합 테스트 (3 backend)

`test:integration:sandbox:{none,docker,podman}` — **각 backend 별 E2E**. 우리도 `seatbelt/bwrap/job` 별 테스트.

#### 13.11 Pre-flight (clean → test) 통합 명령

`npm run preflight` — PR 전 필수. 우리도 `just preflight` (Rust) 또는 `pnpm preflight` (TS) 도입.

#### 13.12 `integration-tests/` 분리 (sandbox 별)

테스트를 production code 와 분리. **E2E 는 실제 binary invoke**. 우리도 동일 패턴.

#### 13.13 `evals/` (LLM 평가)

실제 task 수행 능력 측정. **PR 의 LLM 성능 gate**. 우리 v2+ 검토.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.14 nightly test 의 CI 부담

memory / perf test 가 nightly 에만. **PR gate 없음** — main 브랜치 회귀 늦게 발견. 우리 my_harness 는 PR gate 부터 (nightly 아님).

#### 13.15 npm workspaces + monorepo 의 의존성 그래프 복잡도

7+ packages + workspace 의존성. **dependency graph 시각화 + lockfile 관리** 부담. 우리 my_harness v1 은 단일 package 부터.

#### 13.16 SEA (Single Executable Application) 의 startup time

Node 기반 binary 는 **Node startup + JS init** 으로 Rust binary 대비 느림. **2-5x slower startup**. 우리 my_harness 가 TS 2안 + Node SEA 면 cold start 시간 주의.

#### 13.17 SEA binary 의 asset embedding 한계

SEA 는 asset (이미지 등) embedding 이 가능하지만 **크기 제한** + pack/unpack 복잡. 우리 my_harness 가 TS 2안 + 이미지 asset 시 cargo 의 `include_bytes!` 가 더 단순.

#### 13.18 A2A protocol 의 미성숙

`a2a-server/` 가 **실험적** (README 에 "Experimental"). **spec 미안정**. 우리 v1 에서 채택 시 리스크.

#### 13.19 Hook 시스템 7-모듈 의 복잡도

7 모듈 (Registry/Planner/Handler/Runner/Aggregator/Translator) — **v1 에선 과함**. 우리 my_harness 는 **3-4 모듈** (Registry/Handler/Runner/Aggregator) 로 시작, 필요 시 확장.

#### 13.20 `default: nightly` (PR gate 아닌 nightly)

`memory-tests` 와 `perf-tests` 가 `preflight` 에서 제외. **main 브랜치 회귀 늦게 발견**. 우리도 동일 함정 주의 — v1 부터 PR gate.

#### 13.21 IDE companion (VS Code) 의 마켓 부담

VS Code extension 별도 패키지 — **Marketplace 게시 + 유지보수** 부담. 우리 my_harness 는 CLI only 시작.

#### 13.22 podman support 의 Linux 한정

podman 은 Linux 전용 (macOS 에선 lima 등 우회 필요). **3 backend** 라고 하지만 실은 **Linux = 2개, macOS = 1개 (docker desktop), Windows = 1개 (docker desktop)**.

#### 13.23 OAuth 흐름의 사용자 friction

OAuth 매번 인증 시 redirect 흐름. **MCP server** 마다 OAuth 다르면 사용자에게 부담. 우리 my_harness 가 OAuth 도입 시 **token cache + 만료 전 갱신** 고려.

#### 13.24 `dev/nightly` 버전 (`0.47.0-nightly.20260602`)

nightly 빌드를 npm 에 배포. **사용자가 stable / nightly 명시 안 하면 nightly 받음**. 우리 v1 release 시 stable 만.

#### 13.25 `references/protocol_*.md` 분산

스킬 / 도구 / hook / MCP 등 문서가 각자 위치에 분산. **단일 진입점 (MiniMax.md 같은)** 없음. 우리 my_harness 는 `MiniMax.md` 가 모든 운영 정책의 단일 source.

## §14 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 `oauth-token-storage.ts` 의 실제 저장소

OS keychain 사용? encrypted file? 환경변수? 정확한 mechanism 미확인. 우리 my_harness 가 MCP OAuth 도입 시 직접 참고 필요.

### 14.2 Hook 시스템의 실제 트리거 이벤트

7-모듈 hook 이 **어떤 이벤트** 들에 trigger 되는지? `session.start` / `tool.before` 등 추정. **전체 리스트 + 스키마** 확인 필요.

### 14.3 A2A protocol 의 표준화 상태

A2A 가 표준화 진행 중인지? Google 만? 다른 vendor 도 채택? 우리 채택 결정 시.

### 14.4 IDE companion 의 context 공유 메커니즘

VS Code 에서 선택한 코드를 CLI context 로 어떻게 전달? stdin? IPC? WebSocket?

### 14.5 `dev/nightly` 의 stable 전환 시점

v1.0 시점에 `main` stable? 별도 `stable` branch? roadmap 미확인.

### 14.6 Pre-flight 의 CI 시간

`preflight` 가 가장 무거운 check (clean + install + build + lint + type + test). **CI 시간 영향** 큼. 우리도 동일.

### 14.7 Sandbox 3 backend 의 fallback 정책

docker 없으면 → podman? → none? 자동 fallback 인지 사용자 명시 인지?

### 14.8 `packages/vscode-ide-companion/` 의 사용자 비중

VS Code 사용자 vs CLI only 사용자의 비율. 우리 TASK-005 의 IDE 통합 결정.

### 14.9 Sandbox 컨테이너 안의 LLM 호출

Docker 안에서 LLM API 호출 시 네트워크 egress. **docker network 옵션** 처리.

### 14.10 `evals/` 의 LLM 평가 방법론

실제 task suite? 사람 작성? 자동 생성? **v1 의 우리 my_harness 가 eval 도입 시** 직접 참고.

### 14.11 npm registry 의 publish 정책

Google 사내 패키지? 사외 publish? **공식 npm package** 가 `0.47.0-nightly` 인 게 의외. stable release cadence.

### 14.12 MCP servers 의 OAuth 표준화

Anthropic / OpenAI MCP 사양과 정합? Google 확장? 우리 my_harness 가 MCP host 시 호환성.

### 14.13 설정 파일의 schema versioning

`settings.schema.json` 이 변경 시 user 의 `settings.json` 자동 마이그레이션? 미확인. 우리도 schema versioning + 자동 migration 필요.

### 14.14 컨텍스트 윈도우 1M 토큰의 실제 사용

Gemini 2.5 Pro 의 1M token context — **모두 활용** vs **truncate** vs **partial read**? gemini-cli 의 실제 정책.

---

## §15 v2 갱신 — 06-09 이후 119 commit 종합 (TASK-004 재방문, D-132)

> **갱신 사유**: 기존 v1 분석 (2026-06-07, gemini-cli v0.45.0-nightly.20260602) 이후 upstream `main` 브랜치에 **130 commit** (v0.45.0 → v0.55.1, 10 minor) 반영. TASK-004 1차 재방문 (D-132, 2026-08-14). 본 섹션은 v1 의 14섹션 (586 lines) 을 **append-only** 로 보존하고, §15 (v2 changelog) + §16 (v2 영향 분석) 만 추가.
>
> **시점**: 분석 시점 upstream = `v0.55.1` (2026-08-11 release commit `41327e407`). 우리 v1 분석 시점 = v0.45.0-nightly.20260602. 차이 = 10 minor (v0.45.0 → v0.55.1).
>
> **선정 기준**: 130 commit 전체를 grep 분류한 결과 핵심 영역 = **(a) evals 7 commit** (tool formatter + local report + golden issue + triage eval), **(b) core 안정화 60+ commit** (OAuth, capacity, MCP, quota, ReAct), **(c) caretaker agent 신규 14 commit** (Pub/Sub, Firestore, GCP 배포, triage 워크플로우), **(d) ingestion 3 commit** (issue comment + re-triage), **(e) 기타 46 commit** (a2a, hooks, ide, sanitizers). 본 §15.1 은 **우리 §5.5/§5.10/§5.13/§5.14 에 직접 영향** 10 commit 의 짧은 changelog excerpt 포함.

### §15.1 핵심 10 commit (v0.45.0 → v0.55.1, 영향도순)

> 형식: `<sha> <subject> (#<PR>)` + 1-2줄 excerpt. sha 7자리 prefix.

#### 1. tool call formatter — `4238b0b2` feat(evals): add tool call formatter and integrate failure summaries (#28305)
- **영향 영역**: evals / tool call 가시화
- **핵심**: `evals/tool-log-formatter` 추가 — 도구 호출/응답을 사람이 읽기 좋은 형식으로 변환. **failure summary 와 통합** (review feedback + edge case fix 후속 commit `7f9b99bb6` 포함).
- **관련 파일**: `evals/tool-log-formatter.ts` (신규), `scripts/eval-report-cli.ts` 와 통합.
- **우리 영향**: §5.5 wire format (D-108) tool_calls 가시화 디버깅, §16 (a) 항목.

#### 2. IDE connections — `5024443c` fix(core): resolve swallowed directory mismatch in IDE connections (#28729)
- **영향 영역**: IDE 통합
- **핵심**: `packages/core/src/ide/ide-connection-utils.ts` (+73/-). VS Code / JetBrains IDE companion 연결 시 **directory mismatch silent fail** (error swallow) 수정. 150줄 신규 테스트.
- **우회**: stderr / log 로도 안 나오던 silent fail → 예외 propagation.
- **우리 영향**: §5.10 orchestrator sub-agent (에러 propagation 정확성), §16 (b) 의 variant.

#### 3. Cloud Workstations OAuth redirect — `58ba19945` fix(core): dynamically resolve Cloud Workstations proxy redirect URI for OAuth flows (#28688)
- **영향 영역**: OAuth / Cloud Workstations
- **핵심**: `packages/core/src/mcp/oauth-provider.ts` (+5/-). Cloud Workstations 환경의 proxy redirect URI 를 **dynamic resolve** (정적 fallback 폐기). 95줄 신규 테스트.
- **배경**: Cloud Workstations 사용자 = container 안의 proxy 가 redirect URI rewrite → 기존 정적 URI 매칭 실패.
- **우리 영향**: §5.5 D-51 OAuth (W15 의 PKCE redirect URI 처리), §16 (b) 항목.

#### 4. local report command — `659c7aacd` feat(evals): add local report command and developer documentation (#28369)
- **영향 영역**: evals / reporter
- **핵심**: `scripts/eval-report-cli.ts` (신규 81 lines) + `docs/behavioral-evals.md` (신규 185 lines). CI 없이 **로컬에서 evals 결과 리포트** 생성.
- **우리 영향**: §5.13 LLM Wiki ingest 자동화 (장기 retest), §16 (g) 항목.

#### 5. model capacity fix — `188e255bf` fix(core,cli): resolve false model capacity exhaustion and fix core quota lookup model mapping (#28730)
- **영향 영역**: router / quota
- **핵심**: `core` quota lookup 의 model mapping 버그 — **false capacity exhaustion** (실제 quota 남아있는데 exhausted 표시). patch v0.55.0-preview.2 의 cherry-pick (`da3710eb9`).
- **우리 영향**: §5.5 fallback router (D-38), §16 (e) 항목.

#### 6. MCP OAuth token refresh — `eef19f25c` fix(core): refresh MCP OAuth tokens with the stored client ID (#28481)
- **영향 영역**: MCP / OAuth
- **핵심**: `mcp/oauth-provider.ts` 의 refresh 시 **stored client ID** 사용 (이전 = 환경변수 또는 hardcoded). MCP server 인증의 자동 refresh 안정화.
- **우리 영향**: §5.5 W15.b OAuth 자동 refresh (D-58), §16 (c) 항목.

#### 7. NEEDS_HUMAN lock — `cf22ac7e8` fix(caretaker): clear lock on NEEDS_HUMAN transition (#28601)
- **영향 영역**: caretaker agent (Google internal)
- **핵심**: caretaker 가 issue triage 결과 NEEDS_HUMAN 표시 시 **lock clear** 누락 → 같은 issue 재처리 불가. lock state machine 에 transition 추가.
- **우리 영향**: §5.10 sub-agent state machine (orchestrator 의 NEEDS_HUMAN propagation), §16 (d) 항목.

#### 8. ingestion issue comment — `493113457` feat(ingestion): add issue comment handling and re-triage workflow (#28690)
- **영향 영역**: ingestion (issue → triage workflow)
- **핵심**: GitHub issue comment 발생 시 자동 triage + **re-triage workflow** (사용자 follow-up 응답 시 재평가). Pub/Sub topic 발행 → caretaker 가 consume.
- **우리 영향**: §5.13 LLM Wiki 자동 ingest (외부 signal → 메모리 갱신), §16 (g) 항목.

#### 9. Capacity Exhaustion → Terminal Error — `2139b121b` Reclassifying Capacity Exhaustion as Terminal Error (#28716)
- **영향 영역**: error class / retry logic
- **핵심**: **retry 가능한 transient error** 로 분류되던 capacity exhaustion 을 **terminal error** 로 재분류. patch v0.55.0-preview.1 cherry-pick (`b39816c87`).
- **이유**: capacity exhaustion = retry 안 됨 (계속 같은 결과) → 자동 retry 의 hang 방지. 사용자 명시적 액션 필요.
- **우리 영향**: §5.5 fallback router 의 retry policy (D-38), §16 (e) 항목.

#### 10. (보너스) v0.55.1 release — `41327e407` chore(release): v0.55.1
- **영향 영역**: 메타
- **핵심**: 9 package.json version bump. 8 packages (`a2a-server`, `cli`, `core`, `devtools`, `sdk`, `test-utils`, `vscode-ide-companion`) + `package.json` (root) + `package-lock.json`.
- **우리 영향**: §15.0 inventory version 업데이트.

### §15.2 caretaker agent 신규 (Google internal, §16 분석용 4 commit)

> 우리 my_harness 와 직접 통합되지는 않지만, **Google 의 caretaker 아키텍처** = sub-agent + GCP + state machine + eval framework 통합 사례. 설계 참고용.

- **`1b53dfea2`** feat(caretaker): add GCP deployment script for caretaker agent services (#28529) — Cloud Run 배포
- **`d419cb6b6`** feat(caretaker): publish workable spec event to ready-for-code Pub/Sub topic (#28588) — Pub/Sub 기반 비동기 fan-out
- **`afebb8702`** feat(caretaker-evals): add local golden issue collection and firestore sync tools (#28532) — 평가 dataset + Firestore 동기화
- **`6cb9f2e06`** feat(caretaker-evals): add triage evaluation framework and judge runner (#28530) — LLM-as-judge eval framework

### §15.3 우리 영향 1줄 매트릭스 (10 commit × 7 영향 영역)

| # | commit | (a) §5.5 wire | (b) §5.5 OAuth | (c) §5.5 MCP refresh | (d) §5.10 sub-agent | (e) §5.5 router | (f) §5.14 plugin | (g) §5.13 wiki |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | tool call formatter (#28305) | **강** | — | — | weak | — | — | — |
| 2 | IDE connections (#28729) | — | — | — | **강** | — | — | — |
| 3 | CW OAuth redirect (#28688) | — | **강** | weak | — | — | — | — |
| 4 | local report (#28369) | — | — | — | — | — | — | **강** |
| 5 | model capacity (#28730) | — | — | — | — | **강** | — | — |
| 6 | MCP OAuth refresh (#28481) | — | weak | **강** | — | — | weak | — |
| 7 | NEEDS_HUMAN lock (#28601) | — | — | — | **강** | — | — | — |
| 8 | ingestion issue comment (#28690) | — | — | — | weak | — | — | **강** |
| 9 | Capacity → Terminal (#28716) | — | — | — | — | **강** | — | — |
| 10 | v0.55.1 release | — | — | — | — | — | — | — |

→ §16 영향 분석에서 7 영역별 1-2 commit deep dive.

### §15.4 v1 → v2 인벤토리 갱신 (README.md §1 동기화 필요)

| 필드 | v1 (2026-06-07) | v2 (2026-08-14) |
| --- | --- | --- |
| **version** | 0.45.0-nightly.20260602 | **0.55.1** (2026-08-11) |
| **release cadence** | nightly only | nightly + preview + stable (3 채널) |
| **release channels** | 1 (nightly) | 3 (nightly, preview, stable) — `docs/changelogs/index.md` 명시 |
| **active PR count** | ~3,000 | ~3,100 (정확 미공개, 추정) |
| **commits since 06-09** | 0 | **130** (v0.45.0..v0.55.1) |
| **caretaker agent** | 미존재 | **14 commit** (신규, GCP/Pub/Sub/Firestore 통합) |
| **ingestion workflow** | 단발 triage | **issue comment + re-triage** (Pub/Sub driven) |
| **error taxonomy** | transient/permanent | **transient/permanent + terminal** (3-tier) |
| **OAuth** | 정적 redirect URI | **dynamic redirect URI** (proxy 환경 지원) |
| **MCP** | OAuth + manual refresh | **OAuth + 자동 refresh (stored client ID)** |
| **evals** | CI-only | **CI + local report CLI** (`scripts/eval-report-cli.ts`) |
| **triage** | 없음 | **LLM-as-judge eval framework** (caretaker-evals) |

→ 본 표는 `docs/references/README.md` §1 인벤토리 + §3 영향 분석 동기화 입력.

---

## §16 v2 영향 분석 — 우리 my_harness §5.x 별 직접 매핑 (D-132)

> §15 changelog 의 10 commit 을 **우리 my_harness 7 영역** (a~g) 으로 재그룹. 각 항목 = (영향 §, gemini-cli 근거, 우리 action item, 우선순위).
>
> **우선순위**: 🔴 v1 spec 갱신 필요 (현재 활성 작업) · 🟡 v1.5 백로그 입력 · 🟢 v2+ 참고 (의사결정 시 활용).

### (a) tool call formatter → 우리 §5.5 wire format (D-108) 🔴

**문제**: 우리 my_harness v1.5+ 의 `CompletionRequest::tools: Vec<ToolSpec>` + `CompletionResponse::tool_calls: Vec<ToolCall>` (D-108) LLM structured output. 디버깅 시 tool call 의 **가시화** 가 중요. 현재 = 내부 log 에 JSON dump 만 가능.

**gemini-cli 솔루션**: `evals/tool-log-formatter` (PR #28305) — tool call name + args + result 를 **human-readable** 형식으로 변환. failure summary 와 통합 (review feedback 후속).

**우리 action**:
- `myharness/crates/llm/src/format/` (v1.5+) 신규 모듈: `ToolCallFormatter` trait + `DefaultFormatter` (human-readable) + `JsonFormatter` (현재).
- `Orchestrator::dispatch_tool_call` 의 stderr log 에 `DefaultFormatter` 사용 (D-100/D-101 의 `[tool_call] X → ok` 마커 강화).
- 평가 framework (§5.13 LLM Wiki eval) 의 failure case 에 자동 통합.

**선정 근거**: D-100 follow-up 의 실제 pain point (현재 log 가 LLM message 와 tool call 의 매칭 어려움) + gemini-cli 의 검증된 패턴.

### (b) Cloud Workstations OAuth redirect → 우리 §5.5 D-51 OAuth (W15, W15.b) 🟡

**문제**: 우리 `myharness auth <provider> login` (W15, D-51) 의 OAuth loopback HTTP server 는 **정적 redirect URI** (`http://localhost:8085/callback` 등). Cloud Workstations / GitHub Codespaces / Docker container 같은 **proxy 환경** 에서 redirect URI 가 rewrite 되어 실패.

**gemini-cli 솔루션**: `mcp/oauth-provider.ts` 의 `proxy redirect URI` dynamic resolve (PR #28688). 환경 / host header 기반 URI 재구성.

**우리 action**:
- `myharness/crates/auth/src/oauth/callback.rs` (v1.5+) 에 `redirect_uri_resolver` 모듈 추가.
- 3 fallback: (1) 환경변수 `MYHARNESS_OAUTH_REDIRECT_URI` 명시 (2) request host header 기반 (3) 기본 loopback URI.
- 테스트: Cloud Workstations emulation (header injection test) + GitHub Codespaces 환경변수 패턴.

**선정 근거**: 우리 v1 OAuth 가 local-only 가정. proxy 환경 (Codespaces 등) 사용 시 silence fail 가능. v1.5 부터 안전장치.

### (c) MCP OAuth token refresh → 우리 §5.5 W15.b 자동 refresh (D-58) 🔴

**문제**: 우리 myharness-auth 의 OAuth token refresh (W15.b, D-58) 는 **client_id 를 어디서 가져올지** 미정. 현재 = refresh 요청에 client_id 미포함 또는 placeholder.

**gemini-cli 솔루션**: PR #28481 — MCP OAuth token refresh 가 **stored client ID** 를 token store 에서 read. client_id 별도 저장 (token 과 함께).

**우리 action**:
- `myharness/crates/auth/src/store.rs` (v1.5+) — `~/.myharness/oauth/{provider}.toml` schema 갱신: `client_id` + `client_secret` (optional) 필드 추가. 기존 token-only entry 와 호환 (lazy load).
- `PkceProvider` 의 `refresh()` 가 client_id 를 store 에서 read.
- mini migration: 기존 v1 token entry 에 placeholder client_id 자동 주입 (interactive prompt 또는 `auth login` 재실행 안내).

**선정 근거**: W15.b 의 미해결 결정 (D-58 후속). gemini-cli 의 동일 문제 해결이 SSOT 패턴.

### (d) NEEDS_HUMAN lock → 우리 §5.10 sub-agent (orchestrator) 🟡

**문제**: 우리 §5.10 의 3-tier orchestrator (D-100) + sub-agent (W10, D-48) 는 **NEEDS_HUMAN** (= 사용자에게 결정 위임) 케이스 어떻게 처리? 현재 = 단순 prompt 출력 후 idle. **state machine** 없음.

**gemini-cli 솔루션**: PR #28601 (caretaker) — NEEDS_HUMAN transition 시 **lock clear**. lock state machine: `idle → processing → needs_human → idle` (전환 시 lock release).

**우리 action**:
- `myharness/crates/tui/src/orchestrator.rs` (v1.5+) — `AgentState` enum 추가: `Idle/Processing/NeedsHuman/Error`. `NeedsHuman` 진입 시 sub-agent task lock 해제 + user prompt 즉시 표시.
- `LoopRunner` (W10) 의 interrupt 와 통합 — `NeedsHuman` 도 interrupt 의 한 형태.

**선정 근거**: W10 의 LoopRunner 가 현재 `--max-iterations` 외 종료 조건 미지원. NEEDS_HUMAN = 명시적 종료 조건. gemini-cli 의 state machine 차용.

### (e) Capacity Exhaustion → Terminal Error → 우리 §5.5 router (D-38) 🔴

**문제**: 우리 `FallbackRouter` (W7, D-38) 의 retry policy. capacity exhaustion (rate limit / quota) 발생 시 자동 retry vs fallback provider 전환. 현재 = **단순 transient error** 로 retry → 무한 retry 가능.

**gemini-cli 솔루션**: PR #28716 (release v0.55.0-preview.2) — capacity exhaustion → **terminal error** 분류. + PR #28730 false capacity exhaustion 버그 수정.

**우리 action**:
- `myharness/crates/llm/src/error.rs` (v1.5+) — `LlmError::CapacityExhaustion` variant 추가. `is_terminal()` method.
- `FallbackRouter::handle_error()` — terminal error 시 retry 대신 **fallback provider cascade** (다음 provider).
- false positive 방지: per-provider quota lookup 의 model mapping 정확성 (PR #28730 패치 lesson).

**선정 근거**: D-38 fallback router 의 핵심 결정. transient vs terminal 분류가 retry 전략 직결. gemini-cli 의 3-tier (transient/permanent/terminal) 분류가 우리 router spec 정합.

### (f) TOML extensions 표준 → 우리 §5.14 (D-33) 🟢

**문제**: 우리 §5.4 plugin 시스템 (D-33) 의 manifest format. 현재 결정 보류 (TASK-007). 후보: claude-code `plugin.json` / gemini-cli TOML extensions / goose recipe yaml.

**gemini-cli v2 동향**: extensions 시스템 v0.45.0 → v0.55.1 사이 **TOML extensions** 가 지속 확장 (tools/core 양쪽). MCP server manifest 도 TOML 위주. 우리 §5.14 영향은 **v2+** 의사결정 시 활용.

**우리 action**:
- TASK-007 (`docs/decision_log.md` §11) 에 **TOML extensions** 옵션 추가 검토.
- 비교 매트릭스: JSON (claude-code) vs TOML (gemini-cli) vs YAML (goose) 의 (가독성, schema validation, Rust serde 호환, ecosystem).
- 결론 보류 — v1.5 plugin 시스템 구현 시 3-way 비교 후 결정.

**선정 근거**: gemini-cli 가 TOML 채택이 강제적이진 않지만, **wire format 의 통일성** (settings.toml + extensions.toml + oauth.toml) 이 우리 `~/.myharness/config.toml` 채택과 정합.

### (g) ingestion workflow → 우리 §5.13 LLM Wiki 자동 ingest 🟡

**문제**: 우리 §5.13 LLM Wiki memory (D-32, D-74) 의 자동 ingest. 현재 = 수동 (`wiki-lint` + 수동 commit). **외부 signal → 자동 update** 메커니즘 부재.

**gemini-cli 솔루션**: PR #28690 (ingestion) — GitHub issue comment = 외부 signal → Pub/Sub 발행 → **re-triage workflow** 자동 trigger. (gemini-cli caretakers 의 eval framework PR #28530 도 같은 패턴).

**우리 action**:
- `myharness/crates/llm-wiki/src/ingest/` (v1.5+) — 외부 source adapter: GitHub issue comment / GitHub PR comment / Obsidian second-brain update webhook (D-74).
- workflow: `signal → classify (LLM-as-judge) → update wiki node → emit event (Pub/Sub-like in-process bus)`.
- 평가: re-triage 정확도 (의미 있는 update vs 잡음) — LLM-as-judge framework (PR #28530 차용).

**선정 근거**: D-74 LLM Wiki 의 1차 구현 (manual + lint). v1.5 에서 외부 signal 자동 ingest 가 핵심 가치. gemini-cli 의 Pub/Sub + re-triage pattern.

---

## §17 결정 / 후속 (Decisions & Follow-ups)

### §17.1 결정 (D-132, 2026-08-14)

- **D-132** — TASK-004 재방문, gemini-cli v2 영향 분석 (06-09 이후 130 commit, v0.45.0 → v0.55.1).
- **누적 결정 74 → 75** (D-126 → D-132).
- 본 v2 분석 (`docs/references/gemini-cli.md` §15/§16) = **SSOT 입력**.

### §17.2 v1 → v1.5 백로그 입력 (6 action item)

| # | 영역 | 우선순위 | v1.5 sprint 후보 |
| --- | --- | --- | --- |
| (a) | §5.5 wire format — `ToolCallFormatter` 신규 | 🔴 | W19 (TBD) |
| (b) | §5.5 OAuth — `redirect_uri_resolver` | 🟡 | W20 (TBD) |
| (c) | §5.5 MCP OAuth — `client_id` store schema | 🔴 | W19 (W15.b 후속) |
| (d) | §5.10 sub-agent — `AgentState` state machine | 🟡 | W21 (TBD) |
| (e) | §5.5 router — `CapacityExhaustion::is_terminal()` | 🔴 | W19 (D-38 후속) |
| (g) | §5.13 wiki — 외부 signal ingest adapter | 🟡 | W22 (D-74 후속) |

→ (f) = TASK-007 plugin 결정 시 입력 (TBD).

### §17.3 상호 참조 (Cross-References)

- **본 v2 입력**: `docs/references/README.md` §1 (inventory v2 version 갱신) + §3 (my_harness 영향 분석 동기화)
- **본 v2 후속**: `docs/CONCEPT.md` §5.5 (action a, b, c) / §5.10 (action d) / §5.13 (action g)
- **본 v2 결정**: D-132 (`docs/decision_log.md` §D-132 entry 추가, TBD)
- **본 v2 메모리**: `ai-workflow/memory/backlog/2026-08-14.md` (TBD, 메모리 sync commit)
- **본 v2 후속 reference**: gemini-cli v0.55.1 → v0.56.0 (nightly) 동향 추적 (다음 TASK-004 재방문 시점, 추정 2026-09)

### §17.4 한계 / 리스크

- **130 commit 중 10 commit 만 deep dive** — 나머지 120 commit (a2a-server, hooks, sanitizers, ReAct mitigation, file keychain, OAuth credentials 등) 는 v1 spec 영향 없음 / v2+ 참고 한정. 필요 시 후속 TASK-004 재방문에서 다룸.
- **업스트림 PR # 미공개** — PR 번호는 `#28305` ~ `#28771` 범위이지만 (2026-08 시점), 일부 PR 의 본문 description 은 미수집 (commit message + 변경 파일만 분석).
- **gemini-cli = Google internal** — caretaker agent 영역 (Pub/Sub, Firestore, GCP deployment) 은 직접 차용 불가하나 **아키텍처 패턴** (state machine + LLM-as-judge + Pub/Sub fan-out) 은 차용 가능.
- **§15/§16 의 7 action item 중 🔴 3개** (a/c/e) 는 v1.5 진입 시 우선 처리 대상. W19 sprint planning 시 본 §17.2 백로그 확인.


---

## §18 부록 — 10 commit 의 추가 컨텍스트 (deep dive)

> §15.1 의 10 commit 을 **우리 my_harness 적용 관점** 으로 1-2 추가 문단씩. v1.5 sprint planning 시 본 §18 + §17.2 함께 활용.

### §18.1 tool call formatter (#28305) — 우리 dispatch log

**v1 현재**: `myharness/crates/tui/src/orchestrator.rs` 의 `dispatch_tool_call` 은 `[tool_call] {name} → ok` stderr 마커만 emit. LLM response 와 tool call 의 **상호 연관** 파악 어려움. D-100 테스트 suite (18 tests) 가 `truncate_output` 2000자 truncation 으로 부분 가시화하지만, **failure case** (tool error / timeout) 가 무지성으로 `[tool_call] X → err: ...` 만 출력.

**v2 (gemini-cli)** 적용 후: `ToolCallFormatter::format(call: &ToolCall, result: &ToolResult) -> String` 가
- 입력: tool name + args (BTreeMap) + result status + output
- 출력: `◆ tool_name(args["key1"]="value1", args["key2"]=42) → success (12ms, 1.2KB)` 형식
- failure: `✗ tool_name(args) → error: <message> (kind: Timeout)` 형식
- summary: 동일 form 5회 반복 시 1회 summary 로 collapse (review feedback commit `7f9b99bb6` 의 "edge case fix" 차용)

**구현 위치**: `myharness/crates/llm/src/format/tool_call_formatter.rs` (v1.5+, 신규 ~150 lines + 30 tests).

### §18.2 IDE connections (#28729) — 우리 propagator

**v1 현재**: 우리 TUI 와 외부 환경 (process spawn, IDE) 사이의 error propagation 은 `Box<dyn Error>` 위주. **directory mismatch** (working directory 불일치) 가 silent fail 가능.

**v2 (gemini-cli)** 적용 후: `MyharnessError::IdeConnection { kind: DirectoryMismatch | HostUnreachable | AuthMissing, source: Box<dyn Error> }` 명시적 variant. `?` operator 가 propagation 시 error chain 보존.

**구현 위치**: `myharness/crates/tui/src/error.rs` (v1.5+, `MyharnessError` enum 확장 + `From<io::Error>` 등 변환).

### §18.3 Cloud Workstations OAuth redirect (#28688) — 우리 redirect URI

**v1 현재**: `myharness/crates/auth/src/oauth/callback.rs` 의 `redirect_uri` = `http://127.0.0.1:8085/callback` 고정. `auth login --no-browser` 의 `--redirect-uri` 플래그 없음.

**v2 (gemini-cli)** 적용 후: `redirect_uri_resolver` 우선순위:
1. `MYHARNESS_OAUTH_REDIRECT_URI` 환경변수 (explicit)
2. `request.host_header` 기반 (proxy 환경, Cloud Workstations / Codespaces)
3. loopback default (`http://127.0.0.1:8085/callback`)

**테스트**: `tests/oauth_redirect_uri.rs` — header injection 으로 3가지 fallback 검증.

### §18.4 MCP OAuth token refresh (#28481) — 우리 client_id persistence

**v1 현재**: `~/.myharness/oauth/{provider}.toml` 의 schema 가 `{access_token, refresh_token, expires_at, scope}` 4 필드. `client_id` 없음.

**v2 (gemini-cli)** 적용 후: schema 확장:
```toml
[google]
client_id = "1234567890.apps.googleusercontent.com"
client_secret = ""  # optional, public client 는 빈 문자열
access_token = "ya29..."
refresh_token = "1//0g..."
expires_at = 2026-08-14T12:34:56Z
scope = "https://www.googleapis.com/auth/userinfo.email"
```

**migration**: 기존 v1 entry 에 `client_id = ""` placeholder 자동 주입. `auth login --refresh` 시 prompt 로 client_id 입력.

**구현 위치**: `myharness/crates/auth/src/store.rs` (serde schema 확장 + migration).

### §18.5 NEEDS_HUMAN lock (#28601) — 우리 AgentState

**v1 현재**: `LoopRunner` (W10, D-48) 의 상태 = `{Idle, Running, Completed, Failed}` 4가지. NEEDS_HUMAN 별도 상태 없음 — 단순 stderr 출력 후 idle 복귀.

**v2 (gemini-cli)** 적용 후: `AgentState` 6-state:
```rust
enum AgentState {
    Idle,
    Running { task_id: Uuid, started_at: DateTime },
    NeedsHuman { question: String, context: serde_json::Value },
    Error { last_error: MyharnessError, retry_count: u8 },
    Completed { artifacts: Vec<PathBuf> },
    Cancelled { reason: String },
}
```

**전이**: `Running → NeedsHuman` 시 sub-agent task lock 즉시 release + user prompt 표시. `LoopRunner::tick()` 가 `NeedsHuman` 감지 시 interrupt.

### §18.6 Capacity Exhaustion terminal (#28716) — 우리 is_terminal()

**v1 현재**: `FallbackRouter::handle_error()` 의 retry 결정 = 단순 `error.is_retryable()` bool. retry 가능 = 동일 provider 재시도, retry 불가 = fallback provider cascade.

**v2 (gemini-cli)** 적용 후: 3-tier 분류:
```rust
impl LlmError {
    pub fn is_terminal(&self) -> bool {
        matches!(self, LlmError::CapacityExhaustion(_) | LlmError::AuthMissing(_) | LlmError::ModelDeprecated(_))
    }
    pub fn is_transient(&self) -> bool {
        matches!(self, LlmError::NetworkTimeout(_) | LlmError::RateLimitLimited { .. })
    }
    pub fn should_cascade(&self) -> bool {
        self.is_terminal() || matches!(self, LlmError::QuotaExceeded { .. })
    }
}
```

**router 결정 트리**: `terminal → fallback cascade` / `transient → retry same provider` / `permanent → fallback cascade + log`.

**false positive 방지** (PR #28730 lesson): per-provider quota lookup 의 model mapping 정확성. 우리 `myharness-llm/src/quota.rs` 의 `model_mapping` table 의 unit test 강화.

### §18.7 ingestion issue comment (#28690) — 우리 wiki ingest

**v1 현재**: §5.13 LLM Wiki (D-32, D-74) 의 update = 수동. `wiki-lint` + 수동 commit + 수동 cron 없음.

**v2 (gemini-cli)** 적용 후: 외부 signal adapter (v1.5+ `myharness/crates/llm-wiki/src/ingest/`):
- **source 1**: GitHub issue comment (`gh api` polling or webhook)
- **source 2**: GitHub PR comment (review follow-up)
- **source 3**: Obsidian vault update (D-74 second-brain)
- **workflow**: signal → `WikiIngestor::classify()` (LLM-as-judge) → `WikiNode::update()` → `EventBus::emit()` (in-process Pub/Sub)
- **re-triage**: 동일 signal 의 follow-up 발생 시 `WikiNode::re_evaluate()` 호출 (PR #28530 의 LLM-as-judge framework 차용)

**평가**: re-triage 정확도 (precision/recall) — `caretaker-evals` PR #28530 의 judge runner 패턴 (golden dataset + LLM judge).

### §18.8 model capacity fix (#28730) — 우리 quota lookup

**v1 현재**: 우리 `myharness-llm/src/quota.rs` 의 `model_mapping: HashMap<String, Provider>` 가 정적. model ID 변경 시 silent fail 가능.

**v2 (gemini-cli)** 적용 후: `quota.rs` 의 `model_mapping` Table 에 **fallback chain** 추가:
```rust
fn resolve_quota_model(requested: &str) -> Option<&'static str> {
    MODEL_MAPPING.get(requested)
        .or_else(|| MODEL_ALIAS.get(requested))
        .copied()
}
```

**테스트**: 50+ model ID enum (anthropic: claude-3-5-sonnet-*, openai: gpt-4o-*, gemini: gemini-2.5-*) 에 대해 lookup 정확성 검증.

### §18.9 local report command (#28369) — 우리 eval CLI

**v1 현재**: 우리 `myharness eval` subcommand 없음. evaluation = 수동 (Claude Code review 시에만).

**v2 (gemini-cli)** 적용 후: `myharness eval <set> --format=local` subcommand (v1.5+):
- `myharness/crates/cli/src/eval.rs` 신규
- `scripts/eval-report-cli.ts` (TS, gemini-cli 와 같은) 또는 `myharness eval --output=md` (Rust native)
- 평가 dataset: `tests/eval/{coding,review,fix}_dataset.json` (golden examples)

**연계**: §18.7 의 wiki ingest 의 judge runner 와 동일 dataset 공유.

### §18.10 v0.55.1 release (#41327e407) — 인벤토리 갱신 트리거

본 v2 분석의 **trigger commit**. `docs/references/README.md` §1 인벤토리 표의 `LOC` 는 v1 (2026-06-07) 시점 21,074 lines 였으나, v0.55.1 시점 ≈ ~24,000+ lines (추정, 미확정). 정확한 LOC 는 `cloc upstream/main` 명령 필요 (TBD, v1.5+ 검증).

