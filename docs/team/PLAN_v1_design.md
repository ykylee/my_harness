# my_harness v1 설계 — Team Plan & Task Spec (2026-06-07)

> **본 문서 = v1 컨셉(CONCEPT.md) → 요구사항 → 유스케이스 → 초기 설계 단계의 팀 구성 + 작업 계획**.
>
> - **입력 SSOT**: `docs/CONCEPT.md` (마스터, D-22)
> - **보조 입력**: `docs/PROJECT_PROFILE.md`, `docs/development_log.md`, `docs/references/*` (7 reference)
> - **산출물 3종**: `docs/REQUIREMENTS.md` + `docs/USE_CASES.md` + `docs/architecture/INITIAL_DESIGN.md`
> - **후속 TASK**: TASK-005-1 (v1 Rust MVP 구현) 의 입력 문서
> - **갱신 정책**: 본 PLAN 종료 후 archive. 다음 단계(TASK-005-1) 시 본 문서 참조.

---

## 0. 메타 (Meta)

- **작성일**: 2026-06-07
- **작성자**: Mavis (Mavis / mavis root session)
- **트리거**: yklee 요청 — "이전 컨셉 정리 결과에 따라서 요구사항 정리 → 유스케이스 도출 → 초기 설계 작성, 에이전트 팀 꾸려서 진행, 먼저 팀 구성 + 작업 계획 문서화"
- **입력 SSOT**: `docs/CONCEPT.md` (1,024줄, 12섹션, D-22~D-40 결정 반영)
- **워크플로우 표준**: `standard_ai_workflow` v0.5.0-beta (D-26, native 6 원칙: 한국어 보고 / 컨텍스트 절약 / 상태값 / 이벤트 소싱 / 비참조 / handoff)

---

## 1. 목적 (Why this plan)

CONCEPT.md v1 마스터 SSOT (1,024줄 / 12섹션) 를 **3개 산출물로 가공**해서 TASK-005-1 (v1 Rust MVP 구현) 의 **명확한 입력 문서**를 만든다:

| 산출물 | 답하는 질문 | 핵심 |
| --- | --- | --- |
| **REQUIREMENTS.md** | WHAT — 무엇을 만들어야 하는가 | FR/NFR, 3-도메인, 제약, 안티 미반영, 결정 보류 |
| **USE_CASES.md** | WHO + HOW — 누가 어떻게 쓰는가 | actor × scenario, 명령어 ↔ sub-agent ↔ built-in skill 매핑 |
| **INITIAL_DESIGN.md** | HOW — 어떻게 만드는가 | Harness 5 components 모듈 / 데이터 흐름 / API/CLI 표면 |

**왜 팀인가**:
- (1) **multi-stage delivery chain** — research → analyze → write 의 3 단계 (mavis-team 트리거 충족)
- (2) **adversarial verification** — 각 산출물에 verifier 가 CONCEPT.md SSOT 와 독립 cross-check (D-25 zero coupling, §11 결정 보류 정확 반영, 안티 6 미반영 검증)
- (3) **분량 위험** — 단일 세션에서 3 docs 합계 2,000~3,400줄 작업 시 D-10/D-15 의 worker long Write abort 패턴 재현 위험. 분할로 회피.
- (4) **user 명시 요청** — "에이전트 팀 꾸려서 진행"

---

## 2. 팀 구성 (Team Roster)

**3 agent, 신규 생성 0**:

| Role | Agent | Engine | 책임 |
| --- | --- | --- | --- |
| **Producer — 분석/문서화** | `general` | OpenCode | WP1 requirements + WP2 use-cases (각 700~1,100줄) |
| **Producer — software design** | `coder` | OpenCode | WP3 initial-design (800~1,300줄, Rust 1안 모듈/데이터흐름/API 표면) |
| **Verifier — adversarial** | `verifier` | OpenCode | 3 WP 모두의 SSOT 일관성 / 결정 보류 / 안티 미반영 / 표준 6 원칙 형식 검증 |

**선정 근거**:
- **`general`** (WP1, WP2): 분석/문서화 중심 작업에 적합. CONCEPT.md 정독 + 구조화된 문서 도출.
- **`coder`** (WP3): software engineering 관점 설계에 강점 — 모듈 트리 / 시퀀스 다이어그램 / CLI 표면 / 데이터 흐름.
- **`verifier`**: producer 와 **독립적으로 CONCEPT.md SSOT 와 cross-check** (재도출 방식, producer 의 산출물을 그대로 인용하지 않고 원문 확인).

**신규 agent 생성 ❌ 이유**:
- 본 작업은 **도메인 특화 영역이 아님** (software engineering 일반 영역)
- 기존 3 worker 가 모두 적합
- registry 공간 낭비 + 정리 부담 회피 (mavis-team skill 의 "sticky agent" 경고)

---

## 3. 워크 패키지 (Work Packages)

### 3.1 WP1 — Requirements (요구사항 정리)

| 항목 | 값 |
| --- | --- |
| 산출물 | `docs/REQUIREMENTS.md` |
| Agent | `general` |
| depends_on | 없음 (cycle 1, WP2 와 parallel) |
| 입력 | `docs/CONCEPT.md` (SSOT), `docs/PROJECT_PROFILE.md` |
| 예상 분량 | 600~1,000줄 / 5~7 섹션 |
| timeout | 30 min (1800000ms) |

**산출 구조 (7 sections)**:
1. **프로젝트 컨텍스트** — CONCEPT.md §0/§1 한 줄 요약
2. **기능 요구사항 (FR)** — 3-도메인별 (코드/서버/환경), CONCEPT.md §5.2 (12 명령어) + §5.11 (15 sub-agents) + §5.14 (built-in skills) 매핑
3. **비기능 요구사항 (NFR)** — 성능 / 보안 / 크로스플랫폼 / UX / 관측성 / 설치·배포
4. **제약 사항 (Constraints)** — Rust 1안 (D-36) / Mavis zero coupling (D-25) / 3 fallback (D-15) / 2-계층 압축 (D-30) / XDG-style 디렉토리 (D-31)
5. **결정 보류 (Open Decisions)** — §11 결정 완료 4건 (TASK-005/006/007/008) 정확히 ✅ + TASK-002 ⏸
6. **안티 패턴 미반영 체크리스트** — §8 의 6 anti-pattern 미포함 확인
7. **채택 패턴 반영 매트릭스** — §7 의 23 adopt 중 v1 적용분 (1차 MVP 8개 중심)

**verifier 검증 기준** (독립 cross-check):
- ✅ 모든 § cross-ref 가 CONCEPT.md 원문과 일치 (숫자/사실/섹션 번호)
- ✅ §11 결정 완료 4건 (TASK-005/006/007/008) 정확히 반영
- ✅ TASK-002 ⏸ — server/env 명령 가이드를 yklee 인프라 정보로 채우지 말 것 (placeholder 만)
- ✅ 안티 6 미반영 (closed source, 듀얼 언어, 100+ slash commands, 5 surface, cloud auto memory, subscription)
- ✅ 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)

### 3.2 WP2 — Use Cases (유스케이스 도출)

| 항목 | 값 |
| --- | --- |
| 산출물 | `docs/USE_CASES.md` |
| Agent | `general` |
| depends_on | 없음 (cycle 1, WP1 와 parallel — CONCEPT.md 만 의존) |
| 입력 | `docs/CONCEPT.md` §5.2 (12 명령) + §5.10 (3 mode) + §5.11 (15 sub-agents) + §5.14 (built-in skills), WP1 requirements (참고용, cycle 1 종료 후 사용) |
| 예상 분량 | 700~1,100줄 / 8~10 섹션 |
| timeout | 30 min (1800000ms) |

**산출 구조 (10 sections)**:
1. **Actor 정의** — yklee (primary user) / sub-agent (system actor) / plugin·LLM provider·OS (external actor) / local LLM server (local actor)
2. **Use case catalog (인덱스)** — UC-CODE-* / UC-SERVER-* / UC-ENV-* / UC-AUTH-* / UC-INSTALL-* / UC-CFG-* / UC-MAINT-*
3. **핵심 use case 상세** (3~5개) — UC-CODE-001 (code review) / UC-SERVER-001 (status) / UC-ENV-001 (setup) / UC-AUTH-001 (provider discover+login) / UC-LOOP-001 (loop mode)
4. **3 agent mode × use case 매트릭스** — orchestrator (default) / single (`--mode=single`) / loop (`--mode=loop --goal`)
5. **Built-in sub-agent ↔ use case dispatch 매트릭스** — §5.11 의 15 sub-agents 가 어떤 use case 에 dispatch 되는가
6. **Extension points** — plugin (v1.5+) / MCP server (v1, 4 pre-config) / skill (v1.5+)
7. **Exception flows** — provider fallback (D-38) / context overflow (D-30) / permission deny / hook block / tool error
8. **Out-of-scope 매핑** — §4.2 의 6 out-of-scope → 어떤 use case 가 의도적으로 누락되는가
9. **Cross-platform 분기** — macOS / Linux / Windows 별 차이 (D-31 + D-36)
10. **Acceptance criteria per use case** — 각 use case 의 완료 조건 (테스트 가능하게)

**verifier 검증 기준**:
- ✅ CONCEPT.md §5.2 의 12 명령어 (code 4 + server 4 + env 4) 가 모두 use case 로 커버
- ✅ §5.11 의 15 sub-agents 가 actor 또는 use case participant 로 등장
- ✅ Actor 가 CONCEPT.md 외 새로운 actor 를 발명하지 않음
- ✅ §5.10 의 3 mode 가 §4 에 정확히 매핑
- ✅ 안티 6 미반영 / 표준 6 원칙 형식

### 3.3 WP3 — Initial Design (초기 설계)

| 항목 | 값 |
| --- | --- |
| 산출물 | `docs/architecture/INITIAL_DESIGN.md` |
| Agent | `coder` |
| depends_on | [requirements, use-cases] (cycle 2) |
| 입력 | `docs/CONCEPT.md` (SSOT) + `docs/REQUIREMENTS.md` (WP1) + `docs/USE_CASES.md` (WP2) |
| 예상 분량 | 800~1,300줄 / 10~12 섹션 |
| timeout | 30 min (1800000ms) |

**산출 구조 (12 sections)**:
1. **설계 목표 + 비-목표** — CONCEPT.md §0 의 5 NOT + §4.2 의 6 out-of-scope
2. **아키텍처 overview** — §5.1 의 layered architecture 다이어그램 (ASCII) + 모듈 경계
3. **모듈 구조 (Rust module tree)** — Harness 5 components (Tools/Context/Session/Plugins/Sub-agents) 별 Cargo crate / module 구조
4. **데이터 흐름 (5 sequence diagrams)** — startup / code review / server status / env setup / provider fallback (ASCII mermaid 또는 sequenceDiagram)
5. **CLI 표면** — §5.2 의 12 명령어 + §5.10 의 3 mode flag + §5.5.2 의 12 auth 명령 (총 ~30 entry points)
6. **LLM 통합** — §5.5 의 4 subsections (지원 provider 6개 / 동적 발견+auth / fallback chain / library)
7. **Context 관리** — §5.6 의 2-계층 압축 (Layer 1 always-on + Layer 2 opt-in headroom 3 알고리즘)
8. **Config + State** — §5.12 의 `~/.myharness/` 디렉토리 구조 + §5.9 의 standard_ai_workflow 통합 (native 6 원칙 + 옵션 Mavis 통합)
9. **Security & Permission** — §5.4 (4 permission mode + hook system + secret management)
10. **Plugin / MCP / Skill 확장** — §5.7 + §5.14 (v1 = MCP 4 pre-config, v1.5+ = plugin 4계층 + skill catalog)
11. **Cross-platform 빌드** — macOS / Linux / Windows (D-31 + D-36, cargo-dist 5 install paths)
12. **오픈 이슈 + trade-off** — v1 구현 시 trade-off 표 (rmcp 성숙도 / ratatui 학습곡선 / Kompress-base binary size / D-28 minimax TBD 등)

**verifier 검증 기준**:
- ✅ Rust 1안 스택 정합성 (CONCEPT.md §11.3 D-36) — ratatui + rig-core + rmcp + keyring + cargo-dist
- ✅ 5 components 완전성 — Tools / Context / Session / Plugins / Sub-agents 모두 module tree 에 등장
- ✅ 3-도메인 (code/server/env) 가 데이터 흐름 + use case 매핑에 모두 등장
- ✅ `~/.myharness/` 디렉토리 구조 (D-31) 정확히 반영
- ✅ 결정 보류 정확히 반영 (TASK-002 ⏸ — server/env 명령 가이드는 placeholder)
- ✅ 안티 6 미반영 / 표준 6 원칙 형식
- ✅ §5.5.2 의 auth CLI 12 명령 + §5.5.3 의 dynamic fallback + D-38 모두 반영

---

## 4. 워크플로우 (Execution Plan)

```
┌──────────────────────────────────────────────────────────────┐
│ Cycle 1 (parallel)                                            │
│   ├─ WP1 requirements  → general → verifier                  │
│   └─ WP2 use-cases     → general → verifier                  │
│         ↓ (둘 다 done)                                        │
├──────────────────────────────────────────────────────────────┤
│ Cycle 2 (sequential)                                          │
│   └─ WP3 initial-design → coder → verifier                   │
│         ↓                                                     │
│   plan_complete = true                                        │
└──────────────────────────────────────────────────────────────┘
```

**engine 설정**:
- `max_concurrency: 3` (cycle 1 에서 WP1 + WP2 동시 실행, 1 spare)
- `max_consecutive_failures: 2` (worker abort 2회 연속 시 owner escalate)
- `max_cycles: 10` (충분한 retry 여유, 1차 시도 + verifier reject 시 재시도)
- `auto_reject_retries: 1` (default)

**timeout**: 각 task 1,800,000ms (30 min, default cap). D-16 패턴 대비 1,500줄+ 단일 Write 위험 → worker prompt 에 chunked write 지시 포함.

**WP1 + WP2 parallel 근거**:
- 둘 다 CONCEPT.md 만 primary input (WP1 의 requirements 는 WP2 의 use cases 와 무관하게 도출 가능)
- WP2 는 `5.2 (12 명령) + 5.10 (3 mode) + 5.11 (15 sub-agents) + 5.14 (skills)` 라는 CONCEPT.md 고정 섹션만 의존
- parallel 실행 시 cycle 1 wall time ~50% 절감 (30min → 15min wall)

---

## 5. 검증 (Verification Strategy)

**원칙**: verifier 는 **producer 의 산출물을 그대로 인용하지 않고** CONCEPT.md 원문에서 claim 을 직접 찾아 매칭 (재도출).

**각 WP 의 verify_prompt 가 강제할 항목**:
1. **숫자/사실/섹션 번호** — `CONCEPT.md §X.Y` 의 모든 인용이 원문과 일치
2. **결정 보류** — §11.1 의 TASK-002 ⏸ 정확히 표시, §11.3 의 4 done 결정 정확히 인용
3. **안티 패턴 미반영** — §8 의 6 anti-pattern (closed source / 듀얼 언어 / 100+ slash commands / 5 surface 동시 / cloud auto memory default / subscription) 중 하나라도 산출물에 등장 시 FAIL
4. **표준 6 원칙 형식** — 한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff
5. **SSOT 일관성** — CONCEPT.md 외 새로운 결정/actor/module 발명 시 FAIL (단, v1 Rust 구현에 필요한 module 이름 / type alias / crate 선정은 OK)

**verifier prompt 템플릿** (각 WP 공통):
```
독립적으로 검증하라. producer 의 산출물을 그대로 인용하지 말고,
CONCEPT.md §X.Y 원문에서 직접 claim 을 찾아 매칭하라.

체크리스트:
1. §11 결정 보류 정확 반영 (TASK-002 ⏸ + TASK-005/006/007/008 ✅)
2. §8 안티 6 미반영 (closed source / 듀얼 언어 / 100+ slash commands / 5 surface / cloud auto memory default / subscription)
3. CONCEPT.md cross-ref 무결성 (broken link 0)
4. §5.2 (12 명령) / §5.11 (15 sub-agents) / §5.10 (3 mode) / §5.14 (built-in skills) 가 모두 매핑됨
5. 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
6. 분량 600~1,300줄 범위

PASS / FAIL + 각 체크리스트 항목별 evidence (CONCEPT.md §X.Y 인용).
```

---

## 6. 성공 지표 (KPI)

| 지표 | 목표 |
| --- | --- |
| 산출물 3 docs | 모두 작성 완료 + verifier PASS |
| CONCEPT.md cross-ref 무결성 | 100% (broken link 0) |
| §11 결정 정확 반영 | 4 done + 1 deferred 정확 |
| 안티 패턴 미반영 | 6/6 |
| 분량 (각) | 600~1,300줄 (worker abort 위험 회피) |
| 분할 (chunked write) | 각 WP 4~6 chunk (D-16 패턴 준수) |
| cycle 수 | 2~3 (cycle 1 = WP1+WP2, cycle 2 = WP3, 필요 시 retry) |
| total wall time | < 90 min (parallel + 1 retry 여유) |

---

## 7. 리스크 + 대응 (D-16 패턴 계승)

| 리스크 | 영향 | 대응 |
| --- | --- | --- |
| **Worker long Write abort** (D-10, D-15) | 1,500줄+ 단일 Write 시 worker 세션 errored 빠짐 | **chunked write** (4~6 chunk) + **early deliverable signal** (§1-§3 작성 후 deliverable.md status=in_progress) + **minimal board noise** (start + end 만) — worker prompt 에 명시 |
| **Verifier 잘못 reject** | owner 시간 낭비 | `override_accept` 으로 accept (verifier 의 claim 이 CONCEPT.md 와 무관할 때) |
| **결정 보류 (TASK-002) 가 user 에 의해 채워질 가능** | 산출물 drift | cycle 2 진입 전 user OK 받기 (placeholder 만) |
| **CONCEPT.md 와 산출물 간 drift** | SSOT 일관성 깨짐 | 각 WP 의 verifier 가 cross-ref 검증 (재도출) |
| **Provider 인증 정보 누출** | 보안 위험 | D-06 정책 준수 — 토큰 값 절대 메모리/문서/git 저장 ❌. v1 spec 의 auth 흐름은 **메커니즘**만 기술 (어떤 provider 가 어떤 env var / keychain slot 을 쓰는지) |
| **5+1 worker 동시 abort** | plan stall | `max_consecutive_failures: 2` → 2회째 시 owner escalate, manual take over 또는 cancel + 수동 진행 |

---

## 8. worker prompt 공통 지시 (D-16 패턴)

각 WP 의 worker prompt 에 다음을 **반드시** 포함:

```
[분할 작성 (chunked write) 전략 — D-16 패턴]

1. 단일 Write/Edit call 이 1,500줄 / 30KB 초과 시 worker 세션이 errored 빠지는 빈도가 높음.
2. 따라서 본 작업은 **4~6 chunk 로 분할**하여 작성하라:
   - chunk 1: §1 + §2 (컨텍스트 + FR/actor)
   - chunk 2: §3 + §4 (NFR/use cases / constraints/exception)
   - chunk 3: §5 + §6 (결정 보류 / 안티 미반영)
   - chunk 4: §7 (채택 패턴 매트릭스 / acceptance criteria / module tree)
   - chunk 5: §8+ (out-of-scope / cross-platform / trade-off)
   - chunk 6: 마무리 + handoff
3. chunk 1 (즉시 §1-§3 또는 그 이상) 작성 직후 곧바로:
   - `docs/team/deliverable_<task>.md` 작성 (status=in_progress, 1줄 summary)
   - 그래야 engine 이 alive 확인 가능 + 중간 실패 시 복구 가능
4. board.md 갱신은 start + end 만 (매 tool call 마다 갱신 ❌)
5. 중간 abort 시 resume: chunk 1-2 가 이미 `docs/<output>.md` 에 있는지 확인 후 이어서 append
6. handoff 형식 준수 (D-26): summary / risks / suggested_follow_up / produced_artifacts

[품질 바 — 즉시 반영]

본 문서는 TASK-005-1 (v1 Rust MVP 구현) 의 **입력 문서**다.
- v1 구현 시 별도 doc 재참조 없이 이 문서만으로 Rust 모듈/API/CLI 트리 시작 가능해야 함.
- 모든 claim 에 CONCEPT.md §X.Y cross-ref 필수.
- 코드 작성이 아니라 설계 문서이므로, Rust code snippet 은 의사 코드 수준 (full impl ❌).
- 표준 6 원칙: 한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff.
```

---

## 9. 다음 단계 (Next Steps)

| 순서 | 액션 | owner | 산출물 |
| --- | --- | --- | --- |
| 1 | 본 PLAN_v1_design.md 작성 | Mavis (오너) | `docs/team/PLAN_v1_design.md` ✅ |
| 2 | mavis-team plan YAML 작성 (inline 본 §10) | Mavis (오너) | `.mavis/plans/plan.yaml` |
| 3 | user 검토 + OK | yklee | (의사결정) |
| 4 | `mavis team plan run` | Mavis (오너) | plan_id |
| 5 | Cycle 1: WP1 + WP2 parallel | general × 2 | REQUIREMENTS.md + USE_CASES.md |
| 6 | CycleReport 처리 (accept/reject) | Mavis (오너) | (의사결정) |
| 7 | Cycle 2: WP3 | coder | INITIAL_DESIGN.md |
| 8 | CycleReport 처리 (accept/reject) | Mavis (오너) | (의사결정) |
| 9 | 산출물 3 docs 최종 검토 + 커밋 | Mavis (오너) | git commit (D-07 dual-remote push) |
| 10 | handoff / state / backlog 갱신 (D-26) | Mavis (오너) | session_handoff / state.json / work_backlog |
| 11 | **다음 TASK 시작**: TASK-005-1 (v1 Rust MVP 구현) | Mavis (오너) | cargo workspace init |

---

## 10. plan YAML (mavis-team)

`/Users/yklee/repos/my_harness/.mavis/plans/plan.yaml` 에 별도 저장. 본 §10 은 reference inline.

```yaml
version: 1
plan:
  name: 'my_harness v1 설계 — 요구사항 → 유스케이스 → 초기 설계'
  max_concurrency: 3
  max_consecutive_failures: 2
  max_cycles: 10
  auto_reject_retries: 1
  verifier_config:
    default_verifiers: [verifier]
    audit_sample_rate: 0.0
tasks:
  - id: requirements
    title: 'WP1: REQUIREMENTS.md — v1 요구사항 정리 (CONCEPT.md SSOT 기반)'
    prompt: |
      [목표]
      docs/REQUIREMENTS.md 작성 — 600~1,000줄 / 5~7 sections.
      TASK-005-1 (v1 Rust MVP 구현) 의 입력 문서. 별도 doc 재참조 없이
      본 문서만으로 Rust 모듈 시작 가능해야 함.

      [입력 SSOT]
      - docs/CONCEPT.md (마스터, 1,024줄, 12섹션) — 모든 § cross-ref 필수
      - docs/PROJECT_PROFILE.md (3-도메인 스코프)
      - docs/development_log.md (D-22~D-40 결정 이력)

      [산출 구조 (7 sections)]
      1. 프로젝트 컨텍스트 (CONCEPT.md §0/§1 한 줄 요약)
      2. 기능 요구사항 (FR) — 3-도메인별 (코드/서버/환경)
         - CONCEPT.md §5.2 의 12 명령어 (code 4 + server 4 + env 4) → FR 매핑
         - CONCEPT.md §5.11 의 15 sub-agents → FR participant
         - CONCEPT.md §5.14 의 6 built-in skills (D-38 포함) → FR
      3. 비기능 요구사항 (NFR) — 성능 / 보안 / 크로스플랫폼 / UX / 관측성 / 설치·배포
      4. 제약 사항 (Constraints) — Rust 1안 (D-36) / Mavis zero coupling (D-25) /
         3 fallback (D-15) / 2-계층 압축 (D-30) / XDG-style 디렉토리 (D-31)
      5. 결정 보류 (Open Decisions) — §11 결정 완료 4건 정확히 ✅
         + TASK-002 ⏸ (placeholder 만, server/env 명령 가이드는 채우지 말 것)
      6. 안티 패턴 미반영 체크리스트 — §8 의 6 anti-pattern 미포함 확인
      7. 채택 패턴 반영 매트릭스 — §7 의 23 adopt 중 v1 적용분 (1차 MVP 8개 중심)

      [반드시 준수 — D-16 패턴]
      - **chunked write**: 4~6 chunk 로 분할 (단일 Write 1,500줄+ ❌)
      - **early deliverable signal**: §1-§3 작성 직후 docs/team/deliverable_requirements.md
        (status=in_progress + 1줄 summary)
      - **minimal board noise**: board 갱신 start + end 만
      - **handoff 형식 (D-26)**: summary / risks / suggested_follow_up / produced_artifacts

      [품질 바]
      - 모든 claim 에 CONCEPT.md §X.Y cross-ref
      - 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
      - §11 결정 보류 정확 반영 (TASK-002 ⏸, TASK-005/006/007/008 ✅)
      - 안티 6 미반영 (closed source / 듀얼 언어 / 100+ slash commands / 5 surface 동시 /
        cloud auto memory default / subscription requirement)
      - 토큰 값/시크릿 절대 ❌ (D-06 정책) — 메커니즘만 기술

      [산출물 경로]
      - docs/REQUIREMENTS.md (메인)
      - docs/team/deliverable_requirements.md (early signal + final status)
    assigned_to: general
    verified_by: verifier
    verify_prompt: |
      [독립 cross-check] producer 의 산출물을 그대로 인용하지 말고,
      CONCEPT.md §X.Y 원문에서 직접 claim 을 찾아 매칭하라.

      체크리스트:
      1. CONCEPT.md §11.1 결정 보류 정확 반영 (TASK-002 ⏸) — ❌ server/env 명령 가이드를
         yklee 인프라 정보로 채우지 말 것 (placeholder 만)
      2. CONCEPT.md §11.3 결정 완료 4건 정확 인용 (TASK-005 Rust 1안 / TASK-006 ratatui /
         TASK-007 headroom 3 algo / TASK-008 provider-auto-config)
      3. CONCEPT.md §5.2 의 12 명령어 모두 FR 로 매핑
      4. CONCEPT.md §5.11 의 15 sub-agents 모두 FR participant 로 등장
      5. CONCEPT.md §5.14 의 built-in skills 모두 FR 로 등장
      6. CONCEPT.md §8 안티 6 미반영 (closed source / 듀얼 언어 / 100+ slash commands /
         5 surface 동시 / cloud auto memory default / subscription requirement)
      7. CONCEPT.md cross-ref 무결성 — broken link 0, 모든 § 번호가 원문과 일치
      8. 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
      9. 분량 600~1,000줄 범위
      10. 토큰 값/시크릿 ❌ (D-06 정책)

      PASS / FAIL + 각 체크리스트 항목별 evidence (CONCEPT.md §X.Y 인용).
    timeout_ms: 1800000

  - id: use-cases
    title: 'WP2: USE_CASES.md — 3-도메인 actor × scenario 유스케이스 도출'
    prompt: |
      [목표]
      docs/USE_CASES.md 작성 — 700~1,100줄 / 8~10 sections.
      TASK-005-1 의 입력 문서. 별도 doc 재참조 없이 본 문서만으로
      Rust 모듈 시작 가능해야 함.

      [입력 SSOT]
      - docs/CONCEPT.md §5.2 (12 명령) + §5.10 (3 mode) + §5.11 (15 sub-agents) +
        §5.14 (built-in skills, D-38 포함)
      - docs/REQUIREMENTS.md (WP1, 참고용 — cycle 1 종료 후 사용 가능)
      - docs/PROJECT_PROFILE.md §3.1 (도메인별 작업 명령)

      [산출 구조 (10 sections)]
      1. Actor 정의 — yklee (primary) / sub-agent (system) / plugin·LLM provider·OS (external) /
         local LLM server (local)
      2. Use case catalog (인덱스) — UC-CODE-* / UC-SERVER-* / UC-ENV-* /
         UC-AUTH-* / UC-INSTALL-* / UC-CFG-* / UC-MAINT-* (각 5~15 use case)
      3. 핵심 use case 상세 (3~5개) — UC-CODE-001 (code review) /
         UC-SERVER-001 (status) / UC-ENV-001 (setup) / UC-AUTH-001 (provider discover+login) /
         UC-LOOP-001 (loop mode)
      4. 3 agent mode × use case 매트릭스 — orchestrator (default) / single / loop
      5. Built-in sub-agent ↔ use case dispatch 매트릭스 — §5.11 의 15 sub-agents
      6. Extension points — plugin (v1.5+) / MCP server (v1, 4 pre-config) / skill (v1.5+)
      7. Exception flows — provider fallback (D-38) / context overflow (D-30) /
         permission deny / hook block / tool error
      8. Out-of-scope 매핑 — §4.2 의 6 out-of-scope → 의도적 누락 use case
      9. Cross-platform 분기 — macOS / Linux / Windows (D-31 + D-36)
      10. Acceptance criteria per use case — 각 use case 의 완료 조건 (테스트 가능)

      [반드시 준수 — D-16 패턴]
      - chunked write (4~6 chunk) + early deliverable signal + minimal board noise
      - handoff 형식 (D-26): summary / risks / suggested_follow_up / produced_artifacts

      [품질 바]
      - 모든 claim 에 CONCEPT.md §X.Y cross-ref
      - 표준 6 원칙 형식
      - §11 결정 보류 정확 반영
      - 안티 6 미반영
      - 토큰 값/시크릿 절대 ❌

      [산출물 경로]
      - docs/USE_CASES.md (메인)
      - docs/team/deliverable_use_cases.md (early signal + final status)
    assigned_to: general
    verified_by: verifier
    verify_prompt: |
      [독립 cross-check]

      체크리스트:
      1. CONCEPT.md §5.2 의 12 명령어 (code 4 + server 4 + env 4) 가 모두 use case 로 커버
      2. CONCEPT.md §5.11 의 15 sub-agents 가 actor 또는 use case participant 로 등장
      3. CONCEPT.md §5.10 의 3 mode (orchestrator / single / loop) 가 §4 에 정확히 매핑
      4. CONCEPT.md §5.14 의 built-in skills (D-38 포함) 가 §6 extension 에 등장
      5. Actor 가 CONCEPT.md 외 새로운 actor 발명 ❌
      6. CONCEPT.md §8 안티 6 미반영
      7. CONCEPT.md cross-ref 무결성 — broken link 0
      8. §11 결정 보류 정확 반영 (TASK-002 ⏸ — server/env 명령 가이드는 placeholder)
      9. 표준 6 원칙 형식
      10. 분량 700~1,100줄 범위
      11. 토큰 값/시크릿 ❌

      PASS / FAIL + 각 체크리스트 항목별 evidence (CONCEPT.md §X.Y 인용).
    timeout_ms: 1800000

  - id: initial-design
    title: 'WP3: INITIAL_DESIGN.md — Harness 5 components Rust 모듈/데이터흐름/API 표면'
    prompt: |
      [목표]
      docs/architecture/INITIAL_DESIGN.md 작성 — 800~1,300줄 / 10~12 sections.
      TASK-005-1 의 입력 문서. 별도 doc 재참조 없이 본 문서만으로
      v1 Rust 모듈/API/CLI 트리 시작 가능해야 함.

      [입력 SSOT]
      - docs/CONCEPT.md (마스터, 1,024줄, 12섹션)
      - docs/REQUIREMENTS.md (WP1)
      - docs/USE_CASES.md (WP2)
      - docs/development_log.md §11.3 (D-36 Rust 1안 결정 근거)

      [산출 구조 (12 sections)]
      1. 설계 목표 + 비-목표 — CONCEPT.md §0 의 5 NOT + §4.2 의 6 out-of-scope
      2. 아키텍처 overview — §5.1 의 layered architecture 다이어그램 (ASCII) + 모듈 경계
      3. 모듈 구조 (Rust module tree) — Harness 5 components 별 Cargo crate / module 구조
         - Tools / Context / Session / Plugins / Sub-agents
         - 3rd-party crate 선정 (ratatui + crossterm + rig-core + rmcp + keyring + tree-sitter + cargo-dist)
      4. 데이터 흐름 (5 sequence diagrams) — startup / code review / server status /
         env setup / provider fallback (ASCII mermaid)
      5. CLI 표면 — §5.2 의 12 명령어 + §5.10 의 3 mode flag + §5.5.2 의 12 auth 명령
         (총 ~30 entry points, 각 syntax + subcommand 표)
      6. LLM 통합 — §5.5 의 4 subsections (지원 provider 6개 / 동적 발견+auth /
         fallback chain / library)
      7. Context 관리 — §5.6 의 2-계층 압축 (Layer 1 always-on + Layer 2 opt-in headroom 3 algo)
      8. Config + State — §5.12 의 `~/.myharness/` 디렉토리 구조 + §5.9 의 standard_ai_workflow 통합
      9. Security & Permission — §5.4 (4 permission mode + hook system + secret management)
      10. Plugin / MCP / Skill 확장 — §5.7 + §5.14 (v1 = MCP 4 pre-config, v1.5+ = plugin 4계층)
      11. Cross-platform 빌드 — macOS / Linux / Windows (D-31 + D-36, cargo-dist 5 install paths)
      12. 오픈 이슈 + trade-off — v1 구현 시 trade-off 표 (rmcp 성숙도 / ratatui 학습곡선 /
          Kompress-base binary size / D-28 minimax TBD 등)

      [반드시 준수 — D-16 패턴]
      - chunked write (4~6 chunk) + early deliverable signal + minimal board noise
      - handoff 형식 (D-26)

      [품질 바]
      - 모든 claim 에 CONCEPT.md §X.Y cross-ref
      - Rust 1안 스택 정합성 (CONCEPT.md §11.3 D-36) — ratatui + rig-core + rmcp + keyring + cargo-dist
      - 5 components 완전성 (Tools/Context/Session/Plugins/Sub-agents 모두 module tree 에 등장)
      - 3-도메인 (code/server/env) 가 데이터 흐름 + use case 매핑에 모두 등장
      - `~/.myharness/` 디렉토리 구조 (D-31) 정확히 반영
      - §11 결정 보류 정확 반영 (TASK-002 ⏸ — server/env 명령 가이드는 placeholder)
      - 안티 6 미반영
      - 표준 6 원칙 형식
      - 토큰 값/시크릿 절대 ❌ — auth 흐름의 메커니즘만 (어떤 env var / keychain slot)

      [산출물 경로]
      - docs/architecture/INITIAL_DESIGN.md (메인)
      - docs/team/deliverable_initial_design.md (early signal + final status)
    assigned_to: coder
    depends_on: [requirements, use-cases]
    verified_by: verifier
    verify_prompt: |
      [독립 cross-check]

      체크리스트:
      1. Rust 1안 스택 정합성 (CONCEPT.md §11.3 D-36) — ratatui + rig-core + rmcp +
         keyring + cargo-dist 모두 §3 module tree 에 등장
      2. Harness 5 components 완전성 — Tools / Context / Session / Plugins / Sub-agents
         모두 module tree 에 등장
      3. 3-도메인 (code/server/env) 가 §4 데이터 흐름 + §5 CLI 표면에 모두 등장
      4. `~/.myharness/` 디렉토리 구조 (CONCEPT.md §5.12 D-31) 정확히 §8 에 반영
      5. §11 결정 보류 정확 반영 (TASK-002 ⏸)
      6. CONCEPT.md §5.5.2 의 auth CLI 12 명령 + §5.5.3 의 dynamic fallback + D-38 모두 §5/§6 에 반영
      7. CONCEPT.md §5.6 의 2-계층 압축 (Layer 1 always-on + Layer 2 opt-in headroom 3 algo) §7 에 반영
      8. CONCEPT.md §5.4 (4 permission mode + hook + secret mgmt) §9 에 반영
      9. CONCEPT.md §5.14 (MCP 4 pre-config + skill 6 built-in) §10 에 반영
      10. CONCEPT.md §8 안티 6 미반영
      11. CONCEPT.md cross-ref 무결성 — broken link 0
      12. 표준 6 원칙 형식
      13. 분량 800~1,300줄 범위
      14. 토큰 값/시크릿 ❌

      PASS / FAIL + 각 체크리스트 항목별 evidence (CONCEPT.md §X.Y 인용).
    timeout_ms: 1800000
```

---

## 11. 한 줄 결정 요약 (Owner Decision)

- **팀 구성**: `general` × 2 (WP1, WP2) + `coder` × 1 (WP3) + `verifier` × 1. 신규 agent 0.
- **플랜 구조**: 2 cycles. cycle 1 parallel (WP1+WP2) → cycle 2 sequential (WP3, depends on [WP1, WP2]).
- **예상 total wall time**: ~60~90 min (parallel + 1 retry 여유).
- **다음 트리거**: user OK → `mavis team plan run /Users/yklee/repos/my_harness/.mavis/plans/plan.yaml`.
