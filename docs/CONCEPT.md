# my_harness — 개발 컨셉 (Master Concept / SSOT)

> **본 문서는 my_harness 의 단일 진실 공급원 (single source of truth).**
>
> **D-140 (2026-08-14)**: 제품 = **Owned Surface + Headless Engine**. 화면은 `surface/` (myharness). 엔진은 숨긴 `grok` (`-p` / `agent stdio`). 기본 경로에서 grok TUI 를 열지 않는다.
>
> **구현 사양**: [`architecture/DETAILED_DESIGN_SURFACE.md`](./architecture/DETAILED_DESIGN_SURFACE.md) (SSOT). overlay 엔진 계약은 [`architecture/DETAILED_DESIGN_OVERLAY.md`](./architecture/DETAILED_DESIGN_OVERLAY.md).
>
> **갱신 정책**: 마일스톤 별 갱신. 본 문서 갱신 시 관련 (MiniMax.md, PROJECT_PROFILE.md, REFERENCES.md, README.md, development_log.md) 도 함께 align.
>
> **최종 갱신**: 2026-08-14 (D-140 owned surface). 이전: D-135 overlay 래퍼.

---

## 0. 핵심 Positioning (D-140 owned surface, D-135 엔진 유지)

### my_harness 는

- **화면의 주인** — `surface/` TUI/CLI. 워드마크·타이틀은 `myharness`. 기본 경로에서 grok pager 를 열지 않는다
- **숨긴 엔진** — 설치된 `grok` ≥ 1.0.3. 한 턴은 `grok -p`, 대화 TUI 는 `grok agent stdio` (ACP). `--plugin-dir` 은 agent 전용
- **3-도메인** — `myharness code|server|env …` 와 같은 브랜드의 TUI
- **plugin** — `plugins/myharness/` 를 `grok plugin install --trust` 로 싣는다 (자체 loader 없음)
- **MiniMax 우선** — grok `[model.minimax]`. 칩에는 사용자 alias 만 (`MiniMax-M3`)
- **개발 시 Mavis zero coupling** (D-25 유지)

### my_harness 는 **아니다** (NOT)

- ❌ **standalone 5-component 런타임** — Tools/Session/Plugin loader 를 다시 짜지 않음
- ❌ **grok 소스 포크** (D-134)
- ❌ **기본 경로의 grok TUI** — `myharness engine` 만 벤더 화면 (D-135.8 은 이 경로의 사실)
- ❌ **goose 포크** — 독립 런타임이 필요할 때만
- ❌ **Mavis 결합 런타임 / 4-워커 오케스트레이터**
- ❌ **v0 `myharness/` crates 에 신규 기능** (D-135.6)

### 위치 (Positioning)

```
┌─────────────────────────────────────────┐
│  yklee (user)                            │
└─────────────────────────────────────────┘
              ↓ terminal
┌─────────────────────────────────────────┐
│  myharness (surface/)                   │  ← D-140
│  - 픽셀 / 12 동사 / task / 브랜드        │
│  - ACP 클라이언트 (S4b+)                 │
└─────────────────────────────────────────┘
         ↓ grok -p          ↓ grok agent stdio
         (한 턴 CLI)         (제품 TUI, fd pipe)
┌─────────────────────────────────────────┐
│  grok (Grok Build ≥ 1.0.3)              │  ← 엔진, 화면 아님
│  Tools · Context · Session · Plugins    │
└─────────────────────────────────────────┘
              ↓ [model.minimax]
```

**my_harness** = 3-도메인 하네스. 화면은 우리 것. 엔진은 grok. 설계: [DETAILED_DESIGN_SURFACE.md](./architecture/DETAILED_DESIGN_SURFACE.md).

---

---

## 1. 한 줄 Positioning

**my_harness = yklee 의 3-도메인 하네스** — 화면은 `surface/` (myharness). 엔진은 숨긴 `grok`. MiniMax. 개발 workflow 만 Mavis / standard_ai_workflow.

---

## 2. 타겟 사용자

| 대상 | 사용 패턴 |
| --- | --- |
| **yklee (오너, single user)** | terminal 에서 `myharness <command>` 직접 호출. 3-도메인 (코드/서버/환경) 작업 |
| **plugin 개발자 (v2+)** | `~/.myharness/plugins/<name>/` 에 plugin 직접 작성. marketplace 공유 |

**v1 scope = yklee single user, terminal 직접 실행** (multi-user/marketplace 는 v2+)

---

## 3. 핵심 가치 (3가지)

### 3.1 **3-도메인 표면 + 숨긴 엔진** (D-140)
차별점은 자체 5 components 재구현이 아니다. **화면을 우리가 그리는 것**이다. grok 는 Tools/Session/Plugins 를 뒤에서 돌린다. 사용자는 GROK 워드마크가 아니라 myharness 크롬을 본다.

### 3.2 **Provider 는 grok `[model.*]`** (D-135.4)
MiniMax / Ollama / OpenAI 호환은 `api_backend = chat_completions` + `base_url`. 자체 `rig-core` 클라이언트는 v0 참고 구현 (D-135.6). 외부 orchestrator 는 여전히 불필요 — 경유가 grok sampler 일 뿐.

### 3.3 **3-도메인 동시 + 2-계층 Context 압축** (D-27 + D-30)

코드/서버/환경 3-도메인 모두 통합. **Context 압축은 2 계층**:
- **Layer 1 (필수, D-30)** — model length 한계 대응. **always-on 자동 압축**: token budget 추적 → 한계 근접 시 auto truncate/summarize → /compact (manual). opt-out 불가 (model 자체가 길이 제한 있으므로).
- **Layer 2 (선택, D-27)** — 비용 최적화. **opt-in advanced 압축**: headroom 의 6 알고리즘 (CacheAligner, ContentRouter, CCR, SmartCrusher, CodeCompressor, Kompress-base) 을 우리 Context component 에 built-in. `~/.myharness/config.toml` 에서 `builtin.enabled = true|false`. 기본 `false`. [D-42]

---

## 4. 스코프

### 4.1 In-scope (overlay 1차, D-135)

**3-도메인 표면** (그대로):
- **코드 개발 전반** — 새 기능 구현, 리팩토링, 버그 수정, 리뷰, 테스트, PR 작업
- **기본 서버 관리** — 프로세스/서비스 상태 점검, 로그 확인, 설정 변경, 배포 헬퍼
- **환경 셋업** — 로컬/원격 개발 환경 부트스트랩, 의존성 설치, 셸/도구 설정

**엔진/확장**:
- 설치된 `grok` ≥ 1.0.3 가드
- `plugins/myharness/` (`plugin.json` + skills/commands/agents/hooks)
- MiniMax `[model.minimax]` + env `MINIMAX_API_KEY`
- `myharness task start|end` (D-26, grok 바깥)

플랫폼은 grok 가 지원하는 범위 (공식 바이너리: macOS / Linux. Windows best-effort).

### 4.2 Out-of-scope

- **엔진 5 components 재구현** (tools/session/plugin loader/subagent SDK)
- **grok 소스 포크** / generated workspace 소유
- **v0 `myharness/` crates 에 신규 기능** (D-135.6)
- **표면 TUI + ACP 클라이언트는 in-scope** (`surface/`, D-140). 엔진 pager 를 스킨하는 것은 아님
- grok 로고/키맵/빌트인 slash 를 overlay 로 지우기 — 그 화면을 기본으로 안 연다
- Computer Use / Multi-user / RBAC
- 5 surface 동시 유지 (anti-pattern) — 절대 안 함

---

## 5. overlay 스펙 (D-135 엔진 · D-140 표면)

> 아래 §5.2~§5.14 의 명령·6원칙·디렉터리 **의도**는 유지한다. **화면 구현 주체**는 `surface/` (D-140). **엔진**은 grok. 래퍼 bash 는 S8 전까지 설치 기본. 모듈: [DETAILED_DESIGN_SURFACE.md](./architecture/DETAILED_DESIGN_SURFACE.md).

### 5.1 아키텍처: 화면 = surface, 엔진 = grok ⭐

```
┌─────────────────────────────────────────────┐
│  myharness (surface/)                       │  ← 픽셀 / 12 동사 / task / ACP
├─────────────────────────────────────────────┤
│  plugins/myharness/                         │  ← plugin.json 4계층
│  commands/ · skills/ · agents/ · hooks/     │
├─────────────────────────────────────────────┤
│  grok (installed binary ≥ 1.0.3)            │  ← 엔진, 재구현 금지
│  ┌───────────────────────────────────────┐  │
│  │ 1. Tools     — GrokBuild + Hashline   │  │
│  │ 2. Context   — AGENTS.md + compact    │  │
│  │ 3. Session   — updates.jsonl + FTS    │  │
│  │ 4. Plugins   — --plugin-dir + market  │  │
│  │ 5. Sub-agents — task, depth 1         │  │
│  └───────────────────────────────────────┘  │
├─────────────────────────────────────────────┤
│  grok sampler — 3 backend stream + retry    │
└─────────────────────────────────────────────┘
                    ↓
        [model.minimax] chat_completions
        (Ollama / OpenAI 호환 동일)
```

**참조**: [grok-build.md](./references/grok-build.md) §2·§9·§15, [DETAILED_DESIGN_OVERLAY.md](./architecture/DETAILED_DESIGN_OVERLAY.md)

### 5.2 명령 가이드 (도메인별)

**v1 = 도메인별 3-4 명령** (claude-code 13.30 anti-pattern 회피: 100+ slash commands 안 함).

#### 코드 도메인 (claude-code 13.1 + 13.22)
```bash
myharness code review <pr-url>          # multi-agent code review
myharness code implement "<feature>"    # sub-agent 구현 위임
myharness code test <path>              # test 실행 + 결과 분석
myharness code commit "<message>"       # git workflow
```

#### 서버 도메인 (goose 13.1)
```bash
myharness server status [host]          # 프로세스/서비스 상태
myharness server logs <service> [N]     # 최근 N줄 로그
myharness server deploy <env>           # 배포 헬퍼
myharness server config <action>        # 설정 조회/변경
```

#### 환경 도메인 (opencode 13.1)
```bash
myharness env setup <stack>             # 스택별 부트스트랩
myharness env install <pkgs>            # 의존성 설치
myharness env shell <cmd>               # 셸 명령 + LLM 분석
myharness env diagnose                 # 환경 진단
```

**각 명령 = 래퍼가 grok `-p` / `--agent` / plugin agent 로 번역**. 빌트인 grok 에이전트 (`explore` / `plan` / `general-purpose`) 이름은 섀도잉하지 않음.

### 5.3 설치 / 배포 (D-135.1)

**2층**:

1. **엔진** — 공식 `curl -fsSL https://x.ai/cli/install.sh | bash` → `~/.grok/bin/grok`. 업데이트는 `grok update`.
2. **래퍼** — `scripts/install.sh` → `~/.local/bin/myharness` + `~/.myharness/plugins/myharness/` (D-138). 개발 중에는 `./bin/myharness`. Rust clap 이전은 M3.2 보류.

래퍼는 `grok` 가 PATH 에 있고 버전이 `≥ 1.0.3` 인지 확인한다. 없으면 exit 2 + 설치 URL.

자체 5 install paths / cargo-dist 제품 배포는 **OOS**.

### 5.4 보안 (claude-code 13.8 + 13.4 + 13.13)

**4 permission mode** (claude-code 패턴):
- `default` — 매번 승인
- `acceptEdits` — edit 자동 승인
- `plan` — plan 만 표시, 실행 시 승인
- `bypassPermissions` — 모든 권한 우회 (sandbox 환경)

**Hook system** (claude-code 13.4 hookify 차용):
```
~/.myharness/hooks/
├── warn-rm-rf.md            # "rm -rf" 감지 시 경고
├── require-test-before-commit.md
└── security-pattern.md      # 9 security patterns
```

**markdown 1 file = 1 hook**, restart-free 적용.

**Secret management** (D-06):
- macOS Keychain (Apple Security.framework)
- Windows Credential Manager (wincred)
- Linux Secret Service (libsecret)
- 토큰 값은 메모리/문서/git 저장 금지

### 5.5 LLM 통합 (D-15 + D-28 + D-38 · D-135.4 경유)

**D-135.4**: 제품 경로의 LLM 호출은 grok sampler. MiniMax / Ollama 는 `[model.<name>]` + `api_backend = chat_completions`. 아래 5.5.1~5.5.3 은 v0 자체 클라이언트의 역사적 spec 이며, overlay 1차에서 재구현하지 않는다. `provider-auto-config` 는 grok skill 로 이식 가능 (PR-4).

#### 5.5.1 지원 Provider (D-28, 정적 등록)

| # | Provider | Native SDK | OpenAI 호환 | 비고 |
| - | --- | --- | --- | --- |
| 1 | **claude** (Anthropic) | ✅ `anthropic` SDK | (OpenRouter 경유) | Sonnet 4.5 / Haiku 4 / Opus 4.5 |
| 2 | **codex** (OpenAI) | ✅ `openai` SDK | — | GPT-5 / GPT-5-Codex / GPT-4.1 |
| 3 | **gemini** (Google) | ✅ `google-genai` SDK | (OpenAI 호환 endpoint) | Gemini 2.5 Pro / Flash |
| 4 | **deepseek** | — | ✅ (`https://api.deepseek.com/v1`) | deepseek-chat / deepseek-reasoner |
| 5 | **minimax** | — | ✅ (base_url TBD 검증 필요, D-28) | 모델명/API 형식 검증 |
| 6 | **local LLM** | — | ✅ (`http://localhost:11434/v1` 등) | Ollama / vLLM / LM Studio / llama.cpp |

**추상화 전략**:
- **Premium** (claude/codex/gemini) → **native SDK** (각 vendor 최적 기능: prompt cache, thinking, function calling)
- **OpenAI 호환** (deepseek/minimax/local) → **공통 OpenAI 호환 client** (1개 구현으로 N개)
- **Provider registry** (config.toml) → 사용자 정의 provider 추가 가능 (v1.5+ plugin)

#### 5.5.2 동적 발견 + Per-Provider Auth (D-38, NEW)

**하드코딩 fallback list 폐기**. yklee 환경 / 조직 / 시점에 따라 provider 가용성이 다르므로 **런타임 discovered list** 로 fallback 구성.

**`provider-auto-config` skill** (D-38, v1.5+ 권장, v1 부터 simple 버전):
- **위치**: `~/.myharness/skills/provider-auto-config/SKILL.md`
- **Auto-invoke trigger**: startup / `myharness auth` 명령 / fallback 실패 시
- **동작**:
  1. **Discover** — env vars (`ANTHROPIC_API_KEY` 등) + OS keychain + local LLM server (Ollama :11434 등) + MCP configured providers
  2. **Auth status** — 각 provider 의 auth state 확인
  3. **Build runtime list** — available providers 의 우선순위 자동 구성
  4. **Persist** — runtime list 를 `~/.myharness/state/active-providers.toml` 에 저장
  5. **Fallback chain** — discovered list + 도메인별 override 적용

**CLI 인터페이스 (per-provider auth)**:
```bash
myharness auth list                                    # 모든 provider status
myharness auth <provider>                              # 한 provider status
myharness auth <provider> login                        # OAuth/API key 초기화
myharness auth <provider> logout                       # auth 제거
myharness auth <provider> set-key <key>                # API key 수동 설정
myharness auth <provider> set-key --from-keychain      # keychain 에서 가져오기
myharness auth <provider> test                         # 연결 테스트 (ping model)

myharness auth setup                                    # 모든 provider 일괄 discover + login wizard
myharness auth default <provider>                       # primary 변경
```

**Auth state 저장** (`~/.myharness/state/auth/`):
```
~/.myharness/state/auth/
├── anthropic.toml          # status, last_login, default_model, supports
├── openai.toml
├── gemini.toml
├── deepseek.toml
├── ollama.toml             # local server status (Ollama 실행 중 여부 + models)
└── active-providers.toml   # 현재 discovered list (fallback chain source)
```

**Provider status 예시** (anthropic.toml):
```yaml
provider: anthropic
type: native
sdk: anthropic
status: authenticated        # authenticated | logged_out | error | not_configured
last_login: 2026-06-07T13:00:00+09:00
default_model: claude-sonnet-4-5
available_models: [claude-sonnet-4-5, claude-haiku-4, claude-opus-4-5]
supports: [prompt_cache, thinking, vision, tool_use]
secret_store: keychain
api_key_env: ANTHROPIC_API_KEY
test:
  last_test: 2026-06-07T13:05:00+09:00
  result: ok
  latency_ms: 320
```

**Ollama local 예시** (ollama.toml):
```yaml
provider: ollama
type: openai-compatible
base_url: http://localhost:11434/v1
status: available            # server 실행 중 + models 발견
server: ollama
server_version: 0.5.7
discovered_models: [qwen2.5-coder:32b, llama3:70b, codellama:34b]
default_model: qwen2.5-coder:32b
test:
  last_test: 2026-06-07T13:05:00+09:00
  result: ok
  latency_ms: 80
```

#### 5.5.3 Fallback Chain 동적 구성 (D-38)

**하드코딩 `fallback: [A, B]` → 동적 discovered list**:
```toml
# ~/.myharness/config.toml (D-38 갱신)
[llm]
primary = "<primary-model>"            # primary 는 config (도메인 무관 기본)
fallback_strategy = "discovered"        # discovered (default) | hardcoded (legacy)
fallback_order = ["anthropic", "openai", "gemini", "deepseek", "ollama"]  # discovered 의 우선순위

[llm.domain_mapping]
code = "<primary>"
server = "<discovered-cheapest>"
env = "<discovered-local-or-cheapest>"

[llm.thinking]
code = "enabled"
server = "disabled"
env = "disabled"
```

**동작 (provider-auto-config skill)**:
1. **Discover phase** — auth state + local server scan → `active-providers.toml` 생성
2. **Per LLM call** — runtime 에서 active-providers.toml 읽고 fallback chain 구성
3. **Failure 시** — 해당 provider status → `error` (재시도 안 함), 다음 fallback 시도
4. **Recovery** — `myharness auth <provider> test` 또는 startup 시 status 자동 refresh

**Retry 정책 (D-15, claude-code 2.1.166)**:
- primary 호출 실패 시 → discovered list 순서로 fallback
- **즉시 surface** 되는 error: auth, rate_limit, request_size, transport
- **retry-able** error: overloaded, timeout, transient → 1회 fallback retry

#### 5.5.4 라이브러리 (Rust 1안, D-36)

- `rig-core` 12+ provider (Anthropic/OpenAI/Google/Ollama native)
- 자체 OpenAI 호환 client (deepseek/minimax)
- `keyring` crate (OS keychain)
- `rmcp` 1.4 (MCP)
- **신규**: `auth/` module (per-provider auth + keychain 통합) + `provider/discovery.rs` (런타임 발견)

**모델 prefix 규약** (D-28):
```
anthropic/claude-sonnet-4-5
openai/gpt-5-codex
gemini/gemini-2.5-pro
deepseek/deepseek-reasoner
minimax/<model>
ollama/qwen2.5-coder:32b
```

→ unified identifier 로 config / log / cache key 모두 일관.

> **W7 (2026-06-09) 완료 (D-45)** — myharness-llm crate v1 구현: 6 provider enum + ProviderRegistry + LLMClient trait + rig-core 0.38 Anthropic/Gemini wrapper + OpenAI 호환 (DeepSeek/Ollama/local-llm) wrapper + MockClient + AuthState/AuthStatus + InMemory/Keyring AuthStore (libsecret 부재 환경 graceful fallback) + provider-auto-config discover (env+keychain+local scan) + ActiveProviderChain + FallbackRouter (cascade + per-provider status). 87 tests pass without ANTHROPIC_API_KEY (mock-driven). release 빌드 성공. §5.5 spec 그대로 구현.
> 
> **W8 (2026-06-09) 완료 (D-46)** — myharness-context crate v1 구현: CLAUDE.md loader (project root + parent walk + global fallback) + auto memory (NDJSON append-only, ~/.myharness/memory/auto/) + ContextManager (token budget + /compact Layer 1: Truncate/Summarize-stub/Hybrid) + Layer 2 BuiltinPipeline (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor, 기본 off) + ContextConfig (config.toml [context] 섹션) + ContextOrchestrator (전체 통합). 54 tests pass. release 빌드 성공. §5.6 spec 그대로 구현.
> 
> **W9 (2026-06-09) 완료 (D-47)** — myharness-compression crate v1 구현: Summarizer trait + LlmSummarizer (rig-core LLMClient 호출) + MockSummarizer (test) + TrivialSummarizer (extractive, no LLM) + CCR (Compress-Cache-Retrieve, in-memory `{ccr:N}` marker dictionary, reversible) + Kompress-base v1 simple (whitespace collapse + newline collapse + English stopword 제거 + best-effort stemming, ONNX 없이 text-entropy 기반) + BuiltinRegistry (6 알고리즘 통합 view: CacheAligner/ContentRouter/SmartCrusher/CodeCompressor/Ccr/KompressBase + flags). W9.2 에서 context 의 ContextManager 가 compression::Summarizer 의존 추가, Summarize/Hybrid 전략이 실제 LLM 요약 호출 (block_in_place 사용). compression 총 40 tests, context 56 tests (W9.2 summarize tests +2). release 빌드 성공. §5.6 6 알고리즘 spec 그대로 구현.
> 
> **W10 (2026-06-09) 완료 (D-48)** — myharness-tui crate v1 구현: ratatui::App + crossterm::TtyGuard (Welcome header + MessageList + Input box + status bar, TestBackend snapshot tests) + AppKey (Char/Enter/Backspace/CtrlC/방향키 매핑) + SubAgent trait + 4 hardcoded built-in sub-agents (code-reviewer, code-implementer, env-diagnose, git-operator) + SubAgentRegistry (lookup by kind/domain/name) + Orchestrator (3-tier dispatch: prefix/keyword/default, allowed_tools registry 검증, llm enhance append) + LoopRunner (ralph-wiggum 패턴, --goal + --success-criteria + --max-iterations + interrupt handle) + cli → tui 통합 (code/env/git/ask subcommand + 3-mode TUI shell). 50 tests pass. release 빌드 성공. 실제 binary 동작 확인 (`myharness code review foo.rs`, `myharness env diagnose`, `myharness ask "..."`). §5.10/§5.11 spec 그대로 구현.
> 
> **W11 (2026-06-09) 완료 (D-49) — TASK-005-1 v1 MVP 마지막 wave**: myharness-core crate v1 (workflow + permission + tool_alias 3 모듈) + standard_ai_workflow 6 원칙 native 구현 (TaskStartReport/TaskEndReport/EventLog/HandoffDoc + 한국어 직렬화) + 4 permission mode (default/acceptEdits/plan/bypassPermissions) + tool name alias (PascalCase ↔ snake_case 6 쌍) + MockClient FIFO + Orchestrator fatal_llm_error 옵션 + cli subcommand `task start|end` + `handoff`. core 32 tests + tui 51 tests + llm 90 tests + tools 51 tests + context 56 tests + compression 40 tests + cli workflow 통합 = **333 tests pass** (W11 이전 333 + W11 추가). release 빌드 성공. 실제 binary: `myharness task start --id TASK-001 --title "..." --intent "..."`, `myharness task end --id TASK-001 --summary "..." --risks "environment:libsecret 부재" --follow-up "TASK-002|next|description"`, `myharness handoff --from session-1 --to session-2`. §5.4/§5.9 spec 그대로 구현. **TASK-005-1 v1 MVP 6/8 waves 완료**.
> 
> **W12 (2026-06-09) 완료 (D-50) — MiniMax 실제 API 연결**: librarian 조사로 D-28 TBD 해소 (base_url `https://api.minimax.io/v1` + default `MiniMax-M3` + tool_use/vision/thinking/streaming 모두 지원 + OpenAI-호환 Bearer token + `MINIMAX_API_KEY` env). ProviderMetadata::builtin_minimax() 갱신 + available_models 7종 + tool_use=true. KeyringAuthStore 보완 (env-first + in-memory cache + BackendUnavailable hint 메시지). cli default LLM = `MINIMAX_API_KEY` env 자동 detect → OpenAiCompatProvider 로 real API client 구성 (`MINIMAX_API_HOST`/`MINIMAX_MODEL` env override 가능). discover() smoke test (--ignored, real-api). 95 llm tests + 5 commit dual-push. **다음: yklee 가 `MINIMAX_API_KEY` env 주입 → `cargo run -p myharness -- ask "..."` 으로 real network test 가능** (D-50 final verification).
> 
> **W13 (2026-06-09) 완료 (D-51) — OAuth 2.0 headless auth**: myharness-auth crate v1 (이전 skeleton). 7 모듈: `pkce` (RFC 7636 S256 + state) + `flow` (OAuth 2.0 Authorization Code + PKCE core, provider-agnostic) + `callback` (loopback HTTP server, 5min timeout) + `browser` (xdg-open/open/start) + `store` (`~/.myharness/oauth/{provider}.toml`, chmod 600) + `provider` (MiniMax / OpenAI / Google 3 provider, PKCE public client) + `manager` (login + refresh + status + logout 통합). 38 tests pass. cli subcommand `myharness auth <provider> login|logout|status` + `auth list`. **cli `auth login minimax --no-browser` 으로 OAuth URL 정상 생성 확인** (PKCE S256, state, scope, response_type=code). 4 commit dual-push. **다음: yklee 가 OAuth client ID 등록 (MiniMax console 또는 Google Cloud Console) → `auth login minimax` 으로 browser 자동 open + token 자동 저장**.

### 5.6 Context 관리 (claude-code 13.6 + 2-계층 압축, D-27 + D-30)

**3 계층 + 2-계층 압축 (D-30)**:

1. **`CLAUDE.md` (project root)** — yklee 의 프로젝트별 규칙, 5 surface 공유.
2. **Auto memory** — yklee 의 작업 패턴 자동 학습. `~/.myharness/memory/auto/`
3. **`/compact` slash command** — context 압축. **Layer 1 (필수) + Layer 2 (선택)** 호출.

**2-계층 압축 (D-30)**:

| 계층 | 목적 | always-on? | 비고 |
| --- | --- | --- | --- |
| **Layer 1 (필수)** | model length 한계 대응 | ✅ always-on (opt-out 불가) | token budget 추적 → 한계 근접 시 auto truncate/summarize → /compact (manual) |
| **Layer 2 (선택)** | 비용 최적화 | 🟡 opt-in (`builtin.enabled: true\|false`) | headroom 의 6 알고리즘 (CacheAligner, ContentRouter, CCR, SmartCrusher, CodeCompressor, Kompress-base) 을 우리 Context component 에 built-in |

**Layer 1 (필수, D-30) — 자동 압축 메커니즘**:
- **token budget 추적** — 매 message 마다 현재 사용량 추적
- **한계 근접 시 auto 압축** — 한계 80% 도달 시 자동 trigger
  - truncate: 오래된 message 일부 제거 (keep recent N=5)
  - summarize: 오래된 message 들을 LLM 으로 요약
  - hybrid: 둘 다
- **`/compact` slash command** — user-callable 수동 압축
- **opt-out 불가** — model 자체가 길이 제한 있으므로

**Layer 2 (선택, D-27) — headroom 알고리즘 built-in**:

```toml
# ~/.myharness/config.toml
[context]
compression = "native"        # native | builtin (D-27)

[context.builtin]
enabled = false               # ← 기본 OFF. 사용자가 true 로 켜면 동작
target_ratio = 0.35           # 65% 압축 목표
protect_recent = 5            # 최근 N 메시지 보호

[context.builtin.algorithms]
cache_aligner = true          # CacheAligner — prefix 안정화 (KV cache hit)
content_router = true         # ContentRouter — content type 감지 → 알고리즘 선택
ccr = false                   # CCR — reversible + retrieval (round-trip 비용)
smart_crusher = true          # SmartCrusher — JSON 구조 보존 압축
code_compressor = true        # CodeCompressor — AST-aware (tree-sitter)
kompress_base = false         # Kompress-base ML — 자유 텍스트 (95% 압축, ONNX)
```

**흐름 (D-27: user → harness → (headroom) → LLM)**:

```
yklee 명령
   ↓
my_harness 의 Context component
   ↓
   ├─ CLAUDE.md load
   ├─ auto memory inject
   ├─ /compact (user-callable) or auto-detect
   ↓
   └─ Built-in compression layer (선택적, off 가능)
        ├─ CacheAligner (prefix 안정화) [always on if enabled]
        ├─ ContentRouter (content type 감지)
        │    ├─ JSON → SmartCrusher
        │    ├─ code → CodeCompressor
        │    ├─ log → LogCompressor
        │    └─ text → Kompress-base ML (if enabled)
        └─ CCR (reversible, round-trip)
   ↓
LLM provider API
```

**핵심 (D-27)**:
- **headroom 의 압축 알고리즘을 우리 Context component 에 built-in** — 외부 proxy/MCP 의존 X
- headroom 의 **알고리즘/원리만 참고** (CacheAligner, ContentRouter, CCR, SmartCrusher, CodeCompressor, Kompress-base) — Apache 2.0 알고리즘 디자인
- **선택적, 기본 off** — 사용자가 켜야 동작. native cache (Anthropic prompt cache, OpenAI cached prompt) 만으로도 v1 가능
- **proxy 제약 회피** — proxy mode 의 인증/transport 제약 없음, 우리 harness 의 Tools/Plugins 와 직접 통합 가능

**v1 우선 구현 (3 알고리즘)**:
1. **CacheAligner** — prefix 안정화 (가장 효과 큰 1순위, Anthropic prompt cache hit ↑)
2. **ContentRouter + SmartCrusher** — JSON 출력 (tool result) 65% 압축
3. **CodeCompressor** — code snippet (tree-sitter) 식별자 shorten + 주석 제거

**v1.5+ 구현**:
- CCR (reversible + retrieval) — round-trip 비용 trade-off
- Kompress-base (ONNX) — 95% 자유 텍스트 압축, ML 모델 weight 포함 (~수 MB)

### 5.7 Plugin 시스템 (D-135.3 — grok plugin.json)

자체 Plugin loader / dispatcher 는 **만들지 않는다.** grok 가 읽는 트리를 저장소에 둔다.

```
plugins/myharness/              # --plugin-dir (자동 trust)
├── plugin.json                 # name, version, skills/commands/agents/hooks
├── commands/                   # slash 추가 (빌트인 충돌 시 빌트인 승)
├── agents/                     # .md (빌트인 3 이름 섀도잉 금지)
├── skills/                     # SKILL.md
└── hooks/hooks.json            # PreToolUse 등 15 이벤트
```

사용자 사본: `~/.myharness/plugins/myharness/`. marketplace 는 grok 측 (`grok plugin install`). 우리 marketplace 서버는 OOS.

**TASK-005-2 Sub-task 2 (자체 Plugin 인프라 A1~A4) = OOS.**

### 5.8 런타임 의존 (D-135.1, D-25 개발 분리 유지)

**유일한 엔진 의존**:
- 설치된 **`grok` ≥ 1.0.3** (Grok Build, Apache 2.0)
- LLM 은 grok `[model.*]` 경유 (MiniMax / Ollama / OpenAI 호환)
- OS 표준 라이브러리

**결합하지 않음** (D-25 유지):
- ❌ Mavis / mavis-team — 개발 도구일 뿐
- ❌ 4-워커 (Claude/Codex/Gemini/OpenCode) — sibling
- ❌ grok **소스 트리** — 바이너리만
- ❌ 자체 TUI / tool registry / session store (v0 crates 는 참고만)

**호환되는 외부 도구** (sibling, not dependency):
- 사용자가 plugin 으로 추가 가능 (claude-code 4-계층 plugin 시스템)
- 사용자가 MCP server 추가로 연결 가능
- 사용자가 shell script / 다른 CLI 와 pipe/compose 가능 (Unix philosophy)

**개발 시 사용 도구 (사용자 환경)**:
- my_harness 자체는 Mavis / Mavis 와 무관
- 단, yklee 가 my_harness 를 **개발**할 때 Mavis 를 dev tool 로 사용 (D-01 의 standard_ai_workflow 는 my_harness 개발 workflow 일 뿐, my_harness 의 runtime dependency 아님)

### 5.9 standard_ai_workflow 준수 (D-26, native + 옵션 통합)

my_harness 는 `standard_ai_workflow` (ykylee/standard_ai_workflow) 의 **6 원칙을 native 로 내장**. **Zero coupling** (Mavis 파일 없어도 동작) + **옵션 통합** (Mavis 디렉토리 발견 시 자동 연결).

#### 5.9.1 6 원칙 native 구현 (항상 동작)

| 원칙 | 구현 |
| --- | --- |
| **한국어 보고** | 모든 사용자 facing output 기본 한국어. `--lang=en` 으로 override |
| **컨텍스트 절약** | 결론 + 다음 행동만 출력. 중간 reasoning 노출 안 함 |
| **상태값** | `planned \| in_progress \| blocked \| done` 4 값 (TASK status 출력 시) |
| **이벤트 소싱** | 모든 상태 변경/명령 실행을 `.myharness/log.jsonl` 에 기록 (자체 저장) |
| **비참조 원칙** | 다른 세션/이전 세션 참조 안 함. handoff 만 사용 |
| **handoff 형식** | 모든 work 종료 시 `summary / risks / suggested_follow_up` 구조화 출력 |

#### 5.9.2 옵션 Mavis 통합 (auto-detect, opt-in)

```toml
# ~/.myharness/config.toml (예시)
[workflow]
mode = "auto"               # auto | none | mavis
mavis_root = "~/mavis"      # Mavis 디렉토리 위치
# auto: Mavis 디렉토리 (`.ai-workflow/` 또는 `ai-workflow/`) 발견 시 자동 통합
# none: 항상 my_harness 자체 `.myharness/` 만 사용 (Mavis 무시)
# mavis: 명시적 통합 (Mavis 디렉토리 없으면 에러)
```

**auto mode 동작**:
- `ai-workflow/memory/state.json` 발견 시 → task status 자동 sync
- `ai-workflow/memory/work_backlog.md` 발견 시 → task 등록/갱신
- `ai-workflow/memory/session_handoff.md` 발견 시 → 종료 시 자동 append
- 미발견 시 → my_harness 자체 `.myharness/state/`, `.myharness/handoff/` 사용 (zero coupling 유지)

**호환되는 Mavis 워크플로우 파일**:
- `ai-workflow/memory/state.json` — 워크플로우 상태 캐시
- `ai-workflow/memory/session_handoff.md` — 세션 인계
- `ai-workflow/memory/work_backlog.md` — 작업 인덱스
- `ai-workflow/memory/backlog/YYYY-MM-DD.md` — 일별 백로그
- `ai-workflow/core/global_workflow_standard.md` — 표준 자체 (참조만, my_harness 가 직접 읽지 않음)

#### 5.9.3 Task/handoff 출력 형식 (Mavis 호환)

```yaml
# myharness task start
task:
  id: TASK-005
  title: "my_harness 스택 결정 (Rust 1안 vs TS 2안)"
  status: in_progress
  started_at: 2026-06-07T12:08:24+09:00
  priority: high
context_summary: |
  Rust 1안 vs TS 2안 결정. 입력: REFERENCES.md §5, PROVIDERS.md §9,
  headroom.md §13, claude-code.md §13, README.md §3, CONCEPT.md §5.5/§5.7
constraints: |
  - 단일 binary 우선
  - 3-도메인 동시 지원
  - Mavis zero coupling
output_files:
  - docs/TASK-005_DECISION.md
  - docs/architecture/...
```

```yaml
# myharness task end
task:
  id: TASK-005
  status: done
  completed_at: 2026-06-07T12:35:00+09:00
summary: |
  Rust 1안 (ratatui + rig-core + cargo-dist) 으로 결정. 근거: ...
risks_identified:
  - TUI 학습곡선 (Rust 진입장벽)
  - MCP SDK Rust 성숙도 (rmcp 1.4 검증 필요)
suggested_follow_up:
  - TASK-005-1: ratatui POC 작성
  - TASK-005-2: rig-core integration test
  - TASK-005-3: cargo-dist cross-build 검증
produced_artifacts:
  - docs/TASK-005_DECISION.md
```

#### 5.9.4 우리 my_harness 의 위치 (재확인, D-140)

```
┌─────────────────────────────────────────┐
│  yklee (terminal)                        │
└─────────────────────────────────────────┘
              ↓ `myharness`
┌─────────────────────────────────────────┐
│  surface/ (화면)                         │
│  - 크롬 / 12 동사 / 6원칙 / ACP          │
└─────────────────────────────────────────┘
     ↓ -p (한 턴)          ↓ agent stdio
   ~/.grok/ (엔진)                 ~/.myharness/ (표면)
     sessions/                       state/
     auth.json                       handoff/
     memory/                         log.jsonl
     config.toml                     plugins/
```

**핵심**:
- **6 원칙은 표면이 지킨다**
- **세션 본문은 grok `updates.jsonl`**
- **개발 workflow ≠ 산출물 런타임** (D-25)

---

### 5.10 Agent 모드 (3가지, D-29 · D-140 매핑)

기본 화면은 **우리 TUI** (S8+). grok pager 가 아니다.

- `orchestrator` (default) → `surface/` TUI (`grok agent stdio`)
- `single` → `grok -p` 한 턴
- `loop` → 프롬프트에 goal 삽입 (1차)
- `myharness engine` → 벤더 TUI (opt-in)

my_harness 의 **메인 에이전트**는 기본적으로 **orchestrator** 역할. 작업 카테고리마다 **sub-agent 를 내장**하여 작업 분배 + context 효율화. 모드 변경으로 단일 에이전트 / 무한루프 모드 전환 가능.

| 모드 | 기본? | 동작 | 사용 시나리오 |
| --- | --- | --- | --- |
| **orchestrator** (default) | ✅ | 메인 에이전트 = orchestrator. 작업 카테고리별 sub-agent spawn, 통합 | 일반 작업 (코드/서버/환경) |
| **single** | 🟡 opt-in (`--mode=single`) | 단일 에이전트, sub-agent spawn 안 함. context 직접 처리 | 간단한 Q&A, 단일 파일 작업 |
| **loop** | 🟡 opt-in (`--mode=loop`) | orchestrator + sub-agent + 무한루프 (ralph-wiggum 패턴). goal 달성까지 자동 반복 | well-defined goal (e.g., "fix all failing tests", "implement X until CI green") |

**CLI flag**:
```bash
myharness --mode=orchestrator code review <pr>    # default
myharness --mode=single ask "what does this do?"
myharness --mode=loop --goal "fix all TODO comments" --max-iterations=20 .
```

**Loop mode 상세 (claude-code ralph-wiggum 패턴, D-29)**:
- `--goal "<text>"` — 달성 목표 (필수)
- `--success-criteria "<text>"` — LLM 이 success 평가 기준 (선택)
- `--max-iterations N` — 최대 반복 (default: 20)
- Stop condition: success-criteria 충족 OR max-iterations 도달 OR user Ctrl+C
- 안전: --max-iterations 기본값 + user interrupt 가능 (run-away 방지)

### 5.11 Built-in sub-agents (D-29)

3-도메인 × 4-5 sub-agents = **~15 내장** sub-agent. 각 sub-agent = specialized system prompt + tool restrictions + our Context component 의 sub-set.

| 도메인 | sub-agent | 역할 |
| --- | --- | --- |
| **코드** | `code-reviewer` | PR/code review (multi-aspect: bugs / style / tests) |
| | `code-implementer` | 새 기능 구현, multi-file 변경 |
| | `code-tester` | test 실행 + 결과 분석 + fix 제안 |
| | `code-refactorer` | 리팩토링 (rename / extract / dedup) |
| | `code-searcher` | codebase 검색 + 구조 분석 |
| **서버** | `server-status` | 프로세스/서비스 상태 점검 |
| | `log-analyzer` | 로그 분석 + 이상 패턴 detection |
| | `deployer` | 배포 헬퍼 (ssh / k8s / docker) |
| | `config-manager` | 설정 조회/변경 (with backup) |
| **환경** | `env-setup` | 스택별 부트스트랩 (brew/asdf/dotfiles) |
| | `env-installer` | 의존성 설치 (with idempotency) |
| | `env-shell` | 셸 명령 + LLM 분석 |
| | `env-diagnose` | 환경 진단 (path/version/permission) |
| **Utility** | `git-operator` | git workflow (commit/PR/branch) |
| | `file-searcher` | file glob/find/grep |

**Sub-agent 정의 위치** (D-135):
- 엔진 빌트인 3: `general-purpose` / `explore` / `plan` (이름 섀도잉 금지)
- 우리 도메인 에이전트: `plugins/myharness/agents/*.md`
- v0 하드코딩 Rust/Python sub-agent = 참고만. 신규 추가 금지

**Orchestrator 의 dispatch 로직**:
- user 명령 분석 → 도메인/카테고리 매칭 → 적절한 sub-agent spawn
- sub-agent 결과 통합 → user 에게 한국어 보고 (D-26)

### 5.12 `~/.myharness/` 디렉토리 구조 (D-31 · D-135.5)

yklee 환경 검증: 다른 agent 도구 모두 `~/.<toolname>/` 컨벤션 (claude/codex/gemini/headroom/minimax/jules/coderabbit). 우리도 동일.

**D-135.5**: 엔진 홈은 `~/.grok/` (세션 JSONL, auth.json, grok config). 래퍼 홈은 `~/.myharness/` (task/handoff/log/plugin 사본). `GROK_HOME=~/.myharness` 로 합치지 않음.

**v1 구현 범위 (D-69)**: 표의 **11 top-level dirs + root** 가 `init_home_dir()` (paths.rs:141) 가 자동 생성. sub-dir (state/current.toml, memory/auto/, compression/cache/, sub-agents/<name>/, cache/models/, llm-wiki/ 등) 는 v1.5+ 구현. **OAuth token 실제 dir** = `~/.myharness/oauth/{provider}.toml` (`TokenStore::new()`, store.rs:39) — §5.5.2 의 `state/auth/` 와 다름 (해당 경로는 `state_auth_toml()` = auth **state metadata** 용, status/last_login 등).

```
~/.myharness/                          # ROOT (XDG-aware)
├── config/                           # 사용자 편집 가능 config
│   ├── config.toml                   # 메인 설정 (LLM, mode, compression, permission) [D-42]
│   ├── providers.toml                # provider registry (D-28, D-42)
│   ├── plugins/                      # user plugins (commands/agents/skills/hooks)
│   ├── skills/                       # user skills (claude-code 13.3)
│   ├── hooks/                        # global hooks (markdown rules, D-13.4)
│   └── mcp.json                      # MCP server config (D-33)
├── state/                            # workflow state (D-26, standard_ai_workflow)
│   ├── current.toml                  # current task
│   └── tasks/                        # task history
├── memory/                           # auto + manual memory
│   ├── auto/                         # LLM 자동 축적 (D-26)
│   └── manual/                       # user-marked
├── handoff/                          # session handoff (D-26, Mavis 호환)
├── log.jsonl                         # event log (append-only, D-26)
├── compression/                      # built-in compression artifacts (D-27, D-30)
│   ├── cache/                        # CacheAligner prefix cache
│   ├── summaries/                    # Layer 1 auto-summaries (D-30)
│   └── ccr/                          # CCR reversible storage (D-27, v1.5+)
├── sub-agents/                       # built-in sub-agents (D-29)
│   ├── code-reviewer/
│   ├── code-implementer/
│   └── ...
├── llm-wiki/                         # LLM Wiki memory (D-32, v2+)
│   ├── raw/                          # source materials
│   ├── wiki/                         # compiled interlinked markdown
│   ├── schema/                       # constraints
│   ├── index/                        # searchable
│   └── log/                          # change history
├── runtime/                          # runtime state (not user-edited)
│   ├── lock                          # single instance
│   ├── session.pid
│   └── metrics.json
└── cache/                            # regenerable cache
    ├── models/                       # ONNX (Kompress-base, v1.5+)
    ├── tree-sitter/                  # tree-sitter parsers
    └── embeddings/                   # v2+
```

**Cross-platform (D-31)**:
- macOS / Linux: `~/.myharness/` (XDG-style root)
- Windows: `%USERPROFILE%\.myharness\` (동일)
- 구현: `directories` (Rust) / `env-paths` (TS) cross-platform wrapper
- 우리 v1 = single root (yklee 환경 검증 결과, sibling tools 와 일치)

**옵션 Mavis 디렉토리 발견 시 sync (D-26)**:
- `ai-workflow/memory/state.json` 발견 → `~/.myharness/state/current.toml` 와 sync
- `ai-workflow/memory/work_backlog.md` 발견 → task 등록/갱신
- 미발견 시 → `~/.myharness/` 만 사용 (zero coupling 유지)

### 5.13 LLM Wiki memory (D-32)

**Karpathy's LLM Wiki Pattern (2026-04)**:
> "Instead of just retrieving from raw documents at query time, the LLM incrementally builds and maintains a persistent wiki — a structured, interlinked collection of markdown files that sits between you and the raw sources."

핵심 비유: **"Obsidian is the IDE, the LLM is the programmer, the wiki is the codebase"** — LLM 이 wiki 를 **쓰는** 것 (사용자가 wiki 를 **읽는** 것).

**3 계층 + 운영 메커니즘**:
| 계층 | 내용 | 예시 |
| --- | --- | --- |
| **raw/** | 원본 자료 (변경 안 함) | session log, handoff, log.jsonl |
| **wiki/** | LLM 이 컴파일한 interlinked markdown | `<topic>.md` pages, cross-references, 종합 |
| **schema/** | 제약, 검증 규칙 | "wiki page must have Observations + Relations" |

| 운영 | 내용 |
| --- | --- |
| **index/** | 검색 가능 (entity, topic) |
| **log/** | 변경 이력 (append-only) |
| **lint/** | 검증 (contradiction detection, link integrity) |

**v1: 기본 flat memory (LLM Wiki 미적용)**:
- `~/.myharness/memory/auto/` + `manual/` (단순 파일)
- 검색: ripgrep
- LLM 이 명시적으로 compile 안 함

**v2+: LLM Wiki (D-32)**:
- 3 계층 (raw/wiki/schema) 자동 운영
- LLM 이 task 종료 시 raw → wiki compile (background)
- 사용자 query 시 wiki 에서 cross-reference 따라 탐색
- contradiction detection + resolution
- Karpathy gist reference: `gist.github.com/karpathy/442a6bf555914893e9891c11519de94f`

**우리 적용**:
- v1: LLM Wiki 미구현. 기본 flat memory + handoff 만
- TASK-005-2 (v1.5): LLM Wiki v1 — schema + lint 만
- TASK-005-4 (v2.5): LLM Wiki v2 — full compile + cross-reference
- **TASK-005-2 v1.5 (D-71)**: vault 위치 = **`~/wiki/`** (out-of-repo, Obsidian 직접 open). ai-workflow/ 와 관계 = **consumer** (raw/ai-workflow/ 로 sync, SSOT 아님). 상세: [architecture/DETAILED_DESIGN_LLM_WIKI.md](../architecture/DETAILED_DESIGN_LLM_WIKI.md)

**SSOT 원칙 (R-4)**:
- `CONCEPT.md` = single source of truth (이 문서)
- `~/wiki/` = **derived** view. 위키 내용이 CONCEPT 와 drift 되지 않도록 wiki-lint 스킬 (ai-workflow/skills/wiki-lint/, v1.5) 가 양방향 검증.
- mirror 아님 — wiki/ 의 결정·정책을 본 문서에 다시 반영할 필요 없음. 본 문서가 바뀌면 wiki/ 갱신은 lint 가 알림.

### 5.14 Skill/MCP first-class (D-33)

다른 도구 (claude-code, gemini-cli, goose) 와 **동등한** extension 지원.

#### Skills (claude-code 13.3 차용)

```
~/.myharness/skills/<name>/
├── SKILL.md          # 자동 invoke knowledge (markdown + YAML frontmatter)
├── examples/         # (optional)
└── scripts/          # (optional)
```

**SKILL.md 형식**:
```markdown
---
name: frontend-design
description: Auto-invoke for frontend work (bold design, typography, animations)
auto_invoke:
  triggers: [frontend, UI, design, component]
  priority: high
---

# Frontend Design Skill

## Principles
- Distinctive design (avoid generic AI aesthetic)
- Bold typography, motion, visual details
...
```

**Built-in skills catalog (3-도메인)**:
| skill | 도메인 | invoke trigger |
| --- | --- | --- |
| `code-review-best-practices` | 코드 | PR review, code review |
| `git-workflow` | 코드 | commit, PR, branch |
| `server-health-check` | 서버 | status, health |
| `log-pattern-analysis` | 서버 | log analysis |
| `env-bootstrap` | 환경 | setup, install |
| `dotfiles-sync` | 환경 | dotfiles, shell config |
| **`provider-auto-config`** (D-38) | infra | startup / `auth` / fallback 실패 — 동적 LLM provider 발견 + per-provider auth |

#### MCP (Model Context Protocol)

```
~/.myharness/mcp.json
{
  "mcpServers": {
    "filesystem": {
      "command": "uvx",
      "args": ["mcp-server-filesystem", "/Users/yklee"]
    },
    "github": {
      "command": "uvx",
      "args": ["mcp-server-github"],
      "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" }
    }
  }
}
```

**구현**:
- Rust 1안: `rmcp` (Rust MCP SDK) — goose 와 동일 crate
- TS 2안: `@modelcontextprotocol/sdk` — 공식 표준

**Auto tool exposure** (D-32):
- MCP server 의 tools 가 우리 sub-agent 의 tool registry 에 자동 등록
- `mcp__filesystem__read_file`, `mcp__github__create_pr` 등

**v1: 기본 (3-4개 MCP server pre-config)**:
- `filesystem` (read/write local file)
- `git` (git operations)
- `shell` (bash execution)
- (선택) `github` (PR/issue)

**v1.5+**: marketplace / plugin 으로 사용자 정의 MCP 추가.

---

## 6. 로드맵 (D-135 이후)

> 제품 경로는 overlay. TASK-005-1 v0 Rust MVP 는 **완료된 historical**. 자체 Plugin 인프라 (구 TASK-005-2 Sub-task 2) 는 **OOS**.

| task_id | 핵심 | 상태 |
| --- | --- | --- |
| **TASK-005-1** (v0 runtime) | 자체 Rust crates CLI/TUI | done (historical, D-135.6 참고만) |
| **D-135 PR-1** | `plugins/myharness/` 스캐폴드 | **done** (D-136 M1) |
| **D-135 PR-2** | thin CLI 래퍼 + grok 가드 + 12 동사 | **done** (D-136 M1, `bin/myharness`) |
| **D-135 PR-3** | MiniMax `[model.*]` smoke | **done** (D-137, `setup-model`. live API 는 키 opt-in) |
| **D-135 PR-4** | 3-도메인 skills + PreToolUse | **done** (D-137) |
| **D-135 PR-5** | `task start\|end` 래퍼 | **done** (D-137) |
| **D-138 M3** | `scripts/install.sh` + README 설치 | **done** (Rust clap M3.2 deferred) |
| **D-140 S0** | 문서 SSOT Owned Surface | **done** |
| **D-140 S1** | `surface/` 크롬 TUI (엔진 없음) | **done** |
| **TASK-005-2** 자체 Plugin loader | — | **OOS** (grok plugin.json 이 대체) |
| **TASK-005-3+** 5 surfaces / Computer Use | 엔진(grok ACP) 범위. 우리 재구현 안 함 | deferred |

---

## 7. 채택 패턴 (Adopt 23개, 1차 MVP 우선)

### 1차 MVP (v1, 8개) ⭐
1. **Harness 5 components** (claude-code 13.1) — Tools/Context/Session/Plugins/Sub-agents
2. **CLAUDE.md 표준** (claude-code 13.6) — 우리 `MiniMax.md` 가 동급
3. **Hook markdown rule** (claude-code 13.4 hookify) — `~/.myharness/hooks/*.md`
4. **4 permission mode** (claude-code 13.8) — default/acceptEdits/plan/bypassPermissions
5. **3 fallback model** (claude-code 13.15) — primary + 2 fallback
6. **5 install paths** (claude-code 13.9) — install.sh/ps1/brew/winget/linux pkg
7. **CCR (headroom 13.3)** — MCP server 1안 통합
8. **Provider 비종속** (aider/opencode/goose 13.2) — rig-core 1안 / Vercel AI SDK 2안

### 2차 (v1.5, 7개)
9. Plugin 4-계층 (claude-code 13.3)
10. Auto memory (claude-code 13.5)
11. /compact slash command (claude-code 13.7)
12. MCP server 1안 (claude-code 13.24)
13. Sub-agents + Agent SDK (claude-code 13.22)
14. CacheAligner (headroom 13.5)
15. ContentRouter (headroom 13.4)

### 3차 (v2+, 8개)
16. 5 surfaces cross-surface (claude-code 13.2)
17. Plugin marketplace (claude-code 13.3)
18. Routines (claude-code 13.17)
19. Multi-agent parallel + confidence scoring (claude-code 13.11)
20. Channels (claude-code 13.25)
21. Security 3-tier (claude-code 13.13)
22. Cross-session security (claude-code 13.14)
23. Thinking toggle per-model (claude-code 13.20)

---

## 8. 안티 패턴 (절대 안 함)

1. **closed source + leak 의존** → 우리 코드 MIT/Apache. 엔진은 grok (Apache 2.0 바이너리)
2. **grok 소스 포크를 뼈대로 삼기** (D-134/D-135) — 136만 줄 + generated workspace + PR 거부
3. **5 components 재구현** — 엔진이 이미 가지고 있음
4. **100+ slash commands** → **3-도메인 × 3-4 명령** = ~12 명령 max
5. **5 surface 동시 유지** → 래퍼 CLI + grok TUI 만
6. **cloud auto memory** → local-only (`~/.grok/memory`, `~/.myharness/`)
7. **CONCEPT §0 standalone 을 유지한 채 grok 를 뼈대로 쓰기** — 모순 (폐기됨)
8. **hook fail-open 을 보안 경계로 믿기**
9. **브랜드/키맵/빌트인 slash 를 overlay 로 지우기**

---

## 9. 성공 지표 (KPI)

| 지표 | v1 목표 (3개월) | v2 목표 (6개월) |
| --- | --- | --- |
| **사용 빈도** | yklee 주 5+ 일 사용 | 매일 사용 |
| **도메인 커버리지** | 3-도메인 모두 1+ 명령 사용 | 3-도메인 모두 3+ 명령 사용 |
| **플러그인** | local 3+ (yklee 작성) | marketplace 10+ |
| **Context 압축률** | CCR 60%+ 토큰 절감 | 80%+ |
| **Fallback 발동률** | <5% | <1% |
| **Cross-platform 빌드** | mac/linux/win 3개 동시 | 동일 |
| **Token 비용** | yklee 의 Claude Code 사용 대비 50%↓ | 70%↓ |

---

## 10. 리스크 + 대응

| 리스크 | 영향 | 대응 |
| --- | --- | --- |
| **Worker long Write abort** (D-16) | 분석/문서 작업 지연 | chunked write + early deliverable signal + minimal board noise |
| **grok 업데이트** | 래퍼 플래그/plugin 스키마 깨짐 | `min_version` 가드 + smoke |
| **MiniMax + xAI-only 툴** | image/web_search 실패 | plugin 문서화, 해당 툴 비활성 |
| **Provider API 변경** | 호환성 깨짐 | grok `[model.*]` 가 흡수. 우리 llm crate 신규 작업 금지 |
| **headroom API 변경** | CCR 재구현 유혹 | Layer 2 재구현 OOS (D-66/D-130). grok compaction 사용 |
| **5 surface 점진 확장 시 maintenance 부담** | bug fix 5x | v1 CLI+TUI 만, v2 부터 surface 추가 시 별도 sprint |
| **plugin 생태계 부재** (v1 local only) | 확장성 제한 | v1.5 부터 marketplace beta |
| **3 fallback 의 provider 가용성** | fallback 발동 시 지연 | fallback 도 동일 abstraction 위에서 |
| **Local-only memory** (privacy) | cross-device 사용 불가 | v2+ opt-in cloud with encryption |

---

## 11. 결정 보류 (Open Decisions)

### 11.1 결정 보류 표

| task_id | 결정 | 보류 이유 | 결정 시점 |
| --- | --- | --- | --- |
| **TASK-002** | 도메인별 명령 | yklee 인프라 정보 필요 | yklee 인프라 정보 수령 후 |
| **TASK-005** | 스택 (Rust 1안 vs TS 2안) | — | ✅ **D-36 결정: Rust 1안** (§11.3 참조) |
| **TASK-006** | TUI 라이브러리 (ratatui vs React/Ink) | — | ✅ **D-36 결정: ratatui + crossterm** (TASK-005 종속) |
| **TASK-007** | headroom built-in 알고리즘 구현 우선순위 | — | ✅ **D-37 결정: v1 = 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor), CCR + Kompress-base v1.5+ 로 연기** |
| **TASK-008** | Provider fallback list (3 모델) | — | ✅ **D-38 결정: 하드코딩 폐기 → `provider-auto-config` skill (D-38) 로 런타임 discovered list + per-provider auth 동적 구성** |
| **제품 경로** | overlay / grok 포크 / goose 포크 | — | ✅ **D-135 결정: A overlay** ([DETAILED_DESIGN_OVERLAY.md](./architecture/DETAILED_DESIGN_OVERLAY.md)) |

### 11.2 결정 완료 — D-135 (2026-08-14)

**제품 경로 = Grok Build overlay.**

- 엔진 = 설치된 `grok` ≥ 1.0.3
- 표면 = `myharness code|server|env`
- 확장 = grok `plugin.json` (`--plugin-dir`)
- LLM = `[model.minimax]` / `[model.ollama]`
- 홈 분리 = `~/.grok/` 엔진 + `~/.myharness/` 래퍼
- v0 crates = 참고 구현, 신규 기능 금지
- 자체 Plugin 인프라 (A1~A4) = OOS
- CONCEPT §0 standalone / Direct LLM / Zero external runtime = 폐기

상세: [DETAILED_DESIGN_OVERLAY.md](./architecture/DETAILED_DESIGN_OVERLAY.md) §1 Key Decisions.

### 11.3 결정 완료 (Decided) — D-36

#### TASK-005: 스택 = **Rust 1안** (2026-06-07)

**결정**: yklee 결정 (Rust 1안 우선). 향후 변경 시 재검토.

**선택 근거**:
1. **단일 binary** — `cargo-dist` 로 macOS/Linux/Windows 동시 빌드. 우리 §5.3 5 install paths (D-31) + cross-platform + native auto-update 에 최적
2. **TUI 검증** — `ratatui + crossterm` (codex 가 검증, Rust TUI 표준)
3. **MCP 성숙** — `rmcp` 1.4 (goose 가 사용 중)
4. **Keychain 안정** — `keyring` crate (goose 검증, macOS Keychain / Windows Credential Manager / Linux Secret Service)
5. **빠른 startup + low memory** — 단일 binary = TUI latency ↓
6. **Provider 비종속** — `rig-core` 12+ provider (Anthropic/OpenAI/Google/Ollama native)
7. **headroom 알고리즘 native 구현** — tree-sitter (Rust), CCR (Rust), Kompress-base (ONNX C++ binding) 모두 Rust 생태계 성숙
8. **Desktop 확장 (TASK-005-3, v2.0)** — Tauri (Rust) = 5 surface cross-session 시 single binary + Web view 동시

**참조**: [REFERENCES.md §5.1](./REFERENCES.md), [references/README.md §2 축 1](./references/README.md), [references/PROVIDERS.md §2-3](./references/PROVIDERS.md), [references/claude-code.md §13.1](./references/claude-code.md)

#### TASK-006: TUI 라이브러리 = **ratatui + crossterm** (TASK-005 종속, D-36)

**결정**: TASK-005 = Rust 1안 종속으로 자동 확정. `ratatui` (TUI) + `crossterm` (terminal backend) 사용.

**v1 스택 종합**:
```
Language:    Rust 2024 edition
TUI:         ratatui + crossterm
LLM:         rig-core 12+ provider
MCP:         rmcp 1.4
Secret:      keyring crate
Compression: tree-sitter-rust + ONNX Runtime (Kompress-base, v1.5+)
Build:       cargo + cargo-dist
Distribution: 5 install paths (install.sh / install.ps1 / brew / winget / apt-dnf-apk)
```

**구현 우선순위 (TASK-005-1, v1 MVP)**:
1. Rust 프로젝트 init + cargo workspace
2. ratatui TUI shell (메뉴/스크롤/키바인딩)
3. rig-core LLM client (Anthropic 우선, 1 provider)
4. basic Tools (Read/Write/Edit/Bash)
5. Context (CLAUDE.md load + /compact)
6. standard_ai_workflow output (한국어/상태/handoff)
7. 4 permission mode (§5.4)
8. 1-2 built-in sub-agent (code-reviewer, server-status)

#### TASK-008: Provider fallback = **런타임 discovered list** (D-38, NEW)

**결정**: yklee 결정 — **하드코딩 fallback list 폐기** → **`provider-auto-config` skill** 로 동적 발견 + per-provider auth.

**선택 근거**:
1. **환경 가변성** — yklee 의 API key 보유 상태 / 조직의 SSO / local LLM 가용성 모두 시점/맥락 의존
2. **사용자 개입 최소화** — API key 만 등록하면 자동 fallback chain 구성
3. **확장성** — 새 provider 추가 시 코드 변경 없이 config + auth 만 등록
4. **local-first 우선** — Ollama/vLLM 자동 발견 시 cost 0 fallback
5. **graceful degrade** — primary 실패 시 discovered list 순차 시도, error surface 최소화

**하드코딩 폐기 이유**:
- A안 (Claude-first) 등 5개 옵션 비교는 모두 **"이 시점의 yklee 환경 가정"** — 다른 환경 (CI, 다른 머신) 에선 무효
- 시간 지나면 API key 만료 / 새 provider 등장 / local LLM 켜고 끄기 → fallback 갱신 필요
- 코드로 박으면 환경별 config 분기 폭발

**v1 Phase 1 (TASK-005-1, MVP)**:
- 6 provider 정적 등록 (`config.toml`)
- `auth list` / `auth <provider>` status 조회
- Anthropic API key (env → keychain fallback)
- Ollama local server detect
- **단순 fallback** (config 의 primary + fallback hardcoded) — **동적 발견은 v1.5+**

**v1 Phase 2 (TASK-005-2, v1.5)**:
- `provider-auto-config` skill 정식 구현
- 모든 provider auth (login/logout/test)
- `active-providers.toml` 자동 생성/갱신
- **dynamic fallback chain**

**v1 Phase 3 (TASK-005-3, v2.0)**:
- OAuth flow (Anthropic OAuth, Google OAuth)
- MCP-based provider 등록 (`mcp__*` 자동 discover)
- Multi-region / multi-account

**Skill reference design**: [`docs/skills/provider-auto-config/SKILL.md`](./skills/provider-auto-config/SKILL.md) (D-38)

**영향 결정**:
- §5.5 LLM 통합 전면 갱신 (정적 config → 동적 discover + auth)
- §5.14 Built-in skills 에 `provider-auto-config` 추가
- §5.12 디렉토리 구조에 `~/.myharness/state/auth/` 추가
- D-15 (3 fallback 패턴) — **Phase 1 은 hardcoded 유지**, **Phase 2 부터 동적**
- D-34 (2.1.169 영향) — **D-40 으로 취소, 검증 미진행** (v1 spec 잠금)

---

## 12. 참고 (References)

- **[architecture/DETAILED_DESIGN_OVERLAY.md](./architecture/DETAILED_DESIGN_OVERLAY.md)** — D-135 구현 사양 (제품 경로 SSOT)
- [architecture/INITIAL_DESIGN.md](./architecture/INITIAL_DESIGN.md) — v0 Rust MVP 설계 (historical)
- [REFERENCES.md](./REFERENCES.md) — 1차 8축 (superseded)
- [references/README.md](./references/README.md) — 8-doc 통합 인덱스 + cross-review
- [references/grok-build.md](./references/grok-build.md) — Grok Build 14섹션. D-135 의 실측 근거
- [references/claude-code.md](./references/claude-code.md) — Harness 5 components (엔진 측 패턴)
- [references/headroom.md](./references/headroom.md) — CCR (재구현 OOS)
- [references/PROVIDERS.md](./references/PROVIDERS.md) — v0 provider 비교 (historical)
- [development_log.md](./development_log.md) — 결정 이력
- [PROJECT_PROFILE.md](./PROJECT_PROFILE.md) — 워크플로우 통합
- [MiniMax.md](../MiniMax.md) — 개발 진입점
