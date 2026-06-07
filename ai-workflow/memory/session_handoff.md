# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-07 (D-22~D-38, v1 컨셉 Phase 종료)
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json), [CONCEPT.md](../../docs/CONCEPT.md) (SSOT)

## Current Focus

- **v1 컨셉 Phase 종료 (2026-06-07)** — my_harness 의 SSOT (CONCEPT.md) 확립. 17 섹션 (12 + 5 신규: §5.10~§5.14). 4/5 결정 ✅ 완료, 1/5 (TASK-002) ⏸ 보류.
  - **TASK-005**: Rust 1안 (D-36) — ratatui + rig-core + rmcp + keyring + cargo-dist
  - **TASK-006**: ratatui + crossterm (D-36, TASK-005 종속)
  - **TASK-007**: headroom v1 = 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) (D-37). CCR + Kompress-base v1.5+
  - **TASK-008**: provider-auto-config skill (D-38) — 하드코딩 fallback 폐기, 동적 discovered list + per-provider auth
  - **TASK-002**: ⏸ 보류 (yklee 인프라 정보 의존)
- **다음 단계**: TASK-005-1 (v1 MVP Rust 빌드) — cargo workspace init → ratatui shell → rig-core Anthropic → basic tools → /compact → standard_ai_workflow output
- **D-34 §11.2 pending**: claude-code 2.1.169 changelog 공개 시 Anthropic fallback / context var / MCP 변경 검증

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 도메인별 명령 가이드 (코드/서버/환경): **⏸ deferred** (yklee 인프라 정보 필요, v1 Rust 구현 시점에 재검토)
- TASK-003 Gitea 미러: done (D-05, D-07, D-08)
- TASK-004 CLI/TUI 레퍼런스 분석:
  - 1차 (8축 비교표): done (`docs/REFERENCES.md`, D-09)
  - 2차 (14섹션 심층분석, 7 reference): done (D-10, D-15, D-21) — `docs/references/{codex,aider,goose,opencode,gemini-cli,claude-code,headroom}.md`
  - 7-doc cross-review: done (D-21) — `docs/references/README.md`
- TASK-005 my_harness CLI/TUI 전환: **✅ done (스택 결정)** → **🔜 TASK-005-1 (v1 MVP 빌드)**
  - TASK-005-1 (v1.0 MVP): planned
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
- 2026-06-07 10차 — **TASK-005 결정: Rust 1안** (D-36): §11.1 + §11.3 갱신. v1 스택 종합 (Rust 2024 + ratatui + rig-core + rmcp + keyring + cargo-dist).
- 2026-06-07 11차 — **TASK-007 결정: headroom v1 1안 유지** (D-37): v1 = 3 알고리즘. CCR + Kompress-base v1.5+ 로 연기.
- 2026-06-07 12차 — **TASK-008 결정: provider-auto-config skill** (D-38): §5.5 전면 갱신 (4 subsections) + `docs/skills/provider-auto-config/SKILL.md` reference design 신설.
- 2026-06-07 13차 — **첫 push** (D-20): origin=Gitea (private) + upstream=GitHub (public) 듀얼 remote.
- 2026-06-07 14차 — **관련 문서 align** (D-35): 4 docs 일괄 갱신.
- 누적 18개 커밋 (D-22~D-38 시점, 782679d~33b590e).

## 다음에 할 일 (Next Actions)

- [x] **v1 컨셉 확립** (D-22~D-38) — 5/5 결정 검토 완료 (4 ✅, 1 ⏸)
- [ ] **TASK-005-1 시작** (v1 MVP Rust 빌드) — cargo workspace init → ratatui shell → rig-core Anthropic → basic tools → /compact → standard_ai_workflow output
- [ ] **§5.6 Layer 1 구현** (필수 자동 압축) — token budget 추적 + auto truncate/summarize + /compact
- [ ] **§5.12 디렉토리 자동 생성** (v1 first run 시) — `~/.myharness/{config,state,memory,handoff,compression,sub-agents}/` + `state.json` + `auth/`
- [ ] **Phase 1 of provider-auto-config** — 6 provider 정적 등록 + Anthropic API key (env → keychain) + Ollama local detect
- [ ] **claude-code 2.1.169 changelog 대기** — 공개 시 Anthropic fallback 동작 검증 (D-34 §11.2)
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
