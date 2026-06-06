# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-05
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json)

## Current Focus

- **방향 전환 (2026-06-05 22:00)** — yklee 가 `my_harness` 를 단순 워크플로우 컨슈머가 아니라 *직접 개발/배포할 CLI/TUI 코딩 에이전트 하네스* 의 소스 트리로 키우기로 결정.
  - 타겟 플랫폼: **Windows / Linux / macOS** 동시 지원
  - 표준 AI 워크플로우 (state.json / handoff / backlog / minimax-code 오버레이) 는 그대로 준수
  - 3-도메인 (코드 개발 / 서버 관리 / 환경 셋업) 진입점은 미래 CLI 의 기능 스코프
- **레퍼런스 수집 + Gitea 이전 완료 (2026-06-05~06)** — 5개 오픈소스 하네스를 `/Users/yklee/repos/harness-refs/` 에 클론, Gitea 개인 인스턴스로 push.
  - Gitea: https://homelab.ddn777.synology.me/gitea (yklee 계정, 5개 repo 모두 `private`)
  - 각 repo 구조: `origin` = Gitea (yklee 가 관리), `upstream` = 원본 GitHub (재싱크용)
  - 초기 `--depth 1` 클론의 shallow 문제 → 각 repo `git fetch --unshallow upstream` 으로 해결
  - upstream URL 매핑은 `/Users/yklee/repos/harness-refs/.upstream-urls` 에 저장
  - 4/5 가 `AGENTS.md` / `GEMINI.md` / `CLAUDE.md` 진입점 보유 — 우리 표준 워크플로우가 산업 표준과 정합
- 다음 세션은 TASK-004 (레퍼런스 비교 분석) 부터 진행.

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 [차기] 하네스 워커 smoke check 컨슈머 보정: planned
- TASK-003 [차기] 도메인별 명령 가이드 작성 (코드/서버/환경): planned
- TASK-004 [즉시] CLI/TUI 툴 레퍼런스 5종 비교 분석: planned
- TASK-005 [방향 확정 후] my_harness 의 CLI/TUI 툴 전환: planned
- N/A: blocked

## Key Changes

- 2026-06-05 1차 (21:xx) — `bootstrap_workflow_kit.py` 로 minimax-code 오버레이 적용. 1bfae06 커밋.
- 2026-06-05 2차 (21:xx) — 스코프를 3-도메인으로 확장. `MiniMax.md` 와 `PROJECT_PROFILE.md` 보강. 0266610 커밋.
- 2026-06-05 3차 (22:xx) — **방향 전환**: 5개 오픈소스 하네스 레퍼런스 클론. TASK-004/005 추가. 638cb90 커밋.
- 2026-06-05 4차 (22:xx) — Gitea repo 5개 생성 (private, auto_init=false). push 시도 → shallow 거부.
- 2026-06-06 5차 (20:xx) — upstream 추가 + `git fetch --unshallow` 5개 병렬, Gitea push 성공. origin=Gitea / upstream=GitHub 듀얼 remote 구조 확립.
- 2026-06-06 6차 (21:00) — TASK-004 1차: `docs/REFERENCES.md` draft (5×8 비교표 + 1-페이지 프로필 + 스택 추천). 63fabf0 커밋.
- 2026-06-06 7차 (21:21) — Gitea PAT yklee 가 web UI 에서 발급 + 전달. `git-credential-osxkeychain` (Homebrew git 2.53 내장) 으로 macOS keychain 에 creds 주입. 5개 clone fetch/push 프롬프트 없이 동작 확인.

## 다음에 할 일 (Next Actions)

- [x] TASK-004 1차 draft 작성 (`docs/REFERENCES.md`) — yklee 리뷰 대기
- [x] Gitea PAT 셋업 완료 — 토큰 macOS keychain 에 저장, 5개 clone `git fetch/push` 프롬프트 없이 동작
- [ ] **yklee**: REFERENCES.md §5 의 Rust 1안 vs TypeScript 2안 중 픽
- [ ] TASK-002/003 은 우선순위 medium 으로 후순위. 단, TASK-005 진행 중 새 CLI 의 워크플로우 레이어 설계 시 결론 재활용
- [ ] TASK-004 1차 draft 리뷰 후 yklee 가 deep-dive 1~2개 픽 → §3 1-페이지 프로필 확장
- [ ] TASK-005 시작: 스택 결정 → MVP 범위 → 컨셉 한 줄 → 세부 분해

## Risks & Blockers

- 5개 레퍼런스 중 **Goose** 가 Agentic AI Foundation 으로 이전 중. URL `block/goose` 는 동작하나 곧 aaif.io 가 canonical 이 될 가능성 — 분석엔 영향 없음, 추적.
- `my_harness` 의 현재 standard_ai_workflow 기반 구조와 미래 CLI/TUI 구조가 공존해야 함. 빌드 시스템 / 의존성 / 진입점 충돌 가능. TASK-005.1 (스택 결정) 시 우선 정리.
- 디스크 사용: harness-refs unshallow 후 ~1.3GB 추정. 분석용이라 유지가 기본. 불필요시 `trash` 로 회수.
- Gitea 인증 (해결됨): macOS keychain 에 PAT 저장됨 (`credential.helper = osxkeychain`). 5개 clone fetch/push 프롬프트 없이 동작. **토큰 값은 메모리/문서에 저장하지 않음**. 토큰 회전 시 Mavis 가 재호출 받아 keychain 갱신.
- yklee 비밀번호는 여전히 메모리/문서에 없음.
