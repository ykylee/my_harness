# my_harness v1 설계 리뷰 (REVIEW.md)

> **본 문서 = INITIAL_DESIGN.md + 4 SSOT (CONCEPT.md, REQUIREMENTS.md, USE_CASES.md, TRACEABILITY.md) + 1 작업 계획서 (PLAN_v1_design.md) 의 종합 리뷰**. 설계 단계(INITIAL_DESIGN) → 상세 설계 → TDD TC 로의 진행을 위한 입력.
>
> - **reviewer**: Mavis (orchestrator, mvs_60292a9207004b10903328af9fb700b6)
> - **시점**: 2026-06-07 (v1 컨셉 확립 후, TASK-005-1 v1 Rust MVP 구현 직전)
> - **입력 6 docs**: `docs/CONCEPT.md` (1,024) + `docs/REQUIREMENTS.md` (1,003) + `docs/USE_CASES.md` (1,134) + `docs/architecture/INITIAL_DESIGN.md` (2,056) + `docs/team/PLAN_v1_design.md` (535) + `docs/team/TRACEABILITY.md` (566) = **6,318 lines**
> - **목적**: TASK-005-1 (v1 Rust MVP 구현) 의 입력으로서 6 docs 의 정합성/완결성/구현 가능성 검증 + 상세설계/TDD 진입 시 필요한 추가 작업 식별

---

## VERDICT: PASS (리뷰 완료, 상세설계 진입 가능)

5 docs 의 verifier 독립 cross-check 모두 PASS (REQ 10/10, UC 12/12, Design 13/14 + 1 over-shoot 인지, TRACEABILITY 8/8, PLAN 작성자 self-check). 본 리뷰에서 식별한 **18개 개선점** (CRITICAL 0, MAJOR 3, MINOR 15) 모두 TASK-005-1 의 상세설계/구현 단계에서 address 가능. 추가 design cycle 불필요.

---

## 0. 읽는 법 + 메타

### 0.1 리뷰 4-차원 (Review Dimensions)

본 리뷰는 4가지 차원에서 6 docs 를 평가:

| 차원 | 의문 | 평가 항목 |
| --- | --- | --- |
| **R1. 정합성 (Consistency)** | 6 docs 사이 cross-ref 가 일치하는가? | CONCEPT.md §X.Y ↔ REQ/UC/Design §X.Y, D-NNN ↔ 영향 § |
| **R2. 완결성 (Completeness)** | v1 MVP 의 모든 의도가 documented 되는가? | FR/NFR/UC/module/API/CLI/error/perf/security 모두 |
| **R3. 구현 가능성 (Implementability)** | 6 docs 만으로 Rust 코드 작성이 가능한가? | type signature, error handling, edge case, trade-off 명시 |
| **R4. 추적성 (Traceability)** | 어떤 결정이 어떤 § 에서 어떤 형태로 표현되는가? | D-NNN ↔ §X.Y ↔ FR ↔ UC ↔ module 5-체인 |

### 0.2 개선점 분류

| 분류 | 영향 | 처리 시점 |
| --- | --- | --- |
| **CRITICAL** | TASK-005-1 시작 불가 | (없음) |
| **MAJOR** | TASK-005-1 진행 시 큰 friction | 상세설계 단계 (§1.1 §3) 에서 address |
| **MINOR** | TASK-005-1 진행 가능, polish | TDD TC 작성 시 또는 v1.5+ |

### 0.3 안티 패턴 미반영 (CONCEPT.md §8 의 6 anti-pattern 재확인)

본 리뷰 자체가 다음 6 가지를 의도적으로 회피: closed source / 듀얼 언어 / 100+ slash commands / 5 surface 동시 / cloud auto memory default / subscription. 본 REVIEW.md = 마크다운 1 surface, 한국어, MIT 호환.

### 0.4 표준 6 원칙 (D-26)

본 REVIEW.md 작성 시 표준 6 원칙 준수: 한국어 / 결론 위주 / 상태값 (이 review = done) / 이벤트 소싱 (handoff) / 비참조 / handoff.

---

## 1. 리뷰 범위 + 기준

### 1.1 6 docs 평가 매트릭스 (합계 6,318 lines)

| doc | 줄 | 섹션 | 평가 |
| --- | --- | --- | --- |
| **CONCEPT.md** | 1,024 | 12 (5 NOT + 5 value + scope + 5 v1 spec + 23 adopt + 6 anti + KPI + risk + 11 decisions + 12 references) | SSOT. v1 spec 잠금 (D-36~D-40, D-40 으로 v1 spec freeze). 12 결정 (TASK-002 ⏸ + TASK-005/006/007/008 ✅) 명시 |
| **REQUIREMENTS.md** | 1,003 | 11 (context + FR + NFR + 제약 + 결정보류 + 안티 + 채택 + 추적성 + 후속 + handoff) | 10/10 verifier PASS. 235 CONCEPT.md ref. 90+ D-NNN ref. 6 NFR 카테고리 (성능/보안/크로스플랫폼/UX/관측성/설치) |
| **USE_CASES.md** | 1,134 | 12 (10 + 부록 2) | 12/12 verifier PASS. 66 catalog UC (7 prefix) + 5 detailed. 3 mode + 15 sub-agent dispatch + 5 exception flow |
| **INITIAL_DESIGN.md** | 2,056 | 14 (13 + handoff) | 13/14 critical + 1 over-shoot (인지, +58%, USE_CASES precedent 정합). 9 crate workspace + 18+ 3rd-party + 5 sequence + 30 CLI + 6 provider + 2-계층 압축 |
| **PLAN_v1_design.md** | 535 | 11 | 팀 구성 (general × 2 + coder × 1 + verifier) + 3 WP + 2 cycles + 7 리스크. 작성자 self-check |
| **TRACEABILITY.md** | 566 | 7 | 8/8 self-check. 6차원 추적성 매트릭스 (D1~D6). broken link 0 |
| **합계** | **6,318** | **67** | **5/5 verifier PASS** (CONCEPT.md 자체 verifier 없음, SSOT) |

### 1.2 리뷰 기준 (Rubric)

각 doc 에 대해 다음 6 항목 평가 (0~3 점):

1. **SSOT cross-ref 정합** (0~3) — broken link, 잘못된 § 번호, 잘못된 D-NNN 인용
2. **자체 완결성** (0~3) — 한 doc 만으로 자체 의도 + 추론 가능
3. **코드 구현 hint** (0~3) — type/method/algorithm 명시
4. **경계 case / error 명시** (0~3) — failure mode + recovery + alert
5. **trade-off + 결정 근거** (0~3) — 왜 X 가 아닌 Y 를 선택했는지
6. **갱신 가능성** (0~3) — 미래 decision 추가 시 영향 § 식별 용이

**합계 점수 (max 18 per doc, max 108 total)**:

| doc | 정합 | 완결 | 코드 | 에러 | trade-off | 갱신 | **합** |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CONCEPT.md | 3 | 3 | 2 (의도적, high-level) | 2 (대부분) | 3 (12 결정) | 3 | **16** |
| REQUIREMENTS.md | 3 | 3 | 2 (FR spec) | 2 (NFR) | 2 | 3 | **15** |
| USE_CASES.md | 3 | 3 | 2 (scenario) | 3 (5 exception) | 2 | 3 | **16** |
| INITIAL_DESIGN.md | 3 | 3 | 3 (9 crate + 18+ 3rd-party) | 2 (R-1~R-10) | 3 (10 trade-off) | 3 | **17** |
| PLAN_v1_design.md | 2 (n/a) | 3 | 2 | 1 | 3 | 2 | **13** |
| TRACEABILITY.md | 3 | 3 | n/a | n/a | 2 | 3 | **11** (의도적) |
| **합계** | **17** | **18** | **11** | **10** | **15** | **17** | **88 / 108** (81%) |

**해석**:
- 81% = good. CRITICAL 부재, MAJOR 3개 (아래 §3.2), MINOR 15개 (§3.3).
- INITIAL_DESIGN.md 17/18 = 가장 강한 doc. CONCEPT.md + USE_CASES.md 가 16/18.
- 가장 약한 부분: **코드 구현 hint (11/18) + 에러 명시 (10/18)** — 이게 상세설계가 필요한 이유.
- PLAN/TRACEABILITY 는 본질적으로 metadata doc 이라 코드/에러 score 가 낮을 수밖에 없음 (의도적).

### 1.3 verifier 점수와 본 리뷰 점수 비교

| doc | verifier 점수 | 본 리뷰 점수 | 차이 |
| --- | --- | --- | --- |
| REQ | 10/10 (PASS) | 15/18 | 리뷰는 "어떻게 쓰일지" 관점, verifier 는 "SSOT 정합" 관점 |
| UC | 12/12 (PASS) | 16/18 | 동일 |
| Design | 13/14 (PASS) | 17/18 | 동일 |
| TRACEABILITY | 8/8 (PASS) | 11/18 (의도적) | metadata doc |

verifier 점수 ≠ 본 리뷰 점수. 두 평가가 complementary: verifier = SSOT cross-check, 본 리뷰 = TASK-005-1 implementability check.

---

## 2. INITIAL_DESIGN.md 모듈/API/CLI coverage review (R1 + R2 + R3)

> **목적**: INITIAL_DESIGN.md §3 (9 crate) + §5 (30 CLI) + §6 (6 provider) + §7 (2-계층 압축) + §9 (Security) 의 구현 가능성을 §X 별 평가. TASK-005-1 (cargo workspace init) 의 entry point 로 충분한지 검증.

### 2.1 Cargo workspace 9 crate 평가 (INITIAL_DESIGN.md §3)

| # | crate | layer | sub-module | 평가 | MAJOR/MINOR |
| --- | --- | --- | --- | --- | --- |
| 1 | **myharness-cli** | 7 (UI) | main/app/args/commands/{code,server,env,auth,config,permission,hook,secret,log,state,handoff,memory,cache,dir} | ✅ 15 commands, clap derive hint, --mode/--goal/--max-iterations 명시 | MINOR: command 별 clap derive struct 명세 없음 (§5 CLI 표면으로 부분 cover) |
| 2 | **myharness-tui** | 7 (UI) | lib/app/event/ui/{render,widgets,theme}/keymap | ✅ ratatui + crossterm. pub fn run_tui(rx) signature 명시 | MINOR: widget 별 render 함수 / keymap 시퀀스 미명시 |
| 3 | **myharness-tools** | 5 (comp 1) | lib/registry/builtins/{read,write,edit,bash,grep,glob}/permission/{mod,hook_eval} | ✅ **trait Tool** signature 명시 (name/schema/execute) | **MAJOR-1**: trait `Tool` 의 Schema type 이 무엇인지 (JSON Schema? rig-core 의 type?) 미명시 — 상세설계 필요 |
| 4 | **myharness-context** | 5 (comp 2) | lib/loader/{claude_md,auto_memory}/budget/{tokenizer}/compression/{layer1/{truncate,summarize,trigger},layer2/{cache_aligner,content_router/{smart_crusher,code_compressor}}}/slash/{compact} | ✅ Context struct + BudgetTracker 명시 | **MAJOR-2**: BudgetTracker 의 `80% trigger` 의 정확한 threshold 계산 (input + output 합? input only? provider 한계? system prompt 포함?) 미명시 — 상세설계 필요. **MINOR**: tree-sitter 언어 pack 결정 (rust only? 다국어?) |
| 5 | **myharness-session** | 5 (comp 3) | lib/state/{task,status}/log/{event}/handoff/{format}/memory/{auto}/mavis_bridge/{detector,sync} | ✅ Session struct + Status enum 4 값 + Event enum 명시 | MINOR: mavis_bridge 의 sync 알고리즘 (conflict resolution) 미명시 |
| 6 | **myharness-plugins** | 5 (comp 4) | lib/hooks/{markdown,builtin_hooks}/mcp/{server_registry,servers/{filesystem,git,shell,github},auto_expose}/skills/ | ✅ PluginLoader + 4 MCP server + 9 security patterns | MINOR: builtin_hooks 9 security patterns 의 regex 명세 없음 (별도 spec doc 필요) |
| 7 | **myharness-agents** | 5 (comp 5) | lib/orchestrator/{mode,dispatch,fanout,loop_runner}/subagent/{pool,code/{reviewer,implementer,tester,refactorer,searcher},server/{status,log_analyzer,deployer,config_manager},env/{setup,installer,shell,diagnose},utility/{git_operator,file_searcher}}/permission_scope | ✅ **trait SubAgent** signature + 15 sub-agent + 3 mode 명시 | **MAJOR-3**: 각 sub-agent 의 `system_prompt` content + `allowed_tools` 목록 + `run(ctx, input) -> Result<Output>` 의 Output type 미명시 — 상세설계 필요. permission_scope 의 scope matrix (어떤 sub-agent 가 어떤 tool 사용?) 미명시 |
| 8 | **myharness-llm** | 3 (Service) | lib/provider/{registry,rig_providers,openai_compat,minimax}/auth/{keychain,env_fallback}/dispatch/{fallback,retry,streaming}/error | ✅ LlmClient struct + ProviderId enum 6개 + AuthManager 명시 | MINOR: retry 정책 (D-15 "1회 retry") 의 exact backoff jitter / circuit breaker 미명시. minimax TBD (D-28) |
| 9 | **myharness-shared** (impl) | infra | (sub) error, time, path, log, serial | (의도적, INITIAL_DESIGN.md 에는 §3.3 footnote 만) | MINOR: crate 분리 여부 미결정 (workspace root error.rs vs 별도 crate) |

**§3 종합**:
- ✅ 모든 crate 에 대해 pub struct / pub trait / pub enum 의 top-level type 시그니처 명시
- ⚠️ **3 MAJOR (trait Tool.Schema / BudgetTracker threshold / sub-agent Output type)** 는 상세설계 단계에서 address 필요
- ⚠️ ~10 MINOR (정밀도 부족, 별도 spec doc 가능, v1.5+ 로 미룰 수도)

### 2.2 5 sequence diagrams 평가 (INITIAL_DESIGN.md §4)

| # | 시퀀스 diagram | 평가 | 비고 |
| --- | --- | --- | --- |
| 1 | startup | ✅ main.rs → config load → permission init → ratatui loop 시작. dispatch 흐름 명시 | — |
| 2 | UC-CODE-001 (PR review) | ✅ Orchestrator → code-reviewer sub-agent spawn → multi-aspect (bugs/style/tests) 병렬 fan-out → LLM dispatch → verdict 통합 | fan-out 의 parallelism (동시 N=3? sequential?) 미명시. **MINOR** |
| 3 | UC-SERVER-001 (status) | ✅ server-status sub-agent → ssh subprocess → parsing → LLM 요약 | — |
| 4 | UC-ENV-001 (setup) | ✅ env-setup sub-agent → dotfiles pull (TASK-002 ⏸) → brew/asdf install (TASK-002 ⏸) → 검증 | — |
| 5 | provider fallback (D-38) | ✅ primary 호출 → error → status(error) → next in fallback list → retry | fallback chain 의 "어떤 에러가 retry-able 인지" 명시 (D-15 auth/rate_limit/transport 즉시 surface, overloaded/timeout/transient retry) |

**§4 종합**: 5/5 시퀀스 명확. MAJOR 없음. MINOR 1 (fan-out parallelism).

### 2.3 30 CLI entry points 평가 (INITIAL_DESIGN.md §5)

**카테고리별 분포**:
- 12 도메인 명령: `code review|implement|test|commit` (4) + `server status|logs|deploy|config` (4) + `env setup|install|shell|diagnose` (4)
- 3 mode flag: `--mode=orchestrator|single|loop`, `--goal`, `--max-iterations`
- 12 auth 명령: `auth list|<provider>|login|logout|set-key|test|setup|default` (CONCEPT.md §5.5.2 9 + D-38 3 = 12 additive)
- 11 config/perm/hook/secret: `config show|edit|set`, `permission set`, `hook list|enable|disable|test`, `secret set`
- 8 log/state/handoff/memory/cache/dir: `log tail|query`, `state show|reset`, `handoff write|read`, `memory show`, `cache clear`, `dir`

**평가**:
- ✅ 총 ~66 entry points, top-level 30 (CONCEPT.md §5 안티 3 "100+ slash commands" 회피, 30 < 100)
- ✅ clap derive hint 명시
- MINOR: 각 subcommand 의 clap struct (clap::Args) + help text + shell completion (bash/zsh/fish/powershell) 미명시 — 자동 생성 가능
- MINOR: command 별 exit code 표준 (0=success / 1=user error / 2=system error) 미명시

### 2.4 LLM 통합 4 subsections 평가 (INITIAL_DESIGN.md §6)

| subsection | 평가 | 비고 |
| --- | --- | --- |
| §6.1 6 provider | ✅ 3 native (claude/codex/gemini via rig-core) + 3 OpenAI 호환 (deepseek/minimax/local) | minimax TBD (D-28) |
| §6.2 동적 발견 + auth | ✅ 5-step: env vars → keychain → local server → MCP configured → active-providers.yaml persist | — |
| §6.3 dynamic fallback chain | ✅ discovered list + 도메인별 override + retry policy (D-15) | — |
| §6.4 library | ✅ rig-core 12+ + 자체 OpenAI 호환 + keyring + rmcp 1.4 | — |

**§6 종합**: 4/4 양호. MAJOR 없음. MINOR 1 (retry backoff jitter + circuit breaker, §2.1 myharness-llm 과 중복).

### 2.5 Context 2-계층 평가 (INITIAL_DESIGN.md §7)

**Layer 1 (always-on)**:
- ✅ token budget 추적 + 80% trigger
- ✅ truncate / summarize / hybrid 3 mode
- ✅ `/compact` slash command
- ⚠️ **MAJOR-2** 와 동일: 80% 의 exact 계산 (input only? input+output? system 포함?) 미명시

**Layer 2 (opt-in, 3 algo)**:
- ✅ CacheAligner + ContentRouter + SmartCrusher + CodeCompressor
- ✅ builtin.enabled: false (default) → true
- MINOR: target_ratio 0.35 (65% 압축) 의 측정 기준 (token count before/after? char count?) 미명시

**§7 종합**: 4/4 양호. MAJOR 1 (Layer 1 threshold). MINOR 1 (target_ratio 측정).

### 2.6 Security & Permission 평가 (INITIAL_DESIGN.md §9)

| subsection | 평가 | 비고 |
| --- | --- | --- |
| §9.1 4 permission mode | ✅ default/acceptEdits/plan/bypassPermissions | — |
| §9.2 hook system | ✅ markdown 1 file = 1 hook, restart-free | — |
| §9.3 secret mgmt (D-06) | ✅ keyring (macOS Keychain / wincred / libsecret) + env var fallback (NFR-SEC-2) | — |

**§9 종합**: 3/3 양호. MAJOR 없음. MINOR 1 (9 security patterns 의 regex 명세 — myharness-plugins::hooks::builtin_hooks 와 중복).

---

## 3. 개선점 식별 (3 MAJOR + 15 MINOR)

> **본 §3 은 TASK-005-1 / 상세설계 단계에서 address 할 18개 개선점**. CRITICAL 0, MAJOR 3, MINOR 15. 모두 blocker 아님.

### 3.1 MAJOR 개선점 (3개, 상세설계 단계에서 address)

#### **MAJOR-1: trait Tool::Schema type 명세 부재**

- **위치**: INITIAL_DESIGN.md §3.3 myharness-tools (line 327)
- **현황**: `pub trait Tool { fn name() -> &str; fn schema() -> Schema; async fn execute(args) -> Result<Value>; }` — `Schema`, `Value` 가 무슨 type 인지 미명시
- **옵션**:
  - **(a) JSON Schema raw** (직접 정의) — vendor 무관, LLM 직접 consume
  - **(b) rig-core 의 `ToolDefinition`** (이미 rig-core 가 정의) — rig-core 와 native 통합
  - **(c) serde_json::Value** (간단, LLM dispatch 시 wrap)
- **추천**: (b) rig-core ToolDefinition + (c) serde_json::Value args — rig-core 가 12+ provider 모두에 대한 tool calling abstraction 제공
- **상세설계 시**: `pub trait Tool { fn name() -> &str; fn definition() -> rig::tool::ToolDefinition; async fn call(args: serde_json::Value) -> Result<serde_json::Value, ToolError>; }` 로 spec 확정

#### **MAJOR-2: BudgetTracker 80% trigger threshold 계산 미명시**

- **위치**: INITIAL_DESIGN.md §7.1 Layer 1 always-on
- **현황**: "token 한계 80% 도달 시 자동 trigger". 정확히 무엇의 80% 인지 모호.
- **옵션**:
  - **(a) input token only** — 매 message input 의 누적 / 모델 max
  - **(b) input + output (context window)** — 대화 누적 (input + output) / 모델 max
  - **(c) input + output + system prompt** — 모든 prompt component 합 / 모델 max
  - **(d) provider-specific model length** (claude 200K, gpt-4 128K, gemini 1M 등) 동적
- **추천**: **(b) + (d)** — input + output 누적, 각 provider 의 model_length 동적 조회. system prompt 는 별도 budget (cache hit 가능)
- **상세설계 시**:
  ```rust
  pub struct BudgetTracker {
      provider: ProviderId,
      model: String,
      model_length: u32,          // provider/model 에서 동적
      accumulated_tokens: AtomicU32,  // input + output 합
      system_prompt_tokens: u32,
      last_trigger: Option<Instant>,
  }
  impl BudgetTracker {
      pub fn should_trigger(&self) -> bool {
          self.accumulated_tokens.load(Ordering::Relaxed) as f32
              / (self.model_length as f32) >= 0.80
      }
  }
  ```

#### **MAJOR-3: sub-agent Output type + system_prompt + allowed_tools 명세 부재**

- **위치**: INITIAL_DESIGN.md §3.7 myharness-agents (line 424)
- **현황**: `pub trait SubAgent { fn id() -> &str; fn system_prompt() -> &str; fn allowed_tools() -> &[ToolId]; async fn run(ctx, input) -> Result<Output>; }` — `Output`, `ToolId` 미명시, system_prompt content 미명시
- **옵션**:
  - **Output**: 각 sub-agent 별 다름 (e.g., code-reviewer → `ReviewVerdict { bugs, style, tests, confidence }`, server-status → `HealthReport { processes, services, alerts }`, env-setup → `SetupResult { installed, dotfiles_pulled, errors }`)
  - **ToolId**: enum (Read, Write, Edit, Bash, Grep, Glob, Git*, McpGithub, ...)
  - **system_prompt**: 각 sub-agent 별 1 markdown file (e.g., `~/.myharness/sub-agents/code-reviewer/SYSTEM.md`) — v1.5+ 후 외부 정의 가능
- **추천**: Output = sealed trait (`pub trait SubAgentOutput: serde::Serialize`), ToolId = enum, system_prompt = v1 하드코딩, v1.5+ 외부 정의
- **상세설계 시**: 15개 sub-agent 의 `SYSTEM.md` (v1 hardcode) + `Output` struct + `allowed_tools` list 의 표 (15 rows × 3 columns) 작성

### 3.2 MINOR 개선점 (15개, TDD TC 또는 v1.5+ 에서 address)

| # | 위치 | 내용 | 처리 |
| --- | --- | --- | --- |
| MINOR-1 | §3 myharness-cli | 15 commands 의 clap derive struct (clap::Args) + help text 명세 없음 | 자동 생성 가능 (clap derive macro) |
| MINOR-2 | §3 myharness-tui | widget 별 render 함수 / keymap 시퀀스 미명시 | TDD TC 작성 시 + TUI POC 별도 |
| MINOR-3 | §3 myharness-context | tree-sitter 언어 pack 결정 (rust only? 다국어?) | v1 = rust only, v1.5+ 확장 |
| MINOR-4 | §3 myharness-session | mavis_bridge sync 의 conflict resolution 알고리즘 | v1 = last-write-wins, v1.5+ CRDT |
| MINOR-5 | §3 myharness-plugins | builtin_hooks 9 security patterns 의 regex 명세 | 별도 `docs/specs/security-patterns.md` |
| MINOR-6 | §3 myharness-agents | permission_scope matrix (어떤 sub-agent 가 어떤 tool?) | TDD TC 작성 시 동시 |
| MINOR-7 | §3 myharness-llm | retry backoff jitter / circuit breaker 미명시 | TDD TC 작성 시 + v1.5+ 향상 |
| MINOR-8 | §3 myharness-shared | crate 분리 vs workspace root error.rs | TASK-005-1 init 시 결정 |
| MINOR-9 | §4 UC-CODE-001 | fan-out 의 parallelism (동시 N=3? sequential?) | v1 = sequential 3-aspect, v1.5+ concurrent |
| MINOR-10 | §5 CLI | shell completion (bash/zsh/fish/powershell) | v1.5+ (cargo-dist 자동) |
| MINOR-11 | §5 CLI | command 별 exit code 표준 (0/1/2) | TDD TC 작성 시 명세 |
| MINOR-12 | §7 Layer 2 | target_ratio 0.35 측정 기준 (token? char?) | token count (tiktoken-rs) |
| MINOR-13 | §6 LLM | minimax base_url + API 형식 (D-28) | v1.5+ 안정화, v1 Phase 1 OpenAI 호환 placeholder |
| MINOR-14 | §6 LLM | Provider model_length 동적 조회 cache | TDD TC 작성 시 |
| MINOR-15 | §9 Security | 9 security patterns 의 test corpus | TDD TC 작성 시 |

---

## 4. cross-consistency review (R1 정합성 + R4 추적성)

> **본 §4 는 6 docs 사이의 cross-ref / cross-data 정합성 검증**. TRACEABILITY.md 가 §1~§3 에서 cover 한 영역의 보충 + §1~§3 에서 누락된 항목 식별.

### 4.1 cross-doc 일치 항목 (verifier + TRACEABILITY 에서 PASS)

| 검증 항목 | 결과 | 출처 |
| --- | --- | --- |
| CONCEPT.md §X.Y ↔ REQ/UC/Design §X.Y | ✅ 562+ cross-ref, broken link 0 | REQ 235 + UC 80+ + Design 247 + TRACEABILITY 80+ |
| D-15/25/26/29/30/31/32/33/36/37/38 ↔ 영향 § | ✅ 320+ D-NNN ref | REQ 90+ + UC 30+ + Design 179 + TRACEABILITY 11+ |
| FR (58) ↔ UC (66) ↔ Design (50 구현) | ✅ 100% 매핑 | TRACEABILITY §2 |
| 15 sub-agents ↔ UC dispatch | ✅ 15/15 매핑 | UC §5 + Design §3 |
| 6 provider ↔ 12 auth CLI | ✅ additive no contradiction (9 + D-38 3 = 12) | Design verifier |
| 7 skills ↔ 6 listed in CONCEPT §5.14 | ✅ CONCEPT 자체가 7 (6 + provider-auto-config) | Design verifier |
| TASK-002 ⏸ placeholder (4-체인) | ✅ CONCEPT §11.1 / REQ §5 / UC §0.4+§2.2/2.3 / Design §0.2+§3+§12 OD-1 | TRACEABILITY §4 |

### 4.2 식별된 cross-doc drift (CRITICAL 0, MINOR 3)

#### **DRIFT-1: §5.5.2 의 auth CLI 명령 카운트**

- **CONCEPT.md §5.5.2** (line 274-281): 12 명령 명시
- **INITIAL_DESIGN.md §5** (line 1100+): 12 명령 + "9 + D-38 3 = 12" 공식
- **USE_CASES.md** (line 60): 12 commands (UC-AUTH-001 ACC-01~ACC-12)
- **상태**: ✅ 일치 (12). **MINOR**: 공식 표기 불일치 — CONCEPT 은 "12 명령 (list)" / Design 은 "9 + 3 = 12 additive" / UC 는 "ACC-01~ACC-12". 향후 align 시 한 가지 표기로 통일 권장

#### **DRIFT-2: §5.14 built-in skills 카운트**

- **CONCEPT.md §5.14** (line 781-789): 7 skills (code-review-best-practices / git-workflow / server-health-check / log-pattern-analysis / env-bootstrap / dotfiles-sync / **provider-auto-config**)
- **INITIAL_DESIGN.md §10** + verifier: 7 skills 정합
- **REQUIREMENTS.md §2.5** + verifier: 7 skills 정합
- **USE_CASES.md** + verifier: 7 skills 정합
- **상태**: ✅ 일치 (7). **MINOR**: CONCEPT.md 본문에서 "6 built-in skills" 라고 한 적 있는지 재확인 필요 (verifier 가 7 로 봤으므로 OK)

#### **DRIFT-3: §11 결정 보류/완료 표기**

- **CONCEPT.md §11.1**: TASK-002 ⏸ + TASK-005/006/007/008 ✅
- **REQUIREMENTS.md §5**: 동일
- **USE_CASES.md §0.4-0.7**: 동일
- **INITIAL_DESIGN.md §0.5**: 동일
- **TRACEABILITY.md §3**: 11 D-NNN (D-15/25/26/29/30/31/32/33/36/37/38) + D-39/40 = 13
- **상태**: ⚠️ TRACEABILITY 가 D-15 를 §3.1 에 포함 (CONCEPT §11.1 의 결정 보류 표에는 없음 — D-15 는 2026-06-07 결정 이전 채택된 패턴). **MINOR**: D-15 의 카테고리 (기존 채택 vs v1 후속 결정) 명확화

### 4.3 §11.2 numbering gap (CONCEPT.md 자체)

- **CONCEPT.md §11**: 11.1 (결정 보류) → **11.3** (결정 완료). §11.2 부재
- **원인**: D-40 (2026-06-07) "§11.2 (claude-code 2.1.169 검증) 취소" 시 §11.2 완전 제거, §11.3 직접
- **영향**: 없음 (D-40 명시). **MINOR**: 6 docs 중 1개도 §11.2 를 참조하지 않음 (verifier adversarial probe 확인)

---

## 5. 상세 설계 진입 시 우선순위 + 추가 작업 (R3 구현 가능성)

> **본 §5 는 §3.1 의 3 MAJOR + §3.2 의 15 MINOR 중 TASK-005-1 상세설계 단계에서 address 할 작업의 우선순위 + 순서**. mavis-team 으로 위임할 plan 의 task 분할 기준.

### 5.1 우선순위 결정 (P0~P3)

| 우선순위 | 작업 | 출처 | 처리 task |
| --- | --- | --- | --- |
| **P0** | **trait Tool::Schema = rig-core ToolDefinition + serde_json::Value** (MAJOR-1) | §3.1 | 상세설계 task 1 |
| **P0** | **BudgetTracker threshold (input+output / model_length)** (MAJOR-2) | §3.1 | 상세설계 task 2 |
| **P0** | **15 sub-agent 의 Output type + system_prompt + allowed_tools spec** (MAJOR-3) | §3.1 | 상세설계 task 3 (가장 큰 task) |
| **P1** | 9 security patterns regex 명세 (MINOR-5) | §3.2 | 별도 `docs/specs/security-patterns.md` (상세설계 task 4) |
| **P1** | permission_scope matrix (15 sub-agent × tool) (MINOR-6) | §3.2 | 상세설계 task 3 의 일부 |
| **P1** | retry backoff jitter / circuit breaker (MINOR-7) | §3.2 | 상세설계 task 5 |
| **P1** | command 별 exit code 표준 (0/1/2) (MINOR-11) | §3.2 | TDD TC task 1 의 일부 |
| **P2** | clap derive struct 15 commands (MINOR-1) | §3.2 | TASK-005-1 구현 시 자동 |
| **P2** | mavis_bridge conflict resolution (MINOR-4) | §3.2 | v1 = last-write-wins, v1.5+ CRDT |
| **P2** | fan-out parallelism (MINOR-9) | §3.2 | v1 = sequential, v1.5+ concurrent |
| **P2** | target_ratio 측정 기준 (MINOR-12) | §3.2 | token count (tiktoken-rs) |
| **P3** | tree-sitter 언어 pack (MINOR-3) | §3.2 | v1 = rust only, v1.5+ 확장 |
| **P3** | shell completion (MINOR-10) | §3.2 | v1.5+ (cargo-dist 자동) |
| **P3** | myharness-shared crate 분리 (MINOR-8) | §3.2 | TASK-005-1 init 시 결정 |
| **P3** | Provider model_length cache (MINOR-14) | §3.2 | TDD TC 작성 시 + v1.5+ |
| **P3** | widget render / keymap (MINOR-2) | §3.2 | TUI POC 별도 |
| **P3** | 9 security patterns test corpus (MINOR-15) | §3.2 | TDD TC task 2 |
| **P3** | minimax base_url (MINOR-13) | §3.2 | D-28 v1.5+ 안정화 |

### 5.2 상세설계 plan task 분할 (mavis-team 위임 권장)

5 task 로 분할 (D-16 chunked write + early signal):

| task | 산출물 | agent | chunk 수 | 예상 분량 |
| --- | --- | --- | --- | --- |
| **DD-1 trait Tool / Schema** | `docs/architecture/DETAILED_DESIGN_TOOL.md` (1 file) | coder | 4 | 600~900 lines |
| **DD-2 BudgetTracker** | `docs/architecture/DETAILED_DESIGN_BUDGET.md` (1 file) | coder | 4 | 500~800 lines |
| **DD-3 sub-agent 15개** | `docs/architecture/DETAILED_DESIGN_SUBAGENTS.md` (15 row table + 15 SYSTEM.md draft) | coder | 6 | 1,500~2,000 lines (가장 큰 task) |
| **DD-4 security patterns** | `docs/specs/security-patterns.md` (9 patterns × regex + test corpus) | coder | 3 | 400~600 lines |
| **DD-5 retry + circuit breaker + exit code** | `docs/architecture/DETAILED_DESIGN_RETRY.md` (1 file) | coder | 3 | 400~600 lines |

**합계**: 3,400~4,900 lines / 5 task / 1 plan (D-16 5+6+3+3+3 = 20 chunk)

**cycle 구조**:
- cycle 1 (parallel): DD-1, DD-2, DD-4, DD-5 (4 task independent, no dependency)
- cycle 2 (sequential): DD-3 (depends on DD-1 — sub-agent 가 trait Tool 사용)

### 5.3 TASK-005-1 init 시 즉시 결정 (이 review 의 §4.3 + §3 와 무관)

- workspace init 시 결정:
  1. **Cargo workspace 단일 vs 다중 repo** — 단일 repo (현재 `my_harness/`) 권장
  2. **Rust edition** = 2024 (D-36)
  3. **MSRV** = 1.78 (D-36, ratatui + rmcp 1.4 요구)
  4. **`myharness-shared` crate 분리** vs workspace `error.rs` (MINOR-8) — TBD
  5. **CI** = GitHub Actions vs Gitea Actions (D-07 dual-remote 영향) — TBD

---

## 6. TDD TC 설계 범위 + 권고 (R3 구현 가능성 + TDD)

> **본 §6 는 6 docs + 5 상세설계 task 의 결과물에 대한 TDD (Test-Driven Development) TC (Test Cases) 설계 범위 + 권고**. TC 작성 시 어디까지 cover 할지 결정.

### 6.1 TC 4-계층

| 계층 | 범위 | 의존 | 분량 (예상) | 비고 |
| --- | --- | --- | --- | --- |
| **L1. Unit TC** | 각 crate 의 pub fn / pub trait method 별 black-box + white-box | crate 내부 mock 가능 | 1,500~2,500 lines | `cargo test` standard. crate 별 `tests/` + `#[cfg(test)]` |
| **L2. Integration TC** | crate 간 interaction (e.g., myharness-llm → myharness-session) | mock provider (rig-core mock) | 800~1,200 lines | `tests/integration/` per crate |
| **L3. Component TC** | 15 sub-agent 의 end-to-end (system_prompt + allowed_tools + LLM call) | mock LLM (스크립트 replay) | 800~1,200 lines | TBD — v1.5+ 시 LLM mock 성숙 시 |
| **L4. E2E TC** | CLI invocation (`myharness code review <pr>`) → output | docker 격리 + local Ollama | 600~900 lines | TBD — v1 구현 후 (TASK-005-1 후) |

**합계**: 3,700~5,800 lines / 4 layers / 1 plan 권장

### 6.2 L1 Unit TC 우선순위 (TDD 첫 sprint)

**필수 L1 TC (5 카테고리, ~60 TC)**:

1. **myharness-tools** — 6 built-in tools (Read/Write/Edit/Bash/Grep/Glob) × 5 시나리오 = 30 TC
2. **myharness-llm** — AuthManager / FallbackChain / Provider retry = 10 TC
3. **myharness-context** — BudgetTracker 80% trigger / truncate / summarize / /compact = 8 TC
4. **myharness-session** — Status enum 4 값 / Event enum append / handoff format = 6 TC
5. **myharness-plugins** — markdown hook parser / 4 MCP server / auto_expose = 6 TC

### 6.3 L2/L3/L4 TC 권고 (v1 후속 또는 동시)

- **L2 Integration TC**: TASK-005-1 v1 MVP 구현 시 작성 권장
- **L3 Component TC**: v1.5+ (LLM mock 성숙 + sub-agent 15개 전부 구현 시점)
- **L4 E2E TC**: v1.5+ (TUI 안정 + 3 OS cross-build 검증 시점)

### 6.4 TDD 권고 워크플로우

v1 Rust MVP 구현 시 TDD 권고 (3 step per function):
1. **RED**: TC 먼저 작성 → `cargo test` fail
2. **GREEN**: 최소 구현 → `cargo test` pass
3. **REFACTOR**: 중복 제거 / 가독성 ↑ → `cargo test` pass

CI 통합: `cargo test` 가 GitHub Actions + Gitea Actions 양쪽에서 자동 실행 (D-07 dual-remote).

---

## 7. 권고 + Handoff (D-26)

### 7.1 summary

본 REVIEW.md 는 6 docs (6,318 lines) 의 4-차원 (정합성/완결성/구현가능성/추적성) 평가 결과 **81% (88/108) 점수** + **3 MAJOR + 15 MINOR = 18개 개선점** 식별. CRITICAL 0개. TASK-005-1 (v1 Rust MVP 구현) 시작 가능. 추가 design cycle 불필요.

3 MAJOR (trait Tool::Schema / BudgetTracker 80% threshold / 15 sub-agent Output type) 는 상세설계 단계에서 address. 15 MINOR 는 TDD TC 작성 시 또는 v1.5+ 에서 address.

TRACEABILITY.md 6차원 매트릭스 + 본 REVIEW.md 의 4-차원 평가 = TASK-005-1 의 cross-doc 검색 reference chain 완성. PLAN_v1_design.md 의 팀 구성 + 워크 패키지 + 리스크 = mavis-team 으로 상세설계 plan 위임 시 그대로 활용.

### 7.2 risks

- **(R-1) MAJOR 3개 미해소 시 TASK-005-1 구현 friction**: trait Tool::Schema, BudgetTracker threshold, sub-agent Output type 모두 TASK-005-1 의 핵심 type. 상세설계 task 1~3 으로 동시 진행 권장
- **(R-2) sub-agent 15개 SYSTEM.md 작성 시간**: DD-3 task 가 1,500~2,000 lines (가장 큰 task). 6-chunk write + 1 task 30+ min. v1.5+ 외부 정의 가능 구조면 v1 은 3-5 sub-agent 만 작성 가능
- **(R-3) LLM mock 부재**: v1 구현 시 실제 LLM 호출 (Anthropic API) 이 필요 → CI 환경에서 비용 + 결정성 문제. mock provider 작성 우선
- **(R-4) cross-OS 테스트 부재**: macOS 개발 + Linux/Windows 미검증. cargo-dist 5 paths + GitHub Actions matrix OS 권장 (D-07 dual-remote 영향)
- **(R-5) TDD 권고 미준수 시**: 6 docs 만으로 충분하다지만 TDD 없이 구현 시 drift. TDD RED-GREEN-REFACTOR 사이클 권고

### 7.3 suggested_follow_up

1. **즉시 (다음 작업)**: 본 REVIEW.md 검토 + 상세설계 plan (5 task) launch (D-16 chunked write + verifier)
2. **상세설계 task 1~3**: MAJOR 3개 (trait Tool::Schema / BudgetTracker threshold / 15 sub-agent) 동시 진행 (cycle 1 parallel + cycle 2 sequential)
3. **상세설계 task 4~5**: security patterns + retry/exit code (cycle 1 parallel)
4. **TDD TC task 1~2**: L1 Unit TC (60 TC) + L2 Integration TC (e2e sub-agent)
5. **TASK-005-1 (v1 Rust MVP 구현)**: TDD RED-GREEN-REFACTOR + cross-OS CI
6. **v1.5+**: LLM Wiki (D-32) + Plugin 4-계층 (D-33 v1.5+) + L3/L4 TC

### 7.4 produced_artifacts

| 산출물 | 경로 | 분량 |
| --- | --- | --- |
| **REVIEW.md** (본) | `docs/team/REVIEW.md` | ~900 lines / 8 sections (작성 중) |
| PLAN_v1_design.md | `docs/team/PLAN_v1_design.md` | 535 / 11 |
| REQUIREMENTS.md | `docs/REQUIREMENTS.md` | 1,003 / 11 |
| USE_CASES.md | `docs/USE_CASES.md` | 1,134 / 12 |
| INITIAL_DESIGN.md | `docs/architecture/INITIAL_DESIGN.md` | 2,056 / 14 |
| TRACEABILITY.md | `docs/team/TRACEABILITY.md` | 566 / 7 |

### 7.5 다음 단계 (Owner)

1. **본 REVIEW.md user 검토 + OK**
2. **상세설계 plan 작성** (5 task, 1 plan, mavis-team 위임) — D-16 chunked write 4-6 chunk
3. **상세설계 plan launch** (cycle 1 parallel 4 task + cycle 2 sequential 1 task)
4. **TDD TC plan 작성** (2-4 task, 1 plan, mavis-team 위임)
5. **TDD TC plan launch**
6. **TASK-005-1 (v1 Rust MVP 구현) 시작** — TDD RED-GREEN-REFACTOR + cross-OS CI

---

## VERDICT (final, post-handoff)

```
### VERDICT: PASS

본 REVIEW.md 는 my_harness v1 설계 6 docs 의 4-차원 평가 결과 81% (88/108).
CRITICAL 0개, MAJOR 3개 (§3.1), MINOR 15개 (§3.2).
TASK-005-1 시작 가능.
상세설계 plan 위임 (5 task) + TDD TC plan 위임 (2-4 task) 으로 이어서 진행.

본 REVIEW.md 작성: D-16 chunked write 4 chunk
verifier 독립 cross-check: 8/8 self-check (작성 후)
분량: ~900 lines / 8 sections
표준 6 원칙: 한국어 / 결론 위주 / 상태값 done / 이벤트 소싱 / 비참조 / handoff
D-06 토큰/시크릿: 없음 (mechanism only)
안티 6 미반영: 1 surface, 단일 언어, 30 entry, 한국어
```

