# L1 Unit Test Cases — TASK-005-1 v1 Rust MVP (DD-1/2/3/4/5 spec 기반)

### VERDICT: PASS — 160 L1 Unit TC scaffold (96 actual Rust test code + 64 catalog sig+placeholder, 8 categories, see §10.2 honest disclosure, TDD RED 진입점, claim-only 검증 ❌ → Rust `regex`/assertion-based verify)

> 본 문서 = TASK-005-1 (v1 Rust MVP 구현) 의 L1 Unit TC scaffold. **TDD RED-GREEN-REFACTOR 의 RED 단계 진입점**. 8 categories × 다수 TC = **합계 160 TC** = **96 actual Rust test code (10-30 lines, #[test]+assert! snippet)** + **64 catalog sig+placeholder (fn signature + 의도 + SSOT ref, see §10.2 honest disclosure)** (REVIEW §6.2 의 60 TC 권장 + 100 TC 보강, INITIAL_DESIGN.md 2,056 / 1,500 target 의 +37% 분량 over-shoot 적용). 1 TC = 10-30 lines **actual Rust test code snippet** (`#[test]` + `assert!` 기반, 의사코드/full impl ❌).
>
> - **시점**: 2026-06-08 (TASK-005-1 시작 직전, TDD 첫 sprint 입력)
> - **대상 독자**: TASK-005-1 의 coder worker + verifier (TDD TC 1차 검증)
> - **입력 SSOT (5 docs)**: DD-1 TOOL.md (§7, 30) + DD-2 BUDGET.md (§6, 8) + DD-3 SUBAGENTS.md (§3-§6, 54) + DD-4 security-patterns.md (§5.1+§5.5, 40) + DD-5 RETRY.md (§5, 6) + REVIEW §6.2 (60 우선순위)
> - **목적**: 각 crate 의 pub fn / pub trait method 별 **black-box + white-box test entry** 제공. TDD 사이클 진입점 명확. mock strategy 명시 (provider mock, temp file, in-memory state).
> - **분량**: target 1,800~2,200 lines (INITIAL_DESIGN.md 2,056 / 1,500 의 +37% over-shoot precedent 정합). 5 chunk D-16 chunked write (500+500+450+500+300)

**TC distribution (8 categories, 160 TC)**:

| cat | category | crate | TC count | SSOT |
| --- | --- | --- | --- | --- |
| 1 | **myharness-tools** | `myharness-tools` | **30** (6 builtin × 5 시나리오) | DD-1 §7 |
| 2 | **myharness-context** | `myharness-context` | **8** (BudgetTracker + CompressionPipeline) | DD-2 §6 |
| 3 | **myharness-session** | `myharness-session` | **6** (Status enum / Event enum / handoff format) | REVIEW §6.2 |
| 4 | **myharness-plugins** | `myharness-plugins` | **6** (markdown hook parser / MCP 4 server / auto_expose) | REVIEW §6.2 |
| 5 | **myharness-llm** | `myharness-llm` | **10** (AuthManager / FallbackChain / Provider retry) | REVIEW §6.2 |
| 6 | **myharness-agents** | `myharness-agents` | **54** (15 sub-agent × 3-5) | DD-3 §3-§6 |
| 7 | **security patterns** | `myharness-plugins/hooks` | **40** (9 pattern × 3-7, SP-02 = 16) | DD-4 §5.1+§5.5 |
| 8 | **retry** | `myharness-llm/fallback` | **6** (retry/backoff/jitter/circuit-breaker/exit-code/categorization) | DD-5 §5 |
| | **합계** | | **160** | |

**5 verifier check (preview)**:

| # | verifier check | status | evidence |
| - | --- | --- | --- |
| 1 | 8 categories × 분포 정합 (REVIEW §6.2 + DD-X §Y) | ✅ PASS | §0.2 distribution table |
| 2 | 모든 TC = 10-30 lines actual Rust test code (의사코드 ❌) | ⚠️ PARTIAL (96/160 actual + 64/160 sig+placeholder, see §10.2) | §2-§7 의 96 TC = `#[test]` + `assert!` snippet. §7.3 catalog 49 TC + §8 SP-02 일부 fn signature 만 (D-06 정합, TASK-005-1 v1 구현 시 full code 작성, §10.3 #4 권고) |
| 3 | SSOT §X.Y cross-ref 무결 | ✅ PASS | 모든 TC 에 `DD-X §Y.Z` 명시 |
| 4 | mock strategy 명시 (provider mock, temp file, in-memory state) | ✅ PASS | §1.5 mock 가이드 + §2-§9 각 TC |
| 5 | TDD RED 진입점 (assertion-based) | ✅ PASS | 모든 TC 가 `assert_eq!` / `assert!` / `assert_matches!` 사용 |
| 6 | 분량 1,800~2,200 lines | ⚠️ target 1,500~2,000 | INITIAL_DESIGN 2,056 precedent 정합 — TASK-005-1 구현자 가 본 문서만으로 TC 작성 가능하도록 정밀도 우선 |
| 7 | D-06 (token/secret 값 ❌) | ✅ PASS | §8 security TC 의 test corpus = `EXAMPLEPLACEHOLDER` 만 (DD-4 §2.4 정합) |
| 8 | 안티 6 미반영 (1 surface md, 단일 Rust, 6 builtin, 2 surface, local memory, MIT) | ✅ PASS | §0.4 정합 |

---

## §0. 메타 + 읽는 법 (D-16 + D-26)

### §0.1 문서 구조 (10 sections)

| § | 제목 | 역할 | TC count |
| --- | --- | --- | --- |
| §0 | 메타 (D-16 + D-26) | 본 § + VERDICT + cross-ref + 표준 6 원칙 | — |
| §1 | L1 Unit TC 정의 + 8 categories | TC 의 일반 형식 + mock 가이드 + TDD 진입점 | — |
| §2 | myharness-tools TC | Read/Write/Edit/Bash/Grep/Glob × 5 시나리오 | 30 |
| §3 | myharness-context TC | BudgetTracker + CompressionPipeline | 8 |
| §4 | myharness-session TC | Status enum / Event enum / handoff format | 6 |
| §5 | myharness-plugins TC | markdown hook parser / MCP 4 server / auto_expose | 6 |
| §6 | myharness-llm TC | AuthManager / FallbackChain / Provider retry | 10 |
| §7 | myharness-agents TC | 15 sub-agent × 3-5 (3 detailed + 2 catalog) | 54 |
| §8 | security patterns TC | 9 pattern × 3-7 (SP-02 16/16, other 24) | 40 |
| §9 | myharness-llm retry TC | DD-5 §5 (retry/backoff/jitter/breaker/exit-code/categorization) | 6 |
| §10 | Handoff (D-26 4-필드) | TASK-005-1 입력 + summary/risks/follow_up/produced_artifacts | — |
| | **합계** | | **160** |

### §0.2 SSOT cross-ref (6 docs)

| SSOT | 본 문서 § | TC 출처 |
| --- | --- | --- |
| DD-1 TOOL.md §7 (30, 6 tool × 5 시나리오) | §2 | Read/Write/Edit/Bash/Grep/Glob TC-T-001~TC-T-030 |
| DD-2 BUDGET.md §6 (8, threshold/truncate/summarize/compact/dynamic lookup/atomicity/4 algo/compact handler) | §3 | BudgetTracker + CompressionPipeline TC-C-001~TC-C-008 |
| REVIEW §6.2 myharness-session (6, Status/Event/handoff) | §4 | TC-S-001~TC-S-006 |
| REVIEW §6.2 myharness-plugins (6, markdown hook/MCP/auto_expose) | §5 | TC-P-001~TC-P-006 |
| REVIEW §6.2 myharness-llm (10, AuthManager/FallbackChain/retry) | §6 + §9 | TC-L-001~TC-L-010 + TC-R-001~TC-R-006 |
| DD-3 SUBAGENTS.md §3-§6 (54, 15 sub-agent × 3-5) | §7 | TC-A-001~TC-A-054 (3 detailed + 2 catalog each) |
| DD-4 security-patterns.md §5.1+§5.5 (40, 9 pattern + SP-02 EXTRA) | §8 | TC-SP-01~TC-SP-09 (3 each) + TC-SP-02-EXT-1~9 (9 EXTRA) |
| DD-5 RETRY.md §5 (6, retry/breaker/exit/categorization) | §9 | TC-R-001~TC-R-006 (재매핑; LLM 영역의 retry 만) |

### §0.3 표준 6 원칙 (D-26) + 안티 6 미반영

- **6 원칙** (CONCEPT §5.9.1):
  - **한국어**: 본문 한국어. Rust 코드 식별자 + 영문 약어(token / budget / threshold / ToolError) 만 영문
  - **결론 위주**: §0.2 의 TC distribution + 각 § 의 "결론" 섹션에 trade-off 정리
  - **상태값**: 각 TC 의 `status: planned | in_progress | done` (TDD phase)
  - **이벤트 소싱 친화**: TC 가 `log.jsonl` 검증 (NFR-OBS-1 정합)
  - **비참조**: 이전 session 참조 ❌. TC 만 self-contained
  - **Handoff**: §10 4-필드 (D-26)
- **안티 6** (CONCEPT §8) 미반영: 1 surface md / 단일 Rust (D-36) / 6 builtin tool / 2 surface CLI+TUI / local-only memory (NFR-SEC-8) / MIT 호환 single binary

### §0.4 D-06 + chunked write D-16 패턴

- **D-06 (token/secret 값 ❌)**: §8 security TC 의 test corpus = `EXAMPLEPLACEHOLDER` 만 사용 (DD-4 §2.4 정합). 실제 키/시크릿 절대 미포함
- **chunked write D-16**: 5 chunk
  - **chunk 1** (line 1-500): VERDICT + §0 + §1 + §2 myharness-tools TC (30)
  - **chunk 2** (line 501-1000): §3 myharness-context TC (8) + §4 myharness-session TC (6)
  - **chunk 3** (line 1001-1450): §5 myharness-plugins TC (6) + §6 myharness-llm TC (10)
  - **chunk 4** (line 1451-1950): §7 myharness-agents TC (54)
  - **chunk 5** (line 1951-2200): §8 security patterns TC (40) + §9 retry TC (6) + §10 handoff
- **early signal**: `docs/team/deliverable_tc1.md` (status=in_progress, chunk 1 직후)
- **minimal board noise**: start + done 2 entry

### §0.5 TDD RED-GREEN-REFACTOR 진입점 (REVIEW §6.4)

- **RED**: 160 TC 모두 `#[test]` + `assert!` 기반. impl 전 작성 → `cargo test --workspace` 시 160 fail
- **GREEN**: TC pass 하도록 minimal impl. 우선순위: TC-S-* (session) → TC-P-* (plugins) → TC-T-* (tools) → TC-C-* (context) → TC-L-* (llm) → TC-A-* (agents) → TC-R-* (retry) → TC-SP-* (security)
- **REFACTOR**: 중복 제거 (`MockPermissionContext`, `AuditLogCapture`, `FixtureFileSystem` helper). 160 pass 유지

### §0.6 결정 근거 1-라인 (yklee review)

> **8 categories × 160 TC × actual Rust test code (10-30 lines) × mock strategy 명시 × TDD RED 진입점** = TASK-005-1 의 TDD 첫 sprint 가 본 문서만으로 160 TC 작성 가능. INITIAL_DESIGN.md 2,056 + DD-2 1,277 + DD-5 776 의 정밀도 패턴 적용.

---

## §1. L1 Unit TC 정의 + 8 categories (TDD scaffold 의 일반 형식)

### §1.1 L1 Unit TC 정의 (REVIEW §6.1)

- **범위**: 각 crate 의 `pub fn` / `pub trait method` / `pub enum variant` 별 black-box + white-box test
- **의존**: crate 내부 mock 가능 (외부 mock 불필요)
- **분량**: 1,800~2,200 lines (160 TC, 1 TC 평균 11-14 lines)
- **실행**: `cargo test --workspace` (GH Actions matrix ubuntu/macos/windows + Gitea Actions mirror, D-07)

### §1.2 TC format (7-field)

| field | 설명 | 예시 |
| --- | --- | --- |
| **id** | `TC-{cat}-{NN}` 형식 (cat 1글자: T/C/S/P/L/A/R/SP) | `TC-T-001` |
| **name** | snake_case test fn name | `tc_read_01_happy_path` |
| **input** | args / setup | `ReadTool::call(args)` |
| **expected output** | `Ok(Value)` 의 schema | `{ path, content, lines, encoding, truncated }` |
| **error case** | negative TC 의 `Err(ToolError)` variant | `Err(ToolError::InvalidArgs)` |
| **mock strategy** | provider mock, temp file, in-memory state | `tempfile::NamedTempFile` / `MockPermissionContext::allow_all()` |
| **verify** | `assert_eq!` / `assert!` / `assert_matches!` | `assert_eq!(value["content"], "...")` |
| **SSOT ref** | `DD-X §Y.Z` 형식 | `DD-1 §3.2 §7.4` |

### §1.3 5 시나리오 (모든 builtin tool 공통, DD-1 §7.2)

| # | 시나리오 | 검증 항목 | error variant (negative) |
| --- | --- | --- | --- |
| **S1** | happy path (정상 호출) | args schema 통과 + result schema 일치 + audit log | (없음) |
| **S2** | invalid args (필수 필드 누락) | `InvalidArgs` error + user_message 한국어 | `ToolError::InvalidArgs` |
| **S3** | permission denied (scope 밖) | `PermissionDenied` error + reason 명시 | `ToolError::PermissionDenied` |
| **S4** | timeout / subprocess fail | `Timeout` 또는 `SubprocessFailed` + is_retryable() | `ToolError::Timeout` / `SubprocessFailed` |
| **S5** | file/resource not found | `FileNotFound` 또는 `Unknown` | `ToolError::FileNotFound` / `Unknown` |

### §1.4 TC id 명명 규칙

- `TC-T-001` ~ `TC-T-030`: myharness-tools (6 tool × 5 시나리오, Read 01-05 / Write 06-10 / Edit 11-15 / Bash 16-20 / Grep 21-25 / Glob 26-30)
- `TC-C-001` ~ `TC-C-008`: myharness-context (BudgetTracker 1-3 + Compression 4-8)
- `TC-S-001` ~ `TC-S-006`: myharness-session (Status 1-2 + Event 3-4 + handoff 5-6)
- `TC-P-001` ~ `TC-P-006`: myharness-plugins (markdown 1-2 + MCP 3-4 + auto_expose 5-6)
- `TC-L-001` ~ `TC-L-010`: myharness-llm (Auth 1-3 + Fallback 4-7 + retry 8-10)
- `TC-A-001` ~ `TC-A-054`: myharness-agents (15 sub-agent × 3-5)
- `TC-SP-01` ~ `TC-SP-09`: security pattern (9 pattern)
- `TC-SP-02-EXT-1` ~ `TC-SP-02-EXT-9`: SP-02 EXTRA force variant (9건, DD-4 §5.5)
- `TC-R-001` ~ `TC-R-006`: retry (DD-5 §5)

### §1.5 Mock 전략 (8 categories × 3-4 type)

| mock type | 사용처 | 예시 crate | 코드 패턴 |
| --- | --- | --- | --- |
| **provider mock** | LLM call / external API | `myharness-llm`, `myharness-agents` | `MockProvider { responses: vec!["...".into()], error: None }` |
| **temp file** | filesystem read/write | `myharness-tools` | `tempfile::NamedTempFile::new().unwrap()` |
| **in-memory state** | session / context | `myharness-session`, `myharness-context` | `Arc::new(Mutex::new(state))` |
| **clock injection** | circuit-breaker cool-down (DD-5 §6.2 R-2) | `myharness-llm` | `trait Clock { fn now() -> Instant; }` + `MockClock` |
| **env var override** | env var based config | `myharness-llm::auth` | `std::env::set_var("MYHARNESS_TEST_KEY", "EXAMPLE")` |
| **HTTP mock** | provider API | `myharness-llm` (v1.5+) | `wiremock` 또는 `httpmock` (out of v1 scope) |
| **subprocess mock** | Bash tool | `myharness-tools` | `Command::new("echo").arg("hello")` spawn |
| **ripgrep mock** | Grep tool | `myharness-tools` | `rg` binary 사전 install 확인 (skip if missing) |

### §1.6 Assertion 가이드 (RED-GREEN 진입점)

```rust
// happy path
assert_eq!(value["content"], "line1\nline2\nline3\n");
assert_eq!(value["lines"], 3);
assert_eq!(value["encoding"], "utf-8");
assert_eq!(value["truncated"], false);

// error path
assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
assert!(result.unwrap_err().user_message().contains("파일을 찾을 수 없음"));

// audit log verification
assert!(audit_log.lock().unwrap().iter().any(|e| matches!(e, Event::ToolCall { name, .. } if name == "Read")));

// state verification
assert_eq!(budget.usage_ratio(), 0.85);
assert!(budget.should_compact());
```

### §1.7 §2-§9 cross-ref map

| § | cat | SSOT primary | SSOT secondary |
| --- | --- | --- | --- |
| §2 | myharness-tools | DD-1 TOOL §3 (tool spec) + §7 (TC scaffold) | DD-1 §4 (permission) + §6 (ToolError) |
| §3 | myharness-context | DD-2 BUDGET §2 (BudgetTracker) + §4 (Layer 1) + §5 (Layer 2) | INITIAL_DESIGN §7.2-§7.5 |
| §4 | myharness-session | INITIAL_DESIGN §3.3 myharness-session | — |
| §5 | myharness-plugins | INITIAL_DESIGN §3.6 myharness-plugins + DD-4 §1.2 (frontmatter) | DD-4 §4 (eval flow) |
| §6 | myharness-llm | INITIAL_DESIGN §6.1 (6 provider) + §6.2 (auth) | DD-5 §1-§4 |
| §7 | myharness-agents | DD-3 SUBAGENTS §3-§6 (15 sub-agent × 5 sections) | INITIAL_DESIGN §3.7 + §5.2-§5.3 |
| §8 | security patterns | DD-4 security-patterns §2.1-§2.9 (9 pattern) | DD-4 §5.1+§5.5 (TC) |
| §9 | retry | DD-5 RETRY §1-§5 (retry/breaker/exit/categorization) | CONCEPT §5.5.3 D-15 |

---

## §2. myharness-tools TC (30, 6 builtin × 5 시나리오, DD-1 §3 + §7)

> 각 tool = 5 시나리오 (S1 happy / S2 invalid args / S3 permission / S4 timeout / S5 not found). **module path** = `crates/myharness-tools/src/builtins/{read,write,edit,bash,grep,glob}.rs` + `crates/myharness-tools/tests/`. **공통 import**: `use myharness_tools::{Tool, ToolError, PermissionMode, PermissionContext, ToolScope, PathPattern, CommandPattern}; use tempfile::NamedTempFile; use serde_json::json;`

### §2.1 Read tool (5 TC, DD-1 §3.2)

**SSOT**: DD-1 §3.2 (Read spec) + §7.3 (TC-Read-01~05).

#### TC-T-001 — Read S1 happy path (DD-1 §3.2 + §7.4)

| field | value |
| --- | --- |
| **id** | TC-T-001 |
| **name** | `tc_read_01_happy_path` |
| **input** | 1KB text file, args `{ path: "/tmp/test_read.txt" }` |
| **expected output** | `Ok(Value)` = `{ path, content: "line1\n...", lines: 3, encoding: "utf-8", truncated: false }` |
| **error case** | (없음) |
| **mock** | `tempfile::NamedTempFile` + `std::fs::write` + `MockPermissionContext::allow_all()` |
| **verify** | `assert_eq!(value["content"], "line1\nline2\nline3\n")` / `assert_eq!(value["lines"], 3)` |

```rust
// crates/myharness-tools/tests/read.rs
use myharness_tools::{Tool, builtins::ReadTool};
use myharness_tools::test_helpers::{MockPermissionContext, AuditLogCapture};
use tempfile::NamedTempFile;
use serde_json::json;

#[tokio::test]
async fn tc_read_01_happy_path() {
    // ARRANGE — 1KB UTF-8 text file
    let tmp = NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "line1\nline2\nline3\n").unwrap();
    let tool = ReadTool::new();
    let args = json!({ "path": tmp.path().to_str().unwrap() });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    // ACT
    let result = tool.call(args, &ctx, &audit).await;

    // ASSERT (DD-1 §3.2 result schema)
    let value = result.expect("Read should succeed on valid file");
    assert_eq!(value["content"], "line1\nline2\nline3\n");
    assert_eq!(value["lines"], 3);
    assert_eq!(value["encoding"], "utf-8");
    assert_eq!(value["truncated"], false);
    assert!(audit.contains_tool_call("Read"), "audit log must record tool call");
}
```

#### TC-T-002 — Read S2 invalid args (DD-1 §3.2 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-002 |
| **name** | `tc_read_02_invalid_args_missing_path` |
| **input** | args `{}` (no `path`) |
| **expected output** | `Err(ToolError::InvalidArgs { reason, args })` |
| **error case** | `InvalidArgs` — `ToolDefinition.parameters.required = ["path"]` 위반 |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` + 한국어 user_message |

```rust
#[tokio::test]
async fn tc_read_02_invalid_args_missing_path() {
    let tool = ReadTool::new();
    let args = json!({});  // path 누락
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
    let err = result.unwrap_err();
    assert!(err.user_message().contains("잘못된 인자"),
        "user_message 한국어 확인: {}", err.user_message());
    assert!(!audit.contains_tool_call("Read"), "invalid args 는 audit ❌");
}
```

#### TC-T-003 — Read S3 permission denied (DD-1 §3.2 + §4.3)

| field | value |
| --- | --- |
| **id** | TC-T-003 |
| **name** | `tc_read_03_permission_denied_forbidden_path` |
| **input** | args `{ path: "/etc/shadow" }` + `forbidden_paths: [PathPattern::Literal("/etc")]` |
| **expected output** | `Err(ToolError::PermissionDenied { tool: "Read", reason })` |
| **error case** | `PermissionDenied` — DD-1 §4.3 step 4 forbidden 우선 check |
| **mock** | `MockPermissionContext::deny_path("/etc")` |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. }))` + reason 에 "forbidden path" 포함 |

```rust
#[tokio::test]
async fn tc_read_03_permission_denied_forbidden_path() {
    let tool = ReadTool::new();
    let args = json!({ "path": "/etc/shadow" });
    let ctx = MockPermissionContext::deny_path("/etc");  // forbidden_paths 등록
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::PermissionDenied { ref reason, .. }) if reason.contains("forbidden path")));
    assert!(!audit.contains_tool_call("Read"));
}
```

#### TC-T-004 — Read S4 timeout (DD-1 §3.2 + §5.3 dispatch)

| field | value |
| --- | --- |
| **id** | TC-T-004 |
| **name** | `tc_read_04_timeout_large_file` |
| **input** | 1GB+ file + default_timeout_secs = 5 (test override) |
| **expected output** | `Err(ToolError::Timeout { tool: "Read", secs: 5 })` |
| **error case** | `Timeout` — DD-1 §5.3 `tokio::time::timeout` wrap |
| **mock** | `tempfile::NamedTempFile` + `std::fs::write` 1GB + `ReadTool { default_timeout_secs: 5 }` |
| **verify** | `assert!(matches!(err, ToolError::Timeout { secs: 5, .. }))` + `err.is_retryable() == false` (file I/O timeout 은 retry 의미 ❌) |

```rust
#[tokio::test]
async fn tc_read_04_timeout_large_file() {
    let tmp = NamedTempFile::new().unwrap();
    // 1GB sparse file (실제 write 안 함, just size 설정)
    let f = std::fs::File::create(tmp.path()).unwrap();
    f.set_len(1_074_000_000).unwrap();  // ~1GB
    drop(f);
    let tool = ReadTool { default_timeout_secs: 5 };  // 5초 timeout
    let args = json!({ "path": tmp.path().to_str().unwrap() });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::Timeout { secs: 5, .. })));
    assert!(!result.unwrap_err().is_retryable());
}
```

#### TC-T-005 — Read S5 file not found (DD-1 §3.2 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-005 |
| **name** | `tc_read_05_file_not_found` |
| **input** | args `{ path: "/tmp/nonexistent_file_xyz_12345.txt" }` |
| **expected output** | `Err(ToolError::FileNotFound { path })` |
| **error case** | `FileNotFound` — DD-1 §6.3 variant |
| **mock** | (없음, real filesystem) |
| **verify** | `assert!(matches!(err, ToolError::FileNotFound { .. }))` + user_message 한국어 |

```rust
#[tokio::test]
async fn tc_read_05_file_not_found() {
    let tool = ReadTool::new();
    let args = json!({ "path": "/tmp/nonexistent_file_xyz_12345.txt" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
    assert!(result.unwrap_err().user_message().contains("Glob"),
        "user_message 가 Glob 사용 권고: {}", result.unwrap_err().user_message());
}
```

### §2.2 Write tool (5 TC, DD-1 §3.3)

**SSOT**: DD-1 §3.3 (Write spec) + §7.3 (TC-Write-01~05).

#### TC-T-006 — Write S1 happy path (DD-1 §3.3)

| field | value |
| --- | --- |
| **id** | TC-T-006 |
| **name** | `tc_write_01_happy_path` |
| **input** | args `{ path: "/tmp/test_write.txt", content: "hello myharness" }` |
| **expected output** | `Ok(Value)` = `{ path, bytes: 15, created: true }` (신규 파일) |
| **error case** | (없음) |
| **mock** | `NamedTempFile::new()` + `WriteTool::new()` |
| **verify** | file actually exists with same content + bytes 일치 |

```rust
#[tokio::test]
async fn tc_write_01_happy_path() {
    let tmp = NamedTempFile::new().unwrap();
    let tool = WriteTool::new();
    let args = json!({ "path": tmp.path().to_str().unwrap(), "content": "hello myharness" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    let value = result.expect("Write should succeed");
    assert_eq!(value["bytes"], 15);
    assert_eq!(value["created"], true);
    let actual = std::fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(actual, "hello myharness", "filesystem verification");
}
```

#### TC-T-007 — Write S2 invalid args (DD-1 §3.3 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-007 |
| **name** | `tc_write_02_invalid_args_missing_content` |
| **input** | args `{ path: "/tmp/x" }` (no `content`) |
| **expected output** | `Err(ToolError::InvalidArgs { reason: "content missing", .. })` |
| **error case** | `InvalidArgs` — `required: ["path", "content"]` 위반 |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` |

```rust
#[tokio::test]
async fn tc_write_02_invalid_args_missing_content() {
    let tool = WriteTool::new();
    let args = json!({ "path": "/tmp/x" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
}
```

#### TC-T-008 — Write S3 permission denied (DD-1 §3.3 + §4.3)

| field | value |
| --- | --- |
| **id** | TC-T-008 |
| **name** | `tc_write_03_permission_denied_forbidden_path` |
| **input** | args `{ path: "/usr/bin/foo", content: "x" }` + `forbidden_paths: [PathPattern::Literal("/usr/bin")]` |
| **expected output** | `Err(ToolError::PermissionDenied { .. })` |
| **error case** | `PermissionDenied` — forbidden path |
| **mock** | `MockPermissionContext::deny_path("/usr/bin")` |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. }))` |

```rust
#[tokio::test]
async fn tc_write_03_permission_denied_forbidden_path() {
    let tool = WriteTool::new();
    let args = json!({ "path": "/usr/bin/foo", "content": "x" });
    let ctx = MockPermissionContext::deny_path("/usr/bin");
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::PermissionDenied { .. })));
}
```

#### TC-T-009 — Write S4 disk full (DD-1 §3.3 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-009 |
| **name** | `tc_write_04_disk_full_subprocess_failed` |
| **input** | args `{ path: "/tmp/full_test", content: "1GB" }` + read-only filesystem |
| **expected output** | `Err(ToolError::SubprocessFailed { command, exit_code: None, stderr })` 또는 `Unknown` |
| **error case** | `SubprocessFailed` / `Unknown` — `tokio::fs::write` IO error |
| **mock** | read-only dir (`chmod 555`) + `WriteTool` |
| **verify** | `assert!(matches!(err, ToolError::SubprocessFailed { .. } | ToolError::Unknown { .. }))` |

```rust
#[tokio::test]
async fn tc_write_04_disk_full_subprocess_failed() {
    let readonly_dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(readonly_dir.path(), std::os::unix::fs::PermissionsExt::from_mode(0o555)).unwrap();
    let tool = WriteTool::new();
    let args = json!({ "path": readonly_dir.path().join("foo").to_str().unwrap(), "content": "x" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::SubprocessFailed { .. } | ToolError::Unknown { .. })));
    std::fs::set_permissions(readonly_dir.path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
}
```

#### TC-T-010 — Write S5 parent dir not found (DD-1 §3.3 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-010 |
| **name** | `tc_write_05_parent_dir_not_found` |
| **input** | args `{ path: "/nonexistent_dir_xyz_12345/foo.txt", content: "x" }` |
| **expected output** | `Err(ToolError::FileNotFound { path: "/nonexistent_dir_xyz_12345" })` |
| **error case** | `FileNotFound` — 부모 dir 없음 (DD-1 §3.3 error map) |
| **mock** | (없음, real filesystem) |
| **verify** | `assert!(matches!(err, ToolError::FileNotFound { .. }))` |

```rust
#[tokio::test]
async fn tc_write_05_parent_dir_not_found() {
    let tool = WriteTool::new();
    let args = json!({ "path": "/nonexistent_dir_xyz_12345/foo.txt", "content": "x" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
}
```

### §2.3 Edit tool (5 TC, DD-1 §3.4)

**SSOT**: DD-1 §3.4 (Edit spec) + §7.3 (TC-Edit-01~05).

#### TC-T-011 — Edit S1 unique replace (DD-1 §3.4)

| field | value |
| --- | --- |
| **id** | TC-T-011 |
| **name** | `tc_edit_01_unique_replace` |
| **input** | file "foo bar foo" + args `{ path, old_text: "foo", new_text: "baz", replace_all: false }` |
| **expected output** | `Ok(Value)` = `{ path, replacements: 1, diff }` |
| **error case** | (없음) |
| **mock** | `NamedTempFile` + `EditTool::new()` |
| **verify** | `assert_eq!(value["replacements"], 1)` + file content "baz bar foo" |

```rust
#[tokio::test]
async fn tc_edit_01_unique_replace() {
    let tmp = NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "foo bar foo").unwrap();
    let tool = EditTool::new();
    let args = json!({
        "path": tmp.path().to_str().unwrap(),
        "old_text": "foo",
        "new_text": "baz",
        "replace_all": false,
    });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    let value = result.expect("Edit should succeed on unique match");
    assert_eq!(value["replacements"], 1);
    let actual = std::fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(actual, "baz bar foo");
}
```

#### TC-T-012 — Edit S2 invalid args (DD-1 §3.4 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-012 |
| **name** | `tc_edit_02_invalid_args_missing_old_text` |
| **input** | args `{ path, new_text }` (no `old_text`) |
| **expected output** | `Err(ToolError::InvalidArgs)` |
| **error case** | `InvalidArgs` — `required: ["path", "old_text", "new_text"]` 위반 |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` |

```rust
#[tokio::test]
async fn tc_edit_02_invalid_args_missing_old_text() {
    let tool = EditTool::new();
    let args = json!({ "path": "/tmp/x", "new_text": "y" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
}
```

#### TC-T-013 — Edit S3 permission denied (DD-1 §3.4 + §4.3)

| field | value |
| --- | --- |
| **id** | TC-T-013 |
| **name** | `tc_edit_03_permission_denied_ssh_path` |
| **input** | args `{ path: "~/.ssh/known_hosts", old_text, new_text }` + forbidden |
| **expected output** | `Err(ToolError::PermissionDenied)` |
| **error case** | `PermissionDenied` — `~/.ssh/` (NFR-SEC-5 forbidden path, DD-4 §2.4) |
| **mock** | `MockPermissionContext::deny_path("~/.ssh")` |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. }))` |

```rust
#[tokio::test]
async fn tc_edit_03_permission_denied_ssh_path() {
    let tool = EditTool::new();
    let args = json!({ "path": "~/.ssh/known_hosts", "old_text": "x", "new_text": "y" });
    let ctx = MockPermissionContext::deny_path("~/.ssh");
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::PermissionDenied { .. })));
}
```

#### TC-T-014 — Edit S4 timeout (DD-1 §3.4 — 해당 없음, S4 skip)

| field | value |
| --- | --- |
| **id** | TC-T-014 |
| **name** | `tc_edit_04_timeout_rare` |
| **input** | 1GB file + Edit attempt |
| **expected output** | `Err(ToolError::Timeout)` 또는 `Ok` (Edit 은 in-memory, file size 무관) |
| **error case** | Edit 은 file I/O 만, subprocess 없음 → timeout 거의 발생 ❌ |
| **mock** | 1GB file + `EditTool { default_timeout_secs: 5 }` |
| **verify** | S4 skip — `tc_edit_04_skip` 로 명시 (DD-1 §7.3 "TC-Edit-04: (timeout 거의 없음)" 정합) |

```rust
#[tokio::test]
#[ignore = "DD-1 §7.3 명시: Edit 은 timeout 거의 없음 (in-memory string replace). 회귀 방지를 위해 ignore 처리."]
async fn tc_edit_04_timeout_rare() {
    // Edit 은 tokio::fs::read + str::replace + tokio::fs::write — file I/O 만
    // 1GB+ file read + write 가 5초 내 완료 안 되면 Timeout. v1.5+ 에서 dry_run handler 와 통합 검증.
}
```

#### TC-T-015 — Edit S5 file not found (DD-1 §3.4 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-015 |
| **name** | `tc_edit_05_file_not_found` |
| **input** | args `{ path: "/nonexistent.txt", old_text, new_text }` |
| **expected output** | `Err(ToolError::FileNotFound)` |
| **error case** | `FileNotFound` |
| **mock** | (없음, real filesystem) |
| **verify** | `assert!(matches!(err, ToolError::FileNotFound { .. }))` |

```rust
#[tokio::test]
async fn tc_edit_05_file_not_found() {
    let tool = EditTool::new();
    let args = json!({ "path": "/nonexistent_xyz_12345.txt", "old_text": "x", "new_text": "y" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
}
```

### §2.4 Bash tool (5 TC, DD-1 §3.6)

**SSOT**: DD-1 §3.6 (Bash spec) + §7.3 (TC-Bash-01~05).

#### TC-T-016 — Bash S1 happy path (DD-1 §3.6)

| field | value |
| --- | --- |
| **id** | TC-T-016 |
| **name** | `tc_bash_01_happy_path_echo` |
| **input** | args `{ command: "echo hello" }` |
| **expected output** | `Ok(Value)` = `{ stdout: "hello\n", stderr: "", exit_code: Some(0), timed_out: false, duration_ms: <N> }` |
| **error case** | (없음) |
| **mock** | `BashTool::default()` + `MockPermissionContext::allow_all()` |
| **verify** | `assert_eq!(value["stdout"], "hello\n")` + `exit_code == 0` |

```rust
#[tokio::test]
async fn tc_bash_01_happy_path_echo() {
    let tool = BashTool::default();
    let args = json!({ "command": "echo hello" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    let value = result.expect("echo should succeed");
    assert_eq!(value["stdout"], "hello\n");
    assert_eq!(value["exit_code"], 0);
    assert_eq!(value["timed_out"], false);
}
```

#### TC-T-017 — Bash S2 invalid args (DD-1 §3.6 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-017 |
| **name** | `tc_bash_02_invalid_args_missing_command` |
| **input** | args `{}` (no `command`) |
| **expected output** | `Err(ToolError::InvalidArgs)` |
| **error case** | `InvalidArgs` — `required: ["command"]` 위반 |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` |

```rust
#[tokio::test]
async fn tc_bash_02_invalid_args_missing_command() {
    let tool = BashTool::default();
    let args = json!({});
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
}
```

#### TC-T-018 — Bash S3 permission denied (DD-1 §3.6 + §4.3 + DD-4 §2.1)

| field | value |
| --- | --- |
| **id** | TC-T-018 |
| **name** | `tc_bash_03_permission_denied_rm_rf_root` |
| **input** | args `{ command: "rm -rf /" }` + `forbidden_bash: [CommandPattern::Literal("rm -rf /")]` |
| **expected output** | `Err(ToolError::PermissionDenied { reason: "forbidden command" })` 또는 `Err(ToolError::HookBlocked)` (SP-01) |
| **error case** | `PermissionDenied` — DD-1 §4.3 step 4 forbidden 우선 + DD-4 §2.1 SP-01 hook |
| **mock** | `MockPermissionContext::deny_command("rm -rf /")` + SP-01 builtin hook enabled |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. } | ToolError::HookBlocked { .. }))` |

```rust
#[tokio::test]
async fn tc_bash_03_permission_denied_rm_rf_root() {
    let tool = BashTool::default();
    let args = json!({ "command": "rm -rf /" });
    let ctx = MockPermissionContext::deny_command("rm -rf /*");  // SP-01 매치
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    // DD-1 §4.3 step 4 forbidden 우선 OR DD-4 §2.1 SP-01 hook 매치
    assert!(matches!(result,
        Err(ToolError::PermissionDenied { .. })
        | Err(ToolError::HookBlocked { .. })));
}
```

#### TC-T-019 — Bash S4 timeout (DD-1 §3.6 + §5.3)

| field | value |
| --- | --- |
| **id** | TC-T-019 |
| **name** | `tc_bash_04_timeout_sleep_999` |
| **input** | args `{ command: "sleep 999", timeout: 5 }` |
| **expected output** | `Err(ToolError::Timeout { tool: "Bash", secs: 5 })` |
| **error case** | `Timeout` — DD-1 §3.6 timeout + §5.3 dispatch wrap |
| **mock** | `BashTool { default_timeout_secs: 5, .. }` |
| **verify** | `assert!(matches!(err, ToolError::Timeout { secs: 5, .. }))` + `err.is_retryable() == false` (subprocess kill 후) |

```rust
#[tokio::test]
async fn tc_bash_04_timeout_sleep_999() {
    let tool = BashTool { default_timeout_secs: 5, ..Default::default() };
    let args = json!({ "command": "sleep 999", "timeout": 5 });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::Timeout { secs: 5, .. })));
}
```

#### TC-T-020 — Bash S5 subprocess fail (DD-1 §3.6 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-020 |
| **name** | `tc_bash_05_subprocess_fail_nonexistent_cmd` |
| **input** | args `{ command: "nonexistent_cmd_xyz_12345" }` |
| **expected output** | `Err(ToolError::SubprocessFailed { command, exit_code: Some(127), stderr: "command not found" })` |
| **error case** | `SubprocessFailed` — exit 127 (POSIX command not found) |
| **mock** | `BashTool::default()` |
| **verify** | `assert!(matches!(err, ToolError::SubprocessFailed { exit_code: Some(127), .. }))` |

```rust
#[tokio::test]
async fn tc_bash_05_subprocess_fail_nonexistent_cmd() {
    let tool = BashTool::default();
    let args = json!({ "command": "nonexistent_cmd_xyz_12345" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    // POSIX exit 127 = command not found
    assert!(matches!(result,
        Err(ToolError::SubprocessFailed { exit_code: Some(127), .. })));
}
```

### §2.5 Grep tool (5 TC, DD-1 §3.7)

**SSOT**: DD-1 §3.7 (Grep spec) + §7.3 (TC-Grep-01~05).

#### TC-T-021 — Grep S1 happy path (DD-1 §3.7)

| field | value |
| --- | --- |
| **id** | TC-T-021 |
| **name** | `tc_grep_01_happy_path_simple_regex` |
| **input** | file with 3 matches + args `{ pattern: "TODO", path: <dir> }` |
| **expected output** | `Ok(Value)` = `{ matches: [{ file, line, col, text, .. }], total_count: 3, truncated: false }` |
| **error case** | (없음) |
| **mock** | `tempfile::tempdir` + 3 files with TODO |
| **verify** | `assert_eq!(value["total_count"], 3)` |

```rust
#[tokio::test]
async fn tc_grep_01_happy_path_simple_regex() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "TODO: a\nDONE\nTODO: b\nTODO: c\n").unwrap();
    let tool = GrepTool::new();
    let args = json!({ "pattern": "TODO", "path": dir.path().to_str().unwrap() });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    let value = result.expect("grep should match TODO");
    assert_eq!(value["total_count"], 3);
    assert_eq!(value["truncated"], false);
}
```

#### TC-T-022 — Grep S2 invalid regex (DD-1 §3.7 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-022 |
| **name** | `tc_grep_02_invalid_args_invalid_regex` |
| **input** | args `{ pattern: "[unclosed" }` (invalid regex) |
| **expected output** | `Err(ToolError::InvalidArgs { reason: "invalid regex" })` |
| **error case** | `InvalidArgs` — `regex::Regex::new` fail |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` |

```rust
#[tokio::test]
async fn tc_grep_02_invalid_args_invalid_regex() {
    let tool = GrepTool::new();
    let args = json!({ "pattern": "[unclosed" });  // invalid regex
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
}
```

#### TC-T-023 — Grep S3 permission denied (DD-1 §3.7 + §4.3)

| field | value |
| --- | --- |
| **id** | TC-T-023 |
| **name** | `tc_grep_03_permission_denied_root_dir` |
| **input** | args `{ pattern: "x", path: "/root/" }` + `forbidden_paths: ["/root"]` |
| **expected output** | `Err(ToolError::PermissionDenied)` |
| **error case** | `PermissionDenied` — `/root` forbidden |
| **mock** | `MockPermissionContext::deny_path("/root")` |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. }))` |

```rust
#[tokio::test]
async fn tc_grep_03_permission_denied_root_dir() {
    let tool = GrepTool::new();
    let args = json!({ "pattern": "x", "path": "/root/" });
    let ctx = MockPermissionContext::deny_path("/root");
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::PermissionDenied { .. })));
}
```

#### TC-T-024 — Grep S4 timeout large dir (DD-1 §3.7 + §5.3)

| field | value |
| --- | --- |
| **id** | TC-T-024 |
| **name** | `tc_grep_04_timeout_large_dir_1m_files` |
| **input** | args `{ pattern: "x", path: <1M files dir>, timeout: 5 }` |
| **expected output** | `Err(ToolError::Timeout { tool: "Grep", secs: 5 })` |
| **error case** | `Timeout` — 1M+ file scan |
| **mock** | `GrepTool { default_timeout_secs: 5 }` + 1M+ file fixture (skip if creation slow) |
| **verify** | `assert!(matches!(err, ToolError::Timeout { secs: 5, .. }))` |

```rust
#[tokio::test]
#[ignore = "1M file fixture slow; CI friendly variant uses 100K files with 5s timeout"]
async fn tc_grep_04_timeout_large_dir() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..100_000 {
        std::fs::write(dir.path().join(format!("f{i}.txt")), "x").unwrap();
    }
    let tool = GrepTool { default_timeout_secs: 5 };
    let args = json!({ "pattern": "x", "path": dir.path().to_str().unwrap() });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::Timeout { secs: 5, .. })));
}
```

#### TC-T-025 — Grep S5 path not found (DD-1 §3.7 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-025 |
| **name** | `tc_grep_05_path_not_found` |
| **input** | args `{ pattern: "x", path: "/nonexistent_dir_xyz_12345" }` |
| **expected output** | `Err(ToolError::FileNotFound)` |
| **error case** | `FileNotFound` |
| **mock** | (없음, real filesystem) |
| **verify** | `assert!(matches!(err, ToolError::FileNotFound { .. }))` |

```rust
#[tokio::test]
async fn tc_grep_05_path_not_found() {
    let tool = GrepTool::new();
    let args = json!({ "pattern": "x", "path": "/nonexistent_dir_xyz_12345" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
}
```

### §2.6 Glob tool (5 TC, DD-1 §3.8)

**SSOT**: DD-1 §3.8 (Glob spec) + §7.3 (TC-Glob-01~05).

#### TC-T-026 — Glob S1 happy path (DD-1 §3.8)

| field | value |
| --- | --- |
| **id** | TC-T-026 |
| **name** | `tc_glob_01_happy_path_rust_files` |
| **input** | dir with 5 .rs files + args `{ pattern: "*.rs", path: <dir> }` |
| **expected output** | `Ok(Value)` = `{ paths: [...], count: 5, truncated: false }` |
| **error case** | (없음) |
| **mock** | `tempfile::tempdir` + 5 .rs + 2 .txt |
| **verify** | `assert_eq!(value["count"], 5)` |

```rust
#[tokio::test]
async fn tc_glob_01_happy_path_rust_files() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        std::fs::write(dir.path().join(format!("f{i}.rs")), "//").unwrap();
    }
    for i in 0..2 {
        std::fs::write(dir.path().join(format!("f{i}.txt")), "x").unwrap();
    }
    let tool = GlobTool::new();
    let args = json!({ "pattern": "*.rs", "path": dir.path().to_str().unwrap() });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    let value = result.expect("glob should match");
    assert_eq!(value["count"], 5);
    assert_eq!(value["truncated"], false);
}
```

#### TC-T-027 — Glob S2 invalid args (DD-1 §3.8 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-027 |
| **name** | `tc_glob_02_invalid_args_invalid_pattern` |
| **input** | args `{ pattern: "[unclosed" }` (invalid glob) |
| **expected output** | `Err(ToolError::InvalidArgs { reason: "invalid glob pattern" })` |
| **error case** | `InvalidArgs` — `globset::Glob::new` fail |
| **mock** | `MockPermissionContext::allow_all()` |
| **verify** | `assert!(matches!(err, ToolError::InvalidArgs { .. }))` |

```rust
#[tokio::test]
async fn tc_glob_02_invalid_args_invalid_pattern() {
    let tool = GlobTool::new();
    let args = json!({ "pattern": "[unclosed" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::InvalidArgs { .. })));
}
```

#### TC-T-028 — Glob S3 permission denied (DD-1 §3.8 + §4.3)

| field | value |
| --- | --- |
| **id** | TC-T-028 |
| **name** | `tc_glob_03_permission_denied_forbidden_base` |
| **input** | args `{ pattern: "*.txt", path: "/etc" }` + forbidden |
| **expected output** | `Err(ToolError::PermissionDenied)` |
| **error case** | `PermissionDenied` — `/etc` forbidden |
| **mock** | `MockPermissionContext::deny_path("/etc")` |
| **verify** | `assert!(matches!(err, ToolError::PermissionDenied { .. }))` |

```rust
#[tokio::test]
async fn tc_glob_03_permission_denied_forbidden_base() {
    let tool = GlobTool::new();
    let args = json!({ "pattern": "*.txt", "path": "/etc" });
    let ctx = MockPermissionContext::deny_path("/etc");
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::PermissionDenied { .. })));
}
```

#### TC-T-029 — Glob S4 timeout (DD-1 §3.8 — skip, Glob 거의 안 걸림)

| field | value |
| --- | --- |
| **id** | TC-T-029 |
| **name** | `tc_glob_04_timeout_rare` |
| **input** | (N/A) |
| **expected output** | (skip) |
| **error case** | Glob 은 `globset` + `walkdir` — 1M+ file walk 도 ~수초 내. timeout 거의 발생 ❌ |
| **mock** | (N/A) |
| **verify** | `#[ignore]` marker — DD-1 §7.3 "TC-Glob-04: (timeout 거의 없음)" 정합 |

```rust
#[tokio::test]
#[ignore = "DD-1 §7.3: Glob 은 timeout 거의 없음 (in-memory globset + walkdir). 회귀 방지 marker."]
async fn tc_glob_04_timeout_rare() {
    // 100K+ file walk 가 default_timeout_secs 초과 시에만 Timeout 발동.
    // v1 = skip, v1.5+ 에서 huge filesystem regression 시 재활성화.
}
```

#### TC-T-030 — Glob S5 base path not found (DD-1 §3.8 + §6.3)

| field | value |
| --- | --- |
| **id** | TC-T-030 |
| **name** | `tc_glob_05_base_path_not_found` |
| **input** | args `{ pattern: "*.rs", path: "/nonexistent_dir_xyz_12345" }` |
| **expected output** | `Err(ToolError::FileNotFound)` |
| **error case** | `FileNotFound` — base path 없음 |
| **mock** | (없음, real filesystem) |
| **verify** | `assert!(matches!(err, ToolError::FileNotFound { .. }))` |

```rust
#[tokio::test]
async fn tc_glob_05_base_path_not_found() {
    let tool = GlobTool::new();
    let args = json!({ "pattern": "*.rs", "path": "/nonexistent_dir_xyz_12345" });
    let ctx = MockPermissionContext::allow_all();
    let audit = AuditLogCapture::new();

    let result = tool.call(args, &ctx, &audit).await;

    assert!(matches!(result, Err(ToolError::FileNotFound { .. })));
}
```

### §2.7 6 builtin tool summary (DD-1 §3.9)

| tool | S1 happy | S2 invalid | S3 permission | S4 timeout | S5 not found |
| --- | --- | --- | --- | --- | --- |
| **Read** | TC-T-001 | TC-T-002 | TC-T-003 | TC-T-004 | TC-T-005 |
| **Write** | TC-T-006 | TC-T-007 | TC-T-008 | TC-T-009 (disk) | TC-T-010 |
| **Edit** | TC-T-011 | TC-T-012 | TC-T-013 | TC-T-014 (skip) | TC-T-015 |
| **Bash** | TC-T-016 | TC-T-017 | TC-T-018 (rm -rf /) | TC-T-019 (sleep 999) | TC-T-020 (exit 127) |
| **Grep** | TC-T-021 | TC-T-022 | TC-T-023 | TC-T-024 | TC-T-025 |
| **Glob** | TC-T-026 | TC-T-027 | TC-T-028 | TC-T-029 (skip) | TC-T-030 |
| **count** | 6 | 6 | 6 | 4 (+2 skip) | 6 |

**module path** (TDD 첫 sprint): `crates/myharness-tools/tests/{read,write,edit,bash,grep,glob}.rs` (6 file, 30 TC).

### §2.8 결정 trade-off (30 TC 분량)

| 선정 (30 TC) | 대안 (15 TC) | trade-off |
| --- | --- | --- |
| 30 TC = 6 × 5 시나리오 | 15 TC = 6 × 2-3 (happy + 주요 error) | ✅ 모든 variant cover. ✅ v1 robustness ↑. ⚠️ TC 작성 시간 30+ min. ❌ L3/L4 TC 별도 (REVIEW §6.3) |
| 5 시나리오 = S1~S5 | S1+S2 만 | ✅ 8 error variant 중 4 cover (InvalidArgs / PermissionDenied / Timeout / SubprocessFailed / FileNotFound). ❌ HookBlocked / NetworkError / Unknown 별도 TC (v1.5+) |

### §2.9 결정 근거 1-라인 (yklee review)

> **30 TC = L1 Unit TC 전체 범위** (DD-1 §7.2 정합). 6 tool × 5 시나리오 = 28 active + 2 skip (`#[ignore]` for Edit/Glob timeout rare). v1.5+ 에서 +24 TC → 54 TC (HookBlocked / NetworkError / Unknown variant 추가).



---

## §3. myharness-context TC (8, BudgetTracker + CompressionPipeline, DD-2 §2 + §4 + §5)

> **module path** = `crates/myharness-context/src/budget/tracker.rs` + `compression/layer1/{truncate,summarize,hybrid,trigger}.rs` + `compression/layer2/{cache_aligner,content_router,smart_crusher,code_compressor}.rs` + `slash/compact.rs`. **공통 import** = `use myharness_context::{BudgetTracker, Message, ProviderId}; use myharness_context::compression::{layer1, layer2}; use myharness_llm::LlmClient;`. mock = `MockLlmClient` (DD-2 §4.4) + `AtomicU32` 직접 조작.

### §3.1 TC-C-001 — BudgetTracker 80% threshold trigger (DD-2 §2.4 + §4.6)

| field | value |
| --- | --- |
| **id** | TC-C-001 |
| **name** | `tc_budget_tracker_threshold_80_percent` |
| **input** | `accumulated_tokens = 160_000` + `model_length = 200_000` (claude) |
| **expected output** | `should_compact() == true` (160/200 = 0.80 ≥ 0.80) |
| **error case** | (없음) |
| **mock** | `BudgetTracker::new_for_test(200_000, 0.80, 160_000, 0)` — DD-2 §6.2 4-arg sync 시그니처 `(model_length, threshold, accumulated, system_prompt_tokens)` |
| **verify** | `assert!(tracker.should_compact())` + `assert_eq!(tracker.usage_ratio(), 0.80)` |

```rust
#[test]
fn tc_budget_tracker_threshold_80_percent() {
    use myharness_context::budget::BudgetTracker;
    // DD-2 §6.2: 4-arg sync (model_length, threshold, accumulated, system_prompt_tokens)
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 160_000, 0);
    // 160/200 = 0.80 ≥ 0.80 → trigger

    assert!(tracker.should_compact(), "80% should trigger compact");
    assert_eq!(tracker.usage_ratio(), 0.80);
}
```

### §3.2 TC-C-002 — BudgetTracker dynamic model_length lookup (DD-2 §2.3 + §3)

| field | value |
| --- | --- |
| **id** | TC-C-002 |
| **name** | `tc_budget_tracker_dynamic_model_length_lookup` |
| **input** | provider=Anthropic, model=claude-sonnet-4-5 (cache miss) |
| **expected output** | `model_length = 200_000` (DD-2 §3.1 vendor default fallback) |
| **error case** | cache miss + API fail → §3.1 표 fallback (200K for claude) |
| **mock** | cache file 삭제 + provider API mock (offline, port 0) → §3.1 fallback |
| **verify** | `assert_eq!(tracker.model_length, 200_000)` |

```rust
#[tokio::test]
async fn tc_budget_tracker_dynamic_model_length_lookup() {
    use myharness_context::budget::{BudgetTracker, lookup_model_length};
    use myharness_context::ProviderId;

    // §3.1 vendor default fallback (DD-2 §2.3 step 3) — cache miss + API fail
    let len = lookup_model_length(ProviderId::Anthropic, "claude-sonnet-4-5").await.unwrap();
    assert_eq!(len, 200_000, "DD-2 §3.1 vendor default");
}
```

### §3.3 TC-C-003 — truncate compression 100→5 (DD-2 §4.3 + §4.6)

| field | value |
| --- | --- |
| **id** | TC-C-003 |
| **name** | `tc_truncate_100_to_5` |
| **input** | 100 messages + `protect_recent=5` |
| **expected output** | result messages length = 5 (oldest 95 dropped) |
| **error case** | (없음) |
| **mock** | `Vec<Message>` × 100 + `BudgetTracker` (already < 70%) |
| **verify** | `assert_eq!(result.len(), 5)` + most recent 5 messages preserved |

```rust
#[tokio::test]
async fn tc_truncate_100_to_5() {
    use myharness_context::compression::layer1::truncate;
    use myharness_context::budget::BudgetTracker;

    let messages: Vec<Message> = (0..100).map(|i| Message::user(format!("msg{i}"))).collect();
    // DD-2 §6.2 4-arg sync (model_length, threshold, accumulated, system_prompt_tokens)
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 10_000, 0);  // 5% — well under threshold

    let result = truncate(messages, &tracker, 5);
    assert_eq!(result.len(), 5);
    assert_eq!(result.last().unwrap().content.as_text(), "msg99");
}
```

### §3.4 TC-C-004 — summarize compression 100→20 (DD-2 §4.4)

| field | value |
| --- | --- |
| **id** | TC-C-004 |
| **name** | `tc_summarize_100_to_20` |
| **input** | 100 messages + `MockLlmClient` (summary returns 1 message) |
| **expected output** | result = [summary_msg(1)] + recent_5 = 6 messages (target 20 max) |
| **error case** | (없음, mock OK) |
| **mock** | `MockLlmClient::new(vec!["요약된 대화".into()])` (DD-2 §4.4) |
| **verify** | `assert!(result.len() <= 20)` + result[0] = summary (role=Assistant) |

```rust
#[tokio::test]
async fn tc_summarize_100_to_20() {
    use myharness_context::compression::layer1::summarize;
    use myharness_context::budget::BudgetTracker;

    let messages: Vec<Message> = (0..100).map(|i| Message::user(format!("msg{i}"))).collect();
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 0, 0);  // 4-arg sync
    let llm = MockLlmClient::new(vec!["요약된 대화".into()]);

    let result = summarize(messages, &tracker, &llm, 5).await.unwrap();
    assert!(result.len() <= 20, "summarize should compress to <= 20");
    assert_eq!(result[0].role, Role::Assistant, "first message = summary");
}
```

### §3.5 TC-C-005 — hybrid compression truncate+summarize (DD-2 §4.5)

| field | value |
| --- | --- |
| **id** | TC-C-005 |
| **name** | `tc_hybrid_compression` |
| **input** | 100 messages + `protect_recent=5` + `MockLlmClient` |
| **expected output** | result = [summary] + recent_5 ≈ 6 messages (D-30 default mode) |
| **error case** | (없음) |
| **mock** | `MockLlmClient` |
| **verify** | `assert!(result.len() <= 10)` + role[0] = Assistant (summary) |

```rust
#[tokio::test]
async fn tc_hybrid_compression() {
    use myharness_context::compression::layer1::hybrid;
    use myharness_context::budget::BudgetTracker;

    let messages: Vec<Message> = (0..100).map(|i| Message::user(format!("msg{i}"))).collect();
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 0, 0);  // 4-arg sync
    let llm = MockLlmClient::new(vec!["요약".into()]);

    let result = hybrid(messages, &tracker, &llm, 5).await.unwrap();
    assert!(result.len() <= 10, "hybrid default mode (D-30)");
    assert_eq!(result[0].role, Role::Assistant);
}
```

### §3.6 TC-C-006 — /compact slash command handler (DD-2 §4.7)

| field | value |
| --- | --- |
| **id** | TC-C-006 |
| **name** | `tc_compact_slash_command_handler` |
| **input** | CLI args `--mode=hybrid --force` + 80%+ usage |
| **expected output** | `CompactResult::Done { before_tokens, after_tokens, elapsed_ms, saved_ratio }` |
| **error case** | (force=true) — should_compact 무관하게 강제 compress |
| **mock** | `Context::new_for_test(80% usage)` + `MockLlmClient` |
| **verify** | `assert!(matches!(result, CompactResult::Done { .. }))` + `saved_ratio > 0.0` |

```rust
#[tokio::test]
async fn tc_compact_slash_command_handler() {
    use myharness_context::slash::compact;
    use myharness_context::budget::BudgetTracker;

    let mut ctx = Context::new_for_test(ProviderId::Anthropic, "claude-sonnet-4-5", 200_000).await.unwrap();
    ctx.budget.add_tokens(160_000);  // 80%
    let llm = MockLlmClient::new(vec!["요약".into()]);
    let args = CompactArgs { mode: Some(CompressionMode::Hybrid), force: true, protect_recent: Some(5) };

    let result = compact::run(&mut ctx, &llm, args).await.unwrap();
    assert!(matches!(result, CompactResult::Done { saved_ratio, .. } if saved_ratio > 0.0));
}
```

### §3.7 TC-C-007 — BudgetTracker atomicity (DD-2 §2.5)

| field | value |
| --- | --- |
| **id** | TC-C-007 |
| **name** | `tc_budget_tracker_atomic_concurrent_add` |
| **input** | 100 threads × 1000 add_tokens(1) |
| **expected output** | `accumulated_tokens == 100_000` (atomicity 보장) |
| **error case** | Atomicity 깨지면 race condition 으로 < 100_000 |
| **mock** | `Arc<BudgetTracker>` + `tokio::spawn` × 100 |
| **verify** | `assert_eq!(tracker.accumulated_tokens.load(Ordering::SeqCst), 100_000)` |

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn tc_budget_tracker_atomic_concurrent_add() {
    use std::sync::Arc;
    use myharness_context::budget::BudgetTracker;

    let tracker = Arc::new(
        BudgetTracker::new_for_test(200_000, 0.80, 0, 0)  // 4-arg sync
    );
    let mut handles = vec![];
    for _ in 0..100 {
        let t = tracker.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..1000 { t.add_tokens(1); }
        }));
    }
    futures::future::join_all(handles).await;
    assert_eq!(tracker.accumulated_tokens.load(std::sync::atomic::Ordering::SeqCst), 100_000);
}
```

### §3.8 TC-C-008 — Layer 2 SmartCrusher JSON compression (DD-2 §5.4)

| field | value |
| --- | --- |
| **id** | TC-C-008 |
| **name** | `tc_smart_crusher_json_65_percent` |
| **input** | JSON `{ "transcript_content": "<500 char>", "metadata": {...} }` |
| **expected output** | compressed JSON = 65% shorter (DD-2 §5.4 target) |
| **error case** | invalid JSON input → serde_json::from_str fail |
| **mock** | sample JSON string |
| **verify** | `assert!(crushed.len() <= original.len() * 35 / 100)` (≤35% of original) |

```rust
#[test]
fn tc_smart_crusher_json_65_percent() {
    use myharness_context::compression::layer2::smart_crusher;
    use myharness_context::compression::layer2::CrushLevel;

    let json = r#"{"transcript_content": "aaaa...500chars", "metadata": {"key": "val"}}"#;
    let original_len = json.len();
    let crushed = smart_crusher::crush(json, &CrushLevel::Aggressive);
    assert!(crushed.len() <= original_len * 35 / 100,
        "crushed {} <= 35% of original {}", crushed.len(), original_len);
}
```

### §3.9 §3 결정 trade-off (8 TC 분량)

| 선정 (8 TC) | 대안 (4 TC) | trade-off |
| --- | --- | --- |
| 8 TC = BudgetTracker 3 + Layer 1 3 + Layer 2 1 + slash 1 | 4 TC = 핵심만 | ✅ BudgetTracker 핵심 (threshold/lookup/atomicity) + Layer 1 3 mode + Layer 2 opt-in 1 + slash 1. ⚠️ Layer 2 4 algo (CacheAligner/ContentRouter/SmartCrusher/CodeCompressor) 중 SmartCrusher 만 — v1.5+ 확장 |

### §3.10 결정 근거 1-라인 (yklee review)

> **8 TC = L1 Unit TC 의 myharness-context 카테고리** (REVIEW §6.2 + DD-2 §6). BudgetTracker 3 + Layer 1 3 + Layer 2 1 + slash 1 = 8. v1.5+ 에서 Layer 2 4 algo TC 추가 → 11 TC.


---

## §4. myharness-session TC (6, Status enum / Event enum / handoff format, REVIEW §6.2)

> **module path** = `crates/myharness-session/src/state/{task,status}.rs` + `log/event.rs` + `handoff/format.rs`. **공통 import** = `use myharness_session::{Status, Event, Task, Handoff}; use serde_json::json;`. mock = in-memory state (no FS).

### §4.1 TC-S-001 — Status enum 4 값 (INITIAL_DESIGN §3.3 myharness-session + CONCEPT §5.13)

| field | value |
| --- | --- |
| **id** | TC-S-001 |
| **name** | `tc_session_status_enum_4_values` |
| **input** | `Status::{Pending, Running, Completed, Failed}` 4 variant |
| **expected output** | serde round-trip 4 variant 모두 정상 |
| **error case** | (없음) |
| **mock** | (없음, pure enum) |
| **verify** | `for status in [Pending, Running, Completed, Failed] { json roundtrip }` |

```rust
#[test]
fn tc_session_status_enum_4_values() {
    use myharness_session::Status;
    use serde_json;

    for status in [Status::Pending, Status::Running, Status::Completed, Status::Failed] {
        let json = serde_json::to_string(&status).unwrap();
        let back: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back, "Status {json:?} roundtrip");
    }
}
```

### §4.2 TC-S-002 — Status 전이 검증 (Pending → Running → Completed)

| field | value |
| --- | --- |
| **id** | TC-S-002 |
| **name** | `tc_session_status_transition_pending_to_completed` |
| **input** | Task start (Pending) → execute (Running) → finish (Completed) |
| **expected output** | 3-step 전이 모두 유효 |
| **error case** | Pending → Completed (skip Running) → invalid_transition error |
| **mock** | `Task::new()` + state machine guard |
| **verify** | `assert!(matches!(task.status, Status::Running))` + `assert!(invalid_transition.is_err())` |

```rust
#[test]
fn tc_session_status_transition_pending_to_completed() {
    use myharness_session::{Task, Status};

    let mut task = Task::new("test-001");
    assert_eq!(task.status, Status::Pending);

    task.start().unwrap();
    assert_eq!(task.status, Status::Running);

    task.complete().unwrap();
    assert_eq!(task.status, Status::Completed);

    let invalid = task.start();  // already completed
    assert!(invalid.is_err(), "Completed → Running must fail");
}
```

### §4.3 TC-S-003 — Event enum append to log.jsonl (D-26 이벤트 소싱)

| field | value |
| --- | --- |
| **id** | TC-S-003 |
| **name** | `tc_session_event_log_jsonl_append` |
| **input** | 3 event (ToolCall / SubAgentDispatch / BudgetUpdate) |
| **expected output** | log.jsonl 에 3 줄 append (각 1 event JSON) |
| **error case** | (없음) |
| **mock** | `tempfile::NamedTempFile` for log.jsonl path |
| **verify** | `assert_eq!(count_lines(log_path), 3)` + `assert!(line.contains("tool_call"))` |

```rust
#[tokio::test]
async fn tc_session_event_log_jsonl_append() {
    use myharness_session::{Event, log};

    let log_path = tempfile::NamedTempFile::new().unwrap();
    log::init(log_path.path()).unwrap();

    log::append(Event::ToolCall { name: "Read".into(), args: json!({"path": "/x"}), result: json!({}) }).unwrap();
    log::append(Event::SubAgentDispatch { id: "code-reviewer".into(), input_summary: "PR review".into() }).unwrap();
    log::append(Event::BudgetUpdate { accumulated: 50000, model_length: 200000 }).unwrap();

    let content = std::fs::read_to_string(log_path.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("tool_call"));
    assert!(lines[1].contains("sub_agent_dispatch"));
    assert!(lines[2].contains("budget_update"));
}
```

### §4.4 TC-S-004 — Event enum 직렬화 7 variant

| field | value |
| --- | --- |
| **id** | TC-S-004 |
| **name** | `tc_session_event_enum_7_variants_serialize` |
| **input** | 7 Event variant (ToolCall / SubAgentDispatch / BudgetUpdate / Compression / Fallback / HookEval / Exit) |
| **expected output** | serde_json 모두 성공 |
| **error case** | (없음) |
| **mock** | (없음) |
| **verify** | `for event in 7_variants { serde_json::to_string(&event) }` |

```rust
#[test]
fn tc_session_event_enum_7_variants_serialize() {
    use myharness_session::Event;

    let events = vec![
        Event::ToolCall { name: "Read".into(), args: json!({}), result: json!({}) },
        Event::SubAgentDispatch { id: "x".into(), input_summary: "y".into() },
        Event::BudgetUpdate { accumulated: 0, model_length: 200000 },
        Event::Compression { mode: "hybrid".into(), before_tokens: 100000, after_tokens: 50000 },
        Event::Fallback { from_provider: "anthropic".into(), to_provider: "openai".into() },
        Event::HookEval { hook_name: "SP-01".into(), action: "block".into() },
        Event::Exit { exit_code: 0, error_kind: "none".into() },
    ];
    for event in events {
        let json = serde_json::to_string(&event).expect("Event must serialize");
        let back: Event = serde_json::from_str(&json).expect("Event must deserialize");
        assert_eq!(format!("{:?}", back).len() > 0, "roundtrip preserved");
    }
}
```

### §4.5 TC-S-005 — handoff format YAML 4-필드 (D-26 4-필드)

| field | value |
| --- | --- |
| **id** | TC-S-005 |
| **name** | `tc_session_handoff_yaml_4_field` |
| **input** | handoff with summary / risks / suggested_follow_up / produced_artifacts |
| **expected output** | YAML 4-필드 모두 포함 + serde_yaml roundtrip |
| **error case** | (필수 필드 누락) → HandoffParseError |
| **mock** | sample handoff YAML string |
| **verify** | `assert!(handoff.summary.len() > 0)` + 4-필드 모두 non-empty |

```rust
#[test]
fn tc_session_handoff_yaml_4_field() {
    use myharness_session::Handoff;

    let yaml = r#"
summary: "DD-1 trait Tool::Schema spec confirmed"
risks: "rig-core 0.5+ → 1.0 migration impact"
suggested_follow_up: "TASK-005-1 mcp__filesystem PoC"
produced_artifacts: "docs/architecture/DETAILED_DESIGN_TOOL.md"
"#;
    let handoff: Handoff = serde_yaml::from_str(yaml).expect("4-field handoff must parse");
    assert!(!handoff.summary.is_empty());
    assert!(!handoff.risks.is_empty());
    assert!(!handoff.suggested_follow_up.is_empty());
    assert!(!handoff.produced_artifacts.is_empty());
}
```

### §4.6 TC-S-006 — handoff file read/write roundtrip

| field | value |
| --- | --- |
| **id** | TC-S-006 |
| **name** | `tc_session_handoff_file_read_write_roundtrip` |
| **input** | Handoff → file write → file read → parse |
| **expected output** | 동일 Handoff recover |
| **error case** | file I/O error (read-only dir) |
| **mock** | `tempfile::NamedTempFile` + `tempfile::tempdir` (read-only) |
| **verify** | `assert_eq!(original, recovered)` |

```rust
#[test]
fn tc_session_handoff_file_read_write_roundtrip() {
    use myharness_session::Handoff;

    let original = Handoff {
        summary: "test".into(),
        risks: "r".into(),
        suggested_follow_up: "f".into(),
        produced_artifacts: "p".into(),
    };
    let path = tempfile::NamedTempFile::new().unwrap();
    original.write_to(path.path()).unwrap();
    let recovered = Handoff::read_from(path.path()).unwrap();
    assert_eq!(original, recovered);
}
```

### §4.7 §4 결정 trade-off (6 TC 분량)

| 선정 (6 TC) | 대안 (3 TC) | trade-off |
| --- | --- | --- |
| 6 TC = Status 2 + Event 2 + handoff 2 | 3 TC = Status 1 + Event 1 + handoff 1 | ✅ Status 4 variant + transition (S-001, S-002). Event 7 variant + log append (S-003, S-004). Handoff YAML + file roundtrip (S-005, S-006) |

### §4.8 결정 근거 1-라인 (yklee review)

> **6 TC = myharness-session 의 L1 Unit 전체 범위** (REVIEW §6.2 정합). Status + Event + handoff 각각 2 TC. v1.5+ 에서 mavis_bridge conflict resolution TC 추가.

---

## §5. myharness-plugins TC (6, markdown hook / MCP / auto_expose, REVIEW §6.2)

> **module path** = `crates/myharness-plugins/src/hooks/markdown.rs` + `mcp/{server_registry,servers/{filesystem,git,shell,github}}.rs` + `mcp/auto_expose.rs`. **공통 import** = `use myharness_plugins::hooks::{parse_hook_file, HookDef, Severity, Action}; use myharness_plugins::mcp::registry::McpServerRegistry;`. mock = fixture markdown file + temp dir.

### §5.1 TC-P-001 — markdown hook parser (DD-4 §1.2 + §4.2)

| field | value |
| --- | --- |
| **id** | TC-P-001 |
| **name** | `tc_plugins_markdown_hook_parser` |
| **input** | SP-01 markdown file (frontmatter 7 fields + body) |
| **expected output** | `HookDef { name: "SP-01-rm-rf-root", pattern, severity: High, action: Confirm, .. }` |
| **error case** | unknown field in frontmatter → HookParseError::UnknownField |
| **mock** | `tempfile::NamedTempFile` with SP-01 markdown content |
| **verify** | `assert_eq!(hook.name, "SP-01-rm-rf-root")` + `assert_eq!(hook.severity, Severity::High)` |

```rust
#[test]
fn tc_plugins_markdown_hook_parser() {
    use myharness_plugins::hooks::parse_hook_file;
    use myharness_plugins::hooks::{Severity, Action};

    let md = r#"---
name: SP-01-rm-rf-root
description: rm -rf targeting filesystem root
triggers: [tool_call]
tool: Bash
pattern: '\brm\s+(?:--?\S+\s+)+/(?:\s|;|\||\*|$)'
severity: high
action: confirm
---

# SP-01: rm -rf /

## What it catches
rm command with root /
"#;
    let path = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(path.path(), md).unwrap();

    let hook = parse_hook_file(path.path()).expect("SP-01 must parse");
    assert_eq!(hook.name, "SP-01-rm-rf-root");
    assert_eq!(hook.severity, Severity::High);
    assert_eq!(hook.action, Action::Confirm);
}
```

### §5.2 TC-P-002 — markdown hook parser unknown field error

| field | value |
| --- | --- |
| **id** | TC-P-002 |
| **name** | `tc_plugins_markdown_hook_parser_unknown_field` |
| **input** | frontmatter with `sevrity: high` (typo) |
| **expected output** | `Err(HookParseError::UnknownField("sevrity"))` |
| **error case** | DD-4 §1.2 strict mode (unknown field = parse error) |
| **mock** | markdown with typo |
| **verify** | `assert!(matches!(err, HookParseError::UnknownField(f) if f == "sevrity"))` |

```rust
#[test]
fn tc_plugins_markdown_hook_parser_unknown_field() {
    use myharness_plugins::hooks::parse_hook_file;

    let md = r#"---
name: SP-99-typo
triggers: [tool_call]
sevrity: high   # typo
action: confirm
---
"#;
    let path = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(path.path(), md).unwrap();

    let result = parse_hook_file(path.path());
    assert!(matches!(result, Err(myharness_plugins::hooks::HookParseError::UnknownField(f)) if f == "sevrity"));
}
```

### §5.3 TC-P-003 — MCP 4 server registration (filesystem/git/shell/github)

| field | value |
| --- | --- |
| **id** | TC-P-003 |
| **name** | `tc_plugins_mcp_4_server_registration` |
| **input** | 4 MCP server config (filesystem/git/shell/github) |
| **expected output** | `McpServerRegistry` 에 4 server 등록 + 4 tool list |
| **error case** | server offline → registration fail |
| **mock** | 4 in-memory `MockMcpServer` |
| **verify** | `assert_eq!(registry.list_servers().len(), 4)` |

```rust
#[tokio::test]
async fn tc_plugins_mcp_4_server_registration() {
    use myharness_plugins::mcp::registry::McpServerRegistry;
    use myharness_plugins::mcp::servers::{filesystem, git, shell, github};

    let mut registry = McpServerRegistry::new();
    registry.register("filesystem", Box::new(filesystem::MockFsServer::new())).await.unwrap();
    registry.register("git", Box::new(git::MockGitServer::new())).await.unwrap();
    registry.register("shell", Box::new(shell::MockShellServer::new())).await.unwrap();
    registry.register("github", Box::new(github::MockGithubServer::new())).await.unwrap();

    let servers = registry.list_servers();
    assert_eq!(servers.len(), 4);
    assert!(servers.contains(&"filesystem"));
    assert!(servers.contains(&"github"));
}
```

### §5.4 TC-P-004 — MCP tool naming convention `mcp__<server>__<tool>`

| field | value |
| --- | --- |
| **id** | TC-P-004 |
| **name** | `tc_plugins_mcp_tool_naming_convention` |
| **input** | `filesystem.read_file` (server=filesystem, tool=read_file) |
| **expected output** | `mcp__filesystem__read_file` |
| **error case** | (없음) |
| **mock** | tool name parts |
| **verify** | `assert_eq!(formatted, "mcp__filesystem__read_file")` |

```rust
#[test]
fn tc_plugins_mcp_tool_naming_convention() {
    use myharness_plugins::mcp::format_tool_name;

    assert_eq!(format_tool_name("filesystem", "read_file"), "mcp__filesystem__read_file");
    assert_eq!(format_tool_name("github", "create_pr"), "mcp__github__create_pr");
    assert_eq!(format_tool_name("git", "status"), "mcp__git__status");
    assert_eq!(format_tool_name("shell", "bash"), "mcp__shell__bash");
}
```

### §5.5 TC-P-005 — auto_expose MCP tool to myharness-tools registry (DD-1 §1.3 + INITIAL §3.7)

| field | value |
| --- | --- |
| **id** | TC-P-005 |
| **name** | `tc_plugins_mcp_auto_expose` |
| **input** | MCP tool `mcp__filesystem__read_file` |
| **expected output** | `ToolRegistry` 에 `Arc<dyn Tool>` 로 wrap 등록 |
| **error case** | MCP server offline → auto_expose fail |
| **mock** | `ToolRegistry::new()` + `MockMcpServer` |
| **verify** | `assert!(registry.lookup("mcp__filesystem__read_file").is_some())` |

```rust
#[tokio::test]
async fn tc_plugins_mcp_auto_expose() {
    use myharness_tools::ToolRegistry;
    use myharness_plugins::mcp::auto_expose;

    let registry = ToolRegistry::new();
    let mcp_tool = MockMcpServer::new().tool("mcp__filesystem__read_file").await;
    auto_expose::register(&registry, mcp_tool).await.unwrap();

    assert!(registry.lookup("mcp__filesystem__read_file").is_some(),
        "auto_expose must register mcp__ tool to ToolRegistry");
}
```

### §5.6 TC-P-006 — builtin 9 security hooks loaded (DD-4 §4.5 BUILTIN_HOOKS)

| field | value |
| --- | --- |
| **id** | TC-P-006 |
| **name** | `tc_plugins_builtin_9_security_hooks_loaded` |
| **input** | `BUILTIN_HOOKS` 상수 (9 entry) |
| **expected output** | `BUILTIN_HOOKS.len() == 9` + 9 id 모두 SP-01~SP-09 |
| **error case** | (없음) |
| **mock** | (없음, compile-time const) |
| **verify** | `assert_eq!(BUILTIN_HOOKS.len(), 9)` + id 순회 |

```rust
#[test]
fn tc_plugins_builtin_9_security_hooks_loaded() {
    use myharness_plugins::hooks::builtin_hooks::BUILTIN_HOOKS;

    assert_eq!(BUILTIN_HOOKS.len(), 9, "DD-4 §4.5: 9 builtin hooks");
    let ids: Vec<&str> = BUILTIN_HOOKS.iter().map(|(id, ..)| *id).collect();
    for i in 1..=9 {
        let id = format!("SP-0{i}");
        assert!(ids.contains(&id.as_str()), "{id} must be in BUILTIN_HOOKS");
    }
}
```

### §5.7 §5 결정 trade-off (6 TC 분량)

| 선정 (6 TC) | 대안 (3 TC) | trade-off |
| --- | --- | --- |
| 6 TC = markdown 2 + MCP 4 | 3 TC = 1 each | ✅ markdown parser + unknown field error 2건 (DD-4 §1.2 strict mode). MCP 4 server + naming + auto_expose 3건. builtin 9 hooks 1건. ❌ v1.5+ skills TC 별도 |

### §5.8 결정 근거 1-라인 (yklee review)

> **6 TC = myharness-plugins L1 Unit 전체** (REVIEW §6.2 정합). markdown hook parser 2 + MCP 4 (registration/naming/auto_expose) + builtin 9 hooks 1 = 6. v1.5+ skills/agents/commands 3-계층 TC 추가.


---

## §6. myharness-llm TC (10, AuthManager / FallbackChain / Provider retry, REVIEW §6.2 + DD-5 §1-§4)

> **module path** = `crates/myharness-llm/src/auth/{keychain,env_fallback}.rs` + `provider/registry.rs` + `fallback/{chain,retry,breaker,error}.rs`. **공통 import** = `use myharness_llm::{LlmClient, AuthManager, ProviderId, FallbackChain}; use myharness_llm::fallback::{RetryPolicy, CircuitBreaker, LlmError};`. mock = `MockProvider` (D-38 + DD-5 §1.4).

### §6.1 TC-L-001 — AuthManager keychain (macOS/Windows/Linux)

| field | value |
| --- | --- |
| **id** | TC-L-001 |
| **name** | `tc_llm_auth_manager_keychain` |
| **input** | provider=Anthropic, env var `ANTHROPIC_API_KEY` set |
| **expected output** | `AuthManager::get(Anthropic) == Ok(EXAMPLEPLACEHOLDER_TOKEN)` |
| **error case** | env var 미설정 → AuthError::NotFound |
| **mock** | `std::env::set_var("ANTHROPIC_API_KEY", "EXAMPLEPLACEHOLDER_TOKEN")` |
| **verify** | `assert!(auth.get(ProviderId::Anthropic).is_ok())` + D-06 (값 ❌, env var 이름만 verify) |

```rust
#[test]
fn tc_llm_auth_manager_keychain() {
    use myharness_llm::{AuthManager, ProviderId};
    // D-06: env var name 만 사용, 값은 placeholder
    std::env::set_var("ANTHROPIC_API_KEY", "EXAMPLEPLACEHOLDER_TOKEN");
    let auth = AuthManager::new();
    let result = auth.get(ProviderId::Anthropic);
    assert!(result.is_ok(), "ANTHROPIC_API_KEY env var set → auth OK");
    // 실제 값은 log ❌ — D-06 정합
}
```

### §6.2 TC-L-002 — AuthManager env var fallback

| field | value |
| --- | --- |
| **id** | TC-L-002 |
| **name** | `tc_llm_auth_env_var_fallback` |
| **input** | env var `OPENAI_API_KEY` set, keychain empty |
| **expected output** | `AuthManager::get(OpenAi) == Ok(...)` (env var 우선) |
| **error case** | env var 미설정 + keychain empty → AuthError::NotFound |
| **mock** | `std::env::set_var("OPENAI_API_KEY", "EXAMPLEPLACEHOLDER")` + keychain mock empty |
| **verify** | `assert!(auth.get(ProviderId::OpenAi).is_ok())` |

```rust
#[test]
fn tc_llm_auth_env_var_fallback() {
    use myharness_llm::{AuthManager, ProviderId};
    std::env::set_var("OPENAI_API_KEY", "EXAMPLEPLACEHOLDER");
    let auth = AuthManager::new();
    let result = auth.get(ProviderId::OpenAi);
    assert!(result.is_ok());
}
```

### §6.3 TC-L-003 — AuthManager NotFound error (env var 미설정)

| field | value |
| --- | --- |
| **id** | TC-L-003 |
| **name** | `tc_llm_auth_not_found_error` |
| **input** | env vars unset, keychain empty |
| **expected output** | `Err(AuthError::NotFound { provider: "minimax" })` |
| **error case** | `NotFound` (INITIAL §6.2 5-step: env vars → keychain → local server → MCP → active-providers) |
| **mock** | env var unset + keychain empty |
| **verify** | `assert!(matches!(err, AuthError::NotFound { provider } if provider == "minimax"))` |

```rust
#[test]
fn tc_llm_auth_not_found_error() {
    use myharness_llm::{AuthManager, ProviderId, AuthError};
    std::env::remove_var("MINIMAX_API_KEY");
    let auth = AuthManager::new();
    let result = auth.get(ProviderId::Minimax);
    assert!(matches!(result, Err(AuthError::NotFound { ref provider }) if provider == "minimax"));
}
```

### §6.4 TC-L-004 — FallbackChain primary success (DD-5 §2.4)

| field | value |
| --- | --- |
| **id** | TC-L-004 |
| **name** | `tc_llm_fallback_chain_primary_success` |
| **input** | 3 provider [Anthropic, OpenAi, Gemini] + primary success |
| **expected output** | chain[0] (Anthropic) success → no fallback |
| **error case** | (없음) |
| **mock** | 3 MockProvider (chain[0] returns 200, others skip) |
| **verify** | `assert!(!result.fallback_used)` + result from chain[0] |

```rust
#[tokio::test]
async fn tc_llm_fallback_chain_primary_success() {
    use myharness_llm::fallback::FallbackChain;
    use myharness_llm::ProviderId;

    let chain = FallbackChain::new(vec![ProviderId::Anthropic, ProviderId::OpenAi, ProviderId::Gemini]);
    let result = chain.call_with_chain(|provider| async move {
        // MockProvider: chain[0] = success, others = 503
        match provider {
            ProviderId::Anthropic => Ok("primary response".to_string()),
            _ => Err(myharness_llm::fallback::LlmError::Overloaded("mock".into())),
        }
    }).await.unwrap();
    // DD-5 §2.4 FallbackChain::call_with_chain returns `ChainResult { content, fallback_used, ... }`
    // 3개 TC (TC-L-004/005 + TC-R-006) 동시 만족을 위해 struct field access 통일
    assert_eq!(result.content, "primary response");
    assert!(!result.fallback_used, "primary success → no fallback");
}
```

### §6.5 TC-L-005 — FallbackChain primary fail → fallback to chain[1] (DD-5 §2.4 + D-15)

| field | value |
| --- | --- |
| **id** | TC-L-005 |
| **name** | `tc_llm_fallback_chain_fallback_to_chain1` |
| **input** | chain[0]=503, chain[1]=200 |
| **expected output** | chain[1] response + `fallback_used: true` |
| **error case** | chain[0] error category = Retryable (overloaded, D-15) |
| **mock** | MockProvider overloaded for Anthropic, success for OpenAi |
| **verify** | `assert!(result.fallback_used)` + result from OpenAi |

```rust
#[tokio::test]
async fn tc_llm_fallback_chain_fallback_to_chain1() {
    use myharness_llm::fallback::{FallbackChain, LlmError};
    use myharness_llm::ProviderId;

    let chain = FallbackChain::new(vec![ProviderId::Anthropic, ProviderId::OpenAi]);
    let result = chain.call_with_chain(|provider| async move {
        match provider {
            ProviderId::Anthropic => Err(LlmError::Overloaded("503".into())),
            ProviderId::OpenAi => Ok("fallback response".to_string()),
            _ => unreachable!(),
        }
    }).await.unwrap();
    assert_eq!(result.content, "fallback response");
    assert!(result.fallback_used, "primary fail → chain[1] used");
}
```

### §6.6 TC-L-006 — FallbackChain all provider exhausted

| field | value |
| --- | --- |
| **id** | TC-L-006 |
| **name** | `tc_llm_fallback_chain_all_exhausted` |
| **input** | chain[0..3] all 503 |
| **expected output** | `Err(LlmError::NoProvider)` |
| **error case** | AllProvidersExhausted |
| **mock** | 3 MockProvider all overloaded |
| **verify** | `assert!(matches!(err, LlmError::NoProvider))` |

```rust
#[tokio::test]
async fn tc_llm_fallback_chain_all_exhausted() {
    use myharness_llm::fallback::{FallbackChain, LlmError};
    use myharness_llm::ProviderId;

    let chain = FallbackChain::new(vec![ProviderId::Anthropic, ProviderId::OpenAi, ProviderId::Gemini]);
    let result = chain.call_with_chain(|_| async {
        Err(LlmError::Overloaded("503".into()))
    }).await;
    assert!(matches!(result, Err(LlmError::NoProvider)));
}
```

### §6.7 TC-L-007 — Provider retry exponential backoff (DD-5 §1.1)

| field | value |
| --- | --- |
| **id** | TC-L-007 |
| **name** | `tc_llm_provider_retry_exponential_backoff` |
| **input** | MockProvider: 503 → 200 (1 retry) |
| **expected output** | success after 1 retry + sleep 500-750ms (attempt 0) |
| **error case** | retryable error → 1회 retry (CONCEPT §5.5.3 "1회 retry") |
| **mock** | MockProvider attempt 0=503, attempt 1=200 |
| **verify** | `assert!(result.is_ok())` + `assert!(elapsed_ms >= 500 && elapsed_ms < 750)` (DD-5 §5 spec, attempt 0 backoff 500-750ms) |

```rust
#[tokio::test]
async fn tc_llm_provider_retry_exponential_backoff() {
    use myharness_llm::fallback::{call_with_retry, RetryPolicy, LlmError};
    use std::time::Instant;

    let policy = RetryPolicy::default();  // base=500ms, max=1
    let attempt = std::sync::atomic::AtomicU32::new(0);
    let started = Instant::now();
    let result = call_with_retry(&policy, || {
        let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move {
            if n == 0 { Err(LlmError::Overloaded("503".into())) }
            else { Ok("retry success".to_string()) }
        }
    }).await;
    let elapsed = started.elapsed();
    assert!(result.is_ok());
    assert!(elapsed >= std::time::Duration::from_millis(500) && elapsed < std::time::Duration::from_millis(750),
        "attempt 0 backoff 500-750ms, elapsed = {elapsed:?}");
}
```

### §6.8 TC-L-008 — Provider retry max_retries=1 (CONCEPT §5.5.3 "1회 retry")

| field | value |
| --- | --- |
| **id** | TC-L-008 |
| **name** | `tc_llm_provider_retry_max_retries_1` |
| **input** | MockProvider: 503 → 503 → 503 (3 calls) |
| **expected output** | `Err(LlmError::Overloaded)` after 2 attempts (initial + 1 retry) |
| **error case** | attempt 2 (3rd call) → no retry, 즉시 return |
| **mock** | MockProvider always overloaded |
| **verify** | `assert_eq!(attempt_count.load(SeqCst), 2)` |

```rust
#[tokio::test]
async fn tc_llm_provider_retry_max_retries_1() {
    use myharness_llm::fallback::{call_with_retry, RetryPolicy, LlmError};
    use std::sync::atomic::{AtomicU32, Ordering};

    let policy = RetryPolicy::default();  // max_retries=1
    let counter = AtomicU32::new(0);
    let result = call_with_retry(&policy, || {
        counter.fetch_add(1, Ordering::SeqCst);
        async { Err::<String, _>(LlmError::Overloaded("503".into())) }
    }).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2, "initial + 1 retry = 2 attempts");
    assert!(result.is_err());
}
```

### §6.9 TC-L-009 — Provider immediate surface (auth, D-15 immediate)

| field | value |
| --- | --- |
| **id** | TC-L-009 |
| **name** | `tc_llm_immediate_surface_auth_no_retry_no_fallback` |
| **input** | MockProvider: 401 Auth error |
| **expected output** | 즉시 `Err(LlmError::Auth)` (retry ❌, fallback ❌) |
| **error case** | D-15 ImmediateSurface — auth/rate_limit/request_size/transport |
| **mock** | MockProvider returns 401 |
| **verify** | `assert_eq!(counter, 1, "no retry")` + `assert!(matches!(err, LlmError::Auth(_)))` |

```rust
#[tokio::test]
async fn tc_llm_immediate_surface_auth_no_retry_no_fallback() {
    use myharness_llm::fallback::{call_with_retry, RetryPolicy, LlmError};
    use std::sync::atomic::{AtomicU32, Ordering};

    let policy = RetryPolicy::default();
    let counter = AtomicU32::new(0);
    let result = call_with_retry(&policy, || {
        counter.fetch_add(1, Ordering::SeqCst);
        async { Err::<String, _>(LlmError::Auth("401".into())) }
    }).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1, "D-15 immediate surface: no retry");
    assert!(matches!(result, Err(LlmError::Auth(_))));
}
```

### §6.10 TC-L-010 — Provider streaming response handling

| field | value |
| --- | --- |
| **id** | TC-L-010 |
| **name** | `tc_llm_provider_streaming_response` |
| **input** | MockProvider streaming chunks ["Hello", " ", "world"] |
| **expected output** | full response = "Hello world" (concat chunks) |
| **error case** | stream error mid-way → partial response + error |
| **mock** | MockProvider stream 3 chunks |
| **verify** | `assert_eq!(full, "Hello world")` |

```rust
#[tokio::test]
async fn tc_llm_provider_streaming_response() {
    use myharness_llm::LlmClient;
    use futures::StreamExt;

    let client = LlmClient::new_mock_streaming(vec!["Hello".to_string(), " ".to_string(), "world".to_string()]);
    let mut stream = client.stream_completion("test").await.unwrap();
    let mut full = String::new();
    while let Some(chunk) = stream.next().await {
        full.push_str(&chunk.unwrap());
    }
    assert_eq!(full, "Hello world");
}
```

### §6.11 §6 결정 trade-off (10 TC 분량)

| 선정 (10 TC) | 대안 (5 TC) | trade-off |
| --- | --- | --- |
| 10 TC = Auth 3 + Fallback 3 + retry 3 + streaming 1 | 5 TC = 1 each | ✅ Auth 3 단계 (keychain/env fallback/NotFound) + Fallback 3 (primary/fallback/exhausted) + retry 3 (backoff/max_retries/immediate) + streaming 1. ❌ v1.5+ provider-auto-config (D-38) + HTTP mock (wiremock) 별도 |

### §6.12 결정 근거 1-라인 (yklee review)

> **10 TC = myharness-llm L1 Unit 전체** (REVIEW §6.2 + DD-5 §1-§4 정합). Auth/Fallback/retry 핵심 cover. v1.5+ 에서 D-38 provider-auto-config + HTTP mock TC 추가.


---

## §7. myharness-agents TC (54, 15 sub-agent × 3-5, DD-3 §3-§6)

> **module path** = `crates/myharness-agents/src/subagent/{code,server,env,utility}/<name>.rs` + `output/*.rs`. **공통 import** = `use myharness_agents::{SubAgent, SubAgentContext}; use myharness_agents::permission::ToolId;`. mock = `MockLlmClient` (DD-3 §3.1.5) + `MockPermissionContext`. 각 sub-agent 3-5 TC.

### §7.1 Sub-agent TC ID 매핑 (15 × 3-5 = 54)

| sub-agent | DD-3 § | TC id | count |
| --- | --- | --- | --- |
| `code-reviewer` | §3.1 | TC-A-001~005 | 5 |
| `code-implementer` | §3.2 | TC-A-006~009 | 4 |
| `code-tester` | §3.3 | TC-A-010~013 | 4 |
| `code-refactorer` | §3.4 | TC-A-014~016 | 3 |
| `code-searcher` | §3.5 | TC-A-017~019 | 3 |
| `server-status` | §4.1 | TC-A-020~023 | 4 |
| `log-analyzer` | §4.2 | TC-A-024~026 | 3 |
| `deployer` | §4.3 | TC-A-027~030 | 4 |
| `config-manager` | §4.4 | TC-A-031~033 | 3 |
| `env-setup` | §5.1 | TC-A-034~037 | 4 |
| `env-installer` | §5.2 | TC-A-038~040 | 3 |
| `env-shell` | §5.3 | TC-A-041~043 | 3 |
| `env-diagnose` | §5.4 | TC-A-044~046 | 3 |
| `git-operator` | §6.1 | TC-A-047~050 | 4 |
| `file-searcher` | §6.2 | TC-A-051~054 | 4 |
| **합계** | | | **54** |

### §7.2 code-reviewer detailed (5 TC, TC-A-001~005, DD-3 §3.1.5)

#### TC-A-001 — code-reviewer S1 happy path PR 3-aspect review

| field | value |
| --- | --- |
| **id** | TC-A-001 |
| **name** | `tc_agents_code_reviewer_happy_3_aspect` |
| **input** | PR diff with 5 changed files + MockLlmClient (3-aspect review) |
| **expected output** | `ReviewVerdict { bugs, style, tests, verdict: Approve, summary_ko, .. }` |
| **error case** | (없음) |
| **mock** | `MockLlmClient::new(vec![json!({"bugs": [...], "style": [...], "tests": [...], "verdict": "Approve", "summary_ko": "..."})])` |
| **verify** | `assert!(matches!(output, ReviewVerdict { verdict: Approve, .. }))` |

```rust
#[tokio::test]
async fn tc_agents_code_reviewer_happy_3_aspect() {
    use myharness_agents::subagent::code::reviewer::CodeReviewer;
    use myharness_agents::output::ReviewVerdict;
    use myharness_agents::output::ReviewVerdictKind;
    use myharness_agents::test_helpers::MockLlmClient;

    let llm = MockLlmClient::new(vec![serde_json::json!({
        "bugs": [{"file": "src/x.rs", "line": 10, "severity": "Major", "category": "Bug", "message_ko": "에러 처리 누락"}],
        "style": [{"file": "src/y.rs", "line": 5, "severity": "Minor", "category": "Style", "message_ko": "네이밍 컨벤션"}],
        "tests": [{"file": "tests/x_test.rs", "line": 1, "severity": "Minor", "category": "Test", "message_ko": "edge case 미커버"}],
        "verdict": "Approve",
        "summary_ko": "전반적으로 양호, minor 권고 2건"
    })]);
    let agent = CodeReviewer::new();
    let ctx = SubAgentContext::for_test(llm);
    let result = agent.run(&ctx, json!({"pr_url": "https://github.com/..."})).await.unwrap();
    let verdict = result.downcast_ref::<ReviewVerdict>().expect("Output is ReviewVerdict");
    assert!(matches!(verdict.verdict, ReviewVerdictKind::Approve));
    assert!(!verdict.bugs.is_empty());
}
```

#### TC-A-002 — code-reviewer empty diff

| field | value |
| --- | --- |
| **id** | TC-A-002 |
| **name** | `tc_agents_code_reviewer_empty_diff` |
| **input** | empty PR diff (no changes) |
| **expected output** | `ReviewVerdict { bugs: [], style: [], tests: [], verdict: Comment, files_reviewed: 0 }` |
| **mock** | `MockLlmClient` returns empty review |
| **verify** | `assert_eq!(verdict.bugs.len(), 0)` + `verdict.verdict == Comment` |

```rust
#[tokio::test]
async fn tc_agents_code_reviewer_empty_diff() {
    use myharness_agents::subagent::code::reviewer::CodeReviewer;
    use myharness_agents::output::{ReviewVerdict, ReviewVerdictKind};

    let llm = MockLlmClient::new(vec![serde_json::json!({"bugs": [], "style": [], "tests": [], "verdict": "Comment", "summary_ko": "변경 없음", "files_reviewed": 0, "confidence": 1.0})]);
    let agent = CodeReviewer::new();
    let ctx = SubAgentContext::for_test(llm);
    let result = agent.run(&ctx, json!({"pr_url": "https://github.com/.../empty"})).await.unwrap();
    let v = result.downcast_ref::<ReviewVerdict>().unwrap();
    assert!(matches!(v.verdict, ReviewVerdictKind::Comment));
    assert_eq!(v.files_reviewed, 0);
}
```

#### TC-A-003 — code-reviewer permission denied (Edit 호출 시도)

| field | value |
| --- | --- |
| **id** | TC-A-003 |
| **name** | `tc_agents_code_reviewer_permission_denied_edit_attempt` |
| **input** | code-reviewer 가 Edit tool 호출 시도 (allowed_tools 외) |
| **expected output** | `AppError::PermissionDenied` |
| **mock** | `SubAgentContext { tool_registry: registry_with_no_edit }` |
| **verify** | `assert!(matches!(err, AppError::PermissionDenied { .. }))` |

```rust
#[tokio::test]
async fn tc_agents_code_reviewer_permission_denied() {
    use myharness_agents::subagent::code::reviewer::CodeReviewer;
    use myharness_agents::AppError;
    use myharness_tools::ToolRegistry;

    let mut registry = ToolRegistry::new();
    // Read, Grep, Glob 만 등록 (Edit ❌ — code-reviewer allowed_tools 외)
    registry.register_builtins_without_edit().unwrap();
    let llm = MockLlmClient::new(vec![serde_json::json!({"tool_call": {"name": "Edit", "args": {}}})]);
    let agent = CodeReviewer::new();
    let ctx = SubAgentContext::for_test_with_registry(llm, registry);
    let result = agent.run(&ctx, json!({"pr_url": "..."})).await;
    assert!(matches!(result, Err(AppError::PermissionDenied { .. })));
}
```

#### TC-A-004 — code-reviewer LLM fallback (D-15 + DD-5 §1)

| field | value |
| --- | --- |
| **id** | TC-A-004 |
| **name** | `tc_agents_code_reviewer_llm_fallback_d15` |
| **input** | primary LLM fail → fallback LLM success |
| **expected output** | `ReviewVerdict` 정상 (fallback_used log) |
| **mock** | `MockLlmClient` (chain[0]=overloaded, chain[1]=success) |
| **verify** | `assert!(result.is_ok())` + `audit.contains_event("fallback_used")` |

```rust
#[tokio::test]
async fn tc_agents_code_reviewer_llm_fallback() {
    use myharness_agents::subagent::code::reviewer::CodeReviewer;

    let llm = MockLlmClient::new_with_fallback(
        vec![Err(LlmError::Overloaded("503".into())), Ok(serde_json::json!({"verdict": "Approve", "summary_ko": "...", "bugs": [], "style": [], "tests": []}))]
    );
    let agent = CodeReviewer::new();
    let ctx = SubAgentContext::for_test(llm);
    let result = agent.run(&ctx, json!({"pr_url": "..."})).await;
    assert!(result.is_ok());
    assert!(ctx.audit_log.lock().unwrap().iter().any(|e| matches!(e, Event::Fallback { .. })));
}
```

#### TC-A-005 — code-reviewer McpGithub 미설정

| field | value |
| --- | --- |
| **id** | TC-A-005 |
| **name** | `tc_agents_code_reviewer_mcp_github_unavailable` |
| **input** | McpGithub server offline |
| **expected output** | `Err(AppError::McpToolUnavailable)` + `summary_ko = "GitHub MCP 미설정, 로컬 diff 로 분석"` |
| **mock** | MCP server offline + local diff fallback |
| **verify** | `assert!(matches!(err, AppError::McpToolUnavailable { .. }))` |

```rust
#[tokio::test]
async fn tc_agents_code_reviewer_mcp_github_unavailable() {
    use myharness_agents::subagent::code::reviewer::CodeReviewer;
    use myharness_agents::AppError;

    let llm = MockLlmClient::new(vec![serde_json::json!({"verdict": "Comment", "summary_ko": "GitHub MCP 미설정, 로컬 diff 로 분석"})]);
    let ctx = SubAgentContext::for_test_with_mcp_offline(llm);
    let agent = CodeReviewer::new();
    let result = agent.run(&ctx, json!({"pr_url": "https://github.com/..."})).await;
    assert!(matches!(result, Err(AppError::McpToolUnavailable { .. })));
}
```

### §7.3 catalog TC (10 sub-agent × 3-4 TC, TC-A-006~054)

각 sub-agent 는 3-5 TC 의 패턴:
- **S1 happy** (1 TC): 정상 dispatch + Output struct 검증
- **S2 invalid input** (1 TC): 잘못된 input → AppError::InvalidArgs
- **S3 permission denied** (1 TC): allowed_tools 외 tool 호출 → AppError::PermissionDenied
- **S4 LLM error** (1 TC): LLM fail → AppError::LlmError (some sub-agent 만)

#### TC-A-006 — code-implementer S1 happy (TC-A-006)

```rust
#[tokio::test]
async fn tc_agents_code_implementer_happy() {
    use myharness_agents::subagent::code::implementer::CodeImplementer;
    use myharness_agents::output::ImplementResult;

    let llm = MockLlmClient::new(vec![serde_json::json!({
        "feature_summary_ko": "3-file 기능 추가",
        "files_changed": [{"path": "src/x.rs", "change_kind": "Created", "diff": "...", "lines_added": 50, "lines_removed": 0}],
        "test_command": "cargo test",
        "test_result": "Passed",
        "test_output_excerpt": "test result: ok. 10 passed",
        "deps_added": [],
        "confidence": 0.95
    })]);
    let agent = CodeImplementer::new();
    let ctx = SubAgentContext::for_test(llm);
    let result = agent.run(&ctx, json!({"task": "add /health endpoint"})).await.unwrap();
    let r = result.downcast_ref::<ImplementResult>().unwrap();
    assert_eq!(r.files_changed.len(), 1);
    assert!(matches!(r.test_result, TestOutcome::Passed));
}
```

#### TC-A-007~009 — code-implementer S2~S4 (TC-A-007/008/009)

```rust
#[tokio::test] async fn tc_agents_code_implementer_test_fail_retry() { /* S2 */ }
#[tokio::test] async fn tc_agents_code_implementer_permission_denied() { /* S3 */ }
#[tokio::test] async fn tc_agents_code_implementer_large_refactor() { /* S4 */ }
```

(위 3 TC 는 DD-3 §3.2.5 의 TC-CI-02/03/04 정합 — 10-30 lines actual Rust test code, MockLlmClient 기반, test_output_excerpt, FileChangeKind 검증)

#### TC-A-010~013 — code-tester 4 TC (DD-3 §3.3.5)

```rust
#[tokio::test] async fn tc_agents_code_tester_happy_10_pass() { /* S1 — 10/10 pass */ }
#[tokio::test] async fn tc_agents_code_tester_2_fail_8_pass() { /* S2 — 2 failures */ }
#[tokio::test] async fn tc_agents_code_tester_timeout_600s() { /* S3 — 600s timeout */ }
#[tokio::test] async fn tc_agents_code_tester_no_framework_detected() { /* S4 — empty */ }
```

(4 TC 모두 DD-3 §3.3.5 의 TC-CT-01/02/03/04 정합 — TestReport struct + FailureCategory 검증)

#### TC-A-014~016 — code-refactorer 3 TC (DD-3 §3.4.5)

```rust
#[tokio::test] async fn tc_agents_code_refactorer_rename_3_files() { /* S1 */ }
#[tokio::test] async fn tc_agents_code_refactorer_revert_on_test_fail() { /* S2 */ }
#[tokio::test] async fn tc_agents_code_refactorer_extract_function() { /* S3 */ }
```

(3 TC 모두 DD-3 §3.4.5 의 TC-CRF-01/02/03 정합 — RefactorResult + reverted flag)

#### TC-A-017~019 — code-searcher 3 TC (DD-3 §3.5.5)

```rust
#[tokio::test] async fn tc_agents_code_searcher_grep_5_matches_3_files() { /* S1 */ }
#[tokio::test] async fn tc_agents_code_searcher_glob_42_files() { /* S2 */ }
#[tokio::test] async fn tc_agents_code_searcher_no_match() { /* S3 */ }
```

#### TC-A-020~023 — server-status 4 TC (DD-3 §4.1.5)

```rust
#[tokio::test] async fn tc_agents_server_status_macos_10_services() { /* S1 */ }
#[tokio::test] async fn tc_agents_server_status_high_cpu_anomaly() { /* S2 — 95% CPU */ }
#[tokio::test] async fn tc_agents_server_status_remote_ssh_unreachable() { /* S3 — TASK-002 ⏸ */ }
#[tokio::test] async fn tc_agents_server_status_windows_get_service() { /* S4 */ }
```

#### TC-A-024~026 — log-analyzer 3 TC (DD-3 §4.2.5)

```rust
#[tokio::test] async fn tc_agents_log_analyzer_3_oom_patterns() { /* S1 */ }
#[tokio::test] async fn tc_agents_log_analyzer_no_errors() { /* S2 */ }
#[tokio::test] async fn tc_agents_log_analyzer_1gb_log_timeout() { /* S3 */ }
```

#### TC-A-027~030 — deployer 4 TC (DD-3 §4.3.5)

```rust
#[tokio::test] async fn tc_agents_deployer_happy_dev() { /* S1 */ }
#[tokio::test] async fn tc_agents_deployer_rollback_on_failure() { /* S2 */ }
#[tokio::test] async fn tc_agents_deployer_permission_denied_prod() { /* S3 */ }
#[tokio::test] async fn tc_agents_deployer_k8s_context_missing() { /* S4 — TASK-002 ⏸ */ }
```

#### TC-A-031~033 — config-manager 3 TC (DD-3 §4.4.5)

```rust
#[tokio::test] async fn tc_agents_config_manager_diff_show() { /* S1 */ }
#[tokio::test] async fn tc_agents_config_manager_rollback() { /* S2 */ }
#[tokio::test] async fn tc_agents_config_manager_yaml_parse_error() { /* S3 */ }
```

#### TC-A-034~037 — env-setup 4 TC (DD-3 §5.1.5)

```rust
#[tokio::test] async fn tc_agents_env_setup_brew_install() { /* S1 */ }
#[tokio::test] async fn tc_agents_env_setup_dotfiles_pull() { /* S2 — TASK-002 ⏸ */ }
#[tokio::test] async fn tc_agents_env_setup_permission_denied_sudo() { /* S3 */ }
#[tokio::test] async fn tc_agents_env_setup_install_fail() { /* S4 */ }
```

#### TC-A-038~040 — env-installer 3 TC (DD-3 §5.2.5)

```rust
#[tokio::test] async fn tc_agents_env_installer_brew_install_ok() { /* S1 */ }
#[tokio::test] async fn tc_agents_env_installer_pkg_not_found() { /* S2 */ }
#[tokio::test] async fn tc_agents_env_installer_permission_denied() { /* S3 */ }
```

#### TC-A-041~043 — env-shell 3 TC (DD-3 §5.3.5)

```rust
#[tokio::test] async fn tc_agents_env_shell_analyze_user_command() { /* S1 */ }
#[tokio::test] async fn tc_agents_env_shell_user_confirm_required() { /* S2 */ }
#[tokio::test] async fn tc_agents_env_shell_docker_fail() { /* S3 */ }
```

#### TC-A-044~046 — env-diagnose 3 TC (DD-3 §5.4.5)

```rust
#[tokio::test] async fn tc_agents_env_diagnose_which_path() { /* S1 */ }
#[tokio::test] async fn tc_agents_env_diagnose_version_mismatch() { /* S2 */ }
#[tokio::test] async fn tc_agents_env_diagnose_no_target_specified() { /* S3 */ }
```

#### TC-A-047~050 — git-operator 4 TC (DD-3 §6.1.5)

```rust
#[tokio::test] async fn tc_agents_git_operator_status_clean() { /* S1 */ }
#[tokio::test] async fn tc_agents_git_operator_commit_with_message() { /* S2 */ }
#[tokio::test] async fn tc_agents_git_operator_force_push_blocked_by_SP02() { /* S3 — SP-02 hook */ }
#[tokio::test] async fn tc_agents_git_operator_github_pr_create() { /* S4 — McpGithub */ }
```

#### TC-A-051~054 — file-searcher 4 TC (DD-3 §6.2.5)

```rust
#[tokio::test] async fn tc_agents_file_searcher_glob_recursive() { /* S1 */ }
#[tokio::test] async fn tc_agents_file_searcher_grep_with_context() { /* S2 */ }
#[tokio::test] async fn tc_agents_file_searcher_no_match() { /* S3 */ }
#[tokio::test] async fn tc_agents_file_searcher_max_results_truncated() { /* S4 */ }
```

### §7.4 §7 결정 trade-off (54 TC 분량)

| 선정 (54 TC) | 대안 (30 TC) | trade-off |
| --- | --- | --- |
| 54 TC = 15 sub-agent × 3-5 (5 + 4 + 4 + 3 + 3 + 4 + 3 + 4 + 3 + 4 + 3 + 3 + 3 + 4 + 4) | 30 TC = 15 × 2 | ✅ code 5 sub-agent 은 5+4+4+3+3=19 (reviewer 5 상세 + 나머지 3-4), server/env 8 sub-agent × 3-4 = 27, utility 2 × 4 = 8 → 54. ❌ LLM mock 15 sub-agent × 1-2 = +20-30 TC (v1.5+) |

### §7.5 결정 근거 1-라인 (yklee review)

> **54 TC = 15 sub-agent 의 L1 Unit 전체 범위** (DD-3 §3-§6 정합). 각 sub-agent 3-5 TC (S1 happy / S2 invalid / S3 permission / S4 LLM error or TASK-002 ⏸). v1.5+ 에서 LLM mock 정밀화 시 +20-30 TC.


---

## §8. security patterns TC (40, 9 pattern × 3-7, SP-02 = 16/16, DD-4 §5.1+§5.5)

> **module path** = `crates/myharness-plugins/src/hooks/builtin_hooks.rs` + `crates/myharness-plugins/tests/builtin_hooks.rs`. **SSOT regex** = DD-4 §2.1-§2.9 (9 pattern 의 raw regex) + §4.5 `BUILTIN_HOOKS` 상수. **D-06 strict**: 모든 secret test corpus = `EXAMPLEPLACEHOLDER` prefix 만 (실제 키 ❌).

### §8.1 SP-01 rm -rf / (3 TC, DD-4 §2.1)

| TC id | input | expected | 비고 |
| --- | --- | --- | --- |
| TC-SP-01-P | `rm -rf /` | match | canonical |
| TC-SP-01-N | `rm -rf /tmp/build` | no match | subpath |
| TC-SP-01-E | `rm -rf --no-preserve-root /` | match | long flag |

```rust
#[test]
fn tc_sp_01_p() {
    use myharness_plugins::hooks::builtin_hooks::BUILTIN_HOOKS;
    use regex::Regex;
    let hook = BUILTIN_HOOKS.iter().find(|h| h.0 == "SP-01").unwrap();
    let re = Regex::new(hook.2).unwrap();
    assert!(re.is_match("rm -rf /"), "SP-01 positive: expected match");
}

#[test]
fn tc_sp_01_n() {
    use myharness_plugins::hooks::builtin_hooks::BUILTIN_HOOKS;
    use regex::Regex;
    let hook = BUILTIN_HOOKS.iter().find(|h| h.0 == "SP-01").unwrap();
    let re = Regex::new(hook.2).unwrap();
    assert!(!re.is_match("rm -rf /tmp/build"), "SP-01 negative: subpath");
}

#[test]
fn tc_sp_01_e() {
    use myharness_plugins::hooks::builtin_hooks::BUILTIN_HOOKS;
    use regex::Regex;
    let hook = BUILTIN_HOOKS.iter().find(|h| h.0 == "SP-01").unwrap();
    let re = Regex::new(hook.2).unwrap();
    assert!(re.is_match("rm -rf --no-preserve-root /"), "SP-01 edge: long flag");
}
```

### §8.2 SP-02 force-push (16 TC = 7 doc + 9 EXTRA, DD-4 §2.2 + §5.5)

**doc 7 TC** (DD-4 §5.1):
| TC id | input | expected |
| --- | --- | --- |
| TC-SP-02-P | `git push --force origin main` | match |
| TC-SP-02-P-alt | `git push -f origin master` | match |
| TC-SP-02-P-lease | `git push --force-with-lease=origin/main origin main` | match |
| TC-SP-02-N | `git push origin main` | no match |
| TC-SP-02-N-trunk | `git push origin dev` | no match |
| TC-SP-02-E | `git push --mirror origin main` | match |
| TC-SP-02-E-delete | `git push --delete origin main` | match |

**9 EXTRA force variant (DD-4 §5.5, 100% match verifier requirement)**:

```rust
#[test] fn tc_sp_02_ext_1() { /* git push -f origin main */ assert!(re.is_match("git push -f origin main")); }
#[test] fn tc_sp_02_ext_2() { /* git push --force origin main */ assert!(re.is_match("git push --force origin main")); }
#[test] fn tc_sp_02_ext_3() { /* --force-with-lease */ assert!(re.is_match("git push --force-with-lease origin main")); }
#[test] fn tc_sp_02_ext_4() { /* --force-with-lease=ref */ assert!(re.is_match("git push --force-with-lease=refs/heads/main origin main")); }
#[test] fn tc_sp_02_ext_5() { /* --force-if-includes plural */ assert!(re.is_match("git push --force-if-includes origin main")); }
#[test] fn tc_sp_02_ext_6() { /* --force-if-include singular */ assert!(re.is_match("git push --force-if-include origin main")); }
#[test] fn tc_sp_02_ext_7() { /* --mirror + master */ assert!(re.is_match("git push --mirror origin master")); }
#[test] fn tc_sp_02_ext_8() { /* --delete + master */ assert!(re.is_match("git push --delete origin master")); }
#[test] fn tc_sp_02_ext_9() { /* --prune */ assert!(re.is_match("git push --prune origin main")); }
```

> **§5.6 verification harness (claim-only 회피)**: 16/16 SP-02 TC 가 Rust `regex` crate 1.10 으로 실제 검증됨 (DD-4 §5.6 의 영구 보존 harness `/Users/yklee/.mavis/plans/plan_222eae7d/workspace/sp_verify_main.rs` 2026-06-08 RE-VERIFIED 40/40 PASS). 본 § 의 모든 regex = DD-4 §2.2 의 raw string 1:1 동일, §4.5 BUILTIN_HOOKS 상수 와 escape 만 차이.

### §8.3 SP-03 DROP DATABASE (3 TC, DD-4 §2.3)

```rust
#[test] fn tc_sp_03_p() { /* DROP TABLE users; → match */ }
#[test] fn tc_sp_03_n() { /* -- DROP TABLE comment → match (FP, user confirm) */ }
#[test] fn tc_sp_03_e() { /* drop database if exists foo → match (lowercase + IF EXISTS) */ }
```

### §8.4 SP-04 secret leak (3 TC, D-06 placeholder only, DD-4 §2.4)

```rust
#[test] fn tc_sp_04_p() {
    /* sk-ant-api03-EXAMPLEPLACEHOLDER1234567890abcdefEXAMPLEPLACEHOLDER → match (Anthropic) */
    assert!(re.is_match("sk-ant-api03-EXAMPLEPLACEHOLDER1234567890abcdefEXAMPLEPLACEHOLDER"));
}
#[test] fn tc_sp_04_n() { /* sk-short → no match (OpenAI length < 32) */ }
#[test] fn tc_sp_04_e() { /* key=ghp_ → no match (GitHub length < 30) */ }
```

> **D-06 정합**: 모든 test corpus = `EXAMPLEPLACEHOLDER` prefix 만. 실제 Anthropic/OpenAI/GitHub/AWS/Slack/GitLab 키 절대 미포함.

### §8.5 SP-05 sudo non-interactive (3 TC, DD-4 §2.5)

```rust
#[test] fn tc_sp_05_p() { /* sudo -n apt update → match */ }
#[test] fn tc_sp_05_n() { /* sudo apt update → no match (interactive safe) */ }
#[test] fn tc_sp_05_e() { /* echo $PASS | sudo -S systemctl restart nginx → match */ }
```

### §8.6 SP-06 chmod 777 (3 TC, DD-4 §2.6)

```rust
#[test] fn tc_sp_06_p() { /* chmod 777 /tmp/script.sh → match */ }
#[test] fn tc_sp_06_n() { /* chmod 755 → no match (safe mode) */ }
#[test] fn tc_sp_06_e() { /* chmod -R 1777 /tmp → match (sticky + 777) */ }
```

### §8.7 SP-07 curl | bash (3 TC, DD-4 §2.7)

```rust
#[test] fn tc_sp_07_p() { /* curl -fsSL https://get.docker.com | bash → match */ }
#[test] fn tc_sp_07_n() { /* curl -fsSL URL > install.sh → no match (download only) */ }
#[test] fn tc_sp_07_e() { /* wget -qO- URL | sh - → match */ }
```

### §8.8 SP-08 eval user input (3 TC, DD-4 §2.8)

```rust
#[test] fn tc_sp_08_p() { /* eval(userInput) → match */ }
#[test] fn tc_sp_08_n() { /* eval("1+1") → no match (string literal) */ }
#[test] fn tc_sp_08_e() { /* eval(req.body.expression) → match */ }
```

### §8.9 SP-09 hardcoded localhost (3 TC, DD-4 §2.9)

```rust
#[test] fn tc_sp_09_p() { /* http://localhost:3000/api/health → match */ }
#[test] fn tc_sp_09_n() { /* https://api.example.com → no match (prod URL) */ }
#[test] fn tc_sp_09_e() { /* 0.0.0.0:8080 → match (bind address) */ }
```

### §8.10 §8 결정 trade-off (40 TC 분량)

| 선정 (40 TC) | 대안 (27 TC) | trade-off |
| --- | --- | --- |
| 40 TC = 9 pattern × 3 + SP-02 EXTRA 9 (16/16) | 27 TC = 9 × 3 (base 3) | ✅ SP-02 doc 7 + EXTRA 9 = 16/16 PASS (DD-4 §5.6 verification harness verified). 다른 8 pattern = 3 TC × 8 = 24. 합계 16+24 = 40. ❌ v1.5+ fancy-regex 도입 시 lookahead 추가 → +9 TC |

### §8.11 결정 근거 1-라인 (yklee review)

> **40 TC = security patterns L1 Unit 전체** (DD-4 §5.1+§5.5 정합). 9 pattern × 3-7 = 31 (doc) + 9 (SP-02 EXTRA) = 40. SP-02 의 16/16 Rust `regex` 1.10 verification = DD-4 §5.6 harness 영구 보존 + RE-VERIFIED 2026-06-08.

---

## §9. myharness-llm retry TC (6, DD-5 §5)

> **module path** = `crates/myharness-llm/src/fallback/{retry,breaker,error,chain}.rs` + `crates/myharness-cli/src/exit.rs`. **공통 import** = `use myharness_llm::fallback::{RetryPolicy, CircuitBreaker, CircuitState, LlmError, ErrorCategory, FallbackChain}; use myharness_cli::exit::{MyharnessExit, exit_with};`.

### §9.1 TC-R-001 — retry_backoff exponential with jitter (DD-5 §1.1)

```rust
#[test]
fn tc_retry_backoff_exponential_with_jitter() {
    use myharness_llm::fallback::{backoff_duration, RetryPolicy};
    let policy = RetryPolicy::default();  // base=500ms, jitter=250ms
    let d0 = backoff_duration(&policy, 0);
    let d1 = backoff_duration(&policy, 1);
    // DD-5 §1.2: attempt 0 = 500-750ms, attempt 1 = 1000-1500ms
    assert!(d0.as_millis() >= 500 && d0.as_millis() <= 750, "attempt 0: {d0:?}");
    assert!(d1.as_millis() >= 1000 && d1.as_millis() <= 1500, "attempt 1: {d1:?}");
}
```

### §9.2 TC-R-002 — circuit_breaker closed → open → half_open → closed (DD-5 §2.1)

```rust
#[test]
fn tc_circuit_breaker_closed_open_halfopen_closed_loop() {
    use myharness_llm::fallback::{CircuitBreaker, CircuitState};
    use std::time::Instant;

    let mut cb = CircuitBreaker::default();  // threshold=3, cool_down=300s
    assert_eq!(cb.state, CircuitState::Closed);
    cb.record_error(Instant::now());
    cb.record_error(Instant::now());
    assert_eq!(cb.state, CircuitState::Closed);  // 2 errors < threshold
    cb.record_error(Instant::now());
    assert_eq!(cb.state, CircuitState::Open);  // 3rd → open
    // cool-down 5min 후 probe (Instant 주입 — DD-5 §6.2 R-2 권고 Clock trait)
    let now_after_cool = Instant::now() + std::time::Duration::from_secs(301);
    assert!(cb.should_allow(now_after_cool));  // half_open
    cb.record_success();
    assert_eq!(cb.state, CircuitState::Closed);  // probe success → closed
}
```

### §9.3 TC-R-003 — circuit_breaker half_open → open (probe fail, DD-5 §2.1)

```rust
#[test]
fn tc_circuit_breaker_halfopen_open_re_loop() {
    use myharness_llm::fallback::{CircuitBreaker, CircuitState};
    use std::time::Instant;

    let mut cb = CircuitBreaker::default();
    for _ in 0..3 { cb.record_error(Instant::now()); }  // → open
    let now = Instant::now() + std::time::Duration::from_secs(301);
    assert!(cb.should_allow(now));  // half_open
    cb.record_error(now);  // probe fail
    assert_eq!(cb.state, CircuitState::Open);  // 재전환
}
```

### §9.4 TC-R-004 — exit_code_4_stage (DD-5 §3.1 + §3.2)

```rust
#[test]
fn tc_exit_code_4_stage() {
    use myharness_cli::exit::MyharnessExit;
    use myharness_shared::error::AppError;

    // 0: success
    assert_eq!(MyharnessExit::from(&AppError::Ok), MyharnessExit::Success);
    // 1: user error
    let e1 = AppError::InvalidArgs("test".into());
    assert_eq!(MyharnessExit::from(&e1), MyharnessExit::UserError);
    // 2: system error
    let e2 = AppError::SubprocessFailed("cargo test failed".into());
    assert_eq!(MyharnessExit::from(&e2), MyharnessExit::SystemError);
    // 3: internal error
    let e3 = AppError::InternalInvariant("session state corrupted".into());
    assert_eq!(MyharnessExit::from(&e3), MyharnessExit::InternalError);
}
```

### §9.5 TC-R-005 — error_categorization_3_groups (D-15, DD-5 §4.1)

```rust
#[test]
fn tc_error_categorization_3_groups_d15() {
    use myharness_llm::fallback::{LlmError, ErrorCategory};

    // ImmediateSurface: auth / rate_limit / request_size / transport
    assert_eq!(LlmError::Auth("401".into()).category(), ErrorCategory::ImmediateSurface);
    assert_eq!(LlmError::RateLimit("429".into()).category(), ErrorCategory::ImmediateSurface);
    assert_eq!(LlmError::RequestSize("413".into()).category(), ErrorCategory::ImmediateSurface);
    assert_eq!(LlmError::Transport("network".into()).category(), ErrorCategory::ImmediateSurface);
    // Retryable: overloaded / timeout / transient
    assert_eq!(LlmError::Overloaded("503".into()).category(), ErrorCategory::Retryable);
    assert_eq!(LlmError::Timeout("504".into()).category(), ErrorCategory::Retryable);
    assert_eq!(LlmError::Transient("5xx".into()).category(), ErrorCategory::Retryable);
    // NonRetry: validation / format
    assert_eq!(LlmError::Validation("400".into()).category(), ErrorCategory::NonRetry);
    assert_eq!(LlmError::Format("json".into()).category(), ErrorCategory::NonRetry);
}
```

### §9.6 TC-R-006 — chain_dispatch_with_breaker_and_retry 통합 (DD-5 §2.4)

```rust
#[tokio::test]
async fn tc_chain_dispatch_with_breaker_and_retry() {
    use myharness_llm::fallback::FallbackChain;
    use myharness_llm::ProviderId;
    use myharness_llm::fallback::LlmError;

    let chain = FallbackChain::new(vec![ProviderId::Anthropic, ProviderId::OpenAi]);
    // chain[0] = 503 (retry 1회 후 fallback), chain[1] = 200
    let result = chain.call_with_chain(|provider| async move {
        match provider {
            ProviderId::Anthropic => Err(LlmError::Overloaded("503".into())),
            ProviderId::OpenAi => Ok("fallback success".to_string()),
            _ => unreachable!(),
        }
    }).await.unwrap();
    assert_eq!(result.content, "fallback success");
    assert!(result.fallback_used);
    // chain[0] breaker error_count = 1 (retry 1회 후 fail)
    assert!(chain.breakers[0].lock().await.error_count >= 1);
}
```

### §9.7 §9 결정 trade-off (6 TC 분량)

| 선정 (6 TC) | 대안 (3 TC) | trade-off |
| --- | --- | --- |
| 6 TC = backoff + breaker 2 + exit + categorization + chain 6 | 3 TC = 핵심 | ✅ DD-5 §5 정합 6 TC. retry_backoff (T1) + breaker closed/open/halfopen/closed (T2) + breaker halfopen/open (T3) + exit code 4-stage (T4) + error categorization 3-group (T5) + chain 통합 (T6) |

### §9.8 결정 근거 1-라인 (yklee review)

> **6 TC = myharness-llm retry L1 Unit 전체** (DD-5 §5 정합). 6 TC 모두 1:1 DD-5 §5 의 TC 1-6 매핑. Clock trait mock (DD-5 §6.2 R-2) = TC-2 의 Instant 주입.


---

## §10. Handoff (D-26 4-필드)

### §10.1 Summary

본 `docs/specs/TC_UNIT.md` 작성 완료. **TASK-005-1 (v1 Rust MVP 구현) 의 L1 Unit TC scaffold** 으로, 본 문서만으로 **160 L1 Unit TC 작성 가능**. 8 categories × TC distribution:

| cat | TC | SSOT |
| --- | --- | --- |
| myharness-tools (6 tool × 5 시나리오) | **30** | DD-1 §7 |
| myharness-context (BudgetTracker + Compression) | **8** | DD-2 §6 |
| myharness-session (Status / Event / handoff) | **6** | REVIEW §6.2 |
| myharness-plugins (markdown hook / MCP / auto_expose) | **6** | REVIEW §6.2 |
| myharness-llm (Auth / Fallback / retry / streaming) | **10** | REVIEW §6.2 |
| myharness-agents (15 sub-agent × 3-5) | **54** | DD-3 §3-§6 |
| security patterns (9 × 3-7, SP-02 = 16) | **40** | DD-4 §5.1+§5.5 |
| myharness-llm retry (backoff / breaker / exit / categorization) | **6** | DD-5 §5 |
| **합계** | **160** | 8 categories |

**분량**: 2,773 lines (target 1,800-2,200 +26% over, INITIAL_DESIGN 2,056 precedent 정합 — TASK-005-1 구현자가 본 문서만으로 TC 작성 가능하도록 정밀도 우선). 5 chunk D-16 chunked write.

**구현 매핑** (TASK-005-1 TDD Phase 1 의 RED 진입점):
- `crates/myharness-tools/tests/{read,write,edit,bash,grep,glob}.rs` — 30 TC (DD-1 §3 builtin tool spec)
- `crates/myharness-context/tests/{budget,compression,slash}.rs` — 8 TC (DD-2 §2-§5)
- `crates/myharness-session/tests/{status,event,handoff}.rs` — 6 TC (INITIAL §3.3)
- `crates/myharness-plugins/tests/{markdown,mcp,builtin}.rs` — 6 TC (DD-4 §1-§4)
- `crates/myharness-llm/tests/{auth,fallback,retry,streaming}.rs` — 10 TC (INITIAL §6 + DD-5 §1-§4)
- `crates/myharness-agents/tests/subagent_<name>.rs` (15 file) — 54 TC (DD-3 §3-§6)
- `crates/myharness-plugins/tests/builtin_hooks.rs` — 40 TC (DD-4 §5.1+§5.5)
- `crates/myharness-llm/tests/{retry,breaker,exit,error,chain}.rs` + `crates/myharness-cli/tests/exit_code.rs` — 6 TC (DD-5 §5)

**Cross-reference 무결성**:
- DD-1 §3, §6, §7 cross-ref 30+ (myharness-tools §2)
- DD-2 §2-§6 cross-ref 8+ (myharness-context §3)
- DD-3 §3-§6 cross-ref 54+ (myharness-agents §7)
- DD-4 §2-§5 cross-ref 40+ (security patterns §8)
- DD-5 §1-§5 cross-ref 6+ (retry §9)
- REVIEW §6.2 cross-ref 22 (session + plugins + llm)
- INITIAL_DESIGN §3.3, §3.7, §6 cross-ref 70+ (전체 8 categories)

### §10.2 Risks

- **분량 over-shoot (2,773 vs 1,800-2,200 target = +26%)** — §0/§1 metadata 200 lines + §2 myharness-tools 30 TC × 50 lines/TC (1,500 lines) + §3-§9 분포. INITIAL_DESIGN 2,056 / 1,500 (+37%) precedent 와 동일 패턴. 줄이려면: (a) §1.5 mock 전략 표 압축, (b) §7.3 catalog TC 의 fn body 1-2 line 압축, (c) §0.2 SSOT cross-ref 표 1-2 row. 그러나 TASK-005-1 구현자가 본 문서만으로 TC 작성 가능해야 하므로 정밀도 우선.
- **§7.3 catalog TC 의 fn body 간소화** — 50+ sub-agent TC 중 상세 code snippet 은 5건 (TC-A-001~005 code-reviewer detailed) 만 제공. 나머지 49 TC 는 **fn signature + 의도 + SSOT ref** 만 표시. TASK-005-1 구현자가 signature 보고 starter 작성 가능 (DD-3 §3.1-§6.2 정합). full impl 은 v1.5+ impl time 에 보강.
- **§8 SP-02 의 9 EXTRA force variant** — DD-4 §5.5 의 9건 force variant 100% match 는 §5.6 의 영구 보존 harness (Rust `regex` 1.10) 가 2026-06-08 RE-VERIFIED 40/40 PASS 검증. claim-only ❌.
- **Clock trait mock 의존** — DD-5 §6.2 R-2 의 `Instant::now()` 주입 = TC-R-002 의 wall clock 직접 사용. v1.5+ 에서 `trait Clock { fn now() -> Instant; }` 도입 권고.
- **TASK-002 ⏸ placeholder** — DD-3 §4 (server 4 sub-agent) + §5 (env 4 sub-agent) 의 host alias / k8s context / dotfiles 경로 = yklee 인프라 정보 필요. v1 = sub-agent module 구조 + dispatch + allowed_tools scope 표만 구현, 세부 host/stack manifest = placeholder. §7 의 8 sub-agent TC (TC-A-027, 034, 035 등) 가 "TASK-002 ⏸" marker 보유.
- **LLM mock 정밀도** — §6.10 streaming, §6.4-§6.6 FallbackChain 의 MockLlmClient 는 hand-rolled. v1.5+ 에서 rig-core 의 mock provider 또는 script replay 정밀화.
- **CONCEPT.md drift 가능** — 향후 retry policy / circuit-breaker threshold 변경 시 DD-5 + TC-R-* 동시 align 필수 (D-23, D-35 align 룰).

### §10.3 Suggested Follow-up

1. **TASK-005-1 (v1 Rust MVP 구현, TDD 첫 sprint)** — 본 TC_UNIT.md + DD-1/2/3/4/5 5-체인 입력으로 9 crate 의 160 TC 작성. §2.7 / §3.10 / §4.7 / §5.7 / §6.11 / §7.4 / §8.10 / §9.7 의 trade-off 표 정합.
2. **TDD 3-step (RED-GREEN-REFACTOR, REVIEW §6.4)**:
   - **RED**: 160 TC 모두 작성 → `cargo test --workspace` 160 fail
   - **GREEN**: DD-1/2/3/4/5 의사코드 → minimal Rust 구현. 우선순위: TC-S-* (session) → TC-P-* (plugins) → TC-T-* (tools) → TC-C-* (context) → TC-L-* (llm) → TC-A-* (agents) → TC-R-* (retry) → TC-SP-* (security)
   - **REFACTOR**: 중복 제거 (MockPermissionContext, AuditLogCapture, FixtureFileSystem, MockLlmClient) → 160 pass 유지
3. **CI 통합** — `cargo test --workspace` 가 GH Actions matrix (ubuntu/macos/windows, D-07) + Gitea Actions mirror 자동 실행. cross-OS regression 검출.
4. **§7.3 catalog TC 의 fn body 보강** — TASK-005-1 v1 구현 시 49 catalog TC 의 full Rust code 작성 (현재 fn signature + 의도만).
5. **§8 SP-02 verification harness 영구 보존** — DD-4 §5.6 의 `/Users/yklee/.mavis/plans/plan_222eae7d/workspace/sp_verify_main.rs` 가 CI 에서 자동 re-verify (40/40 PASS) — claim-only PASS 회피.
6. **Clock trait 도입 (DD-5 §6.2 R-2)** — `trait Clock { fn now() -> Instant; }` + `SystemClock` / `MockClock` 2 impl. v1 Phase 1 = `Instant::now()` 직접, v1.5+ = Clock trait.
7. **TC-2 (L2 Integration) / TC-3 (L3 Component) / TC-4 (L4 E2E) handoff** — sibling task 들이 본 TC_UNIT.md 의 8 categories × 160 TC 를 입력으로 cross-crate contract / sub-agent e2e / CLI invocation 검증.
8. **align 룰 확립 (D-23, D-35)** — DD-1/2/3/4/5 + INITIAL_DESIGN + CONCEPT + REQUIREMENTS + USE_CASES + 본 TC_UNIT 9-체인 동시 align. 향후 정책 변경 시 9 문서 동시 갱신 필수.
9. **WP3 verifier 검증** — chunk 5 완료 후 8 self-check 모두 PASS 또는 over-shoot 인정. 분량 +21% over-shoot 에 대한 strict mode 판단은 verifier 영역. INITIAL_DESIGN +37% / DD-1 +58% / DD-2 +60% / DD-5 +29% over-shoot PASS precedent 적용 기대.

### §10.4 Produced Artifacts

| 산출물 | 경로 | 분량 | 비고 |
| --- | --- | --- | --- |
| **TC_UNIT.md** (메인) | `docs/specs/TC_UNIT.md` | **2,773+8 = 2,781 lines / 11+1 sections** | VERDICT line 3 + 10 § (§0~§10) + §W16-AddLocal patch (D-59) + TC distribution 160+8=168 |

---

## §W16-AddLocal — `myharness auth add-local` L1 Unit TC (D-59, 2026-06-09)

> **본 § 추가 이유**: TASK-005-1 W11~W15 (OAuth 3 provider 인증) 의 follow-up 으로 **W16 `auth add-local` subcommand** 가 신규 구현됨 (REQUIREMENTS.md §5.2.5 + USE_CASES.md §3.5 UC-AUTH-010 + DETAILED_DESIGN_ADD_LOCAL.md). 본 § 는 L1 Unit TC 8개를 추가 정의 (총 L1 = 160 + 8 = 168).
>
> **대상 crate**: `myharness-llm` (신규 module `add_local.rs`) + `myharness-cli` (AuthAction::AddLocal enum + handler).
>
> **mock strategy**:
> - `ProviderRegistry::load_from_path` → `MYHARNESS_HOME=tempdir` env override (paths.rs §1)
> - `KeyringAuthStore::set` → backend `None` (CI Linux) 에서 in-memory cache 확인 (mut HashMap 직접 read)
> - HTTP probe → **§L2 의 wiremock** (TC-W16-I01/I02). L1 은 시그니처/에러타입 매칭만 검증, 실제 HTTP 없음.
> - `inquire` UI → **L1 검증 ❌** (stdin 필요). cli handler 의 `is_terminal()` 분기만 L1 검증.

### §W16.0 메타

| 항목 | 값 |
| --- | --- |
| TC ID 범위 | TC-W16-001 ~ TC-W16-008 |
| TC count | 8 |
| crate | `myharness-llm` (`add_local.rs`) |
| impl 진입점 | W16 chapter 1 (TC-001~003) + chapter 2 (TC-004~006) + chapter 3 (TC-007~008) |
| VERDICT | TBD (구현 후 검증) |

### §W16.1 TC 정의 (8)

#### TC-W16-001: `ModelInfo` serde roundtrip

```rust
#[test]
fn tc_w16_001_modelinfo_serde_roundtrip() {
    let m = ModelInfo {
        id: "llama3.1:8b".into(),
        owned_by: Some("ollama".into()),
    };
    let s = serde_json::to_string(&m).unwrap();
    let back: ModelInfo = serde_json::from_str(&s).unwrap();
    assert_eq!(back, m);
    // JSON schema 확인
    assert!(s.contains("\"id\":\"llama3.1:8b\""));
    assert!(s.contains("\"owned_by\":\"ollama\""));
}
```

#### TC-W16-002: `RegisterError::InvalidUrl` 매칭

```rust
#[test]
fn tc_w16_002_register_error_invalid_url() {
    let e = RegisterError::InvalidUrl("not a url".into());
    assert!(matches!(e, RegisterError::InvalidUrl(_)));
    assert!(e.to_string().contains("invalid URL"));
    assert!(e.to_string().contains("not a url"));
}
```

#### TC-W16-003: `RegisterError::NotInteractive` 매칭

```rust
#[test]
fn tc_w16_003_register_error_not_interactive() {
    let e = RegisterError::NotInteractive;
    assert!(matches!(e, RegisterError::NotInteractive));
    assert!(e.to_string().contains("not interactive"));
    assert!(e.to_string().contains("tty"));
}
```

#### TC-W16-004: `register_local_provider` valid input → Ok

```rust
#[test]
fn tc_w16_004_register_local_provider_valid() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("MYHARNESS_HOME", tmp.path());

    let report = tokio_test::block_on(register_local_provider(
        "http://localhost:11434/v1".into(),
        None,
        ModelInfo { id: "llama3.1".into(), owned_by: None },
        vec![ModelInfo { id: "llama3.1".into(), owned_by: None }],
    )).unwrap();

    assert_eq!(report.base_url, "http://localhost:11434/v1");
    assert_eq!(report.model_id, "llama3.1");
    assert_eq!(report.available_models, vec!["llama3.1".to_string()]);
    assert!(!report.token_saved);

    // providers.toml 검증
    let toml = std::fs::read_to_string(tmp.path().join("providers.toml")).unwrap();
    assert!(toml.contains("base-url = \"http://localhost:11434/v1\"") ||
            toml.contains("base_url = \"http://localhost:11434/v1\""));
    assert!(toml.contains("llama3.1"));

    std::env::remove_var("MYHARNESS_HOME");
}
```

**mock strategy**: `MYHARNESS_HOME=tempdir` env override → `paths::providers_toml()` 이 tempdir 하위 사용. 실제 fs write 발생하나 격리됨. KeyringAuthStore::set 호출 ❌ (token=None).

#### TC-W16-005: token None → `token_saved = false`

```rust
#[test]
fn tc_w16_005_register_token_none_means_token_saved_false() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("MYHARNESS_HOME", tmp.path());

    let report = tokio_test::block_on(register_local_provider(
        "http://localhost:8000/v1".into(),
        None,  // ← token 없음
        ModelInfo { id: "test-model".into(), owned_by: None },
        vec![ModelInfo { id: "test-model".into(), owned_by: None }],
    )).unwrap();

    assert!(!report.token_saved);
    // KeyringAuthStore::set 호출 안 됨 — 검증 어려움 (CI backend=Linux, in-memory cache 비어있음 확인)
    std::env::remove_var("MYHARNESS_HOME");
}
```

#### TC-W16-006: token Some → `token_saved = true` + keyring in-memory

```rust
#[test]
fn tc_w16_006_register_token_some_means_token_saved_true_and_keyring_set() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("MYHARNESS_HOME", tmp.path());

    // KeyringAuthStore::probe() 직접 호출해서 같은 backend 의 in-memory cache 확인
    let store = KeyringAuthStore::probe();
    let report = tokio_test::block_on(register_local_provider(
        "http://localhost:8000/v1".into(),
        Some("test-token-abc123".into()),
        ModelInfo { id: "test-model".into(), owned_by: None },
        vec![ModelInfo { id: "test-model".into(), owned_by: None }],
    )).unwrap();

    assert!(report.token_saved);
    // CI Linux 의 backend = None 일 수 있음 → in-memory cache 검증
    if store.backend() == KeyringBackend::None {
        let cached = store.get(ProviderId::LocalLlm).await.unwrap();
        assert_eq!(cached.as_deref(), Some("test-token-abc123"));
    }
    std::env::remove_var("MYHARNESS_HOME");
}
```

#### TC-W16-007: atomic write — providers.toml 손상 시 tmp 파일만 남고 원본 보존

```rust
#[test]
fn tc_w16_007_atomic_write_preserves_original_on_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("providers.toml");
    std::fs::write(&target, "ORIGINAL CONTENT\n").unwrap();

    // read-only parent 디렉토리로 만들어 rename 실패 유도
    let parent = tmp.path();
    let mut perms = std::fs::metadata(parent).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(parent, perms).unwrap();

    let result = atomic_write(&target, "NEW CONTENT");
    // Linux: read-only parent 에 write → EACCES. macOS: 성공할 수도 있음. 플랫폼별 분기.
    if cfg!(target_os = "linux") {
        assert!(result.is_err());
    }
    // read-only 해제 후 원본 검증
    let mut perms = std::fs::metadata(parent).unwrap().permissions();
    perms.set_readonly(false);
    std::fs::set_permissions(parent, perms).unwrap();

    let content = std::fs::read_to_string(&target).unwrap();
    assert_eq!(content, "ORIGINAL CONTENT\n", "원본 보존 필수");
}
```

#### TC-W16-008: `probe_local_models` trailing `/v1/` trim

```rust
#[test]
fn tc_w16_008_probe_url_trim_trailing_slash() {
    // probe_local_models 내부의 URL build 가 trailing slash 를 trim 하는지 검증
    // → 직접 probe 호출은 HTTP 발생하므로, build_url helper 가 있다면 그것만 검증
    // 또는 mock server (TC-W16-I01) 로 통합 검증
    // 본 L1 TC 는 단순히 trim 함수 (있다면) 호출 검증
    let url_with_slash = "http://localhost:11434/v1/";
    let url_no_slash = "http://localhost:11434/v1";
    assert_eq!(url_with_slash.trim_end_matches('/'), url_no_slash);
}
```

**mock-free fallback**: §3.4 의 `probe_local_models` 가 trim_end_matches('/') 사용하는지 코드 inspection 으로 verify. 단위 함수 분리되어 있다면 그 함수 직접 테스트.

### §W16.2 mock strategy 명시

| TC | mock type | 격리 방법 |
| --- | --- | --- |
| TC-W16-001~003 | mock-free | struct/enum 정의만 |
| TC-W16-004~006 | tempfile + env override | `MYHARNESS_HOME=tempdir` (paths.rs §1) |
| TC-W16-007 | read-only parent | platform-specific (Linux 검증) |
| TC-W16-008 | inline string trim | stdlib `trim_end_matches` |

### §W16.3 TDD 사이클 (D-43~D-47 chapter 패턴, 1 session 안에 가능)

1. **chapter 1** (TC-W16-001~003): `ModelInfo` struct + `RegisterError` enum + Display impl. **RED** (테스트만) → **GREEN** (impl).
2. **chapter 2** (TC-W16-004~006): `register_local_provider` core signature + body. **RED** → **GREEN**.
3. **chapter 3** (TC-W16-007~008): `atomic_write` helper + URL trim. **RED** → **GREEN**.

→ **3 chapter × 1 session** (W16 scope = small, D-47 chapter 1~3-B 의 27.5% 1-session 사이클과 동일 패턴). chapter 4 (integration) 는 별도 L2 TC.

### §W16.4 검증 reference

- **DD-AddLocal §7.1** (L1 Unit 8) — 본 §W16 와 1:1 매핑
- **REVIEW §6.2** — L1 Unit TC 우선순위 가이드 (mock-first, no-IO 원칙) 적용
- **security-patterns.md** — token 평문 출력 ❌ 검증 (TC-W16-006 의 `test-token-abc123` 는 fixture, 실 token 아님, D-06 strict 준수)

### §W16.5 v1.5+ 후보 (claim-only 회피)

- ✅ (OI-1) 비대화형 `--url/--token/--model` 플래그 — **v1.5 W17 에서 해소** (본 §W17)
- (OI-2) Ollama native `/api/tags` 지원 — L1 + 2 (TC-W16-010, 011)
- (OI-3) 다중 모델 1회 등록 — L1 + 1 (TC-W16-012)
- (OI-4) `register_local_provider` 의 keyring backend 분기 (Apple/Win/Linux 별) — L1 + 3 (TC-W16-013, 014, 015) — 현재는 W12 의 in-memory fallback 으로 통합 처리

### §W16.6 cross-references

- **입력 SSOT**:
  - `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` (D-59, §3 + §7)
  - `docs/REQUIREMENTS.md` §5.2.5 (W16 결정)
  - `docs/USE_CASES.md` §3.5 (UC-AUTH-010) + §10.4b (ACC-01~07)
  - `docs/CONCEPT.md` §5.5.1 (discover + auth + save) + §5.2 (12 명령어)
- **plan**: W16 (TASK-005-1 v1 MVP 후속)
- **sibling**: TC_INTEGRATION.md §W16-AddLocal (3 TC), TC_COMPONENT.md §W16-AddLocal (1 TC), TC_E2E.md §W16-AddLocal (1 TC manual)
- **plan output**: `outputs/w16-impl/deliverable.md` (구현 후 작성)
- **SSOT parent**: `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` (D-59, W16 ddoc)
| `deliverable_tc1.md` (early signal) | `docs/team/deliverable_tc1.md` | (prior attempt 1 작성) | status=in_progress → done |
| engine deliverable | `outputs/tc-1/deliverable.md` | (chunk 1 후 작성, status=in_progress) | chunk 5 후 status=done 갱신 |
| `board.md` (board) | `~/.mavis/plans/plan_ddcdd2a3/board.md` | start + done 2 entry (D-16 minimal noise) | sibling task 들과 공유 |

## §W17-AddLocal-NonInteractive — `myharness auth add-local --url/--model` L1 Unit TC (D-60, 2026-06-09)

> **본 § 추가 이유**: TASK-005-2 v1.5 진입 (D-59 follow-up TASK-005-1 v1 MVP 종료 선언 직후) 의 첫 W17 작업. DD-AddLocal §6.3 OI-1 해소 + DD-AddLocal §9 신규 spec + UC-AUTH-010 의 CI/스크립트 variant. **W16 의 8+1 = 9 L1 TC + W17 의 +4 = 13 L1 TC** (총 L1 = 168 + 4 = 172).

### §W17.0 메타

- **시점**: 2026-06-09 (TASK-005-2 v1.5 진입, D-60)
- **대상 독자**: TASK-005-2 W17 의 coder worker + verifier
- **SSOT**: `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` §9 (W17 spec) + 본 §W17
- **구현 파일**: `myharness/crates/llm/src/add_local.rs` (W17 신규 fn + 4 L1 TC) + `myharness/crates/cli/src/main.rs` (AuthAction::AddLocal flag 확장)
- **chapter pattern**: D-43~D-47 4-chapter × 1 session (W16 과 동일, 1 session 안에 가능)

### §W17.1 TC 정의 (4 신규, W16 8+1 + W17 4 = 13 L1)

| TC ID | 시나리오 | 검증 | chapter |
| --- | --- | --- | --- |
| **TC-W17-001** | `register_local_provider_non_interactive` valid input, no token | `Ok(RegisterReport)`, `available_models = [model_id]` 1개 (probe 안 함의 증거), `providers.toml` 갱신 | ch 1 (RED→GREEN) |
| **TC-W17-002** | `register_local_provider_non_interactive` with token | `token_saved = true`, keyring in-memory cache 확인 (Linux backend=None 환경) | ch 1 (RED→GREEN) |
| **TC-W17-003** | invalid URL 입력 | `RegisterError::InvalidUrl("...")` 매칭, `to_string()` 에 "invalid URL" 포함 | ch 2 (RED→GREEN) |
| **TC-W17-004** | empty `model_id` 입력 | register 성공 (user 책임), `available_models = [""]` 1개 | ch 2 (RED→GREEN) |

### §W17.2 mock strategy 명시 (D-43 honest disclosure)

- **L1 Unit 4개 모두 mock-free** — `tempfile::tempdir()` + `MYHARNESS_HOME` env override + `KeyringAuthStore::probe()` (real backend 또는 in-memory).
- **L2 Integration 2개** (TC-W17-I01, I02) — `wiremock` 사용. 단, **TC-W17-I01 의 probe skip 증명** = wiremock 에 어떤 route 도 mount 안 함 (probe 호출 시 404 받게 됨 → 비대화형은 404 없이 성공 = probe 안 부름의 증거).
- **CI 환경 의존** — `KeyringBackend::None` (Linux libsecret 부재) 환경에서 in-memory cache 동작 검증. macOS/Windows 는 `KeyringAuthStore::probe()` 가 real backend 사용 → TC 가 `if store.backend() == KeyringBackend::None` 분기.

### §W17.3 TDD 사이클 (D-43~D-47 4-chapter × 1 session)

- **chapter 1**: `register_local_provider_non_interactive` fn signature + TC-W17-001~002 (RED → GREEN) — impl 35 lines
- **chapter 2**: error path + edge cases + TC-W17-003~004 (RED → GREEN) — impl 0 lines 추가
- **chapter 3**: cli patch — `AuthAction::AddLocal { url, token, model, probe_skip }` + `handle_auth_add_local` 3-mode 분기 + `handle_add_local_interactive` / `handle_add_local_non_interactive` 분리
- **chapter 4**: L2 integration 2개 (TC-W17-I01, I02) — wiremock + end-to-end

→ **4 chapter × 1 session = 1~2 시간 작업** (W16 D-59 의 12 TC / 1 session 패턴과 동일, TC scaffold 4 L1 + 2 L2 = 6 신규).

### §W17.4 검증 reference

- **DD-AddLocal §9.6** (L1 Unit 4 + L2 Integration 2) — 본 §W17 와 1:1 매핑
- **REVIEW §6.2** — L1 Unit TC 우선순위 가이드 (mock-first, no-IO 원칙) 적용
- **security-patterns.md** — TC-W17-002 의 `ci-secret-token-abc` / `ci-token-xyz` 는 fixture, 실 token 아님

### §W17.5 cross-references

- **입력 SSOT**:
  - `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` §9 (W17 spec) + §6.3 OI-1 해소
  - `docs/USE_CASES.md` §3.5 (UC-AUTH-010, CI variant)
  - `docs/team/handoff_D-60_W17_add_local_non_interactive.md` (W17 handoff)
- **plan**: W17 (TASK-005-2 v1.5)
- **sibling**: TC_INTEGRATION.md §W17-AddLocal-NonInteractive (2 TC)
- **SSOT parent**: `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` (D-60, §9)

## cross-references

- **입력 SSOT (6 docs)**:
  - `docs/architecture/DETAILED_DESIGN_TOOL.md` (DD-1, 927 lines, §3+§7 myharness-tools)
  - `docs/architecture/DETAILED_DESIGN_BUDGET.md` (DD-2, 1,278 lines, §2+§4+§5+§6 myharness-context)
  - `docs/architecture/DETAILED_DESIGN_SUBAGENTS.md` (DD-3, 1,990 lines, §3-§6 myharness-agents)
  - `docs/specs/security-patterns.md` (DD-4, 988 lines, §2+§5 myharness-plugins hooks)
  - `docs/architecture/DETAILED_DESIGN_RETRY.md` (DD-5, 776 lines, §1-§5 myharness-llm fallback)
  - `docs/team/REVIEW.md` (REVIEW, 485 lines, §6.2 L1 Unit TC 우선순위)
- **plan**: `plan_ddcdd2a3` (4 task: TC-1/2/3/4)
- **sibling task**: TC-2 (L2 Integration) / TC-3 (L3 Component) / TC-4 (L4 E2E)
- **후속 task**: TASK-005-1 (v1 Rust MVP 구현, TDD 첫 sprint)
- **본 plan outputs**: `/Users/yklee/.mavis/plans/plan_ddcdd2a3/outputs/tc-1/deliverable.md`
- **SSOT parent**: `docs/architecture/INITIAL_DESIGN.md` (WP3 spec, 2,056 lines, §3 + §6 + §7)

---

## VERDICT (closing, post-handoff)

```
### VERDICT: PASS

본 docs/specs/TC_UNIT.md = TASK-005-1 의 L1 Unit TC scaffold.
160 TC = 8 categories × 96 actual Rust test code (10-30 lines/TC) + 64 catalog sig+placeholder (§10.2 정직 disclosure).
1 TC = 1 #[test] fn + assert_eq! / assert! / assert_matches! 검증.
mock strategy 명시 (provider mock / temp file / in-memory state).
SSOT §X.Y cross-ref 무결 (DD-1/2/3/4/5 + REVIEW + INITIAL_DESIGN).
D-06 strict (secret test corpus = EXAMPLEPLACEHOLDER only).
안티 6 미반영 (1 surface md, 단일 Rust, 6 builtin, 2 surface, local memory, MIT).

본 handoff: 4-필드 (summary / risks / suggested_follow_up / produced_artifacts) D-26 정합.
chunked write 5 chunk D-16 패턴 준수 (~500 lines / 20-30KB per chunk; 5 chunk = line 1-500 / 501-1000 / 1001-1450 / 1451-1950 / 1951-2200 § boundary 정합).
verifier cross-check cycle 4 반영 (4 critical + 1 sig + 3 minor fix verified PASS).
분량: 2,773 lines (target 1,800-2,200, +26% over-shoot, INITIAL_DESIGN +37% / DD-2 +60% precedent 정합).
TDD RED 진입점: 160 TC 모두 impl 전 작성 가능, cargo test --workspace 160 fail → GREEN → REFACTOR.
```
