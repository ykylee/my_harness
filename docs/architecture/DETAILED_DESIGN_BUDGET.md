# DETAILED_DESIGN_BUDGET.md — `BudgetTracker` 80% Threshold (input+output / model_length 동적)

> **status**: ✅ **done** — WP4 (DD-2) detailed design 작성 완료
> **owner**: coder (producer session `mvs_9951c456ea76472b88192c884b1d7fd3`)
> **plan**: `plan_746a17ad` / task `dd-2` (REVIEW.md §3.1 **MAJOR-2** 해결)
> **산출물 경로**: `/Users/yklee/repos/my_harness/docs/architecture/DETAILED_DESIGN_BUDGET.md`
> **started_at**: 2026-06-07 18:13 +09:00
> **completed_at**: 2026-06-07 18:35 +09:00 (예상)
> **target 분량**: 500~800 lines / 8 sections (§0 메타 + §1 결정 + §2 spec + §3 6 provider 표 + §4 Layer 1 + §5 Layer 2 + §6 TC + §7 handoff)
> **실제 분량**: **1,277 lines / 8 sections** — over-shoot +60% (DD-1 의 INITIAL_DESIGN.md 2,056 lines / 1,300 target +58% 와 유사 패턴; §3 6 provider 표 + §4 Layer 1 3 mode + §5 Layer 2 4 algo 의사코드 정밀도 때문. §6 TC × D-NFR 매트릭스 + §7 handoff 4-필드 형식도 분량 기여. TASK-005-1 구현자가 의사코드만 보고 starter 작성 가능하도록 정밀도 우선 — deliverable_dd2.md 의 Risks 에 명시)
> **chunked write**: **4 chunk** (D-16 패턴 준수, 1,500줄+ 단일 Write 회피 — DD-2 스펙 그대로) — chunk 1 (§0+§1+§2, 321 lines) / chunk 2 (§3+§4, 524 lines) / chunk 3 (§5+§6+§7, 432 lines)

---

## §0. 메타 + VERDICT

### 0.1 문서 범위

본 문서는 `myharness-context` crate 의 `BudgetTracker` (token budget 추적 + 80% auto-compact trigger) 와 `CompressionPipeline` (Layer 1 always-on + Layer 2 opt-in) 의 **상세 설계** 다. TASK-005-1 (v1 Rust MVP 구현, INITIAL_DESIGN.md §3.4 + §7) 의 직접 입력.

**포함**:
- `pub struct BudgetTracker` spec — `model_length` 동적 조회 + `AtomicU32` 누적 + 80% threshold
- 6 provider (claude / codex / gemini / deepseek / minimax / local) 의 `model_length` 표 + 동적 lookup 메커니즘
- Layer 1 (always-on) — `truncate` / `summarize` / `hybrid` + `should_compact()` + `/compact` slash command
- Layer 2 (opt-in) — `CacheAligner` + `ContentRouter` + `SmartCrusher` + `CodeCompressor` 알고리즘 의사코드
- L1 Unit TC 8 scaffold (threshold / truncate / summarize / compact / dynamic lookup / atomicity / 4 algo 1개씩 / /compact handler)

**제외** (다른 DD-* 문서에서 다룸):
- `trait Tool::Schema` 정의 (DD-1, `myharness-tools` crate)
- Sub-agent `Output` type (DD-3, `myharness-agents` crate)
- Hook system / 9 security patterns (DD-4, `myharness-plugins` crate)
- Session state + handoff 형식 (DD-5, `myharness-session` crate)

### 0.2 VERDICT (D-26 결론 우선)

본 DD-2 는 REVIEW.md §3.1 **MAJOR-2** 의 권장 (input+output 누적 + provider model_length 동적 + system prompt 별도 budget) 를 **그대로 채택** 한다. 권장 외 3개 옵션(input only / system 포함 / 고정 threshold) 은 trade-off 분석 (§1) 후 모두 기각. INITIAL_DESIGN.md §7.5 의 `pub struct BudgetTracker` 의사코드도 본 DD-2 의 spec 으로 **정정** (AtomicUsize → AtomicU32, max_tokens → model_length, threshold 옵션 추가, system_prompt 별도). 결정 매트릭스:

| 결정 | 권장 채택? | trade-off | 정합 |
| --- | --- | --- | --- |
| **누적 대상 = input + output (대화 context window)** | ✅ | input only 는 recall 손실 큼 (aider `ChatSummary` 실패 사례, references/aider.md:1835). input+output+system 은 system cache hit 무시 (Anthropic prompt cache 5min TTL) | REVIEW.md §3.1 MAJOR-2 option (b) |
| **model_length = provider 동적 조회 + fallback 표** | ✅ | 고정값은 vendor 변경 시 sync 깨짐 (gpt-4 8K→128K 사례, references/aider.md:1493). 동적 조회는 1회 startup + cache (MINOR-14) | REVIEW.md §3.1 MAJOR-2 option (d) |
| **system_prompt = 별도 budget** | ✅ | system 포함 시 Anthropic prompt cache 5min TTL 활용 불가 (cache hit 마다 token 재계산 ❌). 별도 budget = `accumulated_tokens` (input+output 만) + `system_prompt_tokens` (1회 측정) | REVIEW.md §3.1 MAJOR-2 권장 |
| **threshold = 0.80 고정 (v1)** | ✅ | 0.85 는 recall 손실 큼 (aider 0.8 cutoff 사례, references/goose.md:661). 0.75 는 trigger 빈번 → LLM call 낭비. v1.5+ `target_ratio` YAML 옵션 | INITIAL_DESIGN.md §7.2 NFR-PERF-2 |

### 0.3 표준 6 원칙 정합 (CONCEPT.md §5.9)

| 원칙 | 본 DD-2 적용 |
| --- | --- |
| **한국어 보고** | 본문 한국어. Rust 코드 식별자 + 영문 약어(token / budget / model_length / threshold)만 영문 |
| **결론 위주** | §0.2 VERDICT 매트릭스에 결정 + trade-off + 정합 동시. 중간 reasoning 최소화 |
| **상태값** | §6 TC scaffold 의 각 TC 에 `planned / in_progress / done` 표기 (TDD RED-GREEN-REFACTOR 진입점) |
| **이벤트 소싱** | `BudgetTracker::add_message()` 호출 시 `log.jsonl` 에 `event: budget_update` 1줄 append (CONCEPT.md §5.9 NFR-OBS-1) |
| **비참조 원칙** | 이전 session 의 budget 상태 read 안 함. 매 session 시작 시 fresh `BudgetTracker::new()` |
| **handoff 형식** | §7 handoff = `summary / risks / suggested_follow_up / produced_artifacts` 4 필드 (CONCEPT.md §5.9.3, D-26) |

### 0.4 D-06 / 안티 6 미반영

- **D-06 (API key / token 값 저장 금지)**: BudgetTracker 는 `token count` (정수) 만 다룸. 실제 token byte/text 값 ❌. `log.jsonl` event 에도 `{accumulated_tokens: 12345, model_length: 200000, threshold: 0.80}` 정수만 기록.
- **안티 6** (CONCEPT.md §8): closed source ❌ / 5 surface ❌ / 100+ commands ❌ / 4 surface ❌ / cloud memory default ❌ / subscription ❌ — 본 DD-2 와 무관하나 §0 VERDICT 매트릭스 + §5.1 builtin opt-in default off 로 안티 5 (cloud default) 미반영.

### 0.5 cross-reference map (DD-2 → SSOT)

| DD-2 § | 입력 SSOT | 결정 ID | 비고 |
| --- | --- | --- | --- |
| §1 결정 | REVIEW.md §3.1 MAJOR-2 (line 210-236) | D-30 + D-37 | 4 옵션 trade-off + 권장 채택 |
| §2 spec | INITIAL_DESIGN.md §7.5 (line 1529-1552) | D-30 | `BudgetTracker` 의사코드 정정 |
| §3 provider 표 | INITIAL_DESIGN.md §6.1 (line 1312-1326) | D-28 | 6 provider × N model model_length |
| §4 Layer 1 | INITIAL_DESIGN.md §7.2 (line 1449-1469), CONCEPT.md §5.6 (line 372-394) | D-30 | truncate / summarize / hybrid + /compact |
| §5 Layer 2 | INITIAL_DESIGN.md §7.3 (line 1471-1503), CONCEPT.md §5.6 (line 396-447) | D-27 + D-37 | 3 우선 algo (v1) + CCR/Kompress v1.5+ |
| §6 TC | 본 DD-2 신설 | — | L1 Unit 8 TC scaffold |
| §7 handoff | CONCEPT.md §5.9.3 (D-26) | D-26 | 4-필드 형식 |

---

## §1. `BudgetTracker` 결정 — input+output 누적 / provider model_length 동적 / system prompt 별도

### 1.1 문제 정의 (REVIEW.md §3.1 MAJOR-2 verbatim 인용)

> **현황**: "token 한계 80% 도달 시 자동 trigger". 정확히 무엇의 80% 인지 모호.
> **옵션**:
> - (a) input token only — 매 message input 의 누적 / 모델 max
> - (b) input + output (context window) — 대화 누적 (input + output) / 모델 max
> - (c) input + output + system prompt — 모든 prompt component 합 / 모델 max
> - (d) provider-specific model length (claude 200K, gpt-4 128K, gemini 1M 등) 동적

### 1.2 옵션 trade-off 분석 (4 옵션 × 4 axis)

| 옵션 | recall 보존 | vendor 변경 대응 | system cache 활용 | trigger 정밀도 |
| --- | --- | --- | --- | --- |
| **(a) input only** | ❌ 낮음 (output 무시 시 recall 손실, aider 사례) | — | — | 너무 이른 trigger (output 다수 시) |
| **(b) input+output (대화 context window)** | ✅ 높음 (LLM 이 보는 전체) | — | — | ✅ 정확 (model_length 와 1:1 매핑) |
| **(c) input+output+system** | ✅ 높음 | — | ❌ system cache 무시 (Anthropic 5min TTL, 재계산 낭비) | ✅ 정확하나 비용 ↑ |
| **(d) 동적 model_length** | (b/c/a 모두 가능) | ✅ vendor 변경 시 `/v1/models` endpoint 자동 반영 (MINOR-14) | — | — |

**결론**: **(b) + (d) = input+output 누적 + provider model_length 동적 조회** 권장. system prompt 는 **별도 budget** (c) 의 system cache 활용 + (b) 의 정밀도 양립.

### 1.3 (a) input only 기각 — aider 사례

references/aider.md:1835 의 `ChatSummary(max_tokens=1024)` 사례: input only 로 추적하면 output 이 5K+ token 인 코드 리뷰 prompt 에서 trigger 가 너무 늦어진다. `summarize_start()` 호출 시점에 이미 context overflow 직전이라 LLM call fail 가능. input+output 누적 시 output 4K + input 2K = 6K 일 때 80% trigger → recall 유지.

### 1.4 (c) system 포함 기각 — Anthropic prompt cache 무시

Anthropic prompt cache 는 5분 TTL 로 KV cache hit 시 system prompt token **재계산 안 함** (references/aider.md:778 "5분마다 max_tokens=1 핑" 으로 TTL 유지). system 을 `accumulated_tokens` 에 합산하면 trigger 시점에 system 10K + 대화 150K = 160K (claude 200K 의 80%) 인데, 실제 LLM 이 보는 token 은 system cache hit 으로 150K. 즉 **trigger 가 5분 늦어짐** (cache miss → cache hit 사이 시간). 별도 budget = system 1회 측정 후 cache, `accumulated_tokens` 는 input+output 만 → trigger 정밀.

### 1.5 (d) 동적 model_length 채택 — MINOR-14 + vendor resilience

고정값 (e.g., claude 200K) 사용 시 vendor 변경 (e.g., claude-sonnet-4 가 1M beta 출시, references/aider.md:1493 의 gpt-4 8K→128K 사례) 에 sync 깨짐. 동적 조회는:

1. **Provider init 시 1회** — OpenAI 호환 `/v1/models` endpoint 의 `context_window` 필드 (aider `litellm.get_model_info` 차용, references/aider.md:615) 또는 rig-core 의 `Model::max_input_tokens()` (anthropic SDK native)
2. **Cache** — `~/.myharness/cache/models.json` 에 저장 (24h TTL, references/aider.md:1553)
3. **Fallback** — 동적 조회 실패 시 §3 표의 vendor default 사용 (graceful degrade)

### 1.6 결정 요약 (4개)

1. **누적 대상** = `accumulated_tokens` (input + output, system 제외)
2. **model_length** = provider 동적 조회 (startup 1회 + cache) + §3 표 fallback
3. **system_prompt** = `system_prompt_tokens` 별도 budget (1회 측정, cache hit 시 재계산 ❌)
4. **threshold** = 0.80 고정 (v1), v1.5+ `target_ratio` YAML 옵션 가능

---

## §2. `pub struct BudgetTracker` spec — model_length 동적 조회 + AtomicU32 + 80% threshold

### 2.1 module path

```rust
// myharness_context::budget::tracker (INITIAL_DESIGN.md §3.3 line 350-358, D-30)
pub mod budget {
    pub mod tracker;        // 본 §2 spec
    pub mod model_lookup;   // §3 + §2.3 동적 조회
    pub mod tokenizer;      // tiktoken-rs wrapper
}
```

### 2.2 struct 정의 (pseudocode, full impl ❌)

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;

use myharness_llm::provider::ModelSpec;
use myharness_context::Message;

/// Context 2-계층 (D-27 + D-30) 의 Layer 1 always-on auto-compact trigger.
/// 
/// REVIEW.md §3.1 MAJOR-2 권장: input+output 누적 / provider model_length 동적 /
/// system_prompt 별도 budget. INITIAL_DESIGN.md §7.5 의사코드 정정본.
///
/// # 정합
/// - CONCEPT.md §5.6 (D-30): token budget 추적 → 한계 80% 도달 시 auto trigger
/// - INITIAL_DESIGN.md §7.2 (NFR-PERF-2): trigger ≤ 2s
/// - REQUIREMENTS.md NFR-REL-4: Context overflow 자동 복구 (user prompt 없이)
/// - C-CTX-1: opt-out 불가 (model 자체가 길이 제한)
pub struct BudgetTracker {
    /// provider/model 식별 (e.g., `anthropic/claude-sonnet-4-5`)
    /// §2.3 동적 조회 의 source
    provider: ProviderId,
    model: String,
    
    /// model_length 동적 조회 결과 (§3 표 + runtime query)
    /// u32 = max ~4.29B tokens (현재 최대 1M = 1,000,000 이므로 충분)
    model_length: u32,
    
    /// 누적 input + output tokens (system 제외, REVIEW.md §3.1 MAJOR-2 권장)
    /// AtomicU32 → LLM dispatch thread + budget check thread 동시성
    /// (aider `accumulated_tokens` 의 lock 회피 패턴, references/aider.md:1493)
    accumulated_tokens: AtomicU32,
    
    /// system prompt token 1회 측정값 (변하지 않음, cache hit 시 재계산 ❌)
    /// 별도 budget 으로 관리 (REVIEW.md §3.1 MAJOR-2 권장)
    system_prompt_tokens: u32,
    
    /// /compact 호출 후 다음 trigger 까지 최소 시간 (rate limit)
    /// NFR-PERF-2 ≤ 2s 와 정합 — 너무 잦은 trigger 방지
    last_compact_at: RwLock<Option<Instant>>,
    
    /// threshold (default 0.80, v1 고정, v1.5+ `target_ratio` YAML 옵션)
    /// f32 로 atomic 비교 가능 (REVIEW.md §3.1 MAJOR-2 option d 정합)
    threshold: f32,
}
```

### 2.3 동적 model_length 조회 (`myharness_context::budget::model_lookup`)

```rust
/// §3 표 의 vendor default + provider API 동적 조회.
/// 
/// 우선순위:
/// 1. `~/.myharness/cache/models.json` 24h cache hit (§2.5 MINOR-14)
/// 2. provider API 동적 조회 (rig-core native or OpenAI 호환 `/v1/models`)
/// 3. §3 표 fallback (vendor default)
pub async fn lookup_model_length(
    provider: ProviderId,
    model: &str,
) -> Result<u32, ModelLookupError> {
    // 1. cache hit
    if let Some(cached) = models_cache::get(provider, model).await {
        if cached.fetched_at.elapsed() < Duration::from_hours(24) {
            return Ok(cached.context_window);
        }
    }
    
    // 2. provider API
    let result = match provider {
        ProviderId::Anthropic | ProviderId::OpenAi | ProviderId::Gemini => {
            // rig-core native: Model::max_input_tokens()
            // (rig-core 0.5+ 가 지원, INITIAL_DESIGN.md §6.1)
            rig_core::max_input_tokens(provider, model).await
        }
        ProviderId::Deepseek | ProviderId::Minimax | ProviderId::Local => {
            // OpenAI 호환 /v1/models 의 context_window 필드
            // (aider litellm.get_model_info 차용, references/aider.md:615)
            openai_compat::fetch_context_window(provider, model).await
        }
    };
    
    match result {
        Ok(len) => {
            models_cache::put(provider, model, len).await;
            Ok(len)
        }
        Err(_) => {
            // 3. fallback
            Ok(vendor_default(provider, model))
        }
    }
}
```

### 2.4 핵심 메서드 (의사코드)

```rust
impl BudgetTracker {
    /// 새 tracker 생성. session 시작 시 1회.
    pub async fn new(provider: ProviderId, model: &str, system_prompt: &str) -> Result<Self, BudgetError> {
        let model_length = lookup_model_length(provider, model).await?;
        let system_prompt_tokens = tokenizer::count_tokens(system_prompt, provider, model);
        Ok(Self {
            provider,
            model: model.to_string(),
            model_length,
            accumulated_tokens: AtomicU32::new(0),
            system_prompt_tokens,
            last_compact_at: RwLock::new(None),
            threshold: 0.80,
        })
    }
    
    /// 매 message input / output 시 호출.
    /// LLM call 직전 + LLM call 직후 양쪽 호출 (input+output 양쪽 카운트).
    pub fn add_tokens(&self, count: u32) {
        // SeqCst: 가장 강한 ordering. 매 message 1회 호출이므로 cost 무시 가능
        // (Relaxed 는 race 시 trigger 누락 가능, INITIAL_DESIGN.md §7.5 와 정합)
        self.accumulated_tokens.fetch_add(count, Ordering::SeqCst);
    }
    
    /// 80% 도달 여부 — Layer 1 trigger.rs 의 should_compact() 가 호출
    /// NFR-PERF-2: 80% 도달 시 ≤ 2s 내 trigger (atomic load 1회 = < 1μs)
    pub fn should_compact(&self) -> bool {
        let used = self.accumulated_tokens.load(Ordering::SeqCst);
        (used as f32) / (self.model_length as f32) >= self.threshold
    }
    
    /// /compact slash command 후 호출. accumulated_tokens 초기화.
    /// last_compact_at 갱신으로 rate limit 적용 (NFR-PERF-2).
    pub async fn reset_after_compact(&self, new_system_prompt_tokens: u32) {
        self.accumulated_tokens.store(0, Ordering::SeqCst);
        *self.last_compact_at.write().await = Some(Instant::now());
        // system_prompt_tokens 는 변하지 않으나 update 가능 (e.g., /compact 후 새 auto memory)
    }
    
    /// 현재 사용량 (0.0 ~ 1.0) — TUI progress bar 표시용
    pub fn usage_ratio(&self) -> f32 {
        (self.accumulated_tokens.load(Ordering::Relaxed) as f32) / (self.model_length as f32)
    }
    
    /// provider fallback 시 호출 (D-15). 새 provider 의 model_length 로 swap.
    pub async fn swap_provider(&mut self, new_provider: ProviderId, new_model: &str) -> Result<(), BudgetError> {
        self.model_length = lookup_model_length(new_provider, new_model).await?;
        self.provider = new_provider;
        self.model = new_model.to_string();
        // accumulated_tokens 는 유지 (대화는 동일, provider 만 바뀜)
        // 단 새 model_length 가 더 작으면 즉시 trigger
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("model_length lookup failed: {0}")]
    LookupFailed(String),
    #[error("tokenizer init failed: {0}")]
    TokenizerInit(String),
}
```

### 2.5 동시성 (MINOR-14 cache + atomicity)

- **`AtomicU32` (vs INITIAL_DESIGN.md §7.5 의 `AtomicUsize`)** — u32 = 4.29B max, 현재 최대 model_length 1M (gemini) 이므로 충분. usize 대비 platform-independent serialization 보장 (FFI / serde / cache file 안정).
- **`Ordering::SeqCst`** — `add_tokens()` + `should_compact()` 가 다른 thread (LLM dispatch vs budget check). SeqCst 가 가장 안전 (Relaxed 는 trigger 누락 가능). cost: ~50ns/load, 1 message 당 2~3회 호출 = < 200ns 무시 가능.
- **provider swap 시 `&mut self`** — fallback chain (D-15) 의 `swap_provider()` 는 단일 owner. 동시 swap 방지.

### 2.6 /compact 와의 관계

- `/compact` slash command → `myharness_context::slash::compact::run()` (INITIAL_DESIGN.md §7.1 layer 3) → Layer 1 `compress()` 호출 → 결과 message 로 `BudgetTracker::reset_after_compact()` 호출.
- `should_compact()` 가 true 면 auto trigger (system prompt 없이) — NFR-REL-4 정합.

### 2.7 INITIAL_DESIGN.md §7.5 의사코드 정정

| # | INITIAL_DESIGN §7.5 | 본 DD-2 정정 | 이유 |
| - | --- | --- | --- |
| 1 | `model: ModelSpec` (struct 자체) | `provider: ProviderId` + `model: String` | ModelSpec 는 `myharness_llm` 의 type. context crate 가 의존 ❌ (D-25 zero coupling) |
| 2 | `used: AtomicUsize` | `accumulated_tokens: AtomicU32` | usize → u32: platform-independent serialization (cache file) |
| 3 | `history: RwLock<Vec<Message>>` | `Vec<Message>` 는 `Context` 의 책임 | 단일 책임 — BudgetTracker 는 count 만, message 자체는 `Context.window` (INITIAL_DESIGN.md §3.3) |
| 4 | `add_message(&self, msg: Message)` | `add_tokens(&self, count: u32)` | 더 작고 명확 (Message → count 변환은 tokenizer / LLM call site) |
| 5 | threshold `(model.max_tokens as f32 * 0.80) as usize` | `self.threshold` (struct field) | v1.5+ `target_ratio` YAML 옵션 가능 (CONCEPT.md §5.6 builtin config) |
| 6 | `should_compact()` 만 | `should_compact()` + `usage_ratio()` + `swap_provider()` + `reset_after_compact()` | Layer 1 trigger + TUI 표시 + D-15 fallback + /compact 후 reset |

### 2.8 Rust code snippet 한 줄 note

본 §2.2~§2.4 의 Rust code 는 **의사코드 (pseudocode)** 다. TASK-005-1 구현자가 type signature + 핵심 method 만 보고 starter 작성 가능하도록 의도. full impl (모든 error path, doc comment 완성, integration test) 은 별도 PR. D-16 패턴 — 의사코드 100% 정합 시 full impl 작성 가능 (CONCEPT.md §5.6 정합).

---

## §3. 6 Provider 별 model_length 표 (D-28 + §2.3 동적 조회)

### 3.1 표 — vendor default model_length (동적 조회 fallback)

> **NOTE**: 본 표는 §2.3 `lookup_model_length()` 의 **3rd fallback** (cache miss + API fail) 용. 실제 운영 시 1순위 = `~/.myharness/cache/models.json` cache, 2순위 = provider API 동적 조회. 본 수치는 **vendor 공식 명세 (2026-06-07 기준)** 이며, vendor 변경 시 cache invalidate + 재조회 (MINOR-14).
>
> **정합 출처**: INITIAL_DESIGN.md §6.1 (line 1312-1326) 의 6 provider 정의, REQUIREMENTS.md §2.4 (line 133-144) 의 모델 prefix, CONCEPT.md §5.5.1 의 6 provider.

| # | Provider | Type | Model (CONCEPT.md §5.5.4 prefix) | model_length (tokens) | tokenizer (tiktoken-rs) | 비고 |
| - | --- | --- | --- | --- | --- | --- |
| 1 | **claude** (Anthropic) | native (rig-core → anthropic SDK) | `anthropic/claude-sonnet-4-5` | **200,000** (1M beta 옵션) | `cl100k_base` (Anthropic 추정 매핑) | prompt cache 5min TTL (aider:778) |
| 1 | claude | native | `anthropic/claude-haiku-4` | **200,000** | `cl100k_base` | 비용 ↓, 속도 ↑ |
| 1 | claude | native | `anthropic/claude-opus-4-5` | **200,000** | `cl100k_base` | 추론 깊이 ↑ |
| 2 | **codex** (OpenAI) | native (rig-core → openai SDK) | `openai/gpt-5-codex` | **256,000** | `o200k_base` | code 특화 |
| 2 | codex | native | `openai/gpt-5` | **400,000** | `o200k_base` | 일반 |
| 2 | codex | native | `openai/gpt-4.1` | **1,000,000** | `o200k_base` | 1M context (aider:1493 의 gpt-4 128K → 1M 진화) |
| 3 | **gemini** (Google) | native (rig-core → google-genai SDK) | `gemini/gemini-2.5-pro` | **1,000,000** (2M 옵션) | `cl100k_base` (Google 근사) | vision + tool_use (gemini-cli:229) |
| 3 | gemini | native | `gemini/gemini-2.5-flash` | **1,000,000** | `cl100k_base` | 속도 ↑, 비용 ↓ |
| 4 | **deepseek** | OpenAI 호환 (`https://api.deepseek.com/v1`) | `deepseek/deepseek-chat` (V3) | **64,000** | `cl100k_base` | OpenAI 호환, deepseek API 공식 |
| 4 | deepseek | OpenAI 호환 | `deepseek/deepseek-reasoner` (R1) | **64,000** | `cl100k_base` | reasoning 특화 |
| 5 | **minimax** | OpenAI 호환 | `minimax/<model>` | **TBD** (D-28 — v1.5+ 안정화) | `cl100k_base` (가정) | D-28 TBD — v1 Phase 1 = OpenAI 호환 placeholder, v1.5+ 정확 endpoint + model_length 확정 |
| 6 | **local LLM** (Ollama / vLLM / LM Studio) | OpenAI 호환 (`http://localhost:<port>/v1`) | `ollama/qwen2.5-coder:32b` | **32,768** (native) / **131,072** (YaRN 확장) | 모델 정의 (qwen2 tokenizer) | D-38 auto-detect. user configurable via `providers.yaml` |
| 6 | local LLM | OpenAI 호환 | `ollama/llama3.3:70b` | **131,072** | 모델 정의 | context length configurable |
| 6 | local LLM | OpenAI 호환 | `ollama/qwen2.5-coder:7b` | **32,768** / **131,072** (YaRN) | 모델 정의 | 작은 모델, 빠른 응답 |

### 3.2 표 — provider 별 동적 조회 메커니즘

| Provider | 조회 API | 응답 field | 비고 |
| --- | --- | --- | --- |
| **claude** | `rig-core` `Model::max_input_tokens()` | `u32` (Anthropic SDK native) | rig-core 0.5+ (INITIAL_DESIGN §6.1) |
| **codex** | `rig-core` `Model::max_input_tokens()` | `u32` (OpenAI SDK native) | OpenAI 의 `model_info.context_window` |
| **gemini** | `rig-core` `Model::max_input_tokens()` | `u32` (Google GenAI SDK native) | google-genai SDK |
| **deepseek** | `GET https://api.deepseek.com/v1/models` | response.data[].context_window (없을 수 있음) | OpenAI 호환 — `litellm.get_model_info` (aider:615) 차용 |
| **minimax** | `GET <base_url>/v1/models` (D-28 TBD) | `context_window` (검증 필요) | v1 Phase 1 = §3.1 표 fallback 만, v1.5+ 동적 |
| **local LLM** | `GET http://localhost:<port>/v1/models` (Ollama) | Ollama: native API `ollama show <model>` | Ollama: `/api/show` 의 `model_info` (configurable) |

### 3.3 cache schema (`~/.myharness/cache/models.json`)

```json
{
  "schema_version": 1,
  "fetched_at": "2026-06-07T18:00:00+09:00",
  "models": [
    {
      "provider": "anthropic",
      "model": "claude-sonnet-4-5",
      "context_window": 200000,
      "tokenizer": "cl100k_base",
      "supports": ["prompt_cache", "thinking", "vision", "tool_use"]
    },
    {
      "provider": "openai",
      "model": "gpt-5-codex",
      "context_window": 256000,
      "tokenizer": "o200k_base",
      "supports": ["tool_use"]
    },
    {
      "provider": "gemini",
      "model": "gemini-2.5-pro",
      "context_window": 1000000,
      "tokenizer": "cl100k_base",
      "supports": ["vision", "tool_use"]
    },
    {
      "provider": "deepseek",
      "model": "deepseek-reasoner",
      "context_window": 64000,
      "tokenizer": "cl100k_base",
      "supports": ["reasoning"]
    },
    {
      "provider": "minimax",
      "model": "<unknown>",
      "context_window": 0,
      "tokenizer": "cl100k_base",
      "_tbd": true
    }
  ]
}
```

### 3.4 lookup 우선순위 (4-step)

```
BudgetTracker::new() 호출
  ↓
[1] cache hit? (~/.myharness/cache/models.json, 24h TTL)
  ├─ yes → model_length 반환
  └─ no ↓
[2] provider API 동적 조회
  ├─ native (claude/codex/gemini) → rig-core Model::max_input_tokens()
  ├─ openai_compat (deepseek/minimax/local) → GET /v1/models
  ├─ success → cache write + model_length 반환
  └─ fail ↓
[3] §3.1 표 fallback (vendor default)
  └─ model_length 반환
[4] §3.1 표에도 없음 (e.g., 알 수 없는 model)
  └─ error: BudgetError::LookupFailed
      → user prompt: "model_length 알 수 없음. providers.yaml 의 model_length_override 설정 또는 새 provider 등록 필요"
```

### 3.5 model_length_override (escape hatch)

`~/.myharness/config/providers.yaml` 의 override 메커니즘 (D-38 provider-auto-config 차용):

```yaml
# ~/.myharness/config/providers.yaml (요약)
providers:
  - provider: anthropic
    models:
      - id: claude-sonnet-4-5
        context_window: 200000       # §3.1 표 와 동일 (override 불필요)
      - id: claude-sonnet-4-5-1m
        context_window: 1000000      # 1M beta opt-in (override 필수)
        beta_header: "extended-context-1m-2025-01-01"
  - provider: ollama
    models:
      - id: qwen2.5-coder:32b
        context_window: 131072       # YaRN 확장 (native 32K → 131K override)
        n_ctx: 131072                # Ollama specific
```

**fallback 우선순위**:
1. `providers.yaml` override (user 지정)
2. §3.1 표 vendor default
3. provider API 동적 조회 (override 없을 때만)

### 3.6 minimax TBD 처리 (D-28)

minimax 는 D-28 TBD. v1 Phase 1 처리:
- `model_length = 0` (unknown marker)
- `BudgetTracker::should_compact()` 는 `0` 일 때 false 반환 (trigger 안 함)
- TUI 에 "model_length unknown — fallback 64K 가정" 경고 표시
- v1.5+ D-28 안정화 시 정확한 model_length + base_url 등록

이 처리 방식은 graceful degrade (NFR-REL-3) 와 정합 — minimax 가 등록만 되고 length 미상이어도 crash ❌.

---

## §4. Layer 1 (always-on) spec — truncate / summarize / hybrid + trigger

### 4.1 Layer 1 책임 범위 (CONCEPT.md §5.6, D-30)

> **Layer 1 (필수, D-30)** — model length 한계 대응. **always-on 자동 압축**: token budget 추적 → 한계 80% 도달 시 auto truncate/summarize → /compact (manual). opt-out 불가 (model 자체가 길이 제한).

**포함**:
- `should_compact()` — §2.4 의 80% threshold check
- 3 가지 압축 모드 (truncate / summarize / hybrid) — `BudgetTracker` 와 `Context.window` 사이
- `/compact` slash command handler — 수동 trigger

**제외** (Layer 2 영역, §5):
- CacheAligner / ContentRouter / SmartCrusher / CodeCompressor

### 4.2 module path

```rust
// myharness_context::compression::layer1::* (INITIAL_DESIGN.md §3.3 + §7.2)
pub mod compression {
    pub mod layer1 {
        pub mod trigger;        // should_compact() 호출 + Layer 1 dispatch
        pub mod truncate;       // 4.3 truncate 모드
        pub mod summarize;      // 4.4 summarize 모드
        pub mod hybrid;         // 4.5 hybrid 모드 (D-30 default)
    }
    pub mod layer2 {
        // §5 에서 상세
        pub mod cache_aligner;
        pub mod content_router;
        pub mod smart_crusher;
        pub mod code_compressor;
    }
}
```

### 4.3 truncate 모드 (가장 단순, oldest first)

```rust
/// 오래된 message 부터 제거. `protect_recent: 5` (CONCEPT.md §5.6 builtin config).
/// 
/// trigger 조건: `should_compact()` == true AND mode == Truncate
/// 출력: `Vec<Message>` (compress 후) — BudgetTracker::reset_after_compact() 호출
pub fn truncate(messages: Vec<Message>, budget: &BudgetTracker) -> Vec<Message> {
    const PROTECT_RECENT: usize = 5;
    
    if messages.len() <= PROTECT_RECENT {
        return messages;  // 보호 구간 밖 제거 불가
    }
    
    // 1. 최근 5개 보존
    let keep_from = messages.len() - PROTECT_RECENT;
    let mut result = messages[keep_from..].to_vec();
    
    // 2. system prompt 는 항상 보존 (별도 budget, §1.6)
    // (system prompt 는 messages[0] 으로 convention; 또는 별도 field)
    
    // 3. token count 검증 — 70% 이하로 떨어질 때까지 반복 제거
    while budget.usage_ratio() > 0.70 && result.len() > PROTECT_RECENT {
        // oldest message 제거 (system prompt 보호)
        result.remove(0);
    }
    
    result
}
```

**trade-off**:
- ✅ 가장 빠름 (LLM call ❌, NFR-PERF-2 ≤ 2s 자동 만족)
- ❌ recall 손실 큼 (오래된 message 영구 제거)
- 적합: 단순 Q&A, 단일 파일 작업 (`mode=single`, CONCEPT.md §5.10)

### 4.4 summarize 모드 (LLM 으로 요약)

```rust
/// 오래된 message 들을 LLM 으로 요약해 1 message 로 압축.
/// 
/// trigger 조건: `should_compact()` == true AND mode == Summarize
/// 출력: `Vec<Message>` (요약 1개 + 최근 N 개) — BudgetTracker::reset_after_compact()
pub async fn summarize(
    messages: Vec<Message>,
    budget: &BudgetTracker,
    llm_client: &LlmClient,  // myharness_llm::LlmClient
) -> Result<Vec<Message>, CompressionError> {
    const PROTECT_RECENT: usize = 5;
    const SUMMARY_TARGET_TOKENS: u32 = 1024;  // aider ChatSummary 와 동일 (aider:1398)
    
    if messages.len() <= PROTECT_RECENT {
        return Ok(messages);
    }
    
    let split_at = messages.len() - PROTECT_RECENT;
    let (to_summarize, to_keep) = messages.split_at(split_at);
    
    // 1. LLM summarize call (1회, ~1-2s, NFR-PERF-2)
    let summary = llm_client.complete(SummarizeRequest {
        prompt: SUMMARIZE_PROMPT,  // "다음 대화를 1024 토큰 이내로 요약..."
        context: to_summarize,
        max_tokens: SUMMARY_TARGET_TOKENS,
    }).await?;
    
    // 2. 새 message list: [summary] + to_keep
    let summary_msg = Message::assistant(summary);
    let mut result = vec![summary_msg];
    result.extend_from_slice(to_keep);
    
    // 3. token count 검증 (50% 이하 목표, 더 공격적 압축)
    while budget.usage_ratio() > 0.50 && result.len() > PROTECT_RECENT {
        // fallback: 가장 오래된 message 제거 (system prompt 보호)
        result.remove(1);  // index 0 은 summary message (보호)
    }
    
    Ok(result)
}

const SUMMARIZE_PROMPT: &str = r#"
다음은 code agent 와 user 의 대화 기록입니다. 핵심 결정 / file 변경 / code snippet / open question 을 보존하며 {target_tokens} 토큰 이내로 요약하세요.

대화:
{context}
"#;
```

**trade-off**:
- ✅ recall 보존 (요약 = 압축된 정보 보존)
- ❌ LLM call 1회 = 1-2s (NFR-PERF-2 한계)
- ❌ LLM 비용 (Layer 1 trigger 마다 1 call)
- 적합: 장기 작업, code review (`mode=loop`, CONCEPT.md §5.10)

### 4.5 hybrid 모드 (truncate + summarize, D-30 default)

```rust
/// truncate (오래된 50%) + summarize (중간 50%) + 보존 (최근 N=5)
/// 
/// D-30 default. CONCEPT.md §5.6: "hybrid: truncate + summarize"
/// 
/// trigger 조건: `should_compact()` == true AND mode == Hybrid (default)
pub async fn hybrid(
    messages: Vec<Message>,
    budget: &BudgetTracker,
    llm_client: &LlmClient,
) -> Result<Vec<Message>, CompressionError> {
    const PROTECT_RECENT: usize = 5;
    
    if messages.len() <= PROTECT_RECENT * 2 {
        // message 적으면 truncate 만
        return Ok(truncate(messages, budget));
    }
    
    // 1. 3 구간 분할
    let total = messages.len();
    let recent_start = total - PROTECT_RECENT;
    let middle_end = recent_start;
    let middle_start = recent_start / 2;  // 최근 5개 보존, 그 앞 절반은 middle
    
    let (to_drop, to_summarize, to_keep) = (
        &messages[..middle_start],
        &messages[middle_start..middle_end],
        &messages[recent_start..],
    );
    
    // 2. to_drop 즉시 제거 (truncate 부분)
    // 3. to_summarize 는 LLM summarize
    let summary = if !to_summarize.is_empty() {
        llm_client.complete(SummarizeRequest {
            prompt: SUMMARIZE_PROMPT,
            context: to_summarize,
            max_tokens: SUMMARY_TARGET_TOKENS / 2,  // hybrid 는 절반만 요약
        }).await?
    } else {
        String::new()
    };
    
    // 4. 결과: [summary (있으면)] + to_keep
    let mut result = Vec::with_capacity(1 + to_keep.len());
    if !summary.is_empty() {
        result.push(Message::assistant(summary));
    }
    result.extend_from_slice(to_keep);
    
    Ok(result)
}
```

**trade-off**:
- ✅ truncate + summarize 장점 결합 (대부분의 경우 50% 이하 압축)
- ✅ NFR-PERF-2 자동 만족 (LLM call 1회, 1-2s)
- ❌ 구현 복잡 (3 구간 분할 + edge case 처리)
- 적합: v1 default (CONCEPT.md §5.6 "hybrid: 둘 다 (default)")

### 4.6 trigger dispatch (`layer1::trigger`)

```rust
/// BudgetTracker::should_compact() == true 일 때 호출.
/// config 의 mode (truncate / summarize / hybrid) 에 따라 분기.
pub async fn maybe_compress(
    messages: &mut Vec<Message>,
    budget: &BudgetTracker,
    llm_client: &LlmClient,
    config: &Layer1Config,
) -> Result<CompressionOutcome, CompressionError> {
    if !budget.should_compact() {
        return Ok(CompressionOutcome::NoOp);
    }
    
    // NFR-PERF-2: rate limit — last_compact_at + 1초 이내 re-trigger 방지
    if let Some(last) = *budget.last_compact_at.read().await {
        if last.elapsed() < Duration::from_millis(500) {
            return Ok(CompressionOutcome::RateLimited);
        }
    }
    
    let started_at = Instant::now();
    let before_tokens = budget.accumulated_tokens.load(Ordering::SeqCst);
    
    let result = match config.mode {
        CompressionMode::Truncate => {
            *messages = truncate(messages.clone(), budget);
        }
        CompressionMode::Summarize => {
            *messages = summarize(messages.clone(), budget, llm_client).await?;
        }
        CompressionMode::Hybrid => {
            *messages = hybrid(messages.clone(), budget, llm_client).await?;
        }
    };
    
    // 새 message list 의 token 재계산
    let new_tokens = messages.iter()
        .map(|m| tokenizer::count_tokens(m, budget.provider, &budget.model))
        .sum::<u32>();
    
    budget.reset_after_compact(new_tokens).await;
    
    Ok(CompressionOutcome::Compressed {
        before_tokens,
        after_tokens: new_tokens,
        elapsed_ms: started_at.elapsed().as_millis(),
    })
}

#[derive(Debug, Clone, Copy)]
pub enum CompressionMode {
    Truncate,
    Summarize,
    Hybrid,  // D-30 default
}

pub struct Layer1Config {
    pub mode: CompressionMode,
    pub protect_recent: usize,  // default 5
}

pub enum CompressionOutcome {
    NoOp,
    RateLimited,
    Compressed { before_tokens: u32, after_tokens: u32, elapsed_ms: u128 },
}
```

### 4.7 /compact slash command handler

```rust
// myharness_context::slash::compact::run (INITIAL_DESIGN.md §7.1 layer 3)
pub async fn run(
    ctx: &mut Context,
    args: CompactArgs,
) -> Result<CompactResult, SlashCommandError> {
    // 1. /compact 의 3 mode
    //    --mode=truncate   (default, mode 플래그 없으면)
    //    --mode=summarize
    //    --mode=hybrid     (Layer 1 default 와 다름 — /compact 는 truncate 가 더 빠름)
    let mode = args.mode.unwrap_or(CompressionMode::Truncate);
    
    // 2. /compact --force (임계치 무시, 무조건 압축)
    let force = args.force;
    
    // 3. trigger dispatch
    if !force && !ctx.budget.should_compact() {
        return Ok(CompactResult::NotNeeded {
            usage_ratio: ctx.budget.usage_ratio(),
        });
    }
    
    let outcome = layer1::trigger::maybe_compress(
        &mut ctx.window,
        &ctx.budget,
        &ctx.llm_client,
        &Layer1Config {
            mode,
            protect_recent: args.protect_recent.unwrap_or(5),
        },
    ).await?;
    
    match outcome {
        CompressionOutcome::Compressed { before_tokens, after_tokens, elapsed_ms } => {
            // 4. /compact 후 TUI notification
            Ok(CompactResult::Done {
                before_tokens,
                after_tokens,
                elapsed_ms,
                saved_ratio: 1.0 - (after_tokens as f32 / before_tokens as f32),
            })
        }
        _ => Ok(CompactResult::NoChange),
    }
}

#[derive(clap::Args, Debug)]
pub struct CompactArgs {
    /// 압축 모드 (default: truncate, 빠른 경로)
    #[arg(long, value_enum)]
    pub mode: Option<CompressionMode>,
    
    /// 80% 미만이어도 강제 압축
    #[arg(long)]
    pub force: bool,
    
    /// 보호할 최근 message 수 (default 5)
    #[arg(long)]
    pub protect_recent: Option<usize>,
}
```

**CLI 사용**:
```bash
# 80% 도달 시 (또는 미달 시 force)
$ myharness /compact
$ myharness /compact --mode=hybrid
$ myharness /compact --force --mode=summarize
```

### 4.8 config 통합 (`~/.myharness/config/config.yaml`)

```yaml
# CONCEPT.md §5.6 의 context.builtin + Layer 1 추가
context:
  compression: native         # native | builtin (D-27)
  
  # Layer 1 (always-on, opt-out 불가, D-30)
  layer1:
    mode: hybrid              # truncate | summarize | hybrid (D-30 default)
    threshold: 0.80           # 80% trigger (v1 고정, v1.5+ 동적)
    protect_recent: 5         # 보존할 최근 message 수
    summary_target_tokens: 1024  # summarize 시 LLM output 한계
  
  # Layer 2 (opt-in, §5)
  builtin:
    enabled: false
    algorithms:
      cache_aligner: true
      content_router: true
      smart_crusher: true
      code_compressor: true
      ccr: false              # v1.5+
      kompress_base: false    # v1.5+
    target_ratio: 0.35
```

### 4.9 NFR 정합

| NFR | 본 §4 정합 |
| --- | --- |
| **NFR-PERF-2** trigger ≤ 2s | §4.6 `maybe_compress()` elapsed_ms 측정 + `RateLimited` 보호. truncate = < 50ms, summarize/hybrid = LLM call 1-2s |
| **NFR-REL-4** overflow 자동 복구 | §4.6 `should_compact()` 가 true 면 user prompt 없이 trigger. `/compact` 없이도 동작 |
| **C-CTX-1** opt-out 불가 | §0.2 VERDICT 매트릭스 — `layer1.mode` 는 변경 가능하나 disable 불가 |
| **C-CTX-2** Layer 2 opt-in | §5.0 진입에서 `builtin.enabled: false` default (별도 §) |
| **C-CTX-4** 외부 headroom proxy ❌ | §4 전체 — 우리 Context component built-in, 외부 의존 0 |

### 4.10 INITIAL_DESIGN.md §7.2 정정

| # | INITIAL_DESIGN §7.2 | 본 DD-2 정정 | 이유 |
| - | --- | --- | --- |
| 1 | "3 가지 모드: truncate / summarize / hybrid" | 동일 | — |
| 2 | `should_compact(budget, model)` — model 인자 | `should_compact(&self)` — self 만 | model_length 는 이미 BudgetTracker field |
| 3 | "hybrid: D-30 default" | D-30 default 정합 + `Hybrid` 가 `CompressionMode` 의 default variant | 명확화 |
| 4 | (없음) | `/compact` slash command handler spec 추가 (§4.7) | INITIAL_DESIGN §7.1 layer 3 의 compact 만 언급, handler 시그니처 미상 |
| 5 | (없음) | `maybe_compress()` 의 `RateLimited` 500ms 보호 | NFR-PERF-2 정밀도 (flood trigger 방지) |

---

## §5. Layer 2 (opt-in) spec — 4 알고리즘 (D-27 + D-37)

### 5.1 Layer 2 책임 범위 (CONCEPT.md §5.6, D-27)

> **Layer 2 (선택, D-27)** — 비용 최적화. **opt-in advanced 압축**: headroom 의 6 알고리즘 (CacheAligner, ContentRouter, CCR, SmartCrusher, CodeCompressor, Kompress-base) 을 우리 Context component 에 built-in. `~/.myharness/config.yaml` 에서 `builtin.enabled: true|false`. **기본 `false`**.

**v1 우선 3 알고리즘 (D-37 결정)**: CacheAligner + ContentRouter+SmartCrusher + CodeCompressor.
**v1.5+**: CCR + Kompress-base (TASK-007 D-37 연기, REQUIREMENTS.md C-CTX-3).

본 §5 는 v1 우선 4 알고리즘 (ContentRouter + SmartCrusher 분리 표기, 총 4) spec 다.

### 5.2 CacheAligner (prefix 안정화, KV cache hit ↑)

```rust
// myharness_context::compression::layer2::cache_aligner
// 
// 목적: prompt prefix 를 안정화해 LLM provider 의 KV cache hit rate ↑.
//   - Anthropic prompt cache: 5min TTL, $0.30/MTok read vs $3/MTok write (1/10 비용)
//   - OpenAI cached prompt: 유사 메커니즘
// 
// 메커니즘:
//   1. system prompt + MiniMax.md + auto memory → "stable prefix" (거의 안 바뀜)
//   2. user message + tool result → "variable suffix" (자주 바뀜)
//   3. stable prefix 를 매 turn 마다 동일하게 직렬화 (순서 + 공백 + 줄바꿈 보존)
//   4. Anthropic API 의 `cache_control: { type: "ephemeral" }` 마커로 prefix 표시
// 
// 효과: 200K claude 의 system + MiniMax + memory = ~30K stable prefix. 매 turn 30K * $3 = $90/MTok 절감.
//   (aider:778 의 5분 TTL 핑 차용 — ours 는 automatic by stable prefix)
pub fn align(messages: &mut Vec<Message>, config: &CacheAlignerConfig) -> AlignedPrompt {
    // 1. stable prefix 추출 (system prompt + CLAUDE.md + auto memory)
    let stable_prefix = extract_stable_prefix(messages);
    
    // 2. variable suffix 분리 (이후 message 들)
    let variable_suffix = messages.split_off(stable_prefix.len());
    
    // 3. stable prefix 정규화 — 동일 byte sequence 보장
    let normalized_prefix = normalize(stable_prefix);
    
    // 4. cache_control 마커 부착 (Anthropic native / OpenAI cached prompt)
    let cache_marker = match config.provider {
        ProviderId::Anthropic => json!({"cache_control": {"type": "ephemeral"}}),
        ProviderId::OpenAi => json!({"prompt_cache_key": "myharness-stable"}),
        _ => json!(null),  // OpenAI 호환 (deepseek/minimax/local) 는 cache 미지원
    };
    
    AlignedPrompt {
        prefix: normalized_prefix,
        suffix: variable_suffix,
        cache_marker,
    }
}

pub struct CacheAlignerConfig {
    pub provider: ProviderId,  // Anthropic / OpenAI 만 지원
    pub include_auto_memory: bool,  // default true
    pub include_claude_md: bool,     // default true
}
```

**NFR 정합**:
- **NFR-PERF-3**: CacheAligner 단독 시 latency overhead < 50ms/turn. `normalize()` = O(N) 1회, < 10ms (claude-sonnet-4-5 의 30K stable prefix 기준).
- **CONCEPT.md §5.6 CacheAligner**: 1순위 — 가장 효과 큰 압축 (Anthropic prompt cache hit rate ↑).

### 5.3 ContentRouter (content type 자동 감지)

```rust
// myharness_context::compression::layer2::content_router
// 
// 목적: message content 의 type 감지 (JSON / code / text / log) → 적절한 압축 algo 분기.
//   - JSON (tool result) → SmartCrusher
//   - code (snippet) → CodeCompressor
//   - log / 자유 텍스트 → (v1.5+: Kompress-base)
// 
// detection:
//   1. syntactic: starts_with('{') or '[' → JSON, contains(`fn `| `class `| `import `) → code
//   2. MIME: ContentType header (HTTP response 의 경우)
//   3. AST (느리지만 정확): tree-sitter-rust parse 시도 → success 면 code
pub fn route(message: &Message) -> ContentKind {
    let body = message.content.as_text();
    
    // 1. JSON 감지
    if body.trim_start().starts_with('{') || body.trim_start().starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(body).is_ok() {
            return ContentKind::Json;
        }
    }
    
    // 2. code 감지 (tree-sitter 시도, 실패 시 heuristic)
    if let Ok(_) = tree_sitter_rust::parse(body) {
        return ContentKind::Code(CodeLang::Rust);
    }
    // 다른 언어 (v1.5+: tree-sitter-javascript, tree-sitter-python)
    if body.contains("def ") || body.contains("class ") || body.contains("import ") {
        return ContentKind::Code(CodeLang::Unknown);
    }
    
    // 3. log 감지 (timestamp + log level prefix)
    if REGEX_LOG.is_match(body) {
        return ContentKind::Log;
    }
    
    ContentKind::Text
}

#[derive(Debug, Clone, Copy)]
pub enum ContentKind {
    Json,
    Code(CodeLang),
    Log,
    Text,
}

#[derive(Debug, Clone, Copy)]
pub enum CodeLang {
    Rust,
    Unknown,
}
```

**5.4 SmartCrusher (JSON 구조 보존 압축, 65% ↓)**

```rust
// myharness_context::compression::layer2::content_router::smart_crusher
// 
// 목적: JSON tool result 의 key/structure 보존하며 value 압축.
//   - key 이름 축약 (e.g., "transcript_content" → "tc")
//   - whitespace 제거
//   - 중복 value reference 치환
//   - 65% 압축 목표 (CONCEPT.md §5.6 SmartCrusher)
pub fn crush(json_str: &str, config: &SmartCrusherConfig) -> String {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .expect("SmartCrusher input must be valid JSON");
    
    let crushed = match config.level {
        CrushLevel::Conservative => conservative_crush(value),
        CrushLevel::Aggressive => aggressive_crush(value, config.key_map.as_ref()),
    };
    
    serde_json::to_string(&crushed).unwrap()
}

fn aggressive_crush(mut v: serde_json::Value, key_map: Option<&KeyMap>) -> serde_json::Value {
    match &mut v {
        serde_json::Value::Object(map) => {
            // 1. key 이름 축약
            let mut new_map = serde_json::Map::new();
            for (k, val) in map.drain() {
                let short_k = key_map
                    .and_then(|m| m.shorten(&k))
                    .unwrap_or_else(|| shorten_key(&k));
                let crushed_val = aggressive_crush(val, key_map);
                new_map.insert(short_k, crushed_val);
            }
            *map = new_map;
        }
        serde_json::Value::Array(arr) => {
            // 2. 중복 value reference (v1.5+; v1 는 빈도 분석)
            *arr = arr.drain(..).map(|v| aggressive_crush(v, key_map)).collect();
        }
        serde_json::Value::String(s) => {
            // 3. 긴 string value 압축 (앞 50자 + "..." + hash)
            if s.len() > 200 {
                *s = format!("{}…<{}b>", &s[..50], s.len());
            }
        }
        _ => {}
    }
    v
}

pub struct SmartCrusherConfig {
    pub level: CrushLevel,
    pub key_map: Option<KeyMap>,  // custom shorten table (e.g., "transcript_content" → "tc")
}

pub enum CrushLevel { Conservative, Aggressive }
```

**5.5 CodeCompressor (AST-aware, tree-sitter)**

```rust
// myharness_context::compression::layer2::content_router::code_compressor
// 
// 목적: code snippet 의 identifier shorten + 주석/공백 제거. tree-sitter AST 기반.
//   - 식별자 (변수/함수/타입 이름) → a, b, c, ... 순으로 shorten
//   - 주석 (`//`, `/* */`, `///`) 제거
//   - import/use block 제거 (의미 없는 경우)
//   - 빈 줄 제거
//   - AST 구조 보존 (syntax error 발생 ❌)
pub fn compress(code: &str, lang: CodeLang) -> Result<String, CodeCompressError> {
    let source = code.as_bytes();
    let tree = match lang {
        CodeLang::Rust => tree_sitter_rust::parse(source)?,
        CodeLang::Unknown => tree_sitter_rust::parse(source)?,  // best-effort
    };
    
    let mut identifier_counter: HashMap<String, String> = HashMap::new();
    let mut next_id = 0;
    let mut output = String::with_capacity(source.len());
    
    // 1. AST traverse — identifier collect + comment/whitespace drop
    let mut cursor = tree.walk();
    traverse(&mut cursor, source, &mut identifier_counter, &mut next_id, &mut output)?;
    
    // 2. identifier 치환 (1-character a/b/c/...)
    let compressed = replace_identifiers(&output, &identifier_counter);
    
    Ok(compressed)
}

fn shorten_ident(name: &str, counter: &mut HashMap<String, String>, next: &mut usize) -> String {
    if let Some(short) = counter.get(name) {
        return short.clone();
    }
    let short = identifier_short_name(*next);
    counter.insert(name.to_string(), short.clone());
    *next += 1;
    short
}

fn identifier_short_name(n: usize) -> String {
    // a, b, c, ..., z, a1, b1, ... (aider repo-map 패턴)
    if n < 26 {
        ((b'a' + n as u8) as char).to_string()
    } else {
        format!("{}{}", (b'a' + (n % 26) as u8) as char, n / 26)
    }
}
```

**5.6 4 알고리즘 통합 (dispatch)**

```rust
// myharness_context::compression::layer2::dispatch
// 
// LLM call 직전, message 전체에 대해 4 알고리즘 적용.
// builtin.enabled: true 일 때만 실행.
pub async fn compress_all(
    messages: &mut Vec<Message>,
    config: &BuiltinConfig,
    llm_client: &LlmClient,
) -> Result<Layer2Outcome, Layer2Error> {
    if !config.enabled {
        return Ok(Layer2Outcome::Disabled);
    }
    
    let started_at = Instant::now();
    let before_tokens = total_tokens(messages);
    
    // 1. CacheAligner (prefix 안정화)
    if config.algorithms.cache_aligner {
        let aligned = cache_aligner::align(messages, &CacheAlignerConfig::default());
        // aligned.prefix 에 cache_control 마커 부착 (Anthropic API call 시 사용)
        // 본 함수 내에서는 message 분리만, 실제 API call 은 llm_client 가 처리
        *messages = [aligned.prefix, aligned.suffix].concat();
    }
    
    // 2. ContentRouter + SmartCrusher + CodeCompressor
    if config.algorithms.content_router {
        for msg in messages.iter_mut() {
            let kind = content_router::route(msg);
            match kind {
                ContentKind::Json => {
                    if config.algorithms.smart_crusher {
                        let crushed = smart_crusher::crush(
                            msg.content.as_text(),
                            &SmartCrusherConfig { level: CrushLevel::Aggressive, key_map: None },
                        );
                        msg.content = MessageContent::Text(crushed);
                    }
                }
                ContentKind::Code(lang) => {
                    if config.algorithms.code_compressor {
                        let compressed = code_compressor::compress(
                            msg.content.as_text(),
                            lang,
                        )?;
                        msg.content = MessageContent::Text(compressed);
                    }
                }
                ContentKind::Log | ContentKind::Text => {
                    // v1.5+: Kompress-base
                }
            }
        }
    }
    
    let after_tokens = total_tokens(messages);
    
    Ok(Layer2Outcome::Compressed {
        before_tokens,
        after_tokens,
        elapsed_ms: started_at.elapsed().as_millis(),
        ratio: after_tokens as f32 / before_tokens as f32,
    })
}
```

**5.7 NFR 정합**

| NFR | 본 §5 정합 |
| --- | --- |
| **NFR-PERF-3** CacheAligner < 50ms/turn | §5.2 `normalize()` < 10ms, 전체 dispatch < 50ms |
| **C-CTX-2** Layer 2 opt-in | §5.0 진입 — `builtin.enabled: false` default |
| **C-CTX-3** v1 우선 3 algo (D-37) | §5.0 — 4 algo (ContentRouter+SmartCrusher 분리 표기) |
| **C-CTX-4** 외부 headroom proxy ❌ | §5 전체 — 우리 Context component built-in (CONCEPT.md §0 NOT 5) |

---

## §6. TC scaffold (L1 Unit TC 8 — threshold/truncate/summarize/compact/dynamic lookup/atomicity/4 algo × 1 /compact handler)

### 6.1 TDD RED-GREEN-REFACTOR 진입점

v1 구현 시 TDD 진입점. INITIAL_DESIGN.md §12 + DD-1 §7 의 TC 패턴 정합. 각 TC 는 `planned / in_progress / done` 상태값 (CONCEPT.md §5.9 NFR-UX-4).

| # | TC ID | category | 시나리오 | status | 검증 방법 |
| - | --- | --- | --- | --- | --- |
| 1 | **TC-BT-01** | threshold | `accumulated_tokens = 159999 / model_length = 200000` → `should_compact() == false` | planned | §2.4 의사코드 그대로 |
| 2 | **TC-BT-02** | threshold | `accumulated_tokens = 160000 / model_length = 200000` → `should_compact() == true` (정확히 80% 경계) | planned | §2.4 의사코드 |
| 3 | **TC-BT-03** | truncate | messages 10개 + protect_recent=5 → 5개 반환 (오래된 5개 drop) | planned | §4.3 `truncate()` |
| 4 | **TC-BT-04** | summarize | messages 20개 + LLM mock → 1 summary + 5 recent = 6개 반환, ratio < 50% | planned | §4.4 + LlmClient mock |
| 5 | **TC-BT-05** | compact | layer1 dispatch — Hybrid mode + 80% 도달 → truncate + summarize 동시 발동 | planned | §4.5 `hybrid()` + LlmClient mock |
| 6 | **TC-BT-06** | dynamic lookup | `lookup_model_length(anthropic, claude-sonnet-4-5)` → cache miss → API call → 200000 반환 + cache write | planned | §2.3 + cache mock |
| 7 | **TC-BT-07** | atomicity | 2 thread 동시 `add_tokens(50000)` × 4회 → `accumulated_tokens == 200000` (race condition 없음) | planned | §2.4 + AtomicU32 stress test (loom or std::thread) |
| 8 | **TC-BT-08** | /compact handler | `myharness /compact --mode=truncate --force` → 80% 미만이어도 압축 실행, CompressionOutcome::Compressed 반환 | planned | §4.7 `slash::compact::run()` + clap Args parse |

### 6.2 TC 별 expected 결과 (RED 진입점)

```rust
// TC-BT-01/02 — threshold
#[test]
fn bt_01_under_threshold() {
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 159_999, 0);
    assert!(!tracker.should_compact());
}

#[test]
fn bt_02_at_threshold() {
    let tracker = BudgetTracker::new_for_test(200_000, 0.80, 160_000, 0);
    assert!(tracker.should_compact());
}

// TC-BT-07 — atomicity
#[test]
fn bt_07_concurrent_add_tokens() {
    use std::sync::Arc;
    use std::thread;
    let tracker = Arc::new(BudgetTracker::new_for_test(200_000, 0.80, 0, 0));
    let mut handles = vec![];
    for _ in 0..4 {
        let t = Arc::clone(&tracker);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                t.add_tokens(500);
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    assert_eq!(tracker.accumulated_tokens.load(Ordering::SeqCst), 200_000);
}

// TC-BT-08 — /compact handler
#[tokio::test]
async fn bt_08_compact_force_truncate() {
    let mut ctx = Context::new_for_test(/* ... */);
    ctx.budget.add_tokens(50_000);  // 25% — force 없이면 no-op
    let result = slash::compact::run(&mut ctx, CompactArgs {
        mode: Some(CompressionMode::Truncate),
        force: true,
        protect_recent: None,
    }).await.unwrap();
    assert!(matches!(result, CompactResult::Done { .. }));
}
```

### 6.3 TC × D-NFR 매트릭스

| TC | D-30 | D-27 | NFR-PERF-2 | NFR-PERF-3 | NFR-REL-4 | C-CTX-1 | C-CTX-2 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TC-BT-01 | ✅ | | ✅ | | ✅ | ✅ | |
| TC-BT-02 | ✅ | | ✅ | | ✅ | ✅ | |
| TC-BT-03 | ✅ | | ✅ | | ✅ | ✅ | |
| TC-BT-04 | ✅ | | ✅ | | ✅ | ✅ | |
| TC-BT-05 | ✅ | | ✅ | | ✅ | ✅ | |
| TC-BT-06 | ✅ | | | | ✅ | ✅ | |
| TC-BT-07 | ✅ | | | | ✅ | ✅ | |
| TC-BT-08 | ✅ | | ✅ | | ✅ | ✅ | |
| (Layer 2) | | ✅ | | ✅ | | | ✅ |
| (sum) | 8 | 1 (별도 §) | 6 | 1 (별도 §) | 8 | 8 | 1 (별도 §) |

> **Layer 2 별도 TC**: DD-2 범위 외이나 `TC-L2-01` ~ `TC-L2-04` 로 SmartCrusher 1 / CodeCompressor 1 / CacheAligner 1 / ContentRouter 1 = 4 TC 추가 가능. 본 §6 은 Layer 1 중심 8 TC.

---

## §7. handoff (D-26 4-필드)

### Summary

`docs/architecture/DETAILED_DESIGN_BUDGET.md` (본 DD-2) 작성 완료. **REVIEW.md §3.1 MAJOR-2 의 "input+output 누적 / provider model_length 동적 / system prompt 별도 budget" 권장을 그대로 채택** 하였으며, INITIAL_DESIGN.md §7.5 의 `pub struct BudgetTracker` 의사코드를 `AtomicUsize → AtomicU32` + `model_length` 동적 조회 + `system_prompt_tokens` 별도 + `swap_provider` + `/compact` handler 6개 항목 **정정**. 6 provider 별 model_length 표 (§3.1) 는 vendor default + 동적 조회 메커니즘 (§3.2) + 4-step lookup 우선순위 (§3.4) + user override escape hatch (§3.5) + minimax TBD graceful degrade (§3.6) 으로 구성. Layer 1 spec (§4) 은 truncate / summarize / hybrid 3 모드 + `maybe_compress()` dispatch + rate limit 보호 + `/compact` slash command handler. Layer 2 spec (§5) 은 v1 우선 4 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) 의사코드. L1 Unit TC 8 scaffold (§6) 제공 — TASK-005-1 구현자가 TDD RED-GREEN-REFACTOR 진입점 그대로 사용 가능.

### Risks

1. **TASK-005-1 구현자 의사코드 해석 오류 가능** — §2/§4/§5 의 Rust code 는 의사코드 (pseudocode) 다. error path / doc comment / integration test 미포함. 구현자 misinterpret 시 정합성 깨질 수 있음 → INITIAL_DESIGN.md §7.5 와 본 §2.7 6개 정정 항목 cross-check 필수.
2. **minimax model_length unknown** (D-28) — v1 Phase 1 = graceful degrade (§3.6), user 가 model_length_override 설정해야 정확 trigger. v1.5+ D-28 안정화 시 해결.
3. **3rd-party tokenizer drift** — tiktoken-rs 의 `cl100k_base` / `o200k_base` 가 Anthropic / Google 모델의 실제 tokenizer 와 정확히 일치하지 않을 수 있음 (Anthropic 은 자체 tokenizer 미공개, Google 도 일부 모델은 다름). §3.1 표 의 tokenizer 컬럼은 **근사치** 다. 실제 token count 가 ±5% 정도 차이 가능. trigger 가 5% 일찍/늦게 발동될 수 있으나 **NFR-PERF-2 ≤ 2s** 영향 ❌ (trigger 만 영향).
4. **CacheAligner 의 provider 한정** — Anthropic / OpenAI native 만 지원 (§5.2 cache_marker). deepseek / minimax / local 은 cache 미지원 → 효과 없음. v1 = 일관성 없는 효과 감수, v1.5+ provider 확장.
5. **ContentRouter 의 mis-detection** — JSON 감지 (regex) 가 false positive 가능 (e.g., prose with `{` brace). tree-sitter code 감지 (best-effort) 도 Unknown language 일 수 있음. mis-detection 시 SmartCrusher / CodeCompressor 가 prose 에 적용 → corruption 가능 → §6 TC-L2-01/02 에 corpus 테스트 필수.
6. **/compact force 시 과도 압축** — `--force` 사용 시 protect_recent=5 외 모두 제거. user 의 critical context 손실 가능. CLI confirm prompt 권장 (DD-1 hook eval 와 통합 검토).

### Suggested Follow-up

1. **TASK-005-1 (v1 Rust MVP 구현)** — 본 DD-2 + DD-1 (tool) + DD-3 (sub-agent) + DD-4 (hook) + DD-5 (session) 5-체인 입력으로 `myharness-context` crate 구현 시작. §6 TC 8 scaffold 부터 RED-GREEN-REFACTOR 진행.
2. **TASK-002 해소 후 review** — server/env 명령 가이드 수령 시 본 §4.7 `/compact --mode=truncate` 외 server-specific compact 옵션 (e.g., `--keep-server-logs`) 추가 검토.
3. **minimax D-28 안정화** (v1.5+) — base_url + API 형식 검증 후 §3.1 / §3.2 의 minimax row 갱신 + `state/auth/minimax.yaml` 자동 생성.
4. **tiktoken-rs vs vendor tokenizer** drift 검증 — Anthropic Claude 4.5 / Google Gemini 2.5 Pro 의 실제 tokenizer 와 tiktoken-rs 의 `cl100k_base` 정확도 측정. ±5% 이상 차이 시 §3.1 tokenizer 컬럼 vendor-specific library 로 교체 검토 (예: Anthropic `count-tokens` API).
5. **ContentRouter mis-detection corpus** — §6 TC-L2-01/02 corpus 작성 (DD-2 범위 외이나 v1 구현 시 권장). prose with braces / malformed JSON / multi-language code snippet 등 edge case 30+ sample.
6. **align 룰 확립** (D-23, D-35) — CONCEPT.md §5.6 / INITIAL_DESIGN.md §7 / 본 DD-2 3 문서 cross-ref. 향후 CONCEPT.md 갱신 시 3 문서 동시 align 필수.
7. **verifier 검증** — REVIEW.md §3.1 MAJOR-2 의 verify_prompt 10 항목 (input+output / 6 provider 표 / AtomicU32 / Layer 1+2 / /compact / TC 8 / cross-ref / 6 원칙 / 분량 500-800 / D-06) PASS/FAIL 판정.

### Produced Artifacts

- `docs/architecture/DETAILED_DESIGN_BUDGET.md` (메인 산출물, **본 DD-2, 1,277 lines / 8 sections, 분량 500-800 target 대비 +60% over-shoot — verifier 의 strict mode 판단 영역, DD-1 의 INITIAL_DESIGN.md 2,056 lines / 1,300 target +58% 와 동일 패턴**)
- `docs/team/deliverable_dd2.md` (early signal + final status, D-16 패턴 준수)
- `/Users/yklee/.mavis/plans/plan_746a17ad/outputs/dd-2/deliverable.md` (plan engine verifier 입력)
- `/Users/yklee/.mavis/plans/plan_746a17ad/board.md` (start + done entry append, minimal board noise)

### Cross-references (DD-2 → SSOT)

- 입력 SSOT: `docs/architecture/INITIAL_DESIGN.md` §3.4 (line 350-358) + §7.1-§7.5 (line 1433-1552), `docs/team/REVIEW.md` §3.1 MAJOR-2 (line 210-236), `docs/CONCEPT.md` §5.6 (line 372-451), `docs/REQUIREMENTS.md` §3.1 NFR-PERF-2/3 (line 453-454) + §3.7 NFR-REL-4 (line 522) + §4.4 C-CTX-1~4 (line 579-586)
- 결정 ID: **D-27** (Layer 2 headroom built-in) + **D-30** (Layer 1 always-on 2-계층) + **D-37** (v1 우선 3 algo) + **D-28** (6 provider, §3)
- 후속 task: **TASK-005-1** (v1 Rust MVP 구현) — 본 DD-2 + DD-1 (tool) + DD-3 (sub-agent) + DD-4 (hook) + DD-5 (session) 5-체인 입력
- 본 plan outputs: `/Users/yklee/.mavis/plans/plan_746a17ad/outputs/dd-2/deliverable.md`
- sibling deliverables: `docs/team/deliverable_dd1.md` (DD-1 tool spec), `docs/team/deliverable_dd3.md` (DD-3 sub-agent spec), `docs/team/deliverable_dd4.md` (DD-4 security-patterns), `docs/team/deliverable_dd5.md` (DD-5 session handoff)
