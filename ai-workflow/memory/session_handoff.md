# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-09 (D-46, TASK-005-1 W8 완료 — myharness-context crate v1 + 5 commit dual-push)
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json), [CONCEPT.md](../../docs/CONCEPT.md) (SSOT)

## Current Focus

- **v1 컨셉 Phase 종료 (2026-06-07)** — my_harness 의 SSOT (CONCEPT.md) 확립. 17 섹션 (12 + 5 신규: §5.10~§5.14). 4/5 결정 ✅ 완료, 1/5 (TASK-002) ⏸ 보류.
  - **TASK-005**: Rust 1안 (D-36) — ratatui + rig-core + rmcp + keyring + cargo-dist
  - **TASK-006**: ratatui + crossterm (D-36, TASK-005 종속)
  - **TASK-007**: headroom v1 = 3 알고리즘 (D-37). CCR + Kompress-base v1.5+ (D-46 W8.4 에서 CacheAligner/ContentRouter/SmartCrusher/CodeCompressor v1 구현)
  - **TASK-008**: provider-auto-config skill (D-38) — v1 simple 구현 (D-45)
  - **TASK-002**: ⏸ 보류 (yklee 인프라 정보 의존)
- **TASK-005-1 (D-43 → D-44 → D-45 → D-46)**: W3~W6.5 (tools) + W7 (llm) + W8 (context) 완료 ✅
- **W8 산출물 (D-46)**: myharness-context crate — CLAUDE.md loader (project root + parent walk + global fallback) + auto memory (NDJSON append-only) + ContextManager (token budget + /compact Layer 1: Truncate/Summarize-stub/Hybrid) + Layer 2 BuiltinPipeline (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor, 기본 off) + ContextConfig (config.toml [context] 섹션) + ContextOrchestrator (전체 통합). 54 tests pass. release 빌드 성공.
- **W8 5 commit**: 4d61e85 (W8.1) → 3116114 (W8.2) → fa09aeb (W8.3) → afcccea (W8.4) → faf7f85 (W8.5) → Gitea origin push → GitHub upstream push
- **다음: W9 (compression crate)** — §5.6 Layer 1 + Layer 2 정식 통합, 또는 W10 (TUI shell) 진입. context crate 와 중복 기능 분리 필요.

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 도메인별 명령 가이드 (코드/서버/환경): **⏸ deferred** (yklee 인프라 정보 필요, v1 Rust 구현 시점에 재검토)
- TASK-003 Gitea 미러: done (D-05, D-07, D-08)
- TASK-004 CLI/TUI 레퍼런스 분석:
  - 1차 (8축 비교표): done (`docs/REFERENCES.md`, D-09)
  - 2차 (14섹션 심층분석, 7 reference): done (D-10, D-15, D-21) — `docs/references/{codex,aider,goose,opencode,gemini-cli,claude-code,headroom}.md`
  - 7-doc cross-review: done (D-21) — `docs/references/README.md`
- TASK-005 my_harness CLI/TUI 전환: **✅ done (스택 결정)** → **🔜 TASK-005-1 (v1 MVP 빌드)**
  - TASK-005-1 (v1.0 MVP): **in_progress** (W3~W6.5 tools crate 완료 D-43 + dual-remote push D-44, W7 llm crate 진입 대기)
  - TASK-005-2 (v1.5): planned
  - TASK-005-3 (v2.0): planned
  - TASK-005-4 (v2.5): planned
  - TASK-005-5 (v3.0): planned
- TASK-006 TUI 라이브러리: ✅ done (ratatui, D-36)
- TASK-007 headroom 우선순위: ✅ done (3 알고리즘 v1, D-37)
- TASK-008 Provider fallback: ✅ done (provider-auto-config skill, D-38)

## Key Changes (오늘 — D-22 ~ D-38)

- 2026-06-07 1차 — **CONCEPT.md** SSOT 신설 (D-22): 12 섹션 (positioning/타겟/가치/스코프/v1 MVP spec/v2+ 로드맵/채택 23/안티 6/KPI/리스크/Open decisions/참조).
- 2026-06-07 2차 — **4 docs align to CONCEPT.md** (D-23, D-35): MiniMax.md / PROJECT_PROFILE.md / README.md (root) / PROVIDERS.md.
- 2026-06-07 3차 — **컨셉 교정 1차** (D-24): "외부 4-워커 통합/오케스트레이션" framing 제거. my_harness = sibling standalone tool.
- 2026-06-07 4차 — **컨셉 교정 2차 (Mavis zero coupling)** (D-25): §0.5 다이어그램에서 Mavis/orchestrator/standard_ai_workflow 모두 제거. §5.8 "외부 의존성 없음" 신설.
- 2026-06-07 5차 — **standard_ai_workflow 준수** (D-26): §5.9 신설. 6 원칙 native (한국어/절약/상태/이벤트/비참조/handoff) + 옵션 Mavis 통합.
- 2026-06-07 6차 — **headroom = built-in 압축 layer** (D-27): §0.5/§3.3/§5.6 갱신. 흐름 = `user → my_harness → (built-in 압축) → LLM provider`.
- 2026-06-07 7차 — **Provider 6개 확정 + OpenAI 호환 lingua franca** (D-28): §5.5 전면 갱신. claude/codex/gemini = native SDK, deepseek/minimax/local-llm = OpenAI 호환.
- 2026-06-07 8차 — **§5.10~§5.14 적용** (D-29~D-33): Agent 3 모드 (orchestrator/single/loop) + 15개 built-in sub-agents + `~/.myharness/` 디렉토리 구조 + LLM Wiki memory + Skill/MCP first-class.
- 2026-06-07 9차 — **TASK-NNN 형식 통일 + 2.1.169 pending 표** (D-34): §6 마일스톤 → TASK-005-1~TASK-005-5, §11.1 TASK-006/008 번호, §11.2 claude-code 2.1.169 영향 결정 표.
- 2026-06-09 — **TASK-005-1 W3~W6.5 완료** (D-43): myharness-tools 1차 완성 (5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format). 9 commit Gitea push.
- 2026-06-09 — **dual-remote 적용** (D-44): Gitea PAT `myharness-cli` 발급 (scopes: write:repository, write:user) in `~/.git-credentials` (chmod 600). `git remote add upstream https://github.com/ykylee/my_harness.git`. 9 commit 양쪽 push 완료 (`b266f3b..dfc9d93`).
- 2026-06-07 10차 — **TASK-005 결정: Rust 1안** (D-36): §11.1 + §11.3 갱신. v1 스택 종합 (Rust 2024 + ratatui + rig-core + rmcp + keyring + cargo-dist).
- 2026-06-07 11차 — **TASK-007 결정: headroom v1 1안 유지** (D-37): v1 = 3 알고리즘. CCR + Kompress-base v1.5+ 로 연기.
- 2026-06-07 12차 — **TASK-008 결정: provider-auto-config skill** (D-38): §5.5 전면 갱신 (4 subsections) + `docs/skills/provider-auto-config/SKILL.md` reference design 신설.
- 2026-06-07 13차 — **첫 push** (D-20): origin=Gitea (private) + upstream=GitHub (public) 듀얼 remote.
- 2026-06-07 14차 — **관련 문서 align** (D-35): 4 docs 일괄 갱신.
- 누적 18개 커밋 (D-22~D-38 시점, 782679d~33b590e).
- 2026-06-09 — **TASK-005-1 환경 검증 (D-41)**: W0-1 (Rust toolchain + crate) ✅ + W0-2 (cross-build + keychain + .myharness) ✅. Linux x86_64 (Ubuntu 25.10) / Rust 1.94.1. 12+ crate 전부 가용. Prerequisite 5건 식별 (libsecret-1-dev + 5 rustup target + cargo-dist/binstall + ANTHROPIC_API_KEY + serde_yml). TASK-005-1 진입 가능.
- 2026-06-09 — **TASK-005-1 W3~W6.5 완료 (D-43)**: myharness-tools crate (5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format). 63 tests passed. 9 commit (3f0c9cb~dfc9d93) Gitea push. PAT 설정 보류. W7 llm crate 진입 대기.

## 다음에 할 일 (Next Actions)

- [x] **v1 컨셉 확립** (D-22~D-38) — 5/5 결정 검토 완료 (4 ✅, 1 ⏸)
- [x] **TASK-005-1 환경 검증 (D-41)** ✅
- [x] **W2~W6.5 (D-43)** — tools crate 5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format ✅
- [x] **Gitea push** ✅ (9 commit, 3f0c9cb~dfc9d93)
- [x] **dual-remote push (D-44)** ✅
- [x] **W7 (D-45)** — myharness-llm crate v1 (6 provider + LLMClient + rig-core wrap + auth + discover + chain + router) ✅
- [x] **W7 dual-remote push (D-45)** ✅
- [x] **W8 (D-46)** — myharness-context crate v1 (CLAUDE.md + auto memory + ContextManager + Layer 2 4 알고리즘 + ContextConfig + ContextOrchestrator) ✅
- [x] **W8 dual-remote push (D-46)** ✅
- [ ] **Gitea PAT 회전** (이전 세션 노출 회전 권고) — yklee 가 회전 시 통지
- [ ] **W9 (compression crate)** — §5.6 Layer 1 정식 (LLM-driven summarize) + Layer 2 (CCR + Kompress-base) 또는 W10 (TUI shell) 우선
- [ ] **ANTHROPIC_API_KEY 주입 시 LLM E2E 테스트** (real-anthropic ignored test 활성화)
- [ ] **§5.12 디렉토리 자동 생성** (v1 first run 시) — `~/.myharness/{config,state,memory,handoff,compression,sub-agents}/` + `state.json` + `auth/`
- [ ] **Phase 1 of provider-auto-config** — D-45 W7 에서 v1 simple 구현. v1.5+ 에서 정식 + marketplace.
- [ ] **TASK-002 (도메인별 명령)** — yklee 인프라 정보 수령 후 (SSH 별칭 / Brewfile / dotfiles / 런타임 버전) 진행
- [ ] **헤로쿠 / Synology NAS 인프라 검증** — yklee 가 인프라 정보 입력 시점에 작업

## Risks & Blockers

- **claude-code 2.1.169 changelog 미공개** (D-34 §11.2 pending): context var/cache, MCP, permission 변경이 우리 §5.6/§5.14/§5.4/§5.5 영향 가능. 공개 시 검증 후 §11.2 처리.
- **minimax base_url 미검증** (D-28 TBD): yklee 가 base_url + API 형식 확인 후 v1 또는 v1.5 통합.
- **CCR + Kompress-base 연기** (D-37): v1.5+ TASK-005-2 시 재검토. ONNX 모델 weight ~수MB + CCR round-trip 1회 비용 trade-off.
- **TASK-002 인프라 정보 의존** (D-39): yklee 가 SSH 호스트 목록 / Brewfile / dotfiles / asdf 버전 입력 전까지 보류. v1 Rust 구현 시점에 자연 도출 가능.
- **외부 4-워커 (Claude/Codex/Gemini/OpenCode) sibling 정책 유지** (D-24, D-25): my_harness 가 그 도구들을 통합/오케스트레이션 안 함, sibling 으로만 인식. 추후 4-워커 정책 변경 시 검증.
- **Gitea + GitHub 듀얼 remote** (D-20, D-07): origin=Gitea (private) + upstream=GitHub (public). GitHub public 노출은 의도된 외부 미러링. 토큰 회전 시 yklee 가 Mavis 에 직접 전달.
- **agent memory**: "Worker 세션 long Write call 죽음 패턴" (D-16) — `~/.mavis/agents/mavis/memory/MEMORY.md` 에 영구 저장. 향후 long Write 시 chunked write + early deliverable signal.
- **user memory** (yklee 프로필): Gitea 정보, 작업 스타일, PR 작업 패턴, 분석/리서치 작업 스타일 — `~/.mavis/memory/user.md`.
- **yklee 비밀번호 / 토큰 값**: 메모리/문서/git 저장 금지 (D-06 정책). 회전 시 Mavis 가 매번 새로 전달.
- **TASK-005-1 Prerequisite 5건 미설치**: libsecret-1-dev + gnome-keyring (Linux keychain backend), 5 cross-compile target (rustup), cargo-dist + cargo-binstall, ANTHROPIC_API_KEY (env or keyring), serde_yaml→serde_yml 전환. 설치 후 cargo init 진행.
- **Gitea PAT 미설정 (D-43)**: yklee 가 다음 세션에서 PAT 제공 시 credential store + GitHub upstream 추가 가능. credential helper 가 비어 있어 push 시 인증이 gh-cli 또는 ssh fallback 으로 처리된 것으로 보임 (정확한 메커니즘 미확인).
- **ANTHROPIC_API_KEY absent (D-41 에서 식별)**: LLM E2E 테스트는 키 주입 후. W7 llm crate 는 mock test 위주로 진행. D-45 에서 W7 완료 — KeyringAuthStore 도 libsecret 부재 환경 graceful fallback. ANTHROPIC_API_KEY 주입 시 real-anthropic ignored test 활성화 가능.
- **TASK-002 (도메인별 명령)**: yklee 인프라 정보 의존. v1 Rust 구현 단계 (tools + llm crate 완료) 에 도달했으나 아직 인프라 정보 미수령.
