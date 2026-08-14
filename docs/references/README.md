# Reference Analysis Index & Cross-Review (8 docs)

> **용도**: 8개 reference 분석 (opencode · aider · codex · goose · gemini-cli · headroom · claude-code · **grok-build**) 의 인덱스 + 통합 리뷰. 보고서 "레퍼런스 분석" 섹션 작성용 백데이터.
>
> **갱신 정책**: 새 reference 분석 추가 시 §1 인벤토리 + §2 비교 매트릭스 + §3 my_harness 영향 분석 갱신.
> **2026-08-14**: grok-build 8번째 추가. 2차 심층 14섹션 [grok-build.md](./grok-build.md).

---

## 1. 인벤토리 (Inventory)

| # | repo | GitHub | Gitea | 분석 doc | LOC (참고치) | 라이선스 | 1차 결론 |
| - | --- | --- | --- | --- | --- | --- | --- |
| 1 | **opencode** | sst/opencode | yklee/opencode | [opencode.md](./opencode.md) | 23,682 | MIT | terminal-native, TUI, multi-model |
| 2 | **aider** | Aider-AI/aider | yklee/aider | [aider.md](./aider.md) | 92,570 | Apache 2.0 | git-first, repo 맵, LLM 비종속 |
| 3 | **codex** | openai/codex | yklee/codex | [codex.md](./codex.md) | 94,233 | Apache 2.0 | sandbox isolated, training-grade Rust |
| 4 | **goose** | block/goose | yklee/goose | [goose.md](./goose.md) | 60,045 | Apache 2.0 | local-first, multi-LLM, recipe system |
| 5 | **gemini-cli** | google-gemini/gemini-cli | yklee/gemini-cli | [gemini-cli.md](./gemini-cli.md) | 21,074 | Apache 2.0 | Google integration, OAuth |
| 6 | **headroom** | chopratejas/headroom | yklee/headroom | [headroom.md](./headroom.md) | 42,496 | Apache 2.0 | **context compression layer** ⭐ |
| 7 | **claude-code** | anthropics/claude-code | yklee/claude-code | [claude-code.md](./claude-code.md) | 1,029 (analyzed) | **Anthropic Commercial** (closed) | **harness-first 5 components** ⭐ |
| 8 | **grok-build** | xai-org/grok-build | (미미러, `~/repos/grok-build`) | [grok-build.md](./grok-build.md) | 1,362,619 `*.rs` (test 포함) | Apache 2.0 | **완성된 5-component 제품. 포크 비권장, overlay 권장** ⭐ |

**합계**: 8 docs. grok-build 는 2026-07-15 Apache 2.0 오픈소스. 외부 PR 거부, 모노레포 주기 sync.

---

## 2. 8축 비교 매트릭스 (Comparative Matrix)

> 14섹션 분석의 §1~§14 통합. 한 축 = 한 설계 결정.

### 축 1: 언어 / 런타임

| # | 도구 | 언어 | runtime | 단일 binary |
| - | --- | --- | --- | --- |
| 1 | opencode | TypeScript | Bun | ❌ |
| 2 | aider | Python | Python 3.10+ | ❌ |
| 3 | codex | Rust | tokio | ✅ |
| 4 | goose | Rust + TS | tokio + Node (Electron desktop) | 부분 |
| 5 | gemini-cli | TypeScript | Node.js 20+ | ❌ |
| 6 | headroom | Python + Rust + TS | polyglot | 부분 |
| 7 | claude-code | TypeScript (추정, closed) | Node.js 18+ + native binary | ✅ |
| 8 | grok-build | Rust 1.92 | tokio + ratatui | ✅ (`grok`) |

**우리 my_harness 권장**:
- **1안 Rust** — codex/goose/grok-build 와 같은 (단일 binary, 빠른 startup, low memory)
- **2안 TypeScript** — opencode/gemini-cli 와 같은 (Bun runtime, 빠른 dev cycle)
- **2026-08-14**: 자체 Rust 재구현보다 grok-build overlay (래퍼+plugin) 가 더 짧다. 독립 런타임이 필요하면 여전히 goose 포크.

### 축 2: LLM 통합

| # | 도구 | provider | fallback | thinking |
| - | --- | --- | --- | --- |
| 1 | opencode | multi-model (Claude/GPT/Gemini/+) | ❌ | ❌ |
| 2 | aider | multi-LLM (anthropic/openai/...) | ❌ | ❌ |
| 3 | codex | OpenAI only | ❌ | ❌ |
| 4 | goose | multi-LLM (anthropic/openai/databricks/ollama) | ❌ | ❌ |
| 5 | gemini-cli | Gemini only | ❌ | ❌ |
| 6 | headroom | transparent middleware (provider 비종속) | ❌ | ❌ |
| 7 | claude-code | Claude + 3P (3rd-party integrations) | **✅ 3 in order** | **✅ per-model** |
| 8 | grok-build | xAI 기본 + **custom models** (chat_completions / responses / messages) | 모델별 `[model.*]` | ✅ reasoning effort |

**우리 my_harness 권장**:
- **rig-core (1안) / Vercel AI SDK (2안)** — 12+/15+ provider
- **claude-code 패턴**: fallbackModel 3 in order + per-model thinking toggle (TASK-005 결정 시)

### 축 3: TUI / Frontend

| # | 도구 | TUI | IDE | Web | Desktop |
| - | --- | --- | --- | --- | --- |
| 1 | opencode | ✅ (Bubble Tea-ish) | ❌ | ❌ | ❌ |
| 2 | aider | terminal (prompt_toolkit) | ❌ | ❌ | ❌ |
| 3 | codex | ✅ TUI (Rust, ratatui) | ❌ | ❌ | ❌ |
| 4 | goose | TUI + Desktop (Electron) | ❌ | ❌ | ✅ |
| 5 | gemini-cli | terminal (Ink) | ❌ | ❌ | ❌ |
| 6 | headroom | CLI + Proxy daemon | ❌ | ❌ | ❌ |
| 7 | claude-code | ✅ TUI (React/Ink) | **✅ VS Code** | **✅ Web** | **✅ Desktop** |
| 8 | grok-build | ✅ 풀스크린 ratatui + 마우스 | ACP (`grok agent`) | ❌ | ❌ |

**우리 my_harness 권장**:
- **v1**: CLI + TUI (claude-code 의 cross-surface 는 v2+)
- **2안 TS** 가 TUI/UI 1안 (React/Ink) 와 자연스러움
- 1안 Rust 는 ratatui 가 안정적 (codex 와 같은)

### 축 4: Plugin / Extension 시스템

| # | 도구 | plugin system | complexity | 표준 manifest |
| - | --- | --- | --- | --- |
| 1 | opencode | minimal (config-driven) | low | ❌ |
| 2 | aider | minimal (.aider.conf.yml) | low | ❌ |
| 3 | codex | minimal | low | ❌ |
| 4 | goose | **recipe system** (Goose recipes) | medium | ✅ (yaml) |
| 5 | gemini-cli | **extensions** (TOML, MCP, skills) | **high** | ✅ |
| 6 | headroom | ❌ (PR + 머지 방식) | minimal | ❌ |
| 7 | claude-code | **4-계층 (commands/agents/skills/hooks) + marketplace** | **very high** | ✅ (plugin.json) |
| 8 | grok-build | **동일 4-계층 + marketplace + MCP + ACP** (이미 구현) | **very high** | ✅ (plugin.json, 선택) |

**우리 my_harness 권장**:
- **v1**: minimal (도메인별 명령 inline) + mavis-team skills
- **v2+**: claude-code 의 4-계층 + marketplace (TASK-007 이후)
- gemini-cli 의 TOML manifest 도 가벼운 대안

### 축 5: Context 관리

| # | 도구 | CLAUDE.md 류 | auto memory | compression | MCP |
| - | --- | --- | --- | --- | --- |
| 1 | opencode | AGENTS.md / agents.md | ❌ | ❌ | ✅ (limited) |
| 2 | aider | CONVENTIONS.md / .aider.conf.yml | ❌ | ❌ | ❌ |
| 3 | codex | AGENTS.md | ❌ | ❌ | ❌ |
| 4 | goose | recipe yaml | partial | ❌ | ✅ (recipe+tool) |
| 5 | gemini-cli | GEMINI.md | ❌ | ❌ | ✅ (extensions) |
| 6 | headroom | n/a (middleware) | n/a | **✅ 6 algorithms + CCR** | n/a |
| 7 | claude-code | **CLAUDE.md + auto memory + /compact** | **✅** | **✅** | **✅ first-class** |
| 8 | grok-build | AGENTS.md / CLAUDE.md / `.grok/rules` + auto-compact | experimental | ✅ `xai-grok-compaction` | ✅ rmcp 2.1 |

**우리 my_harness 권장**:
- **CLAUDE.md 표준** (claude-code 패턴, 이미 MiniMax.md 가 동급)
- **Auto memory** (claude-code 13.5)
- **headroom CCR** (TASK-007) — context compression layer
- **MCP server** (claude-code 13.24) — 우리 1안
- **/compact slash command** (claude-code 13.7)

### 축 6: Cross-surface / Session

| # | 도구 | cross-surface | session model |
| - | --- | --- | --- |
| 1 | opencode | ❌ | local |
| 2 | aider | ❌ | local |
| 3 | codex | ❌ | local + sandbox |
| 4 | goose | ❌ (desktop 별도) | local |
| 5 | gemini-cli | ❌ | local |
| 6 | headroom | ❌ (proxy daemon) | n/a |
| 7 | claude-code | **✅ 5 surfaces cross-session** | **cross-surface state** |
| 8 | grok-build | TUI + headless + ACP | `~/.grok/sessions/` JSONL |

**우리 my_harness 권장**:
- **v1**: local session (`state.json` + journal) — standard_ai_workflow 와 결합
- **v2+**: cross-surface (TASK-007)

### 축 7: 보안 / 권한

| # | 도구 | permission mode | hook system | 3-tier security |
| - | --- | --- | --- | --- |
| 1 | opencode | simple | ❌ | ❌ |
| 2 | aider | simple (auto-commit) | ❌ | ❌ |
| 3 | codex | **sandbox isolated** (OS-level) | ❌ | ❌ |
| 4 | goose | simple | ❌ | ❌ |
| 5 | gemini-cli | simple + sandbox | ❌ | ❌ |
| 6 | headroom | n/a | ❌ | n/a |
| 7 | claude-code | **4 mode** | **✅ 85+ hooks** | **✅ pattern + LLM + agentic** |
| 8 | grok-build | **5 mode** + OS sandbox 프로필 | ✅ PreToolUse deny / Stop gate | folder-trust + Landlock/Seatbelt |

**우리 my_harness 권장**:
- **v1**: claude-code 4 mode 차용 (default/acceptEdits/plan/bypassPermissions)
- **v1 hook system**: `.myharness/hooks/*.md` markdown rule (claude-code 13.4 hookify)
- **v2+ security plugin**: security-guidance 3-tier (claude-code 13.13)

### 축 8: Distribution / Install

| # | 도구 | 단일 binary | install script | package manager | cross-platform |
| - | --- | --- | --- | --- | --- |
| 1 | opencode | ❌ (Bun) | install.sh | npm, brew | mac/linux/win |
| 2 | aider | ❌ (PyPI) | uv/pip | brew, pipx | mac/linux/win |
| 3 | codex | ✅ Rust | install.sh | brew, npm, cargo | mac/linux/win |
| 4 | goose | ✅ Rust (CLI) + Electron (Desktop) | install.sh | brew, npm | mac/linux/win |
| 5 | gemini-cli | ❌ (npm) | install.sh | npm | mac/linux/win |
| 6 | headroom | ✅ Python+Rust+TS | pip/npm | brew, pip | mac/linux/win |
| 7 | claude-code | ✅ (추정) | **5 native paths** | brew, winget, apt/dnf/apk | **5 surfaces** |
| 8 | grok-build | ✅ | install.sh / install.ps1 / `grok update` | 공식 채널 | mac/linux/win |

**우리 my_harness 권장**:
- **claude-code 5 install paths 패턴** (install.sh / install.ps1 / brew / winget / linux pkg)
- **stable/latest 듀얼 채널**

---

## 3. my_harness 에 미치는 핵심 영향 (Key Implications)

### 3.1 TASK-005 스택 결정 (Rust 1안 vs TS 2안)

**4개 reference 가 Rust** (codex, goose, grok-build, claude-code 추정), **4개 reference 가 TypeScript** (opencode, gemini-cli, headroom 부분, claude-code 추정). 결론: **양쪽 모두 viable**. 우리 결정 기준:

| 기준 | Rust 1안 (codex/goose) | TS 2안 (opencode/gemini-cli) |
| --- | --- | --- |
| **단일 binary** | ✅ (codex, goose) | ❌ (Bun/Node 필요) |
| **TUI/UI 생태계** | ratatui (안정) | React/Ink (풍부) |
| **Provider 비종속** | rig-core (12+) | Vercel AI SDK (15+) |
| **Tauri/Electron desktop** | Tauri (small bundle) | Electron (heavier) |
| **CI/release toolchain** | cargo + cross | bun + electron-builder |
| **mavis-team 통합** | spawn rust binary (쉬움) | spawn node binary (쉬움) |

**권장**:
- 1안 = **Rust** (단일 binary + ratatui + Tauri) — codex/goose/grok-build 와 같은 안정성
- 2안 = **TypeScript** (Bun + React/Ink + Electron/Tauri) — opencode/gemini-cli 와 같은 dev 속도
- **최종 결정은 yklee 의 도메인별 우선순위** (코드 > 서버 > 환경 → 1안 / 셋업 속도 > 1안 / desktop 중요 → 2안)
- **2026-08-14 (grok-build)**: CONCEPT 5 components 가 grok-build 에 이미 구현됨. 자체 재구현 / 소스 포크보다 **overlay (래퍼 + plugin)** 가 기본 경로. 독립 런타임이 필요하면 goose 포크.

### 3.2 TASK-002 도메인별 명령 (코드/서버/환경)

각 도메인에 가장 적합한 reference 의 패턴 차용:

| 도메인 | 권장 reference | 차용 패턴 |
| --- | --- | --- |
| **코드 개발** | claude-code (13.1 harness) | 5 component harness, plugin 4-계층, CLAUDE.md |
| **서버 관리** | goose (recipe system) | recipe yaml + provider 비종속 + multi-LLM |
| **환경 셋업** | opencode / headroom | transparent middleware + CCR + cache |

→ **도메인별 sub-architecture** 가 다름. 우리 mavis-team 의 도메인 worker 와 1:1 매핑 검토.

### 3.3 TASK-007 headroom 통합 (library/proxy/MCP)

headroom 의 3-모드 (D-13) — 우리 my_harness 통합:

| 모드 | 우리 1안 | 우리 2안 | 비고 |
| --- | --- | --- | --- |
| **library** | `from headroom import compress` 인라인 | 동일 | 우리 Python 통합 시 |
| **proxy** | 우리 my_harness 가 OpenAI 호환 client 의 base_url | 동일 | zero code change |
| **MCP** | 우리 my_harness 가 MCP server 로 `mcp__headroom__compress` 노출 | 동일 | **v1 권장** |

**권장**: **MCP server 모드** — 우리 1안 (D-13) 와 일치, 우리 mavis-team 의 mcp__* 와 자연 통합.

### 3.4 plugin 시스템 (claude-code 4-계층)

가장 성숙: **claude-code 4-계층 (commands/agents/skills/hooks)**.
- **v1**: `~/.myharness/commands/`, `~/.myharness/agents/`, `~/.myharness/skills/`, `~/.myharness/hooks/`
- **manifest**: `plugin.json` (claude-code 와 같은 형식)
- **v2+**: marketplace

### 3.5 TUI (claude-code 13.1, codex ratatui, opencode 1안)

- **1안 Rust**: ratatui (codex 와 같은)
- **2안 TS**: React/Ink (claude-code 와 같은)
- 우리 1안 (Rust) 의 TUI 라이브러리는 **ratatui** 채택 검토

### 3.6 Cross-platform 빌드 (claude-code 5 install paths)

**표준 install matrix**:
- macOS / Linux / WSL: `curl ... | bash`
- Windows PowerShell: `irm ... | iex`
- Windows CMD: `curl ... | install.cmd && del install.cmd`
- Homebrew: `brew install --cask <name>` (stable) / `<name>@latest` (bleeding)
- WinGet: `winget install <vendor>.<name>`
- Linux package: apt / dnf / apk
- npm: deprecated fallback

---

## 4. 우리 my_harness 가 차야 할 패턴 종합 (Adopt 요약)

7 docs 의 §13 Notable Patterns 통합:

### 최우선 (1차 MVP)
1. **Harness 5 components** (claude-code 13.1, arxiv) — Tools/Context/Session/Plugins/Sub-agents
2. **CLAUDE.md 표준** (claude-code 13.6) — 우리 `MiniMax.md` 가 이미 동급
3. **Hook markdown rule** (claude-code 13.4 hookify) — `~/.myharness/hooks/*.md`
4. **4 permission mode** (claude-code 13.8) — default/acceptEdits/plan/bypassPermissions
5. **3 fallback model** (claude-code 13.15) — primary + 2 fallback
6. **5 install paths** (claude-code 13.9) — install.sh/ps1/brew/winget/linux pkg
7. **CCR** (headroom 13.3) — reversible context compression
8. **Provider 비종속** (aider/opencode/goose 13.2) — rig-core / Vercel AI SDK

### 2차 (v1.5)
9. **Plugin 4-계층** (claude-code 13.3) — commands/agents/skills/hooks
10. **Auto memory** (claude-code 13.5) — cross-session 학습
11. **/compact slash command** (claude-code 13.7) — user-callable compression
12. **MCP server 1안** (claude-code 13.24, headroom 13.1) — first-class MCP
13. **Sub-agents + Agent SDK** (claude-code 13.22) — orchestrator
14. **CacheAligner** (headroom 13.5) — KV cache 친화
15. **ContentRouter** (headroom 13.4) — 도메인 × 타입 dispatch

### 3차 (v2+)
16. **5 surfaces cross-surface** (claude-code 13.2) — CLI/VS Code/Desktop/Web/JetBrains
17. **Plugin marketplace** (claude-code 13.3) — community plugins
18. **Routines** (claude-code 13.17) — scheduled + trigger
19. **Multi-agent parallel + confidence scoring** (claude-code 13.11, code-review plugin)
20. **Channels** (claude-code 13.25) — Telegram/Discord/iMessage webhook
21. **Security 3-tier** (claude-code 13.13) — pattern + LLM + agentic
22. **Cross-session security** (claude-code 13.14) — no user authority carry
23. **Computer Use** (claude-code 13.23) — GUI 자동화 (v3)

### 피해야 할 패턴 (Anti)
- **closed source + leak** (claude-code 13.27) → 우리 MIT/Apache
- **듀얼 언어** (headroom 13.15) → 단일 언어
- **100+ slash commands** (claude-code 13.30) → 우리 3-도메인 × 3-4 명령
- **5 surface maintenance 부담** (claude-code 13.36) → 점진 확장
- **subscription requirement** (claude-code 13.34) → 우리 CLI free
- **Cloud auto memory privacy** (claude-code 13.37) → 우리 local-only v1

---

## 5. 7 docs 의 1줄 결론 (한눈에)

| # | 도구 | 우리 my_harness 에게 주는 1줄 |
| - | --- | --- |
| 1 | **opencode** | "terminal-native, multi-model 의 가벼운 TUI" — 우리 2안 reference |
| 2 | **aider** | "git-first, repo 맵, LLM 비종속 의 Python 정석" — 우리 provider 비종속 정석 |
| 3 | **codex** | "sandbox isolated, training-grade Rust 단일 binary" — 우리 1안 reference |
| 4 | **goose** | "local-first, multi-LLM, recipe system" — 우리 서버/환경 도메인 reference |
| 5 | **gemini-cli** | "Google integration, OAuth, extensions" — 우리 OAuth/MCP reference |
| 6 | **headroom** | "transparent context compression middleware" — **우리 토큰 한계 해결 1순위** |
| 7 | **claude-code** | "harness-first 5 components, 5 surfaces, plugin marketplace" — **우리 architecture 청사진** |

---

## 6. 다음 작업 (Next Steps)

1. **TASK-005 결정** (Rust 1안 vs TS 2안) — 본 인덱스 §3.1 입력 활용
2. **TASK-002 도메인별 명령** — §3.2 의 3-도메인 × 1-1 reference 매핑
3. **TASK-007 headroom 통합** (MCP server 1안) — §3.3
4. **plugin 시스템 v1** — §3.4 (claude-code 4-계층 차용)
5. **TUI 라이브러리 결정** — §3.5 (ratatui 1안 / React/Ink 2안)
6. **Cross-platform 빌드** — §3.6 (5 install paths)
7. **본 인덱스 §4 의 우선순위 1차 항목** 8개 → MVP v1 spec
