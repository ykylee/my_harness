# my_harness — 개발 컨셉 (v1 SPEC / Master Concept)

> **본 문서는 my_harness 의 단일 진실 공급원 (single source of truth)**: 7 reference 분석 종합 + yklee 의 작업 컨셉 + TASK-005/002/007 결정 입력을 모두 반영한 v1 MVP 스펙.
>
> **갱신 정책**: 마일스톤 별 갱신. 본 문서 갱신 시 관련 (MiniMax.md, PROJECT_PROFILE.md, REFERENCES.md, README.md, development_log.md) 도 함께 align.
>
> **최종 갱신**: 2026-06-07 (v1 draft, 7 reference 분석 후) — **D-24: orchestration framing 제거** (standalone harness tool 로 재확립)

---

## 0. 핵심 Positioning (D-25 교정 후, Mavis zero coupling)

### my_harness 는

- **완전 standalone CLI/TUI coding agent** — `myharness <command>` 로 terminal 에서 직접 실행
- **Harness-first 5 components** (Tools · Context · Session · Plugins · Sub-agents) — Model + Harness = Agent
- **Direct LLM provider 통신** — Anthropic/OpenAI/Google/local Ollama 등 provider API 와 직접 통신
- **Zero external dependency** — Mavis, Mavis, Mavis, mavis-team, standard_ai_workflow, 4-워커 어느 것과도 결합 없음
- **Sibling to claude-code / codex / aider / goose / gemini-cli / opencode** — 7 reference 분석이 동급 comparison

### my_harness 는 **아니다** (NOT)

- ❌ **다른 도구의 오케스트레이션 도구** (orchestrator 가 아님)
- ❌ **Mavis / Mavis / mavis-team / standard_ai_workflow 와 결합된 도구** — yklee 가 Mavis 로 my_harness 를 개발할 수는 있으나 my_harness 자체는 Mavis 와 무관
- ❌ **외부 4-워커(Claude/Codex/Gemini/OpenCode) 운영/통합 도구** — 그 도구들은 sibling 이지 my_harness 의 dispatch 대상 아님
- ❌ **workflow / state management 시스템** — workflow 는 my_harness 의 concern 아님
- ❌ **외부 headroom proxy 의존** — headroom 의 압축 알고리즘은 **built-in 으로** 우리 Context component 에 내장 (D-27). proxy 방식 의존 안 함

### 위치 (Positioning)

```
┌─────────────────────────────────────────┐
│  yklee (user)                            │
└─────────────────────────────────────────┘
              ↓ terminal 직접 호출
┌─────────────────────────────────────────┐
│  my_harness (CLI/TUI)                    │  ← STANDALONE
│  Harness 5 components                    │
│  ┌───────────────────────────────────┐  │
│  │ Context component:                │  │
│  │  - CLAUDE.md, auto memory         │  │
│  │  - built-in compression ⭐        │  │  ← D-27: headroom 알고리즘 built-in
│  │    (CacheAligner, ContentRouter,  │  │     (선택적, user on/off)
│  │     CCR, SmartCrusher, CodeComp,  │  │
│  │     Kompress-base)               │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
              ↓ Direct LLM API call
              (Anthropic/OpenAI/...)
```

**my_harness** = terminal 에서 직접 실행되는 standalone CLI/TUI. **headroom 의 압축 알고리즘은 built-in 으로** 내장 (외부 proxy 의존 X). 사용자가 끄면 native cache 만 사용. 3-도메인 (코드/서버/환경) 작업 전문.

---

---

## 1. 한 줄 Positioning

**my_harness = yklee 의 개인 코딩 에이전트 CLI/TUI** — Mavis / MiniMax Code 런타임 기반, **Harness-first architecture** (Model + 5 components), **3-도메인** (코드/서버/환경) 동시 지원, **standard_ai_workflow + minimax-code** 듀얼 오버레이.

---

## 2. 타겟 사용자

| 대상 | 사용 패턴 |
| --- | --- |
| **yklee (오너, single user)** | terminal 에서 `myharness <command>` 직접 호출. 3-도메인 (코드/서버/환경) 작업 |
| **plugin 개발자 (v2+)** | `~/.myharness/plugins/<name>/` 에 plugin 직접 작성. marketplace 공유 |

**v1 scope = yklee single user, terminal 직접 실행** (multi-user/marketplace 는 v2+)

---

## 3. 핵심 가치 (3가지)

### 3.1 **Harness-first** (claude-code 13.1)
모델보다 **Harness 5 components** (Tools, Context, Session, Plugins, Sub-agents) 가 차별점. claude-code 의 `Agent = Model + Harness` 청사진 차용. my_harness 단독으로 LLM provider 와 직접 통신하여 코드/서버/환경 작업 수행.

### 3.2 **Provider 비종속** (aider/opencode/goose 13.2 + claude-code 13.15)
12+ (Rust 1안 `rig-core`) 또는 15+ (TS 2안 `Vercel AI SDK`) provider + **3 fallback model** (claude-code 패턴). 어떤 provider 든 my_harness 가 직접 통신 — 외부 orchestrator 불필요.

### 3.3 **3-도메인 동시 + 선택적 built-in 압축** (headroom 13.3, D-27)
코드/서버/환경 3-도메인 모두 통합. **headroom 의 6 압축 알고리즘** (CacheAligner, ContentRouter, CCR, SmartCrusher, CodeCompressor, Kompress-base) 을 **우리 Context component 에 built-in** — 외부 proxy 의존 X. `~/.myharness/config.yaml` 에서 `builtin.enabled: true|false`. 기본값은 `false` (사용자가 켜야 동작). 토큰 한계는 native cache 만으로도 v1 가능.

---

## 4. 스코프

### 4.1 In-scope (v1 MVP)

**3-도메인**:
- **코드 개발 전반** — 새 기능 구현, 리팩토링, 버그 수정, 리뷰, 테스트, PR 작업
- **기본 서버 관리** — 프로세스/서비스 상태 점검, 로그 확인, 설정 변경, 배포 헬퍼
- **환경 셋업** — 로컬/원격 개발 환경 부트스트랩, 의존성 설치, 셸/도구 설정

**3-언어 동시** (cross-platform):
- macOS (Intel + Apple Silicon Universal)
- Linux (Debian/Fedora/RHEL/Alpine)
- Windows (PowerShell/CMD, x64/ARM64)

### 4.2 Out-of-scope (v1)

- 5 surfaces cross-session (claude-code 13.2) — v2+ (TUI/IDE/Web hand-off)
- Plugin marketplace community (claude-code 13.3) — v2+
- Computer Use (claude-code 13.23) — v3+
- Routines / scheduled tasks (claude-code 13.17) — v2+
- Channels (Slack/Telegram webhook) — v2+
- Multi-user / RBAC — v3+
- 5 surface 동시 유지 (claude-code 13.36 anti-pattern) — 절대 안 함

---

## 5. v1 MVP 스펙

### 5.1 아키텍처: Harness 5 components ⭐

```
┌─────────────────────────────────────────────┐
│  User Interface (CLI + TUI)                 │  ← v1 CLI + TUI 만
├─────────────────────────────────────────────┤
│  Command & Tool Layer                       │  ← commands + tools
├─────────────────────────────────────────────┤
│  Harness 5 Components                       │
│  ┌───────────────────────────────────────┐  │
│  │ 1. Tools        — Read/Write/Edit/    │  │
│  │                  Bash/Grep/Glob +     │  │
│  │                  plugin tools        │  │
│  │ 2. Context      — CLAUDE.md +        │  │
│  │                  auto memory +       │  │
│  │                  /compact + CCR      │  │
│  │ 3. Session      — local state.json + │  │
│  │                  standard_ai_workflow│  │
│  │ 4. Plugins      — 4계층 (commands/   │  │
│  │                  agents/skills/hooks)│  │
│  │ 5. Sub-agents   — mavis-team worker  │  │
│  │                  pool + Agent SDK    │  │
│  └───────────────────────────────────────┘  │
├─────────────────────────────────────────────┤
│  Query Engine — streaming, tool dispatch,   │
│                 retry, context compression   │
├─────────────────────────────────────────────┤
│  Service Layer — auth, plugins, state,      │
│                  analytics, secret mgmt     │
├─────────────────────────────────────────────┤
│  Infrastructure — filesystem, Git, config,  │
│                   permissions, secure store  │
└─────────────────────────────────────────────┘
                    ↓
        Claude API + 3P providers
        (rig-core 1안 / Vercel AI SDK 2안)
```

**참조**: claude-code.md §2, arxiv 2604.14228

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

**각 명령 = 1 sub-agent (mini_coder_max / fullstack-dev / etc) 위임** — mavis-team 의 worker pool 활용.

### 5.3 설치 / 배포 (claude-code 13.9 + 13.10)

**5 install paths**:
| OS | 권장 | 대안 |
| --- | --- | --- |
| macOS / Linux | `curl -fsSL https://myharness.dev/install.sh \| bash` | brew `--cask myharness` (stable) / `@latest` (bleeding) |
| Windows (PS) | `irm https://myharness.dev/install.ps1 \| iex` | winget `Yklee.Myharness` |
| Linux package | apt / dnf / apk | install.sh |

- **Auto-update**: native install 만 background. brew/winget 수동
- **Stable vs Latest 듀얼 채널** (claude-code 13.10)
- **단일 binary** (Rust 1안 / TS+Bun 2안)

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

### 5.5 LLM 통합 (claude-code 13.15 + aider 13.2, D-28)

**지원 Provider (D-28, yklee 의 5개 + local)**:

| # | Provider | Native SDK | OpenAI 호환 | 권장 | 비고 |
| - | --- | --- | --- | --- | --- |
| 1 | **claude** (Anthropic) | ✅ `anthropic` SDK | (OpenRouter 경유 가능) | native | Sonnet 4.5 / Haiku 4 / Opus 4.5 |
| 2 | **codex** (OpenAI) | ✅ `openai` SDK | — | native | GPT-5 / GPT-5-Codex / GPT-4.1 |
| 3 | **gemini** (Google) | ✅ `google-genai` SDK | (OpenAI 호환 endpoint 제공) | native | Gemini 2.5 Pro / Flash |
| 4 | **deepseek** | — | ✅ (`https://api.deepseek.com/v1`) | OpenAI 호환 | deepseek-chat / deepseek-reasoner |
| 5 | **minimax** | — | ✅ (base_url TBD 검증 필요) | OpenAI 호환 | 모델명/API 형식 검증 필요 (D-28) |
| 6 | **local LLM** | — | ✅ (`http://localhost:11434/v1` 등) | OpenAI 호환 | Ollama / vLLM / LM Studio / llama.cpp server |

**추상화 전략 (OpenAI 호환 = lingua franca, premium = native)**:
- **Premium providers** (claude/codex/gemini) → **native SDK** 사용 (각 vendor 의 최적 기능: prompt cache, thinking, function calling)
- **OpenAI 호환 providers** (deepseek/minimax/local) → **공통 OpenAI 호환 client** 사용 (1개 구현으로 N개 provider 지원)
- **Provider registry** (config.yaml) → 사용자 정의 provider 추가 가능 (v1.5+ plugin)

**Provider config 예시 (D-28)**:
```yaml
# ~/.myharness/config.yaml
llm:
  primary: anthropic/claude-sonnet-4-5       # 도메인 무관 기본
  fallback:                                   # 3 fallback (D-15, claude-code 2.1.166 패턴)
    - openai/gpt-5-codex                      # codex (OpenAI)
    - ollama/qwen2.5-coder:32b                # local (always-on)
  domain_mapping:                             # 도메인별 model (D-15)
    code: anthropic/claude-sonnet-4-5
    server: anthropic/claude-haiku-4
    env: ollama/qwen2.5-coder:32b
  thinking:                                   # claude-code per-model thinking
    code: enabled
    server: disabled
    env: disabled

  providers:
    # 1. claude (Anthropic) — native
    - name: anthropic
      type: native
      sdk: anthropic                          # anthropic SDK
      api_key_env: ANTHROPIC_API_KEY
      secret_store: keychain                  # macOS Keychain / wincred / libsecret
      supports: [prompt_cache, thinking, vision, tool_use]
      models: [claude-sonnet-4-5, claude-haiku-4, claude-opus-4-5]

    # 2. codex (OpenAI) — native
    - name: openai
      type: native
      sdk: openai
      api_key_env: OPENAI_API_KEY
      secret_store: keychain
      supports: [prompt_cache, tool_use, vision]
      models: [gpt-5, gpt-5-codex, gpt-4.1, gpt-4o]

    # 3. gemini (Google) — native
    - name: gemini
      type: native
      sdk: google-genai
      api_key_env: GOOGLE_API_KEY
      secret_store: keychain
      supports: [prompt_cache, thinking, vision, tool_use]
      models: [gemini-2.5-pro, gemini-2.5-flash]

    # 4. deepseek — OpenAI 호환
    - name: deepseek
      type: openai-compatible
      base_url: https://api.deepseek.com/v1
      api_key_env: DEEPSEEK_API_KEY
      secret_store: keychain
      supports: [reasoning, tool_use]
      models: [deepseek-chat, deepseek-reasoner]

    # 5. minimax — OpenAI 호환 (D-28, base_url 검증 필요)
    - name: minimax
      type: openai-compatible
      base_url: <TBD: 사용자 확인 필요>      # base_url + API 형식 검증
      api_key_env: <TBD: MINIMAX_API_KEY?>
      secret_store: keychain
      supports: [TBD]
      models: [TBD]

    # 6. local LLM — OpenAI 호환 (Ollama default)
    - name: local-llm
      type: openai-compatible
      base_url: http://localhost:11434/v1     # Ollama default
      api_key_env: null                       # 보통 key 불필요
      supports: [varies by model]
      models: auto_discover                   # GET /v1/models 로 자동
      # 다른 local server 예시:
      #   vLLM:    http://localhost:8000/v1
      #   LM Studio: http://localhost:1234/v1
      #   llama.cpp: http://localhost:8080/v1
```

**Retry / Fallback 정책 (D-15, claude-code 2.1.166)**:
- primary 호출 실패 시 → fallback 1 → fallback 2 순서 시도
- **즉시 surface** 되는 error: auth, rate_limit, request_size, transport
- **retry-able** error: overloaded, timeout, transient → 1회 fallback retry
- fallback 발동률 모니터링 (KPI §9, v2 목표 <1%)

**Secret 관리 (D-06)**:
- 모든 API key = **macOS Keychain / Windows Credential Manager / Linux Secret Service** (provider config 의 `secret_store: keychain`)
- 환경변수 fallback (CI/CD 한정, token rotation 시)
- 토큰 값은 메모리/문서/git 저장 금지 (D-06 정책)

**Provider 비종속 library 권장 (PROVIDERS.md 상세)**:
- 1안 (Rust) = `rig-core` (Anthropic/OpenAI/Google/Ollama native SDK 추상화) + 자체 OpenAI 호환 client
- 2안 (TS) = `Vercel AI SDK` (15+ provider) + 자체 OpenAI 호환 client
- 두 안 모두 DeepSeek / local LLM 은 OpenAI 호환 client 로 처리 (built-in)
- **minimax** 은 D-28 TBD: base_url + API 검증 후 v1 또는 v1.5 통합

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

### 5.6 Context 관리 (claude-code 13.6 + built-in headroom 13.3, D-27)

**3 계층 + built-in compression**:
1. **`CLAUDE.md` (project root)** — yklee 의 프로젝트별 규칙, 5 surface 공유.
2. **Auto memory** — yklee 의 작업 패턴 자동 학습. `~/.myharness/memory/auto/`
3. **`/compact` slash command** — context 압축. **built-in compression 알고리즘** 호출.
4. **Built-in compression (D-27, 선택적, 기본 off)** — headroom 의 6 알고리즘을 **우리 Context component 에 내장**. 외부 proxy/MCP 의존 X.

**Built-in compression 설계 (D-27, headroom.md §7.4 6 알고리즘 참고)**:

```yaml
# ~/.myharness/config.yaml
context:
  compression: native       # native | builtin (D-27)
  builtin:
    enabled: false          # ← 기본 OFF. 사용자가 true 로 켜면 동작
    algorithms:
      cache_aligner: true   # CacheAligner — prefix 안정화 (KV cache hit)
      content_router: true  # ContentRouter — content type 감지 → 알고리즘 선택
      ccr: false            # CCR — reversible + retrieval (round-trip 비용)
      smart_crusher: true   # SmartCrusher — JSON 구조 보존 압축
      code_compressor: true # CodeCompressor — AST-aware (tree-sitter)
      kompress_base: false  # Kompress-base ML — 자유 텍스트 (95% 압축, ONNX)
    target_ratio: 0.35      # 65% 압축 목표
    protect_recent: 5       # 최근 N 메시지 보호
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

### 5.7 Plugin 시스템 (claude-code 13.3)

**4 계층** (v1.5+):
```
~/.myharness/plugins/<name>/
├── plugin.json           # manifest
├── commands/             # slash commands
├── agents/               # specialized sub-agents
├── skills/               # auto-invoke knowledge
└── hooks/                # event handlers (markdown rule)
```

**v1 MVP**: local plugin only (commands + hooks). marketplace 는 v2+.

### 5.8 외부 의존성 없음 (Zero external dependency)

my_harness 는 다음 어느 것과도 **결합 없음**:
- ❌ Mavis (Mavis) — chat agent
- ❌ Mavis / mavis-team — orchestration engine
- ❌ standard_ai_workflow — Mavis 의 workflow meta layer
- ❌ 4-워커 (Claude/Codex/Gemini/OpenCode) — sibling 일 뿐

**유일한 런타임 의존**:
- LLM provider API (Anthropic / OpenAI / Google / local Ollama) — **직접 통신**
- OS 표준 라이브러리 (filesystem, network, process)
- (선택) headroom MCP server — 사용자 opt-in

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

```yaml
# ~/.myharness/config.yaml (예시)
workflow:
  mode: auto              # auto | none | mavis
  mavis_root: ~/mavis     # Mavis 디렉토리 위치
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

#### 5.9.4 우리 my_harness 의 위치 (재확인)

```
┌─────────────────────────────────────────┐
│  yklee (terminal 직접)                   │
└─────────────────────────────────────────┘
              ↓ `myharness <command>`
┌─────────────────────────────────────────┐
│  my_harness (CLI/TUI)                    │  ← 100% standalone
│  - Harness 5 components                  │
│  - standard_ai_workflow 6 원칙 native   │  ← 한국어/절약/상태/이벤트/비참조/handoff
│  - LLM provider 직접 통신                │
└─────────────────────────────────────────┘
       ↓                          ↓
   LLM provider           .myharness/ (자체)
                              ├── state/         (항상)
                              ├── handoff/       (항상)
                              ├── log.jsonl      (항상)
                              ├── memory/auto/   (항상)
                              └── plugins/       (항상)
                            ↓ (opt) auto-detect
                       ai-workflow/memory/    (Mavis 디렉토리 발견 시만)
```

**핵심**:
- **my_harness = 자체 `.myharness/` 디렉토리 + standard_ai_workflow 6 원칙 native** — Mavis 없어도 동작
- **옵션 Mavis 통합** — auto-detect 로 seamless, 사용자 flag 로 off 가능
- **Mavis 가 my_harness 를 spawn 해도** 동일한 6 원칙 + 자체 디렉토리 + 옵션 통합
- **my_harness 개발 workflow** (이 repo 의 Mavis + standard_ai_workflow) = **my_harness 산출물과 무관** (D-25)

---

## 6. v2+ 로드맵

| milestone | 핵심 | 채택 패턴 |
| --- | --- | --- |
| **v1.0** (MVP) | CLI + TUI, 3-도메인, single binary | 1차 8개 adopt |
| **v1.5** | Plugin 4-계층, marketplace beta, auto memory | 2차 7개 adopt |
| **v2.0** | TUI/IDE/Web hand-off (5 surfaces), Routines | claude-code 13.2 + 13.17 |
| **v2.5** | Multi-agent parallel + confidence scoring | claude-code 13.11 (code-review) |
| **v3.0** | Computer Use, Multi-user, RBAC | claude-code 13.23 + 13.34 |

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

## 8. 안티 패턴 (6개, 절대 안 함)

1. **closed source + leak 의존** (claude-code 13.27) → **MIT/Apache 2.0 (open)**
2. **듀얼 언어** (headroom 13.15) → **단일 언어** (Rust 1안 OR TS 2안)
3. **100+ slash commands** (claude-code 13.30) → **3-도메인 × 3-4 명령** = ~12 명령 max
4. **5 surface 동시 유지** (claude-code 13.36) → **v1 CLI+TUI only**, 점진 확장
5. **cloud auto memory privacy** (claude-code 13.37) → **v1 local-only**, v2+ opt-in cloud
6. **subscription requirement** (claude-code 13.34) → **CLI free**, v2+ premium 검토

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
| **Provider API 변경** | 호환성 깨짐 | rig-core / Vercel AI SDK 의 abstraction layer 활용 |
| **headroom API 변경** | CCR integration 깨짐 | MCP server 노출 → 우리 쪽만 갱신 |
| **5 surface 점진 확장 시 maintenance 부담** | bug fix 5x | v1 CLI+TUI 만, v2 부터 surface 추가 시 별도 sprint |
| **plugin 생태계 부재** (v1 local only) | 확장성 제한 | v1.5 부터 marketplace beta |
| **3 fallback 의 provider 가용성** | fallback 발동 시 지연 | fallback 도 동일 abstraction 위에서 |
| **Local-only memory** (privacy) | cross-device 사용 불가 | v2+ opt-in cloud with encryption |

---

## 11. 결정 보류 (Open Decisions)

| 결정 | 보류 이유 | 결정 시점 |
| --- | --- | --- |
| **TASK-005 스택** (Rust 1안 vs TS 2안) | yklee 의 desktop 우선순위 | 즉시 결정 가능 (입력 다 갖춤) |
| **TASK-002 도메인별 명령** | yklee 인프라 정보 필요 | yklee 인프라 정보 수령 후 |
| **TASK-007 headroom built-in 알고리즘 구현 우선순위** | CacheAligner/ContentRouter/CCR/SmartCrusher/CodeCompressor/Kompress-base 중 v1 우선 3개 | yklee 가 v1 우선순위 결정 후 |
| **TUI 라이브러리** (ratatui vs React/Ink) | 스택 결정 의존 | TASK-005 결정 후 |
| **Provider fallback list** (3 모델) | yklee 의 LLM 선호/비용 | yklee 결정 후 |

---

## 12. 참고 (References)

- [REFERENCES.md §5 우리 방향성 초안](./REFERENCES.md) — 8축 매트릭스 → 본 컨셉 §5 정합
- [references/README.md](./references/README.md) — 7-doc 통합 인덱스 + cross-review
- [references/claude-code.md](./references/claude-code.md) — Harness 5 components, plugin 4-계층, 5 surfaces
- [references/headroom.md](./references/headroom.md) — CCR, CacheAligner, ContentRouter
- [references/PROVIDERS.md](./references/PROVIDERS.md) — rig-core 1안 / Vercel AI SDK 2안 / litellm proxy
- [development_log.md](./development_log.md) — 본 컨셉 확립까지의 결정 이력 (D-01 ~ D-23)
- [PROJECT_PROFILE.md](./PROJECT_PROFILE.md) — 워크플로우 통합
- [MiniMax.md](../MiniMax.md) — Mavis 진입점
