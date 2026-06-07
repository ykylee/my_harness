# Claude Code 심층 분석 (anthropics/claude-code)

> **분석 소스 명시** (중요): 본 문서는 **closed source** 분석. 다음 출처 결합:
> 1. **anthropics/claude-code repo** 의 inspectable 부분: README.md, CHANGELOG.md (4,263줄), 12개 plugin 소스 (Python/Sh/TS), plugins/README.md, examples/, scripts/
> 2. **공식 docs**: `code.claude.com/docs` (overview, memory, skills, hooks, sub-agents, MCP, agent-sdk, channels, routines, etc.)
> 3. **공개 분석 자료**: arxiv 2604.14228 "Dive into Claude Code" (VILA-Lab), Zain Hasan 의 "Inside Claude Code" blog, CSDN Harness architecture 분석, Reddit r/ClaudeAI 의 leak analysis (1,900 files / 512K LOC TS leaked 2026-03-31)
> 4. **leak 분석 (보조)**: 2026-03-31 leaked source map 기반 reverse engineering. Anthropic 비공인이나 다수 분석가가 cross-reference.
>
> "❓ leak 기반" 항목은 §14 Open Questions 에 격리.

- **문서 목적**: `my_harness` CLI/TUI 의 harness / context / plugin / multi-agent 레이어 설계에 직접 활용 가능한 인사이트 도출
- **범위**: anthropics/claude-code 의 inspectable repo + 공식 docs + 공개 분석. 14섹션 표준 템플릿 + claude-code 특화 부가 분석
- **대상 독자**: yklee, Mavis, TASK-005/007 디자인 리뷰 참여자, 이후 my_harness harness 작업자
- **상태**: complete (1차)
- **최종 수정일**: 2026-06-07
- **관련 문서**: [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [REFERENCES.md](../REFERENCES.md), 그리고 5개 sibling 심층분석 (codex/aider/goose/opencode/gemini-cli/headroom)
- **핵심 결론 (1줄)**: Claude Code 는 "Agent = Model + Harness" 의 가장 정제된 구현. **Harness** = 5 components (Tools/Context/Session/Plugins/Sub-agents), **5 surfaces** (CLI/VS Code/Desktop/Web/JetBrains) cross-surface, **plugin 시스템** (12 official + marketplace) 이 핵심. 우리 my_harness 의 아키텍처 청사진으로 가장 가까움.

---

## 1. 개요 (Overview)

### 1.1 한 줄 요약

**Claude Code** 는 Anthropic 의 **agentic coding tool** 로, terminal · IDE · desktop · web · JetBrains 5개 surface 에서 작동하는 **Claude 모델 harness**. **Harness = Model + (Tools, Context, Session, Plugins, Sub-agents)** 의 5 component architecture. **"Agent = Model + Harness"** 의 가장 정제된 production 구현.

### 1.2 무엇이 다른가 (vs 다른 코딩 도구)

다른 CLI 코딩 에이전트 (aider, opencode, codex, goose, gemini-cli) 와 본질적 차이:
- **Harness-first architecture**: 모델 자체보다 **모델을 둘러싼 5 components** 가 핵심 차별점. "Competition 의 차별화 축이 Model 에서 Harness 로 이동" (CSDN 분석)
- **5 surfaces cross-surface**: 한 세션이 terminal ↔ IDE ↔ web ↔ mobile ↔ iOS 에서 끊김 없이 이어짐 (`claude --teleport`, Remote Control, Dispatch)
- **Plugin marketplace**: 12 official + community marketplace. plugin/agent/hook/skill 4 계층
- **Auto memory**: CLAUDE.md 외에 **auto memory** (Claude 가 자동 축적) — yklee 가 안 적어도 학습 내용 저장
- **Sub-agents + Agent SDK**: 부모 agent 가 자식 agent spawn + 완전 customize 가능한 SDK

### 1.3 라이선스

**Anthropic Commercial Terms of Service** (LICENSE.md). **Closed source**. source code 는 Anthropic 내부만.
- **단**, 2026-03-31 leaked source map (1,900 files / 512K LOC TS) 이 공개됨 — 다수 reverse engineering 분석 존재
- plugin 코드 (12개) 는 MIT (Anthropic 작성, repo 내)
- 1차 분석은 plugin 코드 + 공식 docs + leak 분석 (보조 출처 명시)

### 1.4 사용자 / 대상

- **풀스택 개발자** — terminal · IDE · mobile 모든 surface
- **팀 단위** — plugin marketplace, 슬랙 통합, shared CLAUDE.md
- **DevOps** — CI/CD (GitHub Actions / GitLab CI), scheduled routines
- **Power user** — Agent SDK 로 자체 agent 구축

### 1.5 현재 상태 (2026-06-07)

- **version**: 2.1.168 (2026-06-07 기준, CHANGELOG 4,263줄)
- **release cadence**: ~3-5 releases/week (CHANGELOG 헤더 통계)
- **Code w/Claude 2026 conference**: 2026-05-06, multi-agent · Computer Use · Routines 발표
- **API call 성장**: Claude Code API 사용량 **17x YoY** (Code w/Claude 2026 발표)

### 1.6 핵심 metric

- **5 surfaces**: Terminal (CLI), VS Code, Desktop app, Web (claude.ai/code), JetBrains
- **12 official plugins**: agent-sdk-dev, claude-opus-4-5-migration, code-review, commit-commands, explanatory-output-style, feature-dev, frontend-design, hookify, learning-output-style, plugin-dev, pr-review-toolkit, ralph-wiggum, security-guidance
- **5 install paths**: install.sh, install.ps1, install.cmd, brew cask, winget, npm (deprecated), apt/dnf/apk
- **100+ slash commands** (leak 분석 추정), **85+ hooks**, **146 UI components**, **330+ utils** (leak 분석)
- **CLAUDE.md standard**: Anthropic 통계상 2025년 한해 모든 repo 에 적힌 CLAUDE.md 가 README.md 총합보다 많음 (CMC 인증 농담)

---

## 2. 아키텍처 (Architecture)

### 2.1 한 줄: Agent = Model + Harness

**핵심 공식**: `Agent = Model + Harness` — Model 은 추론 엔진, Harness 는 모델을 둘러싼 5 components.

### 2.2 Harness 의 5 components (CSDN/arxiv 분석 종합)

| Component | 역할 | 비고 |
| --- | --- | --- |
| **Tools** | 모델의 손발 — Read, Write, Edit, Bash, Grep, Glob | plugin 으로 확장 |
| **Context** | 모델의 기억 — CLAUDE.md, system prompt, history, tool defs | auto memory + /compact |
| **Session** | 세션 영속 + cross-surface | claude.ai/code, --teleport |
| **Plugins** | 확장 — slash commands, agents, hooks, MCP servers | 12 official + marketplace |
| **Sub-agents** | 다중 agent 조율 | sub-agents, background agents, Agent SDK |

### 2.3 계층 (leak 분석, arxiv 2604.14228)

```
┌─────────────────────────────────────┐
│  User Interface (5 surfaces)        │  ← Terminal / VS Code / Desktop / Web / JetBrains
├─────────────────────────────────────┤
│  Command & Tool Layer               │  ← 100+ slash commands, 85+ hooks, tools registry
├─────────────────────────────────────┤
│  Query Processing Engine            │  ← streaming, tool dispatch, retry, context compression
├─────────────────────────────────────┤
│  Service Layer                      │  ← auth, plugins, state, analytics
├─────────────────────────────────────┤
│  Infrastructure                     │  ← filesystem, Git, config, permissions, secure storage
└─────────────────────────────────────┘
            ↓
        Claude API (Anthropic + 3P providers)
```

### 2.4 Code 디렉토리 구조 (leak 분석 추정)

- `main entry` — 1개
- `query engine` — streaming LLM call, tool dispatch, retry
- `tools registry` — Read/Write/Edit/Bash/Grep/Glob + plugin tools
- `100+ slash commands` — built-in user commands
- `146 UI components` — Ink/React 기반 TUI
- `85+ hooks` — pre/post event handlers
- `330+ utils` — helpers
- `multi-agent coordinator` — sub-agents
- `remote control` — claude.ai/code bridge
- `task system` — todo/state
- `migration system` — version upgrade

### 2.5 design philosophy 차이

| 도구 | 핵심 |
| --- | --- |
| **Aider** | git-first, repo 맵 |
| **OpenCode** | terminal-native, multi-model |
| **Codex** | sandbox isolated, training-grade |
| **Goose** | local-first, multi-LLM |
| **Gemini CLI** | Google integration |
| **Headroom** | transparent middleware, compression |
| **Claude Code** | **harness-first, 5 surfaces, plugin marketplace** |

---

## 3. 진입점 & CLI (Entry & CLI)

### 3.1 `claude` 명령 (Terminal surface)

```bash
# 가장 단순한 시작
cd your-project
claude

# 비대화형
claude -p "translate new strings into French and raise a PR for review"

# 파이프 (Unix philosophy)
tail -200 app.log | claude -p "Slack me if you see any anomalies"

# 텔레포트 (세션 이동)
claude --teleport

# 버전
claude --version  # 2.1.168 (2026-06-07)
```

### 3.2 Sub-commands (대표)

- `claude` — REPL 시작
- `claude -p "<prompt>"` — 비대화형 (CI/CD 용)
- `claude --version` / `claude update` — 버전 관리
- `claude agents` — sub-agent list
- `claude --teleport` — 다른 surface 로 세션 이동
- `claude --bg-pty-host` — 백그라운드 PTY host (daemon)

### 3.3 주요 flags

- `--fallback-model <model>` — primary model overloaded 시 fallback 3개 순서 시도
- `--mcp-debug` — MCP server debug 정보
- `--permission-mode` — 권한 모드 (auto/accept-edits/plan/bypassPermissions)
- `--thinking disabled` / `MAX_THINKING_TOKENS=0` — thinking 비활성
- `--voice` — voice mode

### 3.4 Slash commands (100+ 추정)

- `/commit` / `/commit-push-pr` / `/clean_gone` — git workflow
- `/feature-dev` — 7-phase feature dev
- `/code-review` — multi-agent PR review
- `/pr-review-toolkit:review-pr` — 7 aspects review
- `/hookify` — hook 자동 생성
- `/plugin-dev:create-plugin` — plugin 생성 wizard
- `/loop` — prompt 반복
- `/schedule` — scheduled task
- `/login` / `/logout` — auth
- `/compact` — context 압축
- `/voice` / `/desktop` — surface 전환

### 3.5 pipe / Unix philosophy

```bash
# 로그 분석
tail -200 app.log | claude -p "Slack me if you see any anomalies"

# CI/CD
git diff main --name-only | claude -p "review these changed files for security issues"

# bulk operations
claude -p "find all TODOs and raise issues"
```

→ 우리 my_harness 의 `MiniMax.md` 도메인별 명령 도 같은 철학 (Unix composability + 한국어 prompt)

### 3.6 install methods (5 native)

| OS | 권장 | 대안 |
| --- | --- | --- |
| macOS / Linux / WSL | `curl -fsSL https://claude.ai/install.sh \| bash` | brew `--cask claude-code` (stable) / `@latest` (bleeding) |
| Windows (PS) | `irm https://claude.ai/install.ps1 \| iex` | winget `Anthropic.ClaudeCode` |
| Windows (CMD) | `curl -fsSL https://claude.ai/install.cmd -o install.cmd && install.cmd && del install.cmd` | winget |
| Linux package | apt / dnf / apk (Debian / Fedora / RHEL / Alpine) | install.sh |

- **Auto-update**: native install 만 background auto-update. brew/winget 은 수동 `brew upgrade` / `winget upgrade`
- **Git for Windows** 권장 (Bash tool 사용 위해). 미설치 시 PowerShell fallback
- npm install **deprecated** (README 명시)

### 3.7 sub-agents 의 진입점

`claude agents` — 등록된 sub-agent list. URL 입력 시 해당 URL 을 first prompt 로 가진 세션으로 filter.

---

## 4. TUI / 프론트엔드 (TUI / Frontend)

### 4.1 5 surfaces (cross-surface session)

| Surface | 대상 | 진입 |
| --- | --- | --- |
| **Terminal CLI** | power user, CI/CD | `claude` (npm/brew/winget) |
| **VS Code extension** | IDE 사용자 | marketplace `anthropic.claude-code` |
| **Desktop app** | 비-IDE, visual diff | claude.ai/download (mac/win x64/arm64) |
| **Web** (claude.ai/code) | 모바일, repo 미보유 | claude.ai/code |
| **JetBrains plugin** | IntelliJ/PyCharm/WebStorm | JetBrains Marketplace (ID 27310) |

### 4.2 VS Code 확장

- inline diffs
- @-mentions (file/symbol)
- plan review
- conversation history in editor
- Cursor marketplace 동시 지원 (`cursor:extension/anthropic.claude-code`)

### 4.3 Desktop app

- Visual diff review
- Multiple sessions side-by-side
- Scheduled recurring tasks
- Cloud session kickoff
- macOS (Intel/Apple Silicon Universal) / Windows x64 / Windows ARM64
- 유료 subscription 필요

### 4.4 Web (claude.ai/code)

- 브라우저, **repo 미보유** 도 작업 가능
- long-running task kickoff + check back
- parallel multi-task
- Claude iOS app 연동
- 무료 tier 없음 (유료)

### 4.5 JetBrains plugin

- IntelliJ IDEA, PyCharm, WebStorm 등
- interactive diff view
- selection context sharing
- beta label

### 4.6 cross-surface handoff

- **Remote Control** — phone/any browser 에서 local session continue
- **Dispatch** — phone 에서 task message → Desktop session 생성
- **claude --teleport** — web/iOS 에서 시작 → local terminal 로 pull
- **/desktop** — terminal session → Desktop 으로 hand off (visual diff)
- **Slack `@Claude`** — Slack bug report → pull request 자동
- **Routines** — Anthropic-managed infra 에서 scheduled task (컴퓨터 꺼져도 실행)

### 4.7 TUI 구현 (leak 분석)

- **146 UI components** — React/Ink 기반 (추정)
- **terminal framework 자체 구현** — readline/raw mode 처리
- **streaming output** — typewriter 효과
- **Ctrl+O** transcript view — duplicated thinking text 수정 (2.1.168)
- **Kitty keyboard protocol** — Shift+non-ASCII 문자 지원 (WezTerm/Ghostty/kitty)
- **flickering 수정** — JetBrains 2026.1+ synchronized output

### 4.8 IDE 통합 (VS Code / JetBrains)

- LSP-style server-host 통신 (추정)
- selection context 자동 공유
- file tree, git status 실시간 sync
- diff visualization (line/word/hunk 단위)

### 4.9 우리 my_harness 의 1안 (CLI-only) vs 2안 (TUI/IDE 확장) 결정

claude-code 가 5 surface 로 진화한 이유 = **사용자 진입점 다양화** (D-13). 우리도 TASK-005 결정 시 **단일 CLI** 부터 시작하되, **plugin marketplace** 와 **harness 추상화** 로 2안 확장 (TUI/Web) 대비는 해두는 게 안전.

---

## 5. LLM 통합 (LLM Integration)

### 5.1 Primary: Claude (Anthropic)

- **model family**: Claude Opus 4.5 (current flagship), Claude Sonnet 4.5, Claude Haiku 4 (추정)
- **3P providers 지원**: third-party integrations (VS Code, CLI) — "claude-code 는 Anthropic first, 3P second"

### 5.2 Fallback model (v2.1.166+)

```json
{
  "fallbackModel": ["claude-sonnet-4-5", "claude-haiku-4", "<third>"]
}
```

- **up to 3 fallback models in order** — primary overloaded/unavailable 시 순서대로 시도
- `--fallback-model` flag 가 interactive session 에도 적용 (v2.1.166)
- Claude API 가 unexpected non-retryable error 시 **fallback model 으로 1회 retry**
- **즉시 surface** 되는 error: auth, rate-limit, request-size, transport

### 5.3 Thinking (확장 thinking)

- `MAX_THINKING_TOKENS` env var / `--thinking disabled` flag / per-model toggle
- v2.1.166 부터: `=0` 또는 `--thinking disabled` 가 **default-on 모델의 thinking 비활성**
- **3P providers 변경 없음** — Anthropic Claude API only
- streaming 중 thinking text 가 Ctrl+O transcript view 에 duplicated → v2.1.168 fix

### 5.4 Provider 비종속: 3P integrations

- VS Code / Terminal CLI 모두 third-party providers 지원
- OpenAI, Google, custom provider (BYO key)
- 우리 `PROVIDERS.md` 의 rig-core / Vercel AI SDK 와 같은 추상화 방향

### 5.5 캐시 (cache) 와 cost

- Anthropic prompt caching 자동 활용 (추정)
- cache_aligner (headroom §13.5 와 같은 pattern) 가 내장
- cost tracking UI 는 desktop / web 에서

### 5.6 vision / multimodal

- image input 지원 (v2.1.168 image-processing error 수정)
- v2.1.168: unprocessable image 가 session 에 들어와도 "image could not be processed" error + extra token 안 씀

### 5.7 computer use (advanced)

- macOS GUI 자동 조작 (terminal · browser · GUI app)
- 2026-03 v2.x feature
- screen capture + vision
- ⚠️ 우리 v1 은 scope 외

### 5.8 우리 my_harness 의 LLM 통합 권장

1. **Primary = Claude Sonnet 4.5 (코드) / Haiku 4 (서버/환경)**
2. **Fallback 3 model** (claude-code 2.1.166 패턴) — claude-code v2 와 같은 안정성
3. **Provider 비종속 추상화** — `PROVIDERS.md` 의 rig-core 1안 / Vercel AI SDK 2안 + litellm proxy fallback
4. **Thinking** — Sonnet 4.5 의 thinking 활성화 (코드 도메인), Haiku 4 는 비활성
5. **Cache 친화** — headroom §13.5 CacheAligner 패턴

---

## 6. 도구/스킬 시스템 (Tool/Skill System)

### 6.1 4 계층 확장 (commands · agents · skills · hooks)

claude-code 의 plugin 은 **4 계층** 의 extension 으로 구성:

| 계층 | 정의 | 예시 |
| --- | --- | --- |
| **Slash commands** | `/` 로 시작하는 사용자 호출 prompt | `/commit`, `/feature-dev`, `/code-review` |
| **Agents** | specialized sub-agent (md 또는 system prompt 기반) | `code-explorer`, `code-reviewer`, `comment-analyzer` |
| **Skills** | 자동 invoke 되는 domain knowledge | `frontend-design`, `writing-rules`, `plugin-development` |
| **Hooks** | event 발생 시 자동 실행되는 shell command | SessionStart, PreToolUse, Stop |

### 6.2 Slash commands (100+ 추정)

`commands/*.md` — markdown file 1개 = 1 command. YAML frontmatter + 자유 prompt 본문.

```markdown
---
description: Guided feature development
argument-hint: Optional feature description
---

# Feature Development

You are helping... Follow systematic approach...
```

→ 우리 my_harness 의 `MiniMax.md` 의 TODO 명령 블록과 같은 컨셉. 다만 claude-code 는 plugin 으로 distribution.

### 6.3 Agents (specialized sub-agents)

`agents/*.md` — system prompt + tools 제한. 예시:
- `code-explorer` — codebase 이해
- `code-architect` — architecture 설계
- `code-reviewer` — quality review
- `comment-analyzer` — PR comment 분석
- `pr-test-analyzer` — test 분석
- `silent-failure-hunter` — silent failure detection
- `type-design-analyzer` — type design review
- `code-simplifier` — simplification

**multi-agent parallel**: code-review plugin 의 5 parallel Sonnet agents — CLAUDE.md compliance · bug detection · historical context · PR history · code comments → confidence-based scoring 으로 false positive filter

### 6.4 Skills (auto-invoke domain knowledge)

`skills/<name>/SKILL.md` — 자동 invoke. 사용자 호출 불필요. 예시:
- `frontend-design` — frontend 작업 시 자동 (Prithvi Rajasekaran, Alexander Bricken 작성)
- `writing-rules` — hookify rule syntax 안내
- `plugin-development` — 7 expert skills (command/skill/hook/MCP integration)

### 6.5 Hooks (event-driven shell)

event 종류:
- **SessionStart** — session 시작 시 (explanatory, learning output style)
- **PreToolUse** — tool 호출 직전 (security-guidance)
- **PostToolUse** — tool 호출 직후 (lint, format)
- **Stop** — session 종료 시 (ralph-wiggum 이 intercept)

**Hookify plugin** — markdown rule 로 hook 생성:
```markdown
# .claude/hookify.warn-rm.local.md
---
name: Warn before rm -rf
pattern: "rm\\s+-rf\\s+/"
action: warn
message: "Be careful with rm -rf!"
---
```

→ **우리가 차야 할 강력한 패턴**: 복잡한 hooks.json 없이 markdown 1 파일 = 1 rule.

### 6.6 MCP (Model Context Protocol)

`/mcp` 또는 `.mcp.json` 으로 외부 tool 등록:
- Google Drive, Jira, Slack, Figma, custom
- read design docs, update tickets, pull data
- official MCP quickstart 가이드 존재

### 6.7 Agent SDK (custom agent)

- **claude code 의 도구 + capabilities** 를 외부 SDK 로 노출
- **full control** over orchestration, tool access, permissions
- "build your own agents powered by Claude Code's tools"
- 우리 my_harness 의 mavis-team / orchestrator 와 같은 위치

### 6.8 Auto memory

- claude code 가 **자동**으로 session 중 학습 내용 저장
- "build commands and debugging insights across sessions"
- **사용자 작성 불필요** (CLAUDE.md 와 별개)
- 다음 session 시작 시 자동 주입

### 6.9 우리 my_harness 의 4-계층 extension

claude-code 의 **commands · agents · skills · hooks 4-계층** 패턴 + 우리 standard_ai_workflow 의 이벤트 소싱 결합:
- `commands/` — 도메인별 TODO 명령 (현재 MiniMax.md inline)
- `agents/` — specialized sub-agent (mini_coder_max / fullstack-dev / ...)
- `skills/` — 자동 invoke domain knowledge (코드 패턴, 서버 관리)
- `hooks/` — SessionStart/PreToolUse/PostToolUse event handler

→ **5 reference 중 가장 풍부한 extension 시스템**. goose/aider 의 단순 command 와 차이.

---

## 7. 컨텍스트 관리 (Context Management)

### 7.1 3 계층 (CLAUDE.md · auto memory · /compact)

| 계층 | 사용자 작성 | 자동 | 우선순위 |
| --- | --- | --- | --- |
| **CLAUDE.md** | ✅ 필수 | ❌ | 명시적 (project root) |
| **Auto memory** | ❌ | ✅ | 자동 (cross-session) |
| **/compact** | on-demand | ✅ | conversation 내 |

### 7.2 CLAUDE.md (project root)

```markdown
# CLAUDE.md (project root)
- 코드 컨벤션: ...
- 빌드 명령: pnpm install && pnpm build
- PR 룰: ...
- 하지 말 것: ...
```

- **session 시작 시 자동 load**
- "set coding standards, architecture decisions, preferred libraries, review checklists"
- 5 surface 모두 동일 file 사용
- **Anthropic 통계**: 2025년 한해 모든 repo 의 CLAUDE.md 가 README.md 총합보다 많음
- → 우리 my_harness 도 project root 에 `CLAUDE.md` 또는 `.MiniMax/AGENTS.md` 권장

### 7.3 Auto memory

- claude code 가 **대화 중 학습**한 내용 자동 저장
- "build commands and debugging insights"
- 사용자 작성 불필요
- cross-session 영속

### 7.4 /compact (context compression)

- claude code 내장 context 압축 command
- "context compression" feature (CSDN 분석)
- 2.1.166 부터 보강 — 긴 세션에서 context 한계 자동 대응
- → 우리 headroom §13.3 CCR (reversible) 과 같은 layer (단, claude code 는 user-callable)

### 7.5 Tool 정의 + history 주입

- 매 turn 마다 system prompt + CLAUDE.md + auto memory + tool defs + history 주입
- "Context 의 정밀함 은 passive 정보 전달 + active 압축/재주입"

### 7.6 Permission system

- 4 mode: `default` · `acceptEdits` · `plan` · `bypassPermissions`
- `--permission-mode` flag
- **SendMessage cross-session** 보안: 다른 session 의 user authority 안 carry. relayed permission request 거부. auto mode 차단 (v2.1.166 hardening)

### 7.7 우리 my_harness 의 context 관리 권장

1. **`CLAUDE.md` 표준** — 우리도 `my_harness` 의 `MiniMax.md` 또는 `.MiniMax/AGENTS.md` 가 claude code 의 CLAUDE.md 역할
2. **Auto memory layer** — long-running session 의 학습 내용 자동 저장
3. **/compact** — claude code 의 user-callable 압축을 우리도 slash command 로 노출
4. **CCR integration** — headroom §13.3 의 reversible + retrieval 패턴 (TASK-007)
5. **Permission system** — 4 mode 동일 (default/acceptEdits/plan/bypassPermissions)
6. **Provider-agnostic compression** — headroom 의 6 algorithms (CacheAligner/ContentRouter/CCR/SmartCrusher/CodeCompressor/Kompress-base)


---

## §8 세션 영속화 (Session Persistence)

### 8.1 cross-surface sessions

claude code 의 **가장 차별화된 기능**:
- **Local terminal session** 시작 → **Phone browser (Remote Control)** 에서 continue
- **Phone 에서 task message** (Dispatch) → **Desktop session** 자동 생성
- **Web / iOS** 에서 시작 → **`claude --teleport`** 로 local terminal pull
- **Terminal** → **`/desktop`** 으로 Desktop hand off (visual diff)

→ **세션은 surface-bound 가 아니라 user-bound**

### 8.2 cloud sessions (claude.ai/code)

- local repo 미보유도 작업 가능
- long-running task kickoff + check back
- parallel multi-task 실행
- session state 가 Anthropic cloud 에 저장

### 8.3 Routines (scheduled)

- **Anthropic-managed infra** 에서 실행 → 컴퓨터 꺼져도 동작
- API call / GitHub event trigger 가능
- web / desktop / CLI `/schedule` 로 생성
- **Desktop scheduled tasks** (local machine) — local file/tool 직접 접근
- **`/loop`** — CLI session 내 prompt 반복 (polling)

### 8.4 Routines 의 트리거 종류

- **Schedule** — cron-style (morning PR review, weekly dep audit)
- **API call** — webhook trigger
- **GitHub event** — PR open, issue labeled, push

### 8.5 Session state 저장 위치

- local: `~/.claude/` (XDG) — CLAUDE.md memory, hooks, settings
- cloud: Anthropic managed (Routines, cross-surface)
- ❓ leak 분석 추정: SQLite 또는 JSONL

### 8.6 우리 my_harness 의 cross-surface 전략

- **v1**: CLI only, local session (`state.json` + journal)
- **v2+**: Web/Dekstop hand-off (TASK-007 이후)
- standard_ai_workflow 의 handoff 가 이미 cross-session 보장

---

## §9 확장 시스템 (Extension System)

### 9.1 plugin 시스템 (12 official + marketplace)

**구조**:
```
plugin-name/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata
├── commands/                # Slash commands (optional)
├── agents/                  # Specialized agents (optional)
├── skills/                  # Agent Skills (optional)
├── hooks/                   # Event handlers (optional)
├── .mcp.json                # External tool config (optional)
└── README.md
```

### 9.2 12 official plugins (2026-06-07 기준, repo 내)

| plugin | version | author (Anthropic) | 핵심 |
| --- | --- | --- | --- |
| **agent-sdk-dev** | 1.0.0 | Ashwin Bhat | Agent SDK 개발 키트 (`/new-sdk-app`) |
| **claude-opus-4-5-migration** | 1.0.0 | William Hu | Sonnet 4.x/Opus 4.1 → Opus 4.5 마이그레이션 |
| **code-review** | 1.0.0 | Boris Cherny | multi-agent PR review + confidence scoring |
| **commit-commands** | 1.0.0 | Anthropic | git workflow (`/commit`, `/commit-push-pr`, `/clean_gone`) |
| **explanatory-output-style** | 1.0.0 | Dickson Tsai | SessionStart hook — educational context |
| **feature-dev** | 1.0.0 | Sid Bidasaria | 7-phase feature dev (`code-explorer/architect/reviewer`) |
| **frontend-design** | 1.0.0 | Prithvi Rajasekaran · Alexander Bricken | auto-invoke UI/UX 가이드 |
| **hookify** | 0.1.0 | Daisy Hollman | markdown rule 기반 hook 자동 생성 |
| **learning-output-style** | 1.0.0 | Boris Cherny | 사용자 코드 참여 요청 (5-10 lines) |
| **plugin-dev** | 1.0.0 | (TBD) | 7 expert skills + AI-assisted creation |
| **pr-review-toolkit** | 1.0.0 | Daisy Hollman | 7 aspects (comments/tests/errors/types/code/simplify) |
| **ralph-wiggum** | 1.0.0 | Daisy Hollman | self-referential AI loop, while-true |
| **security-guidance** | 2.0.0 | David Dworken | 9 security patterns + LLM-powered diff review + agentic commit reviewer |

→ 13개 (README 에 12개 명시, pr-review-toolkit 추가 확인)

### 9.3 Plugin marketplace

- Anthropic 공식 marketplace
- Community marketplace
- `/plugin` command 로 install
- `.claude/settings.json` 으로 project 단위 enable

### 9.4 Plugin-dev plugin 의 7 expert skills (plugin-dev 내부)

1. **command-development** — slash command 작성
2. **skill-development** — skill 작성
3. **plugin-structure** — plugin 표준 구조
4. **plugin-settings** — settings 관리
5. **hook-development** — hook 작성 (validation, linter, schema, test scripts)
6. **mcp-integration** — MCP server 통합
7. **agent-development** — agent 작성 (validate-agent.sh)

→ **plugin-dev plugin 자체가 plugin 개발 가이드** — meta-design.

### 9.5 Hookify plugin — 가장 영리한 패턴

- **markdown rule 1 file = 1 hook**
- `pattern: "rm\\s+-rf\\s+/"` + `action: warn` + `message: "..."`
- restart 없이 **다음 tool use 부터 적용**
- analyze recent conversation 으로 rule 자동 생성 가능
- `/hookify:list` / `/hookify:configure` / `/hookify:help` sub-command

**→ 우리 my_harness 의 hook 시스템도 같은 컨셉 권장**: JSON hooks.json 안 쓰고 `.myharness/hooks/*.md` markdown rule

### 9.6 Plugin 검증 (plugin-dev/scripts)

- `validate-plugin.sh` — plugin.json schema 검증
- `validate-agent.sh` — agent format 검증
- `validate-hook-schema.sh` — hook schema 검증
- `hook-linter.sh` — hook lint
- `test-hook.sh` — hook unit test
- `parse-frontmatter.sh` — YAML frontmatter parser
- `validate-settings.sh` — settings 검증

### 9.7 Security-guidance plugin (가장 mature 한 plugin)

**3-tier security review**:
1. **Pattern-based warnings on edits** — 9 security patterns (command injection, XSS, eval, dangerous HTML, pickle deserialization, os.system)
2. **LLM-powered diff review on Stop** — session 종료 시 diff 전체 LLM 분석
3. **Agentic commit reviewer** — 25+ vulnerability classes (injection, XSS, SSRF, hardcoded secrets)

**architecture** (`plugins/security-guidance/hooks/`):
- `_base.py` — base hook class
- `security_reminder_hook.py` — main entry
- `patterns.py` — 9 patterns
- `session_state.py` — session-level state
- `diffstate.py` — diff tracking
- `gitutil.py` — git utilities
- `ensure_agent_sdk.py` — Agent SDK check
- `review_api.py` — review API client
- `llm.py` — LLM client
- `extensibility.py` — custom pattern extension
- `sg-python.sh` — entrypoint shell

**→ 우리 v2+ security hook 의 reference implementation 으로 적합**

### 9.8 우리 my_harness 의 plugin 시스템 권장

1. **plugin 디렉토리 표준**: `~/.myharness/plugins/<name>/`
2. **4-계층**: commands / agents / skills / hooks
3. **Plugin manifest**: `plugin.json` (name, version, description, author)
4. **Marketplace**: v2+ (v1 은 local plugin 만)
5. **Hook markdown rule**: hookify 컨셉 차용
6. **Plugin-dev plugin 자체**: 우리도 `my_harness-plugin-dev` 로 plugin 개발 가이드 자동화
7. **Security plugin**: security-guidance v2.0 의 3-tier 패턴 차용

---

## §10 빌드 & 배포 (Build & Distribution)

### 10.1 빌드 (closed)

- Anthropic 내부 빌드 (확인 불가)
- ❓ leak 분석 추정: TypeScript + esbuild/Rollup → Node.js bundle + native binary (Rust/Tauri 가능성)
- npm: `@anthropic-ai/claude-code` (deprecated 되었지만 여전히 가능)

### 10.2 5 배포 채널

| 채널 | install | auto-update |
| --- | --- | --- |
| **Native install** (mac/linux/win) | `curl -fsSL https://claude.ai/install.sh \| bash` 등 | ✅ background |
| **Homebrew** | `brew install --cask claude-code` (stable) / `claude-code@latest` (bleeding) | ❌ 수동 `brew upgrade` |
| **WinGet** | `winget install Anthropic.ClaudeCode` | ❌ 수동 `winget upgrade` |
| **Linux package** | apt / dnf / apk | distro 의존 |
| **NPM** | `npm install -g @anthropic-ai/claude-code` (deprecated) | npm |

### 10.3 Stable vs Latest 채널

- **stable** — 약 1주 lag, major regression skip
- **latest** (bleeding) — 출시 즉시

→ 우리 my_harness 도 stable/latest 듀얼 채널 권장

### 10.4 Cross-platform

- macOS (Intel + Apple Silicon Universal)
- Linux (Debian/Fedora/RHEL/Alpine via apt/dnf/apk)
- WSL
- Windows (PowerShell/CMD, x64/ARM64)
- Web (browser)
- iOS app

### 10.5 Desktop app

- macOS Universal DMG
- Windows x64 setup
- Windows ARM64 setup
- 유료 subscription 필수

### 10.6 Release cadence

- **3-5 releases/week** (CHANGELOG 헤더 통계)
- v2.1.168 (2026-06-07)
- v2.1.129 (2026-05-11)
- major release: v0.x → v1.0 → v2.0 (2025-10 추정) → v2.1.x (현재)

### 10.7 우리 my_harness 의 빌드/배포 권장

- **v1**: single binary (Rust 1안 / TS 2안). mac/linux/win 동시
- **install**: install.sh (mac/linux) + install.ps1 (win) + homebrew tap
- **stable/latest 듀얼 채널** — npm `@myharness/cli@latest` + `@myharness/cli@stable`
- **auto-update**: native install 만 background. homebrew/scoop/winget 은 수동

---

## §11 테스트 & 품질 (Testing & Quality)

### 11.1 Closed source 의 한계

- **외부에서 test code 확인 불가**
- plugin-dev 의 7개 test/lint scripts 가 유일한 inspectable test infrastructure
  - `validate-plugin.sh`, `validate-agent.sh`, `validate-hook-schema.sh`, `hook-linter.sh`, `test-hook.sh`
  - `validate-settings.sh`, `validate-agent.sh`
  - **plugin 자체의 unit test** 가 아니라 **plugin 형식 검증** (schema/lint)

### 11.2 Bug fix rate

- CHANGELOG 보면 **major release 마다 10+ bug fix**
- v2.1.168: 12+ fix (image processing, background agent, voice mode, hooks, daemon, ...)
- **rapid iteration culture** — 작은 fix 들을 자주 release

### 11.3 Catastrophic fix (v2.1.166)

- "Hardened cross-session messaging: messages relayed via `SendMessage` from other Claude sessions no longer carry user authority"
- **보안 사고** → 즉시 hardening
- **3P providers unchanged** (Anthropic API only 변경)

### 11.4 우리 my_harness 의 test 인프라 권장

- **plugin 형식 검증** scripts (plugin-dev 와 같은 7 scripts)
- **CI matrix** — mac/linux/win × stable/latest
- **rapid iteration** — bug fix 작은 단위 자주 release (v1 부터)

---

## §12 보안 (Security)

### 12.1 4-tier 보안

1. **Permission mode** — 4 mode (default/acceptEdits/plan/bypassPermissions)
2. **Hook system** — PreToolUse (security-guidance 9 patterns)
3. **LLM review** — Stop 시 diff 전체 LLM 분석
4. **Agentic reviewer** — commit 시 25+ vulnerability classes

### 12.2 Auth

- Claude subscription (max/team/enterprise)
- Anthropic Console (pay-as-you-go)
- 3P provider (BYO key)
- **macOS Keychain** integration (v0.2.30 부터)
- ❓ Windows credential manager / Linux secret service 동일 추정

### 12.3 MDM (Mobile Device Management)

- `examples/mdm/` — enterprise MDM 예시
- organization-wide settings push
- managed policies (`allowedMcpServers`, `deniedMcpServers`, etc.)
- **${VAR} reference** 지원 (v2.1.166 hardening)

### 12.4 Cross-session security (v2.1.166)

- `SendMessage` 를 통한 다른 session 의 user authority 안 carry
- relayed permission request 자동 거부
- auto mode 에서 blocked
- **보안 사고 후 즉시 hardening** — 빠른 패치 culture

### 12.5 Background agent / daemon security

- `claude --bg-pty-host` orphan process 100% CPU spin (v2.1.168 fix)
- managed settings invalid entry → silent disable remaining policies (v2.1.168 fix)

### 12.6 Image processing (v2.1.168)

- unprocessable image 가 session 에 들어와도 error + extra token 안 씀

### 12.7 our my_harness 의 보안 권장

1. **Permission 4 mode** (default/acceptEdits/plan/bypassPermissions)
2. **Hook system** — `.myharness/hooks/*.md` markdown rule (hookify 컨셉)
3. **Pattern-based security** — security-guidance 9 patterns 차용
4. **macOS Keychain / Windows Credential Manager / Linux Secret Service** — D-06 패턴
5. **MDM support** — enterprise v2+
6. **Catastrophic fix cycle** — 보안 사고 시 24-48h hardening

---

## §13 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 Harness-first architecture (Model + Harness 5 components) ⭐

`Agent = Model + Harness` — 모델보다 **Harness 의 5 components** (Tools, Context, Session, Plugins, Sub-agents) 가 핵심 차별점. 우리 my_harness 도 **Harness 추상화** 가 1순위. mavis-team 의 orchestrator 가 Harness 의 instance.

#### 13.2 5 surfaces cross-surface session ⭐

CLI · VS Code · Desktop · Web · JetBrains 5 surface 가 **단일 session model** 공유. `claude --teleport`, Remote Control, Dispatch, /desktop hand-off. 우리 v1 은 CLI 만, v2+ 에서 TUI/IDE/Web hand-off. **사용자 진입점 다양화** = D-13 의 3-모드와 같은 효과.

#### 13.3 Plugin marketplace (12 official + community) ⭐

plugin 4-계층 (commands/agents/skills/hooks) + marketplace. **확장성의 핵심**. 우리 mavis-team 의 mavis/skills/ 와 같은 위치이나 claude-code 가 훨씬 정교. v2+ marketplace 권장.

#### 13.4 Hookify: markdown rule = 1 hook ⭐

복잡한 hooks.json 대신 **markdown 1 file = 1 hook**. `pattern` + `action` + `message` + restart-free 적용. → 우리 v1 부터 `.myharness/hooks/*.md` 권장.

#### 13.5 Auto memory (cross-session)

사용자 작성 없이 claude code 가 자동 학습. 우리도 `~/.myharness/memory/auto/` + standard_ai_workflow 의 handoff 와 결합 권장.

#### 13.6 CLAUDE.md 표준 (project root)

session 시작 시 자동 load, 5 surface 공유. → 우리도 `MiniMax.md` 또는 `.MiniMax/AGENTS.md` 가 동일 역할. (이미 있음)

#### 13.7 /compact (user-callable context compression)

`/compact` slash command 가 context 압축. 우리도 headroom §13.3 CCR 과 결합한 `/compact` 권장.

#### 13.8 4-tier permission system

default/acceptEdits/plan/bypassPermissions. claude code 의 안전성 foundation.

#### 13.9 5 install methods + stable/latest 듀얼 채널

install.sh / install.ps1 / brew / winget / npm (deprecated). 우리 v1 도 동일.

#### 13.10 Background auto-update (native install only)

brew/winget 은 수동 upgrade. native install 만 background. 사용자가 안정성/최신 tradeoff 선택.

#### 13.11 Multi-agent parallel (code-review plugin)

5 parallel Sonnet agents + confidence-based scoring. 우리 mavis-team 의 worker pool 과 같은 컨셉. mavis-team 의 verifier 가 scoring.

#### 13.12 Plugin-dev plugin (meta-design)

plugin-dev plugin 자체가 plugin 개발 가이드. 7 expert skills + validate scripts. **우리도 `my_harness-plugin-dev` 권장**.

#### 13.13 Security-guidance 3-tier (pattern + LLM + agentic)

9 pattern-based warnings + LLM diff review + 25+ vulnerability classes agentic reviewer. **우리 v2+ security plugin reference**.

#### 13.14 Cross-session security (no user authority carry)

`SendMessage` relayed 메시지가 user authority 안 carry. auto mode 에서 relayed permission 자동 거부. **multi-agent 시스템의 필수 패턴**.

#### 13.15 Fallback model (3 in order) ⭐

`fallbackModel: [a, b, c]` + `--fallback-model` flag. primary overloaded 시 순서 시도. 우리 my_harness 의 TASK-005 결정 시 **rig-core 1안 + litellm proxy fallback** 와 결합.

#### 13.16 Unix philosophy: pipe + non-interactive

`tail -200 app.log | claude -p "..."` — CI/CD, bulk operation, scripting. 우리 my_harness 의 `MiniMax.md` TODO 명령과 같은 Unix composability.

#### 13.17 Routines (scheduled + trigger)

cron + API + GitHub event trigger. **Anthropic-managed infra** (컴퓨터 꺼져도). 우리 v2+ scheduled tasks (D-06 의 standard_ai_workflow 와 결합).

#### 13.18 Anthropic 5 component model (arxiv 2604.14228) ⭐

**Tools / Context / Session / Plugins / Sub-agents** — 우리 my_harness 의 **architecture 청사진**. mavis-team 의 5 component 와 정합.

#### 13.19 Cross-platform 단일 binary (추정)

mac/linux/win 동시 + 5 surface. **우리 my_harness 의 단일 언어 (Rust 1안 / TS 2안) 결정 + Tauri/Electron** 등 검토.

#### 13.20 Thinking toggle (env + flag + per-model)

`MAX_THINKING_TOKENS` env, `--thinking disabled` flag, per-model toggle. 우리 my_harness 의 도메인별 (코드/서버/환경) thinking 자동화.

#### 13.21 Permission mode (SendMessage hardening)

v2.1.166 부터 cross-session security 강화. **빠른 패치 cycle** = 우리도 bug fix 단위 release (3-5/week).

#### 13.22 Sub-agents + Agent SDK (custom agent) ⭐

부모 agent 가 자식 spawn + SDK 로 완전 customize. 우리 mavis-team 의 `mavis communication send` 와 동급. **orchestrator 패턴**.

#### 13.23 Computer Use (advanced)

macOS GUI 자동 조작. 우리 v1 scope 외이지만, v3+ "환경 셋업" 도메인에서 활용 가능.

#### 13.24 MCP 통합 (first-class)

mcp config 1 file + `/mcp` command. **first-class** 지원. 우리도 MCP server 노출 1안 권장 (D-13 의 MCP mode).

#### 13.25 Channels (Telegram/Discord/iMessage/webhook)

chat platform → claude code session. 우리 v2+ Slack/Telegram 통합 권장.

#### 13.26 Auto-update (background, native install only)

brew/winget 은 수동. 사용자 선택권.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.27 Closed source + leaked source

claude-code 는 closed. 2026-03-31 leaked source map 으로 reverse engineering 분석 가능하나, **공식 API 와 leak 의 일치 보장 없음**. 우리 my_harness 는 open source 정책 (MIT/Apache).

#### 13.28 npm install deprecated (README 명시)

CLI npm distribution deprecated. → 우리 v1 부터 native install 우선, npm 은 fallback.

#### 13.29 macOS Keychain (v0.2.30) — 늦은 도입

초기 release 에 macOS Keychain 안 쓰고 0.2.30 에서 추가. **우리 v1 부터 keychain/credential manager 필수** (D-06).

#### 13.30 100+ slash commands 의 discoverability 문제

너무 많은 command 는 사용자 기억 부담. 우리 my_harness 는 **도메인별 3-4 명령** 권장 (D-02).

#### 13.31 thinking duplicated text in Ctrl+O (v2.1.168 fix)

streaming 중 bug. **우리 v1 부터 streaming test 필수**.

#### 13.32 orphaned --bg-pty-host process (v2.1.168 fix)

daemon die 시 100% CPU. 우리 v1 부터 daemon lifecycle 검증.

#### 13.33 Anthropic Commercial License (closed)

plugin 코드만 MIT, 본체는 closed. **우리 my_harness 는 MIT or Apache 2.0 (open)**.

#### 13.34 Subscription requirement (Desktop/Web)

Desktop/Web 은 유료 sub 필수. 우리 v1 은 CLI free, v2+ subscription model 검토.

#### 13.35 Computer Use 의 안전성

GUI 자동 조작은 security risk. 우리 v3+ 도입 시 sandbox 필수.

#### 13.36 5 surface 의 maintenance 부담

5 surface 모두 동시 유지 = bug fix 5x. 우리 v1 CLI only, v2+ 점진 확장.

#### 13.37 Auto memory 의 privacy

cross-session 자동 저장 = 사용자 데이터 cloud. 우리 v1 은 local-only, v2+ opt-in cloud.

---

## §14 미해결 질문 (Open Questions)

코드/문서로 답 못 한 것. 메인테이너/이슈/PR 확인 필요.

### 14.1 Harness 5 components 의 정확한 내부 API

arxiv paper + CSDN 분석이 5 components 명시하나, **leak 의 실제 API 와 Anthropic 의 canonical 정의 차이** 가능. 공식 docs 에 5 component 명시 ❌.

### 14.2 claude code 의 정확한 build pipeline

TypeScript → ? native binary (Rust/Tauri) 인지 pure Node bundle 인지. leak 분석도 추측. 우리 v1 의 build toolchain 결정 (rollup/esbuild/Tauri/Electron) 에 참고.

### 14.3 cross-surface session 의 동기화 메커니즘

`claude --teleport` 가 어떻게 session state 를 surface 간 이동시키는지. **WebSocket? CRDT? 단순 snapshot?** 우리 v2+ cross-surface 시 같은 문제.

### 14.4 Auto memory 의 정확한 저장 위치/형식

❓ leak 분석: SQLite? JSONL? cloud-only? 우리 v1 auto memory 설계 시 결정.

### 14.5 Permission 4 mode 의 interaction matrix

mode × tool × hook 의 interaction. 우리 v1 permission 설계 시 reference.

### 14.6 fallback model 의 retry 정책 (v2.1.166)

3 fallback 의 retry vs failover 차이. 우리 fallback 결정 시 필요.

### 14.7 plugin marketplace 의 backend API

official marketplace 의 API, auth, distribution. 우리 v2+ marketplace 시 backend 설계.

### 14.8 security-guidance 의 9 patterns 정확 list

README 에 9 pattern 카테고리만 명시. **각 pattern 의 regex/heuristic** 은 _base.py / patterns.py 소스 확인 필요.

### 14.9 background agent 의 git worktree 처리

background agent 가 git worktree 사용. 우리 v2+ multi-agent + git workflow 시 same pattern.

### 14.10 Routines 의 Anthropic infra 구현

cloud infra 의 어디서 어떻게 실행? container? serverless? 우리 v2+ scheduled task 시 backend 결정.

### 14.11 plugin-dev 의 7 skills 의 우선순위

plugin 작성 시 어떤 skill 부터 consult? 우리 plugin-dev 설계 시 UX 흐름.

### 14.12 claude code 의 첫 release (v0.2.21) 와 현재 v2.1.168 의 architecture 진화

CHANGELOG 의 major architecture 변경 (v0.x → v1 → v2) 시점. 우리 my_harness 도 v0.x 부터 architecture 진화 예상.

### 14.13 Computer Use 의 platform 한계

macOS 만? linux/win 가능? 우리 v3+ 도입 시 결정.

### 14.14 Claude SDK vs Agent SDK 의 차이

❓ docs 의 "Claude SDK" 와 "Agent SDK" 가 같은 것인지 별도인지. 우리 SDK 결정 시.
