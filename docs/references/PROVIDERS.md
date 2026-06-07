# LLM Provider 추상화 — 비교 검토 (2026-06-07)

- 문서 목적: TASK-005 my_harness CLI/TUI 의 LLM 통합 — *어떤 프로바이더와도 통신 가능* 한 추상화 옵션 비교 검토.
- 범위: 1안(Rust) / 2안(TypeScript) 별로 가능한 라이브러리 + 직접 구현 옵션 + 우리 권장안.
- 대상 독자: yklee, Mavis, TASK-005 디자인 리뷰.
- 상태: draft (1차 검토).
- 최종 수정일: 2026-06-07.

## 1. 왜 추상화가 필요한가

my_harness 는 3개 도메인 (코드/서버/환경) 작업. 각 도메인마다 최적 모델이 다름:
- **코드**: Claude Sonnet/Opus, GPT-5, Gemini 2.5 Pro
- **서버**: Claude Haiku (저비용), Gemini Flash (저비용 + 빠름)
- **환경**: 로컬 Ollama (오프라인), Claude Haiku

→ **provider lock-in 없이 모든 모델** 사용 가능해야. v1 부터.

## 2. 후보 옵션 (스택별)

### 2.1 Rust 1안 후보

| 라이브러리 | 별 | provider 수 | streaming | tool calling | cache | 비고 |
| --- | --- | --- | --- | --- | --- | --- |
| **rig-core** | ⭐⭐⭐ | 12+ (OpenAI, Anthropic, Cohere, Google, Ollama, etc.) | ✅ | ✅ | Anthropic prompt caching | actively maintained, 2025 v0.5+ 안정 |
| **genai** (69corp) | ⭐⭐ | 8+ (OpenAI, Anthropic, Google, Ollama) | ✅ | ✅ | ❌ | Rust native, smaller scope |
| **llm-chain** (sobelio) | ⭐ | 6+ | ✅ | ✅ | ❌ | LCEL-like chains, lower activity |
| **openai-rs** | ⭐ | OpenAI only | ✅ | ✅ | ❌ | provider 1개라 부적합 |
| **async-openai** | ⭐ | OpenAI 호환 (Azure, Together, etc.) | ✅ | ✅ | ❌ | openai 호환만 |
| **직접 HTTP** | ❌ | 1 | 1 | 1 | 1 | 모든 provider 직접 구현 (수십~수백 시간) |

**추천: `rig-core`** ⭐
- 12+ provider 통합 (Anthropic, OpenAI, Google, Cohere, xAI, Mistral, DeepSeek, Ollama, etc.)
- Tool calling, streaming, prompt caching 모두 지원
- 활발한 개발 (2025-2026), 정식 release
- 우리 `Mavis` 가 이미 OpenCode 와 정합성 있음 (rig 가 OpenCode 의 1차 옵션)

### 2.2 TypeScript 2안 후보

| 라이브러리 | 별 | provider 수 | streaming | tool calling | cache | 비고 |
| --- | --- | --- | --- | --- | --- | --- |
| **Vercel AI SDK (`ai`)** | ⭐⭐⭐ | 15+ (OpenAI, Anthropic, Google, Mistral, Cohere, Groq, Ollama, etc.) | ✅ | ✅ | Anthropic prompt caching | 사실상 표준, React/Vercel ecosystem |
| **@modelcontextprotocol/sdk** | ⭐ | N/A (MCP 표준) | ✅ | ✅ (MCP) | ❌ | provider 직접 통합 아님, MCP server 로 도구 노출 |
| **ai-sdk-provider-* (커뮤니티)** | ⭐⭐ | +5 (DeepSeek, OpenRouter, etc.) | ✅ | ✅ | ❌ | Vercel AI SDK 확장 |
| **LangChain.js** | ⭐ | 30+ | ✅ | ✅ | ❌ | 너무 무거움, 추상화 깊음 |
| **OpenAI Node SDK** | ⭐ | OpenAI only + 호환 | ✅ | ✅ | ❌ | provider 1개 + 호환 |
| **직접 HTTP** | ❌ | 1 | 1 | 1 | 1 | 동일 |

**추천: `Vercel AI SDK` (`ai`)** ⭐
- 15+ provider 1급 지원
- tool calling, streaming, prompt caching 모두 지원
- 활발한 생태계, 1차 옵션
- 우리 1차 (opencode/gemini-cli) 와 정합

### 2.3 Cross-runtime 옵션 (어느 스택이나)

| 옵션 | 설명 | trade-off |
| --- | --- | --- |
| **litellm (Python)** | subprocess + HTTP proxy. 우리 my_harness 가 litellm HTTP 에 요청. | 우리 1안/2안 어느 스택이든 litellm client library 만 쓰면 OK. **장점**: provider 100+, 검증. **단점**: Python subprocess 필요 (Rust 1안은 묶음 복잡) |
| **OpenRouter** | HTTP proxy. 모든 모델을 한 endpoint 에서. | **장점**: 단일 API key, 100+ model. **단점**: 중간 수수료, 외부 의존 |
| **Together AI / Fireworks / Anyscale** | 특정 provider 의 fast inference | provider-specific, 우리 lock-in 회피 목표와 충돌 |

## 3. 우리 권장안

### 3.1 1안 (Rust) — `rig-core` + 직접 통합

```rust
// 의사코드
use rig::{providers, completion::Chat};

let client = providers::anthropic::Client::new(&api_key);
let agent = client.agent("claude-sonnet-4-5")
    .preamble("You are a coding agent...")
    .tool(ReadFile::new())
    .tool(WriteFile::new())
    .build();

let response = agent.prompt(user_input).await?;
```

**장점**:
- 단일 binary 에 모든 provider 통합
- Rust type system 으로 provider 차이 컴파일타임 검증
- Streaming + tool calling + prompt caching 모두 1급

**단점**:
- provider 추가 시 rig-core PR 필요 (또는 직접 HTTP)
- 우리 my_harness 가 rig-core 의 디자인 결정에 종속

### 3.2 2안 (TypeScript) — `Vercel AI SDK` + MCP

```typescript
// 의사코드
import { anthropic } from '@ai-sdk/anthropic';
import { generateText, tool } from 'ai';

const result = await generateText({
  model: anthropic('claude-sonnet-4-5'),
  prompt: userInput,
  tools: { readFile, writeFile, ... },
});
```

**장점**:
- React/Vercel ecosystem 과 정합
- 15+ provider 1급
- `ai-sdk-provider-*` 로 확장 가능

**단점**:
- Node 단일 binary 어려움 (sea + 큰 binary)
- TypeScript 빌드 크기

### 3.3 Cross-runtime escape hatch — litellm proxy

**어느 스택이든** litellm HTTP proxy 를 fallback 으로:
```
my_harness (Rust or TS)
  → litellm proxy (Python subprocess 또는 별도 컨테이너)
  → OpenAI/Anthropic/Google/...
```

**장점**: provider 100+, 검증, 우리 lock-in 없음
**단점**: 추가 프로세스, Python 의존

## 4. 결정 매트릭스

| 결정 | Rust 1안 | TS 2안 | 비고 |
| --- | --- | --- | --- |
| 추천 라이브러리 | `rig-core` | `Vercel AI SDK (ai)` | 둘 다 활발, 검증 |
| provider 수 | 12+ | 15+ | 사실상 동등 |
| streaming | ✅ | ✅ | |
| tool calling | ✅ | ✅ | |
| prompt caching | Anthropic | Anthropic | |
| 로컬 모델 (Ollama) | ✅ | ✅ | 양쪽 다 |
| 빌드 복잡도 | 단일 binary | Node SEA | Rust 가 단일 binary 유리 |
| 단일 binary 크기 | 작음 (~30MB) | 큼 (~50MB+ with Node) | |
| MCP 통합 | `rmcp` SDK | `@modelcontextprotocol/sdk` | 양쪽 다 1급 |
| 우리 1차 분석 (opencode) 와 정합 | ✅ (opencode 가 TypeScript 임에도 OpenAI/Anthropic client lib 직접 사용) | ✅ | |
| 학습곡선 | 중간 (Rust async + trait) | 낮음 (TS/JS 친숙) | |

## 5. 권장 진행

**MVP v1 (Rust 1안)**: `rig-core` 직접 통합.
- 12+ provider 자동 지원
- 단일 binary 유지
- 도메인별 provider override (코드=Sonnet, 서버=Haiku, 환경=local Ollama) 지원

**Fallback**: litellm HTTP proxy 옵션 (v1.1+)
- 우리 1안이 rig-core 미지원 provider 필요 시
- 또는 v2 에서 web UI (aider/goose 처럼) 추가 시

## 6. 우리 my_harness 의 provider 추상화 인터페이스 (v1 설계)

```rust
// 우리 LLM provider 추상화 (rig-core 위에 얇은 wrapper)
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;  // "anthropic", "openai", "ollama", "google", ...
    fn default_model(&self) -> &str;
    fn supports_caching(&self) -> bool;  // Anthropic prompt cache
    fn supports_tools(&self) -> bool;
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, req: CompletionRequest) -> Result<CompletionStream>;
}

pub struct LlmRouter {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    domain_defaults: HashMap<Domain, String>,  // code → anthropic/sonnet, ...
    cost_tracker: CostTracker,
}

impl LlmRouter {
    pub async fn complete(&self, domain: Domain, req: CompletionRequest) -> Result<...> {
        // 1. 도메인 기본 provider 선택
        // 2. 실패 시 fallback (Google → OpenAI → Ollama 순)
        // 3. 비용 추적
    }
}
```

이 wrapper 가 rig-core 위에 위치 → 우리 도메인 로직 (코드/서버/환경) 은 provider 비종속.

## 7. 미해결 질문

- 우리 자체 ML 모델 (headroom 의 Kompress-base 처럼) 도입 여부? → 일단 v2+ 검토
- OpenRouter / Together 등 aggregator 직접 통합? → v1.1+ 검토
- LLM 응답 캐싱 (같은 prompt + cache hit) 지원? → v1 에서 SQLite + Anthropic prompt cache 통합 검토
- 우리 자체 LLM inference (fine-tuning 후 on-prem) 가능? → v3+ 검토

## 8. 다음 단계

- TASK-005 세부 분해 시: `provider 추상화` 를 첫 마일스톤으로 (스택 결정 후)
- v1 구현: rig-core (Rust) 또는 ai SDK (TS) 1개 provider 만 활성화
- v1.1: 다중 provider + 도메인별 override
- v2: aggregator (litellm/OpenRouter) + 로컬 모델 (Ollama)
