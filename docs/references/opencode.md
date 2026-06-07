# OpenCode (sst/opencode) — 심층 분석

- 문서 목적: `sst/opencode` 레퍼런스의 실제 코드를 14섹션 표준 템플릿으로 분석. 1차 분석 `docs/REFERENCES.md` §3.1 의 1-페이지를 깊이 10배로 확장.
- 범위: opencode 전체 (TypeScript monorepo + Solid.js TUI + Effect 라이브러리)
- 대상 독자: yklee, Mavis, TASK-005 디자인 리뷰 참여자
- 상태: final (1차 draft, 2차 확장)
- 최종 수정일: 2026-06-07
- 관련 문서: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [REFERENCES.md §3.1](../REFERENCES.md), [TASK-005 my_harness CLI/TUI 전환](../../../ai-workflow/memory/backlog/2026-06-05.md)

## §1 개요 (Overview)

- **프로젝트명**: opencode (`sst/opencode`)
- **라이선스**: MIT
- **언어**: TypeScript (Bun 런타임), 일부는 Go SDK
- **메인 binary**: `packages/opencode` (`bin/opencode`)
- **디폴트 브랜치**: `dev` (main 아님, 의도적 — SST 가 feature 트랙과 stable 트랙 분리)
- **코드 규모 (추정)**: monorepo, `packages/opencode` 메인 ~ 수십만 LOC, 22+ packages
- **타겟 사용자**: 풀스택 + 멀티스택 개발자, TUI-네이티브 워크플로 선호자, plugin 으로 확장하는 team
- **1줄 설명**: "AI 코딩 에이전트, TUI-first, JSON-declared plugin 시스템으로 무한 확장"

## §2 아키텍처 (Architecture)

### 2.1 프로세스 모델: Client-Server 분리

opencode 의 가장 두드러진 아키텍처 결정은 **TUI 와 server 의 완전 분리**:

```
┌─────────────────────────────────────────┐
│  TUI (Solid.js, terminal)               │  ← thin client
│  packages/opencode/src/cli/cmd/tui/      │
└────────────────┬────────────────────────┘
                 │ RPC (HTTP/JSON-RPC)
                 ↓
┌─────────────────────────────────────────┐
│  Server (Express-like, Node)             │  ← brain
│  packages/server/src/                   │
│  - HTTP API                              │
│  - Middleware (auth, rate-limit)         │
│  - Routes / handlers                     │
│  - Session store                         │
└────────────────┬────────────────────────┘
                 │ Provider API
                 ↓
┌─────────────────────────────────────────┐
│  LLM Providers (OpenAI, Anthropic, ...)  │
└─────────────────────────────────────────┘
```

**장점**:
- 한 server 에 여러 TUI 동시 attach 가능 (`opencode attach <url>` 명령)
- Server 만 재시작해도 TUI 세션 유지
- Electron / web / IDE 확장이 **같은 server** 에 attach 가능
- 동시 다중 세션 / 멀티 디바이스

### 2.2 모노레포 구조

```
packages/
├── opencode (메인 CLI)
│   ├── src/cli/cmd/tui/ (TUI 진입점)
│   ├── src/cli/cmd/<command>/ (서브커맨드)
│   ├── src/server/ (server 진입점)
│   ├── src/plugin/ (plugin 호스트)
│   ├── src/session/ (세션 추상화)
│   └── src/cli/cmd/tui/worker.ts (TUI server worker)
├── ui (@opencode-ai/ui, 공유 컴포넌트)
├── core (@opencode-ai/core, 도메인 로직)
├── plugin (@opencode-ai/plugin, plugin API)
├── llm (provider 어댑터)
├── sdk (TypeScript SDK)
├── server (HTTP server)
├── desktop (Electron, 별도)
└── ... 22+ packages
```

### 2.3 핵심 추상화

- **Session**: SessionID (UUID), schema (effect Schema)
- **EventV2**: 도메인 이벤트 (typed schema, Effect-기반)
- **Plugin**: `.opencode/plugin/*.{ts,js}` 로 로드, JSON manifest + JS hook
- **Tool**: `read_file`, `write_file`, `bash_run` 등 표준 + custom
- **Provider**: LLM 어댑터 (OpenAI, Anthropic, Bedrock, etc)
- **Skill**: 재사용 가능한 prompt + tool bundle (Markdown + JSON)
- **Agent**: role 정의 (`.opencode/agent/*.md`)

### 2.4 데이터 흐름

```
[User Input]
   ↓
[TUI: app.tsx] → render with @opentui/solid
   ↓ (input event)
[EventV2: tui.prompt.append]
   ↓ (RPC to server)
[Server: POST /session/{id}/message]
   ↓
[Session → Agent → LLM]
   ↓ (streaming)
[TUI: streaming render]
   ↓
[User Output]
```

## §3 진입점 & CLI

### 3.1 메인 진입점

`packages/opencode/src/cli/cmd/cmd.ts` — 모든 커맨드의 registry. `cmd({...})` helper 로 각 커맨드 정의.

### 3.2 CLI 명령 트리

```
opencode
├── run               # 메인: LLM 세션 시작
├── tui               # TUI 모드 (default)
├── attach <url>      # 원격 server 에 TUI 연결
├── server            # Server 모드 (백그라운드)
├── install           # 설치 / 셋업
├── session           # 세션 관리 (list, new, share)
├── plugin            # 플러그인 관리
└── ... <others>
```

### 3.3 서브커맨드 dispatch

`yargs` 기반 인자 파싱 + `cmd()` helper 가 command registration. 각 서브커맨드는 별도 디렉토리 (`packages/opencode/src/cli/cmd/<name>/`).

### 3.4 `attach` (독특한 디자인)

`opencode attach <url>` — 원격 server 에 TUI 클라이언트로 attach. **같은 세션을 다른 디바이스에서 이어서** 가능. 우리 my_harness 의 `MiniMax.md` 운영 정책 (단일 진입점) 과 다른 방향 — opencode 는 **세션 중심**, 우리는 **사용자-세션 매핑** 중심.

### 3.5 인자 파싱

`yargs` (`packages/opencode/src/cli/cmd/cmd.ts:cmd()` helper). `--dir`, `--continue`, `--session` 등 플래그. POSIX 스타일 (--kebab-case).

## §4 TUI/UI 구현

### 4.1 TUI 라이브러리: @opentui/solid

**`@opentui/solid`** — Solid.js reactivity + 커스텀 TUI 렌더러. 직접 만든 라이브러리 (`@opentui/core`, `@opentui/keymap` 등 npm 패키지로 분리).

**왜 자체 라이브러리?**:
- React (ink) 의 한계: 큰 terminal grid 에서 성능
- Bubbletea (Go) 의 transpile 부담: TypeScript-native 가 더 자연스러움
- Solid.js: fine-grained reactivity, React API 와 매우 유사, 번들 크기 작음

### 4.2 TUI 디렉토리 구조

```
packages/opencode/src/cli/cmd/tui/
├── app.tsx              # 메인: createCliRenderer() + RouteProvider
├── keymap.tsx           # 키맵 시스템 (LEADER_TOKEN, OPENCODE_BASE_MODE)
├── attach.ts            # 원격 server attach 진입점
├── worker.ts            # TUI server worker (RPC to server)
├── thread.ts            # RPC client to server
├── event.ts             # TUI 가 정의한 이벤트 (EventV2)
├── layer.ts             # Effect Layer (관심사 분리)
├── attention.ts         # 알림 / 사운드 (defaultSoundPath 등)
├── config/
│   ├── tui.ts           # TUI 설정 스키마
│   ├── tui-schema.ts    # Zod 스키마
│   └── keybind.ts       # 키 바인딩 정의
├── win32.ts             # Windows 콘솔 FFI (bun:ffi → kernel32.dll)
├── validate-session.ts  # SessionID 검증
└── ui/, component/, context/, plugin/ ...
```

### 4.3 render loop

```typescript
// packages/opencode/src/cli/cmd/tui/app.tsx
import { render, useRenderer, useTerminalDimensions } from "@opentui/solid"
import { createCliRenderer, MouseButton, type CliRenderer } from "@opentui/core"
import { RouteProvider } from "@tui/context/route"

const renderer = await createCliRenderer({ /* config */ })
render(() => <RouteProvider><App /></RouteProvider>, renderer)
```

Solid.js signal (`createSignal`, `createMemo`) 가 reactivity. `useEffect` 로 side effect. OpenTUI 의 `CliRenderer` 가 low-level 터미널 IO (escape codes, raw mode).

### 4.4 상태 관리

Solid.js signal + Effect Layer. `packages/opencode/src/cli/cmd/tui/layer.ts` 가 Effect.Layer 로 TUI 의 의존성 주입:

```typescript
// layer.ts
export const CliLayer = Observability.layer.pipe(
  Layer.merge(TuiConfig.layer),
  Layer.provide(Npm.defaultLayer)
)
```

### 4.5 키 바인딩

`@opentui/keymap` — vim-like modal 키맵. `LEADER_TOKEN`, `OPENCODE_BASE_MODE`, `COMMAND_PALETTE_COMMAND` 등 상수. 사용자가 `.opencode/keybind.json` 으로 override.

### 4.6 테마

`packages/ui/src/theme/` — 다중 테마. JSON 정의 + CSS-style. `desktop-theme.schema.json` 으로 검증.

### 4.7 Windows 처리 (`win32.ts`)

```typescript
// packages/opencode/src/cli/cmd/tui/win32.ts
import { dlopen, ptr } from "bun:ffi"

const kernel = () => dlopen("kernel32.dll", {
  GetStdHandle: { args: ["i32"], returns: "ptr" },
  GetConsoleMode: { args: ["ptr", "ptr"], returns: "i32" },
  SetConsoleMode: { args: ["ptr", "u32"], returns: "i32" },
  FlushConsoleInputBuffer: { args: ["ptr", "ptr"], returns: "i32" },
})

export function win32DisableProcessedInput() {
  // ENABLE_PROCESSED_INPUT clear — Windows 콘솔이 Ctrl+C 가공 안 하도록
}
```

`bun:ffi` 로 Windows native API 호출. **Bun 런타임의 강점** — Node 에선 native module 빌드 필요하지만 Bun FFI 는 직접.

### 4.8 Plugin routes

`packages/opencode/src/cli/cmd/tui/plugin/` — TUI 안에서 plugin 의 UI route 자동 등록. 플러그인 작성자가 TUI 메뉴 추가 가능.

## §5 LLM 통합

### 5.1 Provider 추상화

`packages/llm/` — 모든 LLM provider 의 통합 인터페이스. OpenAI 호환 (Anthropic, Bedrock, Google, local 등) 자동 매핑.

### 5.2 Streaming

`packages/llm/src/stream.ts` — SSE / chunked 응답 처리. TUI 는 EventV2 이벤트 스트림으로 변환.

### 5.3 Tool calling

JSON Schema 기반 function calling. Tool 정의는 `.opencode/tool/*.json` 또는 plugin 으로 추가. 모델 출력 → tool call → 실행 → 결과 → 모델 재호출 loop.

### 5.4 Token 추적

`packages/llm/src/usage.ts` (추정) — input/output/cache_read/cache_write tokens 추적. UI 에 표시 + 비용 추정.

### 5.5 Error handling

- Rate limit: exponential backoff + circuit breaker
- Context overflow: 자동 compaction
- Network error: retry with jitter

## §6 도구/스킬 시스템

### 6.1 도구 등록 메커니즘

`.opencode/tool/` 또는 plugin 으로 도구 추가. JSON manifest + JS 핸들러.

### 6.2 내장 도구 (추정)

- `read_file` — 파일 읽기
- `write_file` — 파일 쓰기
- `edit_file` — 부분 수정 (search/replace)
- `bash_run` — shell 명령 실행
- `list_files` — 디렉토리 목록
- `search_files` — grep / regex 검색
- `web_search` — 웹 검색
- `web_fetch` — URL fetch

### 6.3 Skill

`.opencode/skill/*.md` — 재사용 가능한 prompt + tool bundle. 마크다운 frontmatter (YAML) + 본문.

```markdown
---
name: pr-review
description: Review a pull request
tools: [read_file, list_files, bash_run]
---
You are a code reviewer. ...
```

### 6.4 Agent

`.opencode/agent/*.md` — role 정의. 프롬프트 + tool allowlist + 모델.

### 6.5 Permission 모델

`.opencode/config.json` 의 `permissions` 블록 (추정). 도구별 allowlist/blocklist. **사용자 컨펌 단계** (코드 변경 전 diff 표시).

## §7 컨텍스트 관리

### 7.1 파일 읽기 전략

- `read_file` 도구가 page 단위로 chunked read
- 큰 파일은 offset + limit 으로 부분 읽기
- AST-aware 검색 (`tree-sitter-*`) 으로 의미 기반 매칭

### 7.2 Repo 인덱싱

`packages/opencode/src/indexer/` (추정) — ripgrep / tree-sitter 기반 인덱스. SQLite 캐시.

### 7.3 Token 예산

- 모델 context window 마다 자동 추정
- Token 사용량 UI 표시
- 한계 도달 시 자동 compaction (이전 메시지 요약)

### 7.4 요약 / Compaction

`packages/opencode/src/session/compact.ts` (추정) — token-budgeted summarization. aider 의 `ChatSummary(max_tokens=1024)` 와 유사.

### 7.5 Truncation

큰 tool 결과는 자동 truncate + "... (N more lines)" 표시.

## §8 세션 영속화

### 8.1 Session Schema

`packages/opencode/src/session/schema.ts` — `SessionID` (UUID), effect Schema 기반. `Schema.decodeUnknownSync(SessionID)` 로 검증.

### 8.2 Storage

`journal.json` — 세션의 모든 활동 (message, tool call, file edit) JSON Lines. git 친화적.

### 8.3 Resume

`opencode run --continue` 또는 `--session <id>` — 기존 세션 재개. journal replay.

### 8.4 EventV2

```typescript
// packages/opencode/src/event.ts (effect-based)
import { EventV2 } from "@opencode-ai/core/event"

const TuiEvent = {
  PromptAppend: EventV2.define({ type: "tui.prompt.append", schema: { text: Schema.String } }),
  CommandExecute: EventV2.define({ type: "tui.command.execute", schema: { ... } }),
  // ...
}
```

typed events — 도메인 이벤트 + Effect 통합.

## §9 확장 시스템

### 9.1 Plugin 시스템

`.opencode/plugin/*.ts` — TypeScript / JavaScript hook. Manifest (`plugin.json`) + 핸들러.

### 9.2 Plugin API

`packages/plugin/` — `@opencode-ai/plugin` npm 패키지. plugin 작성자가 import. `definePlugin({...})` helper.

```typescript
// 예시 plugin
import { definePlugin } from "@opencode-ai/plugin"

export default definePlugin({
  name: "my-plugin",
  hooks: {
    "session.start": async (ctx) => { /* ... */ },
    "tool.before": async (ctx) => { /* ... */ },
  }
})
```

### 9.3 Hook 이벤트 (추정)

- `session.start`, `session.end`
- `message.before`, `message.after`
- `tool.before`, `tool.after`
- `file.before_edit`, `file.after_edit`

### 9.4 `.opencode/` 디렉토리 (7개)

```
.opencode/
├── agent/         # role 정의 (.md)
├── command/       # slash commands (.json or .md)
├── glossary/      # 도메인 용어집
├── plugin/        # TypeScript / JavaScript plugins
├── skill/         # 재사용 prompt + tool bundle (.md)
├── theme/         # TUI 테마 (.json)
└── tool/          # 커스텀 tool 정의 (.json or .ts)
```

**우리의 MiniMax.md 와 직접 매핑** — 우리 운영 정책 → `.opencode/agent/MiniMax.md` + `command/`, `tool/`, `skill/` 자동 생성 가능.

### 9.5 MCP 통합

`packages/opencode/src/mcp/` (추정) — stdio + HTTP transport. 우리 my_harness 의 `rmcp` 채택 결정과 정합.

## §10 빌드 & 배포

### 10.1 빌드 시스템

- **Bun** — 런타임 + 번들러 + 테스트 러너. `bun test`, `bun build`, `bun run`.
- **Vite** — `packages/ui` 의 dev server / build.
- **Turbo / Nx** — 모노레포 빌드 오케스트레이션 (의심, 미확인).
- TypeScript (`tsconfig.json` per package).

### 10.2 단일 바이너리

`bun build --compile` — Bun 의 native binary 컴파일. **Node + JS 번들 + native code** → 단일 실행 파일.

**장점**: 우리 my_harness 의 단일 binary 목표와 정확히 일치. **TS 2안** 의 큰 강점.

### 10.3 Cross-platform

- Bun: Linux / macOS (Intel + Apple Silicon) / Windows 지원
- `bun build --compile --target=bun-linux-x64` 등 cross-compile
- Native binary 이므로 Node 런타임 의존성 없음

### 10.4 Distribution

- **GitHub Releases**: platform 별 binary (`opencode-linux-x64`, `opencode-darwin-arm64`, ...)
- **Homebrew** (의심, 미확인)
- **npm/npx**: `npx opencode` (Bun 없이도)

### 10.5 Install / Update

- `npm i -g opencode` 또는 download binary
- 자체 update 메커니즘 (의심, 미확인)

## §11 테스트 & 품질

### 11.1 테스트 구조

- `bun test` — built-in test runner
- 각 package 의 `*.test.ts` 또는 `__tests__/`
- `packages/ui/test` — React Testing Library
- `packages/opencode/test/` — CLI integration

### 11.2 테스트 패턴

- Unit test: pure function / utility
- Integration: CLI invoke + assert output
- E2E: 실제 LLM 호출 (CI 비용 큼, nightly 만)
- Snapshot: 컴포넌트 렌더링 (Vitest + testing-library)

### 11.3 품질 도구

- TypeScript strict mode
- ESLint (likely)
- Prettier (likely)
- Effect 의 type system (functional + typed errors)

### 11.4 CI

GitHub Actions — multi-OS (ubuntu, macos, windows) + bun + lint + test + build.

## §12 보안

### 12.1 시크릿 관리

API keys — 환경변수 + `~/.opencode/auth.json` (likely, file-based). Keychain 통합은 미확인. **우리가 강제할 keyring 패턴** 과 대비.

### 12.2 Sandbox

- `bash_run` 도구가 OS-level sandbox 인지 미확인. (opencode 도 aider/goose 처럼 OS sandbox 부재 추정)
- prompt injection 무방어 추정

### 12.3 Permission 모델

도구별 allowlist (plugin/agent config) + 사용자 컨펌 (코드 변경 전 diff). 추정.

### 12.4 네트워크 정책

- Web fetch 도구는 사용자 명시
- LLM provider endpoints — TLS
- No egress filter (theoretical)

### 12.5 Audit log

`journal.json` (세션 활동) + git history (코드 edit). 추정.

## §13 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 Client-Server 분리

TUI 와 server 의 완전 분리는 **best architecture 결정**. 한 server 에 다중 TUI attach 가능. 우리 my_harness v2+ 에서 검토 (MVP v1 은 single binary).

#### 13.2 TUI 라이브러리 자체 개발 (`@opentui/solid`)

OpenTUI 의 npm 패키지 분리 (`@opentui/core`, `@opentui/keymap`) — **TUI 라이브러리 재사용 가능**. 우리 my_harness 도 만약 Rust 1안 (ratatui) 채택 시 우리만의 wrapper crate 분리 검토.

#### 13.3 JSON-declared plugin/agent/skill/tool

`.opencode/` 의 7개 서브디렉토리 모두 **JSON / Markdown 으로 선언**. 우리 my_harness 의 `MiniMax.md` 와 표준 workflow 와 **직접 매핑**. TASK-005 시 우리 `~/.myharness/agent/`, `skill/`, `tool/` 동일 패턴.

#### 13.4 Effect 라이브러리 사용 (`packages/opencode/src/cli/cmd/tui/layer.ts`)

`effect` 라이브러리 — `Layer`, `Effect`, `Schema` 로 functional + typed pattern. 우리 my_harness 1안 (Rust) 의 Tokio + tracing 과 동일한 차원의 추상화.

#### 13.5 Effect-기반 EventV2

`EventV2.define({...})` — typed schema + Effect 통합. 우리도 session state event 의 typed schema 채택 (예: Zod 또는 Rust 의 `serde + ts-rs`).

#### 13.6 `journal.json` (JSON Lines session log)

세션의 모든 활동 JSON Lines. git-friendly, human-readable, external export 가능. 우리 my_harness 의 `state.json` 과 같은 차원 — JSONL 추가 검토.

#### 13.7 Plugin 의 hook 시스템

`session.start`, `tool.before`, `tool.after` 등 typed event hook. 우리 MiniMax.md 의 워커 토폴로지 (orchestrator/worker/doc/code/validation) 와 직접 매핑.

#### 13.8 `cmd()` helper (CLI command registry)

`yargs` + `cmd()` helper 로 command registration 단순화. 우리 my_harness 의 clap (Rust) 또는 commander (TS) 와 동일 — 한 곳에서 정의.

#### 13.9 Windows FFI (`bun:ffi` + `kernel32.dll`)

`win32.ts` 가 Windows native API 직접 호출. Bun 런타임의 강점. **우리 Rust 1안 (Windows Job Object) 또는 TS 2안 (koffi / node-ffi-napi)** 와 같은 패턴.

#### 13.10 `attach <url>` (다른 디바이스에서 같은 세션)

원격 server 에 TUI attach. **세션 중심** 설계. 우리 v2+ 검토 (MVP v1 은 single device).

#### 13.11 `dev` branch default

feature 브랜치 vs stable 분리. **인기있는 feature 일찍 노출 + 안정성 보장**. 우리도 `main` vs `dev` 분리 검토.

#### 13.12 `tui-smoke.tsx` 같은 plugin 기반 smoke test

`.opencode/plugins/tui-smoke.tsx` — TUI 자체를 plugin 으로 테스트. 우리도 TUI smoke test 도입.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.13 OS-level sandbox 부재 (aider/goose 와 동일)

`bash_run` 도구가 OS sandbox (Seatbelt/bwrap/Windows Job) 없이 실행. **우리 my_harness 는 v1 부터 OS sandbox** — 이게 우리 차별점.

#### 13.14 Effect 라이브러리의 학습 곡선

`effect` 는 fp-ts 와 유사한 functional library. **러닝 커브 가파름**. 우리 my_harness 가 Rust 1안이면 `tokio` + `tracing` 으로 충분 (덜 exotic).

#### 13.15 `solid-js` 는 React 경험자에게 낮설음

`@opentui/solid` 는 Solid.js. **React 개발자** (대다수)에게 낯선 API. 우리 my_harness 가 TS 2안 + React 경험자 다수면 `ink` (React) 가 진입장벽 낮음.

#### 13.16 `dev` default branch — stable 다운로드를 복잡하게 함

`opencode` 를 clone 받으면 `dev` branch. **stable 쓰려면 명시적 checkout** 필요. `main` default 가 안정성 측면에선 친숙.

#### 13.17 Plugin + `.opencode/` 의 7개 디렉토리 — v1 복잡도

`.opencode/` 7개 서브디렉토리 + plugin 22+ packages = **초기 학습 곡선 가파름**. 우리 my_harness v1 은 1-2개 디렉토리 + 3-5 commands 부터.

#### 13.18 Bun 런타임 종속성

Bun-only 기능 (`bun:ffi`, `Bun.file()`) — 우리 my_harness 가 Bun 채택 시 lock-in. Node 호환성 부족.

#### 13.19 journal.json 의 무한 성장

세션이 길어지면 journal.json 도 커짐. **compaction 필요**. 우리도 동일.

#### 13.20 Server 의 in-memory state

Server 가 in-memory session state — **restart 시 손실** (journal.json 으로 복원은 가능). 우리도 SQLite + journal.json 분리 검토.

#### 13.21 Plugin 간 의존성 관리 부재

`.opencode/plugin/` 의 plugin 들이 서로 의존 가능. **명시적 dependency graph** 없으면 conflict.

#### 13.22 Effect Layer 깊이

`packages/opencode/src/cli/cmd/tui/layer.ts` 의 `Layer.merge` / `Layer.provide` 체인 — 디버깅 어려움. 우리도 `tokio::main` 단순함 선호.

## §14 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 Server 의 persistence 전략

`journal.json` 만 의존? 아니면 SQLite 같은 외부 store? Restart 후 복원 정확도?

### 14.2 Bun 만의 feature (Bun.file, bun:ffi) 가 core 에 얼마나 침투?

전부 의존하면 Node 호환성 영구 손실. 일부만 의존하면 cross-runtime 가능. 실제 비율?

### 14.3 `solid-js` 의 메모리 footprint

큰 terminal grid (200x60) 에서 signal 기반 reactivity 의 메모리 사용. React 와 비교.

### 14.4 Plugin 의 권한 sandbox

plugin 이 OS 명령 실행 시 sandbox? (의심: 없음)

### 14.5 `attach` 의 인증 메커니즘

원격 server attach 시 토큰 / password? (`ServerAuth` 클래스가 `packages/opencode/src/server/auth.ts` 에 있긴 하나 정확한 흐름 미확인)

### 14.6 Effect 라이브러리의 안정성

`effect` 0.x → 1.0 의 진화. API breaking change 빈도. 우리 채택 결정 시 위험도.

### 14.7 `dev` branch 의 stable 로 머지 주기

`dev` 에서 `main` 으로의 merge cadence — 일별? 주별? 릴리스별?

### 14.8 plugin marketplace / registry

`.opencode/plugin/` 의 plugin 들을 공유하는 registry 존재? (GitHub topic, npm scope, 별도 사이트)

### 14.9 Electron desktop 의 feature parity

`ui/desktop/` 가 TUI 와 100% 동일한 기능? 아니면 subset? 우리 desktop 검토 시 결정.

### 14.10 `journal.json` vs `state.json` (standard_ai_workflow) 의 호환성

우리 standard_ai_workflow 의 `state.json` 과 opencode 의 `journal.json` 의 차이. 통합 가능?

### 14.11 opencode 의 `provider` 추상화 vs litellm

`packages/llm/` 의 provider 시스템이 litellm 처럼 unified interface? 아니면 provider-specific 코드? 우리 1안의 litellm-style 선호와 비교.

### 14.12 `opencode` 의 v1.0 release 일정

`dev` branch 가 영구인가, v1.0 시점에 `main` 으로 merge? roadmap 미확인.
EOF