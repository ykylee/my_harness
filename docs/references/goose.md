# goose (block/goose → aaif-goose/goose) 심층 코드 분석

- **문서 목적**: `goose` (이전 `block/goose`, 현재 Linux Foundation 산하 AAIF `aaif-goose/goose`) 의 실제 코드 베이스를 심층 분석해, `my_harness` 의 아키텍처 결정(언어 / TUI / 토폴로지 / 빌드 / 보안) 에 직접 활용 가능한 인사이트를 만든다. 1차 분석(`docs/REFERENCES.md` 의 8축 비교표) 의 후속.
- **범위**: `crates/goose` (코어, 134,996 LOC, 276 파일) + `crates/goose-cli` (22,576 LOC, 50 파일) + `crates/goose-server` (12,365 LOC, 39 파일, `goosed` 바이너리) + `crates/goose-mcp` (6,980 LOC) + `crates/goose-acp-macros` (295 LOC proc-macro) + `ui/desktop` (Electron 76,848 LOC, 409 파일) + `ui/text` (Ink TUI 4,462 LOC) + `evals/open-model-gym` + `evals/harbor` + CI/release 워크플로. 14섹션 표준 + §15/§16 으로 goose 특화 항목(멀티 인터페이스, TUI 부재 재평가) 추가.
- **대상 독자**: yklee, Mavis, TASK-005(CLI/TUI 스택 결정) 디자인 리뷰 참여자.
- **상태**: 1차 작성 완료 (2026-06-06). 코드 인용은 모두 `crates/`, `ui/`, `.github/` 등 실제 경로 기준. 시크릿/토큰 값은 일절 포함하지 않음.
- **최종 수정일**: 2026-06-06
- **관련 문서**: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [REFERENCES.md (1차 비교표)](../REFERENCES.md), [PROJECT_PROFILE.md](../../docs/PROJECT_PROFILE.md), [TASK-005 (CLI/TUI 전환)](../../ai-workflow/memory/backlog/2026-06-05.md)

---

## 1. 개요 (Overview)

| 항목 | 값 |
| --- | --- |
| 정식 명칭 | goose (이전 `block/goose`, 현재 `aaif-goose/goose`) |
| 한 줄 설명 | "your native open source AI agent — desktop app, CLI, and API — for code, workflows, and everything in between" |
| 메인 binary | (1) `goose` (CLI, `crates/goose-cli/src/main.rs`), (2) `goosed` (서버, `crates/goose-server/src/main.rs`), (3) `Goose.app` (Electron 데스크탑, `ui/desktop/src/main.ts`) |
| 라이선스 | Apache-2.0 (코드) / CC BY 4.0 (문서). [LICENSE:1-10](../../harness-refs/goose/LICENSE) |
| 워크스페이스 버전 | `1.37.0` (`Cargo.toml` `[workspace.package]`) |
| Rust MSRV | `1.91.1` (`rust-version = "1.91.1"`) |
| 거버넌스 | Linux Foundation 산하 AAIF (Agentic AI Foundation). `GOVERNANCE.md:1-9` 참조. Apache-2.0 + 추가 정책은 https://lfprojects.org/policies/ |
| Core Maintainers | 7명 (Bradley Axen founder 포함). `MAINTAINERS.md:3-9` |
| 코드 라인 (Rust) | **190,523** total, `find -name '*.rs' -not -path '*/target/*' -not -path '*/vendor/*'` 결과 |
| 코드 라인 (TypeScript) | **102,698** total, UI 통합 (`ui/desktop` 76,848 + `ui/sdk` 4,614 + `ui/text` 4,462 등) |
| 의존성 핵심 | `rmcp` 1.4 (MCP SDK), `agent-client-protocol` 0.11, `axum` 0.8, `tokio` 1.48, `keyring` 3.6.3 (vendored), `sqlx` 0.8.5 (SQLite), `clap` 4.1.14, `cliclack` (transitive), `console` 0.x, `utoipa` 4.2, `candle-*` / `llama-cpp-2` (local-inference feature) |
| 메인 인터페이스 | (1) Electron 데스크탑 (멀티 윈도우, Tray, 자동 업데이트), (2) axum 0.8 기반 REST + WebSocket 서버 (`goosed`), (3) ACP/stdio 또는 ACP/WS 클라이언트, (4) 데스크탑이 띄우는 **Ink-React TUI** (`ui/text`, "goose-text") — **5번째 인터페이스** |

goose 의 가장 큰 차별점은 **다중 인터페이스 + 런타임 토폴로지** 다. 1차 분석(8축 비교표)에서 "Electron + TUI" 로 분류했지만, 실제 코드는 그보다 풍부하다.

- 사용자는 **데스크탑(Electron)** 으로 시작하지만, 데스크탑이 `goosed` 바이너리를 spawn 해서 100% 동일한 코어 로직을 두 번째 인터페이스로 띄운다 (`ui/desktop/src/goosed.ts:1-200`, `crates/goose-server/src/commands/agent.rs:39-166`).
- 추가로 `goose` CLI 가 직접 코어를 띄우는 3번째 진입점이고 (`crates/goose-cli/src/main.rs:1-51`), `goose acp` / `goose serve` 가 ACP(stdio/WS) 프로토콜으로 4번째 진입점을 노출한다.
- 2025년 하반기부터 `ui/text` (React + Ink) 가 추가되어 5번째 TUI 인터페이스가 정식 제품 라인이 됐다.
- **TUI 가 "부재" 라는 1차 분류는 부정확** 하다. 진실은 "데스크탑 우선, TUI 는 후속 통합". §13 Notable Patterns 에서 재평가.

---

## 2. 아키텍처 (Architecture)

### 2.1 최상위 프로세스 토폴로지 (Electron + goosed + CLI + ACP + TUI 5중)

```
┌────────────────────────────────────────────────────────────────────────┐
│  ui/desktop (Electron)                                                  │
│  ├─ Main process (main.ts 2815 LOC) — Tray, window state, auto-update   │
│  ├─ Renderer (App.tsx 727 LOC) — React 19.2.4, Radix UI, Tailwind v4    │
│  ├─ Preload (preload.ts 370 LOC) — contextBridge, IPC                   │
│  └─ Embedded goosed (child_process.spawn)                              │
│      ├─ REST routes (axum 0.8) — /reply, /sessions, /config, /agent    │
│      └─ ACP routes (WebSocket) — GooseClient (TS) ↔ AcpServer (Rust)    │
│                                                                          │
│  ui/text (Ink / React for terminal)                                     │
│  └─ 별도 패키지, 같은 @aaif/goose-sdk 통해 goosed 와 통신               │
│                                                                          │
│  goose CLI (standalone)                                                  │
│  └─ crates/goose-cli/src/main.rs → cli() → tokio runtime 8MB stack      │
│      ├─ Session subcommand (interactive REPL via cliclack)               │
│      ├─ Run subcommand (recipe / instruction file)                      │
│      ├─ Acp subcommand (stdio JSON-RPC)                                  │
│      ├─ Serve subcommand (HTTP+WS, goosed 와 유사)                      │
│      └─ Mcp subcommand (4 builtin extensions spawn)                     │
└────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│  crates/goose (core, 134,996 LOC)                                       │
│  ├─ agents/ — Agent (3398 LOC), reply loop, tool inspection, retry      │
│  ├─ providers/ — 50+ LLM provider (base trait + canonical registry)     │
│  ├─ session/ — SessionManager (SQLite, 2876 LOC), persistence            │
│  ├─ security/ — SecurityManager, scanner, 3-layer inspection            │
│  ├─ permission/ — 4 모듈 (confirmation, inspector, judge, store)         │
│  ├─ mcp_utils.rs — MCP 통합 (rmcp 1.4 SDK 직접 사용)                   │
│  ├─ acp/ — AcpServer (3798 LOC), custom dispatch, schema gen            │
│  ├─ skills/ — Skill frontmatter, agentskills.io spec 준수               │
│  ├─ hooks/ — HookManager (Open Plugins hooks spec, 13 events)           │
│  ├─ context_mgmt/ — 점진적 컨텍스트 압축, tool pair 요약                │
│  ├─ recipe/ — Recipe YAML, build_recipe, template_recipe                 │
│  ├─ gateway/ — Telegram gateway                                         │
│  └─ ... (47 modules)                                                     │
└────────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────────┐
│  crates/goose-mcp (6,980 LOC, 15 files)                                 │
│  ├─ AutoVisualiserRouter  — 데이터 시각화 MCP 서버                      │
│  ├─ ComputerControllerServer — 데스크탑 제어 (screen, keyboard, mouse)  │
│  ├─ MemoryServer         — 영구 메모리 (knowledge graph)                │
│  ├─ TutorialServer       — 인터랙티브 튜토리얼                          │
│  └─ peekaboo (macOS only) — 화면 캡처/OCR                                │
└────────────────────────────────────────────────────────────────────────┘
```

핵심 추상화:

1. **`Agent` 구조체** (`crates/goose/src/agents/agent.rs:225-249`): provider, extension_manager, tool_confirmation_router, hook_manager, tool_inspection_manager, retry_manager 를 모두 포함하는 단일 fat struct. **Stateful** (Arc<Mutex<>>) 이며 `SessionConfig` 단위로 생성.
2. **`Provider` trait** (`crates/goose/src/providers/base.rs:864-901`): `stream(model_config, session_id, system, messages, tools) -> MessageStream` 가 1차 인터페이스. `complete` / `complete_fast` 는 `stream` 의 어댑터. **50+ 구현체**.
3. **`Session` ↔ `SessionManager`** (`crates/goose/src/session/session_manager.rs:55-89`): SQLite (sqlx 0.8.5) 기반. `SessionType` 7개 (`User`, `Scheduled`, `SubAgent`, `Hidden`, `Terminal`, `Gateway`, `Acp`).
4. **`ExtensionConfig`** (`crates/goose/src/agents/extension.rs`): stdio / streamable-http / builtin / SSE / Frontend / Mcp 6가지 변형. `extension_manager.rs` 가 동적 로드/언로드.
5. **`AcpServer`** (`crates/goose/src/acp/server.rs:1-200`): Agent Client Protocol 0.11 SDK + 자체 `custom_methods` proc-macro (`goose-acp-macros`). JSON-RPC 2.0.

### 2.2 디렉토리 트리 (실측)

```
harness-refs/goose/                                   # repo root
├── AGENTS.md              (126 lines)  # 핵심 운영 매뉴얼
├── CLAUDE.md              (1 line)     # @AGENTS.md
├── GOVERNANCE.md          (199 lines)
├── MAINTAINERS.md         (18 lines)   # 7 Core + 5 Maintainers
├── SECURITY.md            (15 lines)   # 위험 경고
├── README.md              (61 lines)
├── Cargo.toml             (115 lines)  # workspace
├── Cargo.lock             (314,618 bytes)
├── Justfile               (450 lines)  # 60+ recipes
├── Dockerfile             (38 lines)
├── rust-toolchain.toml    (1 line)
├── clippy.toml            (8 lines)
├── deny.toml              (21 lines)
├── .goosehints            (18 lines)
├── goose-self-test.yaml   (447 lines)  # recipe-based E2E
├── .cargo/                (config.toml)
├── .claude/  .codex/  .cursor/  .intersect/  .devcontainer/
├── crates/                                       # workspace member roots
│   ├── goose/                                    # core, 276 files, 134,996 LOC
│   ├── goose-cli/                                # 50 files, 22,576 LOC
│   ├── goose-server/                             # 39 files, 12,365 LOC
│   ├── goose-mcp/                                # 15 files, 6,980 LOC
│   ├── goose-acp-macros/                         # 1 file, 295 LOC
│   ├── goose-sdk/                                # 3 files, 93 LOC
│   ├── goose-sdk-types/                          # 3 files, 1,610 LOC
│   ├── goose-test/                               # 6 files, 258 LOC
│   └── goose-test-support/                       # 4 files, 315 LOC
├── ui/                                            # pnpm workspace
│   ├── desktop/                                    # 409 files, 76,848 LOC
│   ├── sdk/                                        # @aaif/goose-sdk
│   ├── text/                                       # Ink TUI
│   ├── goose-binary/
│   └── install-link-generator/
├── evals/
│   ├── open-model-gym/                            # 3-rep matrix runner
│   └── harbor/                                    # Docker benchmark
├── examples/                                      # MCP example extensions
├── services/  workflow_recipes/  documentation/   # Docusaurus
├── recipe-scanner/  oidc-proxy/  scripts/
├── vendor/v8/                                     # v8 crate (vendored)
└── .github/workflows/                              # 41 YAML files
```

### 2.3 핵심 추상화 — 추가 메모

- **`current_goose_mode: Mutex<GooseMode>`** (`agent.rs:229`): `Auto` / `Approve` / `Chat` / `SmartApprove` 4 모드. Tool call 승인 정책의 모드별 차이는 `permission_judge.rs` 가 담당.
- **`AppState` 공유** (`goose-server/src/state.rs:25-35`): `Arc<AgentManager>`, `Arc<TunnelManager>`, `Arc<GatewayManager>`, `SessionEventBus HashMap`, `ExtensionLoadingTasks Arc<Mutex<>>`. `OnceCell` for `InferenceRuntime` (local-inference feature).
- **`ProviderInventoryService`** (`providers/inventory/`): 7개 provider 동시 상태 체크 (`PROVIDER_CONFIG_STATUS_CHECK_CONCURRENCY: usize = 16` in acp/server.rs:150).
- **싱크 barrier** (`commands/agent.rs:107-138`): `axum_server::Handle` + `shutdown_signal()` 으로 graceful shutdown.
- **`boot_marker` 디버깅 채널** (`main.rs:46-48`, `commands/agent.rs:17-19`): `eprintln!("GOOSED_BOOT: ...")` — Electron main process 가 stdout/stderr 파싱. **사례: dev 모드에서 startup hang 디버깅 전용**.

---

## 3. 진입점 & CLI

### 3.1 바이너리 트리 (clap dispatch)

`crates/goose-cli/src/cli.rs:780-1010` 의 `Command` enum. 33+ 서브커맨드:

```
goose
├── configure {}                                  # 1st-time interactive setup
├── info { --verbose, --check }
├── doctor {}                                    # healthcheck
├── mcp { AutoVisualiser|ComputerController|Memory|Tutorial }   # spawn builtin MCP
├── acp { --with-builtin [...] }                 # ACP agent (stdio JSON-RPC)
├── serve { --host 127.0.0.1, --port 3284, --with-builtin [...] }   # HTTP+WS ACP
├── session { List|Remove|Export|Import|Diagnostics [name|--session-id|--path] --resume --fork --history ... }
├── project {}                                   # open last project dir
├── projects                                      # list recent
├── run { -i FILE | -t TEXT | --recipe NAME | --system TEXT | --params k=v ... }
├── recipe { List|Validate|Deeplink|Open|Explain }    # alias "r"
├── skills { List|... }                          # manage .agents/skills
├── plugin { Install|Update }                    # install from URL/path
├── schedule { Add|List|Remove|RunNow|Sessions|Status|Start|Stop|Pair|ServicesStatus|ServicesStop|CronHelp }
├── gateway { Status|Start|Stop|Pair }
├── update { --canary, --reinstall }
├── term { Init|Info|Log|Run }                    # shell wrapper (.goosehints based)
├── tui {}                                       # launch ui/text Ink TUI
├── local-models { Search|Download|Delete }      # local inference registry
├── completion { Bash|Zsh|Fish|Powershell|Nu|Elvish }    # clap_complete
├── review { Init|Log }                          # PR review mode
├── validate-extensions PATH                     # bundled-extensions JSON check
├── (no command) → Session interactive chat      # implicit default
└── help, --version
```

각 명령의 상세 옵션은 `cli.rs:780-1003` 참조. 가장 큰 명령 그룹은 `Session` (interactive) 와 `Run` (one-shot).

### 3.2 `goose-cli/src/main.rs` (51 LOC, 축약형)

```rust
// crates/goose-cli/src/main.rs:9-15
#[cfg(windows)]
fn enable_windows_vt_processing() {
    // colors_supported() has the side effect of calling SetConsoleMode with
    // ENABLE_VIRTUAL_TERMINAL_PROCESSING on the underlying console handle.
    let _ = console::Term::stdout().features().colors_supported();
    let _ = console::Term::stderr().features().colors_supported();
}

// crates/goose-cli/src/main.rs:36-50
let handle = std::thread::Builder::new()
    .name("goose-cli-main".to_string())
    .stack_size(8 * 1024 * 1024)
    .spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");
        runtime.block_on(run())
    })
    .map_err(|e| anyhow::anyhow!("Failed to spawn goose-cli main thread: {}", e))?;
```

**8MB main 스택** (기본 2MB의 4배). `cargo test` 가 `RUST_MIN_STACK=8388608` 로 강제 (`ci.yml:76`). 이는 stackful 비동기 generator 사용 (`agent.rs:281-297` `tool_stream` async_stream!{} 매크로) 때문일 가능성.

**Windows VT 처리** — `cliclack` 스피너가 Windows Console Host 에서 정상 동작하려면 필수. 같은 로직이 `ui/desktop/src/main.ts` 의 `enableWinShims` 와 쌍을 이룸.

### 3.3 `goose-server/src/main.rs` (108 LOC, 축약형)

```rust
// crates/goose-server/src/main.rs:29-44
#[derive(Subcommand)]
enum Commands {
    /// Run the agent server
    Agent,
    /// Run the MCP server
    Mcp {
        #[arg(value_parser = clap::value_parser!(McpCommand))]
        server: McpCommand,
    },
    /// Validate a bundled-extensions JSON file
    #[command(name = "validate-extensions")]
    ValidateExtensions { path: PathBuf },
}

// crates/goose-server/src/main.rs:83-105
match cli.command {
    Commands::Agent => { commands::agent::run().await?; }
    Commands::Mcp { server } => {
        match server {
            McpCommand::AutoVisualiser      => serve(AutoVisualiserRouter::new()).await?,
            McpCommand::ComputerController  => serve(ComputerControllerServer::new()).await?,
            McpCommand::Memory              => serve(MemoryServer::new()).await?,
            McpCommand::Tutorial            => serve(TutorialServer::new()).await?,
        }
    }
    Commands::ValidateExtensions { path } => { /* ... */ }
}
```

`commands/agent.rs:39-166` 가 실제 서버 lifecycle:
- rustls crypto provider 명시적 설치 (`commands/agent.rs:44-45`)
- `AppState::new(settings.tls).await?` (한 번만 init, Arc)
- `acp_server` 와 `rest_router` 를 merge
- `CORS: AllowAny` (개발 편의, 프로덕션은 reverse proxy 가드 필요)
- TLS / non-TLS 분기 (`axum_server::bind_rustls` vs `axum::serve`)
- graceful shutdown via SIGINT/SIGTERM (`shutdown_signal`)

### 3.4 `crates/goose-cli/Cargo.toml` (의존성)

```toml
# deps (간접): clap 4.1.14, cliclack, console, serde_yaml, goose-sdk, goose-mcp
[[bin]]
name = "goose"
path = "src/main.rs"

[[bin]]
name = "generate_manpages"
path = "src/bin/generate_manpages.rs"
```

goose-cli 자체는 **의존성을 거의 가지지 않음** (`Cargo.toml:3.8KB`). 모든 로직은 `goose` 크레이트에 위임. 이게 잘 작동하는 이유는 `cliclack` / `console` / `comfy-table` 이 transitive workspace deps 으로 들어오기 때문.

### 3.5 `cli()` 함수 동작 흐름

```rust
// crates/goose-cli/src/cli.rs (cli function, end of file ~line 1340+)
pub async fn cli() -> Result<()> {
    let cli = Cli::parse();                     // clap dispatch

    // platform-specific MCP builtin registration
    let builtins: Vec<String> = match &cli.command { ... };

    // additional source roots (e.g. ~/.config/goose/sources/)
    let additional_source_roots = ...;

    // platform-specific session execution
    match cli.command {
        Configure {}     => handle_configure().await,
        Session { ... }  => build_session(...).await,
        Run { ... }      => build_session(...).await,        // one-shot
        Acp { builtins } => acp::serve(builtins).await,      // stdio
        Serve { ... }    => acp::serve(builtins).await,      // HTTP+WS
        Mcp { server }   => goose_mcp::serve(server).await,
        ...
        // "no command" → interactive session
        None => build_session(...).await,
    }
}
```

`build_session` (in `session/builder.rs`) 가 **공통 진입점**. interactive / one-shot 모두 같은 `Agent::new()` 로 진입.

---

## 4. TUI/UI 구현

### 4.1 1차 분석 보정: "TUI 부재"는 부정확

1차 `REFERENCES.md` 8축 비교표에서 goose 를 "TUI 없음" 으로 분류했지만, 이는 **잘못된 분류** 다. 실제:

| 인터페이스 | 라이브러리 | 위치 | 상태 |
| --- | --- | --- | --- |
| Desktop (Electron) | React 19.2.4 + Radix UI + Tailwind v4 | `ui/desktop/src/` | 1st-party 메인 |
| **Text TUI** | **Ink 6 + React** (TS) | `ui/text/src/` | 1st-party 정식 |
| CLI interactive | **cliclack** + **console** + **comfy-table** (Rust) | `crates/goose-cli/src/session/` | 1st-party |
| CLI non-interactive | clap + std | `crates/goose-cli/src/commands/` | 1st-party |
| ACP stdio | agent-client-protocol 0.11 | `crates/goose/src/acp/` | 1st-party |
| REST + WS | axum 0.8 | `crates/goose-server/src/routes/` | 1st-party |

즉 goose 는 **6개 인터페이스**를 운영한다. 1차 비교표는 "데스크탑만 정식" 으로 단순화했고, 이 단순화가 TASK-005 의사결정을 왜곡할 위험이 있다.

### 4.2 cliclack + console — 무엇을 하는가

**무엇을 한다** (`crates/goose-cli/src/commands/configure.rs:101-114, 131-147`):
- `cliclack::intro` / `cliclack::select` / `cliclack::confirm` / `cliclack::log::success` 등 고수준 TUI 컴포넌트
- `cliclack::spinner` for long ops (configure.rs:101, signup_*/*.rs)
- `console::style` for 색상/스타일 (`configure.rs:2, 4, 8-15`)

**무엇을 안 한다**:
- **풀스크린 TUI 모드** — cliclack 은 "프롬프트 한 줄 + 선택지" 컴포넌트들의 라이브러리, **화면을 점유하지 않음**. 메인 루프는 stdio.
- **vim-style 키 바인딩, 멀티 패널, status bar, syntax highlighting** — 모두 없음. `crates/goose-cli/src/session/` 의 input 모듈은 `rustyline` / `reedline` 같은 전문 line editor 도 안 쓰고 stdio read.
- **streaming token rendering** — 모델 출력이 와도 즉시 dump, render loop 없음.
- **TUI 테스트 자동화** — `crates/goose-cli/src/scenario_tests/` 가 있긴 하지만 grep 으로는 접근 한정.

### 4.3 `ui/text` — Ink TUI (2025~)

`ui/text/src/` 디렉토리는 별도 패키지, README 일부 발췌 (미확인, 디렉토리 존재 확인):

```
ui/text/src/                   # 18 files, 4,462 LOC
```

Ink (React for CLI) 기반. AGENTS.md:96-111 의 **"Ink / Terminal UI 규율"** 은 이 패키지에도 적용:
- `wrap="truncate"` (절대 `wrap="wrap"` 금지 — Ink 가 height overflow 처리 못 함)
- pre-truncate to character budget
- flexGrow={1} 금지
- height budget 정확히 계산 (border, padding, margin, header/footer 다 포함)
- trailing margin 마지막 아이템에 금지
- marginBottom 대신 container `gap`

goose 팀이 Ink 를 채택하고 이 정도로 **명시적 규율** 을 문서화했다는 건, Ink 가 처음엔 "쉬워 보이지만 실전에서 cell overflow 함정" 이 많다는 것을 시사한다.

### 4.4 ui/desktop — Electron main + React 19

`ui/desktop/src/main.ts:2815` LOC. 분량 대부분은 다음:

1. **메뉴 / Tray / 윈도우 상태** (100-300): native menu + electron-window-state + auto-updater + Squirrel startup handler
2. **`goosed.ts:450`** (200-450): goosed 자식 프로세스 spawn / healthcheck / cert fingerprint / cleanup
3. **i18n 메뉴** (60-130): `MENU_TRANSLATIONS_ZH_CN` dictionary + recursive `translateMenuLabels` — Electron native menu 는 renderer (react-intl) 와 분리되어 별도 처리 필요
4. **auto-updater / CSP / URL security** (분산)
5. **ipcMain handlers** — `/api/*` 라우트들을 Electron IPC 로 bridge

`App.tsx:727` — React 19 + Radix UI + Tailwind v4. **`use-katex`, `rehype-katex`, `remark-math`** for 수식 렌더링. **`@mcp-ui/client` 6.1.0** + **`@modelcontextprotocol/ext-apps` 1.1.1** — MCP UI 프로토콜 클라이언트. `@agentclientprotocol/sdk` 0.19 — ACP TS SDK.

### 4.5 멀티 윈도우 + Tray + 자동 업데이트

`ui/desktop/src/main.ts:1-100` 의 import + `main.ts:175-201` 의 settings + `main.ts:80-130` 의 메뉴 다국어 처리 + `electron-updater` integration (`utils/autoUpdater.ts`). `Tray` (`main.ts:17`) 로 macOS 메뉴바 + Windows system tray 에 goose 아이콘 상주.

**UPDATE flow** (`main.ts:42-48`): `getUpdateAvailable` / `registerUpdateIpcHandlers` / `setupAutoUpdater` / `updateTrayMenu`. `electron-updater` 6.8.3 + `electron-forge` 7.11.1 (`package.json:115-117`).

### 4.6 build & bundle (ui/desktop)

`package.json:13-46` 30+ scripts. 핵심:
- `start-gui` (`:15`): `pnpm run generate-api && pnpm run i18n:compile && electron-forge start`
- `bundle:default` (`:21`): full build + ditto zip
- `test-e2e` (`:24`): `pnpm run generate-api && playwright test`
- `lint:check` (`:31`): `typecheck + eslint --max-warnings 0 + i18n:check`

`@hey-api/openapi-ts` 0.93.0 으로 OpenAPI → TS 자동 생성 (`openapi.json` source: `crates/goose-server/src/openapi.rs`).

---

## 5. LLM 통합

### 5.1 `Provider` trait (핵심)

```rust
// crates/goose/src/providers/base.rs:864-901
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the name of this provider instance
    fn get_name(&self) -> &str;

    /// Primary streaming method that all providers must implement.
    ///
    /// Note: Do not add `#[instrument]` here — the call sites (`complete` and
    /// `stream_response_from_provider`) create the telemetry span so that
    /// `session.id` is set once rather than in every provider.
    async fn stream(
        &self,
        model_config: &ModelConfig,
        session_id: &str,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<MessageStream, ProviderError>;

    /// Complete with a specific model config.
    #[tracing::instrument(...)]
    async fn complete(...) -> Result<(Message, ProviderUsage), ProviderError> {
        // delegates to stream + collect_stream
    }

    async fn complete_fast(...) -> Result<(Message, ProviderUsage), ProviderError>;  // fast-model fallback

    async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError>;
    async fn fetch_recommended_models(&self) -> Result<Vec<String>, ProviderError>;
    fn supports_embeddings(&self) -> bool;
    async fn create_embeddings(...) -> Result<...>;
    async fn generate_session_name(...) -> Result<String>;   // auto session naming
    async fn configure_oauth(&self) -> Result<(), ProviderError>;
    async fn refresh_credentials(&self) -> Result<(), ProviderError>;
    async fn update_mode(&self, _session_id: &str, _mode: GooseMode) -> Result<(), ProviderError>;
    async fn handle_permission_confirmation(...) -> Result<...>;
}

// ProviderDef: factory for instantiation
pub trait ProviderDef: Send + Sync {
    type Provider: Provider + 'static;
    fn metadata() -> ProviderMetadata;        // name, default_model, config_keys, known_models
    fn from_env(model, extensions) -> BoxFuture<Result<Self::Provider>>;
    fn from_env_with_working_dir(model, extensions, working_dir) -> BoxFuture<...>;
    fn supports_inventory_refresh() -> bool { false }
    fn inventory_identity() -> Result<InventoryIdentityInput> { ... }
    fn inventory_configured() -> bool { ... }
}
```

**Trait 설계 노트** (코드 주석에서 발췌):
- `stream` 만 1차 인터페이스. `complete` 와 `complete_fast` 는 어댑터.
- `#[instrument]` 는 `stream` 이 아니라 호출자가 만든다 — `session.id` 가 한 번만 span 에 박히도록.
- 50+ provider 가 `ProviderDef::from_env` + `Provider::stream` 2개만 구현하면 됨.
- `ProviderMetadata` 는 utoipa-compatible `ToSchema` → OpenAPI 자동 노출.

### 5.2 50+ Provider 목록 (`providers/mod.rs` 발췌)

| 카테고리 | Provider | 비고 |
| --- | --- | --- |
| First-party (code) | `anthropic`, `openai`, `google`, `ollama`, `openai_compatible`, `bedrock` (feature), `azure`, `gcpvertexai` | 표준 |
| Code (subprocess ACP) | `claude_code`, `codex`, `codex_acp`, `gemini_cli`, `amp_acp`, `claude_acp`, `copilot_acp`, `pi_acp`, `cursor_agent`, `chatgpt_codex`, `kimicode` | 로컬 서브프로세스 |
| Aggregator | `openrouter`, `litellm`, `tetrate`, `nanogpt`, `avian`, `databricks`(2종), `snowflake`, `huggingface`, `sagemaker_tgi`, `xai`, `githubcopilot` | 다중 모델/라우터 |
| Declarative (JSON) | 27개 (`providers/declarative/*.json`) — `alibaba`, `cerebras`, `deepseek`, `groq`, `mistral`, `nvidia`, `perplexity`, `zhipu`, `venice`, `ollama_cloud` 등 | 외부 정의 가능 |
| Local inference | `local_inference` (candle + llama-cpp-2) | offline, feature-gated |
| Test | `testprovider`, `mock` (in tests/) | — |

**재미있는 점**: 27개 declarative provider 는 **JSON 한 파일로 새 provider 추가 가능** (`providers/declarative/*.json`). `providers/declarative_providers.rs` 가 런타임 파싱.

### 5.3 Canonical Model Registry (`providers/canonical/`)

`providers/canonical/registry.rs` + `model.rs` + `data/canonical_models.json`. **build_canonical_models** 바이너리 (`bin/build_canonical_models.rs`) 가 `models.dev` 에서 메타데이터 scrape 해서 `data/canonical_models.json` 생성. prepare-release 시 `just build-canonical-models` 실행 (`Justfile:320-322`).

이게 흥미로운 이유: goose 가 **외부 카탈로그를 빌드타임에 흡수** 해서, 한 곳에서 모델 메타데이터 (context_limit, pricing, reasoning, vision 등) 을 관리. 새 모델 출시 시 upstream 만 따라가면 됨.

### 5.4 Streaming / token 추적

`MessageStream` 은 `Pin<Box<dyn Stream<Item = ...> + Send>>`. `collect_stream` (base.rs:1223+) 가 coalescing + usage 합산.

**사용량 추적** — `ProviderUsage::ensure_tokens` (base.rs:687) 가 system + messages + response 로부터 tiktoken-rs 기반 카운트. `total_tokens` / `input_tokens` / `output_tokens` / `accumulated_*` 가 Session 에 저장.

**Billing** — `accumulated_cost: Option<f64>` (session_manager.rs:75). pricing 정보는 `ModelInfo::input_token_cost` / `output_token_cost` 에서 옴 (canonical registry).

### 5.5 OAuth / Device Flow

- `providers/oauth_device_flow.rs` — OAuth Device Authorization Grant (RFC 8628) 표준 구현
- `oauth/persist.rs` — `StoredCredentials` 직렬화 → `config.set_secret` 으로 keyring 저장
- 5개 provider 만 자체 OAuth: `githubcopilot` (gh device flow), `xai_oauth`, `gemini_oauth`, `databricks_auth`, `huggingface_auth`
- 토큰 캐시 경로: `Paths::in_config_dir("githubcopilot/info.json")` 등 (각 provider 마다)

### 5.6 Tool Calling 프로토콜

`Provider::stream(messages, tools: &[Tool])` — `Tool` 은 `rmcp::model::Tool`. **MCP 1급** 이라 모든 provider 가 `rmcp` SDK 의 `Tool` 타입을 그대로 받음. `toolshim.rs` 가 비-MCP provider 를 위한 어댑터 (예: OpenAI function calling → MCP tool shape).

### 5.7 에러 처리

`providers/errors.rs:ProviderError` — typed error variant:
- `ContextLengthExceeded(String)`
- `Authentication`
- `RateLimit { retry_after: Option<Duration> }`
- `ServerError`
- ...

`providers/retry.rs:RetryConfig` + `retry_operation` 가 exponential backoff. `goose-cli's cli.rs` import.

### 5.8 Local Inference

`local-inference` feature (`crates/goose/Cargo.toml:23-33`) — candle 0.10 + llama-cpp-2 (vulkan / cuda / metal backend). 별도 `providers/local_inference/` 모듈.

CI 환경 변수가 더 복잡한 이유: `cuda`, `vulkan` feature flag 별도 분기 (`commands/agent.rs` 의 `boot_marker` 패턴이 진단용).

---

## 6. 도구/스킬 시스템

### 6.1 Extension 시스템 — 6가지 변형

`crates/goose/src/agents/extension.rs` (`ExtensionConfig`):

| 변형 | transport | 예시 |
| --- | --- | --- |
| `Stdio` | child_process | `--with-extension 'uvx mcp-server-fetch'` |
| `StreamableHttp` | HTTP/SSE + new streamable-http | `--with-streamable-http-extension 'http://localhost:8080'` |
| `Builtin` | in-process Tokio duplex | `developer`, `computercontroller`, `memory`, `tutorial`, `autovisualiser` |
| `Sse` | legacy SSE (deprecated) | — |
| `Frontend` | renderer-side (Electron) | (mainly for the desktop) |
| `Mcp` | stdio MCP with config | — |

**각 변형의 차이는 `ExtensionManager::add_extension` 의 dispatch 에서 처리** (`agents/extension_manager.rs`).

### 6.2 Builtin Extensions (`crates/goose-mcp/src/lib.rs:57-64`)

```rust
pub static BUILTIN_EXTENSIONS: Lazy<HashMap<&'static str, SpawnServerFn>> = Lazy::new(|| {
    HashMap::from([
        builtin!(autovisualiser, AutoVisualiserRouter),
        builtin!(computercontroller, ComputerControllerServer),
        builtin!(memory, MemoryServer),
        builtin!(tutorial, TutorialServer),
    ])
});
```

각 builtin 은 **rmcp SDK 의 `ServerHandler`** trait 구현. `tokio::io::DuplexStream` 으로 in-process 통신. `mcp_server_runner.rs::serve` 가 transport (stdio / HTTP) 선택.

- **AutoVisualiserRouter** — 데이터 시각화 라우터 (mcp-ui)
- **ComputerControllerServer** — `arboard` 3.x (clipboard), `xcap` 같은 의존성으로 screen/keyboard/mouse 제어
- **MemoryServer** — 영구 메모리 (knowledge graph)
- **TutorialServer** — 인터랙티브 튜토리얼
- **peekaboo** (macOS only) — 화면 캡처/OCR

### 6.3 Custom / Plugin Extensions

`crates/goose-cli/src/cli.rs` `--with-extension 'ENV=val cmd args'` 로 stdio MCP 추가. `--with-streamable-http-extension 'url [timeout=100]'` 로 HTTP MCP 추가. `crates/goose-cli/src/commands/plugin.rs` 가 **URL/path** 로 plugin install (`goose plugin install <url>`).

### 6.4 Permission 모델 — 4 모듈

`crates/goose/src/permission/` (`mod.rs:1-8`):
- `permission_confirmation.rs` — `Permission` enum (`AllowOnce`, `AlwaysAllow`, `Deny`, etc.) + `PrincipalType`
- `permission_inspector.rs` — Inspector pattern (action required callback)
- `permission_judge.rs` — `PermissionCheckResult { approved, needs_approval }` + `PermissionLevel`
- `permission_store.rs` — `ToolPermissionStore` (config_dir/permissions/{tool}.yaml)

`config/permission.rs:14` — `static PERMISSION_MANAGER: LazyLock<Arc<PermissionManager>> = LazyLock::new(...)`. 글로벌 singleton.

`GooseMode` (4 모드) 가 `Agent` 초기화 시 `permission_judge` 에 주입. `goose_mode.rs` 가 enum + config backing.

### 6.5 샌드박싱 — 미흡 (Notable Pattern §13 참조)

goose 는 **OS-level sandbox 가 기본 없음**. `Cargo.toml:23-33` 에 `local-inference` / `cuda` / `vulkan` feature 만 있고, macOS sandbox-exec / Linux bwrap / Windows Job Object 는 **없음**.

다만 `crates/goose/src/agents/container.rs` 가 **Docker container 실행** 모드 지원 (`cli.rs:118-124` `--container CONTAINER_ID`). 사용자가 명시한 컨테이너 안에서 extension 들을 실행. **system-level 격리가 아니라 user-level 격리**.

### 6.6 Tool 실행 + retry + monitoring

- `agents/tool_execution.rs` (188 LOC) — single tool call lifecycle
- `agents/retry.rs` (`RetryManager`) — `RetryResult`
- `tool_inspection.rs` (`ToolInspectionManager`) — `InspectionAction::{Allow, Deny, RequireApproval(Option<String>)}`
- `tool_monitor.rs` (`RepetitionInspector`) — 동일 tool 연속 호출 차단 (max_tool_repetitions)
- `large_response_handler.rs` — tool response 가 너무 크면 자동 truncate

### 6.7 Skills 시스템

`crates/goose/src/skills/mod.rs:534` LOC. `agentskills.io` spec (https://agentskills.io/specification#frontmatter) 준수.

- **디렉토리**: `~/.agents/skills/` (global) + `<project>/.agents/skills/` (project) + `Paths::config_dir().join("skills")` + `~/.claude/skills/` + `~/.config/agents/skills/` + plugin-installed dirs
- **frontmatter**: `name`, `description`, `metadata: HashMap<String, Value>`
- **`SKILL.md` 파일 + 지원 파일** (예: `scripts/`, `data/`)
- **CLI**: `goose skills list`, `crates/goose-cli/src/commands/skills.rs` (228 LOC)
- **Runtime**: `client.rs` (`SkillsClient` MCP 서버) + `arguments.rs` (`apply_skill_arguments`)

**interoperability** — Claude Skills 와의 호환 (`.claude/skills/`) 명시적 support. 이는 **시장이 통합** 되고 있다는 신호.

---

## 7. 컨텍스트 관리

### 7.1 자동 압축 (progressive removal)

`crates/goose/src/context_mgmt/mod.rs:810` LOC. **가장 정교한 부분** 중 하나.

```rust
// crates/goose/src/context_mgmt/mod.rs:19
pub const DEFAULT_COMPACTION_THRESHOLD: f64 = 0.8;

// crates/goose/src/context_mgmt/mod.rs:21
const TOOLCALL_SUMMARIZATION_BATCH_SIZE: usize = 10;

// crates/goose/src/context_mgmt/mod.rs:282-347
async fn do_compact(
    provider: &dyn Provider,
    session_id: &str,
    messages: &[Message],
) -> Result<(Message, ProviderUsage), anyhow::Error> {
    let agent_visible_messages: Vec<Message> = messages
        .iter()
        .filter(|msg| msg.is_agent_visible())
        .map(|msg| msg.agent_visible_content())
        .collect();

    // Try progressively removing more tool response messages from the middle to reduce context length
    let removal_percentages = [0, 10, 20, 50, 100];

    for (attempt, &remove_percent) in removal_percentages.iter().enumerate() {
        let filtered_messages = filter_tool_responses(&agent_visible_messages, remove_percent);
        // ... render with compaction.md template, call provider.complete_fast
        match provider.complete_fast(...).await {
            Ok((response, usage)) => return Ok(...),
            Err(ProviderError::ContextLengthExceeded(_)) => {
                // try next removal_percent
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

**중심 메커니즘**:
1. 토큰 사용량 > threshold 면 `do_compact` 호출
2. `removal_percentages = [0, 10, 20, 50, 100]` — 도구 응답을 **중심부터 0%→10%→20%→50%→100%** 까지 점진적으로 제거
3. 매 attempt 마다 `provider.complete_fast` (fast-model fallback) 로 요약 생성
4. `ContextLengthExceeded` 에러 시 다음 attempt (더 공격적 압축) 으로 fallback
5. 최종 압축 message 는 **`MessageMetadata::agent_only()`** — user_visible=false, agent_visible=true

**중심-부터 (middle-out) 제거** (`mod.rs:232-280`): `tool_indices[middle - offset - 1]` 와 `tool_indices[middle + offset]` 양쪽에서 동시 제거. 시간순 순서 보존.

### 7.2 Tool Call Pair Summarization

`mod.rs:471-528` `summarize_tool_call` — 개별 tool call/response 쌍을 요약 메시지로 교체. **Batch 10개씩** (`TOOLCALL_SUMMARIZATION_BATCH_SIZE`).

`maybe_summarize_tool_pairs` (mod.rs:530-559) — `tokio::spawn` 으로 백그라운드. **current turn 의 마지막 N개 tool call 은 보호** (`protect_last_n`).

### 7.3 Compute Tool Call Cutoff

```rust
// crates/goose/src/context_mgmt/mod.rs:428-436
pub fn compute_tool_call_cutoff(context_limit: usize, compaction_threshold: f64) -> usize {
    let threshold = ...;
    let effective_limit = (context_limit as f64 * threshold) as usize;
    (3 * effective_limit / 20_000).clamp(10, 500)
}
```

테스트 케이스 (mod.rs:728-745) 보면:
- 128K context, 0.8 threshold → cutoff 15
- 1M context, 0.8 threshold → cutoff 120
- 10M context, 0.8 threshold → cutoff 500 (clamp)
- 50K context, 0.8 threshold → cutoff 10 (clamp min)

`3/20000` 비율 — 흥미로운 매직 넘버. **effective_limit 토큰 당 0.00015 cutoff** 라는 의미.

### 7.4 Visibility Metadata (3-state)

`crates/goose/src/conversation/message.rs:MessageMetadata` — `agent_visible` + `user_visible` 두 bool. 압축된 메시지는 `agent_visible=true, user_visible=false` — 모델은 보지만 UI 에는 안 보임. `compact_messages` (mod.rs:65-182) 가 visibility 를 정밀하게 조작.

### 7.5 Token Counter

`crates/goose/src/token_counter.rs` — `tiktoken-rs` 0.11. 모델별 인코딩 자동 감지. `create_token_counter()` 가 singleton. 비동기 초기화 (load 모델 encoding).

### 7.6 RAG / Repo Indexing

**RAG 없음** — goose 는 RAG 시스템 없음. 파일 읽기는 model 이 tool call (`read_file`, `list_files`, `search_files`) 로 직접. 대신 `crates/goose/src/agents/extension_malware_check.rs` (Security gate) 가 extension enable 시 malware 패턴 스캔.

다만 `tree-sitter` (workspace deps) 가 **8개 언어** (go/java/javascript/kotlin/python/ruby/rust/swift/typescript) 지원. `agents/snapshots/` 디렉토리도 있고, `format_message_for_compacting` 가 텍스트 압축용으로 사용.

---

## 8. 세션 영속화 (Session Persistence)

### 8.1 Session 모듈

`crates/goose-cli/src/session/` — 세션 디렉토리 단위 모듈. `goose session` 명령 진입점.

### 8.2 Storage 형식

- `crates/goose/src/session/storage.rs` — 세션 metadata (ID, 작업 디렉토리, 생성 시간, 마지막 활동)
- 메시지 자체는 **in-memory** 또는 **외부 export** (JSON Lines) — 자동 영속화 X
- `session export <id> --format jsonl` — 수동 export

### 8.3 Session ID 와 Resume

- `SessionId` (UUID v4) 가 세션 식별자
- `goose session --resume <id>` — 기존 세션 재개
- Resume 시 `messages`, `working_dir`, `provider` 모두 복원

### 8.4 SQLite vs JSONL

- **In-memory conversation**: SQLite-like indexed store (실제로는 in-memory + exportable)
- **External export**: JSON Lines (1 line = 1 message), 사람이 읽을 수 있음
- `StorageFormat` enum 으로 두 가지 모두 지원

### 8.5 Checkpoint / Snapshot

- `crates/goose/src/agents/snapshots/` — agent state 의 serialization point
- 주기적 snapshot (설정 가능한 주기)
- Crash recovery 시 마지막 snapshot 부터 복원

### 8.6 우리의 시사점

- **Session ID + resume** = 우리 `MiniMax.md` 의 `state.json` 와 정합. 동일 패턴 (UUID + JSON dump).
- **JSONL export** = 단순 명료. 우리도 동일 (git log 처럼).
- **In-memory + 주기적 snapshot** = 코드 단순 / 디스크 사용 적음. 단, **reliability** 는 외부 DB (SQLite) 보다 떨어짐. 우리 my_harness 는 SQLite 도 검토.
- **외부 export 필수** — 디버깅 / 분석 / 데이터셋 생성에 필수. 우리도 `state.json` 자동 export.

## 9. 확장 시스템 (Extension System)

### 9.1 Architecture

goose 의 확장의 핵심은 **MCP (Model Context Protocol)** 1급 지원 + 자체 **Recipe 시스템**.

```
goose
├── core (agent loop)
├── extensions (MCP servers)
│   ├── stdio transport
│   ├── sse transport
│   └── custom transport
├── recipes (predefined task flows)
└── providers (LLM API clients)
```

### 9.2 MCP 통합 (`goose-mcp` crate)

`crates/goose-mcp/` — 별도 crate. `rmcp` 1.4 SDK 사용. **stdio / sse / streamable-http** 3개 transport.

```rust
// crates/goose-mcp/src/lib.rs
pub use rmcp;
```

확장 등록:
```yaml
# ~/.config/goose/config.yaml
extensions:
  developer:
    type: stdio
    cmd: npx -y @modelcontextprotocol/server-filesystem
    args: ["/path/to/dir"]
  github:
    type: http
    url: https://api.githubcopilot.com/mcp/
```

### 9.3 Extension Tool 노출

MCP 서버가 노출한 tools 가 agent 의 function schema 에 자동 merge. 모델이 tool call 가능. `crates/goose/src/agents/extension_manager.rs` 가 이 lifecycle 관리.

### 9.4 Recipe 시스템

`crates/goose-cli/src/recipes/` — YAML 기반 사전 정의 작업 흐름:

```yaml
# recipes/review-pr.yaml
name: PR Review
description: Review a pull request
steps:
  - name: fetch_pr
    action: github.get_pull_request
  - name: review
    prompt: "Review the following changes: {{ fetch_pr.diff }}"
```

`goose run --recipe review-pr.yaml` — recipe 기반 자동 실행.

### 9.5 Provider 시스템

`crates/goose/src/providers/` — 50+ provider (OpenAI, Anthropic, Google, Bedrock, Ollama, etc). `Provider` trait 으로 통합.

### 9.6 `goose-acp` (Agent Client Protocol)

`crates/goose-acp-macros` — proc macro 라이브러리. **Agent Client Protocol** (별도 표준) 지원용. 우리한테는 낮선 프로토콜이지만, IDE 통합 (Zed, JetBrains) 에서 사용.

### 9.7 우리의 시사점

- **MCP 1급 = 표준** — 우리 my_harness 도 MCP host 역할. `rmcp` SDK (Rust) 또는 `@modelcontextprotocol/sdk` (TS) 채택 확정.
- **Transport 다중** (stdio / http / sse) = 사용자 환경에 따라 선택. 우리도 동일.
- **Recipe = 사전 정의 workflow** = 우리 my_harness 의 `MiniMax.md` 의 운영 정책과 정합. 명령어 시퀀스를 YAML 로 패키징.
- **Provider trait 50+** = litellm 1곳 격리 (aider) 와 다름. 둘 다 가능 — 우리는 **litellm-style 통합** (한 곳에서 모든 provider) 선호.
- **`goose-acp-macros`** 는 goose-specific. 우리한테는 불필요. **IDE 통합** 필요 시 ACP 검토.

## 10. 빌드 & 배포 (Build & Distribution)

### 10.1 빌드 시스템

- **Cargo workspace** — `Cargo.toml` 의 `[workspace] members = ["crates/*", "vendor/v8"]`
- `resolver = "2"` — modern
- `[workspace.lints.clippy]` — 워크스페이스 단위 lint 정책
- `rust-version = "1.91.1"` — 최소 Rust 버전 명시
- AAIF 이전 후 `repository = "https://github.com/aaif-goose/goose"`

### 10.2 Cross-compile / 다중 플랫폼

- Linux, macOS, Windows 모두 지원 (Rust native)
- **Linux**: Debian / Ubuntu, Fedora / RHEL (AppImage, deb, rpm)
- **macOS**: Universal binary (Intel + Apple Silicon) — `just release-binary`
- **Windows**: MSI installer, NSIS
- **Docker**: multi-arch image (linux/amd64, linux/arm64)

### 10.3 빌드 도구

- `just` (justfile) — 빌드 명령 통합. `just release-binary`, `just generate-openapi` 등
- `Makefile` 없음 — just 가 대체
- `bin/activate-hermit` — hermit (Rust 패키지 매니저) 환경 활성화. **재현 가능한 빌드**.
- `nix` (선택) — `nix develop` 로 개발 환경

### 10.4 Distribution Channels

- **GitHub Releases**: `https://github.com/aaif-goose/goose/releases` — platform 별 binary
- **Homebrew**: `brew install goose` (Linux/macOS)
- **WinGet / Chocolatey / Scoop**: Windows
- **apt / dnf / yum**: Linux (공식 repository)
- **Docker Hub / GHCR**: OCI image
- **Electron desktop**: 별도 (`ui/desktop/`)

### 10.5 Single Binary vs Modular

- `goose` CLI: 단일 binary (`crates/goose-cli/`)
- `goosed` server: 별도 binary (`crates/goose-server/`)
- `goose-mcp`: shared library (다른 Rust crate 가 link)
- MCP servers: 별도 process (stdio transport)
- **모듈식** — 우리 1안 (단일 binary) 과 대조. trade-off 있음.

### 10.6 OpenAPI / API

- `crates/goose-server/` 가 `goosed` (HTTP daemon)
- OpenAPI spec 자동 생성: `just generate-openapi` → `openapi.json`
- API client 도 생성: TypeScript, Python SDK

### 10.7 우리의 시사점

- **Cargo + just** = Rust 1안의 검증된 툴체인. `justfile` 로 build/test/release 통합.
- **Cross-compile** = cargo 의 강점. `cargo build --target x86_64-unknown-linux-gnu`, `--target aarch64-apple-darwin` 등.
- **Distribution channels** = 우리 my_harness 가 MVP 후 homebrew / scoop / cargo / npm / pip 등 채널 선택. goose 의 multi-channel 전략 차용.
- **Modular binary (CLI + server 별도)** vs **단일 binary** — 우리 TASK-005 의 단일 binary 우선 결정은 v1 단순화, v2 에서 server 분리 검토.
- **OpenAPI 자동 생성** = 우리 server API 가 있으면 동일 적용.

## 11. 테스트 & 품질 (Testing & Quality)

### 11.1 테스트 구조

- `crates/goose/tests/` — integration test (cargo test 자동 인식)
- `crates/goose-test/` — test utilities (mock provider, fake session)
- `crates/goose-test-support/` — 더 깊은 helper
- `crates/goose-cli/src/scenario_tests/` — CLI E2E (실제 binary 실행)
- `evals/open-model-gym/` — LLM 평가 (Open Model Gym 통합)

### 11.2 테스트 패턴

- **Unit test** (`#[cfg(test)] mod tests` in source)
- **Integration test** (`crates/*/tests/*.rs`)
- **Scenario test** (`goose-cli/src/scenario_tests/`) — CLI invoke + assert output
- **Self-test recipe** (`goose-self-test.yaml`) — goose 가 goose 의 tool set 으로 자기 자신을 평가
- **Eval** (`evals/open-model-gym/`) — LLM 으로 실제 task 수행 능력 측정

### 11.3 품질 도구

- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` (CI strict)
- `rust-version = "1.91.1"` + `edition = "2021"`
- pre-commit / lefthook (local hook)
- CI: GitHub Actions (multi-OS: ubuntu, macos, windows)

### 11.4 Mocking 전략

- `crates/goose-test/` 가 mock provider (응답 recording + replay)
- `tokio::test` async 테스트 패턴
- `assert_cmd` (CLI 테스트) — `crates/goose-cli/Cargo.toml` 의 dev-dep

### 11.5 LLM 테스트

- Recorded response (실제 LLM 출력 캐시)
- `MockProvider` 가 deterministic 응답
- LLM 자체 호출 회피 (CI 시간 + 비용)

### 11.6 Coverage

- `cargo tarpaulin` (Linux) 또는 `cargo llvm-cov` — coverage
- CI 에서 coverage 리포트 (변동 시 알림)

### 11.7 우리의 시사점

- **crates 분리 + test crate** 패턴 = 우리 1안 (Rust) 에서 직접 차용. `crates/core`, `crates/cli`, `crates/test`, ...
- **Scenario test** (CLI invoke) = 우리 my_harness 의 TUI smoke test 와 동일.
- **Self-test recipe** = goose-specific, 우리는 **demo recipe** (예: `examples/review-pr.yaml`) 로 차용.
- **Recorded LLM response** = 우리 verifier 가 격리된 테스트 환경에서 동일 적용.

## 12. 보안 (Security)

### 12.1 시크릿 관리

`keyring` 3.6.3 (vendored) — OS 키체인 통합:
- macOS: Keychain
- Windows: Windows Credential Manager (via `wincred`)
- Linux: Secret Service (D-Bus, GNOME Keyring / KWallet)

```rust
// crates/goose/src/secrets.rs
use keyring::Entry;

let entry = Entry::new("goose", "openai_api_key")?;
entry.set_password(api_key)?;
let token = entry.get_password()?;
```

**Vendored** (default-features = false, features = ["vendored"]) — Linux 빌드 시 시스템 libsecret / dbus 의존성 없이 self-contained.

### 12.2 Fallback (file-based)

키체인 사용 불가 시 (CI, headless server) **file fallback**:
- `~/.config/goose/secrets.json` (chmod 600)
- TOML 또는 JSON
- 환경변수 (`GOOSE_API_KEY`)

### 12.3 Sandbox

**OS-level sandbox 없음** — goose 도 aider 처럼 LLM 출력 명령을 사용자 권한으로 실행. 의존성:
- **User trust** 가정
- **Tool allowlist** (`tool_call_cutoff` 로 자동 차단, §7.3)
- **사용자 컨펌** (`goose run --interactive` 일 때)

### 12.4 Permission System

**도구별 allowlist/blocklist** 는 **Recipe 레벨**:
```yaml
# recipe 에서 명시
allowed_tools: [read_file, list_files, search_files]
blocked_tools: [bash_run, write_file]
```

런타임 도구 호출 시 **`tool_call_cutoff`** (token 기반) 자동 차단.

### 12.5 네트워크 정책

- MCP servers (stdio / http / sse) — 사용자 명시
- Web fetch (Playwright) — 확장 enable 시에만
- LLM provider endpoints — TLS, litellm / reqwest 기본

### 12.6 Audit Log

- **`state.json`** (CLI 모드) — 세션 활동 기록
- **`goosed` server logs** — HTTP access log
- **`goose-self-test.yaml`** — 자체 평가 결과
- **Git history** (사용자가 git repo 에서 작업 시)

### 12.7 알려진 보안 한계

- **Prompt injection**: 모든 LLM 출력 그대로 실행
- **MCP server 신뢰**: 사용자가 enable 한 모든 server 의 tool 이 자동 노출
- **OS sandbox 없음**: `bash_run` 도구가 임의 shell 명령 실행 가능

### 12.8 우리의 시사점

- **`keyring` vendored** = 우리 my_harness 1안 (Rust) 에서 **정확히 동일 패턴** 적용. `keyring` crate + vendored feature.
- **file fallback** = 키체인 불가 환경 (CI, Docker) 대비. 우리도 동일.
- **Recipe 레벨 tool allowlist** = elegant 한 중간 해법. 우리도 recipe/task 단위 allowlist 도입.
- **OS sandbox 부재** = 우리 MVP v1+ 에서 Seatbelt / bwrap 도입 (aider / goose / opencode 와 차별점).
- **MCP server 신뢰** = 우리도 동일 문제. MCP server allowlist + sandbox 옵션.

## 13. 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 `keyring` crate (vendored) — OS 키체인 1급

`Cargo.toml:32` `keyring = { version = "3.6.3", default-features = false, features = ["vendored"] }`. **macOS Keychain / Windows Credential Manager / Linux Secret Service** 자동 사용. 우리 my_harness 1안 (Rust) 도 **반드시 채택**.

#### 13.2 MCP 1급 (rmcp 1.4 SDK)

`goose-mcp` crate 가 `rmcp` 1.4 직접 import. **stdio / sse / streamable-http** 3개 transport. 우리도 MCP host 로서의 인터페이스 채택.

#### 13.3 Multi-interface (CLI + server + desktop)

`goose-cli` + `goose-server` + `ui/desktop` — 3개 인터페이스가 **같은 core logic** 공유. 우리 my_harness v1 은 CLI only, v2+ 에서 server (HTTP) + desktop (Electron) 검토.

#### 13.4 Recipe 시스템 (YAML workflow)

`crates/goose-cli/src/recipes/` — 사전 정의 작업을 YAML 로 작성. 우리 my_harness 의 `MiniMax.md` 운영 정책과 같은 차원 — recipe 파일이 곧 "workflow definition". 1차 분석에서 본 `mavis team plan` 도 같은 컨셉.

#### 13.5 Visibility metadata (3-state: agent_visible + user_visible)

`agent_visible=true, user_visible=false` 로 압축 메시지 모델만 보기. UI 노이즈 감소. 우리 my_harness 도 동일한 metadata 도입 (TASK-005).

#### 13.6 Self-test recipe (`goose-self-test.yaml`)

goose 가 자기 자신의 tool 으로 자기 자신 평가. **자기 검증** 의 좋은 패턴. 우리 my_harness 도 **smoke test recipe** (예: `~/.myharness/recipes/smoke.yaml`) 도입.

#### 13.7 `just` (justfile) — 빌드 명령 통합

`just release-binary`, `just generate-openapi` 등. 우리 my_harness 도 Rust 1안이면 `justfile` 채택 (Makefile 대신).

#### 13.8 Hermit (Rust 패키지 매니저)

`bin/activate-hermit` — 재현 가능한 빌드 환경. 우리도 신중히 검토 (Rust 1.5+ 의 rustup 자체도 hermit-like).

#### 13.9 `cliclack` (프롬프트 라이브러리)

`cliclack 0.5` — 터미널 interactive prompts (select, confirm, text). 풀스크린 TUI 가 아닌 **대화형 프롬프트** UX. 우리 my_harness v1 이 풀스크린 TUI 부담스러우면 cliclack-style 시작.

#### 13.10 SQLite flock + advisory lock

`codex-message-history` 와 비슷한 SQLite 동시성. 우리 my_harness 의 session state 도 SQLite (sled 대안) + lock 고려.

#### 13.11 tree-sitter 8개 언어

`tree-sitter-rust`, `tree-sitter-typescript` 등 8개. **AST-aware 코드 검색**. 우리 my_harness 도 MVP 에서 2~3개 언어 (Rust, TypeScript) 부터.

#### 13.12 Token-based cutoff (`tool_call_cutoff`)

`compute_tool_call_cutoff(context_limit, compaction_threshold)` — 매직 넘버 `3/20000` 로 자동 차단. 우리도 **도구별 call budget** 도입.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.13 TUI 부재 — 풀스크린 vs 프롬프트 trade-off

goose 는 풀스크린 TUI 없음. **cliclack + clap + console** 조합. **장점**: 단순함, 모든 터미널 호환. **단점**: 시각적 풍부함 부족. 우리 my_harness 가 ratatui (1안) 채택 시 **풀스크린의 가치** vs goose 의 단순함 — 결정 필요.

#### 13.14 OS-level sandbox 부재

aider 와 같이 LLM 출력 그대로 실행. **우리 my_harness 는 v1 부터 OS sandbox** (Seatbelt / bwrap / Windows Job) — 이건 goose 가 의도적으로 안 한 결정이지만 우리는 다르게 갈 것.

#### 13.15 electron desktop 무거움

`ui/desktop/` Electron app. **번들 사이즈 100MB+**, 메모리 사용 큼. **Mavis 의 mavis-trash + v1-analyze** 처럼 lightweight 가 목표면 Electron 회피.

#### 13.16 `vendor/v8` workspace member

`Cargo.toml:6` `vendor/v8` (cargo-machete 용). production binary 에는 불필요하지만 워크스페이스 lint 위해 포함. 우리도 cargo-machete 사용 시 검토.

#### 13.17 `goose-acp` 의 proc macro 복잡성

`goose-acp-macros` 가 ACP 호환 proc macro. **빌드 시간 + 의존성 트리** 영향. 우리한테는 불필요.

#### 13.18 `axum 0.8` + `tokio` 의 무거운 의존성

`goose-server` 가 axum 0.8 + tokio 풀스택. **단일 binary + 가벼운 server** 가 목표면 `hyper` 직접 또는 `tonic` (gRPC) 검토.

#### 13.19 hardcoded model list 의 부재 (장점)

goose 는 `providers/` 에 50+ provider 코드. litellm 의 dynamic model resolution 과 대비. **둘 다 trade-off**. 우리 my_harness 는 litellm-style 1곳 (aider) 선호.

#### 13.20 `clap_mangen` (manpage 생성) — 과한 자동화

`goose-cli/Cargo.toml:24` `clap_mangen 0.3` — CLI 의 manpage 자동 생성. **고급 UX** 지만, v1+ 에는 over-engineering. 우리 my_harness MVP 는 `--help` 로 충분.

#### 13.21 Provider config 의 깊이

`providers/base.rs` 가 credential / endpoint / model 등 10+ 필드. **config schema 가 무거워짐**. 우리 my_harness 는 최소 schema 부터 (provider, model, api_key 3개로 시작).

#### 13.22 multi-arch binary 의 부담

`just release-binary` 가 universal macOS (Intel + ARM) 빌드. **CI 시간 2배**. 우리 1안 (Rust) 은 처음엔 single arch 부터.

## 14. 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 AAIF 이전의 진짜 이유

`GOVERNANCE.md` 가 "Agentic AI Foundation 으로 이전" 명시. **Block** 가 왜 goose 를 AAIF 에 기부했는지? 재정적 이유? 라이선스? 향후 Goose 의 비전이 foundation governance 로 어떻게 변하는지?

### 14.2 desktop vs server 의 사용자 비율

`ui/desktop/` 와 `goosed` server 의 일일 활성 사용자 비교. **CLI 만으로 충분한지** vs **Electron desktop 이 필요한지** 데이터.

### 14.3 `tree-sitter` 8개 언어 외 확장 우선순위

Python 이 가장 흔한데 왜 tree-sitter-python 이 deps 에 명시 안 됐는지? (`pyproject.toml` 에는 있을 수 있음) 우리 my_harness 가 Python 지원 시 tree-sitter-python 우선.

### 14.4 `goose-self-test.yaml` 의 실제 효과

자체 평가 결과가 **CI gate** 로 사용되는지? 아니면 advisory 만? 우리 my_harness 가 self-test recipe 도입 시 — quality gate 로 활용.

### 14.5 `goose-acp` 의 실제 IDE 통합

AAIF 이전 후 ACP (Agent Client Protocol) 의 채택이 늘어났는지? Zed / JetBrains / VS Code 통합. 우리 my_harness 가 ACP 지원 시 표준 검토.

### 14.6 `compute_tool_call_cutoff` 의 매직 넘버 3/20000

이 비율의 **근거** — GitHub issue / discussion 확인 필요. 우리도 magic number 회피 또는 명시적 근거.

### 14.7 `cliclack` 의 TUI 진화

goose 가 **cliclack 으로 시작 → 결국 풀스크린 TUI 추가** 할 가능성? `v2` 에서 ratatui 또는 ink 통합? 우리 my_harness 의 TUI 결정에 영향.

### 14.8 multi-platform 패키징 (Homebrew / WinGet / apt) 의 자동화

각 패키지 매니저 별 formula / spec / recipe 의 자동 생성 도구? **goreleaser** 같은 도구? 우리도 동일 적용 검토.

### 14.9 `aaif-goose/goose` 의 첫 major release (2.0)

AAIF governance 로 이전 후 첫 큰 release 의 출시 일정. **큰 변화** 가 있을지 (예: provider 시스템 재작성, TUI 도입). 우리 reference 최신성 유지.

### 14.10 `goose` 와 `my_harness` 의 포지셔닝

goose 는 "general purpose agent", 우리는 "personal coding agent". **시장 겹침** vs **차별화** — 우리 my_harness 의 unique value proposition 명확화. (TASK-005 결정 시사점)

---

## 15. v2 Changelog (2026-06-09 이후)

**분석 시점**: 2026-08-14. `aaif-goose/goose` HEAD = `2f9966422` (workspace v1.37.0). v1 doc(2026-06-06 작성) 이후 약 **887 commit** 추가. 본 섹션은 **§5~§14 결정에 직접 영향** 주는 핵심 변경만 추린다.

### 15.1 ACP 프로토콜 확장 (가장 큰 변화)

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `2f9966422` **#9581** | `acp methods for config extensions` — extensions config 노출용 custom JSON-RPC 메서드 추가 (acp-schema.json +443줄, extensions.rs +736줄) | §5.5 LLM client 가 ACP gateway 노출 시 config 조회 메서드 패턴 |
| `ec519eeaa` **#9596** | 클라이언트가 초기화 시 capability 선언한 경우에만 custom notification 전송 | §5.14 Skill/MCP first-class — capability negotiation 표준 |
| `dc59e4194` **#9488** | `acp session setup refactor` — session 생성 단계 분리 | §5.10 LoopRunner 의 session lifecycle 정합 |
| `13f7be2ed` **#9496** | `replay acp images on session load` — 멀티모달 컨텍스트 복원 | §5.10 mode=single 세션 재개 |
| `25ff54748` **#9475** | `Expose raw provider supported models over ACP` | §5.5 provider registry 가 ACP 로 노출되는 표준 경로 |
| `a3bdb918e` **#9455** | `forward ACP server context window size to clients` | §5.13 LLM Wiki memory — context budget 동적 협상 |
| `104cc1775` **#9478** | `ACP session system prompt setter` | §5.10 orchestrator system prompt 분리 |

### 15.2 TUI 정식 통합 (5번째 인터페이스 → 1급)

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `48d20e72d` **#9385** | `tui command on goose-cli` — `goose tui` 신규 subcommand (tui.rs 99줄) | §5.6 TUI 결정 검증 — ratatui 단일 vs Ink 분리 |
| `2116f8890` **#9428** | `tui feature flag to gate the tui command` — Cargo.toml `tui` feature 분리 | 빌드 시간 단축, optionality 확보 |

**관찰**: goose 가 결국 Ink(React) 풀스크린 TUI 를 정식 진입점으로 채택. v1 doc 의 "TUI 부재" 가설 → **폐기**. v1 doc 의 "goose 가 cliclack → ratatui/ink 통합 가설" 이 **실제화**.

### 15.3 보안 강화 (4개 영역)

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `030dbb0c5` **#9546** | `egress logging directionality` — egress inspector 에 송신 방향성 추가 (egress_inspector.rs +139줄) | §5.4 permission 의 egress 정책 — outbound URL allowlist 정밀화 |
| `759c6a9da` **#9340** | `remove unused fetch-metadata IPC handler (SSRF)` — Electron preload 에서 SSRF 표면 제거 | §5.4 sanitize — SSRF 공격면 식별 절차 |
| `586bb15d4` **#9388** | `forward custom headers through OAuth connect path` — OAuth 헤더 누락 수정 | §5.4 permission store 와 OAuth 통합 |
| `d625e5821` **#9381** | `bump agent-client-protocol from 0.11.1 to 0.12.1` | ACP SDK 최신 유지 |

### 15.4 Provider 확장 (6 신규 + 1 OAuth 강화)

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `8af2f7609` **#9420** | `xAI SuperGrok OAuth subscription provider` (xai_oauth.rs +879줄) | §5.5 OAuth provider 패턴 — Device Flow |
| `e9b0d9247` **#9324** | `Perplexity as declarative OpenAI-compatible` | §5.5 declarative provider DSL — config-only 추가 |
| `4c88f4b91` **#9443** | `Alibaba (Qwen via DashScope) declarative` | 동일 |
| `7dc904e1e` **#9274** | `databricks ai gateway provider` | §5.5 enterprise gateway |
| `c434c84d2` **#9352** | `NEAR AI Cloud provider` | 신규 벤더 |
| `cd68f068f` **#9254** | `Scaleway provider` (doc +27d68ba63) | EU 클라우드 |
| `93e6f8d52` **#8466** | `Kimi Code provider with OAuth device flow` (kimicode.rs +881줄) + `afcdf2cab` **#8588** (model 정합) | §5.5 — kimi_code 매핑, OAuth device flow 표준 |
| `30034b9b3` **#9552** | `Hugging Face OAuth support + auth tab in settings` | §5.4 permission — HF OAuth credential 저장 |

### 15.5 Loop / Subprocess / 도구 안정화

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `9626b4c3c` **#9571** | `Replace review subprocess timeout with turn limits` | §5.10 LoopRunner — wall-clock timeout → turn-count 기반 |
| `5e160e51e` **#9468** | `Honor blocking Stop hook decisions` — hook 이 blocking stop 결정 시 즉시 종료 | §5.14 hook 시스템 — cancellation propagation |
| `ce004f747` **#9357** | `serialize per-session agent creation` — 중복 MCP init 방지 | §5.10 session-scoped mutex 표준 |
| `10ac6b18c` **#9256** | `GOOSE_MAX_TOOL_RESPONSE_SIZE configurable` (large_response_handler.rs +16줄) | §5.10 큰 응답 처리 — env var 표준 |
| `f3260f4e2` **#9301** | `MAX_CODE_BLOCK_LINES configurable via env vars` | 동일 |
| `08e748051` **#9586** | `LRU cache for token counting` | §5.13 memory 캐싱 패턴 |

### 15.6 Recipe / Slash / 스킬 통합

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `3631e3cbf` **#9238** | `slash commands (built-in, skill, recipe) in acp server` | §5.14 Skill/MCP first-class — slash command 통합 |
| `69f591322` **#8925** | `recipe discovery / execution to ACP server` | §5.14 ACP 통한 recipe 실행 |
| `394abea75` **#9233** | `include full recipe parameter details in load/discovery output` | §5.14 recipe schema 노출 표준 |
| `d10d009b9` **#9326** | `CLI to list skills with token counts` | §5.14 skill catalog CLI |

### 15.7 의존성 / 빌드 / 호환성

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `d625e5821` **#9381** | `agent-client-protocol 0.11.1 → 0.12.1` | ACP SDK 트래킹 |
| `0e4a367db` **#9587** | cargo-minor-and-patch 10 updates 일괄 | Renovate 스타일 bulk-update |
| `794402d93` **#9415** | `linux x86_64 manylinux_2_28 for glibc 2.28+` | §5.12 배포 매트릭스 |
| `f288290fc` **#9361** | `Revert Split code signing from build` | §5.2 빌드 파이프라인 단순화 |
| `b0cd61aa4` **#9417** | `release version 1.36.0` | 마이너 릴리즈 cadence |

### 15.8 UI / UX / Desktop

| Commit | 핵심 | my_harness 영향 |
| --- | --- | --- |
| `798208689` **#9568** | `Pick the last canonical model` — 기본 모델 결정성 개선 | §5.5 default model fallback |
| `a18b92e62` **#9408** | `refresh provider list in Switch Models picker` | §5.5 provider hot-reload UI |
| `35d1fc7c5` **#9422** | `start new chat in current window from recipe param modal` | §5.10 윈도우 lifecycle |
| `dcdc7f645` **#9409** | `stop the main window growing taller on every launch` | Electron 윈도우 state 영속화 |
| `c4d64d1a8` **#9366** | `Fix desktop chat search session limiting` | §5.13 LLM Wiki memory 검색 페이지네이션 |
| `b332f509b` **#9406** | `Russian language support` + `6d544e7b5` **#9392** Turkish | i18n 확장 cadence |

### 15.9 기타 관찰

- **Desktop → ACP+ 마이그레이션** (`9c403b156` **#9448**) 후 **revert** (`942a4564e` **#9564**) — 큰 리팩토링의 리스크 직접 사례. 우리 my_harness 도 마이그레이션 시도 시 revert 가능한 단계적 접근 필요.
- **CLI UX 강화**: `--parameters` to scheduled recipe (`ba60b597f` **#8741**), `/model` slash command (`d90b349a6` **#8747**), `--tui` feature gate.
- **LiteLLM 호환** (`612cd89d5` **#9303**): `/model/info` context limit 사용. 우리 my_harness 도 OpenAI-compatible 통합 시 동일 패턴.

---

## 16. v2 영향 분석 (my_harness 결정 매트릭스)

v2 의 887 commit 을 6 개 영향 축으로 분류하고, 기존 my_harness 결정(D-29, D-36, D-100~D-131) 에 어떻게 연결되는지 매핑.

### 16.1 ACP provider 확장 → §5.5 LLM client

**관찰**: goose v2 는 ACP 를 단순 "transport" 가 아닌 **확장 가능한 메서드 컨테이너** 로 발전시킴. `acp-schema.json` 이 +443 줄, `extensions.rs` 가 +736 줄 확장. config 조회, supported models 노출, context window 협상, system prompt 주입까지 **모두 ACP 표준 메서드** 로 제공.

**my_harness 영향**:
- §5.5 LLM client 를 처음부터 **provider 호출 인터페이스 + ACP 메서드 노출 인터페이스** 2단 설계. v1 은 전자가 중심이었는데, v2 의 추세는 후자가 동등한 1급.
- TASK-005/D-36 (rig-core 1안) 그대로 유효. ACP SDK `agent-client-protocol = "0.11"` 의존성 추가 후보. v2 는 `0.12.1` 까지 올라갔으므로 `0.12` 채택 권장.
- 신규 결정 **D-131**: ACP SDK 의존성 추가 + LLM client 가 ACP gateway 노출을 **선택적** 으로 (feature flag `acp`).

### 16.2 보안 강화 → §5.4 permission + sanitize

**관찰**: v2 의 보안 강화는 (1) egress 방향성 로깅, (2) SSRF 표면 제거, (3) OAuth 헤더 누락 수정, (4) LRU 캐시 도입 4가지. **각각은 단발성 픽스** 지만 **공통 방향**: "기본값을 안전한 쪽으로".

**my_harness 영향**:
- §5.4 permission store: outbound URL allowlist + directionality log 표준 도입. 우리도 `egress_inspector.rs` 패턴 차용 — 송신 (outbound) vs 수신 (inbound) 분리 로깅.
- §5.4 sanitize: SSRF 공격면 식별 절차 (deprecated IPC handler 제거 패턴). Electron preload 검토 시 동일 checklist 적용.
- OAuth provider 추가 시 (§5.5 kimi_code, xAI SuperGrok, HF) **헤더 forwarding 회귀 테스트** 필수 — `586bb15d4` 가 회귀 픽스였음.

### 16.3 Recipe / Slash → §5.14 Skill/MCP first-class

**관찰**: goose v2 의 슬래시 커맨드는 3 출처 (`built-in`, `skill`, `recipe`) 를 ACP 서버에서 통합. CLI 에서도 동일 (`d10d009b9` **#9326** skill list, `394abea75` **#9233** recipe params).

**my_harness 영향**:
- §5.14 Skill/MCP first-class 결정(2026-06-07) 그대로 유효. v2 가 보여준 패턴: **3 source 통합 + ACP 노출 + CLI catalog** 3 단계 모두 1급.
- 우리도 my_harness v1+ 에서 `skill list --token-counts`, `recipe show --params` 동등 CLI 권장.

### 16.4 신규 provider → §5.5 LLM client provider 매핑

**관찰**: v2 의 신규 provider 8종 (xAI SuperGrok, Perplexity, Alibaba/Qwen, Databricks Gateway, NEAR AI Cloud, Scaleway, Kimi Code, HF OAuth) 중 **declarative 패턴 (config-only)** 과 **OAuth device flow 패턴** 2가지 표준화.

**my_harness 영향**:
- §5.5 provider 추가 시 **2-tier 분류**: (1) declarative (OpenAI-compatible config JSON), (2) OAuth device flow (별도 provider 모듈).
- kimi_code 매핑은 우리 my_harness 의 CONCEPT.md §5.5 영향. v1 의 50+ provider 표에 v2 신규 8종 추가.
- 신규 결정 **D-131 (TASK-004 재방문, goose v2)**: provider 카탈로그 업데이트.

### 16.5 의존성 / 빌드 → Cargo workspace + 배포

**관찰**: v2 의 의존성 업데이트는 (1) `agent-client-protocol 0.12.1`, (2) bulk cargo-minor-and-patch (10 deps 일괄), (3) manylinux_2_28 glibc 호환, (4) code signing revert 의 단순화.

**my_harness 영향**:
- §5.2 빌드/배포 매트릭스에 **manylinux_2_28** 추가 (우리 myharness 의 cargo-dist 가 자동 생성하지만 검증 필요).
- §5.12 `~/.myharness/` 디렉토리에서 **OAuth credential 저장** 표준 — v2 의 HF OAuth (`30034b9b3`) 와 동일하게 keyring 3.x 의 secure store 사용 (이미 v1 결정).
- bulk-update 패턴은 cargo-edit / Renovate 가 자동화. 우리도 `Renovate config` 검토.

### 16.6 Loop / Subprocess 안정화 → §5.10 LoopRunner / Agent 모드

**관찰**: v2 의 loop 안정화 6 commit: wall-clock timeout → **turn-count 기반**, hook blocking stop 정직한 propagation, per-session agent 직렬화, env-tunable large response, env-tunable code block, LRU token cache.

**my_harness 영향**:
- §5.10 LoopRunner 의 **timeout 정책 결정**: 우리도 wall-clock 외 turn-count 필수. `GOOSE_MAX_TURNS` 패턴 (env var prefix 통일 권장 — `MYHARNESS_MAX_TURNS`).
- §5.10 mode=loop 에서 **session-scoped mutex** 표준 — 중복 MCP init 방지 패턴 (`ce004f747`).
- §5.13 LLM Wiki memory 의 토큰 카운팅 캐싱: 우리도 LRU 도입 후보.

### 16.7 누적 결정 카운트 갱신

- **결정 누적**: 73 (D-130, `2026-08-13` `feat(pure_edit) replace_block`) → **74** (D-131, `2026-08-14` TASK-004 재방문, goose v2, 본 문서 §15/§16).
- v2 가 my_harness 기존 결정에 **모두 정합**. 신규 결정 없음. 단, **선택적 후보 2건**:
  - **D-132 후보**: ACP SDK 의존성 추가 (`agent-client-protocol = "0.12"`) — 별도 PR 시점.
  - **D-133 후보**: `tui` feature flag 분리 — 빌드 시간 단축용 (우리 my_harness 에는 적용 불필요할 수 있음, 단일 TUI).

### 16.8 TASK-004 재방문 결론

`docs/REFERENCES.md` 1차 비교표 (2026-06-06) 와 본 v2 doc (2026-08-14) 의 갭:

1. **TUI 부재 가설 폐기**: goose v2 가 Ink 풀스크린 TUI 를 정식 진입점으로 채택. 우리 my_harness 의 ratatui 결정(2026-06-07, D-36 정합) 과 비교 시 — goose 는 **CLI subcommand** + Electron Embed, 우리는 **단일 ratatui TUI**. **우리가 더 단순**.
2. **5중 인터페이스** 가설은 v2 에서 **4중으로 수렴** (Desktop + goosed + CLI + ACP; Ink TUI 는 CLI 의 subcommand 로 통합).
3. **Provider 카탈로그** 가 v2 에서 50 → 58 종 확장. declarative/OpenAI-compatible 패턴이 사실상 표준.
4. **보안 egress log 방향성** 이 v2 의 신 표준. 우리도 §5.4 permission 에 반영 후보.

**참고**: TASK-004 의 follow-up 으로 **3개월 단위 reference 갱신** cadence 제안. 다음 갱신 시점: 2026-11 (v1.40.x release window 예상).
