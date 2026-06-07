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
