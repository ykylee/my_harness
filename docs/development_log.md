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

---

## 4. 진행 중 / 미해결 (In Progress / Open)

### In Progress
- **TASK-005 my_harness 스택 결정** (Rust 1안 vs TypeScript 2안) — `REFERENCES.md` §5 + `PROVIDERS.md` + `headroom.md` §13 입력 준비됨. 결정만 남음.
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
| ★★★ | **TASK-005 스택 결정** (Rust vs TS) | 본 문서 §1.4, `REFERENCES.md` §5, `PROVIDERS.md` |
| ★★★ | **docs/references/README.md** 통합 인덱스 | §5.3 의 모든 파일 |
| ★★ | **TASK-002 도메인별 명령 가이드** | yklee 인프라 정보 |
| ★★ | **TASK-007 headroom 통합 설계** (library/proxy/MCP) | `headroom.md` §13.1-13.5 |
| ★ | **PROVIDERS.md 실측 비교** (rig-core vs Vercel AI SDK) | TASK-005 결정 후 |
| ★ | **CCR 패턴 my_harness 통합** (headroom §13.3) | TASK-007 후 |
| ★ | **CacheAligner 패턴** (headroom §13.5) | TASK-005 결정 후 |
| ★ | **claude-code 정식 14섹션** | §5.2 의 정식 repo 분석 |
