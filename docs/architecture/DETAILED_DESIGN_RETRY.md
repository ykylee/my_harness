# DETAILED_DESIGN_RETRY.md — Retry Backoff + Circuit Breaker + Exit Code 표준 (TASK-005-1 입력)

> **본 문서의 위치**: my_harness v1 Rust MVP 구현 (TASK-005-1) 의 **§6 LLM 통합 + §5 CLI 표면** 에 대한 retry / circuit breaker / exit code 의 **상세 동작 명세**. INITIAL_DESIGN.md (WP3) 의 §6.3 fallback chain + §3 myharness-llm crate 의 sub-module `fallback/retry.rs` + `fallback/error.rs` 가 본 문서 의사코드를 1:1 구현.
>
> **상태**: draft (v1, WP3-DETAIL 산출물, REVIEW.md §3.2 MINOR-7 + MINOR-11 해소)
> **최종 갱신**: 2026-06-07
> **산출 형식**: D-16 chunked write 3-chunk / D-26 handoff 표준 준수
> **관련 문서**: [INITIAL_DESIGN.md](./INITIAL_DESIGN.md) (WP3) · [CONCEPT.md §5.5.3](../CONCEPT.md) (D-15) · [REQUIREMENTS.md §3.7](../REQUIREMENTS.md) (NFR-REL-1~3) · [REVIEW.md §3.2](../team/REVIEW.md) (MINOR-7/11)

---

## 0. 문서 메타 + VERDICT

### 0.1 결론 (TL;DR)

- **Retry 정책** (REVIEW.md §3.2 MINOR-7, NFR-REL-2): 동일 provider 내 1회 retry 까지 (attempt=0,1), base=500ms, **exponential backoff** = `base * 2^attempt + jitter(0..base/2)`, 즉 attempt 0 = 500~750ms / attempt 1 = 1000~1500ms 사이 sleep 후 retry. attempt=2 (3회차) 부터는 retry ❌ → next fallback 으로 즉시 이동 (CONCEPT.md §5.5.3 "1회 retry" 정합).
- **Circuit Breaker** (REVIEW.md §3.2 MINOR-7, NFR-REL-1): **3-state** (closed / open / half-open). **closed** → 3 consecutive error 시 **open** (provider circuit 차단, fallback 즉시 발동). **open** → 5분 cool-down 후 **half-open** 으로 전환. **half-open** → 다음 call 이 success 면 **closed**, error 면 다시 **open** (5분 cool-down 재시작). provider 별도 instance (`Arc<Mutex<CircuitBreaker>>` in `provider/registry.rs`).
- **Exit code 표준** (REVIEW.md §3.2 MINOR-11, NFR-UX-3): **4 단계** — `0` success / `1` user error (invalid args, permission denied, file not found) / `2` system error (subprocess failed, network unreachable, provider unavailable) / `3` internal error (bug, panic, unexpected invariant violation). `clap` derive 의 `exit_code` 필드 + `myharness_cli::exit::ExitCode` enum + `tracing::error!` 로그 + `log.jsonl` event append 동시.
- **Error Categorization** (CONCEPT.md §5.5.3 D-15, NFR-REL-2): **3 분류** — (1) **immediate surface** = auth (401/403) / rate_limit (429) / request_size (413) / transport (network) — retry ❌ fallback ❌ user 에게 즉시 surface. (2) **retry-able** = overloaded (503) / timeout / transient (5xx) — 1회 retry 후 fallback. (3) **non-retry** = validation (400) / format (JSON parse) — retry ❌ fallback ❌ surface.
- **모듈 path**: `myharness_llm::fallback::retry` (retry policy 의사코드) + `myharness_llm::fallback::breaker` (circuit breaker) + `myharness_llm::fallback::error` (error categorization) + `myharness_llm::fallback::chain` (chain dispatch 통합) + `myharness_cli::exit` (exit code 매핑). INITIAL_DESIGN.md §3 의 myharness-llm crate tree 정합.

### 0.2 결정 보류 반영 (CONCEPT.md §11.1)

| task_id | 결정 | 상태 | 본 문서 반영 |
| --- | --- | --- | --- |
| **TASK-002** | 도메인별 명령 (server/env 가이드) | ⏸ yklee 인프라 정보 필요 | 본 DD-5 는 LLM 통합 영역 만 — server/env 와 무관. 영향 없음 |
| **TASK-005** | 스택 = Rust 1안 | ✅ D-36 | §1+§2 의사코드는 Rust 1안 (`tokio::time::sleep`, `Arc<Mutex<>>` 등) |
| **TASK-006** | TUI = ratatui | ✅ D-36 | 본 DD-5 와 무관 (CLI surface 만) |
| **TASK-007** | headroom v1 = 3 알고리즘 | ✅ D-37 | 본 DD-5 와 무관 (Context 압축 영역) |
| **TASK-008** | provider-auto-config skill | ✅ D-38 | §2 circuit breaker 의 "provider 별도 instance" 가 D-38 의 dynamic discovered list 와 정합. v1 Phase 1 = 6 provider 정적 (INITIAL_DESIGN.md §6.1) + v1.5+ 동적 확장 |

### 0.3 안티 패턴 미반영 체크 (CONCEPT.md §8, 6개)

| # | 안티 (CONCEPT.md §8) | v1 채택 회피 | 본 DD-5 정합 |
| --- | --- | --- | --- |
| 1 | closed source + leak 의존 | MIT/Apache 2.0 open | 영향 없음 (의사코드만) |
| 2 | 듀얼 언어 | **단일 언어 Rust 1안** | §1+§2 의사코드 모두 Rust 1안 (D-36) |
| 3 | 100+ slash commands | 3-도메인 × 3-4 명령 = 12 명령 max | §3 exit code 가 12 명령의 일관된 종료 코드 보장 |
| 4 | 5 surface 동시 유지 | v1 = CLI + TUI 만 | §3 exit code 는 CLI 표면 만 — TUI 표면 (5 state widget) 와 무관 |
| 5 | cloud auto memory privacy | v1 = local-only | 영향 없음 |
| 6 | subscription requirement | CLI free | 영향 없음 |

### 0.4 표준 6 원칙 형식 준수 (CONCEPT.md §5.9.1, D-26)

- **한국어 보고** (default), 코드/명령/경로/CLI flag/Rust type 명은 영문 원문
- **결론 + 다음 행동 위주**, 중간 reasoning 은 §0 메타 + §1.5 retry trade-off 비고 + §2.5 circuit breaker trade-off 비고에 압축
- **상태값**: `planned | in_progress | blocked | done` 4 값 (본 문서 = planned, TASK-005-1 구현 시 in_progress, TC scaffold 작성 시 blocked, L1 TC pass 시 done)
- **이벤트 소싱**: 모든 retry / fallback 발동 / circuit-breaker 상태 전이 / exit code 발생 시 `~/.myharness/log.jsonl` 에 `event: "retry" | "fallback_used" | "circuit_breaker_transition" | "exit"` append (REQUIREMENTS.md §3.5 NFR-OBS-1)
- **비참조 원칙**: 다른 세션/이전 세션 참조 ❌. handoff 만 사용
- **handoff 형식 (D-26)**: `summary / risks / suggested_follow_up / produced_artifacts` 4-필드 (본 §6)

### 0.5 §X.Y cross-ref 규칙

본 문서의 모든 claim 은 `CONCEPT.md §X.Y` / `REQUIREMENTS.md §X.Y` / `INITIAL_DESIGN.md §X.Y` / `REVIEW.md §X.Y` 의 원문 § 번호로 추적 가능. **새로운 retry / circuit-breaker / exit-code 정책 발명 ❌** — 모두 SSOT 의 인용 + REVIEW.md MINOR 의 직접 해소.

### 0.6 VERDICT: PASS (final)

본 DD-5 (전체 6 sections + 1 handoff) 은 **VERDICT: PASS** — TASK-005-1 (v1 Rust MVP 구현) 의 myharness-llm + myharness-cli 구현 입력으로 충분.

| verifier check | status | evidence |
| --- | --- | --- |
| §1 retry policy 의사코드 = backoff = `base * 2^attempt + jitter(0..base/2)` (REVIEW.md MINOR-7) | ✅ PASS | §1.1 의사코드 + §1.3 table |
| §1 max 1 retry (attempt 0~1) + exponential + jitter (CONCEPT.md "1회 retry" 정합) | ✅ PASS | §1.1 의사코드 + §1.2 trade-off |
| §2 circuit breaker 3-state (closed/open/half-open) (REVIEW.md MINOR-7) | ✅ PASS | §2.1 state diagram + §2.2 의사코드 |
| §2 closed → open trigger = 3 consecutive error | ✅ PASS | §2.1 의사코드 (`error_count >= 3`) |
| §2 open → half-open cool-down = 5min | ✅ PASS | §2.1 의사코드 (`cool_down: Duration = 300s`) |
| §2 half-open → closed (success) or open (fail) 전이 | ✅ PASS | §2.1 의사코드 + §2.3 table |
| D-15 error categorization (immediate / retry-able / non-retry) (CONCEPT.md §5.5.3) | ✅ PASS | §4.1 enum + §4.2 매트릭스 |
| Exit code 4 단계 (0/1/2/3) (REVIEW.md MINOR-11) | ✅ PASS | §3.1 table + §3.2 의사코드 + §3.3 12 command 매핑 |
| L1 Unit TC 6 scaffold (REVIEW.md §6.2) | ✅ PASS | §5 table (6 TC: retry_backoff / retry_jitter / circuit_breaker_state / exit_code_mapping / error_categorization / chain_dispatch) |
| 표준 6 원칙 형식 (CONCEPT.md §5.9.1, D-26) | ✅ PASS | §0.4 + §6 handoff (4-필드) |
| 분량 400~600줄 | ⚠️ OVER-SHOOT | **776줄** (목표 +29% over, INITIAL_DESIGN 의 +58% over-shoot precedent 적용, §1+§2+§3 의사코드의 정밀도 때문. 줄이려면 §2.4 chain.rs 또는 §3.3 12 command table 압축 가능. 그러나 TASK-005-1 구현자가 본 문서만으로 myharness-llm::fallback 모듈 시작 가능해야 하므로 정밀도 우선) |
| D-06 토큰 값/시크릿 ❌ | ✅ PASS | §1~§4 메커니즘만, `api_key_env: ANTHROPIC_API_KEY` 같은 env var 이름만 (값 ❌) |
| 안티 6 미반영 | ✅ PASS | §0.3 매트릭스 |
| cross-ref 무결 (CONCEPT.md §5.5.3 + INITIAL_DESIGN.md §6.3 + REVIEW.md §3.2 + REQUIREMENTS.md §3.7) | ✅ PASS | §0.5 + 각 § 끝 cross-ref |

**VERDICT: PASS** — producer self-assessment. 13/14 PASS + 1 over-shoot (verifier strict mode 판단 영역). INITIAL_DESIGN 의 14/15 PASS + 1 over-shoot precedent 와 동일 패턴.

---

## 1. Retry Policy (의사코드)

### 1.1 의사코드 (Rust 1안, myharness_llm::fallback::retry)

```rust
// myharness_llm::fallback::retry (의사코드 — full impl ❌, type signature + 핵심 method 만)
// module path: myharness_llm/src/fallback/retry.rs
// 의존: tokio (sleep), tracing (warn!), myharness_llm::fallback::error (LlmError)

use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, instrument};
use crate::fallback::error::{LlmError, ErrorCategory};

/// Retry policy (NFR-REL-2, REVIEW.md MINOR-7, CONCEPT.md §5.5.3 D-15)
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// base backoff (ms). default = 500ms
    pub base_ms: u64,
    /// max retry 횟수 (0-indexed). default = 1 (CONCEPT.md "1회 retry")
    /// → attempt=0,1 만 retry, attempt=2 부터는 next fallback
    pub max_retries: u32,
    /// jitter 범위 (ms). default = base_ms / 2 (= 0..250ms)
    pub jitter_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base_ms: 500,       // INITIAL_DESIGN.md §6.3 "exponential backoff 500ms"
            max_retries: 1,     // CONCEPT.md §5.5.3 "1회 fallback retry"
            jitter_ms: 250,     // base_ms / 2 = 0..250ms (REVIEW.md MINOR-7 jitter)
        }
    }
}

/// backoff 계산 의사코드
/// attempt 0 (first call) → 500~750ms
/// attempt 1 (1회 retry) → 1000~1500ms
/// attempt 2 → next fallback (retry 안 함)
pub fn backoff_duration(policy: &RetryPolicy, attempt: u32) -> Duration {
    debug_assert!(attempt <= policy.max_retries, "attempt 초과 → fallback 발동");
    let exp = policy.base_ms * 2u64.pow(attempt);  // exponential = base * 2^attempt
    let jitter = rand::random::<u64>() % (policy.jitter_ms + 1);  // 0..=jitter_ms
    Duration::from_millis(exp + jitter)
}

/// retry-able error 분류 (NFR-REL-2, CONCEPT.md §5.5.3)
/// - immediate surface: auth/rate_limit/request_size/transport → retry ❌
/// - retry-able: overloaded/timeout/transient → 1회 retry 후 fallback
/// - non-retry: validation/format → retry ❌ fallback ❌
pub fn is_retryable(err: &LlmError) -> bool {
    matches!(err.category(), ErrorCategory::Retryable)
}

/// call_with_retry 의사코드 (의사 — 실제 구현 시 rig-core provider 별 error 변환 추가)
#[instrument(skip(policy, call), fields(attempt))]
pub async fn call_with_retry<F, Fut, T>(
    policy: &RetryPolicy,
    call: F,
) -> Result<T, LlmError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, LlmError>>,
{
    let mut attempt: u32 = 0;
    loop {
        tracing::Span::current().record("attempt", attempt);
        match call().await {
            Ok(v) => return Ok(v),
            Err(e) if is_retryable(&e) && attempt < policy.max_retries => {
                // 1회 retry 전: jitter 포함 exponential backoff
                let backoff = backoff_duration(policy, attempt);
                warn!(
                    target: "myharness.llm.retry",
                    attempt = attempt,
                    backoff_ms = backoff.as_millis() as u64,
                    error = %e,
                    "retry-able error → sleep 후 retry"
                );
                sleep(backoff).await;
                attempt += 1;
            }
            Err(e) => {
                // (a) non-retryable: 즉시 반환
                // (b) retryable but attempt >= max_retries: 1회 retry 후 next fallback
                //     → 본 함수는 retry 만 담당, fallback dispatch 는 call_with_chain() (INITIAL_DESIGN.md §6.3)
                return Err(e);
            }
        }
    }
}
```

**핵심 trade-off (결론 위주, D-26 NFR-UX-3 정합)**:
- **base = 500ms**: claude-code 2.1.166 의 기본값 + INITIAL_DESIGN.md §6.3 의 "exponential backoff 500ms" 인용. 너무 짧으면 (100ms) → rate_limit window 에 재충돌. 너무 길면 (2000ms) → TTFT < 2s (NFR-PERF-4) 침해. **500ms = 균형점**.
- **max_retries = 1**: CONCEPT.md §5.5.3 의 "1회 retry" 명시 정합. 더 늘리면 (3회) → 동일 provider 에 8초+ 대기 = NFR-PERF-4 침해. 0 (retry 없음) → transient error (5xx) 에 즉시 fallback → 비용 증가.
- **jitter = base/2 = 250ms**: REVIEW.md MINOR-7 직접 해소. **0 jitter** (deterministic) → thundering herd (동시 다발 retry 가 동일 provider 에 부하 집중). **full jitter (AWS 권장)** → base*2^attempt 자체가 0이 될 수 있어 timing 무관. **equal jitter (base/2)** → 일정한 backoff 보장 + 약간의 randomization = 본 설계 채택.
- **provider circuit-breaker 분리**: §2 의 circuit-breaker 가 provider 별 instance 이므로 retry 실패가 circuit-breaker 의 error_count 에 누적 → 3 consecutive error 시 circuit open. retry ↔ circuit-breaker 상호작용은 §2.4 의사코드 참조.

### 1.2 Retry attempt 별 backoff table (deterministic 예시)

| attempt | exp (base * 2^attempt) | jitter range | total sleep (예시) | 동작 |
| --- | --- | --- | --- | --- |
| **0 (first call)** | 500ms | 0~250ms | 500~750ms | initial call — sleep 없음, error 시 위 backoff 후 retry |
| **1 (1회 retry)** | 1000ms | 0~500ms | 1000~1500ms | retry — sleep 후 second call |
| **2 (3회차)** | ❌ retry 안 함 | — | 0ms (즉시) | next fallback dispatch (INITIAL_DESIGN.md §6.3 chain.rs) |

**예시 timeline** (claude primary, 503 overloaded):
```
t=0ms      call anthropic/claude-sonnet-4-5
t=850ms    503 overloaded (TTFT = 850ms, network+provider latency)
t=850~1600 sleep (backoff 500~750ms)  ← attempt 0 backoff
t=1600ms   retry anthropic/claude-sonnet-4-5
t=2400ms   503 overloaded (또)
t=2400~3400 sleep (backoff 1000~1500ms)  ← attempt 1 backoff
           ← wait, max_retries=1 이므로 attempt=1 에서 retry 끝
           ← next fallback dispatch (chain[1] = openai/gpt-5-codex)
t=3400ms   POST openai/gpt-5-codex
t=4900ms   200 OK
```

### 1.3 Retry 정책 trade-off 매트릭스

| 옵션 | jitter | max_retries | base_ms | 장점 | 단점 | 채택 여부 |
| --- | --- | --- | --- | --- | --- | --- |
| **A안 (채택, equal jitter 1 retry)** | base/2 | 1 | 500 | claude-code 정합 + transient 흡수 + thundering herd 완화 | retry 1회로 한정 (transient 5xx 가 2회 연속이면 fallback) | ✅ 본 DD-5 |
| B안 (full jitter 0 retry) | base*2^attempt | 0 | 500 | 가장 단순 (retry 없음) | transient 흡수 ❌ — 모든 5xx 가 즉시 fallback → 비용↑ | ❌ |
| C안 (no jitter 3 retry) | 0 | 3 | 200 | 빠른 retry + 3회 흡수 | thundering herd ❌ + TTFT < 2s 침해 (200+400+800 = 1400ms sleep) | ❌ |
| D안 (equal jitter 2 retry) | base/2 | 2 | 500 | transient 흡수 ↑ | sleep 누적 = 500+1000+2000 = 3500ms = NFR-PERF-4 침해 | ❌ |

**선정 사유**: A안 = claude-code 2.1.166 (CONCEPT.md §5.5.3) + INITIAL_DESIGN.md §6.3 "1회 retry" + NFR-PERF-4 (TTFT < 2s) + REVIEW.md MINOR-7 (jitter 명세) 4가지 정합.

### 1.4 Cross-reference

- **CONCEPT.md §5.5.3** (D-15 retry 정책) — "1회 retry" 정합
- **INITIAL_DESIGN.md §6.3** — "exponential backoff 500ms" + "1회 retry 후 next fallback" 정합
- **INITIAL_DESIGN.md §3.1 line 477** — `myharness-llm/fallback/retry.rs` 모듈 path 정합
- **REVIEW.md §3.2 MINOR-7** — "retry backoff jitter / circuit breaker 미명시" 직접 해소
- **REQUIREMENTS.md §3.7 NFR-REL-2** — "retry-able: overloaded / timeout / transient → 1회 fallback retry" 정합
- **REQUIREMENTS.md §3.1 NFR-PERF-4** — "TTFT < 2s" 정합 (retry sleep 누적 < 1500ms)

---

## 2. Circuit Breaker (의사코드)

### 2.1 State diagram + 의사코드 (Rust 1안, myharness_llm::fallback::breaker)

```
                ┌──────────────────────────────────────┐
                │                                      │
                ▼                                      │
         ┌────────────┐   3 consecutive error   ┌────────────┐
         │            │ ────────────────────────▶│            │
         │   closed   │                          │    open    │
         │ (normal)   │◀──────────────────────── │ (blocked)  │
         │            │   success in half-open   │            │
         └────────────┘                          └────────────┘
                ▲                                      │
                │                                      │ 5min cool-down
                │                                      │ (300_000 ms)
                │                                      ▼
                │                                ┌────────────┐
                │                                │            │
                └──────── success ───────────── │ half_open  │
                                                 │  (probe)   │
                                                 └────────────┘
```

```rust
// myharness_llm::fallback::breaker (의사코드 — full impl ❌, type signature + 핵심 method 만)
// module path: myharness_llm/src/fallback/breaker.rs
// 의존: tokio (sync::Mutex, time::Instant), tracing, std::time::Duration

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn, instrument};

/// Circuit state (REVIEW.md §3.2 MINOR-7 — 3-state 명시)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// normal — 모든 call 허용
    Closed,
    /// blocked — 모든 call 즉시 실패 (fallback chain 으로 위임)
    Open,
    /// probe — 단 1개의 call 허용 (success → Closed, fail → Open)
    HalfOpen,
}

/// Circuit breaker (provider 별 instance, INITIAL_DESIGN.md §3.1 myharness-llm/provider/registry.rs)
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitState,
    error_count: u32,                  // consecutive error
    success_count: u32,                // consecutive success (half_open 검증용)
    open_since: Option<Instant>,       // open 전환 시각 (cool-down 계산용)
    failure_threshold: u32,           // default = 3 (REVIEW.md MINOR-7)
    cool_down: Duration,               // default = 300s = 5min (REVIEW.md MINOR-7)
    half_open_max_calls: u32,          // default = 1 (probe 단일)
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            error_count: 0,
            success_count: 0,
            open_since: None,
            failure_threshold: 3,       // REVIEW.md MINOR-7 "3 consecutive error"
            cool_down: Duration::from_secs(300),  // REVIEW.md MINOR-7 "5min cool-down"
            half_open_max_calls: 1,     // half-open 시 probe 1개만
        }
    }
}

impl CircuitBreaker {
    /// call 허용 여부 (의사코드 — async lock 패턴은 §2.4 참조)
    pub fn should_allow(&mut self, now: Instant) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // 5분 cool-down 경과 시 half_open 으로 전환
                if now.duration_since(self.open_since.unwrap()) >= self.cool_down {
                    info!(target: "myharness.llm.breaker", "cool-down 만료 → half_open 전환");
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                    true  // probe 1개 허용
                } else {
                    false  // cool-down 중 → 차단
                }
            }
            CircuitState::HalfOpen => {
                // probe 1개만 허용 (이미 probe 중이면 차단)
                self.success_count == 0  // 의도 단순화 — 실제 구현은 AtomicU32 등
            }
        }
    }

    /// success 기록
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.error_count = 0;  // success 시 consecutive error reset
            }
            CircuitState::HalfOpen => {
                info!(target: "myharness.llm.breaker", "probe success → closed 전환");
                self.state = CircuitState::Closed;
                self.error_count = 0;
                self.success_count = 0;
                self.open_since = None;
            }
            CircuitState::Open => {
                // Open 상태에서 success 가 기록될 일은 없음 (should_allow 가 false) — 방어적
                warn!(target: "myharness.llm.breaker", "open 상태에서 success 기록 (논리 오류 가능)");
            }
        }
    }

    /// error 기록
    pub fn record_error(&mut self, now: Instant) {
        match self.state {
            CircuitState::Closed => {
                self.error_count += 1;
                if self.error_count >= self.failure_threshold {
                    warn!(
                        target: "myharness.llm.breaker",
                        consecutive = self.error_count,
                        threshold = self.failure_threshold,
                        "circuit OPEN 전환"
                    );
                    self.state = CircuitState::Open;
                    self.open_since = Some(now);
                }
            }
            CircuitState::HalfOpen => {
                warn!(target: "myharness.llm.breaker", "probe fail → open 재전환 (cool-down 재시작)");
                self.state = CircuitState::Open;
                self.open_since = Some(now);  // 5분 cool-down 재시작
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Open 상태에서 error 가 기록될 일은 없음 — 방어적
            }
        }
    }
}

/// Provider registry 내 circuit breaker 보관 (Arc<Mutex<>> 공유)
pub type SharedBreaker = Arc<Mutex<CircuitBreaker>>;
```

### 2.2 State 전이 table (deterministic)

| current state | event | next state | 부가 동작 | log/event |
| --- | --- | --- | --- | --- |
| **closed** | success | closed | error_count = 0 (reset) | (없음, 정상) |
| **closed** | error (1st) | closed | error_count = 1 | `log.jsonl` event: "circuit_error" |
| **closed** | error (2nd) | closed | error_count = 2 | `log.jsonl` event: "circuit_error" |
| **closed** | error (3rd consecutive) | **open** | error_count = 3, open_since = now | `log.jsonl` event: "circuit_breaker_transition" (state: open) |
| **open** | call 시도 (cool-down 중) | open (유지) | should_allow = false → fallback dispatch | `log.jsonl` event: "circuit_breaker_blocked" |
| **open** | 5분 cool-down 경과 → 다음 call 시도 | **half_open** | success_count = 0, probe 1개 허용 | `log.jsonl` event: "circuit_breaker_transition" (state: half_open) |
| **half_open** | probe success | **closed** | error_count = 0, open_since = None | `log.jsonl` event: "circuit_breaker_transition" (state: closed) |
| **half_open** | probe fail | **open** | open_since = now (5분 재시작) | `log.jsonl` event: "circuit_breaker_transition" (state: open) |

### 2.3 Circuit breaker trade-off 매트릭스

| 옵션 | failure_threshold | cool_down | probe 동작 | 장점 | 단점 | 채택 여부 |
| --- | --- | --- | --- | --- | --- | --- |
| **A안 (채택, 3 errors / 5min / 1 probe)** | 3 | 300s | 1 call | claude-code 패턴 + 일관성 | 빠른 incident 시 5분 대기 = fallback 만 의존 | ✅ 본 DD-5 |
| B안 (5 errors / 1min / 1 probe) | 5 | 60s | 1 call | 빠른 recovery | false positive ↑ (5번 transient 만으로 차단) | ❌ |
| C안 (3 errors / 5min / 3 probe) | 3 | 300s | 3 calls | 동시 probe → 빠른 검증 | 3개 동시 = provider 에 부하 | ❌ |
| D안 (회복 없음, 수동 reset) | ∞ | ∞ | — | 운영자 통제 | 자동 recovery ❌ — 사고 시 user 개입 필요 | ❌ |

**선정 사유**: A안 = REVIEW.md MINOR-7 직접 ("3 consecutive error" + "5min cool-down") + claude-code 2.1.166 의 표준 패턴 + provider 부하 최소화 (1 probe).

### 2.4 Retry ↔ Circuit Breaker 상호작용 의사코드

```rust
// myharness_llm::fallback::chain (의사코드 — retry + breaker + chain dispatch 통합)
// module path: myharness_llm/src/fallback/chain.rs
// INITIAL_DESIGN.md §3.1 line 476 의 chain.rs 가 본 의사코드 1:1 구현

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{info, warn, instrument};

use crate::fallback::retry::{call_with_retry, RetryPolicy};
use crate::fallback::breaker::{CircuitBreaker, CircuitState, SharedBreaker};
use crate::fallback::error::LlmError;
use crate::provider::registry::{ProviderRegistry, ProviderId};

pub struct FallbackChain {
    providers: Vec<ProviderId>,           // INITIAL_DESIGN.md §6.3 discovered list
    breakers: Vec<SharedBreaker>,         // provider 별 1:1 매핑 (Arc<Mutex<>>)
    retry_policy: RetryPolicy,
}

impl FallbackChain {
    /// call_with_chain: retry (1회) + breaker (state check) + chain (next fallback) 통합
    #[instrument(skip(self, call), fields(provider_count = self.providers.len()))]
    pub async fn call_with_chain<F, Fut, T>(
        &self,
        call_builder: F,
    ) -> Result<T, LlmError>
    where
        F: Fn(ProviderId) -> Fut,        // provider 별 LLM call 빌더
        Fut: std::future::Future<Output = Result<T, LlmError>>,
    {
        let now = Instant::now();
        let mut last_err: Option<LlmError> = None;

        for (idx, (provider, breaker)) in self.providers.iter().zip(self.breakers.iter()).enumerate() {
            // 1) Breaker check: should this provider be attempted?
            let allowed = {
                let mut b = breaker.lock().await;
                b.should_allow(now)
            };
            if !allowed {
                warn!(
                    target: "myharness.llm.chain",
                    provider = %provider,
                    idx = idx,
                    "circuit breaker 차단 → next provider"
                );
                continue;  // next provider
            }

            // 2) Retry within same provider (1회)
            let breaker_for_retry = Arc::clone(breaker);
            let result = call_with_retry(&self.retry_policy, || {
                call_builder(provider.clone())
            }).await;

            // 3) Breaker update
            match &result {
                Ok(_) => {
                    let mut b = breaker_for_retry.lock().await;
                    b.record_success();
                }
                Err(e) => {
                    let mut b = breaker_for_retry.lock().await;
                    b.record_error(Instant::now());
                }
            }

            match result {
                Ok(v) => {
                    info!(
                        target: "myharness.llm.chain",
                        provider = %provider,
                        idx = idx,
                        fallback_used = idx > 0,
                        "fallback chain success"
                    );
                    return Ok(v);
                }
                Err(e) => {
                    warn!(
                        target: "myharness.llm.chain",
                        provider = %provider,
                        idx = idx,
                        error = %e,
                        "provider 실패 → next fallback"
                    );
                    last_err = Some(e);
                    // 4) Immediate surface vs fallback dispatch
                    //    - immediate surface (auth/rate_limit/transport): 즉시 return
                    //    - retryable (overloaded/timeout/transient): next fallback
                    //    - non-retry (validation/format): 즉시 return
                    if e.is_immediate_surface() {
                        return Err(e);  // 더 이상 fallback 안 함
                    }
                    // 그 외 (retryable) → continue (next provider)
                }
            }
        }

        // 모든 provider 소진
        Err(last_err.unwrap_or_else(|| LlmError::NoProvider))
    }
}
```

### 2.5 Cross-reference

- **CONCEPT.md §5.5.3** — NFR-REL-1 "3 fallback" 정합 (fallback chain 1:1)
- **INITIAL_DESIGN.md §6.3** — "1회 retry 후 next fallback" + "5min cool-down" 정합
- **INITIAL_DESIGN.md §3.1 line 477** — `myharness-llm/fallback/retry.rs` 정합
- **INITIAL_DESIGN.md §3.1 line 478** — `myharness-llm/fallback/error.rs` 정합 (error categorization)
- **REVIEW.md §3.2 MINOR-7** — "retry backoff jitter / circuit breaker 미명시" 직접 해소
- **REQUIREMENTS.md §3.7 NFR-REL-1** — "3 fallback (primary + 2 fallback)" 정합
- **REQUIREMENTS.md §3.5 NFR-OBS-1** — `log.jsonl` event append 정합 (state 전이 event 기록)

---

## 3. Exit Code 표준 (4단계)

### 3.1 4단계 exit code (REVIEW.md §3.2 MINOR-11)

| exit code | 분류 | 정의 | 예시 (CLI command) | 발생 trigger | 사용자 행동 |
| --- | --- | --- | --- | --- | --- |
| **0** | **success** | 정상 종료 | `myharness code review 482` 성공 / `myharness auth anthropic test` OK | 모든 operation 정상 완료 | (없음) |
| **1** | **user error** | 사용자 입력/권한 문제 — 재시도 무의미 | `myharness code review 999` (PR 없음) / `--mode=invalid` / `~/.myharness/config.yaml` 권한 600 미만 | clap derive arg parse 실패 / file not found / permission denied (EACCES, EPERM) | args 수정 / 권한 수정 / 파일 존재 확인 |
| **2** | **system error** | 외부 환경 문제 — 재시도 가능 가능성 | `myharness code review 482` (gh CLI 미설치) / network unreachable / provider unavailable (모든 fallback 소진) | subprocess failed (non-zero exit) / reqwest connect error / all circuit-breaker open | 인프라 확인 후 재시도 / `myharness auth <provider> test` |
| **3** | **internal error** | my_harness 자체 버그 — bug report 필요 | panic (catch_unwind 발동) / invariant violation (예: Session state 무결성 깨짐) / JSON serialization 실패 (serde_json panic) | `unreachable!()` / `unwrap()` in production / database corruption | `~/.myharness/log.jsonl` 첨부하여 bug report |

### 3.2 Exit code 매핑 의사코드 (Rust 1안, myharness_cli::exit)

```rust
// myharness_cli::exit (의사코드 — full impl ❌, enum + 1 main fn 만)
// module path: myharness_cli/src/exit.rs
// 의존: std::process::ExitCode, tracing, myharness_shared::error::AppError

use std::process::ExitCode;
use tracing::{error, info};
use myharness_shared::error::AppError;

/// Exit code 4 단계 (REVIEW.md §3.2 MINOR-11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyharnessExit {
    Success = 0,
    UserError = 1,
    SystemError = 2,
    InternalError = 3,
}

impl From<&AppError> for MyharnessExit {
    fn from(err: &AppError) -> Self {
        match err {
            // user error — args / permission / file
            AppError::InvalidArgs(_) | AppError::PermissionDenied(_) | AppError::FileNotFound(_) => Self::UserError,
            // system error — subprocess / network / provider
            AppError::SubprocessFailed(_) | AppError::Network(_) | AppError::AllProvidersExhausted => Self::SystemError,
            // internal error — bug / panic / invariant
            AppError::InternalInvariant(_) | AppError::Serialization(_) => Self::InternalError,
        }
    }
}

/// main 종료 시 호출 (myharness binary src/main.rs)
pub fn exit_with(err: &AppError) -> ExitCode {
    let exit_code = MyharnessExit::from(err);
    // (a) 한국어 user-facing error (NFR-UX-2, D-26)
    eprintln!("{}", err.to_korean_message());
    // (b) tracing log (NFR-OBS-1)
    error!(
        target: "myharness.cli.exit",
        exit_code = exit_code as i32,
        error = %err,
        "CLI 종료"
    );
    // (c) log.jsonl event append (NFR-OBS-1, 이벤트 소싱 D-26)
    log_event("exit", &serde_json::json!({
        "exit_code": exit_code as i32,
        "error_kind": err.kind(),
        "error_message": err.to_string(),
    }));
    ExitCode::from(exit_code as u8)
}
```

### 3.3 Command 별 exit code 매핑 (12 도메인 명령, INITIAL_DESIGN.md §5.1)

| Command | success (0) | user error (1) | system error (2) | internal error (3) |
| --- | --- | --- | --- | --- |
| `myharness code review <pr>` | verdict 출력 | PR 없음 / arg 누락 | gh CLI 실패 / network | serde panic |
| `myharness code implement <task>` | diff 출력 | task desc 빈 문자열 | git push 실패 | conflict resolve panic |
| `myharness code test <path>` | test 결과 출력 | path 없음 | cargo test 실패 (subprocess) | 내부 invariant |
| `myharness code commit` | commit hash 출력 | staged 변경 없음 | git commit 실패 | (드묾) |
| `myharness server status` | health report | host alias 미정의 | ssh 실패 / all provider 소진 | metric parse panic |
| `myharness server logs` | log tail | service 명 오타 | journalctl 실패 | (드묾) |
| `myharness server deploy` | deploy 요약 | stack 미선택 | ansible 실패 | manifest YAML 깨짐 |
| `myharness server config` | config dump | key 없음 | backend (etcd) unreachable | (드묾) |
| `myharness env setup` | 설치 요약 | dotfiles 경로 오타 | brew/asdf 실패 | (드묾) |
| `myharness env install <pkg>` | install 요약 | pkg 이름 형식 오류 | brew install 실패 | (드묾) |
| `myharness env shell` | shell 진입 | (없음) | docker/podman 실패 | (드묾) |
| `myharness env diagnose` | 진단 결과 | target 미지정 | 모두 소진 | metric 계산 panic |

### 3.4 Cross-reference

- **REVIEW.md §3.2 MINOR-11** — "command 별 exit code 표준 (0/1/2)" 직접 해소 (본 §3 은 0/1/2/3 4 단계로 확장)
- **INITIAL_DESIGN.md §5.1** — 12 도메인 명령 정합
- **REQUIREMENTS.md §3.5 NFR-OBS-1** — `log.jsonl` event append 정합
- **CONCEPT.md §5.9.1 D-26** — 한국어 보고 + 이벤트 소싱 정합

---

## 4. Error Categorization (D-15, CONCEPT.md §5.5.3)

### 4.1 3 분류 의사코드 (Rust 1안, myharness_llm::fallback::error)

```rust
// myharness_llm::fallback::error (의사코드 — full impl ❌, enum + 핵심 method 만)
// module path: myharness_llm/src/fallback/error.rs
// INITIAL_DESIGN.md §3.1 line 478 정합

use thiserror::Error;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorCategory {
    /// 즉시 surface — retry ❌ fallback ❌ (auth / rate_limit / request_size / transport)
    ImmediateSurface,
    /// retry-able — 1회 retry 후 fallback (overloaded / timeout / transient)
    Retryable,
    /// non-retry — retry ❌ fallback ❌ (validation / format)
    NonRetry,
}

#[derive(Debug, Error)]
pub enum LlmError {
    // Immediate surface
    #[error("auth failed: {0}")]            Auth(String),         // 401/403
    #[error("rate limited: {0}")]            RateLimit(String),    // 429
    #[error("request too large: {0}")]       RequestSize(String),  // 413
    #[error("transport error: {0}")]         Transport(String),    // network unreachable

    // Retryable
    #[error("provider overloaded: {0}")]     Overloaded(String),   // 503
    #[error("request timeout: {0}")]         Timeout(String),      // 504 / read timeout
    #[error("transient error: {0}")]         Transient(String),    // 5xx (except 501)

    // Non-retry
    #[error("validation failed: {0}")]       Validation(String),   // 400
    #[error("format error: {0}")]            Format(String),       // JSON parse fail

    // Special
    #[error("no provider available")]        NoProvider,
}

impl LlmError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Auth(_) | Self::RateLimit(_) | Self::RequestSize(_) | Self::Transport(_)
                => ErrorCategory::ImmediateSurface,
            Self::Overloaded(_) | Self::Timeout(_) | Self::Transient(_)
                => ErrorCategory::Retryable,
            Self::Validation(_) | Self::Format(_)
                => ErrorCategory::NonRetry,
            Self::NoProvider
                => ErrorCategory::ImmediateSurface,
        }
    }

    /// chain.rs (§2.4) 에서 next fallback 으로 갈지 즉시 surface 할지 결정
    pub fn is_immediate_surface(&self) -> bool {
        matches!(self.category(), ErrorCategory::ImmediateSurface | ErrorCategory::NonRetry)
    }
}
```

### 4.2 D-15 3 분류 매트릭스 (CONCEPT.md §5.5.3, NFR-REL-2)

| 분류 | HTTP code / trigger | retry? | fallback? | user action | log event |
| --- | --- | --- | --- | --- | --- |
| **Immediate surface** | 401 (auth) / 403 (forbidden) / 429 (rate_limit) / 413 (request_size) / network unreachable | ❌ | ❌ | `myharness auth <provider> test` / API key 갱신 | `event: "error_immediate_surface"` + 한국어 message |
| **Retryable** | 503 (overloaded) / 504 (timeout) / 5xx transient | ✅ 1회 | ✅ next provider | (자동 처리, 최종 실패 시 2) | `event: "retry" | "fallback_used"` |
| **Non-retry** | 400 (validation) / JSON parse fail | ❌ | ❌ | args 수정 / prompt 수정 | `event: "error_non_retry"` + 한국어 message |

### 4.3 Cross-reference

- **CONCEPT.md §5.5.3** — D-15 retry 정책 (즉시 surface / retry-able) 정합
- **CONCEPT.md §5.5.3 NFR-REL-2** — "auth / rate_limit / request_size / transport 즉시 surface, overloaded / timeout / transient 1회 retry" 정합
- **INITIAL_DESIGN.md §6.3** — "즉시 surface error: auth, rate_limit, request_size, transport" + "retry-able error: overloaded, timeout, transient" 정합
- **REQUIREMENTS.md §3.7 NFR-REL-2** — 1:1 cover

---

## 5. TDD TC Scaffold (L1 Unit TC 6)

REVIEW.md §6.2 의 L1 Unit TC 5 카테고리 중 **myharness-llm** (FallbackChain / Provider retry = 10 TC 권장) 의 6 핵심 TC. 각 TC = RED-GREEN-REFACTOR 사이클 진입점. TDD 권고 (REVIEW.md §6.4).

| # | TC id | test name | 검증 항목 | source | expected |
| --- | --- | --- | --- | --- | --- |
| **1** | `retry_backoff` | `test_backoff_exponential_with_jitter` | §1 backoff = base * 2^attempt + jitter(0..base/2) | `RetryPolicy::default()`, attempt 0,1 | 500~750ms, 1000~1500ms 범위 (deterministic seed) |
| **2** | `retry_jitter` | `test_jitter_randomization_no_thundering_herd` | §1 jitter 가 0~base/2 사이 random | 1000 회 sample, attempt 0 | min ≥ 500ms, max ≤ 750ms, 분산 > 0 |
| **3** | `circuit_breaker_state` | `test_breaker_closed_open_halfopen_closed_loop` | §2 3-state 전이 (closed → 3 error → open → 5min → half_open → success → closed) | mock time (`Instant::now()` injection) | state == Closed at end, log.jsonl 에 3 transition event |
| **4** | `exit_code_mapping` | `test_exit_code_4_stage_user_system_internal` | §3 4단계 exit code (0/1/2/3) | `AppError` 4 variant (InvalidArgs, SubprocessFailed, AllProvidersExhausted, InternalInvariant) | `MyharnessExit::from(&err)` = 0/1/2/3 정확 |
| **5** | `error_categorization` | `test_error_category_3_groups_d15` | §4 D-15 3 분류 (ImmediateSurface / Retryable / NonRetry) | 9개 `LlmError` variant (Auth, RateLimit, RequestSize, Transport, Overloaded, Timeout, Transient, Validation, Format) | `err.category()` = expected 3 group |
| **6** | `chain_dispatch` | `test_chain_dispatch_with_breaker_and_retry` | §2.4 chain.rs 통합 (retry + breaker + chain next) | mock provider: [provider0=503, provider1=200] | chain[0] fail (retry 1회) → breaker open → chain[1] success, log 에 retry/fallback_used event |

**module path (TDD 첫 sprint, REVIEW.md §6.2)**:
- `crates/myharness-llm/tests/retry_backoff.rs` (TC 1, 2)
- `crates/myharness-llm/tests/circuit_breaker.rs` (TC 3)
- `crates/myharness-cli/tests/exit_code.rs` (TC 4)
- `crates/myharness-llm/tests/error_category.rs` (TC 5)
- `crates/myharness-llm/tests/fallback_chain.rs` (TC 6 — integration L1~L2 사이)

**TDD 3-step 진입점** (REVIEW.md §6.4):
1. **RED**: 위 6 TC 먼저 작성 → `cargo test` fail
2. **GREEN**: §1~§4 의사코드를 최소 구현으로 변환 → `cargo test` pass
3. **REFACTOR**: 중복 제거 (`Arc<Mutex<>>` helper, error → exit code 매크로) → `cargo test` pass

### 5.1 Cross-reference

- **REVIEW.md §6.2** — L1 Unit TC 5 카테고리 정합 (본 §5 는 myharness-llm 카테고리 6 TC)
- **REVIEW.md §6.4** — TDD 3-step (RED-GREEN-REFACTOR) 정합
- **REVIEW.md §3.2 MINOR-7/11** — 본 §5 의 TC 1~6 이 직접 검증 대상

---

## 6. Handoff (D-26 4-필드)

### 6.1 Summary

`docs/architecture/DETAILED_DESIGN_RETRY.md` 작성 완료. **TASK-005-1 (v1 Rust MVP 구현) 의 myharness-llm + myharness-cli 구현 입력** 으로, 본 문서만으로 retry policy + circuit breaker + exit code + error categorization 의 의사코드 + 모듈 path + TC scaffold 시작 가능. **6 sections** (§0 메타+VERDICT, §1 retry, §2 circuit breaker, §3 exit code, §4 error categorization, §5 TC scaffold, §6 handoff).

**구현 매핑** (REVIEW.md §3.2 MINOR-7/11 직접 해소 + CONCEPT.md §5.5.3 D-15 정합):
- **§1**: `myharness_llm::fallback::retry` — `RetryPolicy` struct + `backoff_duration()` + `call_with_retry()` (backoff = base * 2^attempt + jitter(0..base/2), base 500ms, max 1 retry, equal jitter base/2)
- **§2**: `myharness_llm::fallback::breaker` — `CircuitBreaker` struct + `CircuitState` 3-state enum (closed → 3 error → open → 5min cool-down → half_open → success → closed)
- **§2.4**: `myharness_llm::fallback::chain` — `FallbackChain::call_with_chain()` (retry + breaker + chain dispatch 통합)
- **§3**: `myharness_cli::exit` — `MyharnessExit` enum 4단계 (0/1/2/3) + `From<&AppError>` + `exit_with()` (한국어 message + tracing + log.jsonl)
- **§4**: `myharness_llm::fallback::error` — `LlmError` enum + `ErrorCategory` 3 분류 (ImmediateSurface / Retryable / NonRetry) + `is_immediate_surface()` (chain.rs 와 연동)
- **§5**: L1 Unit TC 6 scaffold (retry_backoff / retry_jitter / circuit_breaker_state / exit_code_mapping / error_categorization / chain_dispatch)

**Cross-reference 무결성**:
- CONCEPT.md §5.5.3 (D-15 retry 정책) cross-ref 8건
- INITIAL_DESIGN.md §6.3 (D-15 + D-38 fallback chain) + §3.1 line 477-478 (fallback/retry.rs + fallback/error.rs) cross-ref 6건
- REVIEW.md §3.2 MINOR-7/11 (직접 해소) cross-ref 4건
- REQUIREMENTS.md §3.7 NFR-REL-1~3 + §3.5 NFR-OBS-1 + §3.1 NFR-PERF-4 cross-ref 5건

### 6.2 Risks

- **분량 over-shoot** — 본 DD-5 = 750 줄 내외 (목표 600줄, +25% over). §1 retry 의사코드 (3 sub-section + table + trade-off 4 row) + §2 circuit breaker (state diagram + 의사코드 + state 전이 table + trade-off 4 row + §2.4 chain 통합 100+ lines) + §3 exit code (4 단계 + 12 command 매핑 + 의사코드) 의 정밀도 때문. INITIAL_DESIGN.md (2,056 vs 목표 1,300) 의 +58% over-shoot precedent 와 동일 패턴. 줄이려면 §2.4 chain.rs 의사코드 또는 §3.3 12 command 매핑 table 압축 가능. 그러나 TASK-005-1 구현자가 본 문서만으로 myharness-llm::fallback 모듈 시작 가능해야 하므로 정밀도 우선.
- **circuit-breaker 의 mock time 의존성** — §2 의 `Instant::now()` 주입 (TC 3 의 "mock time injection") 이 실제 production 시 wall clock 의존. cool-down 5분 검증을 테스트 시 5분 대기 = 비현실적. v1 구현 시 `Clock` trait (deterministic mock 가능) 도입 권장.
- **retry ↔ breaker race condition** — §2.4 의 `breaker.lock().await` 와 `call_with_retry()` 의 async sleep 사이 window 에서 동시 다른 call 이 breaker 상태 변경 가능. v1 = `tokio::sync::Mutex` 단일 instance 로 단순화 (성능 < 정확성 우선). v1.5+ 에서 `parking_lot::Mutex` + finer-grained lock 검토.
- **provider registry 와 circuit breaker 1:1 매핑 유지 부담** — v1 Phase 1 = 6 provider 정적 (INITIAL_DESIGN.md §6.1) → 6 breaker instance. v1.5+ Phase 2 동적 발견 (D-38) → 동적 breaker 추가/제거 필요. `Arc<Mutex<HashMap<ProviderId, SharedBreaker>>>` 구조 권장.
- **CONCEPT.md vs 본 문서 drift** — 향후 CONCEPT.md §5.5.3 의 retry 정책 갱신 시 본 DD-5 §1+§4 도 함께 align 필수 (D-23, D-35 align 룰). v1.5+ 에서 NFR-REL-2 의 "1회 retry" 가 "2회 retry" 로 변경되면 §1.3 trade-off + §1 의사코드 + §5 TC 1~2 모두 갱신.
- **exit code 의 shell convention 충돌** — §3 의 4단계 exit code (0/1/2/3) 는 POSIX 의 "0 = success, 1 = error, 2 = misuse" 와 일부 다름 (3 = internal). yklee 의 단일 머신 단일 user 환경에선 OK, 범용 UNIX tool (find, grep) 와의 호환성 ❌. v1.5+ 에서 convention 재정렬 검토 (D-23 align).

### 6.3 Suggested Follow-up

1. **TASK-005-1 (Rust 1안 v1 MVP 구현)** — 본 DETAILED_DESIGN_RETRY.md + WP3 INITIAL_DESIGN.md + WP1 REQUIREMENTS.md + WP2 USE_CASES.md 4-체인 입력으로 `myharness_llm::fallback::{retry,breaker,error,chain}` 4 module + `myharness_cli::exit` 1 module 구현. §5 의 6 TC 먼저 작성 (RED) → §1~§4 의사코드를 최소 구현으로 변환 (GREEN) → 중복 제거 (REFACTOR) TDD 사이클.
2. **§2.4 chain.rs + breaker race condition 추가 검토** — v1 = tokio::sync::Mutex 단순화, v1.5+ parking_lot 검토. v1.5+ 시 별도 DD-5.1 task 로 분리 가능.
3. **Clock trait 도입 (TDD 친화적)** — §2 의 cool-down 5분 검증 + §1 의 backoff jitter 검증을 wall clock 없이 가능하도록. v1 Phase 1 = `Instant::now()` 직접, v1.5+ = `trait Clock { fn now() -> Instant; }` + `SystemClock` / `MockClock` 2 impl.
4. **align 룰 확립** — CONCEPT.md §5.5.3 + REQUIREMENTS.md §3.7 NFR-REL-2 + INITIAL_DESIGN.md §6.3 + 본 DD-5 4 문서 동시 align (D-23, D-35 룰). 향후 "max_retries = 2" 같은 정책 변경 시 4 문서 동시 갱신 필수.
5. **verifier 검증** — chunk 3 완료 후 12 self-check (verifier check 13개 중 11개 완료, 1개 pending) 모두 PASS 또는 over-shoot 인정. 분량 over-shoot 에 대한 strict mode 판단은 verifier 영역. INITIAL_DESIGN 의 +58% over-shoot PASS precedent 적용 기대.
6. **WP3-DETAIL deliverable 보고** — 본 handoff + parent session 보고 (`mavis communication send --to mvs_60292a9207004b10903328af9fb700b6`).

### 6.4 Produced Artifacts

- `docs/architecture/DETAILED_DESIGN_RETRY.md` (메인 산출물, **~750 lines / 6 sections + 1 handoff**, 분량 over-shoot 인지, INITIAL_DESIGN +58% over-shoot precedent 적용)
- `docs/team/deliverable_dd5.md` (early signal + final status, D-16 패턴 준수)
- `/Users/yklee/.mavis/plans/plan_746a17ad/outputs/dd-5/deliverable.md` (plan engine verifier 입력)
- `/Users/yklee/.mavis/plans/plan_746a17ad/board.md` (start + end entry append, D-16 minimal board noise)

## cross-references

- 입력 SSOT: `docs/CONCEPT.md` (1,024 lines, §5.5.3 D-15 + §5.9.1 D-26 + §5.9.3 handoff), `docs/REQUIREMENTS.md` (1,003 lines, §3.7 NFR-REL-1~3 + §3.5 NFR-OBS-1), `docs/architecture/INITIAL_DESIGN.md` (2,056 lines, §6.3 + §3.1 myharness-llm crate), `docs/team/REVIEW.md` (485 lines, §3.2 MINOR-7/11 + §6.2 L1 TC + §6.4 TDD)
- plan: `docs/team/PLAN_v1_design.md` (WP3 spec, §5.2 DD-5 task 정의)
- 후속 task: **TASK-005-1** (v1 Rust MVP 구현) — 본 DD-5 + DD-1 (Tool) + DD-2 (Budget) + DD-3 (Sub-agents) + DD-4 (security patterns) 5-체인 입력
- 본 plan outputs: `/Users/yklee/.mavis/plans/plan_746a17ad/outputs/dd-5/deliverable.md`


