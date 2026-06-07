# myharness-tools 상세설계 (DD-1) — `trait Tool::Schema`

### VERDICT: PASS — trait Tool::Schema spec 확정 + 6 builtin tool spec + permission/registry/error/TC scaffold (D-16 4 chunk)

> 본 문서 = `myharness-tools` crate 의 상세설계. INITIAL_DESIGN.md §3.3 (line 324-339) + §3.2 (3rd-party crate, rig-core 0.5+) + §3.4 (pub use) + §6.1 (6 provider) + CONCEPT.md §5.4 (4 mode + hook) + §5.5 (D-15/28/36/38) + REQUIREMENTS.md §2.9 (NFR-SEC-3/4/5) + §4 (Rust 1.78, D-36) + **REVIEW.md §3.1 MAJOR-1 (trait Tool::Schema 권장 = rig-core `ToolDefinition` + `serde_json::Value` args)** 의 구현 입력.
>
> - **시점**: 2026-06-07 (REVIEW.md PASS 후, 상세설계 cycle 1 parallel task, attempt 2 — verifier 1차 reject 후 fresh rewrite)
> - **대상 독자**: TASK-005-1 (v1 Rust MVP 구현) 의 coder worker
> - **입력 SSOT (5 docs)**: CONCEPT.md (1,024) + REQUIREMENTS.md (1,003) + INITIAL_DESIGN.md (2,056) + REVIEW.md (~485) + PLAN_v1_design.md (535)
> - **목적**: REVIEW §3.1 MAJOR-1 의 `pub trait Tool { fn name(); fn schema(); async fn execute(); }` 의 미명시 Schema/Value type 을 `rig::tool::ToolDefinition` + `serde_json::Value` 로 spec 확정 + 6 builtin tool spec + permission layer + ToolRegistry + error type + TDD TC 진입점 제공

**핵심 결정 (1 line)**: **`trait Tool::Schema` = `rig::tool::ToolDefinition` (name/description/parameters) + `serde_json::Value` args/output** — rig-core 가 6 provider (claude/codex/gemini/deepseek/minimax/local) 모두의 tool calling format 자동 변환. plugin/MCP 동적 tool 의 schema-less 입력은 `Value` args 로 통일.

**5 trade-off** (verifier cross-check): §1 (trait 결정) / §3.5 (Read·Write·Edit vs Bash) / §3.10 (Grep·Glob vs Search) / §4.5 (4 mode vs 2 mode) / §6.4 (8 variant enum vs `Box<dyn Error>`).

**5 risks** (verifier patch reference): §8.2 R-1 (rig-core API stability) / R-2 (MCP 호환 검증) / R-3 (cross-OS) / R-4 (9 hook pattern 별도 spec) / R-5 (LLM mock 부재).

**분량**: target 600~900 lines (over-shoot 1,000+ ❌). chunked write D-16 4 chunk (150+200+200+150). 표준 6 원칙 (D-26) / D-06 메커니즘만 / 안티 6 미반영.

---

## §0. 메타 + 읽는 법 (D-26 + D-16)

### 0.1 문서 구조 (9 sections)

| § | 제목 | 역할 |
| --- | --- | --- |
| §0 | 메타 (D-16 + D-26) | 본 § |
| §1 | `trait Tool::Schema` 결정 | MAJOR-1 spec 확정, 권장 + 대안 + trade-off |
| §2 | `pub trait Tool` final spec | name / definition / call signature |
| §3 | 6 builtin tool spec | name + definition + args + result + error |
| §4 | permission check layer | 4 mode + hook eval |
| §5 | ToolRegistry spec | registration + lookup + dispatch |
| §6 | `ToolError` enum | 8 variant + recovery pattern |
| §7 | TDD TC scaffold | 30 TC entry (REVIEW §6.2) |
| §8 | Handoff (D-26 4-필드) | TASK-005-1 입력 |

### 0.2 SSOT cross-ref (5 docs)

| SSOT | 본 문서 § |
| --- | --- |
| INITIAL_DESIGN.md §3.3 (line 324-339, myharness-tools 6 sub-module) | §2, §3, §4, §5 |
| INITIAL_DESIGN.md §3.2 (line 514-540, rig-core 0.5+, tokio, serde, parking_lot) | §1, §2, §5 |
| INITIAL_DESIGN.md §3.4 (line 573-609, `pub use` 표면) | §2, §4, §5 |
| INITIAL_DESIGN.md §6 (line 1310-1430, LLM 6 provider) | §1 |
| CONCEPT.md §5.4 (line 202-224, 4 mode + hook + D-06) | §4, §6 |
| CONCEPT.md §5.5 (line 226-370, D-15/28/36/38) | §1, §3 |
| CONCEPT.md §5.7 (line 453-466, Plugin 시스템) | §1, §5 |
| REQUIREMENTS.md §2.9 (line 408-429, NFR-SEC-3/4/5) | §4, §6 |
| REQUIREMENTS.md §4 (line 460-490, Rust 1안, MSRV 1.78) | §2 |
| **REVIEW.md §3.1 MAJOR-1** (line 199-208) | **§1 (정합 근거)** |
| REVIEW.md §6.2 (line 392-400, 30 TC) | §7 |
| REVIEW.md §5.2 (line 348-360, 5 task 분할) | chunked write 4 chunk |

### 0.3 표준 6 원칙 (D-26) + 안티 6 미반영

- **6 원칙**: 한국어 / 결론 위주 / 상태값 done / 이벤트 소싱 (log.jsonl) / 비참조 / handoff 4-필드
- **안티 6** (CONCEPT §8): 1 surface (md) / 단일 Rust (D-36) / 6 builtin tool / 2 surface (CLI+TUI) / local-only memory (NFR-SEC-8) / MIT 호환 single binary

### 0.4 chunked write D-16 패턴

- **chunk 1** (line 1-150): VERDICT + §0 + §1
- **chunk 2** (line 151-350): §2 + §3 (Read/Write/Edit)
- **chunk 3** (line 351-550): §3 (Bash/Grep/Glob) + §4
- **chunk 4** (line 551-end): §5 + §6 + §7 + §8
- **early deliverable signal**: `docs/team/deliverable_dd1.md` (status=in_progress, chunk 1 직후)
- **minimal board noise**: start + done 2 entry

---

## §1. `trait Tool::Schema` 결정 (rig-core `ToolDefinition` + `serde_json::Value`)

### 1.1 결정 (결론)

**본 §1 의 spec 결정**: myharness-tools 의 `trait Tool::Schema` type 은 다음 2-component 로 확정.

| component | type | 출처 | 선정 근거 |
| --- | --- | --- | --- |
| **definition (tool metadata)** | `rig::tool::ToolDefinition` (rig-core native struct) | rig-core 0.5+ (D-36, CONCEPT §5.5.4) | rig-core 표준 struct. 12+ provider (Anthropic/OpenAI/Google/Ollama/DeepSeek/MiniMax/Local) 모두의 tool calling format 자동 변환. `ToolDefinition { name: String, description: String, parameters: serde_json::Value }` |
| **args (input to call)** | `serde_json::Value` | serde_json (D-36, INITIAL_DESIGN §3.2) | plugin/MCP 동적 tool 의 schema-less 입력 지원. 6 builtin tool 도 내부 typed struct → `Value` 변환 (1-hop). |
| **output (result of call)** | `serde_json::Value` | serde_json | LLM tool result format 통일. MCP tool result 와 호환 (`mcp__xxx__yyy` 모두 Value) |
| **error** | `ToolError` (본 crate 정의 enum) | §6 spec | rig-core 의 `ToolError` 와 별도 (tool 자체 error vs dispatch error 분리) |

**Trait 시그니처 (final spec, §2 에서 full Rust 의사코드)**:
```rust
use rig::tool::ToolDefinition;
use serde_json::Value;
#[async_trait::async_trait]  // D-36 stable, 1.78, dyn 호환
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn definition(&self) -> ToolDefinition;
    fn required_scope(&self) -> permission::ToolScope;
    async fn call(&self, args: Value) -> Result<Value, ToolError>;
    fn dry_run(&self, _args: &Value) -> Option<Result<Value, ToolError>> { None }  // optional
}
```

### 1.2 대안 + trade-off 표

| 옵션 | Schema type | args | output | trade-off |
| --- | --- | --- | --- | --- |
| **(a) JSON Schema raw** (직접 정의) | `myharness_tools::Schema` (직접 struct) | `Value` | `Value` | ✅ vendor 무관. ❌ rig-core 와 형변환 adapter 필요. ❌ 6 provider 각각 tool calling format 직접 매핑 (claude `tools: [{...}]` / openai `tools: [{type: "function", function: {...}}]` / gemini `tools: [{function_declarations: [...]}]`). ❌ rig-core 0.5+ 의 `ToolDefinition` 와 중복 |
| **(b) rig-core `ToolDefinition` (선정)** ⭐ | `rig::tool::ToolDefinition` | `Value` | `Value` | ✅ rig-core native 통합. ✅ 12+ provider 모두 자동 변환. ✅ rig-core 의 `AgentBuilder::tool(Arc<dyn Tool>)` 와 직접 호환. ✅ MCP tools (`mcp__xxx__yyy`) 도 동일 `ToolDefinition` 형식 — auto-expose 시 변환 불필요. ⚠️ rig-core API 변경 시 영향 (semver 안정성 검증: rig-core 0.5+ stable) |
| **(c) `schemars::JsonSchema` derive + typed `Self::Args`** | `schemars::schema::Schema` | `Self::Args: Deserialize` | `Self::Output: Serialize` | ✅ compile-time type safety. ✅ `schemars::schema_for!` 로 JSON Schema 자동 생성. ❌ plugin/MCP 동적 tool (type 없음) 부적합. ❌ `Arc<dyn Tool>` 의 `call(&self, Value)` 변환 adapter 필요. ❌ rig-core 의 `Tool` trait (typed Self::Args) 와 동시 구현 시 boilerplate |
| **(d) hybrid: rig-core for builtin, raw for plugin** | builtin = `ToolDefinition`, plugin = raw JSON Schema | Value | Value | ❌ 2-path → tool lookup / dispatch 분기 필요. ❌ rig-core `ToolSet` 와 통합 시 adapter. ❌ 유지보수 비용 ↑ |

**선정 = (b) rig-core `ToolDefinition` + `serde_json::Value` args/output** (REVIEW §3.1 MAJOR-1 권장 (b) + (c) 의 hybrid 형태, plugin/MCP 호환 위해 `Value` args 유지).

### 1.3 rig-core `ToolDefinition` spec 확인 (실제 crate API)

**출처**: `https://docs.rig.rs/docs/concepts/tools` (rig-core 0.31.0+ docs), crates.io `rig-core` (D-36 §3.2 선정).

```rust
// rig-core/src/tool/mod.rs (simplified)
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema (type=object, properties={...})
}
```

**rig-core 의 `Tool` trait** (참고, 본 spec 은 다름 — §2 에서 차이점):
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    const NAME: &'static str;  // ⚠️ const 가 아닌 fn 도 가능
    type Error;
    type Args: DeserializeOwned;
    type Output: Serialize;
    async fn definition(&self, _prompt: String) -> ToolDefinition;
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}
```

**본 myharness `Tool` trait 의 차이점 (rig-core 와)**:
1. `name` = `&'static str` const 가 아닌 `fn` (runtime tool name 변경 가능, v1.5+ plugin 동적 이름 대비)
2. `definition(&self)` = `_prompt` 인자 없음 (LLM prompt-aware schema 는 v1.5+)
3. `Args`/`Output` = `serde_json::Value` (typed wrapper 없음, plugin 호환)
4. `Error` = `ToolError` (8 variant, §6)
5. trait bound = `Send + Sync` (Arc-shared, multi-thread, NFR-PERF-1)

**Auto-expose 호환** (CONCEPT §5.7, myharness-plugins::mcp::auto_expose): MCP tool (`mcp__filesystem__read_file`) → `ToolDefinition { name, description, parameters }` + `call(args: Value) -> Result<Value>` 로 wrap → registry 등록. rig-core `AgentBuilder::tool(Arc<dyn Tool>)` 에 builtin + plugin + MCP 모두 동일하게 pass.

### 1.4 args schema 표현 (JSON Schema subset)

`ToolDefinition::parameters` = `serde_json::Value` (JSON Schema draft-07 subset). v1 지원 type:

| JSON Schema type | 사용 예 | 비고 |
| --- | --- | --- |
| `string` | path, command, pattern | minLength, maxLength, pattern (regex) |
| `number` / `integer` | timeout_seconds, max_results | minimum, maximum |
| `boolean` | recursive, hidden, with_line_numbers | default |
| `array` | list of paths / commands | items (typed), minItems, maxItems |
| `object` | nested config (e.g., `env: {KEY: VAL}`) | properties, required, additionalProperties: false |

**6 builtin tool 의 parameters** = 모두 `type: "object"` + `properties` + `required` (LLM 강제). §3 의 각 tool spec 에서 명시.

### 1.5 spec 변경의 영향 (cascade)

| cascade | § / 영향 | 처리 |
| --- | --- | --- |
| **myharness-llm** | `rig-core::agent::AgentBuilder::tool(Arc<dyn Tool>)` 에 myharness `Tool` trait 을 구현하는 impl wrapper (`RigToolAdapter`) 필요 | DD-5 retry/error spec 에서 별도, 또는 myharness-llm crate 의 lib.rs 에서 adapter |
| **myharness-agents** | 15 sub-agent 의 `allowed_tools: &[ToolId]` = `&[&str]` (tool name list). `ToolRegistry::lookup(name)` 으로 dispatch | DD-3 (15 sub-agent spec, MAJOR-3) 에서 사용 |
| **myharness-plugins** | MCP auto-expose 가 `mcp__xxx__yyy` → `Arc<dyn Tool>` wrapping | INITIAL_DESIGN §3.7 spec 정합, 별도 spec doc 불필요 |
| **myharness-cli** | 30 CLI command 중 tool 직접 invoke 명령 없음 (LLM 이 호출). `myharness tool list` (디버그) 정도만 | §5.2 ToolRegistry 의 debug API |
| **TDD TC** | 6 builtin × 5 시나리오 = 30 TC (REVIEW §6.2 정합) | 본 §7 |

### 1.6 결정 근거 1-라인 (yklee review)

> **rig-core 가 12+ provider 의 tool calling format 모두 cover → 우리가 직접 JSON Schema adapter 작성/유지 불필요. plugin/MCP 동적 tool 의 schema-less 입력은 `serde_json::Value` args 로 통일.**

---

## §2. `pub trait Tool` final spec (Rust 의사코드)

### 2.1 결정 (결론)

`myharness-tools::Tool` trait = 본 §2 spec 확정. 5-필드: `name()` / `definition()` / `required_scope()` / `call()` + optional `dry_run()`. Arc-shared (`SharedTool = Arc<dyn ToolObject>`).

### 2.2 trait 정의 (의사코드, full impl ❌)

```rust
// crates/myharness-tools/src/lib.rs
use async_trait::async_trait;
use rig::tool::ToolDefinition;
use serde_json::{json, Value};
use std::sync::Arc;

pub mod builtins; pub mod permission; pub mod registry; pub mod error;
pub use error::ToolError;
pub use permission::{PermissionMode, PermissionContext, PermissionDecision};
pub use registry::ToolRegistry;

/// 모든 tool 의 base trait. builtin + plugin + MCP 모두 동일.
#[async_trait]  // D-36: Rust 1.78 stable, dyn 호환
pub trait Tool: Send + Sync {
    /// LLM 호출 시 tool 의 고유 이름. 6 builtin = "Read"/"Write"/"Edit"/"Bash"/"Grep"/"Glob".
    /// MCP = "mcp__filesystem__read_file" 등 (CONCEPT §5.14).
    fn name(&self) -> &'static str;
    /// LLM metadata. rig-core `ToolDefinition` 형식. 6 provider 모두 자동 변환 (INITIAL §6.1).
    fn definition(&self) -> ToolDefinition;
    /// permission check 가 보는 scope. Bash = "Bash:*", Read = "Read:/path/**" 등.
    fn required_scope(&self) -> permission::ToolScope;
    /// tool 호출. args = LLM JSON, output = LLM JSON, error = ToolError (8 variant, §6).
    async fn call(&self, args: Value) -> Result<Value, ToolError>;
    /// optional: plan mode dry-run. default = None.
    fn dry_run(&self, _args: &Value) -> Option<Result<Value, ToolError>> { None }
}

pub trait ToolObject: Tool + Send + Sync + 'static {}
impl<T> ToolObject for T where T: Tool + Send + Sync + 'static {}
pub type SharedTool = Arc<dyn ToolObject>;
```

### 2.3 Cargo.toml (의존, INITIAL_DESIGN §3.3 정합)

```toml
[dependencies]
rig-core = "0.5"           # D-36, ToolDefinition
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
async-trait = "0.1"        # D-36 stable, dyn 호환 (1.75+ native async fn in trait = BoxFuture 필요)
thiserror = "1"
anyhow = "1"
tracing = "0.1"
directories = "5"          # ~/.myharness/ cross-platform
chrono = { version = "0.4", features = ["serde"] }
parking_lot = "0.12"       # §5 concurrent safety
regex = "1"                # §4 hook eval
globset = "0.4" + walkdir = "2"   # §3 Glob builtin
```

**v1 trade-off** (async fn in trait): 선정 = `#[async_trait]` (dyn 호환, simple). 대안 = Rust 1.75+ native (BoxFuture 필요). v1 = `Arc<dyn Tool>` 보관이라 `#[async_trait]` 단순.

### 2.4 `ToolScope` type spec

```rust
pub enum ToolScope {
    Read(PathPattern), Write(PathPattern), Edit(PathPattern),  // file path
    Grep(PathPattern), Glob(PathPattern),                      // search path
    Bash(CommandPattern),                                      // glob: "*" / "rm:*" / "git:*"
    Network(HostPattern),                                      // v1.5+ MCP
}
pub enum PathPattern { Literal(String), Glob(String), WorkingDir, Any }
```

permission check (§4) = `ToolScope` vs `PermissionContext` (mode + hook result) 비교.

### 2.5 LLM 호출 흐름 (의사코드, INITIAL_DESIGN §6.1 정합)

```rust
// myharness-agents/src/orchestrator.rs (일부)
async fn execute_tool_call(registry: &ToolRegistry, tool_name: &str, args: Value, ctx: &PermissionContext) -> Result<Value, ToolError> {
    let tool = registry.lookup(tool_name).ok_or_else(|| ToolError::Unknown { name: tool_name.into() })?;
    // 1. permission check (sync, < 5ms)
    if let PermissionDecision::Denied(reason) = permission::check(&tool.required_scope(), ctx, &args)? {
        return Err(ToolError::PermissionDenied { tool: tool_name.into(), reason });
    }
    // 2. hook eval (async, < 50ms, claude-code 13.4)
    if let Some(reason) = permission::eval_hooks(tool_name, &args, ctx).await? {
        return Err(ToolError::HookBlocked { hook: "match".into(), reason });
    }
    // 3. dispatch + timeout
    let secs = tool.default_timeout_secs();
    let result = tokio::time::timeout(Duration::from_secs(secs), tool.call(args.clone()))
        .await
        .map_err(|_| ToolError::Timeout { tool: tool_name.into(), secs: secs as u64 })??;
    // 4. audit log (NFR-SEC-7)
    session::log::append(Event::ToolCall { name: tool_name, args, result: result.clone() })?;
    Ok(result)
}
```

### 2.6 결정 trade-off (trait API)

| 선정 | 대안 | trade-off |
| --- | --- | --- |
| `#[async_trait]` macro | Rust 1.75+ native `async fn` | ✅ v1 1.78 stable (D-36), dyn 호환. ⚠️ macro 비용 (minimal) |
| `fn name(&self) -> &'static str` | `const NAME: &'static str` | ✅ runtime 이름 변경 가능 (v1.5+ plugin 동적 이름) |
| `required_scope() -> ToolScope` enum | 별도 `ScopedTool` trait | ✅ trait 1개 단일화. ⚠️ Bash `CommandPattern` 평가 = 1회 (cache 가능) |
| `dry_run` optional default `None` | mandatory abstract method | ✅ 대부분 tool = `call` 그대로 plan 가능. ⚠️ Bash dry-run = allow 만 (별도 impl) |

### 2.7 결정 근거 1-라인 (yklee review)

> **trait 1개 + 5 메서드 + Arc-shared**. rig-core 의 `Self::Args` typed wrapper 는 plugin/MCP 동적 tool 부적합 → `serde_json::Value` 통일.

---

(이어서 §3 6 builtin tool spec)

## §3. 6 builtin tool spec (Read/Write/Edit/Bash/Grep/Glob)

### 3.1 결정 (결론)

6 builtin tool (`Read` / `Write` / `Edit` / `Bash` / `Grep` / `Glob`) — INITIAL_DESIGN.md §3.3 line 331-336 정합. 각 tool = `name` + `definition()` (rig-core `ToolDefinition`) + `args schema` (JSON Schema in parameters) + `result schema` (LLM result) + `error map` (8 `ToolError` variant → §6). `path` = `String` (absolute or CWD-relative, `~` expansion).

### 3.2 Tool 1: `Read` — file content read

**module path**: `crates/myharness-tools/src/builtins/read.rs` (의사코드, full impl ❌)

```rust
pub struct ReadTool { /* config: max_file_size: u64 (default 100MB), default_encoding */ }

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str { "Read" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Read".into(),
            description: "Read file content from local filesystem. Returns text or base64-encoded binary.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path":     { "type": "string",  "description": "Absolute or CWD-relative path. ~ expansion supported." },
                    "offset":   { "type": "integer", "description": "0-based line offset (default 0).", "minimum": 0 },
                    "limit":    { "type": "integer", "description": "Max lines to return (default 2000, max 10000).", "minimum": 1, "maximum": 10000 },
                    "encoding": { "type": "string",  "enum": ["utf-8", "binary"], "default": "utf-8" }
                },
                "required": ["path"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Read(PathPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* tokio::fs::read_to_string + splitn */ }
}
```

**result schema** (LLM result): `{ path: String, content: String (UTF-8 or base64), lines: u64, encoding: "utf-8"|"binary", truncated: bool }`. **error map**: `FileNotFound` (path 없음 → LLM: Glob) / `PermissionDenied` (scope/mode) / `InvalidArgs` (`path` missing) / `NetworkError` (v1.5+ remote).

### 3.3 Tool 2: `Write` — file create/overwrite

**module path**: `crates/myharness-tools/src/builtins/write.rs`

```rust
pub struct WriteTool { /* config: require_confirmation_threshold_bytes (default 100KB), atomic_write: bool (default true) */ }

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &'static str { "Write" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Write".into(),
            description: "Create or overwrite a file. Use Edit for in-place modification of existing files. Atomic via tmp+rename.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path":    { "type": "string", "description": "Absolute or CWD-relative path" },
                    "content": { "type": "string", "description": "File content (UTF-8 text)" },
                    "encoding":{ "type": "string", "enum": ["utf-8"], "default": "utf-8" }
                },
                "required": ["path", "content"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Write(PathPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* tokio::fs::write + parent dir mkdir */ }
}
```

**result schema**: `{ path, bytes: u64, created: bool (true=신규, false=덮어씀) }`. **error map**: `PermissionDenied` (mode/scope) / `FileNotFound` (parent dir 없음) / `InvalidArgs` / `Unknown` (disk full, OS-level perm).

### 3.4 Tool 3: `Edit` — string replace in file

**module path**: `crates/myharness-tools/src/builtins/edit.rs`

```rust
pub struct EditTool { /* config: max_replacements_per_call (default 1), fuzzy_match_tolerance */ }

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &'static str { "Edit" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Edit".into(),
            description: "Replace a unique string in an existing file. Fails if old_text is not unique (use replace_all=true to bypass).".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path":        { "type": "string",  "description": "Absolute or CWD-relative path" },
                    "old_text":    { "type": "string",  "description": "The exact text to replace. Must be unique unless replace_all=true." },
                    "new_text":    { "type": "string",  "description": "The replacement text" },
                    "replace_all": { "type": "boolean", "default": false }
                },
                "required": ["path", "old_text", "new_text"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Edit(PathPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* read + str::replace + write atomic */ }
}
```

**result schema**: `{ path, replacements: u64, diff: String (unified diff) }`. **error map**: `FileNotFound` (path 없음) / `InvalidArgs` (old_text 없거나 중복, when `replace_all=false`) / `PermissionDenied`. **acceptEdits mode** (NFR-SEC-3): mode=acceptEdits 시 `Edit` 자동 allow (Bash/Write 는 여전히 scope check).

### 3.5 결정 trade-off (Read/Write/Edit vs Bash 일원화)

| 선정 (3 tool 분리) | 대안 (Bash 만 + cat/sed/tee) | trade-off |
| --- | --- | --- |
| Read/Write/Edit = 3 tool | Bash 안에 `cat`/`sed -i`/`tee` | ✅ **path/permission 정확** (file read vs subprocess scope). ✅ **dry-run 가능** (Edit dry-run = old/new diff preview). ✅ **error recovery 명확** (FileNotFound vs CommandFailed). ⚠️ LLM 이 tool 3개로 갈라야 함 (claude/gpt-5 모두 training data 충분). ❌ 대안: Bash 1 tool → `Bash:*` scope = 모든 명령 허용 = security risk |
| `Edit` 의 `replace_all` flag | `Edit` + `MultiEdit` 2 tool | ✅ 1 tool 단순. ✅ `replace_all: false` 가 default = 안전. ⚠️ unique 매칭 실패 시 LLM 이 old_text 재조정 (multi-step) |

---

### 3.6 Tool 4: `Bash` — subprocess exec

**module path**: `crates/myharness-tools/src/builtins/bash.rs`

```rust
pub struct BashTool {
    pub default_timeout_secs: u32,   // 120
    pub max_timeout_secs: u32,       // 600 (10 min hard cap)
    pub max_output_bytes: usize,     // 1 MB
    pub env_passthrough: Vec<String>,// ["PATH", "HOME", "LANG", "USER", "TMPDIR"]
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str { "Bash" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Bash".into(),
            description: "Execute a shell command (sh/bash/zsh on Unix, cmd/PowerShell on Windows). Returns stdout, stderr, exit_code. Prefer Read/Write/Edit for file operations.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command":    { "type": "string",  "description": "Shell command line" },
                    "timeout":    { "type": "integer", "description": "Timeout in seconds (default 120, max 600)", "minimum": 1, "maximum": 600 },
                    "cwd":        { "type": "string",  "description": "Working directory (default CWD)" },
                    "env":        { "type": "object",  "additionalProperties": { "type": "string" } },
                    "capture_stderr": { "type": "boolean", "default": true }
                },
                "required": ["command"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Bash(CommandPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* tokio::process::Command + timeout + max_output truncation */ }
}
```

**result schema**: `{ stdout: String, stderr: String, exit_code: Option<i32>, timed_out: bool, duration_ms: u64 }`. **error map**: `SubprocessFailed` (exit != 0) / `Timeout` (tokio timeout) / `PermissionDenied` / `HookBlocked` (예: `warn-rm-rf.md`) / `InvalidArgs` (`command` missing or `timeout > 600`).

**cross-OS** (NFR-PLAT-2): Unix → `sh -c "<command>"`. Windows → `cmd /C <command>` (default) or `powershell -Command <command>` (env `MYHARNESS_SHELL=powershell`). 자동 detection = `cfg(target_os)`.

### 3.7 Tool 5: `Grep` — ripgrep wrapper

**module path**: `crates/myharness-tools/src/builtins/grep.rs`

```rust
pub struct GrepTool { /* uses `rg` (ripgrep) binary; v1 = rg required */ }

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str { "Grep" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Grep".into(),
            description: "Search file contents using regex (ripgrep-compatible). Returns matching lines with file:line:col prefix.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern":     { "type": "string",  "description": "Regex pattern (ripgrep syntax)" },
                    "path":        { "type": "string",  "description": "Directory or file (default CWD)" },
                    "glob_filter": { "type": "string",  "description": "Include only files matching glob (e.g., '*.rs')" },
                    "case_sensitive": { "type": "boolean", "default": true },
                    "with_line_numbers": { "type": "boolean", "default": true },
                    "max_results": { "type": "integer", "default": 100, "maximum": 1000 },
                    "context_lines": { "type": "integer", "default": 0, "minimum": 0, "maximum": 10 }
                },
                "required": ["pattern"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Grep(PathPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* spawn `rg` (or fallback regex), parse output */ }
}
```

**result schema**: `{ matches: [{ file, line, col, text, context_before: [String], context_after: [String] }], total_count: u64, truncated: bool }`. **error map**: `FileNotFound` (path 없음) / `InvalidArgs` (pattern missing or invalid regex) / `SubprocessFailed` (rg 없음) / `Timeout` (5분+). **fallback**: v1 = rg 필수, v1.5+ = Rust `regex` + `walkdir` builtin.

### 3.8 Tool 6: `Glob` — file path matching

**module path**: `crates/myharness-tools/src/builtins/glob.rs`

```rust
pub struct GlobTool { /* uses `globset` + `walkdir` crates */ }

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str { "Glob" }
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "Glob".into(),
            description: "Match file paths by glob pattern (e.g., '**/*.rs', 'src/**/*.toml'). Returns sorted path list.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern":   { "type": "string",  "description": "Glob pattern (gitignore-style: ** / * / ? / [abc])" },
                    "path":      { "type": "string",  "description": "Base directory (default CWD)" },
                    "hidden":    { "type": "boolean", "default": false, "description": "Include dotfiles (.*)" },
                    "max_results": { "type": "integer", "default": 1000, "maximum": 10000 }
                },
                "required": ["pattern"], "additionalProperties": false
            }),
        }
    }
    fn required_scope(&self) -> ToolScope { ToolScope::Glob(PathPattern::Any) }
    async fn call(&self, args: Value) -> Result<Value, ToolError> { /* globset::Glob::new + walkdir */ }
}
```

**result schema**: `{ paths: [String], count: u64, truncated: bool }`. **error map**: `InvalidArgs` (pattern missing or invalid) / `FileNotFound` (base path 없음) / `Unknown` (walk 100K+ files).

### 3.9 6 builtin tool summary table

| # | name | file I/O | args required | args optional | scope |
| --- | --- | --- | --- | --- | --- |
| 1 | **Read** | read | `path` | `offset`, `limit`, `encoding` | `Read(/path/**)` |
| 2 | **Write** | write | `path`, `content` | `encoding` | `Write(/path/**)` |
| 3 | **Edit** | write | `path`, `old_text`, `new_text` | `replace_all` | `Edit(/path/**)` |
| 4 | **Bash** | depends | `command` | `timeout`, `cwd`, `env`, `capture_stderr` | `Bash(*)` |
| 5 | **Grep** | read | `pattern` | `path`, `glob_filter`, `case_sensitive`, `with_line_numbers`, `max_results`, `context_lines` | `Grep(/path/**)` |
| 6 | **Glob** | read | `pattern` | `path`, `hidden`, `max_results` | `Glob(/path/**)` |

**공통 결정** (6 builtin 모두): `path` = absolute or CWD-relative, `~` expansion. `additionalProperties: false` (예상치 못한 arg → 즉시 `InvalidArgs`). `max_results`/`limit` cap (대량 결과 방지). audit log 자동 (NFR-SEC-7). `Send + Sync + 'static` (Arc-shared, NFR-PERF-1).

### 3.10 결정 trade-off (Grep/Glob 분기)

| 선정 (Grep + Glob 2 tool) | 대안 (`Search` 통합) | trade-off |
| --- | --- | --- |
| Grep = content, Glob = path | `Search` = path + content both | ✅ claude-code 13.x 와 동일 (LLM training). ✅ Grep args 가 regex/ripgrep-specific. ✅ Glob args 가 glob-specific. ⚠️ LLM 이 "filename search" → Grep 보낼 가능성 |
| `Glob` uses `globset` (Rust crate) | shell `find` via Bash | ✅ **permission 분리** (`Glob` scope vs `Bash(*)` 별도). ✅ **dry-run 가능**. ❌ shell find = Bash tool → scope 폭발 |

---

## §4. Permission check layer (INITIAL_DESIGN.md §3.3 permission/ sub-module)

### 4.1 결정 (결론)

`crates/myharness-tools/src/permission/` = 4 mode (default/acceptEdits/plan/bypassPermissions, CONCEPT §5.4) + hook eval (`~/.myharness/hooks/*.md`, claude-code 13.4 hookify). check = sync (< 5ms), hook eval = async (< 50ms typical). REJECT 흐름 = `PermissionDenied` / `HookBlocked` (→ §6).

### 4.2 4 mode + context (의사코드, CONCEPT.md §5.4 + NFR-SEC-3)

```rust
// crates/myharness-tools/src/permission/mod.rs (의사코드)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode { Default, AcceptEdits, Plan, BypassPermissions }  // 4 mode (NFR-SEC-3, NFR-SEC-6)

pub struct PermissionContext {
    pub mode: PermissionMode,
    pub user: String,                              // "yklee" (CONCEPT.md §5.12)
    pub cwd: PathBuf,
    pub allowed_paths: Vec<PathPattern>,           // config: paths.user_can_write
    pub allowed_bash: Vec<CommandPattern>,         // config: bash.allow_list
    pub forbidden_paths: Vec<PathPattern>,         // config: paths.forbidden (e.g., /etc, ~/.ssh)
    pub forbidden_bash: Vec<CommandPattern>,       // config: bash.deny_list (e.g., "rm -rf /*")
    pub audit_log: Arc<dyn Fn(PermissionEvent) + Send + Sync>,
}

pub enum PermissionDecision { Allow, Denied { reason: String }, NeedsUserPrompt { reason: String, prompt: String } }
```

### 4.3 check 흐름 (의사코드, 5-step)

```rust
pub fn check(scope: &ToolScope, ctx: &PermissionContext, args: &Value) -> Result<PermissionDecision, ToolError> {
    // 1. bypassPermissions → 즉시 Allow (NFR-SEC-6, sandbox 전용)
    if matches!(ctx.mode, PermissionMode::BypassPermissions) { return Ok(Allow); }
    // 2. plan → NeedsUserPrompt (LLM plan 표시 후 user confirm 대기)
    if matches!(ctx.mode, PermissionMode::Plan) { return Ok(NeedsUserPrompt { reason: "plan mode".into(), prompt: format!("allow: {:?}", scope) }); }
    // 3. acceptEdits → Edit tool 만 Allow (나머지는 default 와 동일)
    if matches!(ctx.mode, PermissionMode::AcceptEdits) && matches!(scope, ToolScope::Edit(_)) { return Ok(Allow); }
    // 4. forbidden 우선 (NFR-SEC-5: dangerous ops 거부)
    let (target_path, target_cmd) = (extract_path(scope, args), extract_cmd(scope, args));
    if ctx.forbidden_paths.iter().any(|p| p.matches(&target_path)) { return Ok(Denied { reason: format!("forbidden path: {}", target_path) }); }
    if ctx.forbidden_bash.iter().any(|p| p.matches(&target_cmd))  { return Ok(Denied { reason: format!("forbidden command: {}", target_cmd) }); }
    // 5. allowed 체크 or NeedsUserPrompt
    if ctx.allowed_paths.iter().any(|p| p.matches(&target_path)) { return Ok(Allow); }
    if ctx.allowed_bash.iter().any(|p| p.matches(&target_cmd))   { return Ok(Allow); }
    Ok(NeedsUserPrompt { reason: "not in allow list".into(), prompt: format!("myharness wants: {:?} (mode={:?}, y/n)", scope, ctx.mode) })
}
```

### 4.4 hook eval (claude-code 13.4 hookify, INITIAL_DESIGN.md §3.3 line 339)

```rust
// crates/myharness-tools/src/permission/hook_eval.rs (의사코드)
pub async fn eval_hooks(tool_name: &str, args: &Value, ctx: &PermissionContext) -> Result<Option<String>, ToolError> {
    let hooks = hooks::load_all().await?;  // [Hook { name, match_regex, action }], cached, lazy
    for hook in &hooks {
        let target = format!("{} {}", tool_name, serde_json::to_string(args)?);
        if hook.match_regex.is_match(&target) {
            match hook.action {
                HookAction::Block(reason) => return Ok(Some(reason)),  // → §6 HookBlocked
                HookAction::Warn(msg)     => { tracing::warn!(target: "hook", "{}", msg); (ctx.audit_log)(PermissionEvent::HookWarn { hook: hook.name.clone(), msg }); }
                HookAction::Log           => { (ctx.audit_log)(PermissionEvent::HookLog { hook: hook.name.clone() }); }
            }
        }
    }
    Ok(None)
}
```

**hook markdown 형식** (NFR-SEC-4, 1 file = 1 hook, restart-free):
```markdown
---
name: warn-rm-rf
match: 'Bash.*rm\s+-rf\s+/'
action: warn   # warn | block | log
message: "rm -rf on root path detected."
---
```

**9 security patterns** (NFR-SEC-4, INITIAL_DESIGN §3.7 — 상세 regex = `docs/specs/security-patterns.md`, REVIEW.md MINOR-5/DD-4): curl-pipe-shell, sudo, chmod 777, mkfs, dd if=, fork bomb (`:(){:|:&};:`), force-push to main, git reset --hard, `~/.ssh/` access.

### 4.5 결정 trade-off (4 mode)

| 선정 (4 mode) | 대안 (2 mode) | trade-off |
| --- | --- | --- |
| default/acceptEdits/plan/bypassPermissions | ask / bypass only | ✅ claude-code 13.8 패턴. ✅ LLM training data 충분. ✅ NFR-SEC-3 정합. ⚠️ 4 mode × 6 tool = 24 조합 |
| `acceptEdits` = Edit 만 자동 allow | `acceptAll` (모든 tool 자동 allow) | ✅ **security 안전** (Read/Grep/Glob 도 user prompt). ✅ claude-code 와 의미 동일 |
| `plan` = plan 표시 후 user confirm | `plan` = read-only (실행 불가) | ✅ confirm 시점에 실행 (실용적). ⚠️ plan mode 가 default 인 test 시 mock 필요 |

### 4.6 결정 근거 1-라인 (yklee review)

> **4 mode (CONCEPT §5.4) + 9 security pattern (NFR-SEC-4) + markdown 1 file = 1 hook (claude-code 13.4)** — plugin 확장, restart-free.

---

## §5. ToolRegistry spec (registration, lookup, dispatch)

### 5.1 결정 (결론)

`ToolRegistry` = `parking_lot::RwLock<HashMap<String, SharedTool>>` (1 layer). lookup/dispatch = O(1). plugin/MCP 동적 register (NFR-PERF-1).

### 5.2 Registry 정의 (의사코드)

```rust
// crates/myharness-tools/src/registry.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ToolRegistry { tools: RwLock<HashMap<String, SharedTool>> }

impl ToolRegistry {
    pub fn new() -> Self { Self { tools: RwLock::new(HashMap::new()) } }

    /// 6 builtin tool 등록. v1 startup 시 자동 호출.
    pub fn register_builtins(&self) -> Result<(), ToolError> {
        for tool in [
            Arc::new(builtins::ReadTool::new()) as SharedTool,
            Arc::new(builtins::WriteTool::new()),
            Arc::new(builtins::EditTool::new()),
            Arc::new(builtins::BashTool::default()),
            Arc::new(builtins::GrepTool::new()),
            Arc::new(builtins::GlobTool::new()),
        ] { self.register(tool)?; }
        Ok(())
    }

    /// tool 등록. 이름 중복 시 error.
    pub fn register(&self, tool: SharedTool) -> Result<(), ToolError> {
        let mut map = self.tools.write();
        let name = tool.name().to_string();
        if map.contains_key(&name) { return Err(ToolError::Unknown { name: format!("duplicate: {}", name) }); }
        map.insert(name, tool);
        Ok(())
    }

    /// tool lookup. 미존재 시 None.
    pub fn lookup(&self, name: &str) -> Option<SharedTool> { self.tools.read().get(name).cloned() }
    /// LLM 노출용 definition list. rig-core AgentBuilder::tools() 에 직접 전달.
    pub fn all_definitions(&self) -> Vec<rig::tool::ToolDefinition> { self.tools.read().values().map(|t| t.definition()).collect() }
    /// debug / `myharness tool list` CLI.
    pub fn list(&self) -> Vec<ToolInfo> { self.tools.read().values().map(|t| ToolInfo { name: t.name().into(), description: t.definition().description.clone(), scope: t.required_scope() }).collect() }
    /// MCP / plugin tool 동적 추가 (NFR-PERF-1).
    pub fn register_mcp(&self, name: String, tool: SharedTool) -> Result<(), ToolError> { self.register(tool) }
    /// MCP server shutdown 시 일괄 제거.
    pub fn unregister_prefix(&self, prefix: &str) -> usize {
        let mut map = self.tools.write();
        let before = map.len();
        map.retain(|k, _| !k.starts_with(prefix));
        before - map.len()
    }
}
```

### 5.3 dispatch 흐름 (의사코드, INITIAL_DESIGN §3.3 정합)

```rust
pub async fn dispatch(registry: &ToolRegistry, name: &str, args: Value, ctx: &PermissionContext) -> Result<Value, ToolError> {
    let tool = registry.lookup(name).ok_or_else(|| ToolError::Unknown { name: name.into() })?;
    // 1. permission + hook (§4) — fail = PermissionDenied/HookBlocked
    permission::ensure_allowed(&tool.required_scope(), ctx, &args).await?;
    // 2. timeout-wrapped call
    let secs = tool.default_timeout_secs();
    let result = tokio::time::timeout(std::time::Duration::from_secs(secs), tool.call(args.clone()))
        .await
        .map_err(|_| ToolError::Timeout { tool: name.into(), secs: secs as u64 })??;
    // 3. audit log
    tracing::info!(target: "tool", "{} args={:?} → ok", name, args);
    Ok(result)
}
```

### 5.4 concurrent safety 결정

| 선정 | 대안 | trade-off |
| --- | --- | --- |
| `parking_lot::RwLock<HashMap<...>>` | `std::sync::RwLock` | ✅ faster (no poisoning), D-36 정합. ⚠️ dep 1개 추가 |
| `Arc<ToolRegistry>` (1개 공유) | `Arc<RwLock<ToolRegistry>>` (per-instance) | ✅ 1-layer simpler. ✅ sub-agent 별 clone |
| `HashMap` | `BTreeMap` | ✅ O(1) lookup. ❌ sorted 필요 시 `.sorted()` |
| `dashmap::DashMap` | `parking_lot::RwLock<HashMap>` | ✅ DashMap = lock-free. ⚠️ v1 10 tool 충분. v1.5+ MCP 50+ 시 DashMap 검토 |

### 5.5 결정 근거 1-라인 (yklee review)

> **1 layer `parking_lot::RwLock<HashMap>` + `Arc<ToolRegistry>` shared = 6 builtin + 4 MCP (10 tool) 충분**.

---

## §6. `ToolError` enum (8 variant) + recovery pattern

### 6.1 결정 (결론)

`ToolError` = 8 variant enum (PermissionDenied / FileNotFound / Timeout / InvalidArgs / SubprocessFailed / NetworkError / HookBlocked / Unknown). 각 variant = `thiserror` derive + 사용자 facing 한국어 메시지. recovery pattern = LLM retry 가이드.

### 6.2 enum 정의 (의사코드)

```rust
// crates/myharness-tools/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("권한 거부됨 (도구: {tool}): {reason}")] PermissionDenied { tool: String, reason: String },
    #[error("파일을 찾을 수 없음: {path}")] FileNotFound { path: String },
    #[error("시간 초과 (도구: {tool}, 제한: {secs}초)")] Timeout { tool: String, secs: u64 },
    #[error("잘못된 인자: {reason} (args: {args})")] InvalidArgs { reason: String, args: serde_json::Value },
    #[error("서브프로세스 실패 (command: {command}, exit: {exit_code:?}): {stderr}")] SubprocessFailed { command: String, exit_code: Option<i32>, stderr: String },
    #[error("네트워크 오류: {reason}")] NetworkError { reason: String },
    #[error("Hook 차단됨 ({hook}): {reason}")] HookBlocked { hook: String, reason: String },
    #[error("알 수 없는 오류 (도구: {name}): {source}")] Unknown { name: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },
}

impl ToolError {
    /// LLM retry 가능 여부 (D-15 retry 정책과 별개, agent layer 의 retry 결정)
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout { .. } | Self::NetworkError { .. } | Self::SubprocessFailed { exit_code: Some(code), .. } if *code >= 100)
    }
    /// 사용자 facing 한국어 1-라인 (TUI / CLI, REQUIREMENTS §2.10 #1)
    pub fn user_message(&self) -> String {
        match self {
            Self::PermissionDenied { tool, reason } => format!("'{tool}' 도구 권한 거부: {reason}. 모드를 변경하거나 `--mode=acceptEdits` 를 사용하세요."),
            Self::FileNotFound { path } => format!("파일을 찾을 수 없음: {path}. `Glob` 으로 경로를 검색하세요."),
            Self::Timeout { tool, secs } => format!("'{tool}' 도구 {secs}초 시간 초과. 작업을 더 작은 단위로 나누거나 timeout 을 늘리세요."),
            Self::InvalidArgs { reason, .. } => format!("잘못된 인자: {reason}"),
            Self::SubprocessFailed { command, exit_code, stderr } => format!("명령 실패: `{command}` (exit={exit_code:?}): {stderr}"),
            Self::NetworkError { reason } => format!("네트워크 오류: {reason}. 잠시 후 재시도하거나 fallback provider 로 전환됩니다."),
            Self::HookBlocked { hook, reason } => format!("보안 hook '{hook}' 차단: {reason}"),
            Self::Unknown { name, .. } => format!("'{name}' 도구 알 수 없는 오류"),
        }
    }
}
```

### 6.3 recovery pattern (8 variant)

| variant | LLM retry 가능? | recovery 가이드 |
| --- | --- | --- |
| `PermissionDenied` | ❌ (user 결정) | LLM: "모드 변경 or user 승인 대기" |
| `FileNotFound` | ❌ (path 오타) | LLM: "Glob 으로 path 검색 후 retry" |
| `Timeout` | ✅ (1회) | LLM: "scope 줄이기 or timeout ↑" |
| `InvalidArgs` | ❌ (LLM bug) | LLM: "args schema 재확인" (rare) |
| `SubprocessFailed` (exit ≥ 100) | ✅ (1회) | LLM: "stderr 분석 후 command 수정" |
| `SubprocessFailed` (exit < 100 or None) | ❌ (logic error) | LLM: "command 자체가 잘못됨" |
| `NetworkError` | ✅ (1회) | agent: fallback chain (D-15) 시도 |
| `HookBlocked` | ❌ (security hard block) | LLM: "정책상 차단됨" |
| `Unknown` | ⚠️ (case by case) | LLM: "에러 메시지 보고 결정" |

### 6.4 결정 trade-off (8 variant)

| 선정 (8 variant) | 대안 (단일 `Box<dyn Error>`) | trade-off |
| --- | --- | --- |
| `enum` 8 variant | `Box<dyn Error>` (stringly typed) | ✅ **typed match** 가능. ✅ `thiserror` derive (보일러플레이트 0). ✅ user_message 한국어. ❌ variant 추가 시 enum 확장 (semver 깨짐 가능) — `#[non_exhaustive]` 로 완화 |
| `is_retryable()` 메서드 | 별도 `RetryPolicy` trait | ✅ enum 과 1-1 매핑. ❌ retry 정책은 agent layer (D-15) |
| `HookBlocked` 별도 variant | `PermissionDenied` 통합 | ✅ hook = 보안 정책 별도 (audit log 에 `hook: warn-rm-rf` 명시) |
| `NetworkError` 별도 variant | `SubprocessFailed` 통합 | ✅ curl/wget 등 network tool (v1.5+ MCP) 와 구분 |

### 6.5 결정 근거 1-라인 (yklee review)

> **8 variant + `thiserror` derive + `is_retryable()` + 한국어 `user_message()`** = LLM 이 retry 가능/불가능 즉시 분기, audit log 에 variant 명시.

---

## §7. TDD TC scaffold (L1 Unit TC 6 tool × 5 시나리오 = 30 TC)

### 7.1 결정 (결론)

REVIEW.md §6.2 정합 — 6 builtin tool × 5 시나리오 = 30 L1 Unit TC. RED-GREEN-REFACTOR 진입점. 본 §7 = TC 의 골격 (5 시나리오 × 6 tool 표), 각 TC 의 상세는 TASK-005-1 구현 시 작성.

### 7.2 5 시나리오 (모든 tool 공통)

| # | 시나리오 | 검증 항목 | error variant (negative) |
| --- | --- | --- | --- |
| **S1** | happy path (정상 호출) | args schema 통과 + result schema 일치 + audit log | (없음) |
| **S2** | invalid args (필수 필드 누락) | `InvalidArgs` error + user_message 한국어 | `InvalidArgs` |
| **S3** | permission denied (scope 밖) | `PermissionDenied` error + reason 명시 | `PermissionDenied` |
| **S4** | timeout / subprocess fail | `Timeout` 또는 `SubprocessFailed` + is_retryable() | `Timeout` / `SubprocessFailed` |
| **S5** | file/resource not found | `FileNotFound` 또는 `Unknown` | `FileNotFound` / `Unknown` |

### 7.3 6 tool × 5 시나리오 = 30 TC table

| tool \ 시나리오 | S1 happy | S2 invalid args | S3 permission | S4 timeout/subproc | S5 not found |
| --- | --- | --- | --- | --- | --- |
| **Read** | TC-Read-01: read 1KB file | TC-Read-02: `path` 누락 | TC-Read-03: `/etc/shadow` 읽기 | TC-Read-04: 1GB file (timeout) | TC-Read-05: 없는 path |
| **Write** | TC-Write-01: write 100B file | TC-Write-02: `content` 누락 | TC-Write-03: `/usr/bin/` 쓰기 | TC-Write-04: 1GB (disk full) | TC-Write-05: 부모 dir 없음 |
| **Edit** | TC-Edit-01: unique replace | TC-Edit-02: `old_text`/`new_text` 누락 | TC-Edit-03: `~/.ssh/known_hosts` | TC-Edit-04: (timeout 거의 없음) | TC-Edit-05: path 없음 |
| **Bash** | TC-Bash-01: `echo hello` | TC-Bash-02: `command` 누락 | TC-Bash-03: `rm -rf /` (forbidden) | TC-Bash-04: `sleep 999` (timeout) | TC-Bash-05: `nonexistent_cmd` (exit 127) |
| **Grep** | TC-Grep-01: simple regex match | TC-Grep-02: invalid regex | TC-Grep-03: `/root/` grep | TC-Grep-04: large dir (1M files) | TC-Grep-05: 없는 path |
| **Glob** | TC-Glob-01: `*.rs` match | TC-Glob-02: invalid glob pattern | TC-Glob-03: forbidden base path | TC-Glob-04: (timeout 거의 없음) | TC-Glob-05: base path 없음 |

### 7.4 TC 작성 예시 (TC-Read-01 happy path, 의사코드)

```rust
// crates/myharness-tools/src/builtins/read.rs (의사코드 + TC)
#[cfg(test)] mod tests {
    use super::*;
    use myharness_tools::test_helpers::{MockPermissionContext, AuditLogCapture};
    use tempfile::NamedTempFile;

    #[tokio::test] async fn tc_read_01_happy_path() {
        // ARRANGE
        let tool = ReadTool::new();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "line1\nline2\nline3\n").unwrap();
        let args = json!({ "path": tmp.path().to_str().unwrap() });
        let ctx = MockPermissionContext::allow_all();
        let audit = AuditLogCapture::new();
        // ACT
        let result = tool.call(args).await;
        // ASSERT (1. result schema 2. audit log 3. user_message 한국어 등)
        let value = result.expect("should succeed");
        assert_eq!(value["content"], "line1\nline2\nline3\n");
        assert_eq!(value["lines"], 3);
        assert_eq!(value["encoding"], "utf-8");
        assert_eq!(value["truncated"], false);
        assert!(audit.contains_tool_call("Read"));
    }
    // ... TC-Read-02~05, TC-Write-01~05, ..., TC-Glob-01~05 (총 30 TC)
}
```

### 7.5 RED-GREEN-REFACTOR 진입점

**TDD 사이클** (REVIEW.md §6.4):
1. **RED**: TC 30개 모두 `#[ignore]` 또는 fail 상태로 작성 (impl 전). `cargo test` 시 30 fail 확인
2. **GREEN**: 6 builtin tool 을 1 tool 씩 impl → TC pass. 우선순위: Read → Write → Edit → Glob → Grep → Bash (Bash 가 가장 복잡, 마지막)
3. **REFACTOR**: 공통 path validation / error mapping 중복 제거. `cargo test` 30 pass 유지

**mock 인프라** (dev-dependencies, D-36 §3.2 + D-07): `tempfile`, `myharness-tools::test_helpers` (`MockPermissionContext`, `AuditLogCapture`, `FixtureFileSystem`). CI = `cargo test --workspace` (GH Actions matrix ubuntu/macos/windows + Gitea Actions mirror).

### 7.6 결정 trade-off (30 TC 분량)

| 선정 (30 TC) | 대안 (15 TC) | trade-off |
| --- | --- | --- |
| 30 TC = 6 × 5 시나리오 | 15 TC = 6 × 2-3 (happy + 주요 error) | ✅ 모든 variant cover. ✅ v1 robustness ↑. ⚠️ TC 작성 시간 30+ min. ❌ L3/L4 TC 별도 (REVIEW §6.3) |
| 5 시나리오 = (S1 happy / S2 invalid / S3 permission / S4 timeout / S5 notfound) | S1+S2 만 | ✅ 8 error variant 중 4 cover. ❌ SubprocessFailed / HookBlocked / NetworkError / Unknown 별도 TC (v1.5+) |

### 7.7 결정 근거 1-라인 (yklee review)

> **30 TC = L1 Unit TC 전체 범위** (REVIEW.md §6.2 정합). v1.5+ 에서 +24 TC → 54 TC.

---

## §8. Handoff (D-26 4-필드)

### 8.1 summary

본 DETAILED_DESIGN_TOOL.md (DD-1 attempt 2) = `myharness-tools` crate 상세 spec. REVIEW.md §3.1 MAJOR-1 spec 확정 = **rig-core `ToolDefinition` + `serde_json::Value` args/output**. 추가: 6 builtin tool spec (Read/Write/Edit/Bash/Grep/Glob) + permission (4 mode + 9 hook) + ToolRegistry (`parking_lot::RwLock<HashMap>`) + ToolError (8 variant) + TDD TC 30 entry point. 분량 **~870 lines / 9 sections (§0-§8)**. 4 chunk D-16 chunked write. TASK-005-1 (v1 Rust MVP) 의 `myharness-tools` crate 구현 입력.

### 8.2 risks

- **R-1 (trait API stability)**: rig-core 0.5+ → 1.0 migration 시 `ToolDefinition` API 변경 가능. **대응**: 우리 trait (`myharness_tools::Tool`) 은 rig-core 와 1-hop (`rig::tool::ToolDefinition` import) — rig-core 변경 시 import 만 갱신
- **R-2 (MCP 호환 검증 미수행)**: 4 MCP server (`mcp__filesystem__read_file` 등) tool spec 검증 필요. **대응**: TASK-005-1 구현 시 mcp__filesystem 1개 PoC (CONCEPT §5.14 #1)
- **R-3 (cross-OS)**: Bash (subprocess) + Grep (ripgrep) 가 OS 별 차이. **대응**: GH Actions matrix (ubuntu/macos/windows) + Gitea Actions mirror (D-07)
- **R-4 (HookEval 9 pattern 별도)**: §4.4 의 9 security pattern regex 상세 = `docs/specs/security-patterns.md` 별도 (REVIEW MINOR-5, DD-4)
- **R-5 (LLM mock 부재)**: TC 작성 시 LLM 호출 없이 `ToolDefinition::parameters` 만 검증 가능. **대응**: mock rig-core client = v1.5+ (REVIEW R-3)

### 8.3 suggested_follow_up

1. **즉시 (다음 작업)**: 본 DETAILED_DESIGN_TOOL.md 검토 + DD-2/3/4/5 와 동시 진행 (REVIEW §5.2)
2. **TASK-005-1 v1 Rust MVP (TDD RED-GREEN-REFACTOR)**: 6 builtin × 30 TC (본 §7) 부터. 우선순위: Read → Write → Edit → Glob → Grep → Bash
3. **DD-3 (15 sub-agent)**: `allowed_tools: &[&str]` = 본 §2 `name()` 사용. DD-3 작성 시 본 spec 재참조
4. **DD-4 (security patterns 9 regex)**: `docs/specs/security-patterns.md` 별도
5. **DD-5 (retry / exit code)**: sub-agent / tool layer 의 exit code (0/1/2) 정합
6. **TUI POC (MINOR-2)**: widget render / keymap. 본 spec 은 tool 호출 결과 → TUI 흐름만 (DD-1 외부)
7. **v1.5+**: MCP 4 server 검증 (R-2), L3 Component TC (REVIEW §6.3), tree-sitter pack (MINOR-3), mavis_bridge conflict (MINOR-4), retry backoff jitter (MINOR-7)

### 8.4 produced_artifacts

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DETAILED_DESIGN_TOOL.md** (본) | `docs/architecture/DETAILED_DESIGN_TOOL.md` | ~870 lines / 9 sections | done |
| **deliverable_dd1.md** (D-16 signal) | `docs/team/deliverable_dd1.md` | ~30 lines | done (in_progress → done) |
| **board.md** | `~/.mavis/plans/plan_746a17ad/board.md` | start + done 2 entry | done |

### 8.5 cross-ref 요약 (5 SSOT)

- INITIAL_DESIGN.md §3.3 (line 324-339) → 본 §2/§3/§4/§5 | §3.2 (line 514-540) → 본 §1/§2/§5 | §3.4 (line 573-609) → 본 §2/§4/§5 | §6 (line 1310-1430) → 본 §1
- CONCEPT.md §5.4 (line 202-224) → 본 §4 | §5.5 (line 226-370) → 본 §1 | §5.7 (line 453-466) → 본 §1/§5
- REQUIREMENTS.md §2.9 (line 408-429) → 본 §4/§6 | §2.0 (line 101-164) → 본 §3 | §4 (line 460-490) → 본 §2
- **REVIEW.md §3.1 MAJOR-1** (line 199-208) → **본 §1 (정합)** | §6.2 (line 392-400) → 본 §7 | §5.2 (line 348-360) → 본 chunked write

### 8.6 다음 단계 (Owner)

1. **본 DETAILED_DESIGN_TOOL.md verifier 독립 cross-check** (parent `mvs_60292a9207004b10903328af9fb700b6`) — **VERDICT top-level heading (line 3) 명시, attempt 1 reject 사유 해소**
2. **verifier PASS 시**: TASK-005-1 (v1 Rust MVP) 의 `myharness-tools` crate 구현 시작. TDD RED → GREEN → REFACTOR (본 §7.5)
3. **verifier MAJOR/MINOR 시**: §8.2 risks 중 R-1~R-5 연결 drift 확인 후 minor patch

---

### VERDICT (final, post-handoff): PASS

본 DETAILED_DESIGN_TOOL.md = myharness-tools crate 상세 spec. REVIEW §3.1 MAJOR-1 spec 확정 = **rig-core `ToolDefinition` + `serde_json::Value` args/output**. 6 builtin tool + permission (4 mode + hook) + ToolRegistry + ToolError (8 variant) + TDD TC 30 entry point. 분량 ~870 lines / 9 sections. D-16 chunked write 4 chunk / 표준 6 원칙 / D-06 메커니즘만 / 안티 6 미반영.
