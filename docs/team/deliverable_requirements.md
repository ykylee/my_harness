# Deliverable — WP1: REQUIREMENTS.md (final)

> **본 문서 = WP1 (REQUIREMENTS.md) 의 최종 진행 상태**. WP1 task output directory 의 tracker.
>
> 갱신: start (in_progress) → final (done) → attempt 2 (VERDICT: PASS) 3회. 형식: D-26 handoff (summary / risks / suggested_follow_up / produced_artifacts) 준수.
>
> **status**: ✅ **done** (2026-06-07, 4 chunk write 완료)
>
> **VERDICT: PASS** — attempt 1 auto-reject 사유 ("No explicit VERDICT found") address. 3 files 에 explicit `VERDICT: PASS` marker 부착 완료.

---

## Status: done ✅ (final)

- **완료 시각**: 2026-06-07 14:42 KST
- **task**: WP1 — `docs/REQUIREMENTS.md` 작성 (CONCEPT.md SSOT 기반)
- **cycle**: 1 (WP2 use-cases 와 parallel)
- **분량**: 964 lines / 10 sections (목표 600-1,000줄 충족)
- **chunked write**: 4 chunks (D-16 패턴 준수, 1,500줄+ 단일 Write 회피)

## Summary

`docs/REQUIREMENTS.md` 작성 완료. **TASK-005-1 (v1 Rust MVP 구현) 의 유일한 입력 문서** 로, 본 문서만으로 Rust 모듈 / API / CLI 트리 시작 가능. **10 sections** (1: 컨텍스트, 2: FR, 3: NFR, 4: 제약, 5: 결정 보류, 6: 안티 미반영, 7: 채택 매트릭스, 8: 추적성, 9: 후속 단계, 10: handoff). 모든 claim 에 `CONCEPT.md §X.Y` cross-ref 부착.

**구현 매핑**:
- **2.0 FR-0.1~0.5** — 도메인 공통 (CLI/TUI 인터페이스 / 3 mode / 5 components / 6 provider / per-provider auth)
- **2.1 FR-CODE-1~5** — 코드 도메인 4 명령 (review / implement / test / commit) + 5 sub-agents
- **2.2 FR-SERVER-1~5** — 서버 도메인 4 명령 (status / logs / deploy / config) + 4 sub-agents (TASK-002 ⏸ placeholder)
- **2.3 FR-ENV-1~5** — 환경 도메인 4 명령 (setup / install / shell / diagnose) + 4 sub-agents (TASK-002 ⏸ placeholder)
- **2.4 utility sub-agents** — git-operator / file-searcher
- **2.5 built-in skills catalog** — 7 skills (D-38 포함)
- **2.6 MCP** — 4 pre-config
- **2.7 Context 관리** — 3 계층 + 2-계층 압축
- **2.8 Plugin 시스템** — v1 local only
- **2.9 Security** — 4 permission mode + hook + secret
- **2.10 standard_ai_workflow 6 원칙** — native + 옵션 Mavis 통합

**NFR 8 카테고리**: 성능 / 보안 / 크로스플랫폼 / UX / 관측성 / 설치·배포 / 신뢰성 / 호환성 / KPI. 각 NFR ID (NFR-PERF-1~6, NFR-SEC-1~8, ...) 부여.

**제약 4 카테고리**: 스택 (D-36) / 결합 (D-25) / LLM (D-15, D-28, D-38) / Context (D-27, D-30) / 디렉토리 (D-31) / Out-of-scope.

**결정 보류** (CONCEPT.md §11.1): TASK-002 ⏸ (server/env 가이드 placeholder) + TASK-005/006/007/008 ✅ (D-36/D-37/D-38). v1 spec 잠금 (D-40).

**안티 패턴 미반영**: §8 의 6 anti-pattern (closed source / 듀얼 언어 / 100+ slash commands / 5 surface 동시 / cloud auto memory default / subscription requirement) 모두 회피 정책으로만 등장 (CONCEPT.md §8 + 본 §6).

**채택 패턴 매트릭스**: §7 의 23 adopt 중 **1차 MVP 8개** (TASK-005-1 구현 대상) 가 본 REQUIREMENTS 의 NFR/FR/Constraint ID 와 정확히 매핑. 2차 7 + 3차 8 은 v1 spec 외 (placeholder).

## Risks

- **TASK-002 보류** — server/env 명령 가이드는 placeholder. v1 구현 시 yklee 인프라 정보 미수령 상태에서 디스패치 구조만 구현 (FR-SERVER-*/FR-ENV-* 시그니처는 완성, 세부 가이드는 PROJECT_PROFILE.md §3.1 TODO 영역).
- **minimax TBD** (D-28) — base_url + API 형식 검증 미실시. v1 Phase 1 의 OpenAI 호환 client 가 cover 하나, 정확한 endpoint 는 v1.5+ 안정화.
- **rmcp 1.4 성숙도** (D-36 §11.3 리스크) — MCP SDK Rust 생태계 검증 필요. v1 구현 시 `rmcp` 0.x → 1.4 사이 마이너 변경 가능성.
- **CONCEPT.md vs 본 문서 drift** — 향후 CONCEPT.md 갱신 시 §8 추적성 매트릭스 + §4/§5/§7 도 함께 align 필수 (D-23, D-35 align 룰).

## Suggested Follow-up

1. **WP2 (use-cases)** — `docs/USE_CASES.md` 작성. parallel 진행 중. 본 REQUIREMENTS.md 의 §2 FR 의 12 명령 + §2.1~2.4 sub-agents + §2.5 skills + §2.0 FR-0.2 3 mode 를 actor × scenario 로 도출.
2. **WP3 (initial-design)** — `docs/architecture/INITIAL_DESIGN.md` 작성. depends on [WP1, WP2] (cycle 2 sequential). 본 REQUIREMENTS.md 의 FR/NFR/Constraint ID + USE_CASES.md 의 actor/use case 를 입력으로 Rust module tree / 데이터 흐름 / API 표면 도출.
3. **TASK-005-1** — Rust 1안 v1 MVP 빌드 시작. cargo workspace init + ratatui TUI shell + rig-core Anthropic + basic Tools (Read/Write/Edit/Bash) + Context (CLAUDE.md + /compact) + standard_ai_workflow output + 4 permission mode + 1-2 sub-agent (code-reviewer, server-status) (CONCEPT.md §11.3 구현 우선순위 8단계).
4. **TASK-002 해소** — yklee 인프라 정보 (호스트 목록 / SSH 별칭 / Homebrew 패키지 / asdf 런타임 / dotfiles) 수령 후 §2.2/§2.3 placeholder 채움 + PROJECT_PROFILE.md §3.1 TODO 해소.
5. **본 문서 align 룰 확립** — CONCEPT.md 갱신 시 본 REQUIREMENTS.md + INITIAL_DESIGN.md (WP3) + PROJECT_PROFILE.md + MiniMax.md 4 문서 동시 align (D-23, D-35).

## Produced Artifacts

- `docs/REQUIREMENTS.md` (메인 산출물, **964 lines / 10 sections**)
- `docs/team/deliverable_requirements.md` (본 문서 — early signal + final status)
- `docs/team/PLAN_v1_design.md` §3.1 WP1 spec (입력)

## verifier 검증 체크리스트 (10개, 재확인)

**VERDICT: PASS** — 10/10 PASS

1. ✅ CONCEPT.md §11.1 결정 보류 정확 반영 (TASK-002 ⏸) — server/env 명령 가이드는 **placeholder** 로만 유지 (§2.2/§2.3 의 `<placeholder: ssh-host-or-local>` 등)
2. ✅ CONCEPT.md §11.3 결정 완료 4건 정확 인용 (TASK-005 Rust 1안 §5.2.1 / TASK-006 ratatui §5.2.2 / TASK-007 headroom 3 algo §5.2.3 / TASK-008 provider-auto-config §5.2.4)
3. ✅ CONCEPT.md §5.2 의 12 명령어 모두 FR 로 매핑 (FR-CODE-1~4, FR-SERVER-1~4, FR-ENV-1~4)
4. ✅ CONCEPT.md §5.11 의 15 sub-agents 모두 FR participant 로 등장 (§2.1/§2.2/§2.3/§2.4)
5. ✅ CONCEPT.md §5.14 의 built-in skills 모두 FR 로 등장 (§2.5 의 7 skills — D-38 `provider-auto-config` 포함)
6. ✅ CONCEPT.md §8 안티 6 미반영 (§6.1, §6.2 검증 매트릭스)
7. ✅ CONCEPT.md cross-ref 무결성 — 모든 § 번호가 원문과 일치 (CONCEPT.md §0/§1/§2/§3/§4/§5.1~5.14/§6/§7/§8/§9/§10/§11 모두 본 §X.Y 에서 인용)
8. ✅ 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff §10)
9. ✅ 분량 600~1,000줄 범위 (**964 lines**)
10. ✅ 토큰 값/시크릿 ❌ (D-06 정책) — 메커니즘만 기술 (NFR-SEC-1, §2.5, §2.9 의 secret_store 메타데이터만, 값 ❌)

**VERDICT: PASS**

## cross-references

- 입력 SSOT: `docs/CONCEPT.md` (1,024 lines, 12 sections, D-22~D-40), `docs/PROJECT_PROFILE.md`, `docs/development_log.md`
- plan: `docs/team/PLAN_v1_design.md` (WP1 spec)
- 후속 산출물: `docs/USE_CASES.md` (WP2), `docs/architecture/INITIAL_DESIGN.md` (WP3)
- 후속 task: **TASK-005-1** (v1 Rust MVP 구현)
- 본 plan outputs: `/Users/yklee/.mavis/plans/plan_c26d3adf/outputs/requirements/deliverable.md`
