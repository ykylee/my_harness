# Codex (openai/codex) — 심층 분석

- **문서 목적**: TASK-004 1차 비교표(`docs/REFERENCES.md`) 후속. Codex CLI 의 **실제 코드** 를 심층 분석해, `my_harness` 의 아키텍처 결정(Rust vs TS / TUI 라이브러리 / 토폴로지 / 빌드 / 보안 / 확장 시스템) 에 직접 활용 가능한 인사이트를 만든다.
- **범위**: `codex-rs` 워크스페이스의 100+ crate, 핵심 crate(`core` / `tui` / `cli` / `exec` / `hooks` / `message-history` / `rollout` / `sandboxing` / `state` / `app-server`) 의 코드, `AGENTS.md` 의 거버넌스 규칙, `codex-core` 비대화 방지 규율, 모델 컨텍스트 6개 규칙, 800줄 변경 가이드, 샌드박스 환경변수 규약, 10K 토큰 캡, hooks 10 이벤트, 멀티프로세스 app-server 데몬.
- **대상 독자**: yklee, Mavis, TASK-005 디자인 리뷰 참여자, 후속 5개 레퍼런스 분석(`opencode.md` / `aider.md` / `goose.md` / `gemini-cli.md`) 작성자
- **상태**: done (1차 작성)
- **최종 수정일**: 2026-06-06
- **관련 문서**: [REFERENCES.md (1차 비교표)](../REFERENCES.md), [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [PROJECT_PROFILE.md](../../docs/PROJECT_PROFILE.md), [TASK-005 (CLI/TUI 전환)](../../ai-workflow/memory/backlog/2026-06-05.md)

---

## §1 개요 (Overview)

Codex CLI 는 OpenAI 가 공개한 **로컬 코딩 에이전트 CLI/TUI** 다. `codex` 한 바이너리가 (1) 인터랙티브 TUI, (2) 비대화형 `exec`/`review`/`mcp`/`plugin`/`login` 서브커맨드, (3) VSCode / JetBrains / 외부 IDE 와 통신하는 JSON-RPC `app-server` 데몬, (4) macOS Seatbelt / Linux Landlock/Bubblewrap / Windows Job-Object 기반 샌드박스 실행 환경, (5) MCP 서버(stdio) 까지 다 잡는다. 2024년 4월 ChatGPT Pro 구독자에 한정으로 출시, 같은 해 8월 npm/Homebrew/cargo 공개 릴리즈 → 2025년 1월 Apache-2.0 오픈소스 전환 후 5개월 만에 contributor 600+, PR 1,000+ 가 쏟아진 상태다.

| 항목 | 값 |
| --- | --- |
| 라이선스 | Apache-2.0 (전 repo, 2025-01 전환) |
| 메인 언어 | Rust (edition 2024) |
| 메인 binary | `codex` (cli crate, 1,500+ 줄 main.rs + 25+ 서브커맨드) |
| 보조 binary | `codex-tui`, `codex-exec`, `codex-exec-server`, `codex-app-server`, `codex-app-server-test-client`, `codex-file-search`, `codex-bwrap` (vendored bubblewrap), `codex-stdio-to-uds`, `codex-responses-api-proxy` |
| 워크스페이스 멤버 | `codex-rs/Cargo.toml` 의 `members` 배열 — **94개** (utils/ 포함) |
| LOC 규모 | `core` 45K+, `tui` 60K+, `protocol` 5.6K, `app-server` 1.1K, `sandboxing` 5K, `hooks` 4.3K, `state` 4.3K, `config` 4K, `rollout` 4K, `model-provider` 1K+ |
| 빌드 시스템 | Cargo + Bazel(MODULE.bazel / BUILD.bazel) 듀얼, Justfile 래퍼, Nix flake |
| Lint 규율 | workspace lints 30+ clippy deny 룰, `uninlined_format_args` / `unwrap_used` / `expect_used` / `redundant_clone` / `manual_*` 등 모두 `deny` |
| 핵심 차별점 | 1) **모델 컨텍스트 6개 규칙** (no history rewrite / bounded items / 10K 토큰/item 캡 / 1K 토큰 P0 리뷰), 2) **codex-core 비대화 방지 규율** (AGENTS.md 가 명시), 3) **800줄 변경 가이드**, 4) **10-hook events 엔진** (PreToolUse / PostToolUse / SessionStart / Stop / PreCompact / SubagentStart / SubagentStop / UserPromptSubmit / PermissionRequest), 5) **app-server** (멀티프로세스 JSON-RPC 데몬) — IDE 통합, 6) **Trifecta 샌드박스** (Seatbelt+Landlock+Bubblewrap+Windows Job) |

코드 한 줄로 요약: **"Rust 로 짠 TUI 코딩 에이전트의 reference implementation. 거버넌스·테스트·샌드박스·확장성을 모두 1급 시민으로 다룬다."**

---

## §2 아키텍처 (Architecture)

### 2.1 프로세스 모델 & 데이터 흐름

Codex 는 3-티어 멀티프로세스 구조다.

```
+----------------------------------------------------------------------+
|  Tier 1 - User-facing clients                                          |
|  +------------+  +------------+  +------------+                       |
|  | codex (TUI)|  | codex exec |  |   IDE      |  (VSCode/JetBrains)   |
|  | ratatui    |  | batch run  |  | ext        |                       |
|  +-----+------+  +-----+------+  +-----+------+                       |
|        +----------------+----------------+                              |
|              | stdio (JSON-RPC v2)        |  stdio (JSON-RPC v1)       |
|              v                            v                            |
|  +------------------------------------------------------------+         |
|  | Tier 2 - codex-app-server (daemon)                          |         |
|  |  - 멀티 클라이언트 (stdin/stdout / WebSocket / Unix sock)   |         |
|  |  - Remote control / app-server-daemon lifecycle              |         |
|  |  - V1 + V2 동시 지원 (V1 deprecated, V2 가 active)         |         |
|  +------------------------+------------------------------------+         |
|                           | in-process call (or remote WS)              |
|                           v                                             |
|  +------------------------------------------------------------+         |
|  | Tier 3 - codex-core / codex-thread / session                |         |
|  |  - ThreadManager: 멀티 세션 라이프사이클                      |         |
|  |  - CodexThread: 1 세션 = 1 model call loop                  |         |
|  |  - SandboxManager: 3 플랫폼 backend (seatbelt/landlock/...) |         |
|  |  - RolloutRecorder: JSONL append-only session log           |         |
|  |  - state::StateRuntime + state_db (SQLite, sqlx)            |         |
|  +------------------------------------------------------------+         |
|                                                                            |
+----------------------------------------------------------------------+
                                |
                                v
         +--------------------------------------------+
         | Tier 4 - sandboxed child processes          |
         |  macOS: sandbox-exec (Seatbelt .sbpl)       |
         |  Linux: Landlock + seccomp + Bubblewrap     |
         |  Windows: Restricted token + Job Object     |
         |  + optional vendored bubblewrap binary      |
         +--------------------------------------------+
```

핵심 디렉토리 트리 (요약):

```
codex-rs/
|-- Cargo.toml            # 94 members, [workspace.dependencies]
|-- MODULE.bazel / BUILD.bazel
|-- core/                 # 45K LOC, codex-core (의도적 비대, AGENTS.md 가이드)
|-- tui/                  # 60K LOC, codex-tui (lib + bin + md-events)
|   |-- src/lib.rs        # 3009 줄
|   |-- src/app.rs        # 1363 줄
|   |-- src/chatwidget.rs # 2045 줄
|   |-- src/bottom_pane/  # 16K+ LOC (chat_composer.rs 11K)
|   +-- src/bin/md-events.rs  # 15줄, pulldown_cmark 디버그
|-- cli/                  # 25+ 서브커맨드, main.rs 1,500+
|-- exec/                 # codex-exec, 비대화형 driver
|-- exec-server/          # codex-exec-server, 1st-class daemon
|-- app-server/           # JSON-RPC 멀티프로세스 게이트웨이
|   +-- src/lib.rs        # 1120 줄
|-- app-server-protocol/  # v1+v2 정의, ts-rs 바인딩
|-- app-server-client/    # in-process + remote client
|-- app-server-transport/ # stdio/uds/websocket
|-- app-server-daemon/    # lifecycle (Start/Stop/Restart/EnableRemoteControl)
|-- protocol/             # 5.6K, 모든 wire 타입 (v1+v2 동시)
|-- sandboxing/           # 5K, 3 백엔드
|   |-- seatbelt.rs       # 745 줄, macOS
|   |-- landlock.rs       # 105 줄, Linux native
|   |-- bwrap.rs          # 195 줄, Linux bwrap wrapper
|   +-- manager.rs        # 372 줄, 백엔드 dispatch
|-- linux-sandbox/        # vendored landlock C + Rust wrapper
|-- windows-sandbox-rs/   # 1.4K, Job/Token/DACL/WFP, 11 modules
|-- bwrap/                # 45 줄 main.rs, vendored bubblewrap C 빌드
|-- hooks/                # 4.3K, 10 이벤트 엔진
|   |-- src/lib.rs        # HOOK_EVENT_NAMES 10개
|   |-- src/engine/{discovery,dispatcher,command_runner,output_parser,schema_loader}.rs
|   |-- src/events/       # pre_tool_use, post_tool_use, permission_request,
|   |                     #   session_start, user_prompt_submit, stop,
|   |                     #   compact, common
|   +-- src/bin/write_hooks_schema_fixtures.rs
|-- message-history/      # 437 줄 lib.rs, JSONL append-only + lock
|-- rollout/              # 1.8K recorder.rs, JSONL session persistence
|   +-- src/state_db.rs   # 679 줄, SQLite schema
|-- state/                # 4.3K, state_db + log_db (telemetry)
|-- config/               # 4K, TOML multi-layer resolver
|-- core-plugins/         # plugin 번들 lifecycle
|-- core-skills/          # built-in skills (Markdown)
|-- core-api/             # public API surface
|-- ext/
|   |-- extension-api/    # host API for plugins
|   |-- goal/, guardian/, image-generation/, memories/, skills/, web-search/
|   +-- marketplace/      # 원격 마켓플레이스 (startup_sync)
|-- plugin/               # plugin loader
|-- skills/               # user-facing skills loader
|-- realtime-webrtc/      # WebRTC 실시간 통신 (POC)
|-- app-server-test-client/  # integration test client
|-- tools/                # JSON Schema -> Responses API tool registration
|-- code-mode/            # 모델이 직접 코드를 실행하는 mode
|-- codex-api/            # Responses API wrapper (auth, SSE, files, ...)
|-- model-provider/       # OpenAI / Anthropic / Bedrock / Ollama / ...
|-- model-provider-info/  # provider catalog
|-- codex-mcp/            # MCP client wrapper
|-- mcp-server/           # codex mcp-server (stdio)
|-- chatgpt/              # ChatGPT OAuth (device code + PKCE)
|-- login/                # codex login
|-- keyring-store/        # OS keychain
|-- secrets/              # secret masking
|-- prompts/              # 4개 .md prompt template (gpt-5 / gpt-5.1 / gpt-5.2 / apply-patch)
|-- response-debug-context/
|-- compact/              # 컨텍스트 컴팩션
|-- config/
|   |-- config.md
|   +-- config.schema.json
|-- docs/                 # codex-mcp-interface, protocol_v1
|-- scripts/              # codex packaging, format.py
+-- utils/                # 25+ 작은 crate (cli, cargo-bin, pty, cache, ...)
```

### 2.2 핵심 추상화

`codex-thread` (이전 `ConversationManager`) 가 **1 세션 = 1 CodexThread** 를 표현한다. `CodexThread::submit(Op)` 으로 user input, `CodexThread::next_event()` 로 Event stream 을 소비. 3rd-party 확장은 `CodexThread` 만 의존하도록 의도되어 있고, 그 위에 `ext/extension-api/` 가 또 한 겹 host API 를 제공한다.

`codex-core` 가 에이전트 두뇌다. `Session`, `Turn`, `TurnContext` 가 핵심 타입이고, `client.rs` 가 OpenAI Responses API 어댑터, `client_common.rs` 가 `Prompt` / `ResponseStream` / `ResponseEvent`. **하드 룰**: `core` 는 user-visible stdout/stderr 직접 write 금지 (`#![deny(clippy::print_stdout, clippy::print_stderr)]` - `codex-rs/core/src/lib.rs:6`).

### 2.3 모듈 경계 (3-레이어)

```
           +--------------------+
   0-depth |  codex-core  (의도적으로 큼)         | <- 모든 crate 가 의존하는 중심
           |  - session, turn, tools,            |
           |  - mcp, skills, plugins,             |
           |  - landlock/windows_sandbox(2K)       |
           +--------------------+
                      |
           +----------+----------+
   1-depth |  cli / exec / exec-server / app-server |  <- binary 크레이트
           |  tui / chatgpt / mcp-server             |
           +----------------------+
                      |
           +----------+----------+
   N-depth |  hooks, message-history, rollout,      |  <- 작고 독립적인 유틸/영속화
           |  state, config, skills, plugin,        |
           |  protocol, codex-api, codex-mcp         |
           +----------------------+
```

**`codex-core` 비대화 방지 규율** (`AGENTS.md:66-77`): 신규 crate 가 `codex-core` 에 들어가지 말아야 한다. "기존에 적절한 crate 가 없는지" 먼저 확인하고, 없으면 **새 crate 를 워크스페이스에 추가** 한다. `codex-protocol` (5.6K) / `codex-config` (4K) / `codex-utils-*` (25+) 가 그 결과물. 이 룰 덕분에 `codex-core` 는 비대화되지만 최소한 4-5만 LOC 수준에서 멈춤.

---

## §3 진입점 & CLI

### 3.1 메인 바이너리

| 바이너리 | crate | main.rs | 진입점 | 역할 |
| --- | --- | --- | --- | --- |
| `codex` | `codex-cli` | 1,500+ 줄 | `cli/src/main.rs:895` | 멀티툴 + TUI + 25+ 서브커맨드 |
| `codex-tui` | `codex-tui` | 76 줄 | `tui/src/main.rs:1` | TUI 만 (legacy binary) |
| `codex-exec` | `codex-exec` | - | 비대화형 | CI/자동화 |
| `codex-exec-server` | `codex-exec-server` | - | WebSocket/stdio 데몬 | 분산 실행 |
| `codex-app-server` | `codex-app-server` | 1.1K lib | JSON-RPC | IDE 통합 |
| `codex-bwrap` | `codex-bwrap` | 45 줄 | `bwrap/src/main.rs:1` | vendored bubblewrap, `bwrap_main` C symbol 호출 |
| `md-events` | `codex-tui` (bin) | 15 줄 | `tui/src/bin/md-events.rs:1` | pulldown-cmark 디버그 |
| `codex-file-search` | `codex-file-search` | - | `just file-search` | file-search 데모 |

`codex-cli/src/main.rs:895` `fn main` -> `arg0_dispatch_or_else` (재진입 디스패치, `codex-arg0` crate) -> `cli_main(arg0_paths)` -> `MultitoolCli::parse()` -> 25+ 서브커맨드 매치.

`arg0_dispatch` 는 **같은 바이너리가 다른 이름으로 호출** 되면 다른 crate 로 디스패치 (cargo `--bin codex` + `--bin codex-tui` 등). `codex-tui` 의 `tui/src/main.rs:48` 도 같은 패턴.

### 3.2 명령 트리 (1단계)

`codex [OPTIONS] [PROMPT]` - 비대화형: `codex exec` / `codex review` / `codex apply` (alias `a`) / `codex sandbox` / `codex mcp-server` / `codex app-server` / `codex login` / `codex logout` / `codex update` / `codex doctor` / `codex cloud` (alias `cloud-tasks`) / `codex remote-control` / `codex mcp` (서브커맨드) / `codex plugin` (add/list/remove/marketplace) / `codex completion` / `codex features` (list/enable/disable) / `codex debug` (models/app-server/prompt-input/trace-reduce/clear-memories) / `codex execpolicy` (check) / `codex exec-server` / `codex app` (macOS/Windows) / `codex resume` / `codex archive` / `codex unarchive` / `codex fork` / `codex responses-api-proxy` (internal) / `codex stdio-to-uds` (internal).

**서브커맨드 dispatch 의 핵심 구조** (`cli/src/main.rs:119-205`):

```rust
#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    #[clap(visible_alias = "e")]
    Exec(ExecCli),                    // -> codex-exec
    Review(ReviewCommand),
    Login(LoginCommand),
    Logout(LogoutCommand),
    Mcp(McpCli),                      // -> codex-mcp
    Plugin(PluginCli),
    McpServer(McpServerCommand),      // -> codex-mcp-server
    AppServer(AppServerCommand),      // -> codex-app-server
    RemoteControl(RemoteControlCommand),
    App(app_cmd::AppCommand),         // macOS/Windows only
    Completion(CompletionCommand),
    Update,
    Doctor(DoctorCommand),
    Sandbox(HostSandboxArgs),         // type alias: SeatbeltCommand | LandlockCommand | WindowsCommand
    Debug(DebugCommand),              // models | app-server | prompt-input | ...
    Execpolicy(ExecpolicyCommand),    // hidden
    Apply(ApplyCommand),              // alias "a"
    Resume(ResumeCommand),
    Archive(SessionArchiveCommand),
    Unarchive(SessionArchiveCommand),
    Fork(ForkCommand),
    Cloud(CloudTasksCli),             // alias cloud-tasks
    ResponsesApiProxy(ResponsesApiProxyArgs),   // hidden
    StdioToUds(StdioToUdsCommand),    // hidden
    ExecServer(ExecServerCommand),
    Features(FeaturesCli),
}
```

`MultitoolCli::parse()` 직후 `feature_toggles.to_overrides()` 로 `--enable` / `--disable` flag 를 `-c features.<name>=true/false` 로 변환 후 `config_overrides.raw_overrides.extend()` (cli/src/main.rs:912-913). 즉, **모든 서브커맨드가 동일한 config 우선순위 룰을 공유** 한다.

### 3.3 내부 dispatch

`cli/src/main.rs:922-1450` ~530 줄의 `match subcommand` 블록:

```rust
match subcommand {
    None => run_interactive_tui(...).await?,
    Some(Subcommand::Exec(mut exec_cli)) => { ... codex_exec::run_main(exec_cli, ...).await? }
    Some(Subcommand::McpServer(...)) => codex_mcp_server::run_main(...).await?,
    Some(Subcommand::AppServer(...)) => codex_app_server::run_main_with_transport_options(...).await?,
    Some(Subcommand::Sandbox(mut sandbox_cli)) => {
        #[cfg(target_os = "macos")]
        codex_cli::run_command_under_seatbelt(...).await?;
        #[cfg(target_os = "linux")]
        codex_cli::run_command_under_landlock(...).await?;
        #[cfg(target_os = "windows")]
        codex_cli::run_command_under_windows_sandbox(...).await?;
    }
    ...
}
```

플랫폼별 `HostSandboxArgs` 는 `cli/src/main.rs:378-401` 에서 type alias 로 dispatch:

```rust
#[cfg(target_os = "macos")]
type HostSandboxArgs = codex_cli::SeatbeltCommand;
#[cfg(target_os = "linux")]
type HostSandboxArgs = codex_cli::LandlockCommand;
#[cfg(target_os = "windows")]
type HostSandboxArgs = codex_cli::WindowsCommand;
```

각 sandbox 커맨드는 `cli/src/lib.rs:26-141` 에서 `SeatbeltCommand` / `LandlockCommand` / `WindowsCommand` 별도 struct 이지만 `--permissions-profile` / `-C` / `--include-managed-config` / `trailing_var_arg command: Vec<String>` 공통 필드를 공유.

---

## §4 TUI/UI 구현

### 4.1 스택

| 항목 | 선택 | 비고 |
| --- | --- | --- |
| 렌더링 | **ratatui 0.29.0** (workspace pinned) | fork `nornagon/ratatui` (patch.crates-io), + `ratatui-macros 0.6.0` |
| 백엔드 | **crossterm 0.28.1** | fork `nornagon/crossterm`, GitHub pinned rev (`Cargo.toml:528-529`) |
| Markdown | **pulldown-cmark 0.10** + **syntect 5** + **ansi-to-tui 7** | streaming markdown -> ratatui Span |
| 키 매핑 | **codex-config::tui_keymap** (자체) | vim/emacs/기본 |
| 트리/테마 | `ansi-escape` crate, `theme_picker`, `color.rs` | 다크/라이트 자동 감지 |
| 페트 | `tui/src/pets/` | YES, ascii 애니메이션 펫이 idle 시 돌아다님 |

### 4.2 디렉토리 구조

```
tui/src/
|-- lib.rs            (3009 줄, App 진입점)
|-- app.rs            (1363 줄, Toplevel App)
|-- chatwidget.rs     (2045 줄, 메인 채팅 위젯)
|-- main.rs           (76 줄, binary entrypoint)
|-- bin/
|   +-- md-events.rs  (15 줄, pulldown-cmark 디버그)
|-- app/              (28 modules, 20K LOC)
|   |-- event_dispatch.rs        (2245 줄, 이벤트 분배)
|   |-- background_requests.rs   (1178 줄, 백그라운드 job)
|   |-- config_persistence.rs    (1320 줄)
|   |-- pending_interactive_replay.rs (942 줄)
|   |-- app_server_requests.rs   (848 줄)
|   |-- session_lifecycle.rs     (830 줄)
|   |-- thread_routing.rs        (1574 줄)
|   +-- ...
|-- bottom_pane/      (16K+ LOC, 입력 + 모달)
|   |-- chat_composer.rs         (11,183 줄!)  <- AGENTS.md 가 "800줄 초과 시 분리" 명시
|   |-- footer.rs                (2082 줄)
|   |-- mod.rs                   (3000 줄)
|   |-- chat_composer/           (서브모듈)
|   |-- approval_overlay.rs
|   |-- request_user_input/
|   |-- mentions_v2/
|   +-- ...
|-- chatwidget/       (서브모듈, lifecycle/hooks/mcp_startup/...)
|-- resume_picker/
|-- markdown_render/
|-- streaming/
|-- exec_cell/
|-- history_cell/
|-- keymap_setup/
|-- snapshots/        (insta 스냅샷, UI 회귀 테스트)
|-- notifications/
|-- status/
|-- render/           (highlight::highlight_bash_to_lines, renderable::Renderable)
|-- pets/             (ascii_animation.rs)
|-- public_widgets/   (IDE에서 재사용)
|-- onboarding/
+-- tests/            (5,653 줄 in app/tests.rs)
```

### 4.3 Render Loop / 상태 관리

`App` 구조체 (`tui/src/app.rs:1363` 줄) 가 `tokio::select!` 으로 (1) terminal event, (2) app event sender, (3) `codex_thread` 의 next_event, (4) background jobs 를 fan-in. **단일 Mutex + tokio::mpsc + watch channel** 조합 (`app_event_sender.rs`, `app_event.rs`).

`lib.rs:1-4` 가 명시한 거버넌스:

```rust
// Forbid accidental stdout/stderr writes in the *library* portion of the TUI.
// The standalone `codex-tui` binary prints a short help message before the
// alternate-screen mode starts; that file opts-out locally via `allow`.
#![deny(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::disallowed_methods)]
```

`AGENTS.md:122-152` TUI 스타일 컨벤션 (요약):

```text
- "text".into()            <- 짧을 땐
- "text".red().dim()       <- Stylize trait
- vec![..].into()          <- 여러 span
- Line::from(spans)        <- 타입 명시 필요할 때
- textwrap::wrap            <- plain string
- tui::wrapping::word_wrap_lines  <- ratatui Line
- ratatui::Stylize trait 우선 (Style/Span::styled 남용 금지)
- .white() 금지 (디폴트 전경색 우선)
```

**Hard rules** (AGENTS.md:43-55):

- 모듈 < 500 LoC (test 제외)
- 파일 ≈ 800 LoC 넘으면 새 모듈로 분리
- **위반 시 high-touch 파일** (AGENTS.md 가 직접 이름 명시): `app.rs`, `bottom_pane/chat_composer.rs` (11K 줄), `footer.rs`, `chatwidget.rs`, `bottom_pane/mod.rs`. "여기 더 붙이지 마라" 표시.
- `chat_composer.rs` 가 11K 줄인 건 **기술 부채** - AGENTS.md 가 "Don't add new standalone methods to chatwidget.rs unless trivial; prefer new modules" 라고 못 박은 상태.

### 4.4 키 바인딩

`keymap.rs` + `keymap_setup/`. vim / emacs / default 3 모드. `tui_keymap.toml` 로 커스터마이즈. 클립보드 paste burst handling (`bottom_pane/paste_burst.rs`).

### 4.5 TUI 디버깅 도구

`md-events` 15줄 바이너리 (`tui/src/bin/md-events.rs:1`):

```rust
use std::io::Read;
use std::io::{self};

fn main() {
    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {err}");
        std::process::exit(1);
    }
    let parser = pulldown_cmark::Parser::new(&input);
    for event in parser {
        println!("{event:?}");
    }
}
```

`stdin -> pulldown-cmark -> Debug print` 만 하는 극단적 단순 도구지만, "markdown renderer 가 어떻게 토큰화하는지" 디버깅할 때 golden 도구로 쓰인다. **Cargo bin 타겟** 으로 빌드 (`[[bin]] name = "md-events" path = "src/bin/md-events.rs"`).

### 4.6 화면 분할 (Bottom Pane)

```text
+-----------------------------------------------------+
|  Header (현재 모델, git branch, 작업 디렉토리)         |
+-----------------------------------------------------+
|  History Cell Stream (스크롤)                          |
|   - UserMessage, AssistantMessage, ExecCell,          |
|     DiffCell, ToolCallCell, ApprovalRequest,         |
|     McpServerElicitationForm, ReasoningCell, ...      |
+-----------------------------------------------------+
|  ChatComposer (입력 + 히스토리 + 슬래시커맨드)         |
|   - Mentions v2 (@-mentions)                          |
|   - Paste burst buffering                            |
|   - Text wrapping (word_wrap_lines)                  |
+-----------------------------------------------------+
|  Footer (모델, 토큰 사용량, 힌트)                       |
+-----------------------------------------------------+
|  (Optional) Approval overlay / Picker / Modal        |
+-----------------------------------------------------+
```

`bottom_pane/chat_composer.rs` 11K 줄 안에 텍스트 입력 + 히스토리 + 슬래시커맨드 popup + 멘션 코덱 + paste burst 가 다 들어있다. AGENTS.md 가 다음을 못 박음:

```text
- chatwidget.rs: Avoid adding new standalone methods to .../chatwidget.rs
  unless the change is trivial; prefer new modules/files and keep
  chatwidget.rs focused on orchestration.
- When extracting code from a large module, move the related tests and
  module/type docs toward the new implementation so the invariants stay
  close to the code that owns them.
```

### 4.7 TUI 테스트

- **insta snapshot** (`tui/src/snapshots/`) - UI 가시 출력 회귀. AGENTS.md:170-188 가 "any change that affects user-visible UI must include insta snapshot coverage" 라고 못 박음.
- **TestBackend** (`tui/src/test_backend.rs`) - ratatui headless 렌더링
- **pretty_assertions::assert_eq** (AGENTS.md:195) - 더 읽기 좋은 diff
- 5,653 줄의 `app/tests.rs` - 통합 시나리오

CI: `.github/workflows/rust-ci.yml`, `rust-ci-full-nextest-platform.yml` (3 OS matrix), `rust-release-argument-comment-lint.yml`.

---

## §5 LLM 통합

### 5.1 Provider 추상화

`codex-rs/model-provider/` 가 1차 어댑터. 주요 provider 별도 모듈:

```text
model-provider/src/
|-- lib.rs
|-- provider.rs        # ProviderInfo / WireApi / EnvKey
|-- bearer_auth_provider.rs
|-- auth.rs
|-- models_endpoint.rs
+-- amazon_bedrock/    # AWS Bedrock 어댑터 (sub-module)
```

`codex-rs/codex-api/` 가 Responses API wire 레벨 wrapper. `codex-rs/ollama/` `codex-rs/lmstudio/` 가 OSS provider, `codex-rs/responses-api-proxy/` 가 외부 proxy 데몬 (사용자가 자체 API proxy 운영 시).

기본 auth 흐름:
- **ChatGPT OAuth** (`chatgpt/`): `codex login --device-auth` 또는 ChatGPT 계정 로그인. device code + PKCE. 토큰은 `keyring-store/` (OS keychain) 또는 `~/.codex/auth.json`.
- **API key**: `codex login --with-api-key` (stdin 에서 읽음). 환경변수 `OPENAI_API_KEY`.
- **OSS**: ollama, lmstudio, 또는 임의 OpenAI-compatible endpoint. `config.toml` 의 `[model_providers.<name>]` 섹션.

### 5.2 Streaming / Tool Calling / Token 추적

**Client** (`codex-rs/core/src/client.rs` + `client_common.rs`):
- `ModelClient` 가 API 호출 1번을 추상화
- `Prompt` (in) -> `ResponseStream` (out) - SSE streaming
- `ResponseEvent` (in) -> `Event` (out) - 도메인 이벤트

**도구 호출 프로토콜**:
- 표준 OpenAI function calling (구 Responses API `tools` 배열)
- `codex-rs/tools/` 가 JSON Schema -> tool registration: `tool_spec.rs`, `tool_definition.rs`, `tool_discovery.rs`, `tool_executor.rs`, `tool_payload.rs`, `tool_call.rs`, `tool_config.rs`, `dynamic_tool.rs`
- `apply-patch/` (자체) 가 git-style unified diff 포맷의 tool format. 모델이 diff 를 출력하면 apply-patch 가 파싱/적용.

**토큰 추적**:
- `client.rs` 가 `X_RESPONSESAPI_INCLUDE_TIMING_METRICS_HEADER` (외부), `X_CODEX_TURN_METADATA_HEADER` (turn 메타) 헤더 전송
- `ResponseEvent::RateLimits` 가 RPM / TPM 정보 전달
- `TokenUsage` 가 prompt / completion / cached / reasoning / total 분리
- TUI footer 가 매 turn 끝에 `token_usage.to_string()` 표시
- 모델 컨텍스트 6개 규칙 (AGENTS.md:81-90) - 1K 토큰 초과 시 P0 수동 리뷰, 10K/item hard cap

### 5.3 응답 API

Codex 는 **OpenAI Responses API** 를 우선 사용 (`WireApi::Responses`). Chat Completions API 도 fallback 가능. `codex-rs/responses-api-proxy/` 가 데몬 형태의 proxy (원격 환경). `responses-retry.rs` 가 backoff + idempotency 처리.

**에러 처리** (`codex-rs/protocol/src/error.rs`):
- `CodexErr` enum: `RateLimitExceeded`, `Stream`, `InvalidRequest`, `UsageLimitReached`, `LandlockSandboxExecutableNotProvided`, `UnsupportedOperation`, `Fatal`, ...
- `X_CODEX_INSTALLATION_ID_HEADER` (`codex-core` 에서 정의) - telemetry correlation

### 5.4 Responses API Item 종류 (`protocol/src/items.rs`)

```text
Message  (user / assistant message)
Reasoning
FunctionToolCall
FunctionCallOutput
CustomToolCall
ImageGenerationCall
WebSearchCall
McpToolCall
LocalShellCall
Compaction
```

거의 모든 Responses API 스펙을 1:1 매핑. type-safe serde.

---

## §6 도구/스킬 시스템

### 6.1 도구 등록 메커니즘

`codex-rs/tools/` 디렉토리가 도구 정의/실행의 전부. 주요 파일:

```text
tools/src/
|-- lib.rs
|-- mod.rs
|-- spec.rs / spec_plan.rs
|-- tool_spec.rs / spec_plan_tests.rs
|-- tool_definition.rs
|-- tool_discovery.rs
|-- tool_executor.rs
|-- tool_payload.rs
|-- tool_call.rs
|-- tool_config.rs
|-- tool_output.rs
|-- dynamic_tool.rs       # runtime 등록 도구
|-- code_mode.rs          # 코드 실행 모드
|-- function_call_error.rs
|-- image_detail.rs       # 이미지 디테일 처리
|-- json_schema.rs        # JSON Schema 검증
|-- mcp_tool.rs           # MCP 도구 어댑터
+-- request_plugin_install.rs
```

`tools/handlers/` (codex-rs/core/src/tools/handlers/) 가 도구별 handler:
- `shell.rs`, `unified_exec.rs` (REPL 형식 셸)
- `apply_patch.rs` (diff 적용)
- `read_file.rs`, `list_dir.rs`
- `mcp_tool.rs` (MCP 어댑터)
- `multi_agents.rs` (서브에이전트 디스패치)
- `goal/create_goal.rs`, `goal/update_goal.rs` (목표 관리)
- `network_approval.rs` (네트워크 접근)
- `sandboxing.rs` (샌드박스 정책 적용)
- `hosted_spec.rs` (호스티드 도구)

### 6.2 내장 도구 목록 (대표)

| 도구 | 용도 | 비고 |
| --- | --- | --- |
| `shell` / `bash` | 셸 명령 실행 | unified_exec 와 통합, sandbox 자동 적용 |
| `apply_patch` | git-style diff 적용 | 자체 포맷 (core/prompt_with_apply_patch_instructions.md) |
| `read_file` | 파일 읽기 | 라인 range, 토큰 budget |
| `list_dir` | 디렉토리 조회 | glob 패턴 |
| `grep_files` / `code_search` | 코드 검색 | ripgrep (rg) 의존 |
| `web_search` | 웹 검색 | OpenAI native tool |
| `update_plan` | 작업 계획 | UI plan mode |
| `request_user_input` | 사용자 질문 | 4 옵션 멀티셀렉트 |
| `list_mcp_resources` / `read_mcp_resource` | MCP 리소스 | OAuth elicitation |
| `create_goal` / `update_goal` | goal-tracking | token_budget, time_used_seconds |
| `multi_agents` | 서브에이전트 dispatch | role + status |
| `code_mode` | sandboxed JS 실행 | V8 (vendored) |
| `image_generation_call` | 이미지 생성 | DALL-E / gpt-image-1 |

### 6.3 도구 / 권한 / 샌드박싱

- **권한 모델** (`protocol/src/permissions.rs`): `AskForApproval::{OnRequest, OnFailure, Never, Untrusted}` 4단계. `codex exec --ask-for-approval never` 식.
- **Exec policy** (`execpolicy/`, `execpolicy-legacy/`): Starlark 룰셋 (`starlark 0.13.0` dep) 으로 명령별 화이트리스트/블랙리스트. `codex execpolicy check` CLI 로 테스트. `execpolicy.md` 문서 있음.
- **샌드박스** (`sandboxing/`): `SandboxManager` 가 `SandboxType::{MacosSeatbelt, LinuxLandlock, LinuxBwrap, Windows}` 4종 dispatch. policy 가 `permission_profile` (FS + network) 둘 다 제어.

### 6.4 Skills 시스템

`codex-rs/skills/` 가 마크다운 기반 스킬 로더. 디렉토리 트리:

```text
skills/src/
|-- lib.rs
|-- manager.rs / manager_tests.rs
|-- loader.rs / loader_tests.rs
|-- discovery.rs         # 스킬 발견
|-- system.rs            # system 스킬 (codex-rs/core-skills 와 연동)
|-- skill_instructions.rs
|-- render.rs
|-- model.rs
|-- remote.rs
|-- mention_counts.rs
|-- injection.rs / injection_tests.rs
|-- invocation_utils.rs / invocation_utils_tests.rs
+-- manifest.rs
```

`core/src/skills.rs` 가 `SkillsManager` (싱글톤), `core-skills/` 가 built-in 스킬 (마크다운). `core-plugins/` 가 plugin 번들 (zip / tarball / 디렉토리). user 디렉토리 + project 디렉토리 양쪽에서 발견.

스킬 호출은 **@-mention** (코드 내 `@skill_name`) 또는 **Slash command** 로 트리거. Mention syntax (`core/src/mention_syntax.rs`) 가 `@skill`, `@plugin`, `@tool` 3가지 sigil 정의.

### 6.5 Plugin 시스템

`codex-rs/plugin/`:

```text
plugin/src/
|-- lib.rs
|-- assets
|-- catalog.rs           # 마켓플레이스 카탈로그
|-- extension.rs         # extension trait
|-- registry.rs
|-- state.rs
|-- lib.rs
|-- provider/
|   +-- provider.rs      # provider trait
|-- capabilities/        # capability declaration
|-- contributors/        # lifecycle contributor
+-- ...
```

`codex-rs/ext/extension-api/` 가 host API:

```text
ext/extension-api/src/
|-- lib.rs
|-- capabilities.rs
|-- contributors.rs      # ThreadLifecycleContributor, ToolCallInterceptor
+-- state.rs
```

`ext/{goal,guardian,image-generation,memories,skills,web-search}/` 가 1st-party extension 구현.

`ext/extension-api/notes.md` 가 host API 디자인 rationale.

### 6.6 MCP (Model Context Protocol)

`codex-rs/codex-mcp/` (클라이언트), `codex-rs/mcp-server/` (서버):

- 클라이언트: `mcp_connection_manager.rs` (도구/리소스 변경 mutation 단일 entry point), `mcp_tool_call.rs`, `mcp_tool_exposure.rs`, `mcp_skill_dependencies.rs`, `mcp_openai_file.rs`.
- 서버: stdio JSON-RPC, `tools/list`, `tools/call`, `prompts/list`, `resources/list`, `resources/read`, sampling 지원.
- AGENTS.md:32 - "When working with MCP tool calls, prefer using `codex-mcp/src/mcp_connection_manager.rs` to handle mutation of tools and tool calls. Aim to minimize the footprint of changes and leverage existing abstractions rather than plumbing code through multiple levels of function calls."
- `rmcp 1.7.0` (workspace dep, official Rust MCP SDK)
- OAuth: `mcp_tool_approval_templates.rs` 가 elicitation form template 관리

---

## §7 컨텍스트 관리

### 7.1 모델 컨텍스트 6개 규칙 (AGENTS.md:81-90)

이 규칙은 **Codex 의 가장 중요한 거버넌스 문서** 다. 모델 inference 요청에 들어가는 모든 context 는 다음 6가지를 반드시 지켜야 한다:

```text
1. No history rewrite - the context must be built up incrementally.
2. Avoid frequent changes to context that cause cache misses.
3. No unbounded items - everything injected in the model context must
   have a bounded size and a hard cap.
4. No items larger than 10K tokens.
5. Highlight new individual items that can cross >1k tokens as P0.
   These need an additional manual review.
6. All injected fragments must be defined as structs in `core/context`
   and implement ContextualUserFragment trait
```

해석:
- **#1 (no rewrite)**: turn 끝난 user message 를 나중에 수정하지 않음. cache hit 을 위해선 turn boundary 가 안정적이어야 함.
- **#2 (cache 친화)**: 컨텍스트를 자주 재구성하지 말 것. SSE 응답의 prompt token 부분이 prompt-cache key 역할을 하므로, 순서/내용을 자주 바꾸면 cache miss.
- **#3 (bounded)**: 모든 주입 fragment 는 명시적 max size + hard cap.
- **#4 (10K/item)**: 단일 fragment 의 토큰이 10K 넘으면 안 됨. `message-history` 의 `MAX_OUTPUT_TOKENS = 10_000` (`unified_exec/mod.rs:69`), `GUARDIAN_MAX_MESSAGE_TRANSCRIPT_TOKENS = 10_000` / `GUARDIAN_MAX_TOOL_TRANSCRIPT_TOKENS = 10_000` (`core/src/guardian/mod.rs:54-55`) 가 그 예.
- **#5 (1K P0)**: 새 fragment 가 1K 토큰 넘으면 PR 본문에 P0 라벨 + 수동 리뷰 필수.
- **#6 (struct in core/context)**: 모든 fragment 는 `core/src/context/` 에 정의 + `ContextualUserFragment` trait 구현. 일관성 + audit 가능.

### 7.2 파일 읽기 전략

- `codex-rs/file-search/` 가 파일 시스템 검색 (ripgrep / ignore crate 기반)
- `codex-rs/file-watcher/` 가 inotify/FSEvents 파일 변경 감지
- `apply-patch` 도구: 모델이 unified diff 출력 -> parser 가 적용. 라인 컨텍스트 + before/after.
- `list_dir` 도구: 디렉토리 트리 출력
- `read_file` 도구: 라인 range 지원, 토큰 budget 자동 적용

### 7.3 Repo 인덱싱 / RAG

Codex 는 **별도 RAG/임베딩 인덱싱을 하지 않는다**. 대신 (1) ripgrep, (2) file_search, (3) `core-skills` (markdown instruction), (4) `agents_md.rs` (저장소별 AGENTS.md 자동 로드) 로 lazy context injection. aider 의 repomap (PageRank) 같은 정적 그래프는 **없음**.

대신 `core/src/context/` 의 **struct + trait** 시스템이 fragment 단위로 명시적 inject. `contextual_user_message.rs` 가 가장 핵심.

### 7.4 토큰 예산 & 요약

- **TokenUsage** (`tui/src/token_usage.rs`): prompt, completion, cached, reasoning, total 분리 추적
- **Compact** (`core/src/compact.rs`): 컨텍스트 압축. `compact_remote.rs`, `compact_remote_v2.rs` 가 server-side compaction. `PreCompactRequest` / `PostCompactRequest` hook 으로도 트리거.
- **TruncationPolicy** (`core/src/tools/handlers/unified_exec.rs:63`): `Tokens(10_000)` 기본. `codex exec --output-token-budget N` 으로 override.

### 7.5 잘라내기 알고리즘 (예: shell output)

`message-history/src/lib.rs` 의 `enforce_history_limit` (`lib.rs:189-262`) - **line-by-line trim** with hard cap. 0.8 soft cap ratio. 핵심 의사코드:

```text
input: max_bytes, file (mutable, fd with O_APPEND, mode 0o600)
1. read file metadata -> current_len
2. if current_len <= max_bytes: return
3. enumerate line_lengths via BufReader::read_line
4. trim_target = max(soft_cap (0.8 * max_bytes), newest_entry_len)
5. drop oldest lines until current_len <= trim_target
6. seek(drop_bytes), read_to_end (tail), file.set_len(0), write tail
```

`max_bytes` 는 config.toml 의 `history.max_bytes` 에서 옴. `HistoryPersistence::None` 이면 비활성화. `HistoryPersistence::SaveAll` (default) 이면 활성.

### 7.6 핵심 Context Fragments (`core/src/context/`)

총 30+ struct. `ContextualUserFragment` trait 구현:

- `user_instructions.rs` - 시스템 지시 (gpt-5.x prompt + AGENTS.md + skills)
- `user_shell_command.rs` - user 의 셸 명령
- `environment_context.rs` - cwd, sandbox policy, OS
- `permissions_instructions.rs` - 도구 권한 (AskForApproval)
- `available_skills_instructions.rs` - skill 목록 (mention 주입)
- `available_plugins_instructions.rs` - plugin 목록
- `collaboration_mode_instructions.rs` - 협업 모드 (Plan/Execute)
- `personality_spec_instructions.rs` - personality (예: "friendly", "pragmatic")
- `model_switch_instructions.rs` - 모델 변경 알림
- `network_rule_saved.rs` / `approved_command_prefix_saved.rs` - approval 후속
- `realtime_start_instructions.rs` / `realtime_end_instructions.rs` - 실시간 컨텍스트
- `subagent_notification.rs` - subagent 완료 알림
- `turn_aborted.rs` - turn 중단 cleanup
- `guardian_followup_review_reminder.rs` - 자동 거버넌스 리뷰

`mod.rs` 가 이들을 turn context 에 mount.

---

## §8 세션 영속화

### 8.1 두 가지 영속화 시스템

Codex 는 **두 종류의 영속 저장소** 를 동시에 사용한다:

| 시스템 | 위치 | 포맷 | 용도 |
| --- | --- | --- | --- |
| **Message History** | `~/.codex/history.jsonl` | JSONL (1 record = 1 line) | 모든 세션의 user prompt 통합 히스토리 (resume picker 용) |
| **Rollout** | `~/.codex/sessions/rollout-<ISO>-<UUID>.jsonl` | JSONL | **세션 단위** 전체 transcript (재개 가능) |
| **State DB** | `~/.codex/state.db` | **SQLite** (sqlx) | thread metadata, agent jobs, goals, memories, telemetry, audit |
| **Log DB** | `~/.codex/log.db` | SQLite | telemetry 전용 (filter) |

### 8.2 Message History (`codex-message-history` crate)

`codex-rs/message-history/src/lib.rs` (437 줄). 핵심 디자인:

**저장 위치**: `~/.codex/history.jsonl` (HISTORY_FILENAME, lib.rs:46)

**포맷** (lib.rs:6-9 doc comment):
```json
{"session_id":"<uuid>","ts":<unix_seconds>,"text":"<message>"}
```

**핵심 동시성 안전성** (lib.rs:11-15):
```text
To minimize the chance of interleaved writes when multiple processes are
appending concurrently, callers should *prepare the full line* (record +
trailing `\n`) and write it with a **single `write(2)` system call** while
the file descriptor is opened with the `O_APPEND` flag. POSIX guarantees
that writes up to `PIPE_BUF` bytes are atomic in that case.
```

**Lock 전략** (lib.rs:154-180):
- `tokio::task::spawn_blocking` 으로 `std::fs` blocking 작업 위임
- `File::try_lock()` (BSD flock-style) 으로 **advisory exclusive lock**
- `MAX_RETRIES = 10` (lib.rs:52), `RETRY_SLEEP = 100ms` (lib.rs:53) 으로 backoff

**권한** (lib.rs:138-144, 301-313):
```rust
#[cfg(unix)]
{
    options.append(true);
    options.mode(0o600);   // rw------- owner only
}
```

`ensure_owner_only_permissions` (lib.rs:301-313) 가 열 때마다 `0o600` 아닌 경우 chmod. Windows 는 no-op.

**History Soft Cap** (lib.rs:50, 264-270):
```rust
const HISTORY_SOFT_CAP_RATIO: f64 = 0.8;  // hard cap 넘으면 80% 로 trim

fn trim_target_bytes(max_bytes: u64, newest_entry_len: u64) -> u64 {
    let soft_cap_bytes = ((max_bytes as f64) * HISTORY_SOFT_CAP_RATIO)
        .floor()
        .clamp(1.0, max_bytes as f64) as u64;
    soft_cap_bytes.max(newest_entry_len)
}
```

**Lookup** (lib.rs:294-417): `(log_id, offset)` -> `Option<HistoryEntry>`. `log_id` 는 Unix inode, Windows 는 creation time. inode 가 바뀌면 (파일 rotate) `None`. shared lock (`try_lock_shared`) 으로 read.

### 8.3 Rollout (`codex-rollout` crate)

`codex-rs/rollout/src/recorder.rs` (1,821 줄). 세션 단위 전체 transcript 저장.

**저장 위치**: `~/.codex/sessions/rollout-2025-05-07T17-24-21-<UUID>.jsonl` (recorder.rs:71)

**포맷** (recorder.rs:66-72 doc comment):
```text
Rollouts are recorded as JSONL and can be inspected with tools such as:
$ jq -C . ~/.codex/sessions/rollout-2025-05-07T17-24-21-*.jsonl
$ fx ~/.codex/sessions/rollout-2025-05-07T17-24-21-*.jsonl
```

**트랜잭션 모델** (recorder.rs:74-79):
```rust
pub struct RolloutRecorder {
    tx: Sender<RolloutCmd>,
    writer_task: Arc<RolloutWriterTask>,
    pub(crate) rollout_path: PathBuf,
}
```

`mpsc` 기반 비동기 writer. caller 는 `tx.send(RolloutCmd::AddItems(...))` 으로 enqueue. writer task 가 `mpsc` consume + file append + flush. **단일 writer task** 가 I/O 직렬화 -> 동시성 안전.

**Create vs Resume** (recorder.rs:82-96):
```rust
pub enum RolloutRecorderParams {
    Create {
        conversation_id: ThreadId,
        forked_from_id: Option<ThreadId>,
        parent_thread_id: Option<ThreadId>,
        source: SessionSource,
        thread_source: Option<ThreadSource>,
        base_instructions: BaseInstructions,
        dynamic_tools: Vec<DynamicToolSpec>,
        multi_agent_version: Option<MultiAgentVersion>,
    },
    Resume {
        path: PathBuf,
    },
}
```

**Resume/Fork** (cli/src/main.rs:305-376): `codex resume [<SESSION_ID>] [--last] [--all]` 또는 `codex fork`. `--last` 면 가장 최근 세션 자동 선택. `--all` 면 cwd 필터 무시 + CWD 컬럼 표시. `--include-non-interactive` 면 비대화형 세션도 picker 에 포함.

**State DB** (`rollout/src/state_db.rs`, 679 줄): `state_db.rs` 가 **SQLite (sqlx)** 기반 인덱스. thread metadata, agent jobs, goals, memories 테이블. `recorder.rs` 와 별도 crate 인데 이는 state DB 가 rollout 외 telemetry / memories / remote-control 도 저장하므로.

### 8.4 Session Archive

`codex archive <id-or-name>` / `codex unarchive <id-or-name>` 가 세션 hide/show. `find_archived_thread_path_by_id_str`, `ARCHIVED_SESSIONS_SUBDIR` (rollout/lib.rs 재export). `session_archive_commands.rs` 가 TUI 진입점.

### 8.5 Resume Picker

`tui/src/resume_picker/` 가 TUI 의 세션 선택 UI. `--last` 모드는 picker 없이 직행.

---

## §9 확장 시스템

### 9.1 Hooks (가장 큰 확장 지점)

`codex-rs/hooks/` (4.3K LOC). 10 이벤트, 2 포맷(command / prompt / agent) - 현재는 command 만 지원 (engine/dispatcher.rs:468-548 에서 `HookHandlerType::Command` 만 실행, `Prompt {}` 와 `Agent {}` 는 "not supported yet" warning).

**10 이벤트 목록** (hooks/src/lib.rs:19-30):
```rust
pub const HOOK_EVENT_NAMES: [&str; 10] = [
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "SessionStart",
    "UserPromptSubmit",
    "SubagentStart",
    "SubagentStop",
    "Stop",
];
```

**Matcher 가 적용되는 8 이벤트** (lib.rs:37-46): 위에서 `UserPromptSubmit`, `Stop` 제외. 매처 regex 가 `Bash` / `^Edit$` / `apply_patch|Write|Edit` / `*` 등.

**Hook Handler 형태** (`codex-config/src/hook_config.rs`):
```rust
pub enum HookHandlerConfig {
    Command { command: String, command_windows: Option<String>,
              timeout_sec: Option<u64>, async_: bool,
              status_message: Option<String> },
    Prompt {},   // not yet
    Agent {},    // not yet
}
```

### 9.2 Hook 발견 (Discovery) - `engine/discovery.rs` (1,086 줄)

`discover_handlers(config_layer_stack, plugin_hook_sources, ...)` 가:
1. `ConfigLayerStack` 의 모든 layer 순회 (System / User / Project / MDM / EnterpriseManaged / SessionFlags / LegacyManagedConfigTomlFromFile / LegacyManagedConfigTomlFromMdm)
2. 각 layer 에서 (a) `hooks.json` 로드, (b) `config.toml` 의 `[hooks]` 섹션 로드 (`load_toml_hooks_from_layer`). 둘 다 있으면 warning
3. 각 layer 마다 `hook_metadata_for_config_layer_source` 가 `HookSource::{System, User, Project, Mdm, CloudManagedConfig, ...}` 결정
4. Plugin hook source (`PluginHookSource`) 도 추가 (env `PLUGIN_ROOT` / `CLAUDE_PLUGIN_ROOT` / `PLUGIN_DATA` 주입)
5. **Trust status** 계산: `Managed` / `Trusted` (hash match) / `Modified` (hash mismatch) / `Untrusted`. `is_managed` 면 무조건 실행, 아니면 trusted_hash 비교 + `bypass_hook_trust` 옵션
6. `display_order` 순서 유지
7. **Untrusted + bypass=false 이면 실행 안 함**, entry 만 노출 (UI 에서 사용자가 enable 가능)

**Hash 정규화** (discovery.rs:556-580): `command_hook_hash` 가 normalized config-derived identity 의 SHA. hooks.json 과 config.toml 의 등가 hook 이 같은 hash 로 수렴.

### 9.3 Hook Dispatch - `engine/dispatcher.rs` (447 줄)

`select_handlers` 가 매치 + `select_handlers_for_matcher_inputs` (alias 고려). **alias 중복 1회만 실행** (dispatcher.rs:42-43):
```text
Check each configured handler once, even when several compatibility names
match the same regex. A hook like `apply_patch|Write|Edit` should run a
single time for one tool call, not once per matching alias.
```

`execute_handlers` 가 `FuturesUnordered` 로 동시 실행 (parallel). `parse` 함수로 `ParsedHandler<T>` 변환. **`HookScope::{Thread, Turn}`** (dispatcher.rs:142-154).

### 9.4 Hook 실행 - `engine/command_runner.rs` (135 줄)

`run_command` 가 stdin/JSON 입력 + timeout + 환경변수 + cwd 설정. `output_parser.rs` (591 줄) 가 stdout/stderr 파싱. `schema_loader.rs` (150 줄) 가 hooks config 스키마 로드.

**Hook timeout** (discovery.rs:482): default 600 sec, min 1 sec. `bypass_hook_trust` 면 trust 검사 우회.

### 9.5 Plugin 시스템

`codex-rs/plugin/` + `ext/extension-api/`:

**Plugin 번들** (zip / tarball / 디렉토리) 에 포함:
- `manifest.json` - plugin ID, name, capabilities
- `hooks/...` - hook 정의
- `tools/...` - tool 정의
- `skills/...` - skill 마크다운

**Plugin lifecycle** (ext/extension-api/src/contributors.rs):
- `ThreadLifecycleContributor::on_thread_start`, `on_thread_idle`, `on_thread_resume` 등
- `ToolCallInterceptor` (tool call 가로채기)

**Marketplace** (ext/marketplace/):
- `startup_sync.rs` 가 부팅 시 원격 marketplace 와 sync
- `startup_remote_sync.rs` (원격)
- `marketplace_add.rs` / `marketplace_remove.rs` / `marketplace_upgrade.rs` 가 lifecycle
- `plugin_bundle_archive.rs` 가 번들 zip 처리
- `remote_bundle.rs` / `remote_legacy.rs` 가 HTTP fetch

### 9.6 Skills

별도 시스템이지만 plugin 의 부분집합. `core-skills/` 가 built-in, `skills/` 가 user-defined. 마크다운 1 파일 = 1 skill. `core/src/skills.rs` 가 `SkillsManager` (싱글톤), `injection.rs` 가 mention 시 inject.

### 9.7 MCP (이미 §6 에서 다룸)

MCP 서버는 사실상 **외부 확장**. stdio JSON-RPC, OAuth elicitation 지원. `codex mcp add <name> -- <command>` 로 등록. `codex mcp list` / `codex mcp remove` / `codex mcp login` (OAuth).

### 9.8 app-server JSON-RPC API

IDE 통합용 1st-class API. `codex app-server` 로 stdio 또는 WebSocket. **V1 (legacy) + V2 (active) 동시 지원** (`AGENTS.md:241-277`):

```text
- All active API development should happen in app-server v2.
- Follow payload naming consistently: *Params / *Response / *Notification.
- Expose RPC methods as <resource>/<method> (singular resource).
- Always camelCase on wire (#[serde(rename_all = "camelCase")]).
- Set #[ts(export_to = "v2/")] on v2 types.
- Never use skip_serializing_if on v2 payload fields (except explicit no-params requests).
- Use cursor pagination for list methods (cursor + limit, data + next_cursor).
- Every optional field in *Params must be #[ts(optional = nullable)].
```

**TypeScript 자동 생성**: `codex app-server generate-ts --out <DIR>` 로 `.ts` 클라이언트 산출. `app-server-test-client` crate 가 integration test client.

### 9.9 설정 로딩 순서 (Layered)

`codex-rs/config/src/loader/` 가 layered config. 우선순위 (낮->높음):

```text
1. compiled-in defaults
2. system requirements.toml (/etc/codex/requirements.toml)
3. system config.toml
4. user config.toml (~/.codex/config.toml)
5. project config.toml (<cwd>/.codex/config.toml)
6. profile configs (~/.codex/<profile>.config.toml)
7. MDM / enterprise-managed config
8. legacy managed_config.toml
9. session flags (-c / --enable / --disable)
```

`config_layer_source.rs` 가 source variant 정의. `merge.rs` 가 layer 간 merge. `profile_toml.rs` 가 `codex --profile` 처리. `state.rs` 가 최종 merged config 보존.

`AGENTS.md:31` 룰: ConfigToml 변경 시 `just write-config-schema` 로 `codex-rs/core/config.schema.json` 자동 재생성 필수.

---

## §10 빌드 & 배포

### 10.1 듀얼 빌드 시스템 (Cargo + Bazel)

```text
justfile (entry) -> codex-rs/justfile (cargo) + codex-rs/MODULE.bazel (Bazel)
```

**Cargo** (`codex-rs/Cargo.toml`):
- 94 워크스페이스 멤버
- `[workspace.dependencies]` 중앙 관리 (일부 crate 는 개별 `Cargo.toml` 의 `version.workspace = true` / `edition.workspace = true` / `license.workspace = true`)
- `[workspace.lints]` 30+ clippy deny 룰 (Cargo.toml:441-479)
- `[workspace.metadata.cargo-shear]` (Cargo.toml:483-490) - cargo-shear 가 못 찾는 의존성 무시 목록

**Bazel** (`MODULE.bazel`, `BUILD.bazel`):
- 모든 crate 마다 `BUILD.bazel` (총 90+ 파일, `find ... -name BUILD.bazel | wc -l` = 90)
- `rules_rust` (GitHub pinned) 사용
- **AGENTS.md:34-41 의 lockfile 룰**: Cargo dep 변경 시 `just bazel-lock-update` 로 `MODULE.bazel.lock` 갱신. `just bazel-lock-check` 로 drift 감지. CI 가 lockfile drift 검사.
- `include_str!` / `include_bytes!` / `sqlx::migrate!` 같은 compile-time file access 사용 시 crate 의 `BUILD.bazel` 의 `compile_data` / `build_script_data` / test data 갱신 필수 (AGENTS.md:39-41).

**Nix** (`flake.nix`, `flake.lock`): 재현 가능한 빌드 환경. `default.nix`.

### 10.2 Justfile (의사 진입점)

`/Users/yklee/repos/harness-refs/codex/justfile` (178 줄) 가 모든 빌드/테스트/포맷 커맨드 모음:

```text
just codex -- <args>             # cargo run --bin codex
just exec <args>                 # codex exec
just fmt                         # format.py (Rust + Python)
just fix -p <project>            # cargo clippy --fix --tests
just test                        # cargo nextest run --no-fail-fast
just test -p codex-tui           # crate 단위
just bench                       # cargo bench --workspace --bench '*'
just bench-smoke                 # cargo bench -- --test
just install                     # rustup show + cargo fetch
just app-server-test-client      # codex-app-server-test-client 빌드 + 실행
just file-search <args>          # codex-file-search
just write-config-schema         # config schema 갱신
just write-app-server-schema     # app-server schema 갱신
```

### 10.3 Profile & 산출물

`codex-rs/Cargo.toml:492-521`:

```toml
[profile.dev]
debug = "limited"

[profile.dev-small]      # dist 용: 디버그 없음 + symbol strip
inherits = "dev"
opt-level = 0
debug = "none"
strip = "symbols"

[profile.release]
lto = "thin"
split-debuginfo = "off"
strip = "symbols"          # npm CLI 에 임베드되므로 작게
codegen-units = 1          # issue #1411

[profile.ci-test]         # CI: 디스크 압박 줄임
debug = "limited"
inherits = "test"
opt-level = 0
```

**산출물 매트릭스** (README.md:44-54):
- macOS: `codex-aarch64-apple-darwin.tar.gz`, `codex-x86_64-apple-darwin.tar.gz`
- Linux: `codex-x86_64-unknown-linux-musl.tar.gz`, `codex-aarch64-unknown-linux-musl.tar.gz`
- **musl 정적 링크** (glibc 의존성 회피)
- 단일 바이너리 안에 모든 crate 가 link

### 10.4 Cross-platform 패키징

```text
macOS:   .tar.gz (직접 다운로드) / Homebrew cask / npm (코드 섀시 + 바이너리 unzip)
Linux:   .tar.gz (musl) / npm / apt (Debian) / rpm (Fedora) (스크립트: scripts/install)
Windows: PowerShell installer (irm https://chatgpt.com/codex/install.ps1 | iex)
```

**Install 스크립트** (`scripts/install/`, `scripts/build_codex_package.py`, `scripts/stage_npm_packages.py`): GitHub Release 의 platform tarball 을 fetch -> PATH 에 symlink.

**npm wrapper** (`codex-cli/` 디렉토리 = Node.js wrapper, `codex-rs` 와 별도): `npm install -g @openai/codex` 가 `codex` shim 스크립트 + platform 별 prebuilt 바이너리를 platform-specific optional dep 로 다운로드 (`@openai/codex-darwin-arm64`, `@openai/codex-linux-x64-musl` 등). 표준 npm 패키지 패턴.

**Self-update** (`update` 서브커맨드, cli/src/main.rs:1311-1318): `codex update` 가 자체 업데이트 트리거. `update_action.rs` 가 platform 별 업데이트 명령 생성 (PowerShell / sh / cmd). `pid_update_loop` (app-server-daemon) 가 데몬 detached 업데이트 루프.

### 10.5 Bazel 빌드 옵션

`rbe.bzl` (Remote Build Execution) - Google RBE 인프라 사용. `workspace_root_test_launcher.sh.tpl` / `workspace_root_test_launcher.bat.tpl` (Windows). `defs.bzl` 공유 매크로.

`.github/workflows/bazel.yml` 가 CI Bazel 빌드. `.github/workflows/rust-ci-full.yml` 가 3-OS matrix Cargo 빌드. `rust-ci-full-nextest-platform.yml` 가 nextest platform matrix.

---

## §11 테스트 & 품질

### 11.1 테스트 인프라

**Test runner**: `cargo nextest` (AGENTS.md:60-62 명시). `just test` 가 `RUST_MIN_STACK=8388608 cargo nextest run --no-fail-fast` + `just bench-smoke` 자동 호출. **`cargo test` 직접 사용 금지** ("Do not run `cargo test` directly. Use `just test`").

**Test 종류**:

1. **Unit tests** - crate 내부 (`#[cfg(test)] mod tests` 또는 별도 `*_tests.rs`)
2. **Integration tests** - `core/tests/common` (코드명 `core_test_support`), `mcp-server/tests/common` (코드명 `mcp_test_support`), `app-server/tests/common` (`app_test_support`)
3. **Snapshot tests** - `insta` (AGENTS.md:170-188). 특히 `tui/src/snapshots/`
4. **E2E / agent tests** - `core/suite` (AGENTS.md:103-114) 가 integration test pattern. **`test_codex` 로 test instance setup**. agent logic 변경 시 integration test 필수
5. **Smoke tests** - `just bench-smoke` 가 모든 bench target 1회 실행
6. **Mock SSE server** - `scripts/mock_responses_websocket_server.py` 가 Responses API WebSocket mock. CI 통합 테스트용

### 11.2 통합 테스트 패턴 (AGENTS.md:217-230)

```rust
let mock = responses::mount_sse_once(&server, responses::sse(vec![
    responses::ev_response_created("resp-1"),
    responses::ev_function_call(call_id, "shell", &serde_json::to_string(&args)?),
    responses::ev_completed("resp-1"),
])).await;

codex.submit(Op::UserTurn { ... }).await?;

let request = mock.single_request();
// assert using request.function_call_output(call_id) or request.json_body()
```

핵심 helper:
- `mount_sse_once` (preferred) vs `mount_sse_once_match` vs `mount_sse_sequence`
- `ResponseMock::single_request()` (1 POST) vs `ResponseMock::requests()` (모든 POST)
- `ResponsesRequest::body_json / input / function_call_output / custom_tool_call_output / call_output / header / path / query_param` (assertion helper)
- `ev_*` constructors + `sse(...)` builder
- `wait_for_event` (preferred) vs `wait_for_event_with_timeout`

### 11.3 테스트 모듈 조직 (AGENTS.md:158-167)

```rust
#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
```

별도 sibling file + `#[path = ...]` attribute. inline `mod tests { ... }` 는 기존 코드에서 옮기지 말 것 (규칙 적용 X), 새로 추가할 때만.

### 11.4 Test assertions

- `pretty_assertions::assert_eq` (AGENTS.md:195) - 더 읽기 좋은 diff
- **Deep equality 우선** (AGENTS.md:28) - 필드별 비교 < 객체 전체 비교
- `process environment` mutate 금지 (AGENTS.md:197) - 테스트에서 env set 대신 flag / dependency 주입

### 11.5 Test binary / fixture path

- `codex_utils_cargo_bin::cargo_bin("codex")` (AGENTS.md:201) - first-party binary spawn. Bazel runfiles 지원.
- `codex_utils_cargo_bin::find_resource!` (AGENTS.md:203) - fixture 파일 경로 (Bazel + Cargo 양립). `env!("CARGO_MANIFEST_DIR")` 직접 사용 금지 (Bazel 에서 깨짐).

### 11.6 CI Workflows (`.github/workflows/`)

```text
rust-ci.yml                    # main Rust CI
rust-ci-full.yml               # 3-OS matrix (ubuntu, macos, windows)
rust-ci-full-nextest-platform.yml  # nextest + 3-OS platform
rust-release.yml               # release 빌드 + GitHub Release
rust-release-windows.yml       # Windows installer
rust-release-zsh.yml           # zsh completion
rust-release-argument-comment-lint.yml  # argument comment lint
python-runtime-build.yml       # Python runtime build
python-runtime-release.yml     # Python runtime release
python-sdk-release.yml         # Python SDK release
rusty-v8-release.yml           # V8 vendor
sdk.yml                        # SDK CI
v8-canary.yml                  # V8 canary
ci.yml                         # generic CI
bazel.yml                      # Bazel build
cla.yml                        # CLA 봇
dependabot.yaml                # Dependabot
codespell.yml                  # spell check
issue-labeler.yml              # PR labeler
close-stale-contributor-prs.yml
issue-deduplicator.yml
```

CI 가 검사하는 것:
- Rust 3-OS matrix build + test (cargo nextest)
- Clippy (`just fix -p <project>`)
- rustfmt (`just fmt`)
- Snapshot diff (PR reviewer 가 accept)
- Bazel 빌드
- Python SDK / runtime 빌드 + 릴리즈
- Cargo deny (`deny.toml`)
- Blob size policy (`blob-size-policy.yml`)
- argument_comment_lint (`dotslash-argument-comment-lint-config.json`)

### 11.7 Lint 규율 (Cargo.toml:441-479)

`[workspace.lints.clippy]` 30+ deny 룰:

```text
await_holding_invalid_type = "deny"
await_holding_lock = "deny"
expect_used = "deny"
identity_op = "deny"
manual_clamp = "deny"
manual_filter = "deny"
manual_find = "deny"
manual_flatten = "deny"
manual_map = "deny"
manual_memcpy = "deny"
manual_non_exhaustive = "deny"
manual_ok_or = "deny"
manual_range_contains = "deny"
manual_retain = "deny"
manual_strip = "deny"
manual_try_fold = "deny"
manual_unwrap_or = "deny"
needless_borrow = "deny"
needless_borrowed_reference = "deny"
needless_collect = "deny"
needless_late_init = "deny"
needless_option_as_deref = "deny"
needless_question_mark = "deny"
needless_update = "deny"
redundant_clone = "deny"
redundant_closure = "deny"
redundant_closure_for_method_calls = "deny"
redundant_static_lifetimes = "deny"
trivially_copy_pass_by_ref = "deny"
uninlined_format_args = "deny"
unnecessary_filter_map = "deny"
unnecessary_lazy_evaluations = "deny"
unnecessary_sort_by = "deny"
unnecessary_to_owned = "deny"
unwrap_used = "deny"
```

`format!` 의 inline args, `?` over `try!()`, `redundant_closure_for_method_calls` 등도 deny.

`AGENTS.md:14-19` 가 **positional-literal call site convention** 추가:

```text
- Avoid bool or ambiguous `Option` parameters that force callers to write
  hard-to-read code such as `foo(false)` or `bar(None)`. Prefer enums, named
  methods, newtypes, or other idiomatic Rust API shapes when they keep the
  callsite self-documenting.
- When you cannot make that API change and still need a small positional-
  literal callsite in Rust, follow the `argument_comment_lint` convention:
  Use an exact /*param_name*/ comment before opaque literal arguments such
  as `None`, booleans, and numeric literals when passing them by position.
- The parameter name in the comment must exactly match the callee signature.
- You can run `just argument-comment-lint` to run the lint check locally.
```

즉, `foo(/*enable*/ true, /*mode*/ Mode::Strict)` 식 주석 강제. **`just argument-comment-lint`** 로 검사.

### 11.8 Async function in trait (AGENTS.md:21-27)

```text
- Discourage both #[async_trait] and #[allow(async_fn_in_trait)] in Rust traits.
- Prefer native RPITIT trait methods with explicit `Send` bounds on the
  returned future, as in `3c7f013f9735` / `#16630`.
- Preferred trait shape:
    fn foo(&self, ...) -> impl std::future::Future<Output = T> + Send;
- Implementations may still use `async fn foo(&self, ...) -> T` when they
  satisfy that contract.
- Do not use #[allow(async_fn_in_trait)] as a shortcut around spelling the
  future contract explicitly.
```

Rust 2024 edition + RPITIT. Send bound 명시. `#[async_trait]` 사용 금지.

---

## §12 보안

Codex 의 보안은 **defense in depth** 다. 3-티어 샌드박스 + 정책 엔진 + keyring + secret masking + network proxy.

### 12.1 Trifecta 샌드박스

| 플랫폼 | 백엔드 | 코드 | LOC |
| --- | --- | --- | --- |
| **macOS** | Seatbelt (`/usr/bin/sandbox-exec`) + .sbpl 정책 | `codex-rs/sandboxing/src/seatbelt.rs` | 745 |
| **Linux (native)** | Landlock + seccomp (kernel 5.13+) | `codex-rs/sandboxing/src/landlock.rs` | 105 + `linux-sandbox/` 1K |
| **Linux (compat)** | Bubblewrap (bwrap) | `codex-rs/sandboxing/src/bwrap.rs` + `codex-rs/bwrap/` (vendored) | 195 + 45 |
| **Windows** | Restricted token + Job Object + WFP + DACL | `codex-rs/windows-sandbox-rs/src/` | 1.4K |

**매니저** (`sandboxing/src/manager.rs` 372 줄): `SandboxType::{MacosSeatbelt, LinuxLandlock, LinuxBwrap, Windows}` dispatch. `compatibility_sandbox_policy_for_permission_profile` (sandboxing/src/lib.rs:13) 가 permission profile -> SandboxPolicy 변환.

### 12.2 macOS Seatbelt (sandboxing/src/seatbelt.rs:1-100)

```text
const MACOS_SEATBELT_BASE_POLICY: &str = include_str!("seatbelt_base_policy.sbpl");
const MACOS_SEATBELT_NETWORK_POLICY: &str = include_str!("seatbelt_network_policy.sbpl");
const MACOS_RESTRICTED_READ_ONLY_PLATFORM_DEFAULTS: &str =
    include_str!("restricted_read_only_platform_defaults.sbpl");

pub const MACOS_PATH_TO_SEATBELT_EXECUTABLE: &str = "/usr/bin/sandbox-exec";
```

핵심: **`/usr/bin/sandbox-exec` 만 사용** (seatbelt.rs:25-29 comment). `sandbox-exec` 가 PATH 상의 악성 버전으로 교체되는 것을 방어. `/usr/bin/sandbox-exec` 가 변조됐다면 attacker 가 이미 root 라는 가정.

**Network 정책** (seatbelt_network_policy.sbpl, 59 줄): proxy URL 의 loopback host/port 자동 추출 -> `network.inbound/outbound` rule 에 allow. `proxy_loopback_ports_from_env` (seatbelt.rs:43-76) 가 `HTTP_PROXY` / `HTTPS_PROXY` / `SOCKS_PROXY` / `NO_PROXY` 등 env vars 파싱.

`UnixDomainSocketPolicy` enum (seatbelt.rs:88-97): `AllowAll` vs `Restricted { allowed: Vec<AbsolutePathBuf> }` 명시적 분리 - **무시되는 상태 안 가짐** (주석: "Keep allow-all and allowlist modes disjoint so we don't carry ignored state").

### 12.3 Linux Landlock

`codex-rs/sandboxing/src/landlock.rs` (105 줄) + `codex-rs/linux-sandbox/` (vendored landlock C). `spawn_command_under_linux_sandbox` (core/src/lib.rs:49 reexport) 가 sandbox 적용 + child spawn.

Landlock = kernel-level FS access control (5.13+). seccomp-filter 와 결합 가능. `codex-rs/linux-sandbox/build.rs` 가 `seccompiler 0.5.0` (Cargo.toml:361) 으로 BPF filter 빌드.

### 12.4 Linux Bubblewrap (compat)

`codex-rs/bwrap/` 가 vendored bubblewrap (`vendor/bubblewrap/`) + Rust wrapper (45 줄 main.rs).

`bwrap/src/main.rs:1-29`:
```rust
#[cfg(all(target_os = "linux", bwrap_available))]
fn main() {
    unsafe extern "C" { fn bwrap_main(argc: libc::c_int, argv: *const *const c_char) -> libc::c_int; }
    let cstrings = std::env::args_os()
        .map(|arg| CString::new(arg.as_os_str().as_bytes())...)
        .collect::<Vec<_>>();
    let mut argv_ptrs = cstrings.iter().map(CStr::as_ptr).collect::<Vec<_>>();
    argv_ptrs.push(std::ptr::null());
    let exit_code = unsafe { bwrap_main(cstrings.len() as libc::c_int, argv_ptrs.as_ptr()) };
    std::process::exit(exit_code);
}
```

`build.rs:51-68` 가 C source 4개 (`bubblewrap.c`, `bind-mount.c`, `network.c`, `utils.c`) 를 `cc` crate 로 빌드 + `main` 심볼을 `bwrap_main` 으로 rename (`#define main bwrap_main`). **libcap pkg-config 필수**. CODEX_BWRAP_SOURCE_DIR env 로 외부 source override 가능.

### 12.5 Windows Sandbox (windows-sandbox-rs/src/, 1.4K LOC)

```text
lib.rs, env.rs, process.rs, token.rs, cap.rs, identity.rs, allow.rs,
deny_read_acl.rs, deny_read_state.rs, deny_read_resolver.rs,
resolved_permissions.rs, setup.rs, setup_error.rs, spawn_prep.rs,
acl.rs, workspace_acl.rs, hide_users.rs, wfp.rs, wfp_setup.rs,
logging.rs, audit.rs, dpapi.rs, winutil.rs, path_normalization.rs,
proc_thread_attr.rs, ssh_config_dependencies.rs, sandbox_utils.rs,
helper_materialization.rs, desktop.rs, elevated/ (sub-mod), conpty/
```

- `wfp` = Windows Filtering Platform (network policy)
- `dpapi` = Data Protection API (시크릿 암호화)
- `token` = Restricted token (privilege 제거)
- `wfp_setup` 가 부팅 시 1회 WFP 필터 설치
- `desktop` 가 GUI 호환 (DWM, immersive)

### 12.6 Network 정책

`network_policy_decision.rs` (106 줄) + `network_proxy_loader.rs` (393 줄, mtime-based reloader):

- `MtimeConfigReloader` 가 config.toml 의 mtime 변경 감지 -> 자동 reload
- `NetworkProxy` 가 `HTTP_PROXY` / `HTTPS_PROXY` 등 env vars 래핑
- `permission_profile.network` 가 allowlist / open / restricted 결정

### 12.7 권한 / 승인 모델

`protocol/src/permissions.rs` + `core/src/tools/context.rs`:

```text
AskForApproval::{OnRequest, OnFailure, Never, Untrusted}
```

- **OnRequest**: 매 도구 호출마다 사용자 확인
- **OnFailure**: 실패 시 (네트워크/FS 거부) 만 확인
- **Never**: 자동 실행 (CI)
- **Untrusted**: 신뢰 안 되는 source 의 도구는 무조건 OnRequest

`PermissionProfile` (FS allowlist + network policy 결합). `permission_compat.rs` (TUI) 가 v1/v2 호환.

### 12.8 시크릿 / Keychain

- `keyring-store/` (3.6, workspace dep `keyring 3.6`) - macOS Keychain, Linux Secret Service, Windows Credential Vault
- `secrets/` 가 secret masking (log 에 token 안 새도록)
- ChatGPT OAuth 토큰은 keyring 또는 `~/.codex/auth.json` (mode 0o600, keyring 우선)
- `codex login --with-api-key` 가 stdin 으로만 받음 (CLI 인자로 받지 않음 - history 노출 방지)

### 12.9 샌드박스 환경변수 규약 (AGENTS.md:8-10)

이 부분이 Codex 의 가장 까다로운 invariant:

```text
- Never add or modify any code related to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR`
  or `CODEX_SANDBOX_ENV_VAR`.
- You operate in a sandbox where `CODEX_SANDBOX_NETWORK_DISABLED=1` will be
  set whenever you use the `shell` tool. Any existing code that uses
  `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` was authored with this fact in
  mind. It is often used to early exit out of tests that the author knew
  you would not be able to run given your sandbox limitations.
- Similarly, when you spawn a process using Seatbelt (`/usr/bin/sandbox-exec`),
  `CODEX_SANDBOX=seatbelt` will be set on the child process. Integration
  tests that want to run Seatbelt themselves cannot be run under Seatbelt,
  so checks for `CODEX_SANDBOX=seatbelt` are also often used to early exit
  out of tests, as appropriate.
```

**의미**: AI 에이전트(Codex 자신) 가 `shell` tool 로 코드 수정할 때, sandbox 안에서 동작하므로 네트워크 차단됨. 따라서 `CODEX_SANDBOX_NETWORK_DISABLED=1` 분기로 네트워크 필요 테스트 skip. **새 코드에서 이 env var 를 추가/수정하지 말 것** (이미 special meaning 있음).

### 12.10 Audit / Logging

- `sandboxing/src/audit.rs` 가 seatbelt denial log
- `state/src/audit.rs` 가 state 변경 audit
- `windows-sandbox-rs/src/audit.rs` 가 Windows 이벤트 로그
- `state/src/log_db.rs` 가 telemetry (filter 적용)
- `tracing` + `tracing-opentelemetry` + `tracing-appender` 가 structured logging
- `otel/` 가 OpenTelemetry 통합
- `secrets/` 가 secret 자동 masking

### 12.11 Guardian 자동 거버넌스

`codex-rs/core/src/guardian/` (5 modules):

- `mod.rs` (176 줄) - `GUARDIAN_REVIEW_TIMEOUT = 90s`, `MAX_CONSECUTIVE_GUARDIAN_DENIALS_PER_TURN = 3`, `MAX_RECENT_AUTO_REVIEW_DENIALS_PER_TURN = 10`
- **Guardian 10K 토큰 캡**:
  ```rust
  const GUARDIAN_MAX_MESSAGE_TRANSCRIPT_TOKENS: usize = 10_000;
  const GUARDIAN_MAX_TOOL_TRANSCRIPT_TOKENS: usize = 10_000;
  const GUARDIAN_MAX_MESSAGE_ENTRY_TOKENS: usize = 2_000;
  const GUARDIAN_MAX_TOOL_ENTRY_TOKENS: usize = 1_000;
  const GUARDIAN_MAX_ACTION_STRING_TOKENS: usize = 16_000;
  const GUARDIAN_RECENT_ENTRY_LIMIT: usize = 40;
  ```
- **동작**: user 가 `on-request` 승인 요청 시 guardian sub-session 이 별도 모델로 planned action 평가. risk level / user_authorization / outcome (allow/deny) strict JSON 반환. **fail-closed on timeout/parse failure**.
- `GuardianRejectionCircuitBreaker` 가 연속 3회 deny 시 turn interrupt.

이는 **우리한테 가장 모범 사례**: 외부 모델로 권한 결정을 재평가하는 LLM-as-a-judge 패턴.

---

## §13 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

> **이 섹션이 우리한테 가장 중요.** `my_harness` 의 디자인 결정에 직접 인용 가능한 패턴들.

### ✅ 우리가 무조건 차야 할 패턴

#### 13.1 모델 컨텍스트 6개 규칙 (AGENTS.md:81-90)

```text
1. No history rewrite - the context must be built up incrementally.
2. Avoid frequent changes to context that cause cache misses.
3. No unbounded items - everything injected in the model context must
   have a bounded size and a hard cap.
4. No items larger than 10K tokens.
5. Highlight new individual items that can cross >1k tokens as P0.
6. All injected fragments must be defined as structs in `core/context`
   and implement ContextualUserFragment trait
```

**왜 차야 하나**: LLM inference cost / cache hit / context pollution / audit 모두를 잡는 한 줄 짜리 룰셋. 우리 `MiniMax.md` / `AGENTS.md` 에 **그대로** 박아도 된다.

**우리 적용 방안**:
- `my_harness/docs/MiniMax.md` 에 6개 규칙 + 우리 token budget (e.g. 8K/item) 명시
- `my_harness/src/agent/context/` (가칭) 디렉토리에 fragment struct + trait 강제
- PR template 에 "1K+ 토큰 fragment 추가 시 P0" 자동 라벨

#### 13.2 codex-core 비대화 방지 규율 (AGENTS.md:66-77)

```text
Over time, the `codex-core` crate has become bloated because it is the
largest crate, so it is often easier to add something new to `codex-core`
rather than refactor out the library code you need. So: resist adding
code to codex-core!

Particularly when introducing a new concept/feature/API, before adding
to `codex-core`, consider whether:
- There is an existing crate other than `codex-core` that is an
  appropriate place for your new code to live.
- It is time to introduce a new crate to the Cargo workspace for your
  new functionality. Refactor existing code as necessary to make this
  happen.

Likewise, when reviewing code, do not hesitate to push back on PRs that
would unnecessarily add code to `codex-core`.
```

**왜 차야 하나**: 새 crate 만드는 게 더 쉬울 때도 있는데, 관성으로 "원래 거대한 crate" 에 때려넣는 안티패턴. 우리 `my_harness` 도 `core` 가 1만 줄 넘으면 이 룰 적용.

**우리 적용 방안**:
- 우리 `AGENTS.md` 에 동일 룰 박기
- `utils/`, `protocol/`, `state/`, `plugin/`, `hooks/`, `sandboxing/`, `tools/` 식 crate 분리 강제

#### 13.3 800줄 변경 가이드 (AGENTS.md:114-121)

```text
Unless the change is mechanical the total number of changed lines should
not exceed 800 lines. For complex logic changes the size should be
under 500 lines. If the change is larger, explore whether it can be
split into reviewable stages and identify the smallest coherent stage
to land first. Base the staging suggestion on the actual diff,
dependencies, and affected call sites.
```

**왜 차야 하나**: PR 리뷰 피로 줄임 + stable main + 1 PR = 1 concept. 우리도 동일.

**우리 적용 방안**: PR template 의 "Diff size > 800 lines?" 체크박스 + CI 가 자동 계산 (의외로 쉬움 - `git diff --stat` + python 스크립트).

#### 13.4 모듈 < 500 LoC, 파일 < 800 LoC (AGENTS.md:43-55)

```text
- Target Rust modules under 500 LoC, excluding tests.
- If a file exceeds roughly 800 LoC, add new functionality in a new module
  instead of extending the existing file unless there is a strong
  documented reason not to.
- This rule applies especially to high-touch files that already attract
  unrelated changes, such as `codex-rs/tui/src/app.rs`, ...
- Avoid adding new standalone methods to .../chatwidget.rs unless the
  change is trivial; prefer new modules/files and keep chatwidget.rs
  focused on orchestration.
```

**왜 차야 하나**: `chat_composer.rs` 가 11K 줄로 폭주한 게 본인이 직접 언급. 우리가 빨리 도입할수록 좋다.

**우리 적용 방안**:
- 우리 `AGENTS.md` 에 동일한 룰 + high-touch 파일 명시 (e.g. `agent.rs`, `tui/app.rs`)
- PR 본문에 "이 파일 800줄 넘었는데 새 모듈 안 만든 이유" 자동 질문

#### 13.5 10K 토큰 캡 + Soft Cap 0.8 (message-history/src/lib.rs:50, 264-270 + guardian/mod.rs:54-58)

```rust
// message-history
const HISTORY_SOFT_CAP_RATIO: f64 = 0.8;
fn trim_target_bytes(max_bytes: u64, newest_entry_len: u64) -> u64 {
    let soft_cap_bytes = ((max_bytes as f64) * HISTORY_SOFT_CAP_RATIO)
        .floor().clamp(1.0, max_bytes as f64) as u64;
    soft_cap_bytes.max(newest_entry_len)
}

// guardian
const GUARDIAN_MAX_MESSAGE_TRANSCRIPT_TOKENS: usize = 10_000;
const GUARDIAN_MAX_TOOL_TRANSCRIPT_TOKENS: usize = 10_000;
const GUARDIAN_MAX_MESSAGE_ENTRY_TOKENS: usize = 2_000;
const GUARDIAN_MAX_TOOL_ENTRY_TOKENS: usize = 1_000;
```

**왜 차야 하나**: 가장 중요한 토큰 가드레일. 우리 history/rollout/guardian 모두에 동일 적용.

**우리 적용 방안**:
- `my_harness/src/persistence/history.rs` 에 동일 0.8 soft cap + 10K hard cap
- `my_harness/src/agent/transcript.rs` 에 guardian 10K 메시지/도구 캡
- `just lint-context` 스크립트: 모든 struct 가 10K 미만 자동 검사 (rustc proc-macro 또는 별도 validator)

#### 13.6 BSD flock + O_APPEND + atomic single write (message-history/src/lib.rs:11-15, 137-180)

```text
To minimize the chance of interleaved writes when multiple processes are
appending concurrently, callers should *prepare the full line* (record +
trailing `\n`) and write it with a **single `write(2)` system call** while
the file descriptor is opened with the `O_APPEND` flag. POSIX guarantees
that writes up to `PIPE_BUF` bytes are atomic in that case.
```

```rust
// 1. lock + 0o600 + O_APPEND
let mut options = OpenOptions::new();
options.read(true).write(true).create(true);
#[cfg(unix)] { options.append(true); options.mode(0o600); }
let mut history_file = options.open(&path)?;
ensure_owner_only_permissions(&history_file).await?;

// 2. spawn_blocking + try_lock + retry
tokio::task::spawn_blocking(move || -> Result<()> {
    for _ in 0..MAX_RETRIES {
        match history_file.try_lock() {
            Ok(()) => {
                history_file.seek(SeekFrom::End(0))?;
                history_file.write_all(line.as_bytes())?;
                history_file.flush()?;
                enforce_history_limit(&mut history_file, history_max_bytes)?;
                return Ok(());
            }
            Err(TryLockError::WouldBlock) => std::thread::sleep(RETRY_SLEEP),
            Err(e) => return Err(e.into()),
        }
    }
    ...
}).await??;
```

**왜 차야 하나**: 여러 agent 프로세스 (TUI, exec-server, app-server) 가 동시에 같은 history.jsonl 에 append 할 때 데이터 손상 방지. 우리도 multi-process 지원하면 반드시 필요.

**우리 적용 방안**:
- `my_harness/src/persistence/jsonl_append.rs` (가칭) - 위 패턴 그대로 copy
- O_APPEND + O_NONBLOCK (BSD lock) + spawn_blocking + 10 retry + 100ms backoff

#### 13.7 Cargo + Bazel 듀얼 빌드 (MODULE.bazel + Cargo.toml)

**왜 차야 하나**: RBE (Remote Build Execution) 로 CI 시간 단축 + Google 내부 표준.

**우리 적용 방안** (선택): 우리도 같은 듀얼 가능. 다만 초기엔 Cargo only 도 OK. Bazel 도입 시점 = contributor > 20 명, CI 30분+ 일 때.

#### 13.8 Trifecta 샌드박스 (sandboxing/src/ + windows-sandbox-rs + bwrap/)

**왜 차야 하나**: OS 별 native sandbox 모두 지원. `codex sandbox <cmd>` 도 별도 진입점. 우리도 (1) 시작은 macOS Seatbelt 만, (2) Linux 추가 시 Landlock, (3) Windows 추가 시 Job Object 순.

**우리 적용 방안** (간소화):
- v1: `dangerouslyDisableSandbox` 옵션 + user prompt 명시
- v2: macOS Seatbelt (sandbox-exec)
- v3: Linux Landlock (kernel 5.13+ 가정)
- v4: Windows Job (PowerShell + Win32 API)

#### 13.9 Guardian LLM-as-a-judge (core/src/guardian/)

```text
1. Reconstruct compact transcript (user intent + recent assistant + tool context)
2. Ask dedicated guardian review session to assess planned action
3. Return strict JSON (risk_level, user_authorization, outcome, rationale)
4. Fail closed on timeout/parse failure
5. Apply outcome
```

**왜 차야 하나**: 단순 rule-based allow/deny 보다 훨씬 안전. circuit breaker (3 연속 deny -> interrupt) 도 영리함.

**우리 적용 방안**:
- v1: rule-based only (allow/deny list)
- v2: 동일 모델에 "이 명령 위험해?" 추가 prompt
- v3: 별도 guardian 세션 (다른 temperature / system prompt) - **OpenAI 의 표준 패턴**

#### 13.10 Mention syntax (core/src/mention_syntax.rs)

```text
@skill_name   → SkillsManager 주입
@plugin_name  → Plugin 호출
@tool_name    → 도구 직접 호출
```

`PLUGIN_TEXT_MENTION_SIGIL`, `TOOL_MENTION_SIGIL` 상수로 노출. 우리도 user prompt 에서 `@server`, `@db`, `@deploy` 식 도메인 mention 도입.

#### 13.11 Config layer + profile v2 (config/src/)

```text
layer priority: system < user < project < profile < mdm < enterprise < session-flags
- system requirements.toml (/etc/codex/requirements.toml)
- user config.toml (~/.codex/config.toml)
- project config.toml (<cwd>/.codex/config.toml)
- profile (<name>.config.toml) — -p 플래그
- MDM (macOS managed preferences)
- enterprise managed config
- session flags (-c / --enable)
```

**왜 차야 하나**: enterprise 사용자가 "내부 default config 강제" 가능. 우리도 server_manager 도메인이라 동일 필요.

**우리 적용 방안**:
- v1: user config.toml only
- v2: + project config.toml (`./.myharness/config.toml`)
- v3: + system + enterprise override

#### 13.12 Inferred hooks trust (hooks/src/engine/discovery.rs:582-600)

```rust
fn hook_trust_status(is_managed, current_hash, trusted_hash) -> HookTrustStatus {
    if is_managed { HookTrustStatus::Managed }
    else { match trusted_hash {
        Some(h) if h == current_hash => HookTrustStatus::Trusted,
        Some(_) => HookTrustStatus::Modified,
        None => HookTrustStatus::Untrusted,
    }}
}
```

**왜 차야 하나**: hook 의 hash 가 saved trusted_hash 와 다르면 "Modified" 표시 -> 사용자 확인. **First-run 에 untrusted 면 실행 안 함** (default). `--dangerously-bypass-hooks-trust` 로 강제 가능.

**우리 적용 방안**:
- `my_harness/src/hooks/trust.rs` (가칭) - 동일 4-status enum
- 1st run = Untrusted, 사용자 accept 후 Trusted

#### 13.13 Tests as Documentation (tui/snapshots/ + AGENTS.md:170-188)

```text
Requirement: any change that affects user-visible UI (including adding
new UI) must include corresponding `insta` snapshot coverage (add a new
snapshot test if one doesn't exist yet, or update the existing snapshot).
Review and accept snapshot updates as part of the PR so UI impact is
easy to review and future diffs stay visual.
```

**왜 차야 하나**: UI 회귀 자동 감지 + PR review 가 자동으로 visual diff 검토.

**우리 적용 방안** (Rust UI 사용 시): insta 도입. (TS UI 면 Storybook + Chromatic)

#### 13.14 model_provider 분리 (model-provider/ + model-provider-info/)

provider 의 catalog 정보 (auth, model list, capabilities) 와 어댑터 로직 분리. 우리도 `my_harness/src/providers/{openai,anthropic,local,ollama}/` 식 분리.

#### 13.15 Single writer task pattern (rollout/src/recorder.rs:74-79)

```rust
pub struct RolloutRecorder {
    tx: Sender<RolloutCmd>,
    writer_task: Arc<RolloutWriterTask>,  // 단일 백그라운드 task
    pub(crate) rollout_path: PathBuf,
}
```

caller 는 `tx.send(...)` 만. **단일 task 가 file I/O 직렬화**. 우리 history/rollout 도 동일.

#### 13.16 arg0 dispatch (codex-arg0 crate)

같은 binary 가 다른 이름 (`codex`, `codex-tui`, ...) 으로 호출되면 다른 crate 로 dispatch. **40MB 대신 1 binary, N entrypoint** = 다운로드 크기 / cold start 단축.

#### 13.17 Named constants over magic numbers (everywhere)

```rust
const MAX_RETRIES: usize = 10;
const RETRY_SLEEP: Duration = Duration::from_millis(100);
const HISTORY_SOFT_CAP_RATIO: f64 = 0.8;
const GUARDIAN_REVIEW_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_CONSECUTIVE_GUARDIAN_DENIALS_PER_TURN: u32 = 3;
const DEFAULT_EXEC_COMMAND_TIMEOUT_MS: u64 = 10_000;
```

**왜 차야 하나**: tuning 가능 + 코드 인용으로 정책 변경 논의 가능.

#### 13.18 Strict mode config (`--strict-config` flag)

`codex --strict-config` 가 config.toml 의 unknown field 에 대해 error. (v1/v2 호환성 + 디버깅). 우리도 동일.

#### 13.19 Pre-commit gates (AGENTS.md 의 lint 규율)

```text
1. just fmt              # rustfmt
2. just fix -p <proj>    # clippy --fix
3. just test -p <proj>   # nextest
4. just argument-comment-lint
5. cargo insta accept -p codex-tui
```

각 단계 자동화. 우리도 `pre-commit` + `just` 도입 시 동일 5단계.

#### 13.20 TUI 일관성 강제 (tui/src/lib.rs:1-4 + AGENTS.md:122-152)

```rust
#![deny(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::disallowed_methods)]
```

라이브러리 측에서 stdout/stderr 직접 write 금지. `binary` (main.rs) 만 allow. → TUI 와 비대화형이 같은 lib 를 공유해도 깨지지 않음.

#### 13.21 `feature` flag 3-stage (codex-features/src/)

`Stage::{UnderDevelopment, Experimental, Stable, Deprecated, Removed}`. `codex features list` CLI 가 effective state 표시. 우리도 동일.

#### 13.22 `doctor` 명령 (cli/src/doctor.rs)

로컬 설치 / config / auth / runtime health 진단. 우리도 v1 부터 `myharness doctor` 추천.

#### 13.23 Marketplace + startup_sync (ext/marketplace/)

부팅 시 원격 마켓에서 plugin 가져오기. **opt-in**, 미설정 시 no-op. 우리 v2+ 에서 고려.

#### 13.24 State DB + Log DB 분리 (rollout/state_db.rs + state/log_db.rs)

`state.db` = thread metadata, agent jobs, goals, memories, audit
`log.db` = telemetry only (filter 적용 가능)

별도 SQLite 파일 = I/O isolation + 백업 단위 분리. 우리도 동등.

---

### 💡 놀라운 패턴 (Unexpected brilliance)

#### 13.25 vendored bubblewrap + C-rename macro (bwrap/build.rs:51-77)

`bubblewrap` (GNOME upstream) 의 C source 4개 (`bubblewrap.c` 등) 를 vendoring + `cc` crate 로 빌드 + `#define main bwrap_main` 으로 **entrypoint 이름 변경**. 결과: 외부 의존성 zero + cross-compile 가능 + 한 binary 가 bubblewrap 기능 흡수.

#### 13.26 `app-server` v1 + v2 동시 지원 (AGENTS.md:241-277)

IDE 통합 호환성을 위해 legacy v1 + active v2 를 **같은 binary 에서 동시 제공**. 새 API 는 v2, v1 은 deprecated path. 우리도 app-server 만들면 동일.

#### 13.27 Cursor pagination (AGENTS.md:265-267)

```text
- For new list methods, implement cursor pagination by default:
  request fields `pub cursor: Option<String>` and `pub limit: Option<u32>`,
  response fields `pub data: Vec<...>` and `pub next_cursor: Option<String>`.
```

모든 list API 에 일관된 pagination. 우리도 `my_harness list-sessions`, `my_harness list-tools` 등 도입 시 동일.

#### 13.28 ts-rs auto-bindings (app-server-protocol/src/)

`#[ts(export_to = "v2/")]` 매크로가 v2 Rust struct -> TypeScript 자동 생성. IDE 클라이언트가 zero-cost.

#### 13.29 Camel case on wire, snake_case in config.toml (AGENTS.md:246-247)

```text
- Always expose fields as camelCase on the wire with
  `#[serde(rename_all = "camelCase")]` unless a tagged union or explicit
  compatibility requirement needs a targeted rename.
- Exception: config RPC payloads are expected to use snake_case to mirror
  config.toml keys.
```

**wire = camelCase, file = snake_case**. config.toml 키와 RPC 필드명 직접 매핑 위해. 좋은 룰.

#### 13.30 `as_app_server_client` 같은 env-aware 분기 (cli/src/main.rs:378-401, 1346-1397)

`HostSandboxArgs` 가 `cfg(target_os = "...")` type alias. match arm 도 동일. **OS 별 별도 커맨드 struct + 별도 dispatch** 가 같은 인터페이스로 합쳐짐. 우리도 mac/linux/windows 별 sandbox struct + dispatch 만들면 동일.

#### 13.31 arg0-as-dispatch (codex-arg0)

1 binary = N entrypoint. 다운로드 1번 = 모든 entry 사용. 우리 npm wrapper 만들면 동일.

#### 13.32 agents_md 자동 로드 (core/src/agents_md.rs + DEFAULT_AGENTS_MD_FILENAME)

`AGENTS.md` (또는 `.codex/AGENTS.md`, project local) 가 있으면 자동으로 컨텍스트에 inject. 우리도 우리 `MiniMax.md` 를 `~/.myharness/MiniMax.md` 또는 `.myharness/MiniMax.md` 에서 자동 로드.

#### 13.33 opt-in + opt-out 둘 다 지원

`--enable <feature>` / `--disable <feature>` / `config.toml` 의 `[features] <name> = true/false`. 다양한 사용자 진입점.

#### 13.34 Lock file 도 Bazel 관리 (AGENTS.md:34-37)

```text
- If you change Rust dependencies (Cargo.toml or Cargo.lock), run
  `just bazel-lock-update` from the repo root to refresh MODULE.bazel.lock.
- After dependency changes, run `just bazel-lock-check` from the repo
  root so lockfile drift is caught locally before CI.
```

Bazel lockfile 도 같이 갱신. CI 가 drift 검사.

#### 13.35 mtime-based config reloader (network-proxy-loader.rs:393)

`MtimeConfigReloader` 가 mtime polling 으로 config 변경 자동 반영. file watcher (notify crate) 안 써도 됨. 가벼움.

#### 13.36 agent identity + agent jobs (state/src/runtime/agent_jobs.rs + agent_identity crate)

에이전트 별 identity (token, signing key) + job queue. 멀티 에이전트 환경 표준. 우리 server_manager 도메인에서 도입 검토.

---

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.37 `chat_composer.rs` 11K 줄 (tui/src/bottom_pane/chat_composer.rs)

**기술 부채의 표본**. AGENTS.md 가 직접 "Don't add new standalone methods" 라고 못 박은 상태. 처음부터 모듈 분리했어야 했다.

**교훈**: 첫 500줄 부터 분리하는 게 후 11K 줄 리팩토링보다 쉽다. 우리도 "이 파일이 1,000줄 넘으면 무조건 PR 막는다" 룰 필요.

#### 13.38 `codex-core` 비대화 (AGENTS.md:66)

AGENTS.md 가 직접 "resist adding code to codex-core" 라고 한 게, **이미 45K 줄로 커진 후** 룰이 만들어진 것. 사전 예방 vs 사후 룰.

**교훈**: 우리는 5K 줄 단위부터 룰 적용.

#### 13.39 `unwrap_used` deny 만으로 충분치 않음 (이건 오히려 좋음이지만)

`anyhow!`, `expect_err!`, `panic!` 등은 catch 안 됨. **테스트 + review** 가 1차 방어. lint 는 2차.

#### 13.40 Cargo + Bazel 듀얼의 복잡도 (90+ BUILD.bazel)

듀얼 빌드의 유지보수 비용. 94 crate x 2 빌드 시스템 = 188 설정 파일. 작은 팀엔 과함.

**교훈**: contributor > 50 또는 CI 30분+ 시점에만 Bazel 도입.

#### 13.41 `apply_patch` 자체 포맷 (core/prompt_with_apply_patch_instructions.md)

git diff 와 비슷하지만 **다른** 자체 포맷. 모델이 학습 필요. 표준 tool call (write_file + edit_file) 2종이면 충분했을 수도.

**교훈**: tool format 은 표준 우선. 자체 포맷 = 모델 호환성 비용.

#### 13.42 `application/vnd.codex.*` 같은 자체 MIME (mcp_openai_file.rs)

MCP 표준 대신 자체 스키마. 호환성 떨어질 수 있음.

#### 13.43 realtime-webrtc/ crate 는 POC 일 가능성

200-300 LOC 짜리. production-readiness 불명. 우리도 POC crate 는 별도 표시.

#### 13.44 `tui_keymap` (config crate 안에 200줄)

vim/emacs/default 3개만. 학습 비용 > 가치. 우리는 default 만 + 사용자 직접 remap 안 함.

#### 13.45 `pets` (ascii_animation.rs)

agent idle 시 돌아다니는 펫. 엔터프라이즈 사용자에겐 노이즈. 옵트인이어야.

#### 13.46 `ext/extension-api` 가 `codex-core` 의 1st-party

plugin host API 인데 codex-core 와 결합 강함. **3rd-party plugin author** 가 이걸 안정적으로 의존하기 어렵다. 우리도 plugin host API 는 `protocol` (1st-class) 에 두고, 1st-party ext 는 별도.

#### 13.47 `core/src/lib.rs:122-128` 에 `ConversationManager` deprecated alias

```rust
#[deprecated(note = "use ThreadManager")]
pub type ConversationManager = ThreadManager;
```

`ConversationManager` -> `ThreadManager` rename 후 12+ month deprecation window. 좋은 패턴이지만 우리 같은 신규 프로젝트엔 불필요 (처음부터 이름 잘 짓기).

#### 13.48 Sandbox env var 룰 (AGENTS.md:8-10) 의 비대칭

`CODEX_SANDBOX_NETWORK_DISABLED=1` 와 `CODEX_SANDBOX=seatbelt` 가 agent 자신이 만든 sandbox detection 용. **새 코드에서 추가/수정 금지** (invariant). AI 가 만든 코드가 자기 자신의 detection 룰을 바꾸면 자기 자신이 disable 시킬 수 있어서.

**교훈**: 우리도 "AI 가 만든 코드가 자기 자신의 거버넌스를 disable 못 하게" invariant 박기.

#### 13.49 Bazel + Cargo dep 변경 시 2 step (AGENTS.md:34-37)

`just bazel-lock-update` + `just bazel-lock-check` 필수. 자동화 잘 되어 있지만, **lockfile 두 개** 가 사고 지점. 우리 듀얼 빌드는 신중히.

#### 13.50 Windows sandbox 1.4K LOC

11+ 서브모듈 (wfp, dpapi, token, cap, acl, ...). Windows 가 정말 복잡. 우리 v1 에서 Windows 지원 시 모듈 분리 + OS abstraction layer 신중히.

## §14 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 800 줄 변경 가이드의 실효성

`AGENTS.md` 가 "큰 변경은 800 줄 이하, 복잡 로직은 500 줄" 이라 못 박았는데, 실제 PR 통계는? `gh pr list --repo openai/codex --state merged --limit 100 --json additions | jq '[.[] | select(.additions > 800)]'` 로 검증 필요. 룰 자체는 좋지만 **현실** 에서 얼마나 지켜지는지 미지수.

### 14.2 `codex-core` 의 실제 크기

44.6K LOC 라고 추정한 건 `core/src` 디렉토리 카운트. `core-api`, `core-plugins`, `core-skills` 분리했어도 cargo workspace 멤버십으로는 core 의 일부. **AGENTS.md 의 비만 방지 룰** 이 정말 효과를 봤는지, 다른 crate 로 분산한 양이 얼마인지 — 1 년 전 vs 현재 LOC diff 필요.

### 14.3 `MCP` vs 자체 프로토콜의 사용 비율

`codex-rs/mcp-server/` vs `rmcp` vs 자체 `application/vnd.codex.*`. **end-user 가 codex 쓸 때 MCP 서버 몇 개 연결하는지, 자체 포맷이 더 많이 쓰이는지** 실측 데이터 없음. 우리 my_harness 에서 MCP 우선 vs 자체 우선 결정에 필요.

### 14.4 ratatui 의 한계

codex 가 ratatui + crossterm 으로 풀스크린 TUI 만드는데, **이미지/그래픽** (mermaid 다이어그램, syntax highlight 등) 처리는? ratatui 는 텍스트 기반이라 터미널 이미지 프로토콜 (kitty/iTerm) 지원이 약할 것. codex 가 어떻게 우회하는지 (없으면? 외부 뷰어 띄우기?) 미확인.

### 14.5 `codex-message-history` 의 마이그레이션 전략

SQLite flock + advisory lock 쓰는데, **포맷 변경 시 마이그레이션** 어떻게? `codex_message_history::schema_version` 같은 거 있는지? 1.0 출시 후 schema 변경 시 user data 손실 가능성. 우리 설계 시 `state.json` schema versioning 미리 박아야 함.

### 14.6 Linux 의 WSL/macOS 의 Rosetta 영향

Linux sandbox 가 Landlock + bwrap 인데, **WSL2** (Windows 위 Linux) 에서 Landlock 이 안 보일 수 있음. macOS Apple Silicon Rosetta 환경에서 Codex 의 native binary 가 Intel 전용이면? — cross-platform 우리 결정 시 검증 필요.

### 14.7 `AGENTS.md` 자체의 진화

이 문서가 openai/codex repo 의 `AGENTS.md` 인데, OpenAI 가 자주 업데이트 (2026-05~06 사이 6 model-visible-context 룰 추가 등). 우리 `MiniMax.md` 도 동일하게 **반복적 refinement** 필요. 정적 문서 X.

### 14.8 OpenAI 외 모델 provider

Codex 가 `model_provider` 추상화 갖는데, **Anthropic / Google / local model** 까지 extend 가능한지? 아니면 GPT 계열 전용? 우리 my_harness 가 모델 비종속 목표면 critical.

### 14.9 보안 위협 모델

`shell-escalation` crate 가 Landlock + bwrap / Seatbelt 로 syscall 제한하는데, **prompt injection** 으로 모델이 위험 명령 생성하는 시나리오 차단 방법은? 사용자 컨펌 + allowlist 패턴만으로 충분? codex 가 runtime 에 trust boundary 를 어떻게 정의하는지 (sandbox 외) 미확인.

### 14.10 `codex-tui` 의 성능

TUI 가 chatwidget (chatwidget.rs 중심) + bottom_pane + 6+ 서브모듈. **모델 응답 streaming** 이 와도 frame rate 떨어지지 않는지? 200K context + stream JSON 1초당 5KB → TUI 가 어떻게 처리? 우리 design 시 profiling 필요.

### 14.11 `codex-tui` 와 codex-app-server 의 차이

둘 다 client 처럼 보이지만, app-server 가 `--app-server` 플래그로 별도 실행. **데몬 모드** vs **인터랙티브 TUI** 의 책임 분담 — 우리 my_harness 가 server-client 분리 (TASK-005 결정) 시 참고. app-server 가 gRPC/RPC 정의 어디에?
