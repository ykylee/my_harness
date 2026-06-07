# Headroom 심층 분석 (chopratejas/headroom)

- **문서 목적**: `my_harness` CLI/TUI 의 컨텍스트/캐시/툴 결과 처리 레이어 설계에 직접 활용 가능한 인사이트 도출
- **범위**: `/Users/yklee/repos/harness-refs/headroom` 의 실제 코드 (Python + Rust). 14 섹션 표준 템플릿 + headroom 특화 부가 분석
- **대상 독자**: yklee, Mavis, TASK-005 디자인 리뷰 참여자, 이후 my_harness 컨텍스트 레이어 작업자
- **상태**: complete (1차)
- **최종 수정일**: 2026-06-07
- **관련 문서**: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [opencode.md](./opencode.md), [codex.md](./codex.md), [aider.md](./aider.md), [goose.md](./goose.md), [gemini-cli.md](./gemini-cli.md)

---

## 1. 개요 (Overview)

### 1.1 한 줄 요약

**Headroom** 은 AI 에이전트가 LLM 으로 보내는 모든 컨텍스트(툴 출력, 로그, RAG 청크, 파일, 히스토리)를 직전에 압축하는 "context compression layer" 다. **60–95 % 토큰 절감**, **로컬 실행**, **reversible (CCR)**, **3가지 통합 모드** (library · proxy · MCP) 가 핵심 슬로건이다.

### 1.2 누구를 위한 도구

- **AI 코딩 에이전트 사용자** — Claude Code · Codex · Cursor · Aider · GitHub Copilot CLI 사용자
- **LLM 애플리케이션 개발자** — `from headroom import compress` 로 인라인 압축
- **장기 실행 에이전트** — 대화/세션이 길어질수록 KV 캐시·컨텍스트 윈도우 한계에 부딪히는 워크로드
- **다중 에이전트 오케스트레이터** — SharedContext 로 에이전트 간 압축 컨텍스트 공유

### 1.3 라이선스

**Apache 2.0** ([LICENSE](https://github.com/chopratejas/headroom/blob/main/LICENSE), `README.md:19` 의 배지). 상용 서비스는 별도 ([ENTERPRISE.md](https://github.com/chopratejas/headroom/blob/main/ENTERPRISE.md) — `hello@headroomlabs.ai` 한 줄짜리).

### 1.4 코드 규모 (1차 측정치)

| 영역 | LOC | 측정 |
| --- | --- | --- |
| `headroom/*.py` (top-level) | **6,005** | `wc -l headroom/*.py` |
| `headroom/transforms/*.py` (transforms 만) | **12,354** | `wc -l headroom/transforms/*.py` |
| `crates/**/*.rs` (Rust core) | **65,942** | `find crates -name '*.rs' | xargs wc -l` |
| `tests/*.py` | (테스트 §11 참고) | 200+ test_*.py |
| **총 Python (headroom/ + tests/)** | 약 **30,000+ LOC** | 추정 |
| **총 Rust (crates/)** | **65,942 LOC** | (Rust 가 headroom-core, headroom-py, headroom-proxy 등 5+ crate) |
| 압축 알고리즘/transform 파일 수 | **22 transforms** (1 디렉토리) | `headroom/transforms/*.py` |
| CCR 컴포넌트 수 | 6 modules | `headroom/ccr/*.py` |

### 1.5 메인 바이너리 / 분배

| 패키지 | 채널 | 식별자 | 비고 |
| --- | --- | --- | --- |
| `headroom-ai` (Python) | PyPI | [`headroom-ai`](https://pypi.org/project/headroom-ai/) | `pip install "headroom-ai[all]"` |
| `headroom-ai` (TypeScript SDK) | npm | [`headroom-ai`](https://www.npmjs.com/package/headroom-ai) | `npm install headroom-ai` |
| `headroom-openclaw` (OpenClaw plugin) | npm + GH Packages | `headroom-openclaw` | OpenClaw 통합 |
| Docker | ghcr.io | `ghcr.io/chopratejas/headroom:latest` | `docker pull` |
| Kompress-base ML 모델 | HuggingFace | [`chopratejas/kompress-base`](https://huggingface.co/chopratejas/kompress-base) | 모델 자체 Apache 2.0 |

`pyproject.toml` 이 version canonical, npm/PyPI/crate 3곳 동기화 (PR.md 기준 canonical+commit-height 알고리즘).

### 1.6 핵심 디자인 슬로건 (README:11)

> **60–95 % fewer tokens · library · proxy · MCP · 6 algorithms · local-first · reversible**

---

## 2. 아키텍처 (Architecture)

### 2.1 프로세스 모델

Headroom 은 **하나의 Python 패키지 + Rust 확장 + 선택적 ML 모델** 의 트리플이다. 외부 의존은 `transformers` + `onnxruntime` (또는 `torch`) + `httpx` (proxy 시) + `mcp` (MCP server 시).

```
┌────────────────────────────────────────────────────────────────────┐
│  User application  (Claude Code, Codex, Cursor, Aider, your app)  │
└─────────────────────────────────┬──────────────────────────────────┘
                                  │  prompts · tool outputs · logs
                                  ▼
       ┌──────────────────────────────────────────────────────┐
       │   Headroom (runs locally — your data stays here)     │
       │   ────────────────────────────────────────────────   │
       │   A. compress() library call                          │
       │      └─→ TransformPipeline (CacheAligner → Router)   │
       │   B. Proxy server (FastAPI/uvicorn on :8787)         │
       │      └─→ ASGI handlers · per-provider routes         │
       │   C. MCP server (stdio transport)                     │
       │      └─→ headroom_compress/retrieve/stats tools      │
       │                                                      │
       │   Core algorithms:                                   │
       │     • CacheAligner (detector-only, I2 invariant)     │
       │     • ContentRouter → SmartCrusher / CodeCompressor  │
       │                    / Kompress / LogCompressor / ...  │
       │     • CCR (Compress-Cache-Retrieve) — reversible      │
       │                                                      │
       │   Substrate:                                          │
       │     • Rust core (crates/headroom-*) via PyO3         │
       │     • Kompress-base (ModernBERT dual-head) ONNX/MPS  │
       │     • TOIN learning (cross-user, telemetry opt-in)   │
       │     • Cross-agent memory (SharedContext)             │
       │     • headroom learn (failure mining → AGENTS.md)    │
       └──────────────────────────┬───────────────────────────┘
                                  │  compressed prompt
                                  ▼
                LLM provider  (Anthropic · OpenAI · Bedrock · …)
```

### 2.2 핵심 추상화 4개

| 추상화 | 정의 위치 | 역할 |
| --- | --- | --- |
| `Transform` (ABC) | `headroom/transforms/base.py:30` | 모든 압축 단위. `apply(messages, tokenizer) → TransformResult` |
| `PipelineStage` (Enum, 11개) | `headroom/pipeline.py:16` | 표준 라이프사이클 이벤트 |
| `PipelineEvent` (dataclass) | `headroom/pipeline.py:47` | 라이프사이클 페이로드 |
| `PipelineExtension` (Protocol) | `headroom/pipeline.py:67` | 확장이 구현할 단일 메서드 (`on_pipeline_event`) |

### 2.3 핵심 디렉토리 트리 (depth 3)

```
headroom/                                  # Python 패키지 (entry: headroom/__init__.py:312 LOC)
├── __init__.py         312  # compress()·client()·CompressConfig 공개 surface
├── compress.py         348  # from headroom import compress — one-function API
├── pipeline.py         178  # 11-stage lifecycle + PipelineExtensionManager
├── hooks.py            151  # CompressionHooks (pre/post_compress, compute_biases)
├── onnx_runtime.py      77  # ONNX helpers (mallloc_trim, session options)
├── client.py         1,043  # HeadroomClient — 통합 SDK
├── config.py           683  # HeadroomConfig + 모든 transform configs
├── ccr/                    # Compress-Cache-Retrieve (reversible)
│   ├── __init__.py    106  # 공개 surface
│   ├── tool_injection.py   # CCR_TOOL_NAME = "headroom_retrieve"
│   ├── response_handler.py # CCR tool call 자동 핸들링
│   ├── context_tracker.py  # 압축 컨텍스트 추적 + proactive expansion
│   ├── batch_processor.py  # batch API CCR 호출 처리
│   ├── batch_store.py      # BatchContextStore
│   └── mcp_server.py   939  # headroom MCP server (stdio)
├── transforms/                # 22 transforms, total 12,354 LOC
│   ├── base.py            78  # Transform ABC, split_frozen()
│   ├── pipeline.py       438  # TransformPipeline orchestrator
│   ├── cache_aligner.py  388  # **detector-only** (P2-23, I2 invariant)
│   ├── content_router.py 2,677 # ContentRouter — the brain
│   ├── smart_crusher.py   910  # Rust-backed via PyO3 (Stage 3c.1b)
│   ├── code_compressor.py 2,036 # tree-sitter, 8 langs, AST-preserving
│   ├── kompress_compressor.py 1,147 # ModernBERT dual-head, ONNX/MPS
│   ├── search_compressor.py   373 # grep/ripgrep 결과
│   ├── log_compressor.py      516 # build/test 출력
│   ├── diff_compressor.py     171 # git diff
│   ├── html_extractor.py      233 # HTML → text
│   ├── adaptive_sizer.py      308 # 컨텍스트 압박도 적응
│   ├── anchor_selector.py     770 # 핵심 부분 앵커링
│   ├── tag_protector.py       131 # XML 태그 보존
│   ├── read_lifecycle.py      490 # Read 툴 stale/superseded 탐지
│   ├── compression_policy.py  207 # 압축 정책 (auth_mode 등)
│   ├── content_detector.py    435 # magika + regex + Rust detect
│   ├── compression_summary.py 243 # 압축된 코드 요약
│   ├── compression_units.py   351 # 압축 단위 표준화
│   ├── error_detection.py     162 # 오류 패턴
│   ├── observability.py        77 # CompressionObserver interface
│   └── __init__.py            213
├── proxy/                  # ASGI proxy server (FastAPI/uvicorn)
│   ├── server.py  / handlers/  / interceptors/  / memory_*.py
│   ├── modes.py  / auth_mode.py  / cost.py
│   └── (40+ files)        # handlers, models, modes, interceptors
├── providers/              # provider-specific slices
│   ├── registry.py  / base.py  / anthropic.py  / openai.py  / google.py
│   ├── claude/  / codex/  / gemini/  / copilot/  / cursor/  / aider/  / openclaw/
│   └── litellm.py  / cohere.py
├── cli/                    # Click-based CLI
│   ├── main.py  / proxy.py  / wrap.py  / learn.py
│   ├── mcp.py  / perf.py  / init.py  / install.py
│   ├── memory.py  / tools.py  / evals.py
│   └── (15+ files)
├── models/                 # ML 모델 메타
│   ├── ml_models.py  / registry.py  / config.py
├── cache/                  # compression_store, prefix cache
├── install/                # `headroom wrap claude` 등 wrapper 설치
├── copilot_auth.py         632 LOC
├── copilot_macos_keychain.py 124
├── copilot_linux_secret.py    106
└── ... (60+ modules total)

crates/                                # Rust core (65,942 LOC total)
├── headroom-core/                     # the engine (magika, smart_crusher, ccr)
├── headroom-py/                       # PyO3 bindings (Python ←→ Rust)
├── headroom-proxy/                    # Rust proxy alternative
├── headroom-relevance/                # BM25 + embedding relevance scoring
├── headroom-parity/                   # Python ↔ Rust parity test runner
└── examples/                          # examples
```

### 2.4 데이터 흐름 (canonical request lifecycle)

`headroom/pipeline.py:32` 의 `CANONICAL_PIPELINE_STAGES` (튜플 11개):

```
SETUP → PRE_START → POST_START
  → INPUT_RECEIVED → INPUT_CACHED → INPUT_ROUTED → INPUT_COMPRESSED → INPUT_REMEMBERED
  → PRE_SEND → POST_SEND → RESPONSE_RECEIVED
```

기본 파이프라인은 **2-stage**: `CacheAligner` → `ContentRouter` (deprecated `IntelligentContextManager`/`RollingWindow` 는 **PR-B1** 에서 retired — `transforms/pipeline.py:40-43`).

핵심 메시지 변환은 모두 `Transform.apply()` 단일 인터페이스. 11개 lifecycle stage 는 proxy 와 SDK 양쪽에서 같은 event 를 emit — 확장은 한 곳에서 작성하면 양쪽에서 동작.

### 2.5 Rust ↔ Python 경계

`crates/headroom-py` (PyO3) 가 Python surface 노출. `headroom._core` 로 import:

- `SmartCrusher` (Python class 가 단순히 Rust 메서드 위임, `headroom/transforms/smart_crusher.py:165-167`)
- `detect_content_type` (magika→unidiff→PlainText chain, `content_router.py:124`)
- `ccr_get`, `ccr_len` (Rust CCR store 직접 접근)

`smart_crusher.py:1-9` 의 docstring 명시:

> The Python implementation has been retired (Stage 3c.1b, 2026-04-27). All array compression now goes through `headroom._core.SmartCrusher` (built from `crates/headroom-py`). Byte-equality of the two implementations was verified against 17 recorded fixtures.

→ **"Rust 가 hot path, Python 이 orchestration"** 패턴. ML 모델도 ONNX (light) → PyTorch (full) 양쪽 모두 lazy import + semaphore 로 동시성 제어 (`kompress_compressor.py:51-55`).

---

## 3. 진입점 & CLI

### 3.1 CLI entry

```python
# headroom/cli.py:1-6
"""Backwards compatibility - CLI moved to headroom.cli package."""
from headroom.cli import main
if __name__ == "__main__":
    main()
```

본체: `headroom/cli/main.py:21-32` — Click group `main`, `click.version_option(get_version(), "--version", "-v", prog_name="headroom")`. `-?` 도 alias 로 지원 (`_apply_help_aliases`, `main.py:60-69`).

### 3.2 명령 트리

Click group `main` 아래 10개 서브커맨드 (`cli/main.py:38-48` 의 `_register_commands`):

```
headroom
├── proxy              # `headroom proxy --port 8787` (uvicorn ASGI server)
├── wrap                # `headroom wrap claude|codex|cursor|aider|copilot`
├── memory              # `headroom memory list / stats`  (numpy/hnswlib 필요)
├── learn               # `headroom learn` — 실패 세션 마이닝 → CLAUDE.md/AGENTS.md
├── mcp                 # `headroom mcp serve / install` — MCP server
├── tools               # `headroom tools` — bundled tools 진단
├── perf                # `headroom perf` — 절감 통계
├── evals               # `headroom evals suite --tier 1` — 벤치마크
├── install             # `headroom install`
└── init                # `headroom init` (devcontainer 설정 등)
```

### 3.3 proxy 서브커맨드 — 옵션 트리 (발췌, `cli/proxy.py:41-200`)

`@main.command()` 아래 25+ `@click.option`:

| 옵션 | envvar | 기본 | 설명 |
| --- | --- | --- | --- |
| `--host` | `HEADROOM_HOST` | `127.0.0.1` | bind 호스트 |
| `--port / -p` | `HEADROOM_PORT` | `8787` | bind 포트 |
| `--workers` | `HEADROOM_WORKERS` | `1` | uvicorn workers |
| `--limit-concurrency` | `HEADROOM_LIMIT_CONCURRENCY` | `1000` | 동시 연결 한도 |
| `--max-connections` | `HEADROOM_MAX_CONNECTIONS` | `500` | 업스트림 HTTP max |
| `--max-keepalive` | `HEADROOM_MAX_KEEPALIVE` | `100` | keep-alive 한도 |
| `--mode` `[token\|cache]` | `HEADROOM_MODE` | `token` | 압축 vs 캐시 우선 모드 |
| `--intercept-tool-results` | – | off | ast-grep Read outliner (opt-in) |
| `--no-optimize` | – | off | 압축 비활성 (passthrough) |
| `--no-cache` | – | off | 시맨틱 캐시 비활성 |
| `--no-rate-limit` | – | off | rate limit 비활성 |
| `--proxy-extension` (multi) | `HEADROOM_PROXY_EXTENSIONS` | – | entry-point 별 proxy extension enable |
| `--no-subscription-tracking` | `HEADROOM_NO_SUBSCRIPTION_TRACKING` | off | Anthropic subscription poller 비활성 |
| `--subscription-poll-interval` | `HEADROOM_SUBSCRIPTION_POLL_INTERVAL` | `300` | Anthropic usage 폴링 간격 (1–3600s) |
| `--retry-max-attempts` | – | `3` | 업스트림 재시도 |
| `--connect-timeout-seconds` | – | `10` | 업스트림 connect timeout |
| `--anthropic-pre-upstream-concurrency` | `HEADROOM_ANTHROPIC_PRE_UPSTREAM_CONCURRENCY` | `max(2, min(8, cpu_count))` | **cold-start replay storm 방지**, 503 fail-fast |
| `--anthropic-pre-upstream-acquire-timeout-seconds` | `…_ACQUIRE_TIMEOUT_SECONDS` | `15.0` | semaphore 대기 타임아웃 |
| `--anthropic-pre-upstream-memory-context-timeout-seconds` | `…_MEMORY_CONTEXT_TIMEOUT_SECONDS` | … | memory context timeout |

→ **옵션이 envvar 까지 자동 매핑** (Click 의 `envvar=...` 기능). 모든 production-grade 설정이 environment override 가능. 우리 my_harness 도 채택 권장.

### 3.4 `headroom wrap` — agent wrapper

`headroom/cli/wrap.py` (별도 파일) 가 `headroom wrap claude|codex|cursor|aider|copilot` 명령 처리. 각 agent 별로:

1. `headroom proxy --port 8787` 시작
2. agent 의 환경변수 (ANTHROPIC_BASE_URL 등) 를 proxy 로 redirect
3. agent 프로세스 launch

`wrap` 시 `--memory`, `--code-graph` 등의 agent-specific 플래그 (README:136-141).

### 3.5 Library entry (가장 단순한 사용법)

```python
# headroom/compress.py:158-198
def compress(
    messages: list[dict[str, Any]],
    model: str = "claude-sonnet-4-5-20250929",
    model_limit: int = 200000,
    optimize: bool = True,
    hooks: Any = None,
    config: CompressConfig | None = None,
    **kwargs: Any,
) -> CompressResult:
```

`singleton pipeline` + `lazy` 패턴 (`compress.py:72-74`):

```python
_pipeline = None
_pipeline_lock = threading.Lock()

def _get_pipeline() -> Any:
    if _pipeline is not None:
        return _pipeline
    with _pipeline_lock:
        if _pipeline is not None:  # double-check
            return _pipeline
        from headroom.transforms import TransformPipeline
        _pipeline = TransformPipeline()
        return _pipeline
```

→ double-check locking + lazy import 로 cold-start 가볍게.

### 3.6 MCP entry

`headroom/ccr/mcp_server.py:69-72` 가 tool 4개 노출:

```python
CCR_TOOL_NAME = "headroom_retrieve"
COMPRESS_TOOL_NAME = "headroom_compress"
STATS_TOOL_NAME = "headroom_stats"
READ_TOOL_NAME = "headroom_read"   # opt-in via HEADROOM_MCP_READ=on
```

`headroom mcp serve` (stdio transport), `headroom mcp install` (Claude Code 에 자동 등록).

---

## 4. TUI/UI 구현

### 4.1 TUI 없음 (의도적)

**Headroom 에는 TUI 가 없다.** 이건 의도된 결정이다. 3가지 통합 모드 (library · proxy · MCP) 가 모두 non-interactive 백엔드 서비스이고, agent 사용자는 Claude Code / Cursor 같은 **외부 TUI/IDE** 안에서 headroom 을 사용한다.

대신 headroom 은 **observability dashboard** 를 `docs/` (Next.js, `docs/package.json:36` 확인) 로 별도 제공:

```
docs/
├── app/                    # Next.js App Router
├── components/             # UI components
├── content/                # MDX docs
├── observability.md        # 관측 가능성 가이드
├── package.json            # Next.js 의존성
├── next.config.mjs
└── postcss.config.mjs
```

**Dashboard 의 의존성** (`docs/package.json`):

| 의존성 | 용도 |
| --- | --- |
| `next` (15.x 추정) | App Router |
| `react`, `react-dom` | UI |
| `@radix-ui/*` | accessible primitives |
| `lucide-react` | icons (우리 프로젝트와 동일) |
| `recharts` | 그래프 |
| `framer-motion` | 애니메이션 |
| `tailwindcss` | 스타일 |
| `shiki` | 코드 하이라이팅 |

→ Next.js + Radix + Tailwind + Recharts + Lucide — **우리 Devhub_example stack 과 동일** (`docs/components` 디렉토리 자체가 우리 디자인 시스템과 호환).

### 4.2 터미널 UX (proxy 시작 시)

proxy 시작 시 stdout 에 banner + 진단 정보 출력 (`headroom/cli/proxy.py` 의 `@main.command()` 안에서 click.echo). 사용자가 직접 보는 UI 가 아니라 agent 의 stderr / log.

### 4.3 설치 시 출력

`headroom wrap claude` 실행 시:
1. proxy 가 spawn 되는 PID/port
2. `COPILOT_PROVIDER_API_URL=...` 등 환경변수 안내
3. agent 가 redirect 되어야 할 base URL 안내 (paste-once)

### 4.4 CLI 디자인 디테일

- **`-?` 를 `--help` 의 alias** 로 지원 (`_apply_help_aliases`, `cli/main.py:60-69`) — Windows 사용자를 위한 호환성
- 모든 옵션이 `envvar=` 매핑 — CI / Docker / systemd 에서 wrapping 불필요
- `--proxy-extension` 가 multi-value + comma-separated list (`multiple=True`) — 동적 활성화
- `metavar="[token|cache]"` 로 `--mode` 의 legacy alias (`token_mode`, `cache_mode`, `token_savings`, `cost_savings`, `token_headroom`) 를 숨김 — canonical 2개만 노출, 내부 caller 는 여전히 옛 이름 사용 가능 (`cli/proxy.py:88-114`)

### 4.5 비-TTY 환경 고려

proxy 는 `os.environ` 만 읽고 stdin 받지 않음. systemd / docker / k8s 에서 그대로 실행 가능. headroom mcp 는 stdio transport 만 사용 (HTTP transport 없음) — agent 가 subprocess 로 spawn.

### 4.6 Rust CLI / TUI 단서

`crates/headroom-proxy/src/handlers/*` 가 ASGI handler (FastAPI/axum 추측). **TUI 라이브러리 의존 없음** — terminal control 도 하지 않음 (banner 출력은 click.echo).


## §5 LLM 통합 (LLM Integration)

### 5.1 압축의 LLM 통합 관점

**Headroom 은 LLM 을 직접 호출하지 않음** — **incoming prompt 만 압축**하고 outgoing call 은 사용자의 LLM client 에 위임. 이게 가장 큰 설계 결정. 즉 **transparent middleware**:
- 사용자의 `anthropic.Anthropic()`, `openai.OpenAI()`, `litellm.completion()` 호출은 그대로
- 그 호출 직전에 `compress(messages)` 만 추가
- 모델/프로바이더 비종속 — 압축 후 어떤 LLM 으로든 전송 가능

### 5.2 provider 비종속 (Library mode)

```python
# Anthropic
from anthropic import Anthropic
from headroom import compress
client = Anthropic()
compressed = compress(messages, model="claude-sonnet-4-5-20250929")
response = client.messages.create(model="claude-sonnet-4-5-20250929", messages=compressed.messages)

# OpenAI
from openai import OpenAI
client = OpenAI()
compressed = compress(messages, model="gpt-4o")
response = client.chat.completions.create(model="gpt-4o", messages=compressed.messages)

# LiteLLM (50+ providers)
import litellm
compressed = compress(messages, model="bedrock/claude-sonnet")
response = litellm.completion(model="bedrock/claude-sonnet", messages=compressed.messages)
```

→ **우리 my_harness 의 rig-core (Rust) 또는 Vercel AI SDK (TS) 와 같은 차원** — 프로바이더 1곳에서 격리.

### 5.3 proxy mode 의 provider 비종속

proxy 가 OpenAI 호환 API 를 노출 (`/v1/chat/completions`). **모든 OpenAI 호환 client** 가 그대로 사용 가능:
- `openai.OpenAI(base_url="http://localhost:8787")`
- `litellm.completion(..., base_url="http://localhost:8787")`
- `httpx.post("http://localhost:8787/v1/chat/completions", ...)`

Anthropic SDK 사용 시 **Anthropic-format 변환 레이어** (`/v1/messages` 엔드포인트) — `headroom/proxy/server.py` 에서 처리.

### 5.4 MCP mode 의 provider 비종속

headroom 자체가 **MCP server 역할** (stdio transport). agent 가 `mcp__headroom__compress` 같은 도구로 호출. agent 자체는 어떤 provider 든 — headroom 은 tool 만 노출.

### 5.5 토큰화 (tokenization)

`headroom/tokenizer.py` — multi-model 토크나이저 통합:
- `tiktoken` (OpenAI 모델)
- `anthropic-tokenizer` 또는 자체 구현 (Anthropic 모델)
- `sentencepiece` (Google)
- 모델별 자동 감지 (model name → tokenizer)

`headroom/tokenizers/` — 다중 토크나이저 백엔드.

### 5.6 캐시 (cache) 와 KV cache hit

**CacheAligner** 가 prefix 를 안정화해서 provider KV cache hit 율을 높임. Anthropic prompt caching, OpenAI cached prompt 등 provider 별 캐싱 메커니즘 모두 활용. **`headroom/cache/`** 의 prefix_tracker, dynamic_detector, registry 등이 핵심.

### 5.7 비용 추적 (cost tracking)

`headroom/pricing/` — 모델별 token-to-cost 매핑. 압축 전/후 비용 차이 표시.

## §6 도구/스킬 시스템 (Tool/Skill System)

### 6.1 통합 모드 = 3가지 (Tool 관점)

| 모드 | 도구 사용 방식 | 사용자 코드 변경 |
| --- | --- | --- |
| **Library** | `from headroom import compress` 인라인 호출 | 1줄 추가 |
| **Proxy** | OpenAI 호환 client 의 `base_url` 만 교체 | 1줄 수정 |
| **MCP** | headroom MCP server 등록 (stdio), agent 가 tool 로 사용 | MCP config 1줄 |

### 6.2 Library mode API

```python
# headroom/compress.py
def compress(
    messages: list[dict],
    model: str,
    compress_user_messages: bool = False,
    target_ratio: float = 0.35,  # 목표 압축 비율
    protect_recent: int = 5,  # 최근 N 메시지 보호
    **kwargs
) -> CompressResult:
    """메시지 리스트 압축. CompressResult.messages / .tokens_saved / .compression_ratio."""
```

`CompressConfig` (dataclass) — 모든 옵션.

### 6.3 Proxy mode API

```bash
# headroom proxy --port 8787 --upstream https://api.anthropic.com
```

```python
# 사용자 코드 (zero code change)
client = Anthropic(base_url="http://localhost:8787")  # 이 한 줄만!
```

### 6.4 MCP mode API

```json
// .claude/mcp_servers.json 또는 동등 설정
{
  "mcpServers": {
    "headroom": {
      "command": "uvx",
      "args": ["headroom", "mcp"]
    }
  }
}
```

agent 가 `mcp__headroom__compress(messages, model=...)` 같은 tool 호출로 사용.

### 6.5 CCR (Context Cache Reduction) — reversible 의 핵심

`headroom/ccr/` — **reversible compression** + 로컬 storage + retrieval tool. 압축된 메시지는 **원문 손실 없이** 로컬에 저장. 모델이 원문 필요시 `headroom_retrieve` tool 로 fetch.

**워크플로우**:
```
[User input + huge tool output]
   ↓ compress()
[압축된 메시지 (메타데이터만)]
   ↓ LLM 호출
[LLM 이 원문 필요시 → headroom_retrieve(tool_id) 호출]
   ↓ agent 가 headroom MCP tool 실행
[원문 fetch → LLM 에 주입]
```

장점: **압축률 100% 도달** (원문 0 토큰). 단점: **LLM 이 retrieval 호출** 해야 함 — 1 round-trip 비용.

### 6.6 도구 정책 (allow/block)

- `--protect-recent N` — 최근 N 메시지 압축 안 함
- `--target-ratio 0.35` — 목표 65% 압축
- `compress_user_messages=True` — user 메시지도 압축 (off by default — user intent 보호)
- `mode=[token|cache]` — 압축 전략 (token mode vs cache mode)

## §7 컨텍스트 관리 (Context Management)

### 7.1 컨텍스트 압축 = headroom 의 핵심

3 계층:
1. **Token mode** — 단순 토큰 수 감소 (CCR 없이, 일방향)
2. **Cache mode** — KV cache hit 율 최적화 (prefix 안정화)
3. **CCR** (Context Cache Reduction) — reversible + retrieval

### 7.2 압축 파이프라인

`headroom/transforms/pipeline.py` + `headroom/pipeline.py`:
```
detect_content_type → select_compressor → apply_compression → emit
```

1. **detect_content_type** — JSON / AST / log / HTML / diff / search / shell output 등 자동 감지
2. **select_compressor** — ContentRouter 가 적절한 compressor 매핑
3. **apply_compression** — 선택된 compressor 로 실제 압축
4. **emit** — 압축된 메시지 + 메타데이터 (tokens_saved, 원문 위치 등)

### 7.3 detect_content_type 알고리즘

`headroom/transforms/content_detector.py` — 정규식 + 휴리스틱:
- `^[{[]` → JSON
- `^diff --git` → diff
- HTML 태그 → HTML
- tool name prefix → log / search result / shell output
- AST tree-sitter 매치 → code (Python / TS / Rust 등)

### 7.4 압축 알고리즘 (6 main algorithms)

| Algorithm | 대상 | 방법 | 비고 |
| --- | --- | --- | --- |
| **CacheAligner** | 모든 prompt | prefix 안정화 (whitespace normalization, ordering) | KV cache hit 율 ↑ |
| **ContentRouter** | meta | content type 감지 + compressor 선택 | 다른 알고리즘의 dispatcher |
| **CCR** (Context Cache Reduction) | 전체 | reversible 압축 + 로컬 storage + retrieve tool | 토큰 100% 절감 가능 |
| **SmartCrusher** | JSON | 구조 보존 압축 (key shorten, value dedup) | code 도구 출력 |
| **CodeCompressor** | code | AST-aware (tree-sitter) — 식별자 shorten, 주석 제거, import 그룹화 | Python/TS/Go/Rust/Java 등 |
| **Kompress-base** | 자유 텍스트 | 자체 학습 ML 모델 (ONNX, HuggingFace `chopratejas/kompress-base`) | 95% 압축, local-first |

### 7.5 transforms/ 의 20+ 모듈

`adaptive_sizer`, `anchor_selector`, `code_compressor`, `cache_aligner`, `diff_compressor`, `error_detection`, `html_extractor`, `log_compressor`, `read_lifecycle`, `search_compressor`, `smart_crusher`, `tag_protector`, `pipeline`, `observability`, `compression_policy`, `compression_summary`, `compression_units`, `content_detector`, `content_router`, `kompress_compressor` 등.

### 7.6 토큰 예산 (token budget)

`CompressConfig.target_ratio` (default 0.35) — **65% 압축** 목표. 모델/사용자 요구에 따라 조정.

### 7.7 압축 후 호출 흐름 (cache-friendly)

`cache_aligner.py` — **prefix 안정화**:
- 메시지 순서 표준화 (system → user → assistant → user → ...)
- whitespace 정규화
- tool_call ID 정렬
- 같은 prompt 의 두 호출이 **같은 prefix hash** 갖도록

→ provider KV cache hit 율 증가 (Anthropic cache_write/cache_read 비용 절감).

## §8 세션 영속화 (Session Persistence)

### 8.1 session 모듈

`headroom/parser.py` + `headroom/storage/` — 세션 메시지 파싱/저장.

### 8.2 CCR 의 storage

**CCR** 의 reversible storage 가 headroom 의 핵심 차별점:
- `~/.local/share/headroom/ccr/` (XDG 경로) 또는 사용자 지정
- 압축된 메시지의 **원문** + **메타데이터** (compression_id, original_tokens, compressed_tokens, timestamp)
- SQLite 또는 flat file (확인 필요)

### 8.3 Resume

CCR 사용 시 `headroom_retrieve(id)` 로 원문 복원. library mode 에서:
```python
compressed = compress(messages, model=...)
# ... LLM 호출 ...
# LLM 이 retrieval tool 호출시
original = retrieve(compressed_id)  # 로컬에서 fetch
```

### 8.4 SharedContext (cross-agent)

`headroom/shared_context.py` — 다중 agent 가 같은 압축 컨텍스트 공유. **multi-agent 시스템** 에서 중복 작업 방지.

## §9 확장 시스템 (Extension System)

### 9.1 plugin 시스템 부재 (의도적)

**Headroom 은 plugin 시스템 없음** — 압축 알고리즘 추가 시 **PR + 머지** 필요. aider/goose 와 같은 미니멀리즘. 우리 my_harness 와 같은 결정.

### 9.2 대신 3-모드 + CCR

확장 포인트:
- **CompressConfig** — 옵션 조정
- **mode** (token/cache/ccr) — 전략 선택
- **transforms/** — 새 알고리즘 PR

### 9.3 20+ transforms 의 발견 방법

`headroom/transforms/` 의 모든 .py 자동 로드. **`PipelineStage` enum + lazy import** 패턴 추정.

### 9.4 MCP 통합 (자신의 server)

`headroom/mcp_registry/` (추정) — headroom 의 MCP server 등록. agent 가 `mcp__headroom__compress` 같은 tool 호출.

## §10 빌드 & 배포 (Build & Distribution)

### 10.1 빌드 시스템

- **Python**: `pyproject.toml` + setuptools (또는 hatchling)
- **Rust**: `crates/*` Cargo workspace
- **TypeScript**: 일부 (npm package) — `headroom-ai` npm registry

### 10.2 3개 deployment 형식

| 형식 | 진입점 | 사용 |
| --- | --- | --- |
| **CLI** | `headroom` (Python) | compression / cache / proxy / mcp / learn 서브커맨드 |
| **Proxy daemon** | `headroom proxy` (Rust binary, ASGI server) | long-running HTTP daemon |
| **MCP server** | `headroom mcp` (stdio) | agent 와 pair |
| **Library** | `from headroom import compress` (Python) / `import { compress } from 'headroom-ai'` (TS) | inline 호출 |

### 10.3 Distribution

- **PyPI**: `pip install headroom-ai`
- **npm**: `npm install headroom-ai`
- **HuggingFace**: `chopratejas/kompress-base` (ML 모델)
- **GitHub Releases**: Rust binary

### 10.4 Cross-platform

- Python: 어디서나 (3.10+)
- Rust proxy: Linux / macOS / Windows native binary
- TypeScript: Node 18+

## §11 테스트 & 품질 (Testing & Quality)

### 11.1 테스트 구조

`tests/` — pytest 기반:
- `test_proxy/test_openai_backend_path.py`
- `test_proxy/test_transformations_feed.py`
- `test_proxy/test_mcp_stats_aggregation.py`
- `test_cache/test_prefix_tracker.py`
- `test_cache/test_dynamic_detector.py`
- `test_cache/test_anthropic.py` / `test_openai.py` / `test_semantic.py`
- `test_strands_tokenizer.py` (tokenizer integration)
- `test_provider_codex_install.py`
- `test_sse_thinking_blocks.py` (SSE stream)

### 11.2 테스트 패턴

- **Unit test**: per-module
- **Integration test**: proxy + cache end-to-end
- **Mock**: LLM mocking (`headroom/mocks/`)
- **Provider-specific tests**: anthropic / openai / codex / strands

### 11.3 CI

GitHub Actions (추정) — multi-OS, multi-Python, Rust.

## §12 보안 (Security)

### 12.1 데이터 privacy

**로컬 실행** (`local-first`) — 모든 압축이 사용자 머신에서. **원격 LLM 호출 없음** (headroom 자체는). 압축된 메시지만 LLM 으로 전송. **원문** 은 CCR mode 가 아니면 LLM 에 안 감.

### 12.2 API key 관리

`headroom/providers/` — provider credentials 관리. 환경변수 패턴 (Anthropic API key 등). **OS keychain 통합 추정** (macOS Keychain, Windows Credential Manager).

### 12.3 OAuth

`headroom/copilot_auth.py`, `headroom/copilot_macos_keychain.py`, `headroom/copilot_linux_secret.py` — **GitHub Copilot OAuth + 플랫폼별 secret 저장** (macOS Keychain / Linux Secret Service). 우리 1안 (Rust) 와 동일 패턴.

### 12.4 MCP 보안

`headroom/mcp_registry/` — MCP server 의 allowlist. 사용자 명시. **unknown MCP server 자동 enable 안 함** (추정).

### 12.5 Sandbox

`headroom` 자체는 압축만 하므로 **sandbox 불필요** (코드 실행 X). 단, **`headroom learn`** 기능 (확인 필요) — `cross-agent memory` 가 사용자 데이터 모을 가능성. `ENTERPRISE.md` 의 데이터 처리 정책 참조.

## §13 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 3-모드 통합 (Library · Proxy · MCP)

`from headroom import compress` (1줄 추가) / `headroom proxy` (zero code change) / `headroom mcp` (MCP server). **사용자 진입점 다양화** — 우리 my_harness 도 1안 = CLI 직접 통합, 2안 = Proxy (zero change), 3안 = MCP server. **사용자 친화도 극대화**.

#### 13.2 Provider 비종속 (50+ model 압축 호환)

`compress(messages, model="claude-sonnet-4-5")` 가 50+ provider 와 호환. **모델 ID 만 바꾸면 모든 provider 작동**. 우리 my_harness 의 rig-core (Rust) 또는 Vercel AI SDK (TS) 와 같은 차원 — **provider 1곳에서 격리**.

#### 13.3 CCR (Context Cache Reduction) — reversible + retrieval ⭐

**압축률 100% 도달 가능** + LLM 이 원문 필요시 retrieval tool 로 fetch. **1-round-trip 비용** vs **압축 한계 제거**. 우리 my_harness 의 토큰 한계 문제 (오늘 밤 4개 worker errored) 의 **근본적 해결책**. MVP v1 부터 도입 검토.

#### 13.4 6 algorithms 의 dispatcher 패턴 (ContentRouter)

`ContentRouter` 가 content type 감지 → 적절한 compressor 선택. 우리 my_harness 도 **3-도메인 (코드/서버/환경) + 4-타입 (코드/로그/diff/자유 텍스트) 의 router** 도입.

#### 13.5 CacheAligner (KV cache 친화)

`cache_aligner.py` — prefix 안정화로 provider KV cache hit ↑. **Anthropic prompt cache / OpenAI cached prompt / Google implicit cache** 모두 활용. 우리 my_harness 의 토큰 비용 50%↓ 잠재력.

#### 13.6 학습된 ML 모델 (Kompress-base, 95% 압축)

자체 학습 (HuggingFace 공개). **자유 텍스트** 95% 압축. **ONNX runtime** 으로 local inference (no API call). 우리 my_harness 가 자체 ML 모델 가질 필요는 없지만, **외부 모델 의존 시 ONNX** 가 portable.

#### 13.7 20+ transforms 의 lazy registry 패턴

`headroom/transforms/` 의 모든 .py 자동 발견. 우리 my_harness 의 `~/.myharness/tool/` 또는 `skill/` 도 동일.

#### 13.8 Local-first (no API call for compression)

`local-first` — 모든 압축이 로컬. **프라이버시 + 비용 0** (압축 자체). 우리 my_harness 의 **tool call 결과 압축** 도 동일 정책.

#### 13.9 OpenAI 호환 Proxy endpoint

proxy 가 `/v1/chat/completions` 노출 → **모든 OpenAI 호환 client** 즉시 사용. **zero code change**. 우리 my_harness 도 같은 패턴 가능 (proxy 모드 시).

#### 13.10 CLI 옵션의 envvar 매핑

모든 옵션이 `envvar=` 매핑 → CI / Docker / systemd wrapping 불필요. 우리 my_harness 도 동일 (MiniMax.md 의 운영 정책 자동화).

#### 13.11 Proxy 의 stdio / non-TTY 호환

proxy 가 stdin 안 받음 → systemd / docker / k8s 그대로. 우리 my_harness 도 daemon mode 동일.

#### 13.12 metavar 로 legacy alias 숨김

`metavar="[token|cache]"` — canonical 2개만 노출, 내부 caller 는 옛 이름 사용 가능. **하위 호환성** 의 elegant 한 패턴. 우리 my_harness 도 `--mode` 옵션 동일 적용.

#### 13.13 설치 시 paste-once 안내

`COPILOT_PROVIDER_API_URL=...` 출력. **사용자 1회 paste** 끝. 우리 my_harness 의 install UX 도 동일 — 한 번에 끝.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.14 자체 ML 모델 유지보수 부담

`Kompress-base` 가 HF 공개지만, **모델 업데이트 / 호환성 유지** 부담. 우리 my_harness 는 ML 모델 자체 호스팅 ❌. **외부 모델 (litellm + litellm-proxy-ml) 활용** 검토.

#### 13.15 Rust + Python 듀얼 언어

`crates/` (Rust proxy) + `headroom/` (Python core) + `headroom-ai` (TS). **3 언어** = 빌드 시스템 / 의존성 관리 부담. 우리 my_harness 는 **단일 언어** (Rust 1안 OR TS 2안, 절대 둘 다 X).

#### 13.16 compress() 의 async 미지원 추정

`compress(messages, model=...)` 가 sync 호출 (추정). **큰 컨텍스트** 시 blocking 가능. 우리 my_harness 는 async API 우선.

#### 13.17 CCR 의 round-trip 비용

`headroom_retrieve` 호출이 **1 round-trip**. **TTFT (Time To First Token) 증가**. 우리 my_harness 가 CCR 도입 시 **non-blocking** 또는 **streaming** 으로 retrieve 결과 처리.

#### 13.18 단일 압축 (multi-pass 없음)

`compress()` 가 **1-pass**. **재귀 압축** (compressed → compress → compress) 미지원. 우리 my_harness 도 1-pass 충분 — multi-pass 는 v2+ 검토.

#### 13.19 cross-encoder re-ranking 부재

`headroom` 자체는 **정보 검색 (IR) 의 re-ranking** 안 함. 단순 token-level 압축. **semantic 검색** 필요 시 별도 시스템 (예: `headroom learn`). 우리 my_harness v1 은 단순 압축, v2+ semantic 검색.

#### 13.20 OpenAI 호환 API 가 Anthropic 메시지 변환 시 손실

proxy 의 `/v1/messages` 가 OpenAI 포맷으로 변환할 때 **Anthropic 특화 기능** (thinking blocks, prompt caching metadata) 손실 추정. 우리 my_harness 도 provider 비종속 추상화 시 **공통 분모** 만 지원, provider-specific 기능은 별도 채널.

## §14 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 압축 정확도 vs token 절감 비율의 Pareto 곡선

`target_ratio=0.35` 일 때 정확도 손실 %? Kompress-base 의 벤치마크 데이터. **v1 도입 시 우리 자체 벤치마크** 필요.

### 14.2 우리 my_harness 의 도메인 (코드/서버/환경) 별 최적 compressor

3-도메인 각각에 어떤 compressor 가 최적인지? Code → CodeCompressor, Server → log_compressor + error_detection, 환경 → shell output. 우리 자체 평가.

### 14.3 우리 my_harness 의 worker 세션 long Write 문제의 CCR 적용

오늘 밤 4 worker 가 long Write 중 abort. **CCR 도입 시 해결되는지**, 아니면 **단순 token mode** (CCR 없이) 만으로 충분한지. A/B 테스트 필요.

### 14.4 Kompress-base 의 ONNX 모델 크기 / 로딩 시간

HuggingFace `chopratejas/kompress-base` 가 **수 MB ~ 수십 MB** 추정. 로딩 시간 첫 호출 시 부담. **lazy load** + cache 전략.

### 14.5 `headroom learn` 의 작동 방식

`ENTERPRISE.md` 와 `learn/` 디렉토리. **cross-agent memory** 가 어떻게 작동하는지 — `learn` 서브커맨드 + `~/.config/headroom/learn/` ? 우리 my_harness 의 `state.json` 와 정합 가능?

### 14.6 Rust crates/ 의 정확한 역할

`crates/` 가 65,942 LOC 인데, 이게 `headroom proxy` 외 다른 도구? `headroom-ccr-storage`? `headroom-mcp-server`? 우리 my_harness 가 Rust 도구 의존 시 `cargo` 통합.

### 14.7 `headroom wrap claude` 외 다른 에이전트 wrap 지원

`headroom wrap claude` 만 봤는데, Codex / Gemini CLI / Aider / Cursor 등 다른 에이전트 wrap 도 지원? 우리 my_harness 의 **mavis / mavis-code 와 동급** 인가.

### 14.8 `tests/test_cache/test_anthropic.py` 등 provider-specific 테스트

`headroom` 가 provider 별 cache API 를 정확히 어떻게 통합? **Anthropic cache_control** / **OpenAI cache_control** / **Google implicit cache**. 우리 my_harness 가 rig-core 위에 우리 cache layer 추가 시 참고.

### 14.9 압축의 latency 영향

`compress(messages, model=...)` 가 **얼마나 빠른지**? 100K token 메시지 압축 시 ms 단위? 우리 my_harness 의 streaming 응답 latency 영향.

### 14.10 로컬 Ollama + headroom 통합 가능성

`compress(messages, model="ollama/llama3")` 가 작동하는지? 우리 my_harness 의 "환경 셋업" 도메인에서 로컬 모델 사용 시.

### 14.11 ONNX vs TensorFlow Lite vs 다른 런타임

`headroom` 가 ONNX 선택한 이유? 우리 v2+ 자체 ML 모델 도입 시 비교.

### 14.12 CCR storage 의 실제 백엔드 (SQLite?)

`headroom/storage/` — SQLite 인지, file 인지, hybrid 인지. 우리 my_harness 의 session state (`state.json` + journal) 와 통합 가능?

### 14.13 headroom 의 license + 상용 정책

Apache 2.0. ENTERPRISE.md 의 상용 정책. 우리 my_harness 가 headroom 통합 시 **binary distribution** 영향.

### 14.14 `headroom` 의 첫 release (v0.1.0) 와 현재 버전

GitHub Releases + PyPI 버전. 우리 my_harness 가 **안정 버전** pin 할지 **최신 main** 추적할지.
