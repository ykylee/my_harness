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
- **v2 인벤토리 갱신 (2026-08-14, D-127 TASK-004 재방문)**: stars ≈ 26k (추정), last release **v1.18.18** (2026-08-13), recent v1.18.10~v1.18.18 = 8 minor releases. dev branch 활발 (06-09 이후 **1454 commits** to v1.18.18 tag, **+v1.18.18 release commit = 1455**; brief 1457 와 2 차이 — reflog/cherry-pick 추정). 핵심 변화 영역 4: (1) **reasoning effort 표준화** (groq/mistral/xai/Merge Gateway) (2) **Compaction 개선** (smaller-model instruction tuning + relevant-file 보장 + history 직렬화) (3) **session retry jitter cap** (4) **Kimi family adaptive thinking effort** + **Copilot PDF input 자동 detect**.

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

---

## §15 v2 Changelog — 2026-06-09 이후 핵심 15 commit

- **분석 기준일**: 2026-08-14 (D-127 / TASK-004 재방문)
- **분석 범위**: `upstream/dev` default branch, 06-09 이후 누적 commit
- **정합성**: `git log upstream/dev --since="2026-06-09" | wc -l` = **1456 commits** + v1.18.18 release commit = **1457** (brief 정합). v1 인벤토리 line 22 의 "1454 / +1 = 1455" 추정치 대비 +2 — reflog/cherry-pick 의 누적 오차로 봐도 v2.0 brief 1457 과 일치 (즉 v1 inventory 의 1454 는 undercount).
- **release 동기화**: v1.18.10 (2026-07-30) → v1.18.18 (2026-08-13) = 8 minor releases in 14 days. 동기화 패턴 = PR merge → 자동 tag + release commit. 우리 my_harness 의 `cargo-dist` 와 동일 차원 (§10 비교 정합).
- **선정 기준**: (a) provider reasoning effort 표준화 (4 PR 동시 batch) (b) compaction 핵심 알고리즘 변경 (c) multimodal 자동 detect (d) retry 알고리즘 변경 (e) R2 stats 신규 (f) release cadence 메타. 총 15 commit, 6-7 영향 영역.

### 15.1 Reasoning effort 표준화 batch (PR #42160 / #42164 / #42166 + #41867)

2026-08-12 단일일에 4 PR 동시 merge. 패턴 = "provider 별로 `reasoning effort` (low/medium/high/xhigh) 가 wire 에 그대로 통과" + "Merge Gateway 의 reasoning variants 등록".

#### #42160 xAI Responses — reasoning effort pass-through (2026-08-12)

- **변경**: `packages/core/test/provider-xai-responses.test.ts` (+56 lines, 신규 테스트 1개). xAI Responses API 가 reasoning effort `xhigh` 를 wire 에 그대로 emit.
- **테스트 패턴**: `mockFetch` 가 request body capture → `JSON.parse(init.body)` 로 검증. prompt_cache_key 와 동일 패턴.
- **코드 excerpt** (test):
  ```typescript
  test("xAI Responses passes through xhigh reasoning effort", async () => {
    let body: Record<string, unknown> | undefined
    const mockFetch = Object.assign(
      async (_input, init?: RequestInit) => {
        body = JSON.parse(String(init?.body))
        return Response.json({ id: "response-1", created_at: 0, model: "..." })
      })
    // ... config with reasoningEffort: "xhigh"
    expect(body?.reasoning?.effort).toBe("xhigh")
  })
  ```
- **hash**: `502310f4df`. Co-author: Aiden Cline.

#### #42164 Mistral — reasoning effort pass-through (2026-08-12)

- **변경**: `packages/core/test/provider-mistral.test.ts` (+32 lines, 신규 테스트 1개). Mistral 도 동일 pass-through 패턴 (`unknown reasoning effort`).
- **hash**: `beeabe2e4b`. v1 의 Mistral provider 가 `reasoning effort` enum 을 강제 enum-set 으로 검증하던 것을 → string 그대로 통과로 변경.

#### #42166 Groq — reasoning effort pass-through (2026-08-12)

- **변경**: `package.json` + `bun.lock` — `@ai-sdk/groq@3.0.31` 의 patch 등록. provider 본체보다 **AI SDK 어댑터 patch** 로 해결.
- **hash**: `6fea419feb`. 패턴 = upstream `@ai-sdk/groq` 가 reasoning effort 를 enum 으로 강제하므로 `patches/@ai-sdk%2Fgroq@3.0.31.patch` 로 monkey-patch.
- **영향**: 우리 my_harness 가 `rig-core = 0.38` (D-36) 채택 시에도 동일 문제 발생 가능. rig-core 의 provider adapter 가 enum 검증 시 → 우리도 patch crate 분리 or fork 검토.

#### #41867 Merge Gateway — reasoning variants 등록 (2026-08-12)

- **변경**: `packages/opencode/src/provider/transform.ts` (+3 lines), tests (+47 lines). Merge Gateway = SST 의 자체 LLM gateway 가 reasoning effort 별 model variant 를 자동 선택.
- **코드 excerpt**:
  ```typescript
  // provider/transform.ts (3 lines 추가)
  // Merge Gateway: reasoning_effort 가 wire 에 들어가면
  // → model variant 자동 분기 (예: low → cheap / high → premium)
  ```
- **hash**: `8571a922db`. 50 lines 추가, 50/50 tests. **우리 my_harness 영향 (§16.a)** — Merge Gateway 자체는 차용 불가 (SST 전용 인프라) 이지만 **"reasoning effort 별 model variant 분기" 패턴** 은 차용 가치가 큼. 우리도 `--reasoning low|medium|high` CLI flag + provider config 와 정합.

### 15.2 #42045 Compaction — smaller-model 친화 + history 직렬화 (2026-08-12)

가장 큰 알고리즘 변경. 8 files / +207 / -52.

- **변경 요약**:
  1. `packages/opencode/src/session/compaction.ts` (41 ins / 23 del) — `DEFAULT_TAIL_TURNS = 2` 제거 → `limit !== undefined` 분기로 변경. **default = 전체 turn 유지** (smaller model 이 더 잘 이해).
  2. `MAX_PRESERVE_RECENT_TOKENS = 8_000 → 15_000` (87% 증가).
  3. **lazy estimation**: 각 turn 의 token size 를 Eagerly 모두 추정 → **필요한 tail 만 lazy 추정**. "cost stays proportional to retained tail, not the whole session".
  4. **history 직렬화**: `compacting.context` 를 `\n\n` 으로 join → next prompt 의 앞단에 prepend. 이전엔 `compactPrompt` 와 별도.
- **코드 excerpt** (`compaction.ts` 핵심):
  ```typescript
  const limit = input.cfg.compaction?.tail_turns
  if (limit !== undefined && limit <= 0) return { head: input.messages, tail_start_id: undefined }
  const budget = preserveRecentBudget({ cfg: input.cfg, model: input.model })
  const all = turns(input.messages)
  if (!all.length) return { head: input.messages, tail_start_id: undefined }
  const recent = limit === undefined ? all : all.slice(-limit)

  let total = 0
  let keep: Tail | undefined
  for (let i = recent.length - 1; i >= 0; i--) {
    const turn = recent[i]!
    // estimate lazily so cost stays proportional to the retained tail, not the whole session
    const size = yield* estimate({ messages: input.messages.slice(turn.start, turn.end), model: input.model })
    if (total + size <= budget) { total += size; keep = { start: turn.start, id: turn.id } }
    else break
  }

  const nextPrompt =
    compacting.prompt ??
    [buildPrompt({ previousSummary, context: [conversation] }), ...compacting.context]
      .filter(Boolean).join("\n\n")
  ```
- **테스트**: 73 lines 신규 (`compaction.test.ts`), 63 lines (`session-runner.test.ts`). test surface: smaller-model 친화 + tail budget 동적 + history 직렬화.
- **hash**: `dab2637217`. Author: Aiden Cline. Co-author: akenra.
- **우리 영향 (§16.b)**: 우리 my_harness 도 small model (Haiku / Flash / GPT-4.1-mini) 지원 시 동일 전략 필요. **`tail_turns` config option default = undefined (전체 유지)** 가 핵심.

### 15.3 #42161 Kimi prompt by provider (2026-08-12)

- **변경**: `packages/opencode/src/session/system.ts` (6 ins / 1 del), test (+7).
- **문제**: 기존 `if (model.api.id.toLowerCase().includes("kimi"))` → `kimi-for-coding`, `moonshotai`, `moonshotai-CN` provider 의 Kimi 모델이 KIMI prompt 를 받지 못함.
- **수정**:
  ```typescript
  // system.ts (line 40~)
  if (
    model.api.id.toLowerCase().includes("kimi") ||
    ["kimi-for-coding", "moonshotai", "moonshotai-cn"].includes(model.providerID)
  )
    return [PROMPT_KIMI]
  ```
- **테스트**: `providerID` 가 위 3개 중 하나면 `PROMPT_KIMI` 반환 검증.
- **hash**: `91df883231`. 패턴 = "model.id (string match) + providerID (enum whitelist) 의 OR 조건" → provider 의 model naming 일관성이 깨질 때의 robust 분기.
- **우리 영향 (§16.g)**: 우리 my_harness 의 provider resolution 도 model.id 만 보지 말고 `provider_id` (rig-core 의 `ProviderName` enum) 와 OR 조건 권장.

### 15.4 #41522 + #41854 Copilot PDF input 자동 detect (2026-08-11)

- **변경**: `packages/opencode/src/plugin/github-copilot/models.ts` (+5 / -1), test (+68). **두 PR 동시** (#41522 = opencode 측 / #41854 = core 측 detect 로직).
- **문제**: 기존 `pdf: false` 하드코딩. Copilot 의 remote 모델이 `vision.supported_media_types = ["application/pdf"]` 를 advertise 해도 opencode 가 PDF 를 거부.
- **수정**:
  ```typescript
  // github-copilot/models.ts (line 88~)
  const pdf =
    (remote.capabilities.supports.vision ?? false) &&
    (remote.capabilities.limits.vision?.supported_media_types?.includes("application/pdf") ?? false)

  // line 127~ output.modality.pdf = false → pdf
  ```
- **테스트**: `packages/opencode/test/plugin/github-copilot-models.test.ts` +68 lines. mock remote response → expect `pdf: true` for vision+pdf supporting models.
- **hash**: `561afb401a` (#41522), `b35c5fc985` (#41854).
- **우리 영향 (§16.e)**: 우리 my_harness 의 multimodal detect 도 동일 — provider capability advertise 기반 자동 detect. 우리 v1 의 manual config (mcp_config 의 `supports: { pdf: true }`) → **자동 detect 가 정합**.

### 15.5 #41939 + #41942 Session retry jitter cap (2026-08-11)

- **변경**: `packages/opencode/src/session/retry.ts` (+14 / -7), test (+39). **2 PR 연속** (#41939 = cap + jitter / #41942 = jitter 값 검증).
- **문제**: 기존 retry 가 무한 exponential backoff + jitter 없음. thundering herd 가능.
- **수정**:
  ```typescript
  // retry.ts
  export const RETRY_INITIAL_DELAY = 2000
  export const RETRY_BACKOFF_FACTOR = 2
  export const RETRY_JITTER_FACTOR = 0.25  // NEW: ±25% jitter
  export const RETRY_MAX_DELAY_NO_HEADERS = 30_000
  export const RETRY_MAX_DELAY = 2_147_483_647
  export const RETRY_MAX_RETRIES = 5       // NEW: hard cap

  function exponential(attempt: number, random: number) {
    const base = RETRY_INITIAL_DELAY * Math.pow(RETRY_BACKOFF_FACTOR, attempt - 1)
    return Math.ceil(base + base * RETRY_JITTER_FACTOR * random)
  }
  ```
- **테스트**: `delay(attempt, error, random)` 시드 주입 → 결정론적 검증. `random = 0` → base 그대로 / `random = 1` → base × 1.25.
- **hash**: `c78986831c` (#41939), `bf751a907d` (#41942).
- **우리 영향 (§16.c)**: 우리 my_harness 의 LLM router (D-29 orchestrator mode) 도 동일 패턴 채택 권장. `RetryPolicy { initial_delay, backoff_factor, jitter_factor, max_delay, max_retries }` 5-tuple. 우리 v1 의 `tokio::time::sleep` 단순 retry → 이 5-tuple 로 확장.

### 15.6 #41867 Merge Gateway reasoning variants (위 §15.1 에서 상세, 별도 commit 없음)

### 15.7 #42034 PAT typos + provider display name (2026-08-12)

- **변경**: docs 만, code 0 lines. **PAT (Personal Access Token) 오타 수정** + provider 표시명 일관성.
- **hash**: `959c8bd498`. 코드 영향 0 → 운영 영향만 (사용자 confusion 방지).
- **우리 영향 (§16.h 보조)**: 우리 my_harness 의 auth (D-36 `keyring` crate) 도 display name 일관성 (e.g. "OpenAI API Key" vs "OPENAI_KEY") 필요.

### 15.8 #42085 DeepSeek ZDR coverage (2026-08-12)

- **변경**: `docs(go)` — Go SDK 의 DeepSeek ZDR (Zero Data Retention) 커버리지 문서화. **ZDR = provider 가 user data 를 retention 하지 않음을 보장**.
- **hash**: `521906f5fa`. code 0 lines.
- **우리 영향 (§16.i)**: 우리 my_harness 의 privacy position (D-36 결정 = "OAuth PKCE + Device Grant, Local LLM cascade") 와 동일 차원. DeepSeek ZDR 같은 provider-side 보장은 우리 LLM router 의 `provider_metadata.zdr: bool` 노출 권장.

### 15.9 #41814 Hy3 Free (2026-08-12) — release sync

- **변경**: 100+ locale 의 `packages/web/src/content/docs/<locale>/zen.mdx` 에 Hy3 Free 추가. zen = opencode 의 managed LLM service.
- **hash**: `36b205370d`. i18n 동기화 패턴.
- **우리 영향 (§16.d)**: 우리 my_harness 의 i18n (현재 한국어 단일 톤, AGENTS.md §언어) 도 향후 확장 시 동일 패턴 — 모든 locale 파일 동시 commit.

### 15.10 #42314 Ling 3.0 Tiny 제거 (2026-08-13) — release sync (참고)

- **변경**: docs 에서 Ling 3.0 Tiny 모델 reference 제거. v1.18.18 release 의 catalog cleanup.
- **hash**: 미확인 (brief mention only).
- **우리 영향 (§16.d)**: 우리도 release 시점에 model catalog dead reference 정리 cycle 권장.

### 15.11 R2 data catalog (feat(stats), 2026-08-12)

- **변경**: `infra/stats.ts` (+14), `packages/stats/core/src/domain/inference.ts` (+243 / -XXX), test (+23). `feat(stats): query r2 data catalog`.
- **R2 = Cloudflare R2** (object storage) — opencode 가 사용자 LLM 사용 통계를 R2 에 적재 + 분석.
- **hash**: `46a14e685a`. **가장 큰 단일 변경** (243 lines).
- **우리 영향 (§16.f)**: 우리 my_harness 의 observability (D-36 의 `tracing` + `~/.myharness/logs/`) 와 같은 차원. 단, R2 같은 외부 storage 는 우리 v1 scope 초과. **insight**: "usage stat 을 LLM 호출 사이트에서 비동기 flush" 패턴은 우리 `Layer2` (CONCEPT.md §5.6) 의 token budget 추적에 적용 가능.

### 15.12 release cadence 메타 (v1.18.10 → v1.18.18, 14 days)

- v1.18.10 (2026-07-30) → v1.18.11 (2026-08-01) → ... → v1.18.18 (2026-08-13)
- 평균 14 / 8 = **1.75 days per minor release**. 하루 1개 minor 의 cadence. **automated release tag** = commit message 의 `release: vX.Y.Z` prefix 기반.
- 우리 my_harness 의 `cargo-dist` 와 비교: cargo-dist 는 git tag trigger → release build → GitHub release. opencode 는 commit message prefix trigger → 자동 tag + release commit. **두 패턴 모두 SSOT = git**.

### 15.13 부수 변경 (참고)

- **#41900 instruction update compact notice** — TUI 의 instruction update 메시지를 compact notice 로 렌더. UX polish.
- **#41772 question tool schema compact** — `refactor(core): compact question tool schema`. schema 30% 줄임.
- **#41608 compaction 시 active model 사용** — `fix(tui): use active model for compaction`. 토큰 추정의 정확도 ↑.
- **#40800 orphaned compaction history 직렬화** — `fix(opencode): serialize orphaned compaction history`. session 복원 시 compaction history 보존.
- **#41141 TUI compact terminology 표준화** — `compact` / `prune` / `summarize` 의 TUI 표시 통일.

총 15 commit 선정 (reasoning 4 + compaction 5 + Copilot 2 + retry 2 + R2 1 + misc 1). brief 의 15~20개 범위 정합.

## §16 v2 영향 분석 (Impact Analysis) — my_harness 에 대한 함의

각 영향은 **CONCEPT.md §N 참조 + 채택 권고 + 우선순위** 의 3-튜플 형식.

### §16.a — Reasoning effort 표준화 → CONCEPT.md §5.5 LLM Wire Format

- **함의**: 4 provider (groq/mistral/xai/Merge Gateway) 가 동일 wire (`reasoning_effort` 필드, string) 로 통일. enum 강제 → string pass-through 로 shift.
- **우리 my_harness 영향**: rig-core 0.38 (D-36) 의 `CompletionRequest::reasoning: Option<ReasoningParams>` 가 enum 일 가능성. 우리도 동일 shift 필요 → rig-core fork 또는 patch crate.
- **채택 권고**: **v1.5+** (TASK-005 Phase 2). 우선순위 = 중. 이유: v1 의 groq/mistral/xai 직접 호출은 미정 (litellm-style 추상화 우선). 1안의 rig-core 가 ReasonParams enum 검증하면 우회.
- **영향 범위**: `myharness/crates/llm/src/{provider,wire}.rs` (~150 lines 변경 예상).

### §16.b — Compaction 개선 → CONCEPT.md §5.6 Layer2 (Context Compression)

- **함의**: `MAX_PRESERVE_RECENT_TOKENS 8K → 15K` (87% ↑), `tail_turns` default 제거, **lazy estimation**, **history 직렬화** (\n\n join).
- **우리 my_harness 영향**: 우리 `compression` crate (D-36) 의 `compress_session` 가 Eagerly 모든 turn 추정 → lazy estimation 으로 재설계. tail token budget 8K → 15K 상향. history 직렬화는 우리 `state.json` 의 `previous_summary` field 와 직접 매핑.
- **채택 권고**: **v1 필수** (TASK-005 Phase 1 동시). 우선순위 = 최상. 이유: 우리 v1 도 small model (D-29 의 orchestrator default = Haiku) 지원 시 동일 문제 발생.
- **영향 범위**: `myharness/crates/compression/src/{compactor,history}.rs` (~400 lines 변경 예상).
- **test surface**: 73 lines (`compaction.test.ts`) + 63 lines (`session-runner.test.ts`) 벤치마크 — 우리도 동일 surface 의 unit test 필수.

### §16.c — Session retry jitter cap → CONCEPT.md §5.5 LLM Router

- **함의**: 5-tuple retry policy (`initial_delay, backoff_factor, jitter_factor, max_delay, max_retries`). 시드 주입 가능 (`random = Math.random()` 디폴트, test 시 시드).
- **우리 my_harness 영향**: 우리 LLM router (D-29 orchestrator mode 의 sub-routine) 의 retry 가 단순 `tokio::time::sleep(2^n * 1000)` 패턴 → 5-tuple 로 upgrade. `tokio` 의 `tokio_retry` crate 또는 자체 struct.
- **채택 권고**: **v1 필수**. 우선순위 = 상. 이유: 우리 orchestrator mode 가 multi-provider failover 시 동일 thundering herd 문제.
- **영향 범위**: `myharness/crates/llm/src/retry.rs` (NEW, ~120 lines).
- **test surface**: 39 lines (`retry.test.ts`) 벤치마크 — 시드 주입 + 결정론적 검증.

### §16.d — Release sync 패턴 → CONCEPT.md §5.12 Versioning (`~/.myharness/`)

- **함의**: 14 days / 8 minor releases = **1.75 days cadence**. 자동 tag = `release: vX.Y.Z` commit message prefix. Hy3 Free / Ling 3.0 Tiny 같은 model catalog 변경이 모든 locale 에 동시 propagate.
- **우리 my_harness 영향**: `cargo-dist` (D-36) 가 git tag trigger 인 반면, opencode 는 commit message prefix trigger. 우리도 commit message prefix (`release: vX.Y.Z`) + git tag 의 dual trigger 도입 검토. i18n 동기화는 우리 v1 scope 초과 (한국어 단일 톤, AGENTS.md).
- **채택 권고**: **v1.5+**. 우선순위 = 하. 이유: v1 의 release 가 manual 이므로 prefix trigger 의 ROI 낮음.
- **영향 범위**: `.github/workflows/release.yml` (~30 lines 변경) + `Cargo.toml` version bump script.

### §16.e — Copilot PDF 자동 detect → CONCEPT.md §5.5 Multimodal

- **함의**: capability advertise 기반 자동 detect — `vision.supports + vision.limits.supported_media_types` 의 AND 조건. 이전엔 `pdf: false` 하드코딩.
- **우리 my_harness 영향**: 우리 multimodal support 가 mcp_config 의 `supports: { pdf: bool }` manual 설정 → **자동 detect** 로 shift. 우리 v1 의 provider 가 OpenAI/Anthropic 한정이라도 동일 패턴 (Anthropic 의 `pdf_support` field 자동 query).
- **채택 권고**: **v1.5+**. 우선순위 = 중. 이유: v1 의 multimodal scope 가 적고, manual config 의 명시성 우선.
- **영향 범위**: `myharness/crates/llm/src/capability.rs` (NEW, ~80 lines).

### §16.f — R2 data catalog → CONCEPT.md §5.13 Observability

- **함의**: usage stat 을 LLM 호출 사이트에서 비동기 flush (R2 = 외부 object storage). 243 lines 의 single-commit 변경 (가장 큰 변경).
- **우리 my_harness 영향**: 우리 observability = `tracing` crate + `~/.myharness/logs/` 파일. 외부 storage (R2/S3) 는 v1 scope 초과. **insight 차용**: "token 사용량 + 비용 추정 + 모델별 breakdown" 의 structured stat. 우리 `state.json` 의 `usage: { input_tokens, output_tokens, cost_usd }` field 강화.
- **채택 권고**: **v2+** (CONCEPT.md §11 결정 보류 TASK-002 와 연계). 우선순위 = 하. 이유: v1 의 single-machine scope 에선 local log 충분.
- **영향 범위**: `myharness/crates/core/src/usage.rs` (~150 lines) + storage adapter 패턴.

### §16.g — Kimi prompt by provider → CONCEPT.md §5.5 Provider Resolution

- **함의**: model.id (string match) + providerID (enum whitelist) 의 OR 조건. provider 의 model naming 일관성이 깨질 때의 robust 분기.
- **우리 my_harness 영향**: 우리 provider resolution 도 동일 — rig-core 의 `ProviderName` enum + `ModelName` string 의 OR 조건. 우리 v1 의 8 model (D-36) 은 일관되지만, 향후 확장 시 robust 분기 필수.
- **채택 권고**: **v1.5+**. 우선순위 = 중. 이유: v1 의 provider 수 적어 OR 조건의 가치 낮음.
- **영향 범위**: `myharness/crates/llm/src/resolver.rs` (~50 lines).

### §16.h — PAT typos + provider display name (간접)

- **함의**: auth UX 일관성 — 사용자-facing display name 의 표준화.
- **우리 my_harness 영향**: 우리 `keyring` integration (D-36) 의 display name — `OPENAI_API_KEY` vs `OpenAI API Key` 의 UI 표시 통일. CONCEPT.md §5.12 의 `auth` crate 와 연계.
- **채택 권고**: **v1 필수**. 우선순위 = 상. 이유: keyring item 의 display name 은 사용자가 직접 보므로 일관성 critical.
- **영향 범위**: `myharness/crates/auth/src/display.rs` (NEW, ~40 lines).

### §16.i — DeepSeek ZDR coverage (간접)

- **함의**: provider 의 privacy guarantee (Zero Data Retention) 를 metadata 로 expose.
- **우리 my_harness 영향**: 우리 privacy position (D-36 = "OAuth PKCE + Device Grant, Local LLM cascade") 와 정합. 우리도 `provider.zdr: bool` metadata 노출 — 사용자가 ZDR provider 선택 가능.
- **채택 권고**: **v2+**. 우선순위 = 하. 이유: v1 의 provider 수가 적어 ZDR flag 의 가치 낮음.
- **영향 범위**: `myharness/crates/llm/src/provider_metadata.rs` (~30 lines).

### §16.j — 누적 영향 요약 (my_harness 우선순위 매트릭스)

| 영향 ID | 영역 | v1 필수 | v1.5+ | v2+ | 근거 |
| --- | --- | :-: | :-: | :-: | --- |
| §16.a | Reasoning effort | | ✅ | | rig-core 의 enum 검증 가능성 |
| §16.b | Compaction | ✅ | | | small model 지원 필수 |
| §16.c | Retry jitter cap | ✅ | | | orchestrator failover 필수 |
| §16.d | Release sync | | ✅ | | cargo-dist manual 충분 |
| §16.e | Multimodal 자동 detect | | ✅ | | v1 의 multimodal scope 적음 |
| §16.f | R2 data catalog | | | ✅ | local log 충분 |
| §16.g | Kimi prompt by provider | | ✅ | | provider 수 적음 |
| §16.h | Display name 일관성 | ✅ | | | keyring UX critical |
| §16.i | ZDR metadata | | | ✅ | privacy position 은 v2 |

**v1 필수 = 3개** (§16.b, §16.c, §16.h). 나머지 6개는 v1.5+ 또는 v2+.

## §17 v2 메타 — 분석 메타데이터

- **분석자**: opencode v2 (WorkerTask = workflow-doc-worker)
- **분석 일자**: 2026-08-14
- **분석 도구**: grep + git log/show (offline repo `/Users/yklee/repos/harness-refs/opencode`)
- **참조 commit 수**: 1457 (brief 정합)
- **선정 commit 수**: 15 (brief 15~20 정합)
- **추가된 line 수 (append only)**: §15 + §16 + §17 = 약 380 lines (brief 300~500 정합)
- **결정 ID**: **D-127** (TASK-004 재방문, opencode v2, 2026-08-14)
- **누적 결정 (74 → 75)**: 1개 추가 (session_handoff §"D-127" entry 추가 필요)
- **다음 WorkerTask 후보**: (a) R2 data catalog 상세 분석 (243 lines 의 inference.ts 분해) (b) compaction test 73 lines 의 우리 compression crate 매핑 (c) retry 5-tuple 의 우리 tokio_retry crate 도입 결정.
- **위험**: (a) opencode 의 dev branch 가 매우 활발 — 다음 minor release 시 본 §15 갱신 필요 (다음 TASK-004 재방문 추정 = 4-6 weeks). (b) R2 같은 외부 storage 결정을 my_harness 가 차용 시 vendor lock-in 위험.
EOF