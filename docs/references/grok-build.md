# grok-build (xai-org/grok-build) 심층 코드 분석

- 문서 목적: SpaceXAI `Grok Build` (`grok` CLI/TUI) 의 실제 코드 베이스를 14섹션으로 분석해, my_harness 가 이를 8번째 reference 로 쓰고 뼈대/오버레이 결정을 코드 인용으로 뒷받침한다.
- 범위: 로컬 클론 `/Users/yklee/repos/grok-build` (`origin xai-org/grok-build`). workspace members 79 (codegen 62 + common 11 + build 1 + prod/mc 1 + third_party 4). 설치된 바이너리 `grok 1.0.3`. 시크릿/토큰 값은 인용하지 않음.
- 대상 독자: yklee, 오케스트레이터, TASK-004 / v3 reset 리뷰
- 상태: 2차 심층 (2026-08-14). 코드 인용은 클론 상대 경로.
- 최종 수정일: 2026-08-14
- 관련 문서: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [README.md](./README.md), [CONCEPT.md](../CONCEPT.md), [goose.md](./goose.md)

---

## 1. 개요 (Overview)

| 항목 | 값 |
| --- | --- |
| 정식 명칭 | Grok Build (`grok`) |
| 한 줄 | SpaceXAI 풀스크린 TUI 코딩 에이전트. headless / ACP / leader 동시 지원 |
| 메인 binary | 소스 산출물 `xai-grok-pager` · clap 이름 / 공식 설치명 `grok` |
| 패키지 버전 | `0.2.106` (`xai-grok-pager-bin/Cargo.toml`) |
| 설치 버전 | `grok 1.0.3 (1a29d5bc12d4) [stable]` (yklee, 2026-08-14) |
| 라이선스 | Apache 2.0. `LICENSE:1` `Copyright 2023-2026 SpaceXAI` |
| 오픈소스 | 2026-07-15. **외부 PR 거부** (`CONTRIBUTING.md`) |
| 거버넌스 | SpaceXAI 모노레포 주기 sync. `SOURCE_REV` = `ba69d70c2f7d70a130a323b2becdf137af784c7f` |
| 공개 커밋 | 5 (publish + `Synced from monorepo`) |
| Rust | 1.92.0. 크로스 타깃 linux gnu x86_64 / aarch64 |
| LOC | `*.rs` 1,362,619 (test 포함). 상위: pager 431k / shell 345k / tools 115k / workspace 80k |
| 홈 | `$GROK_HOME` 기본 `~/.grok` |

버전 상수:

```6:10:crates/codegen/xai-grok-version/src/lib.rs
pub const VERSION: &str = match option_env!("GROK_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
```

composition-root:

```9:16:crates/codegen/xai-grok-pager-bin/Cargo.toml
# Composition-root binary for the Grok Build TUI. The artifact is still named
# `xai-grok-pager`.
[[bin]]
name = "xai-grok-pager"
path = "src/main.rs"
```

7 reference 가 나눠 가진 것(5 components · OS sandbox · ACP · skills/hooks/MCP · 3 API backend)이 **한 제품에 이미 조립**되어 있다. 동시에 로그인·구독·remote settings·updater·이미지/비디오가 xAI 제품에 묶인 **모놀리스**다. 공개 트리는 커뮤니티 프로젝트가 아니라 모노레포 덤프.

---

## 2. 아키텍처 (Architecture)

### 2.1 프로세스 토폴로지

```
grok (xai-grok-pager-bin)
├─ TUI  xai-grok-pager  ──ACP──►  MvpAgent (in-process thread)
│                                      = xai-grok-shell
│                                      ├ sampler (3 backend)
│                                      ├ tools + workspace
│                                      ├ session JSONL
│                                      └ MCP / hooks / skills / plugins
├─ grok -p …            headless one-shot
├─ grok agent stdio     ACP JSON-RPC on stdout
├─ grok agent serve     WS :2419
└─ grok agent leader    ~/.grok/leader.sock  (다른 클라이언트가 공유)
```

기본 TUI 는 **별도 `grok-shell` 바이너리를 exec 하지 않는다.** pager 가 in-process 스레드로 `MvpAgent` 를 띄운다.

```36:89:crates/codegen/xai-grok-pager/src/acp/spawn.rs
pub async fn spawn_grok_shell(...) -> Result<SpawnedAgent> {
    let (acp_client, acp_agent) = acp_channels();
    let spawn_fn = Box::new(move |client_tx| {
        let gateway = AcpGatewaySender::new(client_tx);
        let mut agent = MvpAgent::with_models(gateway, &agent_config, auth_manager, models_manager);
        Ok(Rc::new(agent))
    });
    let handle = spawn_agent_thread_direct(spawn_fn, acp_agent, agent_cancel.clone())?;
```

주석: *"Simplified to only support GrokShell (in-process) mode."*

leader 모드일 때만 자기 자신을 `agent leader` 로 spawn:

```1390:1412:crates/codegen/xai-grok-shell/src/leader/mod.rs
fn spawn_leader_subprocess(env_urls: &LeaderEnvUrls) -> Result<u32, ConnectionError> {
    let exe = resolve_exe_for_spawn()?;
    let mut cmd = Command::new(exe);
    cmd.arg("agent").arg("leader");
    cmd.arg("--no-exit-on-disconnect");
```

TUI 연결 분기 (`app/mod.rs:619-632`): `use_leader` → `connect_via_leader`, 아니면 `acp::connect`.

### 2.2 진입 `main`

```1607:1672:crates/codegen/xai-grok-pager-bin/src/main.rs
fn main() {
    xai_grok_pager_minimal::install();
    // jemalloc / crash handler / mermaid worker / sentry / user-guide extract
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| panic!("failed to start tokio runtime: {e}"));
    let result = run_and_shutdown(runtime, async_main(), RUNTIME_SHUTDOWN_GRACE);
```

`async_main` → `PagerArgs::parse_and_apply_cwd()` → 서브커맨드 / `-p` / `app::run`.

### 2.3 핵심 crate

| crate | 역할 |
| --- | --- |
| `xai-grok-pager-bin` | composition root. jemalloc + sandbox-enforce |
| `xai-grok-pager` | TUI, clap, slash, ACP 클라이언트 |
| `xai-grok-shell` | `MvpAgent`, auth, MCP, sampling 연결, subagent, session |
| `xai-grok-agent` | agent `.md`, 프롬프트 조립, skill/plugin discovery |
| `xai-grok-tools` | 레지스트리 + grok_build/codex/opencode/hashline 구현 |
| `xai-grok-workspace` | FS / VCS / checkpoint / **permission 파이프라인** |
| `xai-grok-sampler` | 3 backend 스트리밍 + retry |
| `xai-grok-mcp` | rmcp 2.1 격리 (reqwest 0.13) |
| `xai-grok-hooks` | 15 이벤트, PreToolUse/Stop 게이트 |
| `xai-grok-memory` | experimental FTS5 + optional sqlite-vec |
| `xai-grok-compaction` | intra/inter compact |
| `xai-grok-sandbox` | Landlock / Seatbelt (`nono`) |
| `xai-acp-lib` | ACP JSON-RPC |
| `xai-grok-plugin-marketplace` | git marketplace |

루트 `Cargo.toml:1`: `# Auto-generated workspace root. Prefer editing per-crate Cargo.toml files.`

### 2.4 디렉터리 트리 (실측)

```
crates/codegen/          62 crate
  xai-grok-pager-bin/    composition root
  xai-grok-pager/        TUI
    src/acp/             in-process MvpAgent spawn + leader connect
    src/actions/         키 바인딩 단일 소스
    src/app/             event_loop, app_view, cli.rs
    src/slash/commands/  64 slash 구현
    src/views/           모달 / dashboard
  xai-grok-pager-render/ theme + draw_frame
  xai-grok-pager-minimal/  --minimal IoC hooks
  xai-grok-shell/        런타임
    src/agent/           MvpAgent, models, config
    src/auth/            grok.com OAuth / device / auth.json
    src/session/         JSONL persistence, compaction, rewind
    src/leader/          leader.sock
    src/sampling/        sampler 연결
  xai-grok-agent/        prompt, skills, plugins, agents_md
  xai-grok-tools/        registry + implementations/{grok_build,codex,opencode,hashline}
  xai-grok-workspace/    permission/, git, checkpoints
  xai-grok-sampler/      3 backend stream + retry
  xai-grok-mcp/          rmcp 2.1 격리
  xai-grok-hooks/
  xai-grok-sandbox/
  xai-grok-memory/
  xai-codebase-graph/
  xai-grok-plugin-marketplace/
  … (auth/config/update/telemetry/voice/…)
crates/common/           11 crate (compaction, tool-protocol/runtime/types, computer-hub)
prod/mc/cli-chat-proxy-types/
third_party/             Mermaid 스택
```

pager `src/` 와 shell `src/` 가 각각 제품의 절반이다. LOC 도 pager 431k + shell 345k.

---

## 3. 진입점 & CLI

정의: `crates/codegen/xai-grok-pager/src/app/cli.rs`. clap 프로그램 이름 `grok`.

### 3.1 서브커맨드 트리

`Command` (`cli.rs:8-141`):

```
grok
├── (no subcommand)          풀스크린 TUI. 위치 인자 PROMPT = 초기 메시지
├── -p / --single / --print  headless one-shot
├── agent
│   ├── stdio                ACP on stdout
│   ├── headless
│   ├── serve [--bind 127.0.0.1:2419] [--secret]
│   └── leader
├── inspect [--json]
├── leader {list|info|kill}
├── login [--oauth] [--device-auth]
├── logout
├── mcp {list|add|remove|doctor}
├── plugin {list|install|uninstall|enable|disable|marketplace …}
├── memory clear
├── models
├── sessions {list|search|delete}
├── setup / update / version / completions
├── wrap <CMD>               OSC 52 clipboard PTY
├── export / trace / share(hidden)
├── worktree {list|show|rm|gc|db}
├── workspace (hidden)       Computer Hub
└── dashboard
```

`-p` 정의 (`cli.rs:482-491`):

```482:491:crates/codegen/xai-grok-pager/src/app/cli.rs
    /// Single-turn prompt. Prints the response to stdout and exits.
    #[clap(
        short = 'p',
        long = "single",
        alias = "print",
        value_name = "PROMPT",
        conflicts_with_all = &["prompt_json", "prompt_file"]
    )]
    pub single: Option<String>,
```

동위: `--prompt-json`, `--prompt-file`, `--output-format plain|json|streaming-json|streaming-messages-json`.

### 3.2 자주 쓰는 플래그

| 플래그 | 역할 |
| --- | --- |
| `-m / --model` | 모델 ID |
| `--always-approve` / `--yolo` | 툴 자동 승인 |
| `--permission-mode` | `default` / `acceptEdits` / `auto` / `dontAsk` / `bypassPermissions` / `plan` |
| `--allow` / `--deny` | 권한 규칙 (alias `--allowedTools`) |
| `--agent` / `--agents JSON` | 에이전트 선택 / 인라인 정의 |
| `--plugin-dir` | 이번 프로세스만, **자동 trust** |
| `--sandbox` / `GROK_SANDBOX` | OS 샌드박스 프로필 |
| `-r / --resume`, `-c / --continue`, `--fork-session` | 세션 |
| `-w / --worktree` | isolated git worktree |
| `--experimental-memory` / `--no-memory` | 크로스 세션 메모리 |
| `--system-prompt-override` | 기본 프롬프트+rules 통째 교체 |
| `--minimal` / `--fullscreen` | screen_mode |
| `--reasoning-effort` / `--effort` | reasoning 티어 |

`parse_and_apply_cwd` (`cli.rs:774-783`) 는 argv0 가 `grok` 또는 `agent` 일 때만 그 이름을 clap 에 넘긴다. 래퍼 심볼릭 링크의 도움말 브랜드가 남는 이유.

### 3.3 `async_main` dispatch

`pager-bin/src/main.rs:1681-2025` 대략:

1. `PagerArgs::parse_and_apply_cwd`
2. `Command::*` 이면 해당 핸들러 (`login` / `plugin` / `agent` / …) 후 exit
3. `agent` 이고 `use_leader` 이면 `connect_or_spawn` (stdio/headless)
4. `agent` 로컬: `Stdio → run_stdio_agent`, `Serve → run_agent_server`, `Leader → run_leader`, 나머지 `run_headless`
5. `-p` / `--prompt-json` / `--prompt-file` → `run_single_turn`
6. 그 외 → `xai_grok_pager::app::run` (TUI)

CONCEPT §5.2 의 `code|server|env` 서브커맨드는 **없다**. 도메인 동사는 래퍼가 `-p` / `--agent` / slash 로 번역해야 한다.

---

## 4. TUI/UI 구현

### 4.1 스택

- ratatui **0.29** + crossterm (`unstable-widget-ref`, `unstable-backend-writer`)
- 자체 `xai-ratatui-textarea`, `xai-ratatui-inline`
- 렌더 분리 crate `xai-grok-pager-render`

### 4.2 루프

`app::run` → `event_loop::run`. 입력 스레드 + `crossterm::event::poll/read`. dirty 일 때만 `app.draw(terminal)`.

```3795:3810:crates/codegen/xai-grok-pager/src/app/app_view.rs
    pub fn draw(&mut self, terminal: &mut PagerTerminal) {
        self.draw_inner(terminal);
    }
    fn draw_inner(&mut self, terminal: &mut PagerTerminal) {
        if self.screen_mode.is_minimal() {
            if let Some(hooks) = crate::minimal_hook::hooks() {
                (hooks.draw)(self, terminal);
            }
            return;
        }
```

주석: ratatui `try_draw` 를 우회하고 `crate::render::draw::draw_frame` 을 쓴다.

`ScreenMode`: `Fullscreen` / `Inline` / `Minimal` (`app/mod.rs:269-283`). 설정 `[ui] screen_mode = "fullscreen" | "minimal"`.

### 4.3 키 바인딩

단일 소스 `pager/src/actions/defaults.rs` — *"All key bindings are defined here — not scattered across event handlers."*

`actions/mod.rs:240-251` `lookup(event, When) → ActionId`. 사용자 config 로 키맵을 덮어쓰는 API 는 없다. overlay 로 키바인딩을 못 바꾸는 이유.

### 4.4 테마

`xai-grok-pager-render/src/theme/`: `GrokNight` / `GrokDay` / `TokyoNight` / `RosePineMoon` / `OscuraMidnight` / `Auto`. slash `/theme`.

로고·윈도 타이틀·"Grok" 카피는 overlay 로 못 지운다.

### 4.5 slash 명령

`pager/src/slash/commands/` **64 파일**. 빌트인 예:

| 군 | 명령 |
| --- | --- |
| 세션 | `/new` `/resume` `/fork` `/rename` `/export` `/share` `/rewind` `/timeline` |
| 모델 | `/model` `/effort` `/theme` `/vim` `/multiline` |
| 확장 | `/plugin` `/mcp` `/hooks` `/skills` |
| 에이전트 | `/plan` `/tasks` `/agents` `/personas` `/compact` |
| 기타 | `/btw` `/imagine` `/docs` `/feedback` `/dashboard` |

빌트인 slash 는 제거·개명 불가. 스킬 `user-invocable` 이 `/name` 을 추가한다. 충돌 시 빌트인이 bare name 유지.

---

## 5. LLM 통합

### 5.1 3 backend (xAI 전용 아님)

```1010:1021:crates/codegen/xai-grok-sampling-types/src/types.rs
pub enum ApiBackend {
    #[default]
    ChatCompletions,  // /v1/chat/completions
    Responses,        // /v1/responses
    Messages,         // Anthropic /v1/messages
}
```

`supports_native_schema`: ChatCompletions / Responses 만. Messages 는 schema 가 tool use 를 막아서 StructuredOutput 툴로 우회.

sampler `run_one_attempt` (`sampler/src/actor/request_task.rs:419-461`) 가 backend 별 raw stream → L2 transform (`stream/chat_completions.rs`, `responses.rs`, `messages.rs`).

### 5.2 기본 카탈로그

임베드: `crates/codegen/xai-grok-models/default_models.json`.

```1:16:crates/codegen/xai-grok-models/default_models.json
{
  "default": "grok-4.5",
  "web_search": "grok-4.20-multi-agent",
  "models": [
    {
      "id": "grok-4.5",
      "context_window": 500000,
      "api_backend": "responses",
      "supports_reasoning_effort": true,
```

런타임 `ModelsManager` (`shell/src/agent/models.rs`): 임베드 JSON + `/v1/models` fetch + etag 캐시. 우선순위 CLI > ENV > `config.toml` > remote settings > baked JSON.

### 5.3 커스텀 모델

`[model.<name>]` → `ConfigModelOverride` (`shell/src/agent/config.rs:3570+`). 필드: `model`, `base_url`, `api_key`, `env_key`, `api_backend`, `extra_headers`, `context_window`, `temperature`, `top_p`, `max_completion_tokens`, `agent_type`, `max_retries`, `reasoning_effort`, `hidden` 등.

MiniMax / Ollama / OpenAI 호환은 `base_url` + `api_backend = "chat_completions"` 로 붙는다. 로그인 UI·구독·이미지/비디오는 여전히 xAI.

### 5.4 retry / token

- `GROK_MAX_RETRIES` > 모델 `max_retries` > **15** (`sampler/src/retry.rs`)
- `TokenUsage { prompt, completion, total, reasoning, cached_prompt }` (`sampling-types/src/conversation.rs:648-668`)
- 스트림 완료 시 tracing span 에 `output_tokens` / `reasoning_tokens` 기록

---

## 6. 도구/스킬 시스템

### 6.1 레지스트리

`ToolRegistryBuilder::new` (`tools/src/registry/types.rs:657-746`). 키 `{Namespace}:{id}` 예: `GrokBuild:read_file`.

| 구간 | 등록 |
| --- | --- |
| GrokBuild | Bash, ReadFile, SearchReplace, ListDir, Grep, KillTask, KillTerminalCommand, TodoWrite, UpdateGoal, TaskOutput, GetTerminalCommandOutput, WaitTasks, **Task**, WebSearch, WebFetch, Lsp, ImageGen/Edit, ImageToVideo, ReferenceToVideo, Enter/ExitPlanMode, AskUserQuestion, Monitor, SchedulerCreate/Delete/List |
| Codex | ApplyPatch, CodexListDir, CodexGrepFiles, CodexReadFile |
| OpenCode | bash/read/edit/write/grep/glob/todowrite/skill |
| meta | MemorySearch, MemoryGet, SearchTool, UseTool (MCP 지연) |
| Concise | Read/Replace/Bash 짧은 스키마 |
| Hashline | hashline_read / hashline_edit / hashline_grep |

`ToolKind::DeployApp` 은 enum 에만 있고 `new()` 미등록.

```33:50:crates/codegen/xai-grok-tools/src/types/tool.rs
pub enum ToolNamespace {
    GrokBuild, GrokBuildConcise, GrokBuildHashline, Codex, OpenCode, MCP,
}
```

### 6.2 네임스페이스 전환

두 축.

1. **에이전트 프리셋** (`GROK_AGENT` / `--agent` / 모델 `agent_type` / ACP profile / `[agent]`):
   - `grok-build` — 기본
   - `grok-build-concise`
   - `codex` — **strict harness** (커스텀 프롬프트, default tools 미주입)
   - `opencode` — non-strict
   - `explore` / `plan` — 읽기 전용 서브셋
2. **file_toolset** (`[toolset] file_toolset = "hashline"`): `read_file/search_replace/grep` 슬롯만 Hashline 으로 교체. D-104/D-105 가 따라가던 원본.

### 6.3 Permission

에이전트 모드 (`agent/src/config.rs:953-975`):

`Default | AcceptEdits | Auto | DontAsk | BypassPermissions | Plan`

파이프라인 (`workspace/src/permission/manager.rs:1294-1648`), **앞에서 이김**:

1. compiled policy
2. **Policy Deny** (YOLO 보다 앞)
3. YOLO / AlwaysApprove
4. session / persisted grant
5. auto + policy Allow
6. auto classifier (연속 3 / 총 20 한도 후 유저)
7. sandbox bash auto-allow
8. access-kind default (Read=safe, MCP=grant 또는 프롬프트)
9. 유저 프롬프트

### 6.4 Skills

Scope 숫자 (낮을수록 승): Local=0 → Repo=1 → User=2 → Server=3 → Bundled=4 → Plugin=5.

수집: cwd→git root 의 `.grok/.agents/.claude/.cursor/skills` → `~/.grok` → `[skills].paths` → bundled → plugin. first-seen-wins. **gitignore 무시**. 숨김은 `[skills].ignore` / `disabled`.

`SKILL.md` frontmatter: `name`, `description`(≤1024), `when-to-use`, `paths`, `allowed-tools`, `model`, `effort`, `user-invocable`(기본 true), `disable-model-invocation`, `license`, `compatibility`, `argument-hint`, `metadata.*`. 깊이 ≤ 5. `commands/*.md` 는 Claude 식 slash.

---

## 7. 컨텍스트 관리

### 7.1 프로젝트 규칙

파일명 (`compat.rs:401-414`):

`Agents.md`, `Claude.md`, `CLAUDE.md`, `CLAUDE.local.md`, `AGENT.md`, `AGENTS.md` + (compat) `.claude/CLAUDE.md`.

**`GROK.md` 는 인식하지 않는다.**

추가로 `<dir>/.grok/rules/*.md`, compat `.claude/rules`, `.cursor/rules`, `$GROK_HOME/rules/`. AGENTS.md 는 **gitignore 준수**. 깊은 파일이 우선. `AgentsMdTracker` 가 도구 경로에서 상향 walk (최대 10 레벨).

### 7.2 Compaction

- 세션 inter-compact: `auto_compact_threshold_percent` **기본 85** (`agent/src/compaction.rs`)
- intra-compact: 같은 85%, `min_steps=3`, `min_compactable_tokens=5000` (`xai-grok-compaction`)
- `--compaction-mode summary|transcript|segments`
- two-pass prefire 옵션 (`[features] two_pass_compaction`)

### 7.3 Memory (experimental)

게이트: `--experimental-memory` / `GROK_MEMORY=1` / `[memory] enabled`. 기본 **off**.

```
~/.grok/memory/
  MEMORY.md
  {slug}-{blake3_8}/
    MEMORY.md
    index.sqlite     # chunks + FTS5 + optional vec
    sessions/YYYY-MM-DD-*.md
```

도구: `MemorySearch`, `MemoryGet`.

### 7.4 codebase-graph

tree-sitter 인덱스. 컨텍스트 창에 통째로 넣지 않고 ACP `x.ai/code/goto-definition` 등 온디맨드 내비. `[features] codebase_indexing` 기본 on.

---

## 8. 세션 영속화

### 8.1 레이아웃

```
~/.grok/sessions/
  session_search.sqlite
  {encode_cwd_dirname(cwd)}/{session_id}/
    summary.json              # resume 필수
    updates.jsonl             # 권위 있는 ACP 스트림
    chat_history.jsonl        # 모델에 보낸 raw (파생)
    rewind_points.jsonl
    plan.json / plan_mode.json
    signals.json / feedback.jsonl / btw_history.jsonl
    goal/state.json
    images/  prompts/  subagents/{id}/
```

상수 (`shell/src/session/storage/mod.rs:26-33`):

```26:33:crates/codegen/xai-grok-shell/src/session/storage/mod.rs
pub(crate) const SUMMARY_FILE: &str = "summary.json";
pub(crate) const CHAT_HISTORY_FILE: &str = "chat_history.jsonl";
pub(crate) const UPDATES_FILE: &str = "updates.jsonl";
```

`CHAT_FORMAT_VERSION = 1` (`persistence.rs:26-29`): v0 = legacy ChatRequestMessage, v1 = ConversationItem. `updates.jsonl` 이 없으면 resume 시 `chat_history` 에서 재구성. torn append 는 해당 줄만 skip, corrupt 는 `.corrupt` 로 보존.

### 8.2 Resume

- `-r [id]` 생략 시 최근
- `-c` 현재 cwd 최신
- `--fork-session` 새 ID
- `--restore-code` 원 커밋 checkout
- resume 시 `--sandbox` 가 저장 프로필과 다르면 **거절** (세션 생성 시 고정)

SQLite 는 대화 본문이 아니다. `session_search.sqlite` = FTS 검색, memory `index.sqlite` = 청크 인덱스.

---

## 9. 확장 시스템

CONCEPT 4-계층이 **이미 first-class**.

### 9.1 Plugins

`plugin.json` camelCase (`agent/src/plugins/manifest.rs:132-170`): `name`(kebab 1–64), `version`, `skills`/`commands`/`agents`/`hooks`/`mcpServers`/`lspServers` (path 또는 inline). 없으면 convention (`skills/`, `.mcp.json`, `hooks/hooks.json`) 으로도 동작. fallback `.grok-plugin/plugin.json` → `.claude-plugin/plugin.json`.

discovery 우선: `--plugin-dir`(자동 trust) → 프로젝트 `.grok/plugins` → `~/.grok/plugins` + `installed-plugins` → `[plugins].paths`.

marketplace 인덱스: `.grok-plugin/marketplace.json` (또는 `.claude-plugin/`). 설치 `grok plugin install <src> --trust`. 기본 **off** — enable 필요.

trust 이중화: 프로젝트 = folder-trust (`trusted_folders.toml`). ConfigPath = `~/.grok/trusted-plugins`. 비신뢰 시 skills/agents 는 목록만, hooks/MCP 차단.

### 9.2 Hooks (15 이벤트)

```12:35:crates/codegen/xai-grok-hooks/src/event.rs
pub enum HookEventName {
    SessionStart, SessionEnd, Stop, StopFailure,
    PreToolUse, PostToolUse, PostToolUseFailure, PermissionDenied,
    UserPromptSubmit, Notification,
    SubagentStart, SubagentStop, SubagentEnd,
    PreCompact, PostCompact,
}
```

- PreToolUse: stdout `{"decision":"deny","reason":"…"}` 또는 exit 2 → **툴 차단**
- Stop / SubagentStop: `{"decision":"block"}` → 턴 종료 막고 모델에 reason. 턴당 8회
- 나머지: observe. 실패는 **fail-open**

로딩: `~/.grok/hooks/*.json` + 프로젝트 `.grok/hooks` (folder-trust) + Claude/Cursor settings + 플러그인 + config TOML.

crate rustdoc 은 아직 "4 events" 를 주장한다. **문서↔구현 불일치.**

### 9.3 MCP

crate `xai-grok-mcp`: rmcp 2.1 + reqwest 0.13 **격리** (워크스페이스 나머지는 0.12).

transport: Stdio / StreamableHttp. 설정 `[mcp_servers.<name>]`. OAuth 토큰은 `$GROK_HOME/mcp_credentials.json` (auth.json 과 분리, unix 0600). `[mcp] max_output_bytes` 기본 20_000.

### 9.4 Subagents

도구 id **`task`** (문서 별칭 `spawn_subagent`). 형제 `get_task_output` / `wait_tasks` / `kill_command_or_subagent`.

빌트인 3: `general-purpose` / `explore` / `plan`. 프로젝트 `.grok/agents/*.md` 만 빌트인 이름 섀도잉. 유저 `~/.grok/agents` 는 추가만.

```29:31:crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
/// Maximum nesting depth for subagents. A top-level session is depth 0;
/// the first subagent is depth 1. Subagents cannot spawn further subagents.
pub const MAX_SUBAGENT_DEPTH: u32 = 1;
```

`isolation = none | worktree`. `cwd` 와 worktree 상호배타. personas/roles 는 `<system-reminder>` overlay.

---

## 10. 빌드 & 배포

| 항목 | 내용 |
| --- | --- |
| 공식 설치 | `curl -fsSL https://x.ai/cli/install.sh \| bash` → `~/.grok/bin/grok` |
| 업데이트 | `grok update` (`xai-grok-update`). 채널 stable / alpha / enterprise. GCS fallback |
| 소스 빌드 | `cargo run -p xai-grok-pager-bin`. DotSlash + `bin/protoc` 필수 |
| 루트 Cargo.toml | **generated, read-only** |
| rust-toolchain | 1.92.0, linux gnu 2타깃만 명시 |
| hardening | `obfstr` + `cryptify`. 시스템 프롬프트 암호화 템플릿 |
| 공개 CI | `.github/workflows` **없음** (모노레포 CI 가 본체) |
| Windows | best-effort, 이 트리에서 미테스트 |

포크 시 updater 채널·바이너리 이름·generated workspace 를 전부 직접 소유해야 한다.

---

## 11. 테스트 & 품질

| crate | 테스트 |
| --- | --- |
| `xai-grok-pager` | `tests/` 235파일. `pty_e2e/` + yaml 시나리오 + `leader_pty_e2e/` |
| `xai-grok-shell` | ~35 통합 (`test_built_binary_e2e`, `test_mcp_integration`, `test_stop_hook_e2e`) |
| `xai-grok-sandbox` | `deny_paths_e2e.rs` |
| 기타 | hooks / mcp / sampler / telemetry / crash-handler |

권장 명령: `cargo test -p <crate>` (풀 워크스페이스는 느림). clippy.toml / rustfmt.toml 루트. 커버리지 수치는 공개 트리에 없음.

---

## 12. 보안

### 12.1 OS 샌드박스

기본 **off**. `--sandbox` / `GROK_SANDBOX`.

```60:68:crates/codegen/xai-grok-sandbox/src/profiles.rs
pub enum ProfileName {
    Workspace, Devbox, ReadOnly, Strict, Off, Custom(String),
}
```

| 프로필 | 읽기 | 쓰기 | child net (Linux) |
| --- | --- | --- | --- |
| workspace | 전체 | CWD + `~/.grok` + tmp | 허용 |
| devbox | 전체 | `/data` 제외 거의 전부 | 허용 |
| read-only | 전체 | `~/.grok` + tmp | 차단 |
| strict | CWD + 시스템 | CWD + `~/.grok` + tmp | 차단 |
| off | 무제한 | 무제한 | 무제한 |

구현: Linux Landlock + seccomp, macOS Seatbelt (`nono` CapabilitySet). child-net 은 macOS 에서 no-op. `~/.grok/sandbox.toml` + 프로젝트는 **이름 추가만** (글로벌 프로필 hollow-out 방지).

### 12.2 folder-trust

`~/.grok/trusted_folders.toml`. 최장 prefix. cwd-상대 `.grok/` 로 self-trust 불가 (`user_grok_home` 만). 로컬 비릴리스 빌드는 inert(항상 신뢰). 릴리스 기본 on. 프로젝트 hooks/MCP/LSP 공통 게이트.

### 12.3 시크릿

- `~/.grok/auth.json` unix 0600, **평문**. 키체인 미사용 (`secure_file.rs` 주석이 인정)
- MCP: `mcp_credentials.json` 분리
- Windows flock/chmod 대부분 no-op

### 12.4 Telemetry

```16:21:crates/codegen/xai-grok-telemetry/src/config.rs
pub enum TelemetryMode {
    #[default]
    Disabled,
    SessionMetrics,
    Enabled,
}
```

소스 기본 Disabled. Mixpanel 은 `mixpanel_enabled && token` 둘 다 필요. 공식 바이너리는 remote settings 로 켤 수 있음. `[features] telemetry = false` 로 고정 가능.

---

## 13. 주목할 패턴

### ✅ 우리가 차야 할 것 (Adopt)

1. **ACP 로 TUI / 래퍼 / IDE 분리** — `myharness` → `grok agent stdio` 가 정공법. in-process 기본, leader 는 공유 백엔드.
2. **plugin = skills+agents+hooks+MCP 한 단위** — CONCEPT 4-계층 재구현 불필요.
3. **3 API backend** — MiniMax 는 `chat_completions` + `base_url`.
4. **permission deny > YOLO** — 정책이 만능 승인보다 앞선다.
5. **PreToolUse + folder-trust** — 서버/환경 도메인 가드.
6. **JSONL 세션 + `updates.jsonl` 권위본** — v0 `state.json` 보다 성숙. torn-append self-heal.
7. **Hashline / Codex / OpenCode 도구 포트** — D-104 방향의 원본. `file_toolset` 슬롯 교체.
8. **서브에이전트 깊이 1 + worktree isolation**
9. **MCP 를 leaf crate 로 격리** — rmcp/reqwest 메이저 충돌을 composition root 가 안 떠안음.
10. **텔레메트리 fail-closed + 이중 게이트**
11. **sandbox 프로젝트 프로필 additive-only**
12. **composition-root 바이너리** — jemalloc/sandbox feature 를 bin 에만.

### ❌ 피해야 할 것 (Anti-patterns)

1. **소스 포크를 뼈대로 삼기** — 136만 줄 + generated workspace + 모노레포 dump + 기여 거부.
2. **암호화 프롬프트 패치** — 부분 수정 불가. override 또는 포기.
3. **브랜드/updater/키맵을 overlay 로 지우기** — 안 됨.
4. **hook fail-open 을 보안 경계로 믿기** — 타임아웃/크래시는 툴을 통과시킴.
5. **평문 auth.json 을 키체인으로 착각**
6. **trust store 삼중화** — `trusted_folders.toml` + `trusted-plugins` + 레거시 hook trust.
7. **hooks rustdoc 을 정책 소스로 쓰기** — 4 vs 15 이벤트.
8. **CONCEPT §0 standalone sibling 을 유지한 채 grok 를 뼈대로 쓰기** — 모순.
9. **공개 트리 CI 부재를 "테스트 없다"로 오해** — pty e2e 는 방대, CI 만 모노레포 쪽.

---

## 14. 미해결 질문

1. 모노레포 sync 주기와 공개 트리 lag (커밋 5개 vs 설치 1.0.3).
2. 암호화 프롬프트 템플릿의 라이선스/재배포 범위.
3. 공식 바이너리가 remote settings 로 telemetry 를 켜는 조건.
4. MiniMax / Ollama 를 기본으로 쓸 때 `image_gen` / `web_search` 등 xAI-only 툴 실패 모드.
5. `GROK_HOME=~/.myharness` 분리 시 세션·인증 이전 비용.
6. Gitea / `harness-refs` 미러를 둘지 (용량 1.3M LOC).
7. `web_search` 키 `grok-4.20-multi-agent` 가 `models[]` 에 없는 이유 (원격 카탈로그 전제?).
8. `Theme` / `ActionId` 전량 표 — 이번 분석에서 필드 전개 안 함.

---

## 15. my_harness 영향 (코드로 닫힌 결론)

| CONCEPT 5 | grok-build | 1:1 |
| --- | --- | --- |
| Tools | `xai-grok-tools` + namespace + permission | 예 |
| Context | agent prompt + compaction + memory + AGENTS.md | 예 (프롬프트 암호화) |
| Session | JSONL `updates.jsonl` + FTS | 예 |
| Plugins | plugin.json + marketplace + hooks/MCP | 예 |
| Sub-agents | `task` + depth 1 + worktree | 예 |

** autonomously 재구현할 이유가 코드 상으로 사라졌다.** 남은 분기는 두 개뿐이다.

| 선택 | 코드가 말하는 것 |
| --- | --- |
| **Overlay** | `--plugin-dir` 자동 trust + `grok agent stdio` + `[model.*]` + PreToolUse. 3-도메인 명령은 래퍼. |
| **독립 런타임** | 포크 대상은 grok-build 가 아니라 **goose**. grok 는 패턴 이식 소스. |

이 문서는 결정을 내리지 않는다. 14섹션 실측만 고정한다.
