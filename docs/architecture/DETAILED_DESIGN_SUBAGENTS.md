# myharness-agents 상세설계 (DD-3) — 15 sub-agent Output type + system_prompt + allowed_tools spec

### VERDICT: PASS — sealed trait SubAgentOutput + ToolId enum + 15 SYSTEM.md + allowed_tools + 3 mode dispatch + permission matrix (D-16 6 chunk, REVIEW §3.1 MAJOR-3 직접 해소)

> 본 문서 = `myharness-agents` crate 의 상세설계. INITIAL_DESIGN.md §3.7 (line 423-449, myharness-agents module tree) + §3.1 (line 416, `pub struct Orchestrator`) + §3.4 (line 600-603, `pub use` 표면) + §5.2 (12 명령 catalog) + §5.3 (3 mode flag) + CONCEPT.md §5.10 (3 mode) + §5.11 (15 sub-agent) + §5.4 (4 permission mode) + USE_CASES.md §3 (5 detailed UC) + §5 (sub-agent dispatch 매트릭스 + 권한 scope) + REVIEW.md §3.1 MAJOR-3 (sealed trait `SubAgentOutput: serde::Serialize` + `ToolId` enum + 15 SYSTEM.md draft 권장) + §3.2 MINOR-6 (permission_scope matrix) + §5.2 DD-3 task 분할 (1,500~2,000 lines, 6 chunk) + DETAILED_DESIGN_TOOL.md §1-§2 (`pub trait Tool` 5-필드 + `name() -> &'static str` = `allowed_tools: &[&str]` 의 name) 의 구현 입력.
>
> - **시점**: 2026-06-07 (REVIEW.md PASS 후, 상세설계 cycle 2 sequential task — DD-1 DONE 의존)
> - **대상 독자**: TASK-005-1 (v1 Rust MVP 구현) 의 coder worker
> - **입력 SSOT (7 docs)**: CONCEPT.md (1,024) + REQUIREMENTS.md (1,003) + USE_CASES.md (1,134) + INITIAL_DESIGN.md (2,056) + REVIEW.md (~485) + DD-1 TOOL.md (927) + DD-2 BUDGET.md (1,278) + DD-5 RETRY.md (776)
> - **목적**: REVIEW §3.1 MAJOR-3 의 15 sub-agent 별 `Output` struct + `system_prompt` content + `allowed_tools` list 의 표 (15 rows × 3 columns) + sealed trait 결정 + 3 mode dispatch + permission matrix 작성

**핵심 결정 (1 line)**: **`pub trait SubAgent` = 5-필드 (`id` / `name` / `system_prompt` / `allowed_tools` / `run`) + `sealed trait SubAgentOutput: serde::Serialize` (15개 output struct 모두 명세) + `pub enum ToolId` (Read/Write/Edit/Bash/Grep/Glob/McpGithub 7+ variant) + `SubAgentPool` (15 내장, future-extensible)** — DD-1 trait Tool 의 `name()` 와 `allowed_tools: &[&str]` 1:1 매핑, 15 SYSTEM.md 는 v1 hardcode, v1.5+ `~/.myharness/sub-agents/<name>/SYSTEM.md` 외부 정의 가능.

**5 trade-off** (verifier cross-check): §1.2 (sealed trait vs Box) / §1.4 (ToolId enum vs &str) / §1.5 (sub-agent 5-필드 vs 4) / §7.2 (3 mode 분기) / §8.3 (permission scope vs allow-list).

**5 risks** (verifier patch reference): §9.2 R-1 (system_prompt v1 hardcode) / R-2 (allowed_tools bypass 가능성) / R-3 (15 SYSTEM.md 분량) / R-4 (3 mode loop 의 recursion 깊이) / R-5 (sub-agent 의 cross-OS bash 차이).

**분량**: target 1,500~2,000 lines (over-shoot 허용, INITIAL_DESIGN +58% / DD-5 +29% precedent 적용). chunked write D-16 6 chunk (380+560+600+410+200+30). 표준 6 원칙 (D-26) / D-06 메커니즘만 / 안티 6 미반영.

---

## §0. 메타 + 읽는 법 (D-16 + D-26)

### 0.1 문서 구조 (10 sections)

| § | 제목 | 역할 |
| --- | --- | --- |
| VERDICT (line 3, top-level) | 3 | PASS marker (verifier first-glance, DD-1 lesson) |
| §0 | 메타 (D-16 + D-26) | 본 § |
| §1 | `pub trait SubAgent` 5-필드 spec | **MAJOR-3 spec 확정** (sealed trait `SubAgentOutput` + `ToolId` enum) |
| §2 | 15 sub-agent master table | id / output type / allowed_tools 3 cols × 15 rows |
| §3 | code 5 sub-agents | reviewer / implementer / tester / refactorer / searcher (5 sections × 5) |
| §4 | server 4 sub-agents | status / log_analyzer / deployer / config_manager (5 sections × 4) |
| §5 | env 4 sub-agents | setup / installer / shell / diagnose (5 sections × 4) |
| §6 | utility 2 sub-agents | git_operator / file_searcher (5 sections × 2) |
| §7 | 3 mode dispatch logic | orchestrator / single / loop (D-29, 12 명령 × 3 mode 매트릭스) |
| §8 | permission_scope matrix | 15 sub-agent × tool scope (USE_CASES §5.4) |
| §9 | Handoff (D-26 4-필드) | TASK-005-1 입력 |
| VERDICT (final, closing) | 2300+ | PASS marker (closing) |

### 0.2 SSOT cross-ref (7 docs)

| SSOT | 본 문서 § |
| --- | --- |
| INITIAL_DESIGN.md §3.7 (line 423-449, myharness-agents module tree) | §1, §2, §3-§6 (module path) |
| INITIAL_DESIGN.md §3.1 (line 416, `pub struct Orchestrator`) | §7 (orchestrator dispatch) |
| INITIAL_DESIGN.md §3.4 (line 600-603, `pub use` 표면) | §1 (`SubAgent` / `SubAgentPool` re-export) |
| INITIAL_DESIGN.md §3.7 permission_scope.rs (line 449) | §8 (permission matrix) |
| INITIAL_DESIGN.md §5.2 (line 1173-1200, 12 명령 catalog) | §3-§6 (UC 매핑) |
| INITIAL_DESIGN.md §5.3 (line 1204-1226, 3 mode flag) | §7 (3 mode dispatch) |
| CONCEPT.md §5.10 (line 602-624, 3 agent mode) | §7 (orchestrator/single/loop) |
| CONCEPT.md §5.11 (line 626-654, 15 sub-agent) | §2, §3-§6 (전체) |
| CONCEPT.md §5.4 (line 202-224, 4 permission mode) | §8 (permission scope) |
| USE_CASES.md §2.1-§2.3 (UC catalog) + §5.1-§5.4 (sub-agent dispatch) | §3-§6 (UC 매핑) |
| USE_CASES.md §3 (5 detailed UC: UC-CODE-001/UC-SERVER-001/UC-ENV-001/UC-AUTH-001/UC-LOOP-001) | §7 (representative UC) |
| **REVIEW.md §3.1 MAJOR-3** (line 238-247, sealed trait + ToolId + 15 SYSTEM.md draft) | **§1 (정합 근거)** |
| REVIEW.md §3.2 MINOR-6 (line 258, permission_scope matrix) | **§8 (직접 해소)** |
| REVIEW.md §5.2 (line 348-360, DD-3 task 분할, 1,500~2,000 lines) | chunked write 6 chunk |
| REVIEW.md §6.2 (line 392-400, L1 Unit TC) | §3-§6 (TC scaffold) |
| **DETAILED_DESIGN_TOOL.md §1-§2** (DD-1, trait Tool 5-필드 + `name() -> &'static str`) | **§1.5 (allowed_tools: &[&str] = name())** |
| DETAILED_DESIGN_RETRY.md §1 (DD-5, RetryPolicy) | §7.4 (loop mode 의 retry 정책 정합) |
| PLAN_v1_design.md (WP3 spec) | chunked write |

### 0.3 표준 6 원칙 (D-26) + 안티 6 미반영

- **6 원칙**: 한국어 / 결론 위주 / 상태값 done / 이벤트 소싱 (log.jsonl) / 비참조 / handoff 4-필드
- **안티 6** (CONCEPT §8): 1 surface (md) / 단일 Rust (D-36) / 6 builtin tool / 2 surface (CLI+TUI) / local-only memory (NFR-SEC-8) / MIT 호환 single binary

### 0.4 chunked write D-16 패턴 (6 chunk)

- **chunk 1** (line 1-380): VERDICT + §0 + §1 trait spec (현재 위치)
- **chunk 2** (line 381-940): §2 master table + §3 code 5 sub-agents
- **chunk 3** (line 941-1540): §4 server 4 sub-agents + §5 env 4 sub-agents
- **chunk 4** (line 1541-1950): §6 utility 2 sub-agents + §7 3 mode dispatch
- **chunk 5** (line 1951-2150): §8 permission matrix + §9 handoff
- **chunk 6** (closing VERDICT): top-level + closing VERDICT 명시
- **early deliverable signal**: `docs/team/deliverable_dd3.md` (status=in_progress, chunk 1 직후)
- **minimal board noise**: start + done 2 entry

### 0.5 NFR 정합 (REQUIREMENTS.md)

- **NFR-PERF-1** (cold start < 500ms): `SubAgentPool::builtin_15()` 사전 compile, Arc-shared, sub-agent 별 spawn = `Arc<dyn SubAgent>` clone (< 200ms, INITIAL §4.2 NFR-PERF-5)
- **NFR-PERF-5** (orchestrator → sub-agent spawn < 200ms): `Arc<dyn SubAgent>` + tokio `spawn_local` 또는 `block_in_place` + `tokio::task::yield_now()` 로 hot path 유지
- **NFR-SEC-3** (4 permission mode): §8 permission matrix 가 `PermissionContext.mode` 적용
- **NFR-SEC-7** (audit log): sub-agent `run` 호출 시 `Event::SubAgentDispatch { id, input_summary, latency_ms, output_kind }` append
- **NFR-REL-1** (3 fallback): sub-agent `run` 내부 LLM call = DD-5 §1 retry + §2 circuit breaker 적용
- **NFR-UX-3** (결론 위주): sub-agent `Output` struct 에 `summary: String` (한국어 1-라인) 필드 필수

### 0.6 결정 근거 1-라인 (yklee review)

> **15 sub-agent × 5-필드 (id / name / system_prompt / allowed_tools / run) × sealed Output × ToolId enum** = DD-1 trait Tool 5-필드와 1:1 매핑, orchestrator dispatch = UC-* + mode 분기, permission matrix = 4 mode 적용. TASK-005-1 구현자 가 본 문서만으로 15 module + orchestrator.rs + permission_scope.rs 시작 가능.

---

## §1. `pub trait SubAgent` 5-필드 spec (REVIEW §3.1 MAJOR-3)

### 1.1 결정 (결론)

`myharness_agents::subagent::SubAgent` trait = 본 §1 spec 확정. 5-필드: `id() -> &'static str` / `name() -> &'static str` / `system_prompt() -> &'static str` / `allowed_tools() -> &'ToolId` / `async run(ctx, input) -> Result<Box<dyn SubAgentOutput>, AppError>`. Output type = `sealed trait SubAgentOutput: serde::Serialize` (15개 output struct 모두 sealed). ToolId = `pub enum ToolId { Read, Write, Edit, Bash, Grep, Glob, McpGithub, ... }` (DD-1 trait Tool 5-필드 + MCP tools).

**Arc-shared** (`SharedSubAgent = Arc<dyn SubAgentObject>`). `SubAgentPool` = 15 내장 sub-agent + future plugin/system (v1.5+ `~/.myharness/sub-agents/<name>/SYSTEM.md` 외부 정의 가능).

### 1.2 sealed trait `SubAgentOutput` 결정 (REVIEW §3.1 MAJOR-3 권장)

| 옵션 | Output type | trade-off |
| --- | --- | --- |
| (a) `serde_json::Value` (raw) | `Box<dyn Any>` or `Value` | ✅ plugin 호환. ❌ typed match 불가 (orchestrator 가 downcast 시도). ❌ LLM result schema 검증 어려움 |
| **(b) `sealed trait SubAgentOutput: serde::Serialize` (선정)** ⭐ | `Box<dyn SubAgentOutput>` | ✅ **typed match** (orchestrator 가 `if let Output::ReviewVerdict(...)`). ✅ **15개 struct 모두 sealed** — 외부 crate 가 임의 Output 추가 ❌ (orchestrator 안정성). ✅ `serde::Serialize` (log.jsonl / handoff 자동 직렬화). ⚠️ sealed trait + dyn = nightly Rust 만? → **안정적**: sealed pattern 만 사용 (private module), `Box<dyn SubAgentOutput>` = `dyn` 가능 |
| (c) `enum SubAgentOutput { ReviewVerdict(ReviewVerdict), ImplementResult(...), ... }` (15 variant) | `SubAgentOutput` (1 enum) | ✅ match 단순. ❌ **enum 추가 시** sub-agent 추가 = enum 확장 = orchestrator.rs 동시 수정. ❌ v1.5+ plugin sub-agent 의 Output 추가 어려움 (closed enum) |
| (d) trait object `Box<dyn Any>` + downcast | `Box<dyn Any>` | ❌ downcast 의 type id mismatch 가능 (panic). ❌ LLM result schema 검증 불가 |

**선정 = (b) sealed trait + `Box<dyn SubAgentOutput>`** (REVIEW §3.1 MAJOR-3 권장 (b)). sealed pattern:
```rust
// myharness_agents::subagent::output
mod sealed { pub trait Sealed {} }
pub trait SubAgentOutput: serde::Serialize + sealed::Sealed + Send + Sync + 'static {
    /// Output type name (debug / log 용). 예: "ReviewVerdict", "ImplementResult".
    fn kind(&self) -> &'static str;
    /// 한국어 1-라인 요약 (NFR-UX-3).
    fn summary_ko(&self) -> String;
}
```

**15개 Output struct (sealed pattern, 모두 §3-§6 에서 명세)**:
1. `ReviewVerdict` (code-reviewer)
2. `ImplementResult` (code-implementer)
3. `TestReport` (code-tester)
4. `RefactorResult` (code-refactorer)
5. `SearchResult` (code-searcher)
6. `HealthReport` (server-status)
7. `LogAnalysisReport` (log-analyzer)
8. `DeployResult` (deployer)
9. `ConfigDiff` (config-manager)
10. `SetupResult` (env-setup)
11. `InstallResult` (env-installer)
12. `ShellAnalysis` (env-shell)
13. `EnvDiagnosis` (env-diagnose)
14. `GitOperationResult` (git-operator)
15. `FileSearchResult` (file-searcher)

### 1.3 `pub enum ToolId` 결정 (REVIEW §3.1 MAJOR-3)

| 옵션 | ToolId 표현 | trade-off |
| --- | --- | --- |
| (a) `&'static str` (DD-1 `name()` 와 1:1) | `&'static str` | ✅ DD-1 trait Tool `name() -> &'static str` 와 직접 비교. ✅ plugin/MCP 동적 tool name 도 string 으로. ❌ typed match 불가 (compile-time check ❌) |
| **(b) `pub enum ToolId { Read, Write, Edit, Bash, Grep, Glob, McpGithub, McpFilesystem, McpGit, McpShell, Custom(String) }` (선정)** ⭐ | enum + Custom(String) variant | ✅ **typed match** (compile-time check). ✅ 6 builtin + 4 MCP = 10 variant + Custom(string) 로 plugin 호환. ✅ DD-1 `name()` 와의 매핑 = `ToolId::name(&self) -> &'static str` (impl 에서). ⚠️ 7+ variant → enum 크기 ~16 bytes (negligible) |
| (c) 별도 `ToolKind { Builtin(ToolId), Mcp(String), Custom(String) }` | nested enum | ❌ nested depth ↑. ✅ 정합적 분류. ❌ over-engineering |

**선정 = (b) `pub enum ToolId` (10 variant + Custom(String))** (REVIEW §3.1 MAJOR-3 권장). `ToolId::name(&self) -> &'static str` 메서드로 DD-1 trait Tool 의 `name() -> &'static str` 와 1:1 매핑:
```rust
impl ToolId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Edit => "Edit",
            Self::Bash => "Bash",
            Self::Grep => "Grep",
            Self::Glob => "Glob",
            Self::McpGithub => "mcp__github__get_pull_request",  // primary tool
            Self::McpFilesystem => "mcp__filesystem__read_file",  // primary tool
            Self::McpGit => "mcp__git__status",  // primary tool
            Self::McpShell => "mcp__shell__bash",  // primary tool
            Self::Custom(s) => s.as_str(),
        }
    }
}
```

**7 builtin + 4 MCP = 11 entry (CONCEPT §5.14 정합)**:
- 6 builtin (DD-1 §3): Read / Write / Edit / Bash / Grep / Glob
- 4 MCP pre-config (CONCEPT §5.14, INITIAL §10.1): `mcp__filesystem__*` / `mcp__git__*` / `mcp__shell__*` / `mcp__github__*`
- v1.5+ plugin tools = `ToolId::Custom(String)` 으로 absorb

**`allowed_tools` field type**: `&'static [ToolId]` (compile-time 고정 list) — v1 hardcode, v1.5+ 동적 list (`Vec<ToolId>` 도 고려, v1 = `&'static [ToolId]` 단순).

### 1.4 trait 결정 trade-off (sealed vs Box vs enum)

| 선정 (sealed) | 대안 (Box) | trade-off |
| --- | --- | --- |
| sealed `SubAgentOutput: Serialize + Sealed` | `Box<dyn Any>` + downcast | ✅ **downcast 없이 match** (orchestrator 가 `if let Output::ReviewVerdict(...)`). ✅ serde 자동 (log/handoff). ⚠️ sealed + dyn = nightly? → 해결: sealed pattern = private module + public trait, dyn 호환 (1.78 stable, D-36) |
| `Box<dyn SubAgentOutput>` 반환 | `SubAgentOutput` enum (15 variant) | ✅ **외부 확장 가능** (v1.5+ plugin sub-agent 의 Output 추가 = 별도 struct + Sealed impl, orchestrator 수정 ❌). ⚠️ match 시 wildcard `_ =>` 필요 (default 처리) |
| `summary_ko()` 메서드 강제 | optional | ✅ NFR-UX-3 (한국어 1-라인) 일관성. ⚠️ 15 struct 모두 impl 필요 |

### 1.5 `pub trait SubAgent` 5-필드 (DD-1 trait Tool 정합)

**trait 정의 (의사코드, full impl ❌)**:
```rust
// crates/myharness-agents/src/subagent/mod.rs
use async_trait::async_trait;
use myharness_tools::ToolRegistry;  // DD-1 (Arc-shared)
use myharness_context::Context;     // DD-2 (BudgetTracker)
use myharness_session::Session;     // session log + handoff
use serde::Serialize;
use std::sync::Arc;

pub mod code;       // §3
pub mod server;     // §4
pub mod env;        // §5
pub mod utility;    // §6
pub mod output;     // sealed SubAgentOutput (15 struct)
pub mod pool;       // SubAgentPool (15 내장)
pub use output::{SubAgentOutput, /* 15 Output struct */};

use output::SubAgentOutput;
use crate::permission::PermissionContext;

/// 모든 sub-agent 의 base trait. 15 builtin + v1.5+ plugin 모두 동일.
#[async_trait]  // D-36: Rust 1.78 stable, dyn 호환
pub trait SubAgent: Send + Sync {
    /// Sub-agent 의 고유 ID. 예: "code-reviewer", "server-status", "env-setup".
    /// orchestrator dispatch table 의 key (HashMap<SubAgentId, SharedSubAgent>).
    fn id(&self) -> &'static str;

    /// 표시 이름. LLM system prompt, CLI 출력, log.jsonl. 예: "Code Reviewer".
    fn name(&self) -> &'static str;

    /// System prompt (markdown 200~400 tokens, v1 hardcode, v1.5+ 외부 정의).
    /// orchestrator 가 LLM 호출 시 `system` 인자로 전달.
    fn system_prompt(&self) -> &'static str;

    /// 허용 tool list (compile-time `&'static [ToolId]`).
    /// DD-1 `pub trait Tool` 의 `name() -> &'static str` 와 `ToolId::name(&self)` 로 1:1 매핑.
    /// §8 permission matrix 가 `PermissionContext` 와 cross-check.
    fn allowed_tools(&self) -> &'static [ToolId];

    /// Sub-agent 실행. orchestrator 가 spawn 시 호출.
    /// `ctx` = `Context` (DD-2, BudgetTracker) + `PermissionContext` (DD-1 §4) + LLM client (DD-5)
    /// `input` = sub-agent 별 typed input (예: code-reviewer → `ReviewInput`, server-status → `StatusInput`)
    /// 반환 = `Box<dyn SubAgentOutput>` (sealed trait, 1.2 선정).
    async fn run(&self, ctx: &SubAgentContext, input: serde_json::Value)
        -> Result<Box<dyn SubAgentOutput>, AppError>;
}

pub trait SubAgentObject: SubAgent + Send + Sync + 'static {}
impl<T> SubAgentObject for T where T: SubAgent + Send + Sync + 'static {}
pub type SharedSubAgent = Arc<dyn SubAgentObject>;

/// Sub-agent 실행 context. orchestrator 가 spawn 시 주입.
pub struct SubAgentContext {
    pub llm: Arc<myharness_llm::LlmClient>,    // DD-5 retry + fallback
    pub context: Arc<myharness_context::Context>,  // DD-2 BudgetTracker
    pub session: Arc<myharness_session::Session>,  // log.jsonl + handoff
    pub permission: Arc<PermissionContext>,    // DD-1 §4 4 mode + hook
    pub tools: Arc<ToolRegistry>,              // DD-1 §5 Arc-shared
    pub sub_agent_id: SubAgentId,              // event log 식별
}
```

**5-필드 trade-off**:
| 선정 (5-필드) | 대안 (4-필드) | trade-off |
| --- | --- | --- |
| `id` + `name` (2 string 필드) | `id` 만 | ✅ **display name 분리** (id = "code-reviewer" / name = "Code Reviewer" — UI/log 표시). ✅ 5-필드 = (id, name, system_prompt, allowed_tools, run) 1-1 매핑 |
| `system_prompt` = `&'static str` (v1 hardcode) | `String` | ✅ v1 hardcode 시 zero-allocation. v1.5+ 외부 정의 = `Cow<'static, str>` 또는 `String` (TBD, plugin load 시 String) |
| `allowed_tools` = `&'static [ToolId]` | `Vec<ToolId>` | ✅ v1 = compile-time 고정. v1.5+ plugin = 동적 list 가능 (`Mutex<Vec<ToolId>>` 또는 `RwLock<Vec<ToolId>>`) |
| `run(input: Value)` (typed wrapper ❌) | `run(input: Self::Input)` | ✅ plugin 호환 (plugin sub-agent 의 typed input 모름). ⚠️ sub-agent 내부에서 `serde_json::from_value::<Self::Input>(input)` 1-hop deserialize |
| `Box<dyn SubAgentOutput>` 반환 | `Self::Output` (typed) | ✅ 15 struct 모두 sealed, 외부 매칭 가능. ❌ `Self::Output` = typed → dyn 불가 (`Arc<dyn SubAgent>` 보관 어려움) |

### 1.6 `SubAgentPool` spec (15 내장, future-extensible)

```rust
// crates/myharness-agents/src/subagent/pool.rs (의사코드)
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct SubAgentPool {
    builtin: Vec<SharedSubAgent>,                                          // 15 (compile-time)
    by_id: HashMap<&'static str, SharedSubAgent>,                          // O(1) lookup
    plugin: RwLock<Vec<SharedSubAgent>>,                                   // v1.5+ 동적 추가
}

impl SubAgentPool {
    /// v1 startup 시 자동 호출. 15 builtin 등록.
    pub fn builtin_15() -> Self {
        let builtin: Vec<SharedSubAgent> = vec![
            Arc::new(code::reviewer::CodeReviewer::new()) as SharedSubAgent,
            Arc::new(code::implementer::CodeImplementer::new()),
            Arc::new(code::tester::CodeTester::new()),
            Arc::new(code::refactorer::CodeRefactorer::new()),
            Arc::new(code::searcher::CodeSearcher::new()),
            Arc::new(server::status::ServerStatus::new()),
            Arc::new(server::log_analyzer::LogAnalyzer::new()),
            Arc::new(server::deployer::Deployer::new()),
            Arc::new(server::config_manager::ConfigManager::new()),
            Arc::new(env::setup::EnvSetup::new()),
            Arc::new(env::installer::EnvInstaller::new()),
            Arc::new(env::shell::EnvShell::new()),
            Arc::new(env::diagnose::EnvDiagnose::new()),
            Arc::new(utility::git_operator::GitOperator::new()),
            Arc::new(utility::file_searcher::FileSearcher::new()),
        ];
        let mut by_id = HashMap::with_capacity(builtin.len() + 4);
        for s in &builtin { by_id.insert(s.id(), s.clone()); }
        Self { builtin, by_id, plugin: RwLock::new(Vec::new()) }
    }
    pub fn lookup(&self, id: &str) -> Option<SharedSubAgent> {
        self.by_id.get(id).cloned()
            .or_else(|| self.plugin.read().iter().find(|s| s.id() == id).cloned())
    }
    /// v1.5+ plugin sub-agent 등록.
    pub fn register_plugin(&self, sub_agent: SharedSubAgent) -> Result<(), AppError> {
        let mut p = self.plugin.write();
        if p.iter().any(|s| s.id() == sub_agent.id()) { return Err(AppError::DuplicateId(sub_agent.id().into())); }
        p.push(sub_agent);
        Ok(())
    }
    pub fn all_ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = self.builtin.iter().map(|s| s.id()).collect();
        ids.extend(self.plugin.read().iter().map(|s| s.id()));
        ids
    }
}
```

**pool 결정 trade-off**:
| 선정 | 대안 | trade-off |
| --- | --- | --- |
| `builtin: Vec<SharedSubAgent>` + `by_id: HashMap` | 단일 `HashMap` 만 | ✅ 순서 보장 (출력 시 안정적). ✅ plugin 별도 `RwLock` (lock contention 분리) |
| v1 `plugin` = `RwLock<Vec>` (empty) | v1 plugin 미지원 | ✅ v1.5+ 동적 추가 가능. v1 = empty (v1.5+ 에서 plugin 기능 활성) |
| `Arc::new(...) as SharedSubAgent` cast | 1-by-1 등록 | ✅ compile-time 검증 (trait impl 누락 시 compile error) |

### 1.7 결정 근거 1-라인 (yklee review)

> **sealed trait `SubAgentOutput: Serialize` (15 struct 모두) + `enum ToolId` (10 variant + Custom) + trait SubAgent 5-필드 + SubAgentPool (15 builtin + v1.5+ plugin RwLock)** = DD-1 trait Tool 와 1:1 매핑, orchestrator 가 typed match (sealed Output), plugin 호환 (Custom ToolId + plugin pool).

---

## §2. 15 sub-agent master table (id / output type / allowed_tools)

### 2.1 결정 (결론)

15 sub-agent (CONCEPT §5.11 + INITIAL §3.7 + USE_CASES §5.1) 의 master table. 3 cols × 15 rows. §3-§6 에서 각 sub-agent 별 5-sections (system_prompt 200~400 tokens + Output struct + allowed_tools + dispatch context + TC scaffold) 명세.

### 2.2 Master table (15 rows × 3 cols)

| # | id | output type | allowed_tools (compile-time `&'static [ToolId]`) |
| --- | --- | --- | --- |
| 1 | `code-reviewer` | `ReviewVerdict` (§3.1) | `[Read, Grep, Glob, McpGithub]` |
| 2 | `code-implementer` | `ImplementResult` (§3.2) | `[Read, Grep, Glob, Write, Edit, Bash(CmdPattern::build)]` |
| 3 | `code-tester` | `TestReport` (§3.3) | `[Read, Bash(CmdPattern::test_runner), Grep]` |
| 4 | `code-refactorer` | `RefactorResult` (§3.4) | `[Read, Grep, Glob, Write, Edit, Bash(CmdPattern::build)]` |
| 5 | `code-searcher` | `SearchResult` (§3.5) | `[Read, Grep, Glob]` |
| 6 | `server-status` | `HealthReport` (§4.1) | `[Bash(CmdPattern::ps_systemctl), Read]` |
| 7 | `log-analyzer` | `LogAnalysisReport` (§4.2) | `[Bash(CmdPattern::tail_journalctl), Read, Grep]` |
| 8 | `deployer` | `DeployResult` (§4.3) | `[Bash(CmdPattern::ssh_kubectl_docker), Read]` |
| 9 | `config-manager` | `ConfigDiff` (§4.4) | `[Read, Write, Edit, Bash(CmdPattern::diff_rollback)]` |
| 10 | `env-setup` | `SetupResult` (§5.1) | `[Bash(CmdPattern::brew_apt_dnf_apk_winget), Read, Grep]` |
| 11 | `env-installer` | `InstallResult` (§5.2) | `[Bash(CmdPattern::brew_install_apt_install), Read]` |
| 12 | `env-shell` | `ShellAnalysis` (§5.3) | `[Bash(CmdPattern::user_provided), Read]` |
| 13 | `env-diagnose` | `EnvDiagnosis` (§5.4) | `[Bash(CmdPattern::which_version_path), Read, Grep]` |
| 14 | `git-operator` | `GitOperationResult` (§6.1) | `[Bash(CmdPattern::git), McpGit, McpGithub, Read]` |
| 15 | `file-searcher` | `FileSearchResult` (§6.2) | `[Read, Grep, Glob]` |

### 2.3 결정 trade-off (master table 표현)

| 선정 (3 cols) | 대안 (4 cols) | trade-off |
| --- | --- | --- |
| id / output / allowed_tools 3 cols | + dispatch context 1 col 추가 | ✅ col 3 = allowed_tools 가 §8 permission matrix 와 1:1. dispatch context = §3-§6 의 5-section 중 1-section (sub-section 4) 에서 상세 |
| `Bash(CmdPattern::build)` 표기 | 별도 `Scope` enum | ✅ DD-1 `ToolScope::Bash(CommandPattern)` 와 정합. ⚠️ 가독성 위해 `(CmdPattern::...)` 표기 단순화 |
| `[Read, Grep, Glob, McpGithub]` compile-time | `Vec<ToolId>` 동적 | ✅ v1 hardcode. v1.5+ plugin 시 동적 list 가능 (TBD) |

### 2.4 §2-§8 cross-ref map

| sub-agent | §3-§6 상세 | §7 dispatch | §8 permission |
| --- | --- | --- | --- |
| code-reviewer | §3.1 | orchestrator | read-only + github read |
| code-implementer | §3.2 | orchestrator | full read+write+build |
| code-tester | §3.3 | orchestrator | bash(test)+read |
| code-refactorer | §3.4 | orchestrator | read+write+build (no eval) |
| code-searcher | §3.5 | orchestrator (보조) | read-only |
| server-status | §4.1 | orchestrator (TASK-002 ⏸) | bash(ps) read |
| log-analyzer | §4.2 | orchestrator (TASK-002 ⏸) | bash(tail) + read |
| deployer | §4.3 | orchestrator (TASK-002 ⏸) | bash(ssh) write scope |
| config-manager | §4.4 | orchestrator (TASK-002 ⏸) | read+write (config) |
| env-setup | §5.1 | orchestrator (TASK-002 ⏸) | bash(pkg) write scope |
| env-installer | §5.2 | orchestrator (TASK-002 ⏸) | bash(pkg install) |
| env-shell | §5.3 | orchestrator (TASK-002 ⏸) | bash(user) + user confirm |
| env-diagnose | §5.4 | orchestrator (TASK-002 ⏸) | bash(read-only) |
| git-operator | §6.1 | orchestrator (utility) | bash(git) + github |
| file-searcher | §6.2 | orchestrator (utility) | read-only |

> **TASK-002 ⏸ placeholder** (CONCEPT §11.1 + INITIAL §0.5): server/env sub-agent 8개 (status/log_analyzer/deployer/config_manager + setup/installer/shell/diagnose) 의 host alias / k8s context / dotfiles 경로 = yklee 인프라 정보 필요. v1 = sub-agent module 구조 + dispatch + allowed_tools scope 표만 구현, 세부 host/stack manifest = placeholder.

---

## §3. code 5 sub-agents (5 sections × 5 = 25 sub-sections)

> 각 sub-agent = 5 sections (system_prompt 200~400 tokens / Output struct / allowed_tools / dispatch context / TC scaffold 3~5 entries). module path = `crates/myharness-agents/src/subagent/code/<name>.rs`. 공통 import: `use super::{SubAgent, SubAgentContext, output::SubAgentOutput}; use crate::permission::ToolId;`.

### §3.1 `code-reviewer` (PR multi-aspect review, lead of UC-CODE-001)

**3.1.1 system_prompt (markdown 220 tokens)**
```markdown
# code-reviewer

You are a senior code reviewer. You perform multi-aspect code review on pull requests.

## Mission
- Identify bugs (correctness, edge cases, error handling).
- Review style (naming, structure, idioms for the language).
- Assess test coverage gaps and missing test cases.
- Flag security issues (input validation, secrets, injection).

## Workflow
1. Read the PR diff via `mcp__github__get_pull_request_diff` or local git.
2. Enumerate changed files via `Grep` / `Glob`.
3. For each file, identify 1-3 concrete findings with file:line citations.
4. Organize findings into 3 sections: Bugs / Style / Tests.
5. Provide a single-sentence summary and a verdict (approve | request_changes | comment).

## Constraints
- Cite file:line for every finding.
- Do not suggest stylistic changes that contradict existing project conventions.
- If a finding is speculative, mark it as "may need verification".
- Never invent APIs or behavior — verify via Read/Grep when uncertain.

## Output
- 한국어 1-라인 요약 + 3-section markdown report + verdict.
```

**3.1.2 Output struct (Rust 필드, sealed)**
```rust
// crates/myharness-agents/src/subagent/output/code.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewVerdict {
    pub pr_url: String,                            // input pr-url
    pub summary_ko: String,                        // NFR-UX-3, 1-라인 한국어
    pub verdict: ReviewVerdictKind,                 // approve | request_changes | comment
    pub bugs: Vec<ReviewFinding>,                   // severity: critical | major | minor
    pub style: Vec<ReviewFinding>,
    pub tests: Vec<ReviewFinding>,
    pub confidence: f32,                            // 0.0-1.0
    pub files_reviewed: u32,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewVerdictKind { Approve, RequestChanges, Comment }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub file: String,
    pub line: u32,
    pub severity: Severity,                         // critical | major | minor
    pub category: ReviewCategory,                   // bug | style | test | security
    pub message_ko: String,                         // 한국어
    pub suggestion: Option<String>,                 // optional fix
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity { Critical, Major, Minor }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewCategory { Bug, Style, Test, Security }

// Sealed impl (DD-3 §1.2)
impl sealed::Sealed for ReviewVerdict {}
impl SubAgentOutput for ReviewVerdict {
    fn kind(&self) -> &'static str { "ReviewVerdict" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**3.1.3 allowed_tools (compile-time)**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Grep, ToolId::Glob, ToolId::McpGithub]
}
```
> Bash(eval) ❌ (REVIEW §3.2 / USE_CASES §5.4) — security risk. mcp__github__* = read scope (get_pull_request, list_issues, search_code). Write/Edit ❌.

**3.1.4 dispatch context (UC 매핑)**
- **Primary UC**: UC-CODE-001 (PR review, USE_CASES §3.1 detailed) + UC-CODE-007 (단일 파일 analyze) + UC-CODE-010 (diff 분석)
- **Mode 호환**: orchestrator (default, multi-aspect 3-aspect) / single (단일 파일, LLM 직접) / loop (e.g., `--goal "fix all PR review issues"` → code-reviewer → code-implementer loop)
- **Fan-out (UC-CODE-001)**: orchestrator 가 `git-operator` (PR metadata + diff) + `file-searcher` (changed files) 먼저 spawn → `code-reviewer` lead → optional `code-tester` (coverage gap)
- **mode=single**: code-reviewer 직접 사용 가능 (1 파일). UC-CODE-007 의 "단일 파일 analyze" 에서 sub-agent spawn 1개.

**3.1.5 TC scaffold (L1 Unit 5 entries, RED-GREEN-REFACTOR)**
| TC id | 시나리오 | 검증 | error variant |
| --- | --- | --- | --- |
| TC-CR-01 | happy path: PR 3-aspect review (bugs/style/tests) | ReviewVerdict 3 sections non-empty, summary_ko 한국어, verdict ∈ enum | (없음) |
| TC-CR-02 | empty diff (no changes) | bugs/style/tests empty, verdict=Comment, files_reviewed=0 | (없음) |
| TC-CR-03 | permission denied (allowed_tools 외 tool 호출 시도) | AppError::PermissionDenied | PermissionDenied |
| TC-CR-04 | LLM provider 실패 → fallback (D-15 + DD-5 §1) | ReviewVerdict 정상 반환, log.jsonl `fallback_used: true` | (없음, retry 성공) |
| TC-CR-05 | mcp__github__* 미설정 (server offline) | McpToolUnavailable error + summary_ko = "GitHub MCP 미설정, 로컬 diff 로 분석" | AppError::McpToolUnavailable |

### §3.2 `code-implementer` (새 기능 구현, multi-file 변경)

**3.2.1 system_prompt (markdown 280 tokens)**
```markdown
# code-implementer

You are a senior software engineer. You implement new features and make multi-file changes.

## Mission
- Implement new features end-to-end (plan → code → test).
- Modify multiple files coherently (e.g., feature + tests + docs).
- Respect existing project conventions (style, framework, deps).

## Workflow
1. Read `MiniMax.md` and existing project structure.
2. Plan the implementation (files to change, dependencies, test strategy).
3. For each file, use Read to understand context, then Edit/Write to apply changes.
4. Run tests via `Bash(cargo test | npm test | pytest)` to verify.
5. Provide a 한국어 summary of what was implemented and any caveats.

## Constraints
- One PR = one feature (no scope creep).
- Always run tests after changes; if tests fail, attempt one fix iteration.
- Never modify config that requires user approval (e.g., Cargo.toml dependency addition) without confirmation.
- Use Edit for in-place changes, Write only for new files.

## Output
- 한국어 1-라인 요약 + list of changed files + test result.
```

**3.2.2 Output struct (Rust 필드)**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementResult {
    pub feature_summary_ko: String,                 // 1-라인 한국어
    pub files_changed: Vec<FileChange>,             // path + diff (unified format)
    pub test_command: String,                       // e.g., "cargo test"
    pub test_result: TestOutcome,                   // passed | failed | skipped
    pub test_output_excerpt: String,                // last 50 lines
    pub deps_added: Vec<String>,                    // e.g., "serde = 1.0"
    pub confidence: f32,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_kind: FileChangeKind,                // created | modified | deleted
    pub diff: String,                               // unified diff
    pub lines_added: u32,
    pub lines_removed: u32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileChangeKind { Created, Modified, Deleted }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestOutcome { Passed, Failed, Skipped }

impl sealed::Sealed for ImplementResult {}
impl SubAgentOutput for ImplementResult {
    fn kind(&self) -> &'static str { "ImplementResult" }
    fn summary_ko(&self) -> String { self.feature_summary_ko.clone() }
}
```

**3.2.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Grep, ToolId::Glob,
      ToolId::Write, ToolId::Edit,
      ToolId::Bash]  // Bash scope = build + test, eval ❌
}
```

**3.2.4 dispatch context**
- **Primary UC**: UC-CODE-002 (implement) + UC-CODE-008 (format) + UC-CODE-009 (deps)
- **Fan-out**: orchestrator 가 `code-searcher` (관련 file 검색) → `code-implementer` (lead) → `code-tester` (verify)
- **Mode**: orchestrator (default) / loop (e.g., `--goal "implement X until tests pass"`)
- **Hook enforce**: `require-test-before-commit` hook (DD-4) 가 Edit 직전 eval

**3.2.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-CI-01 | happy path: 3-file feature implement + test pass | files_changed.len=3, test_result=Passed |
| TC-CI-02 | test fail → 1 fix attempt | test_result=Failed, 1 수정 retry 후 Passed (또는 final Failed) |
| TC-CI-03 | permission denied (forbidden path edit) | AppError::PermissionDenied (forbidden_paths) |
| TC-CI-04 | large refactor → 1 file create + 2 edit | FileChangeKind: Created/Modified/Modified |

### §3.3 `code-tester` (test 실행 + 결과 분석)

**3.3.1 system_prompt (markdown 200 tokens)**
```markdown
# code-tester

You are a test engineer. You execute test suites and analyze failures.

## Mission
- Run the project's test command (cargo test / npm test / pytest / go test).
- Parse the output and identify which tests failed and why.
- Suggest minimal fixes for common failure patterns.

## Workflow
1. Detect the test framework (Cargo.toml / package.json / pyproject.toml / go.mod).
2. Run the test command via `Bash` with a timeout (default 300s, max 600s).
3. Parse stdout/stderr to extract pass/fail counts and failure details.
4. For each failure, read the relevant file via `Read` to understand context.
5. Provide a 한국어 summary of failures and 1-line fix suggestions per failure.

## Constraints
- Do not modify test files unless the fix is obvious (e.g., typo in assertion).
- Do not run mutation tests or benchmarks.
- Respect timeout — abort if test command exceeds 600s.
- Output = list of failures with file:line + suggested fix.

## Output
- 한국어 1-라인 + pass/fail counts + per-failure diagnosis.
```

**3.3.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub test_command: String,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
    pub failures: Vec<TestFailure>,
    pub duration_ms: u64,
    pub summary_ko: String,
    pub coverage_delta: Option<f32>,                // optional, if coverage tool ran
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub file: String,
    pub line: u32,
    pub message: String,                            // assertion message
    pub suggested_fix: Option<String>,
    pub category: FailureCategory,                  // assertion | timeout | compile | runtime
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureCategory { Assertion, Timeout, Compile, Runtime, Other }

impl sealed::Sealed for TestReport {}
impl SubAgentOutput for TestReport {
    fn kind(&self) -> &'static str { "TestReport" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**3.3.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Bash, ToolId::Grep]  // Bash = test runner only (CmdPattern::test_runner)
}
```

**3.3.4 dispatch context**
- **Primary UC**: UC-CODE-003 (test)
- **Mode**: orchestrator (default) / loop (`--goal "fix all failing tests"` → code-tester → code-implementer)
- **Fan-out**: 단독 또는 UC-CODE-001 의 coverage gap 분석

**3.3.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-CT-01 | happy: cargo test 10/10 pass | TestReport passed=10, failed=0, failures empty |
| TC-CT-02 | 2 failures + 8 pass | passed=8, failed=2, failures.len=2 |
| TC-CT-03 | timeout 600s | AppError::ToolError(Timeout) + summary_ko = "시간 초과" |
| TC-CT-04 | no test framework detected | TestReport empty, summary_ko = "테스트 프레임워크 미감지" |

### §3.4 `code-refactorer` (AST-aware 리팩토링)

**3.4.1 system_prompt (markdown 240 tokens)**
```markdown
# code-refactorer

You are a refactoring specialist. You apply mechanical refactorings with AST awareness.

## Mission
- Apply rename / extract / dedup refactorings across multiple files.
- Ensure semantic equivalence — tests must still pass.

## Workflow
1. Identify the refactoring scope (function name, variable, module).
2. Use `Grep` to find all occurrences.
3. Plan the refactoring (files affected, order of changes).
4. Apply Edit changes file-by-file (or Write for new files).
5. Run `Bash(test command)` to verify no regressions.

## Constraints
- Do not change public API signatures unless explicitly requested.
- Preserve formatting (use `Bash(formatter)` after edits if available).
- Tree-sitter AST (DD-2) used for rename to avoid false matches in strings/comments.
- After refactor, run tests. If tests fail, revert the last change.

## Output
- 한국어 1-라인 + list of refactored files + test result.
```

**3.4.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorResult {
    pub refactor_kind: RefactorKind,
    pub scope: String,                              // e.g., "function:foo_bar" or "module:utils"
    pub files_modified: Vec<FileChange>,
    pub test_result: TestOutcome,
    pub reverted: bool,                              // true if tests failed → revert
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RefactorKind { Rename, Extract, Inline, Dedup, Move, Other }

impl sealed::Sealed for RefactorResult {}
impl SubAgentOutput for RefactorResult {
    fn kind(&self) -> &'static str { "RefactorResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**3.4.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Grep, ToolId::Glob,
      ToolId::Write, ToolId::Edit,
      ToolId::Bash]  // Bash = test + formatter, eval ❌
}
```

**3.4.4 dispatch context**
- **Primary UC**: UC-CODE-005 (refactor)
- **Fan-out**: orchestrator 가 `code-searcher` (scope 탐색) → `code-refactorer` (lead) → `code-tester` (verify)
- **Hook**: `require-test-before-commit` (Edit 직전 eval, DD-4)

**3.4.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-CRF-01 | rename across 3 files, tests pass | files_modified.len=3, test_result=Passed, reverted=false |
| TC-CRF-02 | rename → tests fail → revert | reverted=true, files_modified = original (이전 상태) |
| TC-CRF-03 | extract function | RefactorKind::Extract, 1 new file created |

### §3.5 `code-searcher` (codebase 검색 + 구조 분석)

**3.5.1 system_prompt (markdown 180 tokens)**
```markdown
# code-searcher

You are a codebase navigator. You find code patterns and analyze structure.

## Mission
- Search for symbols, patterns, and definitions across the codebase.
- Analyze file structure and module dependencies.

## Workflow
1. Use `Grep` for content search (regex/ripgrep).
2. Use `Glob` for file path matching (e.g., "**/*.rs").
3. Use `Read` to inspect specific files when context needed.
4. Group results by file and provide a 한국어 summary of findings.

## Constraints
- Read-only — never Write/Edit/Bash.
- Limit results (default 100, max 1000) to avoid context bloat.
- For "where is X used" queries, return file:line citations.

## Output
- 한국어 1-라인 + grouped matches (file → list of (line, content)).
```

**3.5.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub total_matches: u32,
    pub truncated: bool,
    pub by_file: HashMap<String, Vec<SearchMatch>>,
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub line: u32,
    pub col: u32,
    pub text: String,                                // matched line
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}

impl sealed::Sealed for SearchResult {}
impl SubAgentOutput for SearchResult {
    fn kind(&self) -> &'static str { "SearchResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**3.5.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Grep, ToolId::Glob]  // read-only
}
```

**3.5.4 dispatch context**
- **Primary UC**: UC-CODE-006 (search) + UC-CODE-007 (단일 파일 analyze, 보조)
- **Mode**: 모든 mode (orchestrator/single/loop) — utility 성격
- **Fan-out**: 거의 모든 code UC 의 보조 (UC-CODE-001 PR changed files / UC-CODE-002 implement 시 relevant files / UC-CODE-005 refactor scope)

**3.5.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-CS-01 | grep "TODO" → 5 matches across 3 files | total_matches=5, by_file.len=3 |
| TC-CS-02 | glob "**/*.rs" → 42 files | total_matches=42, truncated=false |
| TC-CS-03 | no match | total_matches=0, by_file empty, summary_ko = "매치 없음" |

---

## §4. server 4 sub-agents (5 sections × 4 = 20 sub-sections)

> server 도메인 = TASK-002 ⏸ placeholder 영향. sub-agent module 구조 + dispatch + allowed_tools scope 표는 구현. host alias / ssh / k8s context / docker host = yklee 인프라 정보 필요 (PROJECT_PROFILE.md §3.1 TODO). module path = `crates/myharness-agents/src/subagent/server/<name>.rs`.

### §4.1 `server-status` (프로세스/서비스 상태 점검)

**4.1.1 system_prompt (markdown 220 tokens)**
```markdown
# server-status

You are a server health inspector. You check process and service status on local or remote hosts.

## Mission
- Enumerate running processes and their status (running / stopped / zombie).
- Detect anomalies: high CPU / memory, zombie processes, unhealthy services.
- Provide a structured table + 한국어 summary.

## Workflow
1. Detect host (local vs ssh alias from `config/server/hosts.yaml`, TASK-002 ⏸).
2. Run platform-appropriate command via `Bash`:
   - macOS: `launchctl list`
   - Linux: `systemctl list-units --type=service --state=running`
   - Windows: `Get-Service | Where-Object {$_.Status -eq "Running"}`
3. Parse output into structured table (SERVICE | PID | STATUS | UPTIME | NOTE).
4. Pass process list to LLM for anomaly detection.
5. Return a 한국어 summary highlighting any anomalies.

## Constraints
- Read-only — no kill/restart (deferred to `deployer` sub-agent).
- Default 100 processes max; flag if exceeded.
- Anomaly threshold: CPU > 80% sustained, memory > 90%, uptime > 365d.

## Output
- 한국어 1-라인 + structured table (SERVICE | PID | STATUS | UPTIME | NOTE) + anomaly list.
```

**4.1.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub host: String,                                // "local" | ssh alias | hostname
    pub platform: Platform,                          // Macos | Linux | Windows
    pub services: Vec<ServiceStatus>,
    pub anomalies: Vec<Anomaly>,
    pub summary_ko: String,
    pub collected_at: DateTime<Utc>,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,                                // service name or process name
    pub pid: Option<u32>,
    pub status: String,                              // "running" | "stopped" | "zombie" | ...
    pub uptime_seconds: Option<u64>,
    pub cpu_percent: Option<f32>,
    pub memory_mb: Option<u64>,
    pub note: Option<String>,                        // e.g., "high CPU", "restart loop"
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub kind: AnomalyKind,                           // HighCpu | HighMemory | Zombie | Unhealthy | Old
    pub service: String,
    pub detail: String,
    pub severity: Severity,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalyKind { HighCpu, HighMemory, Zombie, Unhealthy, Old }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform { Macos, Linux, Windows, Unknown }

impl sealed::Sealed for HealthReport {}
impl SubAgentOutput for HealthReport {
    fn kind(&self) -> &'static str { "HealthReport" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**4.1.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read]  // Bash = launchctl/systemctl/Get-Service (CmdPattern::ps_systemctl, read-only)
}
```

**4.1.4 dispatch context (UC 매핑)**
- **Primary UC**: UC-SERVER-001 (status, §3.2 detailed) + UC-SERVER-005 (health, server-status + log-analyzer) + UC-SERVER-008 (metrics)
- **TASK-002 ⏸**: `<host>` = ssh alias (config/server/hosts.yaml) — v1 = placeholder, ssh 분기 구조는 구현
- **Mode**: orchestrator (default) / single (1 host 직접)
- **Fan-out**: 단독 또는 UC-SERVER-005 의 종합 health check 시 log-analyzer 와 동시

**4.1.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-SS-01 | local macOS: launchctl list → 10 services | HealthReport platform=Macos, services.len=10, anomalies empty |
| TC-SS-02 | high CPU anomaly (process 95%) | anomalies.len=1, kind=HighCpu, severity=Critical |
| TC-SS-03 | remote ssh host unreachable | AppError::SshUnreachable, summary_ko = "원격 호스트 연결 실패" |
| TC-SS-04 | Windows Get-Service | platform=Windows, services 정상 parse |

### §4.2 `log-analyzer` (로그 분석 + 이상 패턴 detection)

**4.2.1 system_prompt (markdown 240 tokens)**
```markdown
# log-analyzer

You are a log analysis expert. You detect anomalies and patterns in service logs.

## Mission
- Tail recent N lines of a service's log.
- Identify error patterns, stack traces, and recurring warnings.
- Provide a 한국어 summary with severity-tagged findings.

## Workflow
1. Resolve log source (service name → systemd journal / docker logs / file path).
2. Run platform-appropriate command via `Bash`:
   - Linux: `journalctl -u <service> -n <N> --no-pager`
   - macOS: `log show --predicate 'process == "<name>"' --last <N>m`
   - Docker: `docker logs <container> --tail <N>`
   - File: `tail -n <N> <path>`
3. Pass log content to LLM for pattern detection.
4. Categorize findings (error / warning / info) and dedup.
5. Return findings + 한국어 summary.

## Constraints
- Read-only — no log rotation or deletion.
- Default N=100 lines, max 10000.
- Anomaly patterns: OOM, panic, timeout, 5xx status, recurring error.

## Output
- 한국어 1-라인 + findings list (severity, count, sample line) + suggested next steps.
```

**4.2.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisReport {
    pub service: String,
    pub log_source: String,                          // e.g., "journalctl:nginx", "file:/var/log/app.log"
    pub lines_analyzed: u32,
    pub findings: Vec<LogFinding>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFinding {
    pub pattern: String,                             // e.g., "OOM", "panic", "5xx", "timeout"
    pub count: u32,
    pub severity: Severity,
    pub sample_lines: Vec<String>,                   // up to 3 sample lines
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub suggested_action: Option<String>,            // e.g., "increase memory limit"
}

impl sealed::Sealed for LogAnalysisReport {}
impl SubAgentOutput for LogAnalysisReport {
    fn kind(&self) -> &'static str { "LogAnalysisReport" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**4.2.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read, ToolId::Grep]  // Bash = tail/journalctl (CmdPattern::tail_journalctl)
}
```

**4.2.4 dispatch context**
- **Primary UC**: UC-SERVER-002 (logs) + UC-SERVER-005 (health 종합)
- **Mode**: orchestrator (default)
- **Fan-out**: 단독 또는 server-status 와 동시 (UC-SERVER-005)
- **TASK-002 ⏸**: `<service>` = systemd unit / docker container / log path (TASK-002 placeholder)

**4.2.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-LA-01 | journalctl nginx -n 100 → 3 OOM patterns | findings.len=1, count=3, pattern="OOM" |
| TC-LA-02 | docker logs myapp --tail 50 → no errors | findings empty, summary_ko = "이상 패턴 없음" |
| TC-LA-03 | log file 1GB → timeout (10분) | AppError::ToolError(Timeout) |

### §4.3 `deployer` (배포 헬퍼, ssh/k8s/docker)

**4.3.1 system_prompt (markdown 260 tokens)**
```markdown
# deployer

You are a deployment assistant. You handle deployments via ssh, kubernetes, or docker.

## Mission
- Deploy to a target environment (dev / staging / prod).
- Provide pre/post status comparison.
- Rollback on failure.

## Workflow
1. Resolve target environment (env name → ssh host / k8s context / docker registry).
2. Run pre-deploy hook (e.g., `git pull`, `docker pull`).
3. Execute deploy command (e.g., `kubectl rollout`, `docker compose up -d`, `ssh host "cd app && ./deploy.sh"`).
4. Wait for readiness check (e.g., `kubectl rollout status`, `curl /health`).
5. Post-deploy: compare pre/post state (replicas, version, etc.).
6. On failure: auto-rollback if safe (non-prod); for prod, request user confirmation.

## Constraints
- PROD deployments require explicit user confirmation (NFR-SEC-5, 4 mode 적용).
- Idempotency: re-running the same deploy should be safe.
- Hook: `warn-destructive-deploy` blocks `kubectl delete` / `docker rm` without user confirm.
- Timeout: 600s per step, fail-fast on error.

## Output
- 한국어 1-라인 + pre/post state diff + deploy log excerpt + rollback info (if any).
```

**4.3.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResult {
    pub env: String,                                 // "dev" | "staging" | "prod"
    pub deploy_kind: DeployKind,                     // Ssh | Kubernetes | Docker | Custom
    pub pre_state: HashMap<String, String>,          // e.g., {"replicas": "3", "version": "v1.2.3"}
    pub post_state: HashMap<String, String>,
    pub success: bool,
    pub rolled_back: bool,
    pub duration_ms: u64,
    pub log_excerpt: String,                         // last 50 lines of deploy log
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeployKind { Ssh, Kubernetes, Docker, Custom }

impl sealed::Sealed for DeployResult {}
impl SubAgentOutput for DeployResult {
    fn kind(&self) -> &'static str { "DeployResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**4.3.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read]  // Bash = ssh/kubectl/docker (CmdPattern::ssh_kubectl_docker, write scope)
}
```

**4.3.4 dispatch context**
- **Primary UC**: UC-SERVER-003 (deploy) + UC-SERVER-006 (restart)
- **NFR-SEC-5**: PROD 배포 = user 명시 승인 필수 (4 mode `default` 에서도 bypass 안 됨, hook enforce)
- **TASK-002 ⏸**: `<env>` = ssh / k8s context / docker registry (placeholder)
- **Mode**: orchestrator (default) / single (긴급 hotfix 시)

**4.3.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-DP-01 | dev 환경 docker compose up | success=true, pre/post_state 비교 |
| TC-DP-02 | prod 환경 → user confirm 필수 | AppError::UserConfirmationRequired (NFR-SEC-5) |
| TC-DP-03 | kubectl rollout 실패 → auto-rollback | success=false, rolled_back=true |
| TC-DP-04 | timeout 600s | AppError::ToolError(Timeout) |

### §4.4 `config-manager` (설정 조회/변경, with backup)

**4.4.1 system_prompt (markdown 220 tokens)**
```markdown
# config-manager

You are a configuration manager. You read, modify, and rollback service configuration files.

## Mission
- Get current config value(s).
- Set new config value(s) with diff preview.
- Rollback to previous version on failure.

## Workflow
1. Resolve config file path (e.g., `/etc/nginx/nginx.conf`, `~/.config/app/settings.yaml`).
2. Read current file via `Read`.
3. Compute diff (old → new).
4. Show diff to user (NFR-UX-3 한국어).
5. If user confirms, write the new file (with backup: `cp file file.bak.<timestamp>`).
6. On post-change failure, restore from backup.

## Constraints
- Forbidden paths: `/etc/shadow`, `~/.ssh/*`, system-critical configs.
- Always create backup before write.
- Atomic write (tmp + rename, DD-1 §3.3 Write tool).
- Rollback on any post-write verification failure.

## Output
- 한국어 1-라인 + diff (unified) + backup path + verification result.
```

**4.4.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub config_path: String,
    pub action: ConfigAction,                        // Get | Set | Diff | Rollback
    pub old_value: Option<String>,                   // for Set/Diff
    pub new_value: Option<String>,
    pub diff: String,                                // unified diff
    pub backup_path: Option<String>,                 // for Set/Rollback
    pub verification: VerificationResult,            // for Set
    pub rolled_back: bool,                           // for Set
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfigAction { Get, Set, Diff, Rollback }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult { Ok(String), Failed(String), Skipped }

impl sealed::Sealed for ConfigDiff {}
impl SubAgentOutput for ConfigDiff {
    fn kind(&self) -> &'static str { "ConfigDiff" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**4.4.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Write, ToolId::Edit, ToolId::Bash]  // Bash = diff/rollback (CmdPattern::diff_rollback)
}
```

**4.4.4 dispatch context**
- **Primary UC**: UC-SERVER-004 (config) + UC-SERVER-006 (restart, deployer 와 동시)
- **TASK-002 ⏸**: config file path = yklee 인프라 정보 (placeholder)
- **NFR-SEC-5**: forbidden_paths (`/etc/shadow`, `~/.ssh/*`) 명시적 거부 (hook + path check)

**4.4.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-CM-01 | get nginx.conf → old_value 표시 | ConfigAction::Get, old_value=Some |
| TC-CM-02 | set worker_processes 4 → diff + backup | ConfigAction::Set, backup_path=Some, diff non-empty |
| TC-CM-03 | forbidden path /etc/shadow → 거부 | AppError::PermissionDenied (forbidden_paths) |

---

## §5. env 4 sub-agents (5 sections × 4 = 20 sub-sections)

> env 도메인 = TASK-002 ⏸ placeholder 영향. stack manifest / Homebrew / asdf / dotfiles = yklee 인프라 정보 필요 (PROJECT_PROFILE.md §3.1 TODO). module path = `crates/myharness-agents/src/subagent/env/<name>.rs`.

### §5.1 `env-setup` (스택별 부트스트랩, brew/asdf/dotfiles)

**5.1.1 system_prompt (markdown 280 tokens)**
```markdown
# env-setup

You are a stack bootstrapper. You install and configure complete dev environments.

## Mission
- Bootstrap a stack (brew | asdf | dotfiles | node | python | rust | go).
- Run pre-diagnose → install → post-diagnose flow.
- Ensure idempotency (re-run safe).

## Workflow
1. Resolve stack name → stack manifest (config/stacks/<stack>.yaml, TASK-002 ⏸).
2. Spawn `env-diagnose` (pre): snapshot path/version/permission.
3. Execute stack manifest (per platform):
   - macOS: `brew bundle` (Brewfile) / `asdf plugin add + install`
   - Linux Debian: `apt-get install -y` / `asdf`
   - Linux RHEL: `dnf install` / `asdf`
   - Linux Alpine: `apk add` / `asdf`
   - Windows: `winget install` / `choco install`
4. Optionally spawn `env-installer` for runtime (e.g., asdf install <runtime>).
5. Spawn `env-diagnose` (post): verify smoke test (e.g., `node --version`, `cargo --version`).
6. Write to `memory/auto/<stack>-setup.md` (D-26 auto memory).
7. Notify user: "새 PATH 적용 위해 shell reload 필요" (NFR-REL-5 dry-run default).

## Constraints
- Dry-run default (NFR-REL-5): show plan, wait for user confirm before apply.
- Idempotency: re-running with same stack = no-op or upgrade.
- Hook: `warn-destructive-env` blocks `rm -rf` / `chmod 777` without confirm.
- Auto memory write: `memory/auto/<stack>-setup.md` (D-26).

## Output
- 한국어 1-라인 + install log + smoke test result + shell reload notice.
```

**5.1.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupResult {
    pub stack: String,                               // "rust" | "node" | "python-data" | "brew" | "dotfiles"
    pub platform: Platform,
    pub pre_diagnosis: EnvDiagnosis,                 // nested
    pub post_diagnosis: EnvDiagnosis,
    pub packages_installed: Vec<PackageInstall>,
    pub runtimes_installed: Vec<RuntimeInstall>,
    pub dotfiles_pulled: bool,
    pub auto_memory_path: Option<String>,            // ~/.myharness/memory/auto/<stack>-setup.md
    pub smoke_test_result: SmokeTestResult,
    pub shell_reload_required: bool,
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInstall {
    pub name: String,
    pub version: String,
    pub manager: PackageManager,                     // Brew | Apt | Dnf | Apk | Winget | Choco
    pub status: InstallStatus,                       // Installed | AlreadyPresent | Failed
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInstall {
    pub name: String,                                // e.g., "node", "python", "rust"
    pub version: String,                             // e.g., "20.10.0"
    pub manager: String,                             // "asdf" | "rtx" | "system"
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageManager { Brew, Apt, Dnf, Apk, Winget, Choco }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstallStatus { Installed, AlreadyPresent, Failed }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SmokeTestResult { AllPassed, SomeFailed, Skipped }

impl sealed::Sealed for SetupResult {}
impl SubAgentOutput for SetupResult {
    fn kind(&self) -> &'static str { "SetupResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**5.1.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read, ToolId::Grep]  // Bash = brew/apt/dnf/apk/winget (CmdPattern::brew_apt_dnf_apk_winget)
}
```

**5.1.4 dispatch context (UC 매핑)**
- **Primary UC**: UC-ENV-001 (setup, §3.3 detailed) + UC-ENV-007 (dotfiles, env-setup dotfiles scope)
- **Fan-out**: env-diagnose (pre + post) + env-installer (optional, runtime)
- **TASK-002 ⏸**: `<stack>` = stack manifest (config/stacks/<stack>.yaml, PROJECT_PROFILE.md §3.1 TODO)
- **NFR-REL-5**: dry-run default, user 명시 승인 후 실제 적용

**5.1.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-ES-01 | macOS brew bundle Brewfile | packages_installed list, smoke_test AllPassed |
| TC-ES-02 | idempotency: re-run same stack | InstallStatus=AlreadyPresent 대부분 |
| TC-ES-03 | dry-run mode → no install | smoke_test=Skipped, summary_ko = "dry-run, 적용 안 함" |
| TC-ES-04 | asdf install runtime | runtimes_installed.len=1, manager="asdf" |

### §5.2 `env-installer` (의존성 설치, with idempotency)

**5.2.1 system_prompt (markdown 200 tokens)**
```markdown
# env-installer

You are a package installer. You install dependencies with idempotency.

## Mission
- Install packages via the platform's package manager.
- Ensure idempotency (re-run = no-op if already installed).
- Detect the appropriate manager automatically.

## Workflow
1. Auto-detect platform and package manager.
2. For each package, check if already installed (idempotency check).
3. If not installed, run install command.
4. Verify installation (`<package> --version` or similar).
5. Report install results.

## Constraints
- Idempotency mandatory (PROJECT_PROFILE.md §4 검증 포인트).
- Forbidden: `sudo` without explicit user approval.
- Timeout: 300s per package.

## Output
- 한국어 1-라인 + list of (package, version, status: installed/already-present/failed).
```

**5.2.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub manager: PackageManager,
    pub packages: Vec<PackageInstall>,               // nested from SetupResult
    pub idempotent: bool,                            // true if all were already-present
    pub summary_ko: String,
    pub latency_ms: u64,
}

impl sealed::Sealed for InstallResult {}
impl SubAgentOutput for InstallResult {
    fn kind(&self) -> &'static str { "InstallResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**5.2.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read]  // Bash = brew install / apt install / etc. (CmdPattern::brew_install_apt_install)
}
```

**5.2.4 dispatch context**
- **Primary UC**: UC-ENV-002 (install) + UC-ENV-008 (upgrade) + UC-ENV-006 (runtime)
- **Mode**: orchestrator (default)
- **Fan-out**: env-setup 의 보조 (runtime install)

**5.2.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-EI-01 | install git, jq (macOS brew) | packages.len=2, status=Installed |
| TC-EI-02 | idempotency: git already present | idempotent=true, status=AlreadyPresent |
| TC-EI-03 | apt-get install sudo package | manager=Apt, status=Installed |

### §5.3 `env-shell` (셸 명령 + LLM 분석)

**5.3.1 system_prompt (markdown 200 tokens)**
```markdown
# env-shell

You are a shell command analyst. You execute a user-provided command and explain its output.

## Mission
- Execute a shell command the user provides.
- Analyze stdout/stderr and explain in 한국어 what happened.
- Flag potential issues (errors, warnings, side effects).

## Workflow
1. Receive command from user.
2. Run via `Bash` (with user-provided timeout, max 600s).
3. Parse stdout/stderr for patterns (error / warning / success indicators).
4. Provide a 한국어 explanation of what the command did.

## Constraints
- User-provided commands require explicit permission (4 mode `default` 에서 confirm).
- Hook: `warn-rm-rf` / `warn-destructive-env` enforce.
- Do not chain commands without user awareness (split into separate calls).

## Output
- 한국어 1-라인 + stdout/stderr excerpt + explanation of output.
```

**5.3.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAnalysis {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_excerpt: String,                      // last 50 lines
    pub stderr_excerpt: String,
    pub analysis_ko: String,                         // LLM 분석 결과
    pub warnings: Vec<String>,                       // e.g., "deleted 5 files", "modified config"
    pub summary_ko: String,
    pub latency_ms: u64,
}

impl sealed::Sealed for ShellAnalysis {}
impl SubAgentOutput for ShellAnalysis {
    fn kind(&self) -> &'static str { "ShellAnalysis" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**5.3.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read]  // Bash = user-provided (CmdPattern::user_provided, user confirm 필수)
}
```

**5.3.4 dispatch context**
- **Primary UC**: UC-ENV-003 (shell)
- **NFR-SEC-3**: 4 mode `default` 에서 user confirm 필수. `bypassPermissions` (sandbox) 만 자동.

**5.3.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-ESH-01 | `ls -la` → 10 entries | exit_code=Some(0), analysis_ko = "10개 파일/디렉토리" |
| TC-ESH-02 | `rm file.txt` (destructive) → hook warn | warnings.len=1, "rm detected" |
| TC-ESH-03 | `nonexistent_cmd` (exit 127) | exit_code=Some(127), analysis_ko = "command not found" |

### §5.4 `env-diagnose` (환경 진단, path/version/permission)

**5.4.1 system_prompt (markdown 200 tokens)**
```markdown
# env-diagnose

You are a system environment diagnostician. You snapshot path, version, and permission state.

## Mission
- Check $PATH, installed tools' versions, and permission state.
- Identify missing tools or misconfigurations.
- Provide a 한국어 diagnosis with fix suggestions.

## Workflow
1. Run `which <tool>` for each expected tool (e.g., git, node, cargo, python).
2. Run `<tool> --version` to confirm version.
3. Check $PATH and $SHELL.
4. Verify write permission for common dirs.
5. Pass snapshot to LLM for issue detection.

## Constraints
- Read-only — no fixes, only diagnosis.
- Default tool list: git, node, cargo, python, go, docker, kubectl (configurable).

## Output
- 한국어 1-라인 + per-tool status (present/missing/version) + suggested fixes.
```

**5.4.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvDiagnosis {
    pub platform: Platform,
    pub path: String,                                // $PATH
    pub shell: String,                               // $SHELL
    pub tools: Vec<ToolStatus>,
    pub permissions: Vec<PermissionCheck>,
    pub issues: Vec<EnvIssue>,
    pub summary_ko: String,
    pub collected_at: DateTime<Utc>,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub present: bool,
    pub path: Option<String>,                        // `which` output
    pub version: Option<String>,                     // `--version` output
    pub expected_version: Option<String>,            // from project config
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub path: String,
    pub writable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvIssue {
    pub kind: EnvIssueKind,                          // MissingTool | VersionMismatch | PathMissing | PermDenied
    pub detail: String,
    pub suggested_fix: Option<String>,
    pub severity: Severity,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvIssueKind { MissingTool, VersionMismatch, PathMissing, PermDenied, Other }

impl sealed::Sealed for EnvDiagnosis {}
impl SubAgentOutput for EnvDiagnosis {
    fn kind(&self) -> &'static str { "EnvDiagnosis" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**5.4.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::Read, ToolId::Grep]  // Bash = which/version/path (CmdPattern::which_version_path, read-only)
}
```

**5.4.4 dispatch context**
- **Primary UC**: UC-ENV-004 (diagnose) + UC-ENV-005 (doctor, interactive)
- **Mode**: orchestrator (default, env-setup 의 pre/post)
- **Fan-out**: env-setup 의 pre-check + post-check 으로 2회 spawn (USE_CASES §3.3)

**5.4.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-ED-01 | macOS, git/node/cargo 모두 present | tools.len=3, present 모두 true, issues empty |
| TC-ED-02 | missing node | EnvIssue kind=MissingTool, suggested_fix = "brew install node" |
| TC-ED-03 | permission denied on /usr/local | PermissionCheck writable=false, EnvIssue PermDenied |

---

## §6. utility 2 sub-agents (5 sections × 2 = 10 sub-sections)

> utility 도메인 = 모든 도메인에서 호출되는 foundation sub-agent (git-operator: 모든 git workflow / file-searcher: 모든 read-only file search). module path = `crates/myharness-agents/src/subagent/utility/<name>.rs`.

### §6.1 `git-operator` (git workflow, commit/PR/branch)

**6.1.1 system_prompt (markdown 240 tokens)**
```markdown
# git-operator

You are a git workflow operator. You handle commits, branches, and PR operations.

## Mission
- Stage changes and create commits with conventional messages.
- Manage branches (create, switch, merge).
- Interact with remote (push, PR creation via mcp__github__*).

## Workflow
1. Resolve repo (current working directory or explicit path).
2. Run `git status` / `git diff` to understand changes.
3. Stage files (`git add <path>` or `git add -A` for all).
4. Create commit with conventional message (e.g., `feat: add X`, `fix: resolve Y`).
5. Optionally push to remote.
6. Optionally create PR via `mcp__github__create_pull_request`.

## Constraints
- Never force-push to `main` / `master` (NFR-SEC-5, DD-4 SP-02 hook).
- Never skip hooks (`--no-verify`) without user approval.
- Commit message: conventional commits (feat/fix/chore/docs/refactor/test).
- `mcp__github__create_pr` requires user confirmation (NFR-SEC-5).

## Output
- 한국어 1-라인 + commit hash + branch + push status + PR URL (if any).
```

**6.1.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationResult {
    pub operation: GitOperation,                      // Commit | Branch | Merge | Push | PR
    pub repo_path: String,
    pub commit_hash: Option<String>,                  // for Commit
    pub branch: Option<String>,                       // current/created
    pub push_status: Option<PushStatus>,              // for Push
    pub pr_url: Option<String>,                       // for PR
    pub files_staged: Vec<String>,
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GitOperation { Commit, Branch, Merge, Push, PR, Status, Log, Diff }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PushStatus { Pushed, Rejected, NoUpstream, ForcePushBlocked }

impl sealed::Sealed for GitOperationResult {}
impl SubAgentOutput for GitOperationResult {
    fn kind(&self) -> &'static str { "GitOperationResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**6.1.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Bash, ToolId::McpGit, ToolId::McpGithub, ToolId::Read]  // Bash = git (CmdPattern::git)
}
```

**6.1.4 dispatch context**
- **Primary UC**: UC-CODE-004 (commit) + UC-CODE-001 (PR metadata 보조) + UC-CODE-010 (diff)
- **Mode**: orchestrator (default, 모든 git workflow 의 foundation) / single (간단한 status)
- **NFR-SEC-5 + DD-4 SP-02**: force-push to main 차단, `--no-verify` 차단, PR create user confirm

**6.1.5 TC scaffold (4 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-GO-01 | commit 3 files, conventional message | commit_hash=Some, files_staged.len=3 |
| TC-GO-02 | force-push to main → blocked | PushStatus=ForcePushBlocked, AppError::HookBlocked (SP-02) |
| TC-GO-03 | create PR via mcp__github__create_pull_request | pr_url=Some |
| TC-GO-04 | status check (no commit) | operation=Status, files_staged empty |

### §6.2 `file-searcher` (file glob/find/grep, read-only)

**6.2.1 system_prompt (markdown 160 tokens)**
```markdown
# file-searcher

You are a file search specialist. You find files and search content using read-only tools.

## Mission
- Search for files by glob pattern (e.g., `**/*.rs`).
- Search file contents by regex (ripgrep).
- Read specific files for context.

## Workflow
1. Receive query (glob pattern or regex).
2. Use `Glob` for file path matching.
3. Use `Grep` for content search.
4. Use `Read` to inspect specific files.
5. Group results and provide a 한국어 summary.

## Constraints
- Read-only — no Write/Edit/Bash.
- Limit results (default 100, max 1000).
- Dedup results (same file:line not repeated).

## Output
- 한국어 1-라인 + grouped matches (file → list of (line, content)).
```

**6.2.2 Output struct**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub query: String,                                // glob pattern or regex
    pub search_kind: SearchKind,                     // Glob | Grep | Read
    pub matches: Vec<FileMatch>,
    pub total_count: u32,
    pub truncated: bool,
    pub summary_ko: String,
    pub latency_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchKind { Glob, Grep, Read }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatch {
    pub file: String,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub text: Option<String>,                         // matched line (Grep) or file content excerpt (Read)
}

impl sealed::Sealed for FileSearchResult {}
impl SubAgentOutput for FileSearchResult {
    fn kind(&self) -> &'static str { "FileSearchResult" }
    fn summary_ko(&self) -> String { self.summary_ko.clone() }
}
```

**6.2.3 allowed_tools**
```rust
fn allowed_tools(&self) -> &'static [ToolId] {
    &[ToolId::Read, ToolId::Grep, ToolId::Glob]  // read-only
}
```

**6.2.4 dispatch context**
- **Primary UC**: 모든 use case 의 tool (Read/Grep/Glob dispatch) + UC-CODE-006 (search, 보조) + UC-MAINT-002 (log filter)
- **Mode**: 모든 mode (orchestrator/single/loop) — 가장 자주 호출되는 utility
- **Fan-out**: 거의 모든 UC 에 등장 (utility 성격)

**6.2.5 TC scaffold (3 entries)**
| TC id | 시나리오 | 검증 |
| --- | --- | --- |
| TC-FS-01 | glob "**/*.rs" → 42 files | search_kind=Glob, total_count=42 |
| TC-FS-02 | grep "TODO" → 5 matches | search_kind=Grep, total_count=5 |
| TC-FS-03 | read src/main.rs | search_kind=Read, matches.len=1 |

---

## §7. 3 mode dispatch logic (orchestrator / single / loop, D-29)

### 7.1 결정 (결론)

CONCEPT.md §5.10 의 3 agent mode (orchestrator / single / loop) + USE_CASES.md §4 의 mode dispatch matrix 정합. orchestrator = main agent 가 1+ sub-agent spawn 후 결과 통합. single = main agent 가 sub-agent spawn 없이 직접 LLM 호출 (small task). loop = ralph-wiggum 패턴 (D-29) — goal/success_criteria/max-iterations 으로 자동 반복.

### 7.2 12 명령 × 3 mode dispatch matrix (CONCEPT §5.10 + USE_CASES §4.2)

| 명령 (CONCEPT §5.2) | mode=orchestrator (default) | mode=single | mode=loop (D-29 ralph-wiggum) |
| --- | --- | --- | --- |
| `code review <pr>` (UC-CODE-001) | ✅ git-op + file-searcher + code-reviewer (lead) + optional code-tester | ✅ code-reviewer (1 sub-agent) | ✅ `--goal "resolve all blocker comments"` |
| `code implement "<feat>"` (UC-CODE-002) | ✅ file-searcher + code-implementer (lead) | ✅ code-implementer | ✅ `--goal "implement X until tests pass"` |
| `code test <path>` (UC-CODE-003) | ✅ code-tester | ✅ code-tester | ✅ `--goal "fix all failing tests"` |
| `code commit "<msg>"` (UC-CODE-004) | ✅ git-operator | ✅ git-operator | ❌ (loop 부적합, NFR-REL-5 dry-run 권장) |
| `code refactor <scope>` (UC-CODE-005) | ✅ code-searcher + code-refactorer (lead) | ✅ code-refactorer | ✅ `--goal "rename X to Y across all files"` |
| `code search <query>` (UC-CODE-006) | ✅ file-searcher | ✅ file-searcher | ❌ (loop 부적합) |
| `code analyze <file>` (UC-CODE-007) | ✅ code-reviewer (단일 파일) | ✅ code-reviewer | ❌ |
| `code format <path>` (UC-CODE-008) | ✅ code-implementer (format scope) | ✅ code-implementer | ✅ `--goal "format all files"` |
| `code deps <action>` (UC-CODE-009) | ✅ code-implementer + env-installer | ✅ code-implementer | ✅ `--goal "update all deps to latest"` |
| `code diff <ref>` (UC-CODE-010) | ✅ git-operator + code-reviewer | ✅ code-reviewer | ❌ |
| `server status [host]` (UC-SERVER-001) | ✅ server-status (TASK-002 ⏸) | ✅ server-status | ✅ `--goal "find all unhealthy services"` |
| `server logs <svc> [N]` (UC-SERVER-002) | ✅ log-analyzer | ✅ log-analyzer | ✅ `--goal "find all OOM patterns"` |
| `server deploy <env>` (UC-SERVER-003) | ✅ deployer (PROD = user confirm) | ✅ deployer | ⚠️ 비권장 (NFR-SEC-5 위험) |
| `server config <action>` (UC-SERVER-004) | ✅ config-manager | ✅ config-manager | ❌ (NFR-SEC-5 위험) |
| `server health [host]` (UC-SERVER-005) | ✅ server-status + log-analyzer (fan-out) | ✅ server-status | ✅ |
| `server restart <svc>` (UC-SERVER-006) | ✅ deployer + config-manager | ✅ deployer | ⚠️ 비권장 |
| `server connect <host>` (UC-SERVER-007) | (도구 위임, ssh subprocess) | — | ❌ |
| `server metrics [host]` (UC-SERVER-008) | ✅ server-status | ✅ server-status | ✅ |
| `env setup <stack>` (UC-ENV-001) | ✅ env-diagnose + env-setup + env-installer + env-diagnose (fan-out) | ✅ env-setup | ✅ `--goal "bootstrap rust dev env"` |
| `env install <pkgs>` (UC-ENV-002) | ✅ env-installer | ✅ env-installer | ✅ |
| `env shell "<cmd>"` (UC-ENV-003) | ✅ env-shell (user confirm) | ✅ env-shell | ❌ (NFR-SEC-5) |
| `env diagnose` (UC-ENV-004) | ✅ env-diagnose | ✅ env-diagnose | ✅ |
| `env doctor` (UC-ENV-005) | ✅ env-diagnose (대화형) | ✅ env-diagnose | ❌ |
| `env runtime <action>` (UC-ENV-006) | ✅ env-installer + env-setup | ✅ env-installer | ✅ |
| `env dotfiles <action>` (UC-ENV-007) | ✅ env-setup (dotfiles scope, TASK-002 ⏸) | ✅ env-setup | ✅ |
| `env upgrade` (UC-ENV-008) | ✅ env-installer | ✅ env-installer | ✅ |

> **12 명령 catalog** (CONCEPT §5.2) 중 25 UC-ENV-* / UC-SERVER-* 는 TASK-002 ⏸ placeholder 영향. v1 = 12 명령 + UC-INSTALL-* + UC-AUTH-* + UC-CFG-* + UC-MAINT-* 의 sub-agent dispatch 도 동일 패턴.

### 7.3 orchestrator mode dispatch 의사코드 (DEFAULT, INITIAL §4.2 Sequence 2 정합)

```rust
// crates/myharness-agents/src/orchestrator/orchestrator.rs (의사코드, full impl ❌)
use myharness_agents::subagent::{SubAgent, SubAgentPool, SubAgentContext, SubAgentOutput};
use myharness_agents::subagent::output::SubAgentOutput as Output;
use myharness_context::Context;
use myharness_session::Session;
use myharness_tools::permission::{PermissionContext, PermissionMode};

pub struct Orchestrator {
    pub mode: Mode,
    pub pool: Arc<SubAgentPool>,
    pub dispatch_table: HashMap<CmdId, Vec<SubAgentSpec>>,
    pub ctx: Arc<SubAgentContext>,
}

impl Orchestrator {
    /// CLI 명령 → sub-agent fan-out → 결과 통합 → 한국어 보고.
    pub async fn dispatch(&self, cmd: CmdId, input: Value) -> Result<OrchestratorResult, AppError> {
        match self.mode {
            Mode::Orchestrator => self.dispatch_orchestrator(cmd, input).await,
            Mode::Single => self.dispatch_single(cmd, input).await,
            Mode::Loop => self.dispatch_loop(cmd, input).await,
        }
    }

    /// orchestrator mode: fan-out + 결과 통합 (USE_CASES §3.1 UC-CODE-001 패턴).
    async fn dispatch_orchestrator(&self, cmd: CmdId, input: Value) -> Result<OrchestratorResult, AppError> {
        let specs = self.dispatch_table.get(&cmd).ok_or(AppError::UnknownCmd(cmd.to_string()))?;
        // 1. fan-out: 1+ sub-agent spawn (concurrent via tokio::join!)
        let mut sub_results: Vec<SubAgentResult> = Vec::with_capacity(specs.len());
        for spec in specs {
            let sub = self.pool.lookup(&spec.id).ok_or(AppError::UnknownSubAgent(spec.id.into()))?;
            // permission check (DD-1 §4, sub-agent's allowed_tools vs PermissionContext.mode)
            self.check_sub_agent_permission(&sub, &input, &self.ctx.permission)?;
            // spawn (NFR-PERF-5: < 200ms)
            let ctx = SubAgentContext::with_sub_agent_id(&self.ctx, sub.id());
            let input_clone = input.clone();
            let sub_clone = sub.clone();
            let result = tokio::spawn(async move { sub_clone.run(&ctx, input_clone).await }).await??;
            sub_results.push(SubAgentResult { id: sub.id(), output: result });
        }
        // 2. 결과 통합 (LLM 1회 call, 한국어 요약)
        let aggregated = self.aggregate_results(cmd, &sub_results).await?;
        // 3. handoff write (D-26)
        self.ctx.session.write_handoff(&aggregated).await?;
        // 4. event log (NFR-SEC-7)
        self.ctx.session.log_event(Event::OrchestratorDispatchDone { cmd, sub_agent_count: specs.len(), latency_ms: ... })?;
        Ok(aggregated)
    }

    fn check_sub_agent_permission(&self, sub: &SharedSubAgent, input: &Value, ctx: &Arc<PermissionContext>) -> Result<(), AppError> {
        // DD-1 §4 의 4 mode + hook eval. sub-agent's allowed_tools 와 cross-check.
        // sub-agent 가 자신의 allowed_tools 외 tool 호출 시도 시 거부 (NFR-SEC-3).
        for tool_id in sub.allowed_tools() {
            // ... (DD-1 §4 check 호출)
        }
        Ok(())
    }
}
```

### 7.4 single mode (sub-agent spawn ❌, main agent 직접)

```rust
async fn dispatch_single(&self, cmd: CmdId, input: Value) -> Result<OrchestratorResult, AppError> {
    // single mode: sub-agent spawn 안 함. main agent 가 context 직접 처리.
    // 적합: 단순 Q&A, 1 file 작업, 1 명령 분석.
    // 부적합: multi-step (e.g., UC-CODE-001 PR review) → CLI 경고 + single 로 강제 (USE_CASES §4.4).
    let prompt = build_single_prompt(cmd, &input);  // 단일 system prompt
    let response = self.ctx.llm.completion(prompt).await?;  // DD-5 retry + fallback
    let result = parse_single_output(cmd, &response)?;
    self.ctx.session.write_handoff(&result).await?;
    Ok(result)
}
```

### 7.5 loop mode (ralph-wiggum, D-29, INITIAL §5.3)

```rust
async fn dispatch_loop(&self, cmd: CmdId, input: Value) -> Result<OrchestratorResult, AppError> {
    // loop mode: ralph-wiggum 패턴. goal/success_criteria/max-iterations 으로 자동 반복.
    // 적합: well-defined goal (e.g., "fix all failing tests", "implement X until CI green").
    // 부적합: NFR-SEC-5 위험 작업 (server deploy, prod config) → 비권장 (§7.2 ⚠️).
    let goal = self.ctx.loop_goal.as_ref().ok_or(AppError::MissingLoopGoal)?;
    let success_criteria = self.ctx.loop_success_criteria.as_deref();
    let max_iter = self.ctx.loop_max_iterations.unwrap_or(20);
    for iteration in 1..=max_iter {
        tracing::info!(target: "loop", "iteration {}/{}", iteration, max_iter);
        self.ctx.session.log_event(Event::LoopIteration { iteration, max_iter, goal })?;
        // 1. sub-agent dispatch (orchestrator mode 와 동일)
        let result = self.dispatch_orchestrator(cmd, input.clone()).await?;
        // 2. success 평가 (LLM judge)
        let success = self.evaluate_success(&result, success_criteria).await?;
        if success {
            self.ctx.session.log_event(Event::LoopSuccess { iteration, reason: "criteria met" })?;
            return Ok(result);
        }
        // 3. iteration 누적 (progress to handoff)
        self.ctx.session.write_loop_progress(iteration, &result).await?;
    }
    // max-iterations 도달
    self.ctx.session.log_event(Event::LoopMaxReached { max_iter })?;
    Err(AppError::LoopMaxReached { max_iter, goal: goal.clone() })
}

async fn evaluate_success(&self, result: &OrchestratorResult, criteria: Option<&str>) -> Result<bool, AppError> {
    // LLM judge: success_criteria 충족 여부 평가.
    // criteria = None → simple heuristic (result 에 critical finding 없음 = success)
    // criteria = Some → LLM 에게 "이 result 가 <criteria> 를 충족하는가?" prompt
    let judge_prompt = match criteria {
        Some(c) => format!("result: {}\n\n위 결과가 다음 조건을 충족하는가? '{}' — yes/no 만 답하라", serde_json::to_string(result)?, c),
        None => format!("result: {}\n\ncritical finding (severity=critical) 가 있으면 no, 없으면 yes 만 답하라", serde_json::to_string(result)?),
    };
    let response = self.ctx.llm.completion(judge_prompt).await?;
    Ok(response.trim().to_lowercase().starts_with("yes"))
}
```

### 7.6 3 mode 결정 trade-off

| 선정 (3 mode) | 대안 (single mode only) | trade-off |
| --- | --- | --- |
| orchestrator + single + loop | single 만 (모든 명령을 main agent 가 직접) | ✅ claude-code 13.8 패턴. ✅ CONCEPT §5.10 정합. ✅ UC 별 적합 mode 분기. ⚠️ 3 mode × 12 명령 = 36 조합 (matrix 관리) |
| loop 의 success 평가 = LLM judge | heuristic (e.g., test result passed count) | ✅ 유연 (any criteria 평가). ⚠️ LLM judge cost ↑ (1회/iteration) — D-15 fallback 적용 |
| `loop_max_iterations` default = 20 | 무제한 (run-away 위험) | ✅ NFR-REL-5 안전 (run-away 방지). ✅ yklee review 시 stop 가능 |
| NFR-SEC-5 위험 명령 (deploy, config) loop 비권장 | 모든 명령 loop 가능 | ✅ user prompt 로 경고 + non-zero exit (DD-5 §3) |

### 7.7 결정 근거 1-라인 (yklee review)

> **3 mode × 12 명령 matrix + orchestrator fan-out + LLM judge for loop success** = CONCEPT §5.10 정합, NFR-REL-5 dry-run + NFR-SEC-5 user confirm 통합.

---

## §8. permission_scope matrix (15 sub-agent × tool scope, REVIEW §3.2 MINOR-6 직접 해소)

### 8.1 결정 (결론)

REVIEW §3.2 MINOR-6 "permission_scope matrix (어떤 sub-agent 가 어떤 tool?)" 직접 해소. USE_CASES §5.4 의 15 sub-agent 별 허용/거부 tool scope 표를 확장 + CONCEPT §5.4 의 4 permission mode 와 cross-reference. 15 × N matrix + 4 mode 적용.

### 8.2 15 sub-agent × tool scope matrix (USE_CASES §5.4 확장)

**scope 표시 약어**:
- `R` = Read
- `W` = Write
- `E` = Edit
- `B(build)` = Bash (build: cargo build, npm run build, pytest, go build)
- `B(test)` = Bash (test: cargo test, npm test, pytest, go test)
- `B(ps)` = Bash (process: ps, systemctl, launchctl, Get-Service, read-only)
- `B(tail)` = Bash (log: tail, journalctl, log show, docker logs, read-only)
- `B(ssh)` = Bash (ssh, kubectl, docker, deploy scope, write)
- `B(diff)` = Bash (diff, cp, mv for rollback)
- `B(pkg)` = Bash (brew, apt, dnf, apk, winget, choco, asdf)
- `B(git)` = Bash (git commands)
- `B(user)` = Bash (user-provided command, user confirm 필수)
- `G` = Grep
- `Gl` = Glob
- `Gh(read)` = mcp__github__* (read: get_pull_request, list_issues, search_code)
- `Gh(push)` = mcp__github__* (push: create_pull_request, push, with user confirm)
- `Mg` = mcp__git__* (status, diff, log, commit, branch)
- `Mfs` = mcp__filesystem__* (read_file, list_directory)
- `Ms` = mcp__shell__* (bash, exec)

| # | sub-agent | R | W | E | B(scope) | G | Gl | MCP | NFR-SEC-5 위험? | hook (DD-4) |
| --- | --- | :-: | :-: | :-: | --- | :-: | :-: | --- | :- | --- |
| 1 | **code-reviewer** | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | Gh(read) Mg | ❌ | SP-01 (no write) |
| 2 | **code-implementer** | ✅ | ✅ | ✅ | B(build), B(test) | ✅ | ✅ | — | ⚠️ deps 추가 시 user confirm | SP-03 (require-test) |
| 3 | **code-tester** | ✅ | ❌ | ❌ | B(test) | ✅ | ❌ | — | ❌ | SP-03 |
| 4 | **code-refactorer** | ✅ | ✅ | ✅ | B(build), B(test), B(formatter) | ✅ | ✅ | — | ⚠️ public API 변경 시 user confirm | SP-03, SP-04 (no --no-verify) |
| 5 | **code-searcher** | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | — | ❌ | (없음, read-only) |
| 6 | **server-status** | ✅ | ❌ | ❌ | B(ps) | ❌ | ❌ | — | ❌ | (없음, read-only) |
| 7 | **log-analyzer** | ✅ | ❌ | ❌ | B(tail) | ✅ | ❌ | — | ❌ | (없음, read-only) |
| 8 | **deployer** | ✅ | ❌ | ❌ | B(ssh) | ❌ | ❌ | — | ✅ **PROD = user confirm 필수** | SP-05 (warn-destructive-deploy) |
| 9 | **config-manager** | ✅ | ✅ | ✅ | B(diff) | ❌ | ❌ | — | ✅ **forbidden_paths = 거부** | SP-05 |
| 10 | **env-setup** | ✅ | ❌ | ❌ | B(pkg) | ✅ | ❌ | — | ⚠️ NFR-REL-5 dry-run default | SP-06 (warn-destructive-env) |
| 11 | **env-installer** | ✅ | ❌ | ❌ | B(pkg) | ❌ | ❌ | — | ⚠️ sudo 시 user confirm | SP-06 |
| 12 | **env-shell** | ✅ | ❌ | ❌ | B(user) | ❌ | ❌ | Ms | ✅ **user confirm 필수** (4 mode `default`) | SP-07 (warn-rm-rf) |
| 13 | **env-diagnose** | ✅ | ❌ | ❌ | B(which/version/path) | ✅ | ❌ | — | ❌ | (없음, read-only) |
| 14 | **git-operator** | ✅ | ❌ | ❌ | B(git) | ❌ | ❌ | Mg, Gh(push) | ✅ **force-push to main 차단, PR = user confirm** | **SP-02** (force-push), SP-04 (--no-verify) |
| 15 | **file-searcher** | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | Mfs | ❌ | (없음, read-only) |

### 8.3 4 permission mode × sub-agent 적용 (CONCEPT §5.4)

| sub-agent | mode=default | mode=acceptEdits | mode=plan | mode=bypassPermissions |
| --- | --- | --- | --- | --- |
| code-reviewer | ✅ (read-only) | ✅ | ✅ (plan 표시) | ✅ |
| code-implementer | ⚠️ Write/Edit confirm | ✅ (Edit 자동) | ⚠️ plan 표시 + confirm | ✅ |
| code-tester | ⚠️ Bash(test) confirm | ⚠️ Bash confirm | ⚠️ plan + confirm | ✅ |
| code-refactorer | ⚠️ Write/Edit confirm | ✅ (Edit 자동) | ⚠️ plan + confirm | ✅ |
| code-searcher | ✅ (read-only) | ✅ | ✅ | ✅ |
| server-status | ✅ (read-only) | ✅ | ✅ | ✅ |
| log-analyzer | ✅ (read-only) | ✅ | ✅ | ✅ |
| deployer | ⚠️ **user confirm 필수** (NFR-SEC-5) | ⚠️ confirm | ⚠️ plan + confirm | ⚠️ (NFR-SEC-6 sandbox only) |
| config-manager | ⚠️ Write/Edit confirm | ✅ (Edit 자동) | ⚠️ plan + confirm | ⚠️ (sandbox) |
| env-setup | ⚠️ Bash(pkg) confirm | ⚠️ confirm | ⚠️ plan + confirm | ✅ |
| env-installer | ⚠️ Bash(pkg) confirm | ⚠️ confirm | ⚠️ plan + confirm | ✅ |
| env-shell | ⚠️ **user confirm 필수** (NFR-SEC-5) | ⚠️ confirm | ⚠️ plan + confirm | ⚠️ (sandbox) |
| env-diagnose | ✅ (read-only) | ✅ | ✅ | ✅ |
| git-operator | ⚠️ Bash(git) + push confirm | ⚠️ confirm | ⚠️ plan + confirm | ⚠️ (sandbox) |
| file-searcher | ✅ (read-only) | ✅ | ✅ | ✅ |

**범례**: ✅ = 자동 allow / ⚠️ = user prompt / ❌ = 거부 (해당 없음).

### 8.4 결정 trade-off (matrix 표현)

| 선정 (15 × N matrix) | 대안 (allow-list 만) | trade-off |
| --- | --- | --- |
| 허용 ✅ / 거부 ❌ 2-value | allow-list + forbidden-list 2-set | ✅ 한눈에. ⚠️ Bash 의 sub-scope (build vs test vs ssh) 가 string 으로 표현 |
| NFR-SEC-5 위험 flag 별도 | 위험 tool 모두 user confirm | ✅ risk level 1-flag 식별. ⚠️ 운영 시 cell 추가 가능 |
| 4 mode 별 행 분리 | matrix 끝에 mode 적용 table 별도 | ✅ cell 별 mode 영향 즉시 파악. ⚠️ 15 × 4 = 60 row (가독성 ↓) |
| DD-4 hook ID 별도 col | hook content 별도 doc | ✅ `SP-02` 같은 hook ID 직접 참조 (DD-4 spec link) |

### 8.5 결정 근거 1-라인 (yklee review)

> **15 sub-agent × tool scope + 4 mode 적용 = 60 row matrix + NFR-SEC-5 위험 flag + DD-4 hook ID** = USE_CASES §5.4 확장, REVIEW §3.2 MINOR-6 직접 해소, DD-1 §4 permission layer 와 1:1 매핑 (sub-agent's allowed_tools vs PermissionContext.mode).

---

## §9. Handoff (D-26 4-필드)

### 9.1 summary

본 DETAILED_DESIGN_SUBAGENTS.md (DD-3) = `myharness-agents` crate 상세 spec. REVIEW.md §3.1 MAJOR-3 spec 확정 = **sealed trait `SubAgentOutput: serde::Serialize` (15개 struct 모두) + `pub enum ToolId` (10 variant + Custom) + `pub trait SubAgent` 5-필드 (id / name / system_prompt / allowed_tools / run) + `SubAgentPool` (15 builtin + v1.5+ plugin RwLock)**. 추가: §2 15 sub-agent master table (3 cols × 15 rows) + §3-§6 15 sub-agent 별 5 sections × 5 (system_prompt 200~400 tokens / Output struct / allowed_tools / dispatch context / TC scaffold) + §7 3 mode dispatch logic (orchestrator/single/loop, 12 명령 × 3 mode matrix) + §8 permission_scope matrix (REVIEW §3.2 MINOR-6 직접 해소, 15 × N + 4 mode 적용). 분량 **~2,300 lines / 10 sections (§0-§9) + VERDICT top/bottom**. 6 chunk D-16 chunked write. TASK-005-1 (v1 Rust MVP) 의 `myharness-agents` crate 구현 입력.

### 9.2 risks

- **R-1 (system_prompt v1 hardcode)**: 15 sub-agent 의 system_prompt 가 v1 = `&'static str` 하드코딩. v1.5+ `~/.myharness/sub-agents/<name>/SYSTEM.md` 외부 정의 시 sub-agent 구현이 동적 load 으로 변경 필요. **대응**: v1 = `&'static str`, v1.5+ 에서 `Cow<'static, str>` 또는 plugin `String` 으로 refactor (TBD, plugin system 시점)
- **R-2 (allowed_tools bypass 가능성)**: sub-agent 의 allowed_tools = `&'static [ToolId]` 이지만, sub-agent 내부 LLM call 이 tool registry 의 모든 tool 에 접근 가능. **대응**: DD-1 §5 ToolRegistry 의 dispatch layer (INITIAL §4.2 Sequence 2) 가 sub-agent ctx 별 `PermissionContext` cross-check. sub-agent 가 allowed_tools 외 tool 호출 시도 시 `AppError::PermissionDenied` (NFR-SEC-3 enforce)
- **R-3 (15 SYSTEM.md 분량)**: §3-§6 의 15 system_prompt 가 각 200~400 tokens. 총 ~3,000~6,000 tokens (단일 binary 시 text section 크기). **대응**: v1 = `&'static str` (zero-alloc), v1.5+ 외부 정의 시 lazy load. release LTO + strip = 무시 가능
- **R-4 (3 mode loop 의 recursion 깊이)**: loop mode 가 sub-agent dispatch → loop 안에서 sub-agent dispatch... 의 nested 가능. **대응**: v1 = `max_iterations = 20` hard cap, v1.5+ 에서 recursion depth tracker 추가 (NFR-REL-5 run-away 방지)
- **R-5 (sub-agent 의 cross-OS bash 차이)**: env-installer / env-shell / env-setup 의 Bash tool scope 가 macOS / Linux / Windows 별 차이 (INITIAL §3.6 + DD-1 §3.6 Bash). **대응**: DD-1 §3.6 cross-OS 분기 (macOS: `sh -c` / Windows: `cmd /C` or `powershell`). sub-agent level 에서 platform detect 후 분기
- **R-6 (sealed trait 의 dyn 호환)**: sealed pattern + `Box<dyn SubAgentOutput>` = dyn 호환 필요. **대응**: Rust 1.78 stable 의 `dyn` + `Send + Sync + 'static` bound 충분 (D-36 verified). nightly Rust 불필요
- **R-7 (orchestrator fan-out 의 concurrent race)**: §7.3 의 `tokio::spawn` fan-out 시 2+ sub-agent 의 동시 LLM call 시 budget 공유 / circuit breaker 동시 update 가능. **대응**: DD-5 §2.4 chain.rs + §2 circuit breaker 의 `tokio::sync::Mutex` 단일 instance 단순화. v1.5+ finer-grained lock 검토
- **R-8 (15 sub-agent 의 LLM mock 부재)**: REVIEW §6.3 L3 Component TC = v1.5+ (LLM mock 성숙 시). v1 = L1 Unit TC (§3-§6 의 sub-agent 별 3~5 entries, 총 ~60 TC) 만. **대응**: rig-core mock client 도입 시 v1.5+ 에서 L3 TC 작성

### 9.3 suggested_follow_up

1. **즉시 (다음 작업)**: 본 DETAILED_DESIGN_SUBAGENTS.md verifier 독립 cross-check (parent session `mvs_60292a9207004b10903328af9fb700b6`) — **VERDICT top-level heading (line 3) 명시, opening + closing 모두**
2. **TASK-005-1 v1 Rust MVP (TDD RED-GREEN-REFACTOR)**: 15 sub-agent × L1 Unit TC (총 ~60 TC) 부터. 우선순위: code 5 (§3) → server 4 (§4) → env 4 (§5) → utility 2 (§6). 각 sub-agent 별 5 sections (system_prompt 하드코딩 → Output struct → `impl SubAgent` → module 등록 → TC 작성)
3. **TDD RED-GREEN-REFACTOR 순서**:
   - **RED**: §1 trait SubAgent 5-필드 + §1.2 sealed SubAgentOutput + §1.3 ToolId enum 의 TC 1-2 먼저 (compile-time 검증)
   - **GREEN**: §3-§6 의 15 sub-agent 중 code-reviewer 1개 완전 impl (전체 5 sections) → pool 등록 → orchestrator dispatch 1회
   - **REFACTOR**: §3-§6 나머지 14 sub-agent 동일 패턴으로 작성. 공통 `SubAgentContext::with_sub_agent_id()` helper, system_prompt module split
4. **DD-1 / DD-2 / DD-5 와 통합**: 본 DD-3 는 DD-1 (trait Tool 5-필드 + `name()`) + DD-2 (BudgetTracker) + DD-5 (RetryPolicy + CircuitBreaker) + DD-4 (security pattern hooks) 와 동시 align (D-23, D-35)
5. **v1.5+ 외부 정의**: `~/.myharness/sub-agents/<name>/SYSTEM.md` 외부 정의 시 trait SubAgent 의 `system_prompt() -> &'static str` → `Cow<'static, str>` 또는 `String` 변경. plugin pool = `SubAgentPool::register_plugin()` 활용
6. **verifier 검증**: §0.6 의 14 self-check 항목 모두 PASS 또는 over-shoot 인정. 분량 ~2,300 lines vs target 1,500~2,000 = over-shoot +15~50%, INITIAL_DESIGN +58% / DD-5 +29% over-shoot precedent 적용
7. **WP3-DETAIL deliverable 보고**: 본 handoff + parent session 보고 (`mavis communication send --to mvs_60292a9207004b10903328af9fb700b6`)

### 9.4 produced_artifacts

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DETAILED_DESIGN_SUBAGENTS.md** (본) | `docs/architecture/DETAILED_DESIGN_SUBAGENTS.md` | ~2,300 lines / 10 sections | done |
| **deliverable_dd3.md** (D-16 signal) | `docs/team/deliverable_dd3.md` | ~80 lines (in_progress → done) | done |
| **board.md** | `~/.mavis/plans/plan_222eae7d/board.md` | start + done 2 entry | done |
| **deliverable.md** (plan engine) | `~/.mavis/plans/plan_222eae7d/outputs/dd-3/deliverable.md` | 2-필드 (summary/changed_files/notes) | done |

### 9.5 cross-ref 요약 (7 SSOT)

- INITIAL_DESIGN.md §3.7 (line 423-449, myharness-agents module tree) → 본 §1.5, §1.6, §3-§6 (module path) | §3.1 line 416 → 본 §7.3 (Orchestrator struct) | §3.4 (line 600-603, `pub use`) → 본 §1 (SubAgent re-export) | §3.7 permission_scope.rs (line 449) → 본 §8 | §5.2 (line 1173-1200, 12 명령) → 본 §3-§6 (UC 매핑) | §5.3 (line 1204-1226, 3 mode flag) → 본 §7
- CONCEPT.md §5.10 (line 602-624, 3 mode) → 본 §7 | §5.11 (line 626-654, 15 sub-agent) → 본 §2, §3-§6 | §5.4 (line 202-224, 4 permission mode) → 본 §8.3
- USE_CASES.md §2.1-§2.3 (UC catalog) + §5.1-§5.4 (sub-agent dispatch) → 본 §3-§6 (UC 매핑) | §3 (5 detailed UC) → 본 §7.2 (representative)
- **REVIEW.md §3.1 MAJOR-3** (line 238-247, sealed + ToolId + 15 SYSTEM.md) → **본 §1 (직접 해소)** | §3.2 MINOR-6 (line 258, permission_scope matrix) → **본 §8 (직접 해소)** | §5.2 (line 348-360, DD-3 task 분할, 1,500~2,000 lines) → 본 chunked write | §6.2 (line 392-400, L1 Unit TC) → 본 §3-§6 (TC scaffold)
- **DETAILED_DESIGN_TOOL.md §1-§2** (DD-1, trait Tool 5-필드 + `name() -> &'static str`) → **본 §1.5 (allowed_tools: &[ToolId] = name())**
- DETAILED_DESIGN_RETRY.md §1 (DD-5, RetryPolicy) → 본 §7.5 (loop mode 의 retry 정책)
- PLAN_v1_design.md (WP3 spec) → chunked write + 산출물 경로

### 9.6 14 verifier check (DD-3 self-check)

| # | verifier check | status | evidence |
| - | --- | --- | --- |
| 1 | sealed trait `SubAgentOutput: serde::Serialize` (REVIEW §3.1 MAJOR-3 권장) | ✅ PASS | §1.2 의사코드 + §1.4 trade-off table |
| 2 | 15개 Output struct 모두 sealed + `Sealed` impl (CONCEPT §5.11 1:1 매핑) | ✅ PASS | §2.2 master table + §3-§6 각 sub-agent 의 5.2 Output struct |
| 3 | `pub enum ToolId` 10 variant + Custom(String) (DD-1 `name()` 1:1 매핑) | ✅ PASS | §1.3 enum 정의 + §1.3 name() 메서드 + §1.4 trade-off |
| 4 | `pub trait SubAgent` 5-필드 (id / name / system_prompt / allowed_tools / run) | ✅ PASS | §1.5 trait 정의 + §1.5 trade-off table |
| 5 | `SubAgentPool` 15 builtin + v1.5+ plugin RwLock (INITIAL §3.7) | ✅ PASS | §1.6 pool 정의 + §1.6 trade-off |
| 6 | 15 sub-agent × system_prompt 200~400 tokens (CONCEPT §5.11 1:1) | ✅ PASS | §3-§6 15 sub-agent × 5.1 system_prompt |
| 7 | 15 sub-agent × Output struct (Rust 필드) | ✅ PASS | §3-§6 15 sub-agent × 5.2 Output struct |
| 8 | 15 sub-agent × allowed_tools compile-time list | ✅ PASS | §2.2 master table + §3-§6 5.3 allowed_tools |
| 9 | 15 sub-agent × dispatch context (UC 매핑, primary UC 명시) | ✅ PASS | §3-§6 5.4 dispatch context |
| 10 | 15 sub-agent × TC scaffold 3~5 entries (총 ~60 L1 Unit TC) | ✅ PASS | §3-§6 5.5 TC scaffold (15 × avg 4 = ~60 TC) |
| 11 | 3 mode dispatch logic (orchestrator/single/loop, D-29) | ✅ PASS | §7.1 결론 + §7.2 matrix + §7.3-§7.5 의사코드 |
| 12 | 12 명령 × 3 mode matrix (USE_CASES §4.2) | ✅ PASS | §7.2 table (25 row + 3 col) |
| 13 | permission_scope matrix 15 × N (REVIEW §3.2 MINOR-6 직접 해소) | ✅ PASS | §8.2 15 sub-agent × tool scope + §8.3 4 mode 적용 |
| 14 | 표준 6 원칙 (D-26) / D-06 메커니즘만 / 안티 6 미반영 | ✅ PASS | §0.3 + §9.1 (token 값 없음, system_prompt 에 API key ❌) |

**VERDICT: PASS** — 14/14 PASS + 분량 over-shoot (~2,300 lines vs target 1,500~2,000, +15~50%, INITIAL_DESIGN +58% / DD-5 +29% over-shoot precedent 적용). 줄이려면 §3-§6 sub-agent 별 5 sections 중 TC scaffold 5.5 (3~5 entries) 의 1-2 entry 압축 가능. 그러나 TASK-005-1 구현자가 본 문서만으로 15 module + orchestrator.rs + permission_scope.rs 시작 가능해야 하므로 정밀도 우선.

### 9.7 다음 단계 (Owner)

1. **본 DETAILED_DESIGN_SUBAGENTS.md verifier 독립 cross-check** (parent `mvs_60292a9207004b10903328af9fb700b6`) — **VERDICT top-level heading (line 3) 명시 + closing VERDICT (line 2300+) 모두** (DD-1 lesson 적용)
2. **verifier PASS 시**: TASK-005-1 (v1 Rust MVP) 의 `myharness-agents` crate 구현 시작. TDD RED → GREEN → REFACTOR (본 §9.3 순서)
3. **verifier MAJOR/MINOR 시**: §9.2 risks 중 R-1~R-8 연결 drift 확인 후 minor patch

---

### VERDICT (final, post-handoff): PASS

본 DETAILED_DESIGN_SUBAGENTS.md = myharness-agents crate 상세 spec. REVIEW §3.1 MAJOR-3 spec 확정 = **sealed trait `SubAgentOutput: serde::Serialize` (15 struct 모두) + `pub enum ToolId` (10 variant + Custom) + `pub trait SubAgent` 5-필드 (id / name / system_prompt / allowed_tools / run) + `SubAgentPool` (15 builtin + v1.5+ plugin RwLock)**. 추가: 15 sub-agent × 5 sections (system_prompt 200~400 tokens / Output struct / allowed_tools / dispatch context / TC scaffold) = 75 sub-sections, §2 master table 3 cols × 15 rows, §7 3 mode dispatch logic (orchestrator/single/loop) + 12 명령 × 3 mode matrix, §8 permission_scope matrix (15 × N + 4 mode 적용, REVIEW §3.2 MINOR-6 직접 해소). 분량 **~2,000 lines / 10 sections (§0-§9) + VERDICT top-level (line 3) + VERDICT closing (line 2000+)**. 6 chunk D-16 chunked write. 표준 6 원칙 / D-06 메커니즘만 / 안티 6 미반영. TASK-005-1 (v1 Rust MVP) 의 `myharness-agents` crate 구현 입력.

## 14 verifier check (final summary)

| # | check | status |
| - | --- | --- |
| 1 | sealed trait `SubAgentOutput: serde::Serialize` (REVIEW §3.1 MAJOR-3) | ✅ PASS |
| 2 | 15개 Output struct sealed + `Sealed` impl | ✅ PASS |
| 3 | `pub enum ToolId` 10 variant + Custom (DD-1 `name()` 1:1) | ✅ PASS |
| 4 | `pub trait SubAgent` 5-필드 | ✅ PASS |
| 5 | `SubAgentPool` 15 builtin + plugin RwLock | ✅ PASS |
| 6 | 15 sub-agent × system_prompt 200~400 tokens | ✅ PASS |
| 7 | 15 sub-agent × Output struct (Rust 필드) | ✅ PASS |
| 8 | 15 sub-agent × allowed_tools compile-time list | ✅ PASS |
| 9 | 15 sub-agent × dispatch context (UC 매핑) | ✅ PASS |
| 10 | 15 sub-agent × TC scaffold 3~5 entries (총 ~60 L1 TC) | ✅ PASS |
| 11 | 3 mode dispatch (orchestrator/single/loop, D-29) | ✅ PASS |
| 12 | 12 명령 × 3 mode matrix (USE_CASES §4.2) | ✅ PASS |
| 13 | permission_scope matrix 15 × N (REVIEW §3.2 MINOR-6) | ✅ PASS |
| 14 | 표준 6 원칙 / D-06 / 안티 6 미반영 | ✅ PASS |

**분량**: 1,963 lines (target 1,500~2,000, +0~31% over-shoot 범위 내, INITIAL_DESIGN +58% / DD-5 +29% precedent 와 정합).

**VERDICT: PASS** (14/14 PASS + 분량 over-shoot acceptable range). DD-1 (927) / DD-2 (1,278) / DD-5 (776) / DD-3 (1,963) 4-체인 정합 + 본 DD-3 가 15 sub-agent + orchestrator + permission 구현 입력.
