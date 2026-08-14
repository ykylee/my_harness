# my_harness 개발 백데이터 (Development Log / Report Backdata)

> **용도**: yklee 의 보고/리뷰/외부 공유용 raw data. 본 문서만 읽어도 my_harness 프로젝트의 컨셉·의사결정·개발흐름이 복원 가능하도록 self-contained.
>
> **갱신 정책**: 매 milestone 마다 §2 의사결정 + §3 타임라인에 append. 절대로 기존 항목 수정/삭제 ❌ (audit trail). 1차 작성: 2026-06-07.

---

## 0. 메타 (Meta)

- **프로젝트**: `my_harness` — yklee 의 개인 코딩 에이전트 CLI/TUI
- **저장소**: `https://github.com/ykylee/Devhub_example.git` (GitHub) + `https://homelab.ddn777.synology.me/gitea/yklee/my_harness` (Gitea mirror, private)
- **런타임**: Mavis / MiniMax Code
- **오버레이 표준**: `standard_ai_workflow` v0.5.0-beta + `minimax-code` harness overlay
- **시작일**: 2026-06-05
- **대상 플랫폼**: Windows / Linux / macOS 동시 지원
- **스코프**: 3-도메인 — (a) 코드 개발 전반, (b) 기본 서버 관리, (c) 환경 셋업

---

## 1. 컨셉 (Concept)

### 1.1 시작 컨셉 (2026-06-05 ~ 06-05 22:00)

처음 의도는 "외부 4-워커 워크플로우(Claude/Codex/Gemini/OpenCode) 를 우리 하네스 (mini_coder_max · fullstack-dev 등) 가 컨슈밍하는 표준 운영체계". 즉, Mavis 의 mavis-team 으로 외부 워커에게 작업을 위임하고, 결과/리뷰/세션 상태를 우리 하네스가 추적·기록하는 **오케스트레이터 역할**.

### 1.2 1차 방향 전환 (2026-06-05 22:00) — 컨슈머 → 직접 개발

yklee 가 판단: 외부 워커 컨슈밍만으로는 **yklee 만이 진화 가능한** 도구가 안 됨. **my_harness 자체가 yklee 가 직접 개발/배포하는 CLI/TUI 코딩 에이전트** 가 되어야 함. 즉:
- 4-워커 (외부) 와 **직교** — 우리 하네스는 **운영 정책/세션 추적/상태 동기화** 담당
- 도메인별 (코드/서버/환경) 명령이 내장된 CLI/TUI
- 표준 AI 워크플로우(state.json, handoff, backlog) 가 백엔드를 지탱

### 1.3 적용 표준

- **`standard_ai_workflow`** v0.5.0-beta: 표준 6필드 헤더, 한국어 보고, 컨텍스트 절약, 이벤트 소싱, 비참조 원칙, 상태값 `planned|in_progress|blocked|done`
- **`minimax-code` harness overlay**: `.MiniMax/agents/` 5종 워커, `MiniMax.md` 진입점, `ai-workflow/core/`
- **외부 워커 division**: Claude(아키텍처/리뷰), Codex(구현), Gemini(보안/대안), OpenCode(자동화) — `docs/governance/worker_division.md`

### 1.4 핵심 차별점 (현재까지)

1. **토큰 한계 문제** 를 context compression 으로 해결 → headroom library 1순위 검토
2. **다중 reference 분석** 기반의 의사결정 (TASK-004 1차 8축 + 2차 14섹션 × 6 reference)
3. **standard AI workflow** + **minimax-code** 듀얼 오버레이 (다른 어떤 CLI 도구도 안 함)
4. **Gitea 미러** 통한 reference repo 진화 추적 (private homelab)

---

## 2. 핵심 의사 결정 (Key Decisions) — append-only

> 형식: `날짜 | 결정 | 이유 | 트레이드오프`

| # | 일자 | 결정 | 이유 | 트레이드오프 |
| - | ---- | ---- | ---- | ----------- |
| D-01 | 2026-06-05 | `standard_ai_workflow` v0.5.0-beta + `minimax-code` 오버레이 적용 | yklee 의 기존 표준 + minimax-code 의 Mavis 통합 | 2개 표준 동시 진화 시 sync 부담 |
| D-02 | 2026-06-05 | 3-도메인 스코프 (코드/서버/환경) 확정 | yklee 의 실제 사용 패턴 (CLI 도구 + 서버 운영 + 셋업) | 도메인 추가 시 재설계 |
| D-03 | 2026-06-05 22:00 | **방향 전환**: 단순 워크플로우 컨슈머 → **CLI/TUI 직접 개발** | yklee 만이 진화 가능한 트리 필요 | 작성/유지보수 부담 ↑ |
| D-04 | 2026-06-05 | 4-워커 (Claude/Codex/Gemini/OpenCode) division 룰 도입 | 각 워커 강점 분리 | 룰 업데이트 시 워크플로우 영향 |
| D-05 | 2026-06-06 | 5개 reference clone + Gitea 미러 (opencode/aider/codex/goose/gemini-cli) | TASK-004 심층분석 + Gitea 진화 추적 | ~1GB storage + Gitea 운영 |
| D-06 | 2026-06-06 | **Gitea PAT macOS keychain 보관** (global `credential.helper=osxkeychain`) | 토큰 값 메모리/문서/git 저장 금지 정책 | 토큰 회전 시 매번 yklee 가 재발급 |
| D-07 | 2026-06-06 | **dual-remote 구조** (origin=Gitea, upstream=GitHub) | push 시 Gitea 우선, GitHub sync 는 수동 | remote URL 관리 부담 |
| D-08 | 2026-06-06 | **unshallow** Gitea push 시 (`--depth 1` → `unshallow`) | Gitea 1.25.5 가 shallow clone 거부 | push 시간/대역폭 |
| D-09 | 2026-06-06 | TASK-004 1차: 5 reference × 8축 비교표 | TASK-005 스택 결정의 1차 입력 | 8축은 정성적, 정량 검증 별도 |
| D-10 | 2026-06-06 | **5-심층분석 owner 직접 takeover** (worker long Write abort) | 1500줄+ 단일 Write 가 worker 세션 errored | 4-5h owner 작업 시간 |
| D-11 | 2026-06-06 | **claude-code 추가** (anthropics/claude-code 정식 repo + 2차 분석) | TASK-004 reference 보강 | (없음) |
| D-12 | 2026-06-06 | **claude-code 유출 repo 미클론** 결정 | IP 민감, 정식 repo + 2차 분석으로 충분 | 유출된 패턴 일부 미반영 |
| D-13 | 2026-06-07 | **PROVIDERS.md** 작성 (rig-core / Vercel AI SDK / litellm proxy 3-way 비교) | TASK-005 스택 결정의 provider 추상화 입력 | 결정 보류 (실측 필요) |
| D-14 | 2026-06-07 | **headroom 6번째 reference 추가** | context compression (토큰 한계 해결) insight 필요 | 분석 시간 (~2h) |
| D-15 | 2026-06-07 | **headroom 분석 owner 직접 takeover** (worker abort 2회) | chunked write 전략 + early deliverable signal 적용했으나 Edit append 중 abort | (D-10 와 동일) |
| D-16 | 2026-06-07 | **chunked write 전략 영구화** (worker long Write abort 대응) | agent memory 기록 | chunk 수 결정 부담 |
| D-17 | 2026-06-07 | **백데이터 문서** (본 문서) 신설 | 보고/리뷰용 self-contained 레퍼런스 | doc 자체 유지보수 부담 |
| D-18 | 2026-06-07 | Gitea `headroom` private repo push 완료 | dual-remote 동일 정책 | (D-07 과 동일) |
| D-19 | 2026-06-07 | mavis 환경 (`XDG_CONFIG_HOME=/Users/yklee/.mavis/...`) 의 gh CLI macOS keychain fallback 충돌 → `~/.mavis/agents/mavis/gh` → `~/.config/gh` symlink | mavis 격리 환경에서도 keychain 정상 사용 | symlink 유지 (mavis 재시작 후에도) |
| D-20 | 2026-06-07 | **Gitea + GitHub dual-remote 첫 push** — `origin=https://homelab.ddn777.synology.me/gitea/yklee/my_harness.git` (private) + `upstream=https://github.com/ykylee/my_harness.git` (public) | dual-remote 정책 (D-07) my_harness 레포에도 적용 | GitHub repo public 노출 (의도된 외부 미러링) |
| D-21 | 2026-06-07 | **claude-code 7번째 reference 통합** — `docs/references/claude-code.md` (1,029줄, 14섹션, closed source 분석, 13.1-13.26 adopt + 13.27-13.37 anti-pattern) + `docs/references/README.md` (7-doc 통합 인덱스 + 8축 비교 매트릭스 + my_harness 영향 분석) | claude-code 누락 확인 → 공개 분석 자료(arxiv 2604.14228, Zain Hasan blog, CSDN, Reddit leak analysis) + repo 의 inspectable 부분(plugin 12개 + CHANGELOG 4,263줄) 결합 | closed source 의 leak 분석 의존 (❓ 표시) |
| D-22 | 2026-06-07 | **my_harness v1 컨셉 확립** — `docs/CONCEPT.md` (마스터 SSOT) 신설. 7 reference 분석 종합 + yklee 작업 컨셉 + TASK-005/002/007 결정 입력 통합. 12섹션 (positioning/타겟/가치/스코프/v1 MVP spec/v2+ 로드맵/채택 23/안티 6/KPI/리스크/Open decisions/참조) | 7-doc 분석 후 컨셉 통합 필요. 각 문서가 독립 결정하면 일관성 깨짐. 단일 SSOT 필요. | 컨셉 갱신 시 관련 문서 (MiniMax/PROJECT_PROFILE/REFERENCES/PROVIDERS) 도 함께 align 필수 |
| D-23 | 2026-06-07 | **기존 문서 align to CONCEPT.md** — `MiniMax.md`, `PROJECT_PROFILE.md`, `REFERENCES.md`, `PROVIDERS.md` 의 메타데이터 + 도메인별 명령 섹션 + 7-doc 확장 섹션 + claude-code 3-fallback 패턴 추가. 각 문서 본래 목적 유지 + CONCEPT.md SSOT 참조 추가. | 7-doc 분석 결과가 기존 문서에 미반영. 동기화 필요. | 향후 컨셉 갱신 시 4 문서 동시 align 룰 |
| D-24 | 2026-06-07 | **CONCEPT.md 컨셉 교정 1차** — "외부 4-워커 통합/오케스트레이션" framing 제거. my_harness = standalone harness tool 로 재확립. Mavis 가 spawn 가능한 worker framing 추가. | yklee 가 "외부 4-워커 운영은 맞지 않음" 교정 — my_harness 는 sibling standalone tool 일 뿐 | (D-25 에서 더 보강) |
| D-25 | 2026-06-07 | **CONCEPT.md 컨셉 교정 2차 (Mavis zero coupling)** — §0.5 다이어그램에서 Mavis/orchestrator/standard_ai_workflow 모두 제거. §2 타겟 사용자에서 Mavis 행 삭제. §5.8 "외부 의존성 없음" 섹션 신설. **my_harness = 100% standalone, Mavis/Mavis/mavis-team/standard_ai_workflow 어느 것과도 결합 없음**. 유일한 런타임 의존 = LLM provider API + (선택) headroom MCP | yklee 가 "Mavis 랑도 관계 없이 동작되어야" 교정 — my_harness 는 Mavis 와 zero coupling. yklee 가 my_harness 개발 시 Mavis 를 dev tool 로 쓸 수는 있으나 my_harness 자체는 Mavis 를 모름. | 향후 docs 에 Mavis 언급 시 my_harness 본체 vs my_harness 개발 workflow 구분 필수 |
| D-26 | 2026-06-07 | **standard_ai_workflow 준수 (native + 옵션 통합)** — §5.9 신설. **6 원칙 native 구현** (한국어 보고 / 컨텍스트 절약 / 상태값 / 이벤트 소싱 / 비참조 / handoff 형식 — 항상 동작) + **옵션 Mavis 통합** (auto-detect 로 `ai-workflow/memory/` 발견 시 sync, 미발견 시 자체 `.myharness/` 만 사용). Task/handoff 출력 형식 Mavis 호환. Zero coupling 유지 (Mavis 디렉토리 없어도 동작) | yklee 가 "우리 하네스는 기본적으로 standard ai workflow를 준수해서 동작되도록 해볼 수 있을까?" — native 준수 + 옵션 통합의 하이브리드 채택. | 향후 my_harness 가 Mavis 와 동시 사용 시 호환성 자동 보장 |
| D-27 | 2026-06-07 | **headroom = built-in 압축 layer (외부 proxy 의존 X)** — §0.5 NOT list + §3.3 + §5.6 갱신. **흐름: user → my_harness → (built-in 압축) → LLM provider**. headroom 의 6 알고리즘 (CacheAligner/ContentRouter/CCR/SmartCrusher/CodeCompressor/Kompress-base) 을 우리 Context component 에 **built-in 으로 내장**. 외부 headroom proxy/MCP 의존 안 함. 기본 off (사용자 opt-in). v1 우선 3개 (CacheAligner + ContentRouter+SmartCrusher + CodeCompressor), v1.5+ CCR/Kompress-base. | yklee 가 "사용자 - harness - (headroom) - llm provider 의 순서" 제안 + "proxy 방식은 제약 있음" → built-in 으로 우리 Context component 에 심기. | 향후 headroom upstream 변경 시 우리 코드만 갱신. 외부 의존성 0. |
| D-28 | 2026-06-07 | **Provider 6개 확정 (5 named + 1 local) + OpenAI 호환 lingua franca** — §5.5 LLM 통합 전면 갱신. **claude/codex/gemini = native SDK** (각 vendor 최적 기능), **deepseek/minimax/local-llm = OpenAI 호환 client** (1개 구현으로 N개). 3 fallback model (D-15) + 도메인별 mapping + per-provider config (api_key_env / secret_store / supports). 모델 prefix 규약 (`anthropic/claude-sonnet-4-5`, `ollama/qwen2.5-coder:32b` 등). | yklee 의 5개 사용 provider (codex, gemini, minimax, deepseek, claude) + local LLM 요구. minimax 는 D-28 TBD (base_url + API 형식 검증 필요). | 향후 provider 추가 시 동일 패턴 (native or openai-compatible 등록). plugin 으로 사용자 정의 provider 도 가능 (v1.5+). |
| D-29 | 2026-06-07 | **Agent 3 모드 + Built-in sub-agents** — §5.10 + §5.11 신설. **3 모드** (orchestrator 기본 / single opt-in / loop opt-in ralph-wiggum 패턴, `--mode=orchestrator\|single\|loop` + `--goal/--max-iterations/--success-criteria`). **Built-in sub-agents ~15개** (3-도메인 × 4-5: code-reviewer/implementer/tester/refactorer, server-status/log-analyzer/deployer/config-manager, env-setup/installer/shell/diagnose, git-operator/file-searcher). Orchestrator 가 user 명령 분석 → 카테고리 매칭 → sub-agent spawn. | yklee 가 "기본 에이전트는 오케스트레이터 역할로 동작하고 주요 작업은 작업 카테고리마다 서브 에이전트를 내장해서 작업 분배 및 context 효율화" + "모드 변경에 따라 단일 에이전트 모드" + "목표를 설정하면 목표 달성까지 무한루프" 요구. | 향후 TASK-005-1 (v1 MVP) 시 sub-agent 15개 중 우선 4-5개 구현. TASK-005-4 (v2.5) 에 multi-agent parallel. |
| D-30 | 2026-06-07 | **2-계층 Context 압축 (Layer 1 필수 + Layer 2 선택)** — §5.6 갱신. **Layer 1 (always-on)**: token budget 추적 → 한계 80% 도달 시 auto truncate/summarize → /compact (manual). opt-out 불가 (model 자체가 길이 제한). **Layer 2 (opt-in)**: headroom 6 알고리즘 built-in (CacheAligner/ContentRouter/CCR/SmartCrusher/CodeCompressor/Kompress-base) — `builtin.enabled: true\|false` 기본 false. | yklee 가 "모델 length 한계에 따른 자동 context 압축 기능은 필수로 필요" — Layer 1 must, Layer 2 optional 분리. | v1 시 Layer 1 구현 필수, Layer 2 는 plugin 형태로 opt-in. |
| D-31 | 2026-06-07 | **`~/.myharness/` 디렉토리 구조 확립** — §5.12 신설. yklee 환경 검증: 다른 agent 도구 모두 `~/.<toolname>/` 컨벤션 (claude/codex/gemini/headroom/minimax/jules/coderabbit). 우리도 동일 컨벤션 + XDG-style 내부 분리 (config/state/memory/handoff/log/compression/sub-agents/llm-wiki/runtime/cache). Cross-platform (mac/linux/win 동일). 옵션 Mavis 디렉토리 발견 시 sync (D-26). | yklee 가 "user-specific 디렉토리 구조 보고 운영 환경 컨텍스트 잡아둬" — sibling tool 컨벤션 + 우리 v1 spec 정합. | v1 시 디렉토리 자동 생성, v1.5+ marketplace. |
| D-32 | 2026-06-07 | **LLM Wiki memory layer (Karpathy pattern, v2+)** — §5.13 신설. 3 계층 (raw/wiki/schema) + index/log/lint 운영. "Obsidian is the IDE, the LLM is the programmer, the wiki is the codebase". v1 = 기본 flat memory (LLM Wiki 미적용), v1.5+ = schema+lint, v2.5+ = full compile + cross-reference. Reference: `gist.github.com/karpathy/442a6bf555914893e9891c11519de94f`. | yklee 가 "llm wiki 의 컨셉을 참고하면 어떨까?" — Karpathy 2026-04 LLM Wiki Pattern (3-layer architecture). | v1 simple memory, v2+ LLM Wiki 자동 운영. |
| D-33 | 2026-06-07 | **Skill/MCP first-class (claude-code/goose 동급)** — §5.14 신설. **Skills** = `~/.myharness/skills/<name>/SKILL.md` (claude-code 13.3) + built-in 6 skills (3-도메인). **MCP** = `~/.myharness/mcp.json` + `rmcp` (Rust) / `@modelcontextprotocol/sdk` (TS) + 4 pre-config server (filesystem/git/shell/github). MCP tool 자동 노출 (`mcp__*`). | yklee 가 "skill/mcp 지원은 다른 도구와 동등하게" 요구. | v1.5+ marketplace, plugin 으로 사용자 정의. |
| D-34 | 2026-06-07 | **TASK-NNN 형식 통일 + 2.1.169 pending 표** — §6 v2+ 로드맵 v1.0/v1.5/v2.0/v2.5/v3.0 마일스톤 → **TASK-005-1~TASK-005-5** (sub-task of TASK-005 스택 결정). §11.1 결정 보류 표의 TUI 라이브러리 → **TASK-006**, Provider fallback list → **TASK-008** (TASK-007 headroom 사이 번호 보존). **§11.2 신설** = claude-code 2.1.169 영향 결정 pending 표 (context var/cache → §5.6/D-32, MCP → §5.14/D-33, permission → §5.4, fallback → §5.5/D-15). 2.1.169 changelog 공개 시 검증. | yklee 가 "마일스톤 TASK-005-1 형식으로" + "2.1.169 영향 결정 §11.1 관련 항목으로" 요구. | 향후 task_id 추적 가능, 외부 reference 변경 시 영향 결정 즉시 식별. |
| D-35 | 2026-06-07 | **관련 문서 align to CONCEPT.md v1 (D-22~D-34)** — 4 docs 일괄 갱신. `MiniMax.md` + `docs/PROJECT_PROFILE.md` 의 관련 문서 에 §5.10~§5.14 + §11 결정 보류 cross-ref 추가. `README.md` (root) 전체 재작성 — my_harness v1 산출물 vs 개발 workflow 분리 (D-25) 명확화 + v1 핵심 컨셉 10개 결정 table + v1 CLI 명령 (§5.2, §5.10) + v1 산출물 디렉토리 구조 (D-31) + TASK-NNN 다음 결정 (§11.1). `docs/references/PROVIDERS.md` 관련 섹션에 §5.14 (MCP), §11.2 (2.1.169 pending) cross-ref 추가. | yklee 가 "관련 문서 align으로 해" 요구. 기존 4 문서가 D-22~D-34 결정 미반영. | 향후 D-23 (이전 align) + D-35 (이번 align) = align 룰 확립. 컨셉 갱신 시 4 문서 동시 갱신. |
| D-36 | 2026-06-07 | **TASK-005 결정: Rust 1안 (스택)** — yklee 결정. v1 MVP = **Rust 1안** (codex/goose 모델). §11.1 갱신 (TASK-005, TASK-006 → ✅ 결정 완료), **§11.3 신설** (결정 근거 + v1 스택 종합: Rust 2024 + ratatui + rig-core + rmcp + keyring + cargo-dist). TASK-006 자동 확정 = ratatui + crossterm. PROJECT_PROFILE.md 적용 환경 에 Rust 1안 명시. **v1 스택 결정 후속 작업**: TASK-005-1 (v1 MVP 빌드) = cargo workspace init → ratatui shell → rig-core Anthropic → basic tools → /compact → standard_ai_workflow output. | yklee 가 "1안으로 우선 결정하자" — 8개 선정 근거 (단일 binary / TUI 검증 / MCP 성숙 / keychain 안정 / 빠른 startup / provider 비종속 / headroom native / Desktop 확장). | 향후 TASK-005-1~TASK-005-5 순차 진행. TS 2안 으로 변경 시 재검토. |
| D-37 | 2026-06-07 | **TASK-007 결정: headroom v1 우선순위 1안 유지** — yklee 결정. v1 = **3 알고리즘** (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor). **CCR + Kompress-base 는 v1.5+ (다음 개발 페이즈) 로 연기**. §11.1 TASK-007 ✅ 결정 완료 표시. | yklee 가 "1안 유지하자. 나머지는 다음 개발 페이즈에서 진행" — 3 알고리즘 권장 유지 (단일 LLM call latency 와 round-trip 비용 충돌 회피, ONNX 모델 weight 부담 회피, binary size 가벼움). | CCR + Kompress-base 는 v1.5+ (TASK-005-2) 또는 그 이후 페이즈에서 재검토. v1 단계에서는 Layer 1 (필수 자동 압축, D-30) + Layer 2 의 3 알고리즘 으로 충분. |
| D-38 | 2026-06-07 | **TASK-008 결정: 하드코딩 fallback 폐기 → `provider-auto-config` skill (동적 발견 + per-provider auth)** — yklee 결정. **하드코딩 fallback list 폐기**, 런타임 discovered list + per-provider auth. **§5.5 전면 갱신** (정적 config → 동적 discover + auth, 4 subsections: 지원 provider / 동적 발견+auth / fallback chain / 라이브러리). **`docs/skills/provider-auto-config/SKILL.md` 신설** (reference design, v1 구현 시 base) — auto-invoke trigger, discover 로직 (env/keychain/local server), per-provider auth state, CLI interface (`myharness auth list/login/logout/set-key/test`), active-providers.yaml, v1 Phase 1/2/3 분리, Rust sample code. §5.14 Built-in skills 에 `provider-auto-config` 추가. §11.1 TASK-008 ✅ 결정 완료 + §11.3 TASK-008 결정 근거 5개 + v1 Phase 1/2/3 분리. | yklee 가 "모델 구성 및 배치는 상황에 따라 달라질 수 있으니 auth가 연결된 프로바이더 및 로컬 구성을 토대로 동적으로 구성할 수 있도록 스킬을 만들어두자. 그리고 각 프로바이더 별 auth 관련 기능도 제공해야해" — 환경 가변성 / 사용자 개입 최소화 / 확장성 / local-first / graceful degrade. | Phase 1 (TASK-005-1 MVP) = 6 provider 정적 + Anthropic key + Ollama detect + hardcoded fallback. Phase 2 (TASK-005-2 v1.5) = `provider-auto-config` skill 정식 구현 + dynamic fallback. Phase 3 (TASK-005-3 v2.0) = OAuth + MCP-based discover. |
| D-39 | 2026-06-07 | **세션 마무리 (v1 컨셉 Phase 종료)** — `session_handoff.md` / `work_backlog.md` / `state.json` 갱신. **5/5 결정 검토 완료** (TASK-002 ⏸ 보류 + TASK-005/006/007/008 ✅). 8 done tasks. 다음 세션 시작점 = **TASK-005-1 (v1 MVP Rust 빌드)**. backlog 3 files 갱신, state.json 에 decisions 섹션 신설 (decided 4 / deferred 1). | yklee 가 "세션 마무리 준비하자" — standard_ai_workflow 운영 원칙 (handoff / state / backlog 갱신). | 다음 세션 TASK-005-1 시작. v1 Rust 구현 시점에 TASK-002 (도메인별 명령) 자연 도출 가능. |
| D-40 | 2026-06-07 | **§11.2 (claude-code 2.1.169 pending 검증) 취소** — yklee 결정. 2.1.169 changelog 공개 시 검증 안 함, **v1 spec 잠금 (Rust 1안 / ratatui / headroom 3 algo / provider-auto-config)**. §11.2 섹션 완전 제거. §11.3 TASK-008 의 "D-34 (2.1.169 영향) — Anthropic fallback 동작 검증 후 Phase 2 에 반영" → "**D-40 으로 취소, 검증 미진행** (v1 spec 잠금)" 으로 갱신. | yklee 가 "claude-code 2.1.169 검증 안할거야 11.2 내용 확인하고 취소해" — 2.1.169 영향 미검증 결정. 공개 채널 (GitHub release, feed.xml, CHANGELOG.md) 모두 v2.1.168 까지만 노출 + 검증 부담 회피 + v1 spec 의존성 제거. | 향후 2.1.169 이상 변경 시점에 v1 spec 영향 별도 평가 (현재 v1.5+ 에서 처리). |
| D-41 | 2026-06-09 | **TASK-005-1 환경 검증 완료** — Linux x86_64 / Rust 1.94.1 / 12+ crate 가용. Prerequisite 5건 (libsecret-1-dev, 5 cross target, cargo-dist/binstall, ANTHROPIC_API_KEY, serde_yml) 설치 후 cargo workspace init 진입 | 환경 검증 단계라 blocker 없음. API key 미설정으로 LLM E2E 는 키 주입 후. | (없음) |
| D-42 | 2026-06-09 | **config 포맷 = TOML 통일** — v1 의 모든 사용자 편집 config / state / provider registry 를 TOML 로 통일. `serde_yaml` 0.9.34-deprecated + `serde_yml` 0.0.13-deprecated 회피. JSON 파일 (auth snapshot / metrics / mcp.json / log.jsonl / state.json) 은 유지. CONCEPT.md §5.12 / §5.5 / §5.6 19곳 갱신. | yklee 가 (a) TOML 만 결정 — single-user + TUI config 충분 + Cargo 표준 TOML 안정성. | CONCEPT.md 표면 일관성 ↑ / 1개 crate 의존성 (`toml`) 추가. |
| D-43 | 2026-06-09 | **TASK-005-1 W3~W6.5 완료 + Gitea push** — myharness-tools crate 1차 완성: 5 tool (Read/Write/Edit/Bash/Grep/Glob) + 4 permission mode (default/acceptEdits/plan/bypassPermissions) + 9 위험 패턴 sanitizer (Strict/Permissive/Off) + JSON Schema (schemars 1.2) + 5 provider wire format (Anthropic/OpenAI/DeepSeek/Ollama/llama.cpp/litellm). 9 commit (3f0c9cb~dfc9d93) Gitea push. 63 tests passed. | yklee 결정 — (a) 5 tool foundation, (b) 4 permission mode 적용, (c) Bash sanitization, (d) JSON schema + dispatch, (a) full OpenAI 호환 검증 + 보강. librarian 조사 결과로 W6.5 의 3건 사실 정정 (DeepSeek Beta URL, $schema draft-07, Response side). | tool_use 호환성 ↑, 5 provider 모두 wire format 지원. Gitea 단일 remote (D-20 미적용) 한계. |
| D-44 | 2026-06-09 | **dual-remote (D-20) 적용** — `origin=https://homelab.ddn777.synology.me/gitea/yklee/my_harness` (Gitea, private) + `upstream=https://github.com/ykylee/my_harness` (GitHub, public). Gitea PAT `myharness-cli` 발급 (scopes: write:repository, write:user) in `~/.git-credentials` (chmod 600). GitHub auth via gh CLI (ykylee, scopes: repo/workflow/gist/read:org). 9 commit 양쪽 push 완료. | yklee 가 Gitea password 제공 + GitHub 인증 확인 요청. D-20 정책 (2026-06-07 수립, 그 동안 미적용) 완전 이행. | GitHub public 노출은 의도된 외부 미러링 (D-20). gh CLI `--show-token` 으로 GitHub PAT 가 stdout 노출됨 — 회전 권고. |
| D-42 | 2026-06-09 | **config 포맷 = TOML 통일** (D-42) — v1 의 모든 사용자 편집 config / state / provider registry 를 TOML 로 통일. `serde_yaml` 0.9.34-deprecated + `serde_yml` 0.0.13-deprecated 회피. JSON 파일 (auth state / metrics / mcp.json / log.jsonl / state.json) 은 유지. CONCEPT.md §5.12 / §5.5 / §5.6 19곳 갱신. | yklee 가 (a) TOML 만 결정 — single-user + TUI config 충분 + Cargo 표준 TOML 안정성. | CONCEPT.md 표면 일관성 ↑ / 1개 crate 의존성 (`toml`) 추가. |
| D-43 | 2026-06-09 | **TASK-005-1 W3~W6.5 완료 + Gitea push** (D-43) — myharness-tools crate: 5 tool (Read/Write/Edit/Bash/Grep/Glob) + 4 permission mode (default/acceptEdits/plan/bypassPermissions) + 9 위험 패턴 sanitizer (Strict/Permissive/Off) + JSON Schema (schemars 1.2) + 5 provider wire format (Anthropic/OpenAI/DeepSeek/Ollama/llama.cpp/litellm). 9 commit (3f0c9cb~dfc9d93) Gitea push. 63 tests passed. Gitea PAT 설정은 다음 세션 보류. | yklee 결정 (a) full 검증 + 보강 (OpenAI 호환 wire format librarian 조사 결과 반영). | tools crate 1차 완성. 다음: W7 (llm crate 진입, rig-core 0.38 + Anthropic provider). |
| D-45 | 2026-06-09 | **TASK-005-1 W7 (myharness-llm crate v1) 완료** (D-45) — 6 provider enum + ProviderRegistry + LLMClient trait + rig-core 0.38 Anthropic/Gemini wrapper + OpenAI 호환 (DeepSeek/Ollama/local-llm) wrapper + MockClient + AuthState/AuthStatus + InMemory/Keyring AuthStore (libsecret 부재 환경 graceful fallback) + provider-auto-config discover (env+keychain+local scan) + ActiveProviderChain + FallbackRouter (cascade + per-provider status + retry policy). 87 tests pass. release 빌드 성공. 5 commit (29052ad~7264589) → Gitea origin + GitHub upstream push. ANTHROPIC_API_KEY absent 환경에서 mock-driven 검증. | yklee 결정 (a) 3안: 전체 §5.5 + provider-auto-config skill v1 simple (W7 단독 범위). | CONCEPT.md §5.5 spec 그대로 구현. 다음: W8 (context, CLAUDE.md + memory + /compact Layer 1) 진입. |
| D-46 | 2026-06-09 | **TASK-005-1 W8 (myharness-context crate v1) 완료** (D-46) — CLAUDE.md loader (project root + parent walk + global fallback) + auto memory (NDJSON append-only) + ContextManager (token budget + /compact Layer 1: Truncate/Summarize-stub/Hybrid) + Layer 2 BuiltinPipeline (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor, 기본 off) + ContextConfig (config.toml [context] 섹션) + ContextOrchestrator (전체 통합). 54 tests pass. release 빌드 성공. 5 commit dual-push. | TASK-005-1 spec 따른 자동 진행 (yklee 결정 불필요). | context crate v1 완성. 다음: W9 (compression, Layer 1 정식 + Layer 2 CCR/Kompress-base) 또는 W10 (TUI shell) 진입. |
| D-47 | 2026-06-09 | **TASK-005-1 W9 (myharness-compression crate v1) 완료** (D-47) — Summarizer (LLM-driven summarize, ContextManager Hybrid 정식 통합) + CCR (Claude Code Recursive split, 단일 round-trip 비용 1회 trade-off) + Kompress-base v1 simple (트리머리 휴리스틱) + BuiltinRegistry (6 알고리즘: CacheAligner / ContentRouter / SmartCrusher / CodeCompressor / CCR / Kompress-base) + Layer 1/2 Pipeline 통합. 40 tests pass. release 빌드 성공. 5 commit dual-push. | TASK-005-1 spec 따른 자동 진행. | context Layer 1 Summarize/Hybrid 가 LLM-driven 으로 정식 작동. 다음: W10 (TUI shell). |
| D-48 | 2026-06-09 | **TASK-005-1 W10 (myharness-tui crate v1) 완료** (D-48) — ratatui App + crossterm backend + 4 SubAgent (code-reviewer / code-implementer / code-tester / git-operator, 3-도메인 code 도메인 1차) + Orchestrator (서브에이전트 dispatch + 결과 통합) + LoopRunner (--mode=loop --goal --max-iterations) + cli 통합 (code/env/git/ask subcommand) + TUI test backend (buffer). 51 tests pass. release 빌드 성공. 5 commit dual-push. | TASK-005-1 spec 따른 자동 진행. | TUI shell 완성. 다음: W11 (core crate, standard_ai_workflow output + 4 permission 완성). |
| D-49 | 2026-06-09 | **TASK-005-1 W11 (myharness-core crate v1) 완료** (D-49)** — standard_ai_workflow 6 원칙 native (한국어/절약/상태/이벤트/비참조/handoff) + 4 permission mode (default/acceptEdits/plan/bypassPermissions) + tool name alias (Read/Grep/Glob 대문자 ↔ read/grep/glob_ 소문자) + MockClient FIFO (sequential mock response) + Orchestrator fatal_llm_error (LLM 호출 실패 시 fatal 종료) + cli task start|end subcommand + handoff writer. 32 tests pass. 5 commit dual-push. **v1 MVP 6/8 waves 완료** + cli entry 통합. | TASK-005-1 spec 따른 자동 진행. | v1 MVP 6/8 waves. 다음: W12 (MiniMax LLM API 연결, D-50) + W13 (OAuth 2.0 headless auth, D-51). |
| **D-50** | 2026-06-09 | **TASK-005-1 W12 (MiniMax LLM API 연결) 완료** (D-50) — librarian 조사 (⭐⭐⭐⭐⭐ 5/5): `https://api.minimax.io/v1` OpenAI-호환, `MiniMax-M3` default, 7 models (M3/M2.7/M2.7-highspeed/M2.5/M2.5-highspeed/M2.1/M2), Bearer token (MINIMAX_API_KEY env), tool_use 지원, no-CORS, JSON streaming. `ProviderMetadata::builtin_minimax()` 갱신 + `KeyringAuthStore` in-memory cache (libsecret 부재 fallback) + `MINIMAX_API_HOST` env override (base_url). cli default LLM = `MINIMAX_API_KEY` env 자동 detect → `OpenAiCompatProvider` 흐름 검증. 5 commit dual-push. | yklee 결정 (a) 1안: 6 provider 가 CONCEPT §5.5 spec 그대로, minimax 우선 active (W12 task). | minimax TBD (D-28) 해소. 다음: W13 (OAuth 2.0 headless auth, MiniMax/OpenAI/Google 3 provider). |
| **D-51** | 2026-06-09 | **TASK-005-1 W13 (myharness-auth crate v1) 완료** (D-51) — 7 모듈: `pkce` (RFC 7636 S256 + state) + `flow` (OAuth 2.0 Authorization Code + PKCE core, provider-agnostic) + `callback` (loopback HTTP server 127.0.0.1, 5min timeout, port 0 random) + `browser` (xdg-open/open/start) + `store` (`~/.myharness/oauth/{provider}.toml`, chmod 600, MYHARNESS_HOME env override) + `provider` (MiniMax/OpenAI/Google 3 provider, 모두 PKCE public client, client_secret 없음) + `manager` (AuthManager login/refresh/status/logout + AuthError::is_not_found()). 38 tests pass. cli auth subcommand (`auth list|login|logout|status`) 추가. 4 commit dual-push. **MiniMax OAuth endpoint 확정**: `https://api.minimax.io/oauth/authorize` + `/oauth/token`. 7bc0931 까지 누적 30+ commit dual-push. | yklee 결정 (a) 1안: OAuth 2.0 headless auth + 3 provider (W13 task). | OAuth headless auth v1 완성. 다음: W13.5 (env override) + W13.6 (mock e2e test) + yklee real OAuth flow (client_id 의존). |
| **D-52** | 2026-06-09 | **TASK-005-1 W13.5 (OAuth env override) + W13.6 (mock e2e test) 완료** (D-52) — W13.5: `OAUTH_PROVIDERS` static `LazyLock<HashMap<...>>` 제거 → `oauth_providers()` 매번 새 instance (env 변경 즉시 반영). 3 provider `from_env()` 생성자 추가. `MYHARNESS_OAUTH_CLIENT_ID_{MINIMAX,OPENAI,GOOGLE}` env override + `MINIMAX_API_HOST` base_url override. env 검증: `MYHARNESS_OAUTH_CLIENT_ID_MINIMAX=cp-test-12345` → authorize URL 에 정상 반영. W13.6: local mock HTTP server (`TcpListener` + raw HTTP request line) + `MockProvider` (trait `&self` receiver, custom endpoints) + `auth_manager_end_to_end_with_mock_server` test: build_authorize_url → reqwest get → 302 redirect → callback params → `exchange_code` → JSON token response → `TokenStore::save`. **real network 없이** 전체 OAuth 2.0 + PKCE flow 검증. CI 환경 가능. 39 auth tests pass (W13.6 +1). 2 commit dual-push (Gitea + GitHub). | TASK-005-1 spec 따른 자동 진행. | **v1 MVP 8/8 waves 완료**. 다음: TASK-005-1 종료 선언 또는 TASK-005-2 (v1.5) 진입 결정. |
| **D-53** | 2026-06-09 | **TASK-005-1 W14 (MiniMax Device Authorization Grant) 완료** (D-53, D-52 follow-up) — W13 의 MiniMax Authorization Code + PKCE redirect flow 가 실제로 404. **MiniMax OAuth 는 표준 redirect 가 아니라 Device Authorization Grant 변형 사용** (OpenClaw/Hermes 와 동일, RFC 8628 의 MiniMax 구현). W14 에서 전면 교체: 새 `MinimaxDeviceOAuth` provider + `DeviceCodeProvider` trait. `client_id = 78257093-7e40-4613-99e0-527b14b39113` (OpenClaw/Hermes 공통, 모든 client 가 동일 값). `scope = group_id profile model.completion`. region: 한국 default = global (`https://api.minimax.io`). CN: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env override. 새 `device_flow.rs` module: `request_code` (POST `/oauth/code`) + `poll_token` (POST `/oauth/token`) + `poll_until_success` (loop until expired_in). 기존 `MinimaxOAuth` (Authorization Code) 는 deprecated 표시만, 그대로 둠 (OpenAI/Google redirect flow 와 일관성). 1 commit dual-push. | yklee 가 MiniMax console 에서 redirect URL 등록 불가 확인 → device flow 가 MiniMax 의 정식 OAuth 패턴임을 발견 (OpenClaw/Hermes client_id 공통). | MiniMax OAuth 404 문제 해소. 다음: W14.4 (`--no-browser` 의미 수정). |
| **D-54** | 2026-06-09 | **TASK-005-1 W14.4 (`--no-browser` 의미 수정) 완료** (D-54, D-52 follow-up) — W14 의 `--no-browser` 가 polling 도 중단하던 버그. 의도: user 가 직접 URL 을 browser 에 paste 한 시나리오. `AuthManager::login_minimax_device` signature 변경: `interactive: bool` → `(auto_open_browser: bool, non_interactive: bool)`. 3 모드 확정: (1) **default** = URL 출력 + browser 자동 open + polling + save, (2) **`--no-browser`** = URL 출력 + polling + save (user 가 직접 paste), (3) **`--non-interactive`** = URL 출력만 + 즉시 종료 (CI/스크립트). cli `AuthAction::Login` 에 `--non-interactive` flag 추가. mock e2e test `login_minimax_device_no_browser_polling_saves` 추가 (POST /oauth/code + POST /oauth/token + TokenStore save). 42 auth tests pass, 367 workspace tests pass, 0 fail, 2 ignored. 1 commit dual-push. | yklee 의 "browser open 안 해도 user 가 URL paste 하면 polling 계속되어야" 요구. | device flow 3 모드 확정. 다음: W14.5 (polling output + expired_in 단위). |
| **D-55** | 2026-06-09 | **TASK-005-1 W14.5 (polling output + expired_in 단위) 완료** (D-55, D-52 follow-up) — W14.4 의 `--no-browser` 가 polling 시작 후 stdout 출력 없는 문제 + expired_in 단위 잘못 해석. (a) `login_minimax_device` polling 진입 전 URL + user_code + expires_in 항상 `tracing::info!` 출력 (browser 자동 open 무관). (b) `poll_until_success` OpenClaw 와 동일한 1.5x backoff + cap 10s + floor 1s. (c) `expired_in` 단위 수정: MiniMax 가 **milliseconds 단위 unix timestamp** 로 응답 (D-52 follow-up 확인). 우리 now 도 `timestamp_millis()` 로 비교. `/1000` 으로 wait_secs 표시. 검증: `./myharness auth login minimax --no-browser` 실행 시 URL + user_code + `expires_in: 299s` 정상 표시 + `mini_max ms-unix-ts=1780991724849` 디버그. mock e2e test `login_minimax_device_with_mock_server` 통과 (interval=1s). 42 auth tests pass. 1 commit dual-push. | W14.4 의 출력 부족 + polling 무한루프 가능성 발견. | device flow UX 개선. 다음: W14.6 (token 응답 단위 일관). |
| **D-56** | 2026-06-09 | **TASK-005-1 W14.6 (device_token_to oauth 단위 변환) 완료** (D-56, D-52 follow-up) — `device_token_to_oauth` 응답의 `expired_in` 이 ms 단위 unix timestamp 인데 우리 `TokenStore::save` 는 초 단위 `expires_at` (Unix seconds) 를 기대. ms → s 변환 추가. 42 auth tests pass. 1 commit dual-push. | W14.5 검증 중 ms/s 혼동 발견. | OAuth token 만료 시각 일관성 확보. |
| **D-57** | 2026-06-09 | **TASK-005-1 W15.a (OAuth token 자동 resolve) 완료** (D-57, D-52 follow-up) — W14 까지 `ask` 호출 시 `MINIMAX_API_KEY` env var 필수. OAuth 으로 `auth login` 했어도 env var 없으면 MockClient fallback. yklee 의 "수동 env 불편" 문제. cli LLM client builder 를 `resolve_llm_client()` helper 로 추출. opencode 의 multi-source credential chain 패턴 차용 (env vars + `~/.opencode/auth.json`). 우선순위: (1) `~/.myharness/oauth/minimax.toml` 의 OAuth access_token (env var 보다 우선) → (2) `MINIMAX_API_KEY` env var (regular API key) → (3) `ANTHROPIC_API_KEY` env var → (4) MockClient fallback. token 만료 시 WARN + env var fallback. **자동 refresh 는 W15.b** 에서 LLM client 레벨로 추가. 1 commit dual-push. | yklee 의 "OAuth login 후 env var 없이도 LLM 호출" 요구. | OAuth 사용성 개선. 다음: W15.b (자동 refresh) + TASK-005-1 종료 선언 또는 TASK-005-2 (v1.5) 진입. |
| **D-58** | 2026-06-09 | **TASK-005-1 W15.b (OAuth token 자동 refresh) 완료** (D-58, D-52 follow-up) — W15.a 의 "token 만료 시 env var fallback" 은 manual workaround. cli crate 에 `RefreshingLlmClient` wrapper 추가 (LLMClient trait 구현). **401 식별**: `LlmError::ProviderCall(msg)` 의 `msg.to_lowercase()` 에 "401" / "unauthorized" / "auth" (단 "oauth" 제외) 키워드 → 만료로 간주. **refresh 흐름**: `AuthManager::ensure_fresh(Arc<dyn OAuthProvider>)` 호출 → 만료 + refresh_token 있으면 새 token → store save → 새 `OpenAiCompatProvider(base_url, NEW_TOKEN, model)` 빌드 → **retry 1회**. retry 1회 한정 (무한루프 방지). **refresh_token 없으면**: expired token 그대로 retry → 401 surface. `resolve_llm_client()` OAuth 경로(1번)에만 wrap. 부수 변경: `TokenStore` / `AuthManager` `#[derive(Clone)]`, `cli/Cargo.toml` 에 `async-trait` + `tempfile` (dev) + `chrono` (dev) + `serde_json` (dev). 9 cli tests (`is_unauthorized_*` 6 + `with_no_stored_token` 1 + `without_refresh_token` 1 + `e2e_401_refresh_retry_200` 1). 388 workspace tests, 1 commit dual-push. | yklee 의 "long-running daemon 에서 OAuth token 자동 갱신" 요구. | OAuth 만료 자동 처리. 다음: TASK-005-1 종료 선언 또는 TASK-005-2 (v1.5) 진입. |
| **D-100** | 2026-06-30 | **방향 전환 + A-min text-based tool dispatch 1차 cycle** — oh-my-pi (`can1357/oh-my-pi`, v15.1.8, 11.1k~14.7k stars, MIT, Mario Zechner 의 `badlogic/pi-mono` fork) 를 **reference + 부분 차용 (Hybrid 안)** 으로 다듬기 결정 (yklee). 차용 후보: Hashline 편집 / Skill·Extension 시스템 / hindsight memory / LSP+DAP. 비차용 (유지): OAuth PKCE+Device / Local LLM cascade / R-4 backup / W15.a/b / standard_ai_workflow / 한국어 workflow / 백업 안전장치. **선결 = 사용 가능한 형태** (binary OK / init_home_dir OK / auth OK / **LLM credential 0건 P0** → `MINIMAX_API_KEY` 주입으로 해결). **A-min 1 cycle**: `agent.rs` +52 (tool_spec_section) + `orchestrator.rs` +194 (extract_tool_call + dispatch loop max 3 round) + 5 test 추가 (18/18 tui PASS, clippy clean). Real LLM `code review <file>` → 3 round 자동 dispatch / `env diagnose` → 3 round Bash. 1 commit (444633d, main 머지). 누적 결정 47 → **48**. | TASK-005-2 v2.0 자체 진척 vs 외부 reference 의 v1.5+ 채택 — Hybrid 가 trade-off 최소화. | max_round 3 → 5/10, Bash 결과 visible, A-proper native tool calling (v1.5+) 미적용. 다음: A-min polish + prompt 개선. |
| **D-101** | 2026-06-30 | **A-min follow-up polish** — 3가지 fix: (1) `max_tool_rounds 3 → 10 default` + `with_max_tool_rounds(n)` builder. (2) tool result stdout **visible in response** (2000자 truncation, 이전엔 `[tool_call] X → ok` 마커만). (3) `dispatch_tool_call` 에 `with_confirm_override(true)` 추가 — AcceptEdits + confirm_override → Bash prompt skip (비대화형 hang 방지). 4 test 추가. **22/22 tui PASS (D-100 18 + D-101 4 신규)**, clippy clean. Real LLM `env diagnose` → 10 round 자동 dispatch + `[tool_result]` 에 uname/PATH/whoami/pwd stdout visible + prompt 안 뜸. 누적 결정 48 → **49**. 1 commit (0a9d94d dual-push). | D-100 의 max_round 3 부족 + 비대화형 환경 Bash prompt hang 위험. | max_round 10 도 큰 file 부족 + LLM 같은 Bash 반복 (prompt 개선 필요) + A-proper native tool calling 미적용. 다음: prompt 개선 + dedup. |
| **D-102** | 2026-06-30 | **Prompt 개선 + dedup 안전망 (LLM 무한 루프 방지)** — D-101 follow-up 에서 `env diagnose` 가 max_round 10 round 도 같은 Bash 명령 (PATH, whoami, pwd) 반복 호출 → 무한 루프. 2가지 fix: (1) `tool_spec_section` Stop conditions 4가지 (enough info / same tool+args 반복 / last 2-3 similar / previous turn covered) + safety net 명시. (2) `canonical_tool_call(name, args)` helper (BTreeMap key 정렬, 순서 무관) + `call_counts: HashMap` 추적 + **2회 중복 시 synthetic final prompt** + break. 5 test 추가. **27/27 tui PASS (D-100 18 + D-101 4 + D-102 5 신규)**, clippy clean. Real LLM `ask "1+1은?"` → LLM tool 안 쓰고 즉시 plain 응답 (`2입니다.`) — prompt stop condition 작동 확인. 2회 중복 시점에 즉시 break → 효율 + 비용 절감. 누적 결정 49 → **50**. 1 commit (1a71a88 dual-push). | D-101 의 "LLM 같은 Bash 반복" 위험. | synthetic final prompt 1회만 / A-proper native tool calling 미적용 / 큰 file chunked Read 권장 prompt 필요. 다음: large file chunked Read + Hashline 진입. |
| **D-103** | 2026-06-30 | **Prompt 보강 (large file chunked Read + overlap dedup)** — D-102 follow-up. LLM 이 큰 파일 (770 lines) full Read → token 낭비 + 같은 파일 chunked Read offset 중복 가능. 2가지 fix: (1) Read description 강화 (">500 lines, ~200 chunk, offset+limit 권장"). (2) Large files 섹션 4가지 (Glob first / no overlap / progress forward / always offset+limit for >500). `tool_spec_section` +20 lines. 3 test 추가. **68/68 tui PASS (D-100 18 + D-101 4 + D-102 5 + D-103 3 + 다른 38)**, clippy clean. Real LLM `code review myharness/crates/cli/src/main.rs` (770 lines) → 1st Read full + 2nd Read same → **D-102 dedup 자동 발동** + final prompt. **이전 10 round → 현재 2 round, ~5x 빨라짐**. 누적 결정 50 → **51**. 1 commit (99a2bad dual-push). | D-102 의 "큰 file 낭비" 한계. | LLM prompt 무시 가능 → 강제하려면 tool wrapper (v1.5+) / 1000+ lines 은 chunked Read + content fingerprint (Hashline) 가 진짜 해결책. 다음: oh-my-pi Hashline 진입. |
| **D-104** | 2026-07-01 | **oh-my-pi Hashline 분석 + Read v2 (LINE:TEXT + 4-hex content_hash)** — `@oh-my-pi/hashline` v15.11.0 (MIT, Mario Zechner 의 `badlogic/pi-mono` fork) 의 LINE:TEXT prefix + content hash tag 패러다임을 v1 점진 차용 1차 cycle 로 채택. D-103 의 "1000+ lines 진짜 큰 파일은 content fingerprint 가 진짜 해결책" 한계의 직접 응답. **구현**: (1) `myharness/crates/tools/src/content_hash.rs` 신규 — `compute_content_hash` = SHA-256 truncate low 16-bit → 4-hex uppercase (oh-my-pi `HL_FILE_HASH_LENGTH=4` 정합, sha2 0.11 이미 workspace, xxhash-rust 새 dep 회피) + `normalize_for_hash` (line 별 trailing whitespace trim + final newline 보존) + `format_line_anchored` (1-indexed absolute line, phantom-trailing row 회피). (2) `read.rs` v2 — output 항상 LINE:TEXT format default + metadata `{path, size, line_count, format:line_text, content_hash, hash_length:4, start_line, end_line}`. chunked Read 도 절대 line 번호 보존. (3) `Cargo.toml` tools crate sha2 dep 추가. (4) `lib.rs` content_hash mod + re-export. (5) **spec memo `ai-workflow/memory/hashline_v2_spec.md`** (200+ lines, 9-section — 왜 Hashline / 9-area 점진 차용 결정 table / content hash sha2 vs xxhash / Read v2 spec / Edit v2 line_anchored spec §5 / v1.5+ tree-sitter + v2 snapshot store roadmap). **12 test 추가** (content_hash 8 + read 4 신규). **검증**: cargo clippy --workspace --all-targets -- -D warnings clean (sha2 0.11 → 3 clippy catch 한 번에 text rewrite fix) / cargo test --workspace --lib **436 pass + 0 fail + 2 ignored** (D-103 baseline 424 → +12 회귀 0) / Invariant: full vs chunked 동일 content_hash. **효과**: (a) D-105 Edit v2 가 hash check 로 stale anchor 자동 reject. (b) LLM 이 LINE:TEXT 그대로 보고 line 번호 cite (`5:fn main()`). (c) oh-my-pi 의 `prompt.md` tight-range / 1-hunk-per-range 룰셋이 D-105 line_anchored prompt 에 그대로 적용 가능. **차용 범위 결정**: D-104 = Read v2 (LINE:TEXT + content_hash) 2 area 점유. D-105+ = Edit v2 line_anchored 3 area. v1.5+ = tree-sitter `replace block N` 2 area. v2 = SnapshotStore + 3-way merge 2 area. (총 9-area 점진 채움, 한 cycle 1-2 area.) **한계**: (a) Edit v2 line_anchored 미구현 (D-105), (b) tree-sitter 미도입, (c) sha2 vs xxhash 16-bit fingerprint 의미론 동일. **누적 결정 51 → 52**. 1 commit (43bc908 dual-push). main = `43bc908`. | D-103 의 "content fingerprint 가 진짜 해결책" 한계. Hybrid 안의 9-area 점진 채움 1차. | (a) Edit v2 미구현 — D-105+, (b) tree-sitter 미도입 — D-106+, (c) sha2 vs xxhash 16-bit 동일 — D-105 시 xxhash-rust 도입 결정 가능. 다음: D-105 Edit v2 line_anchored 진입. |
| **D-127** | 2026-08-14 | **TASK-004 재방문 — opencode v2 영향 분석** — opencode = 06-09 이후 **1,457 commit** (default branch = `dev`, NOT main). 7 reference 중 가장 활발. **§15 v2 Changelog**: 핵심 15 PR (#42160/42164/42166 reasoning effort batch / #42045 compaction / #42161 Kimi prompt by provider / #41522 Copilot PDF / #41939/#41942 session retry jitter cap / R2 data catalog 243 lines / #42085 DeepSeek ZDR / #41814 Hy3 Free). **§16 v2 영향 분석** (10 항목): (a) reasoning effort 표준화 → §5.5 LLM wire format (v1.5+) (b) compaction → §5.6 Layer2 (v2+) (c) **session retry jitter cap** → §5.5 router (v1 즉시) (d) release sync 패턴 → §5.12 versioning (v1.5+) (e) Copilot PDF detect → §5.5 multimodal (v2+) (f) R2 data catalog → §5.13 observability (v2+) (g) Kimi prompt by provider → §5.5 provider (v1.5+). **+297 lines append**. main = `6057dbb`. **누적 결정 74 → 75**. | TASK-005-2 v2.0 Hybrid 안 vs opencode 의 활발한 v1.18.17~18 release — 직접 영향 §5.5 router / §5.6 Layer2 / §5.13 observability. | (a) §5.5 wire format v1.5+ 작업 시 (b) §5.5 router 즉시 (c) §5.13 observability v2+. |
| **D-128** | 2026-08-14 | **TASK-004 재방문 — aider v2 영향 분석** — aider = 06-09 이후 **0 commit** (정직 검증). HEAD = `5dc9490bb` / tag = `v0.86.3.dev`. v1 의 14섹션 분석 (git-first, repo map, LLM 비종속, .aider.conf.yml, CONVENTIONS.md, architect 2-model, attribution logic) 은 현행 upstream 과 1:1 정합. **my_harness 영향 = 0, 결정 변경 불요**. **+128 lines append**. main = `e02853c`. **누적 결정 75 → 76**. | reference verification 결정 (no architecture impact). | 5-triggers + 90일 cadence 로 v3 revisit (2026-11-12 추정). |
| **D-129** | 2026-08-14 | **TASK-004 재방문 — codex v2 영향 분석** — codex = 06-09 이후 **1,996 commit** (가장 활발). 우리 v0.0.x 와 같은 Rust stack. **§15 v2 Changelog**: 핵심 15 PR (#38390 effective permission / #38384 skill validation / #38383 Luna samples / #38381 in-process requests / #38380 user msg styling / #38377 Guardian V2 parent fs / #38368 Luna sampler / #38363 risk scores / #38362 exec-server byte-budget / #38361 hook rejection / #38358 orphan output / #38356 sandboxed streaming / #38336 Guardian V2 extension scaffold / #38321 gRPC code-mode deterministic / #38306 inline visualization sandboxed). **§16 v2 영향 분석** (6 영역): (a) **effective permission** → §5.4 permission mode 의 v0 회귀 차단 (**P0 즉시**) (b) Skill validation → §5.14 skill system 1차 cycle (v1.5+) (c) interrupted turn recovery → §5.10 LoopRunner (d) Luna sampler → §5.5 reasoning model (e) in-process unbounded queue → §5.5 LLM client architecture (P1, 결정 보류) (f) sandboxed streaming → §5.10 Bash tool sanitize. **+523 lines append**. main = `eec2c88`. **누적 결정 76 → 77**. | codex 의 app-server + Guardian V2 + Skill validation 의 직접 차용 가치. | (a) §5.4 permission v0 회귀 차단 즉시 (b) §5.14 skill system v1.5+ (c) §5.10 LoopRunner v1.5+. |
| **D-130** | 2026-08-14 | **TASK-004 재방문 — headroom v2 영향 분석 + D-66/D-67/D-68 tract 재평가** — headroom = 06-09 이후 **106 commit (실측)** / prompt 의 1085 = 1 order of magnitude underestimate. v0.23.0 (2026-06-04) + Unreleased. **§15 v2 Changelog**: v0.23.0 8 핵심 (Copilot subscription / CCR workspace scope / memory READ-ONLY + fail-closed / docker Python 3.13 / CVE remediation 3 item / tag format vX.Y.Z / cli wrap-subcommand) + Unreleased (startup log noise suppression / loopback guard / retry_max_attempts zero guard / async subprocess / Neo4j credential / concurrent iteration). **D-130 prompt correction**: RTK (Rust observability) = **v0.22.4 변경 (PR #493/494, 2026-05-25)**, v0.23.0 영향 아님 (prompt 의 link = hallucination, D-73 §3 lesson 회귀). **§16 v2 영향 분석** (9 follow-up): (a) Copilot subscription → §5.5 provider 등록 (b) **CCR workspace scope** → §5.6 Layer2 CCR + 즉시 1 commit (~150 lines) (c) cli wrap-subcommand → §5.10 sub-agent dispatch + v1.5+ Sub-task 2 (provider-auto-config Skill) 동시 차용 (d) memory READ-ONLY + fail-closed → §5.6 Layer2 memory + 즉시 1 commit (~80 lines) (e) tag format vX.Y.Z → §5.12 versioning (f) Learned error recovery 4-layer fix → §5.13 LLM Wiki + v1.5+ Sub-task 5 (Learn Plugin 1차 cycle). **§16.12 핵심**: **D-66/D-67/D-68 tract 재평가 REJECT** (Rust 측 안정성 ↑ = RTK(v0.22.4) + Rust proxy metrics(headroom 자체) → ONNX 무관) → **v2.0 ONNX 백로그 = OOS 유지**. **+207 lines append**. main = `f6cbc30`. **누적 결정 77 → 78**. | headroom v0.23.0 의 CCR workspace + memory fail-closed 의 직접 차용 가치. D-66/D-67/D-68 tract 의 rust 측 안정성 ↑ 가설 검증 — REJECT. | (a) SqliteMemoryStore FTS5 schema + workspace_path (즉시) (b) Memory fail-closed (즉시) (c) cli wrap-subcommand (v1.5+) (d) Learned Plugin (v1.5+ Sub-task 5). |
| **D-131** | 2026-08-14 | **TASK-004 재방문 — goose v2 영향 분석** — goose = 06-09 이후 **661 commit**. **§15 v2 Changelog**: ACP 프로토콜 확장 (8 commit, schema.json +443 / extensions.rs +736) + TUI 정식 통합 + 보안 강화 (egress direction, SSRF, OAuth 헤더, ACP SDK 0.12.1) + Provider 8종 추가 (xAI SuperGrok OAuth / Kimi Code DF / Perplexity / Qwen DashScope / Databricks GW / NEAR AI / Scaleway / HF OAuth) + Loop 안정화 6 commit + Recipe/Slash 통합 + 의존성 (agent-client-protocol 0.12.1, manylinux_2_28). **§16 v2 영향 분석** (6 영향 축): (a) ACP SDK 0.12 추가 → §5.5 LLM client (v2+, feature `acp`) (b) 보안 강화 → §5.4 permission + sanitize (c) Recipe/Slash → §5.14 Skill/MCP first-class (d) **provider 50→58 declarative + OAuth device flow 2-tier** → §5.5 (v1.5+) (e) 의존성 → §5.2 빌드 (f) Loop 안정화 → §5.10 LoopRunner. **+173 lines append**. main = `1a86b44`. **누적 결정 78 → 79**. | goose 의 ACP + 8 provider 추가의 직접 차용 가치. | (a) §5.5 provider 등록 (v1.5+) (b) §5.4 permission + sanitize (c) §5.10 LoopRunner 안정화. |
| **D-132** | 2026-08-14 | **TASK-004 재방문 — gemini-cli v2 영향 분석** — gemini-cli = 06-09 이후 **130 commit**, v0.45.0 → v0.55.1 (10 minor). **§15 v2 Changelog**: 핵심 10 PR (v0.55.1 changelog / PR #28305 tool call formatter / PR #28729 IDE connections fix / PR #28688 Cloud Workstations OAuth redirect / PR #28369 local report / PR #28730 model capacity fix / PR #28481 **MCP OAuth tokens refresh with stored client ID** / PR #28601 NEEDS_HUMAN lock / PR #28690 ingestion issue comment / PR #28716 Capacity Exhaustion terminal). **§16 v2 영향 분석** (7 영향): (a) tool call formatter → §5.5 wire format (b) Cloud Workstations OAuth → §5.5 D-51 OAuth (c) **MCP OAuth token refresh** → §5.5 W15.b 자동 refresh (d) NEEDS_HUMAN lock → §5.10 sub-agent (e) Capacity Exhaustion → §5.5 router (f) TOML extensions 표준 → §5.14 (g) ingestion workflow → §5.13 LLM Wiki 자동 ingest. **+391 lines append**. main = `f500f87`. **누적 결정 79 → 80**. | gemini-cli 의 TOML extensions + MCP OAuth refresh 의 직접 차용 가치. | (a) §5.5 wire format (b) §5.14 TOML extensions 표준 (v1.5+) (c) §5.13 LLM Wiki 자동 ingest. |
| **D-133** | 2026-08-14 | **TASK-004 재방문 — claude-code v2 영향 분석 + D-34/D-40 §11.2 정합 검증** — claude-code = 06-09 이후 **594 commit (실측)** / prompt 의 66 = 자동화 commit 만 (525 자동화 + 68 실질). **D-34/D-40 §11.2 잠금 정합 검증**: 06-09 이후 claude-code 변경 = 우리 §5.6/§5.14/§5.4/§5.5 영향 0. **D-40 의 §11.2 완전 제거 결정이 정합이었음 사후 확인**. 결정 변경 불요. **§15 v2 Changelog**: 자동화 88% + docs + security + oncall + CI + plugin (작업자 분류). **§16 v2 영향 분석**: 핵심 7 commit (PR #79898 AWS gateway / #61584 OIDC federation / #56784 SHA pin / #43824 shell injection / #33472 confirmed flag / #45866 MDM / #30066 gh.sh wrapper) + **영향 매트릭스 (§5.5/§5.6/§5.4/§5.13/§5.14 = 영향 0)**. **+266 lines append**. main = `c7b03f1`. **누적 결정 80 → 81**. | D-34 §11.2 결정의 사후 검증. 영향 0 이라 D-40 의 잠금이 정합이었음 확인. | 없음 — D-40 의 §11.2 잠금 정합 유지. |

---

## 3. 개발 흐름 (Development Timeline) — append-only

### 2026-06-05 — 부트스트랩
- `git init -b main` → my_harness 레포 생성
- `standard_ai_workflow` minimax-code 오버레이 적용 (MiniMax.md, .MiniMax/agents/, ai-workflow/{core,memory,scripts}/)
- 표준 6필드 헤더 + 한국어 보고 + 상태값 `planned|in_progress|blocked|done` 적용
- PROJECT_PROFILE.md §1, §3.1, §4 갱신 — 3-도메인 스코프 확정
- 4-워커 division 룰 (`docs/governance/worker_division.md`) 추가

### 2026-06-05 22:00 — 방향 전환
- **단순 컨슈머 → CLI/TUI 직접 개발** 로 피벗 (D-03)
- 새 TASK-001 ~ TASK-005 인덱스 등록
- TASK-001: smoke 보정, TASK-002: 도메인별 명령, TASK-003: Gitea mirror, TASK-004: reference 분석, TASK-005: 스택 결정

### 2026-06-06 — 1차 reference 분석 + Gitea 미러
- 5 reference clone (opencode/aider/codex/goose/gemini-cli) — `/Users/yklee/repos/harness-refs/`
- Gitea private repo 5개 push — PAT in macOS keychain (D-06)
- dual-remote + unshallow 적용 (D-07, D-08)
- 5-심층분석 (14섹션) 시도 — **4/5 worker long Write abort** → owner 직접 takeover
- TASK-004 1차 8축 비교표 (`docs/REFERENCES.md`)
- 14섹션 표준 템플릿 (`docs/references/ANALYSIS_PLAN.md`)
- claude-code 추가 — `anthropics/claude-code` 정식 repo + 2차 분석 (`davccavalcante/claude-code` 등) (D-11, D-12)

### 2026-06-07 — 2차 분석 + 결정 보류
- **PROVIDERS.md** (3-way 비교: rig-core 12+ / Vercel AI SDK 15+ / litellm proxy 50+) — TASK-005 입력
- **headroom** 6번째 reference clone + 분석 시도 (plan_52a216af, 60min timeout)
  - 1차: §1-§4 (390줄) + early deliverable.md (in_progress) — chunked write 작동
  - 2차: §5-§7 Edit append 중 **worker abort** (session errored) → plan cancel
  - 3차: owner 직접 §5-§14 append (473줄) → 863줄 / 14섹션 완성
- headroom Notable Patterns 13 adopt + 7 anti-pattern 추출 — 우리 my_harness 설계 직접 입력
- **mavis 환경 gh CLI keychain 충돌** 발견 + symlink 워크어라운드 (D-19)
- **본 백데이터 문서** (D-17) 신설
- **Gitea + GitHub dual-remote 첫 push** (D-20) — origin (Gitea, private) + upstream (GitHub, public), 두 커밋 (headroom + dev log) 모두 푸시
- **claude-code 7번째 reference 분석** (D-21) — 1,029줄 14섹션 분석 + 7-doc 통합 인덱스. 8축 비교 매트릭스 + my_harness 영향 분석 §3 + Adopt 23개 (1차 8 / 2차 7 / 3차 8) + Anti 6개
- **my_harness v1 컨셉 확립** (D-22) — `docs/CONCEPT.md` 마스터 SSOT 신설. 12섹션 (positioning/타겟/가치/스코프/v1 MVP spec/v2+ 로드맵/채택 23/안티 6/KPI/리스크/Open decisions/참조)
- **기존 문서 align** (D-23) — MiniMax.md / PROJECT_PROFILE.md / REFERENCES.md / PROVIDERS.md 의 메타 + 도메인 명령 + 7-doc 확장 + claude-code 3-fallback 섹션 추가. CONCEPT.md SSOT 참조.
- **CONCEPT.md 컨셉 교정 1차** (D-24) — "외부 4-워커 통합/오케스트레이션" framing 제거. my_harness = standalone harness tool. Sibling to claude-code/codex/aider/goose/gemini-cli/opencode.
- **CONCEPT.md 컨셉 교정 2차 (Mavis zero coupling)** (D-25) — §0.5 다이어그램에서 Mavis/orchestrator/standard_ai_workflow 모두 제거. §2 타겟에서 Mavis 행 삭제. §5.8 "외부 의존성 없음" 신설. **my_harness = 100% standalone**, 유일한 런타임 의존 = LLM provider API + (선택) headroom MCP.
- **standard_ai_workflow 준수 (D-26)** — §5.9 신설. 6 원칙 native (한국어/절약/상태/이벤트/비참조/handoff) + 옵션 Mavis 통합 (auto-detect `ai-workflow/memory/`, 미발견 시 자체 `.myharness/`). Zero coupling 유지.
- **headroom built-in 압축 layer (D-27)** — §0.5 / §3.3 / §5.6 갱신. 흐름 = `user → my_harness → (built-in 압축) → LLM provider`. headroom 의 6 알고리즘을 우리 Context component 에 built-in. 외부 proxy 의존 X, 기본 off, v1 우선 3개 알고리즘.
- **Provider 6개 확정 + OpenAI 호환 lingua franca (D-28)** — §5.5 전면 갱신. claude/codex/gemini = native SDK, deepseek/minimax/local-llm = OpenAI 호환 client. 3 fallback (D-15) + 도메인별 mapping. 모델 prefix 규약 도입. minimax TBD (base_url 검증 필요).
- **Agent 3 모드 + Built-in sub-agents (D-29)** — §5.10 + §5.11 신설. orchestrator (default) / single / loop 모드, 15개 built-in sub-agents (3-도메인 × 4-5).
- **2-계층 Context 압축 (D-30)** — §5.6 갱신. Layer 1 (필수 자동 압축) + Layer 2 (opt-in headroom 6 알고리즘).
- **`~/.myharness/` 디렉토리 구조 (D-31)** — §5.12 신설. sibling tool 컨벤션 (claude/codex/gemini/headroom/minimax 모두 `~/.<toolname>/`) + XDG-style 내부 분리.
- **LLM Wiki memory (D-32)** — §5.13 신설. Karpathy 3-layer (raw/wiki/schema) + index/log/lint. v1 flat, v2+ LLM Wiki.
- **Skill/MCP first-class (D-33)** — §5.14 신설. skills (claude-code 13.3 차용) + MCP (rmcp/@modelcontextprotocol/sdk) + 4 pre-config server.
- **TASK-NNN 형식 통일 + 2.1.169 pending 표 (D-34)** — §6 마일스톤 → TASK-005-1~TASK-005-5. §11.1 TUI/Provider fallback → TASK-006/008. §11.2 신설 = claude-code 2.1.169 영향 결정 pending 표.
- **관련 문서 align to CONCEPT.md v1 (D-35)** — MiniMax.md / PROJECT_PROFILE.md 관련 문서 에 §5.10~§5.14 cross-ref. README.md (root) 전체 재작성 (v1 산출물 vs 개발 workflow 분리). PROVIDERS.md cross-ref 갱신.
- **TASK-005 결정: Rust 1안 (D-36)** — yklee 결정. v1 MVP 스택 = Rust 2024 + ratatui + rig-core + rmcp + keyring + cargo-dist. §11.1 TASK-005/006 ✅ 결정 완료. §11.3 신설 (결정 근거 8개 + v1 스택 종합). PROJECT_PROFILE.md 적용 환경 갱신.
- **TASK-007 결정: headroom v1 1안 유지 (D-37)** — yklee 결정. v1 = 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor). CCR + Kompress-base 는 v1.5+ (다음 개발 페이즈) 로 연기.
- **TASK-008 결정: provider-auto-config skill (D-38)** — yklee 결정. 하드코딩 fallback 폐기 → 동적 discovered list + per-provider auth. §5.5 전면 갱신 (4 subsections). `docs/skills/provider-auto-config/SKILL.md` reference design 신설. v1 Phase 1 (정적 + Anthropic + Ollama) / Phase 2 (skill 정식) / Phase 3 (OAuth + MCP).
- **세션 마무리 (D-39)** — v1 컨셉 Phase 종료. session_handoff.md / work_backlog.md / state.json 갱신. 5/5 결정 검토 완료 (TASK-002 ⏸ + TASK-005/006/007/008 ✅). 다음 세션 시작점 = TASK-005-1 (v1 MVP Rust 빌드).
- **§11.2 claude-code 2.1.169 검증 취소 (D-40)** — yklee 결정. v1 spec 잠금 (Rust 1안 / ratatui / headroom 3 algo / provider-auto-config). §11.2 완전 제거.

### 2026-06-09 — TASK-005-1 환경 검증 (D-41)
- W0-1 (Rust toolchain + crate 가용성): Rust 1.94.1 / 2024 edition / native target `x86_64-unknown-linux-gnu` ✅
- W0-2 (cross-build + keychain + .myharness): cross target 5/6 미설치, Linux keychain backend 부재 (secret-tool / gnome-keyring-daemon 없음), ~/.myharness/ 부재 (clean slate) ✅
- 핵심 crate 12+ 전부 가용: rig-core 0.38.1, rmcp 1.7.0, ratatui 0.30.1, keyring 4.0.1, tree-sitter 0.26.9 등
- Prerequisite 5건 식별: (1) libsecret-1-dev + gnome-keyring, (2) 5 cross rustup target, (3) cargo-dist + cargo-binstall, (4) ANTHROPIC_API_KEY, (5) serde_yaml → serde_yml 전환
- TASK-005-1 진입 가능 ✅ (cargo init 즉시 가능, prerequisite 설치 후 진행)

### 2026-06-09 — D-42 config TOML 통일
- D-42 결정: v1 의 모든 사용자 편집 config / state / provider registry 포맷을 YAML → TOML 로 일괄 교체
- 이유: `serde_yaml` 0.9.34-deprecated + `serde_yml` 0.0.13-deprecated (두 crate 모두 unmaintained). yklee single-user + TUI config 만 필요 → TOML (Cargo 표준) 으로 충분
- JSON 파일 (auth state snapshot / metrics / mcp.json / log.jsonl / state.json / ai-workflow/*.json) 은 format 그대로 유지
- 영향: CONCEPT.md 19곳 .yaml → .toml syntax 변환 + SKILL.md YAML 예시 TOML 변환 + README.md / state.json / development_log.md 결정 기록 반영
- `serde_yaml` / `serde_yml` crate 의존 회피 → W2 (cargo workspace init) 시 `toml` crate 만 의존성 추가

### 2026-06-09 — TASK-005-1 W3~W6.5 (D-43) + Gitea push
- W2: cargo workspace init — myharness-tools crate 구조 확립 (`crates/myharness-tools/`)
- W3: basic Tools (Read/Write/Edit/Bash/Grep/Glob) + cli 인수 (tool + provider + profile + timeout)
- W4: 4 permission mode (default/acceptEdits/plan/bypassPermissions) + permission_denied.rb integration test
- W5: Bash sanitization — 9 위험 패턴 (strict/off/permissive) + sanitizer integration test
- W6: JSON Schema (schemars 1.2) — ToolCall 구조 schema + auto-generate
- W6.5: 5 provider wire format (Anthropic/OpenAI/DeepSeek/Ollama/llama.cpp/litellm) — ToolRequest/ToolResponse 구조 통일
- 테스트 총 63 passed (51 unit + 4 schema integration + 3 permission + 3 sanitizer + 2 compat)
- 9 commit (3f0c9cb d8e68e1 a6a014c d371586 ec1f704 daf566f d5264b6 25a60e0 dfc9d93) → Gitea origin push
- Gitea PAT 미설정 — push 시 credential helper 가 gh-cli 또는 ssh fallback 으로 처리된 것으로 추정
- 다음: W7 (llm crate 진입, rig-core 0.38 + Anthropic provider) — ANTHROPIC_API_KEY absent 이므로 mock test 위주

### 2026-06-09 — dual-remote 적용 (D-44)
- D-20 dual-remote 정책 (2026-06-07 수립) 을 본 세션에서 완전 이행
- Gitea PAT `myharness-cli` 직접 발급 (Gitea web Basic auth + API POST `/api/v1/users/yklee/tokens`, scopes: write:repository, write:user) — `~/.git-credentials` 에 저장 (chmod 600)
- `git remote add upstream https://github.com/ykylee/my_harness.git` — GitHub public repo 추가
- GitHub 인증: gh CLI 사용 (ykylee account, scopes: repo/workflow/gist/read:org) — PAT 직접 보관 불필요
- 9 commit (`3f0c9cb`..`dfc9d93`) → Gitea origin + GitHub upstream 양쪽 push 완료
- 보안 주의: gh CLI `--show-token` 옵션 사용 시 GitHub PAT 가 stdout 노출됨 — W7+ 시작 전 회전 권고

### 2026-06-09 — D-45 TASK-005-1 W7 (myharness-llm crate v1)
- 6 provider enum + ProviderRegistry + LLMClient trait + rig-core 0.38 Anthropic/Gemini wrapper + OpenAI 호환 wrapper (DeepSeek/Ollama/local-llm) + MockClient + AuthState/AuthStatus + InMemory/Keyring AuthStore + provider-auto-config discover + ActiveProviderChain + FallbackRouter
- 87 tests pass. release 빌드 성공. 5 commit (29052ad~7264589) → Gitea + GitHub dual-push
- ANTHROPIC_API_KEY absent 환경 → mock-driven 검증. Key 주입 시 real-anthropic ignored test 활성화

### 2026-06-09 — D-46 TASK-005-1 W8 (myharness-context crate v1)
- CLAUDE.md loader (project root + parent walk + global fallback) + auto memory (NDJSON append-only) + ContextManager (token budget + /compact Layer 1: Truncate/Summarize-stub/Hybrid) + Layer 2 BuiltinPipeline (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor, 기본 off) + ContextConfig (config.toml [context] 섹션) + ContextOrchestrator
- 54 tests pass. 5 commit dual-push

### 2026-06-09 — D-47 TASK-005-1 W9 (myharness-compression crate v1)
- Summarizer (LLM-driven summarize, ContextManager Hybrid 정식 통합) + CCR (Claude Code Recursive split, 단일 round-trip 비용 1회 trade-off) + Kompress-base v1 simple (트리머리 휴리스틱) + BuiltinRegistry 6 알고리즘 (CacheAligner/ContentRouter/SmartCrusher/CodeCompressor/CCR/Kompress-base) + Layer 1/2 Pipeline 통합
- 40 tests pass. 5 commit dual-push. context Layer 1 Summarize/Hybrid 가 LLM-driven 으로 정식 작동

### 2026-06-09 — D-48 TASK-005-1 W10 (myharness-tui crate v1)
- ratatui App + crossterm backend + 4 SubAgent (code-reviewer / code-implementer / code-tester / git-operator) + Orchestrator (서브에이전트 dispatch + 결과 통합) + LoopRunner (--mode=loop --goal --max-iterations) + cli 통합 (code/env/git/ask subcommand) + TUI test backend (buffer)
- 51 tests pass. 5 commit dual-push. TUI shell 완성

### 2026-06-09 — D-49 TASK-005-1 W11 (myharness-core crate v1, v1 MVP 6/8 waves 완료)
- standard_ai_workflow 6 원칙 native + 4 permission mode + tool name alias (Read/Grep/Glob ↔ read/grep/glob_) + MockClient FIFO (sequential mock response) + Orchestrator fatal_llm_error (LLM 호출 실패 시 fatal 종료) + cli task start|end subcommand + handoff writer
- 32 tests pass. 5 commit dual-push. **6/8 waves 완료 + cli entry 통합**

### 2026-06-09 — D-50 TASK-005-1 W12 (MiniMax LLM API 연결, minimax TBD D-28 해소)
- librarian 조사 (⭐⭐⭐⭐⭐ 5/5): `https://api.minimax.io/v1` OpenAI-호환, `MiniMax-M3` default, 7 models (M3/M2.7/M2.7-highspeed/M2.5/M2.5-highspeed/M2.1/M2), Bearer token (MINIMAX_API_KEY env), tool_use 지원, no-CORS, JSON streaming
- `ProviderMetadata::builtin_minimax()` 갱신 + `KeyringAuthStore` in-memory cache (libsecret 부재 fallback) + `MINIMAX_API_HOST` env override (base_url)
- cli default LLM = `MINIMAX_API_KEY` env 자동 detect → `OpenAiCompatProvider` 흐름 검증
- 5 commit dual-push. **MiniMax 가 v1 default LLM**

### 2026-06-09 — D-51 TASK-005-1 W13 (myharness-auth crate v1, OAuth 2.0 headless auth)
- 7 모듈: `pkce` (RFC 7636 S256 + state) + `flow` (OAuth 2.0 Authorization Code + PKCE core, provider-agnostic) + `callback` (loopback HTTP server 127.0.0.1, 5min timeout, port 0 random) + `browser` (xdg-open/open/start) + `store` (`~/.myharness/oauth/{provider}.toml`, chmod 600, MYHARNESS_HOME env override) + `provider` (MiniMax/OpenAI/Google 3 provider, 모두 PKCE public client) + `manager` (AuthManager login/refresh/status/logout + AuthError::is_not_found())
- 38 tests pass. cli auth subcommand (`auth list|login|logout|status`) 추가
- **MiniMax OAuth endpoint 확정**: `https://api.minimax.io/oauth/authorize` + `/oauth/token`
- 4 commit dual-push. 누적 30+ commit dual-push

### 2026-06-09 — D-52 TASK-005-1 W13.5 + W13.6 (v1 MVP 8/8 waves 완료)
- **W13.5**: `OAUTH_PROVIDERS` static `LazyLock<HashMap<...>>` 제거 → `oauth_providers()` 매번 새 instance (env 변경 즉시 반영). 3 provider `from_env()` 생성자 추가. `MYHARNESS_OAUTH_CLIENT_ID_{MINIMAX,OPENAI,GOOGLE}` env override + `MINIMAX_API_HOST` base_url override. env 검증: `cp-test-12345` 정상 반영
- **W13.6**: local mock HTTP server (`TcpListener` + raw HTTP request line) + `MockProvider` (trait `&self` receiver, custom endpoints) + `auth_manager_end_to_end_with_mock_server` test: build_authorize_url → reqwest get → 302 redirect → callback params → `exchange_code` → JSON token response → `TokenStore::save`. **real network 없이** 전체 OAuth 2.0 + PKCE flow 검증. CI 환경 가능
- 39 auth tests pass. 2 commit dual-push (Gitea + GitHub)
- **v1 MVP 8/8 waves 완료**: tools + llm + context + compression + tui + core + MiniMax API + OAuth headless auth. 364 tests pass, 0 fail, 2 ignored

### 2026-06-09 — D-53 TASK-005-1 W14 (MiniMax Device Authorization Grant, D-52 follow-up)
- W13 의 MiniMax Authorization Code + PKCE redirect flow 가 404. **MiniMax OAuth 는 Device Authorization Grant 변형 사용** (OpenClaw/Hermes 와 동일, RFC 8628 의 MiniMax 구현)
- 새 `MinimaxDeviceOAuth` provider + `DeviceCodeProvider` trait. `client_id = 78257093-7e40-4613-99e0-527b14b39113` (OpenClaw/Hermes 공통)
- `scope = group_id profile model.completion`. region: 한국 default = global (`https://api.minimax.io`). CN: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env override
- 새 `device_flow.rs` module: `request_code` (POST `/oauth/code`) + `poll_token` (POST `/oauth/token`) + `poll_until_success` (loop until expired_in)
- 기존 `MinimaxOAuth` (Authorization Code) 는 deprecated 표시만, 그대로 둠 (OpenAI/Google redirect flow 와 일관성)
- 1 commit dual-push. 누적 33+ commit

### 2026-06-09 — D-54 TASK-005-1 W14.4 (`--no-browser` 의미 수정, D-52 follow-up)
- W14 의 `--no-browser` 가 polling 도 중단하던 버그. signature 변경: `interactive: bool` → `(auto_open_browser: bool, non_interactive: bool)`
- 3 모드 확정: (1) **default** = URL 출력 + browser 자동 open + polling + save, (2) **`--no-browser`** = URL 출력 + polling + save, (3) **`--non-interactive`** = URL 출력만 + 즉시 종료
- cli `AuthAction::Login` 에 `--non-interactive` flag 추가
- mock e2e test `login_minimax_device_no_browser_polling_saves` 추가
- 42 auth tests pass, 367 workspace tests pass, 0 fail, 2 ignored. 1 commit dual-push

### 2026-06-09 — D-55 TASK-005-1 W14.5 (polling output + expired_in 단위, D-52 follow-up)
- `login_minimax_device` polling 진입 전 URL + user_code + expires_in 항상 `tracing::info!` 출력 (browser 자동 open 무관)
- `poll_until_success` OpenClaw 와 동일한 1.5x backoff + cap 10s + floor 1s
- `expired_in` 단위 수정: MiniMax 가 **milliseconds 단위 unix timestamp** 로 응답. 우리 now 도 `timestamp_millis()` 로 비교. `/1000` 으로 wait_secs 표시
- 검증: `./myharness auth login minimax --no-browser` 실행 시 URL + user_code + `expires_in: 299s` 정상 + `mini_max ms-unix-ts=1780991724849` 디버그
- 42 auth tests pass. 1 commit dual-push

### 2026-06-09 — D-56 TASK-005-1 W14.6 (device_token_to oauth 단위 변환, D-52 follow-up)
- `device_token_to_oauth` 응답의 `expired_in` ms → s 변환 추가. `TokenStore::save` 는 초 단위 `expires_at` (Unix seconds)
- 42 auth tests pass. 1 commit dual-push

### 2026-06-09 — D-57 TASK-005-1 W15.a (OAuth token 자동 resolve, D-52 follow-up)
- **누락분 정정 (D-104 sync, 2026-07-01)**: §2 표의 D-100/D-101/D-102/D-103/D-104 결정 entry 누락 (이전 결정 들의 §2 표 + §3 timeline 모두 v1 MVP 까지만 반영). 5 entry §2 표 + 본 §3 timeline 2 date heading 추가. §2 표 append-only + §3 timeline append-only 정책 정합. **누락은 메모리 sync workflow 점검 (2026-06-14 d0a223e) 의 1차 점검 이후 cross-session drift** — 다음 세션 memory sync 후 cross-check 강화 권고.

### 2026-06-30 — 방향 전환 + A-min tool dispatch (D-100 ~ D-103)

- **D-100 (방향 전환)**: oh-my-pi Hybrid 안 결정 (yklee). Hashline 편집 / Skill·Extension / hindsight memory / LSP+DAP 점진 차용 후보. OAuth PKCE+Device / Local LLM cascade / R-4 backup / W15.a/b / standard_ai_workflow / 한국어 workflow / 백업 안전장치 유지. 선결 = 사용 가능한 형태 (`MINIMAX_API_KEY` 주입으로 real LLM 작동).
- **D-100 (A-min 1차 cycle)**: text-based tool dispatch (agent.rs tool_spec_section + orchestrator.rs extract_tool_call + dispatch loop max 3 round). 5 test 추가. 18/18 tui PASS. clippy clean. real LLM `code review <file>` / `env diagnose` 자동 dispatch.
- **D-101 (A-min polish)**: max_round 3 → 10 default + tool result stdout visible (2000자 truncation) + `with_confirm_override(true)` 비대화형 hang 방지. 4 test 추가. 22/22 tui PASS.
- **D-102 (prompt + dedup 안전망)**: stop conditions 4가지 + `canonical_tool_call` BTreeMap 정렬 + `call_counts` 2회 중복 시 synthetic final prompt + break. 5 test 추가. 27/27 tui PASS. `ask "1+1은?"` 즉시 plain 응답.
- **D-103 (large file chunked Read)**: Read description ">500 lines, ~200 chunk, offset+limit" + Large files 섹션 4가지. 3 test 추가. 68/68 tui PASS. 770 lines file → 2 round + D-102 dedup 자동 발동 (이전 10 round → ~5x 빨라짐).
- 누적 결정 47 → **51**.

### 2026-07-01 — D-104 oh-my-pi Hashline 1차 cycle (Read v2)

- **content_hash.rs 신규**: `compute_content_hash` (sha2 truncate low 16-bit → 4-hex uppercase, oh-my-pi `HL_FILE_HASH_LENGTH=4` 정합) + `normalize_for_hash` (line 별 trailing whitespace trim + final newline 보존) + `format_line_anchored` (1-indexed absolute line, phantom-trailing 회피).
- **read.rs v2**: LINE:TEXT format default + metadata `format/content_hash/hash_length/start_line/end_line` 노출. chunked Read 절대 line 번호 보존 (D-103 anchor 1차 보강).
- **spec memo** `ai-workflow/memory/hashline_v2_spec.md` (200+ lines, 9 section): 왜 Hashline / 9-area 점진 차용 결정 table / content hash 결정 sha2 vs xxhash / Read v2 spec / Edit v2 line_anchored spec §5 / tree-sitter + snapshot store roadmap.
- **sha2 0.11 tools crate dep 추가** (xxhash-rust 미도입 — session-scope 16-bit fingerprint 으로 충돌 회피 충분).
- **12 test 추가** (content_hash 8 + read v2 4 신규 = chunked absolute line preserved + full vs chunked hash 동일성 invariant).
- **검증**: cargo clippy --workspace --all-targets -- -D warnings clean (sha2 0.11 → 3 catch text rewrite fix) / cargo test --workspace --lib **436 pass + 0 fail + 2 ignored** (D-103 baseline 424 → +12 회귀 0).
- **차용 범위 결정 (9-area 점진 채움)**: D-104 = LINE:TEXT + content_hash 2 area. D-105+ = Edit v2 line_anchored 3 area. v1.5+ = tree-sitter `replace block N` 2 area. v2 = SnapshotStore + 3-way merge 2 area.
- **다음**: D-105 Edit v2 line_anchored 진입 (spec §5 구현).
- 누적 결정 51 → **52**. main = `43bc908`.
- W14 까지 `ask` 호출 시 `MINIMAX_API_KEY` env var 필수 → OAuth login 후 env var 없으면 MockClient fallback
- cli LLM client builder 를 `resolve_llm_client()` helper 로 추출. opencode multi-source credential chain 패턴
- 우선순위: (1) `~/.myharness/oauth/minimax.toml` OAuth access_token → (2) `MINIMAX_API_KEY` env → (3) `ANTHROPIC_API_KEY` env → (4) MockClient fallback
- token 만료 시 WARN + env var fallback. **자동 refresh 는 W15.b** 에서 LLM client 레벨 추가
- 1 commit dual-push. 누적 37+ commit

---

## 4. 진행 중 / 미해결 (In Progress / Open)

### In Progress
- **TASK-005-1 (D-43~D-52 + D-53~D-58 follow-up 완료, 38+ commit dual-push)** — v1 MVP 8/8 waves + W14 (Device Authorization Grant) + W14.4~14.6 (`--no-browser` 3 모드 + 단위 변환) + W15.a (OAuth token 자동 resolve) + W15.b (OAuth token 자동 refresh). 388 tests pass, 0 fail, 2 ignored. **종료 선언 또는 TASK-005-2 (v1.5) 진입 대기** (yklee 결정).
- 6개 심층분석 + claude-code + PROVIDERS.md 의 통합 인덱스 (`docs/references/README.md`) — 미작성

### Open
- my_harness 의 도메인별 (코드/서버/환경) 명령 가이드 — yklee 의 개인 인프라 정보 필요 (TASK-002)
- my_harness 의 token compression layer — headroom library/proxy/MCP 3-mode 중 픽 (TASK-007 예정)
- PROVIDERS.md 의 3-way 실측 비교 (rig-core 1안 vs Vercel AI SDK 2안) — 별도 sprint
- 4-워커 division 룰 vs 우리 하네스의 boundary — 현재 룰 그대로 유지 (의사결정 D-04)

---

## 5. 참고 자료 인벤토리 (Reference Inventory)

### 5.1 표준/오버레이 (외부 의존)
- `ykylee/Standard-AI-Workflow` v0.5.0-beta
- `ykylee/minimax-code` harness overlay
- `MiniMax Code` 런타임 (외부 4-워커: Claude/Codex/Gemini/OpenCode)

### 5.2 reference repo (7개, 모두 Gitea private 미러 + GitHub dual-remote)
| repo | GitHub | Gitea | 분석 doc |
| --- | --- | --- | --- |
| opencode | sst/opencode | yklee/opencode | `docs/references/opencode.md` |
| aider | Aider-AI/aider | yklee/aider | `docs/references/aider.md` |
| codex | openai/codex | yklee/codex | `docs/references/codex.md` |
| goose | block/goose | yklee/goose | `docs/references/goose.md` |
| gemini-cli | google-gemini/gemini-cli | yklee/gemini-cli | `docs/references/gemini-cli.md` |
| claude-code | anthropics/claude-code | yklee/claude-code | (2차 분석 인용, 정식 14섹션 미작성) |
| headroom | chopratejas/headroom | yklee/headroom | `docs/references/headroom.md` |

### 5.3 우리 프로젝트 산출물 (my_harness)
- `MiniMax.md` — Mavis 진입점, 도메인별 TODO 명령
- `docs/PROJECT_PROFILE.md` — 3-도메인 스코프 + 도메인별 명령 §3.1
- `docs/REFERENCES.md` — TASK-004 1차 8축 비교표
- `docs/references/ANALYSIS_PLAN.md` — 14섹션 표준 템플릿
- `docs/references/PROVIDERS.md` — LLM provider 3-way 비교
- `docs/references/{codex,aider,goose,opencode,gemini-cli,headroom}.md` — 6개 심층분석 (14섹션)
- `docs/development_log.md` — **본 문서**
- `ai-workflow/memory/{state.json,session_handoff.md,work_backlog.md,backlog/}` — 워크플로우 상태
- (v1 추가 예정) `Cargo.toml` / `myharness/` source tree — v1 MVP Rust 빌드 (TASK-005-1) 산출물
- (v1 W3~W6.5) `crates/myharness-tools/` — 5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format. 11 file + `Cargo.toml` + `SKILL.md`

### 5.4 mavis 인프라 메모
- agent memory: `~/.mavis/agents/mavis/memory/MEMORY.md` — worker long Write call 죽음 패턴 (D-16)
- user memory: `~/.mavis/memory/user.md` — yklee 프로필 + 작업 스타일
- plan outputs: `~/.mavis/plans/plan_30f3d6bf/` (취소, 직접 takeover), `plan_52a216af/` (취소, 직접 takeover)
- 환경: `XDG_CONFIG_HOME=/Users/yklee/.mavis/agents/mavis` — gh CLI keychain 충돌 → `~/.mavis/agents/mavis/gh → ~/.config/gh` symlink (D-19)

---

## 6. 다음 milestone 후보 (Next Milestones)

> 우선순위: ★★★ = 즉시, ★★ = 1주 내, ★ = 차후

| 우선순위 | milestone | 의존 |
| --- | --- | --- |
| ★★★ | **TASK-005 스택 결정** (Rust vs TS) | **결정 완료 — Rust 1안** (D-36, 2026-06-07, 본 문서 §5 정식 기록). TASK-005-1 W2 (`myharness/` workspace init) 진행 중. 의존성 고정: `rig-core = "0.38"`, `rmcp = "1.7"`, `ratatui`, `keyring`, `cargo-dist`. 후속 의존 결정들은 모두 Rust 확정. |
| ★★★ | **TASK-006 TUI 라이브러리** (ratatui vs React/Ink) | **결정 완료 — ratatui** (D-36 의 TASK-005 Rust 1안 정합 자동 확정, 본 문서 §5). v0.1.0 부터 `myharness/crates/tui/` 진행. |
| ★★★ | **docs/references/README.md** 통합 인덱스 | §5.3 의 모든 파일 |
| ★★ | **TASK-002 도메인별 명령 가이드** | yklee 인프라 정보 (별도 세션) |
| ★★ | **TASK-007 headroom 통합 설계** (library/proxy/MCP) | `headroom.md` §13.1-13.5 |
| ★ | **PROVIDERS.md 실측 비교** (rig-core vs Vercel AI SDK) | TASK-005 결정 완료 후 — rig-core 측만 검증 |
| ★ | **CCR 패턴 my_harness 통합** (headroom §13.3) | TASK-007 후 |
| ★ | **CacheAligner 패턴** (headroom §13.5) | Rust stack 정합 후 (TASK-005 결정) |
| ★ | **claude-code 정식 14섹션** | §5.2 의 정식 repo 분석 |
| ★ | **TASK-008 Provider fallback list** (3 모델) | yklee 의 LLM 선호/비용 |

> **Note**: 본 §6 표의 결정 표기는 README.md / PROJECT_PROFILE.md / AGENTS.md / MiniMax.md 의 결정 표시와 SSOT 정합. 결정 본 기록은 본 문서 §5 (D-NN) 가 1차 출처.

### 2026-08-14 — TASK-004 재방문 (D-127~D-133, 7 reference v2 영향 분석)

- **사용자 trigger**: "세월이 많이 지나서 레퍼런스들이 발전을 많이 했어. 다시 조사하자"
- **선택**: 전체 7-doc 재조사 + 결론 갱신
- **7 worktree** (`analysis/<name>-v2` branch) + 7 워커 (D-16 chunked write + D-73 prompt lesson). 순차 실행 (rate limit 회피).
- **06-09 이후 발전량**: opencode 1457 (default branch=dev) / aider 0 / codex 1996 / headroom 106 (실측, prompt 1085 = hallucination) / goose 661 / gemini-cli 130 / claude-code 594 (실측, prompt 66 = 자동화 commit 만)
- **7 reference 영향 분석** (각 1 commit, +1,987 lines total)
- **D-130 핵심**: D-66/D-67/D-68 tract 재평가 REJECT (Rust 측 안정성 ↑ = RTK(v0.22.4) + Rust proxy metrics(headroom 자체) → ONNX 무관). **v2.0 ONNX 백로그 = OOS 유지**
- **D-133 핵심**: D-34/D-40 §11.2 잠금 정합 검증. 우리 영향 0. 결정 변경 불요
- **다음 1순위 (yklee 결정)**: D-130 즉시 follow-up 2 commit (CCR + Memory) / D-130 §16.3 cli wrap → TASK-005-2 v1.5+ Sub-task 2 / D-127 §16.c session retry / D-129 §16.2 effective permission
- **누적 결정 69 → 81** (D-127~D-133, 7 신규). main = `8782abf`. 결정 log 후속

### 2026-08-14 — D-134 + D-135 overlay 재구성

- **D-134**: grok-build 8번째 reference, 14섹션 실측. overlay vs 포크 미결정으로 종료
- **D-135 (본 세션, yklee 확정)**: 제품 경로 = **A overlay**. `grok` 엔진 + myharness 래퍼/plugin
- **문서**: CONCEPT §0/§5.1/§5.7/§5.8/§6/§8 갱신. 신규 [`docs/architecture/DETAILED_DESIGN_OVERLAY.md`](./architecture/DETAILED_DESIGN_OVERLAY.md). README / PROFILE / MiniMax / AGENTS / REFERENCES / grok-build §15 / INITIAL_DESIGN·REQUIREMENTS 배너
- **OOS**: 자체 Plugin 인프라 (A1~A4), grok 소스 포크, v0 crates 신규 기능
- **다음**: PR-1 plugin 스캐폴드 → PR-2 thin CLI
- **누적 결정 77 → 78** (D-135). 코드 0

### 2026-08-14 — D-136 overlay 구현 계획 + M1

- 로드맵/WBS: `docs/architecture/OVERLAY_IMPLEMENTATION_PLAN.md` (M0–M4)
- M1: `plugins/myharness/` + `bin/myharness` + `scripts/overlay_smoke.sh` PASS
- 다음: M2 MiniMax + skills + PreToolUse + task
- 누적 결정 78 → **79**

### 2026-08-14 — D-137 overlay M2

- setup-model + 3-도메인 skills/hooks + task start/end
- overlay_smoke 확장 PASS. live MiniMax 는 키 opt-in
- 다음 M3 install.sh
- 누적 결정 79 → **80**

### 2026-08-14 — D-138 overlay M3

- `scripts/install.sh` + README 설치. Rust clap 보류
- smoke PASS. 다음 M4 (승인) 또는 live MiniMax
- 누적 결정 80 → **81**

### 2026-08-14 — D-140 S0+S1 Owned Surface

- 제품 재설계 SSOT `docs/architecture/DETAILED_DESIGN_SURFACE.md`
- CONCEPT §0 화면=surface, 엔진=숨긴 grok. 기본 grok TUI 문장 폐기
- `surface/` 크롬 TUI + brand remap. `cargo test --manifest-path surface/Cargo.toml` PASS
- 다음 PR-S2 12 동사
