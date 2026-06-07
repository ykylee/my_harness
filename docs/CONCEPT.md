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
- ❌ **필수 headroom 통합** — `headroom` 은 **선택적 (optional) MCP server**, 사용자가 켜고 끔

### 위치 (Positioning)

```
┌─────────────────────────────────────────┐
│  yklee (user)                            │
└─────────────────────────────────────────┘
              ↓ terminal 직접 호출
              ↓ (또는 Mavis 가 spawn — Mavis 와 zero coupling)
┌─────────────────────────────────────────┐
│  my_harness (CLI/TUI)                    │  ← STANDALONE
│  Harness 5 components                    │
└─────────────────────────────────────────┘
              ↓ Direct LLM API call
              (Anthropic/OpenAI/...)
              ↓ (optional, user on/off)
              headroom MCP server
```

**my_harness** = terminal 에서 직접 실행되는 standalone CLI/TUI. Mavis / Mavis / 4-워커 어느 것과도 무관. 3-도메인 (코드/서버/환경) 작업 전문.

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

### 3.3 **3-도메인 동시 + 선택적 CCR** (headroom 13.3)
코드/서버/환경 3-도메인 모두 통합. **CCR (Context Cache Reduction)** 는 **선택적 (optional) MCP server** 통합 — `~/.myharness/config.yaml` 에서 `headroom.enabled: true|false`. 기본값은 `false` (사용자가 켜야 동작). 토큰 한계는 CCR 없이도 provider 의 native cache 로 대응 가능.

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

### 5.5 LLM 통합 (claude-code 13.15 + aider 13.2)

**Provider 비종속**:
- 1안 (Rust) = `rig-core` (12+ provider)
- 2안 (TS) = `Vercel AI SDK` (15+ provider)
- Cross-runtime fallback = `litellm proxy`

**3 fallback model** (claude-code 2.1.166 패턴):
```yaml
# ~/.myharness/config.yaml
llm:
  primary: claude-sonnet-4-5
  fallback:
    - claude-haiku-4
    - ollama/qwen2.5-coder
  thinking:
    code: enabled      # Sonnet
    server: disabled   # Haiku
    env: disabled      # local
```

**도메인별 model 자동 매핑** (claude-code per-model thinking 패턴):
- 코드 = Sonnet 4.5 + thinking
- 서버 = Haiku 4 + no thinking
- 환경 = local Ollama + no thinking

### 5.6 Context 관리 (claude-code 13.6 + 선택적 headroom 13.3)

**3 계층**:
1. **`CLAUDE.md` (project root)** — yklee 의 프로젝트별 규칙, 5 surface 공유. 우리 v1 = `MiniMax.md` 가 이미 동급 (Mavis 의 메타 진입점).
2. **Auto memory** — yklee 의 작업 패턴 자동 학습. `~/.myharness/memory/auto/`
3. **`/compact` slash command** — context 압축. **선택적 headroom MCP server** integration (사용자 on/off).

**CCR integration (선택적, 기본 off)**:
```yaml
# ~/.myharness/config.yaml
context:
  compression: native   # native (provider cache only) | headroom-mcp
  headroom:
    enabled: false      # ← 기본 OFF. 사용자가 true 로 켜면 동작
    mode: token         # token | cache | ccr
    target_ratio: 0.35
    mcp_server: headroom
```

- **native 모드 (기본)**: provider native cache (Anthropic prompt cache, OpenAI cached prompt) 만 사용
- **headroom-mcp 모드 (opt-in)**: 우리 my_harness 가 headroom MCP server 호출. `mcp__headroom__compress(messages, model)` — 65-95% 토큰 절감. `mcp__headroom__retrieve(id)` — 원문 복원 (CCR mode 시)

→ **v1 부터 양 모드 지원**, 사용자가 선택. CCR 은 v1 핵심 ❌ (anti-pattern 회피).

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
| **TASK-007 headroom 통합** (선택적 MCP) | 사용자 opt-in 방식 + MCP server 사용 가능 여부 | MCP server 검증 후 |
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
