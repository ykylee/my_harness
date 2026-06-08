# Deliverable TC-3 (final, done)

> **status**: ✅ **done** — 3 chunk write 완료
> **owner**: coder (producer session `mvs_a46000c22e6b45c7bf306dc47c9a7f9e`)
> **plan**: `plan_ddcdd2a3` / task `tc-3`
> **산출물 경로**: `/Users/yklee/repos/my_harness/docs/specs/TC_COMPONENT.md`
> **started_at**: 2026-06-08 05:55 +09:00
> **completed_at**: 2026-06-08 06:10 +09:00
> **target 분량**: 800~1,200 lines / 8 sections
> **실제 분량**: **1,012 lines / 8 sections** (§0 메타, §1 L3 정의 + LLM mock 전략, §2 code 5, §3 server 4, §4 env 4, §5 utility 2, §6 3 mode dispatch, §7 handoff + VERDICT top/bottom) — within target range
> **chunked write**: **3 chunk** (D-16 패턴 준수, chunk 1: line 1-304 / chunk 2: 305-734 / chunk 3: 735-1012 closing VERDICT)

---

## Summary

`docs/specs/TC_COMPONENT.md` 작성 완료. **TASK-005-1 (v1 Rust MVP 구현) 의 myharness-agents crate 15 sub-agent e2e TC scaffold** 으로, 본 문서만으로 L3 Component TC 33 entries 작성 가능. **8 sections + VERDICT top-level (line 3) + VERDICT closing (line 991+)**. REVIEW.md §6.4 (L3 권고) + DD-3 §3-§6 (15 sub-agent) + DD-1 §2-§3 (trait Tool + 6 builtin) + DD-2 §4-§5 (2-tier 압축) + DD-5 §3-§4 (retry/breaker) 의 L3 통합 TC scaffold.

**구현 매핑** (5 SSOT 정합 + 33 TC):
- **§1**: L3 Component TC 정의 (L1/L2/L3/L4 4-계층) + LLM mock 3-전략 (rig-core mock provider + script replay hybrid + mock file system) + TC common 5-step pattern (ARRANGE → CONTEXT BUILD → SUB-AGENT RUN → ASSERT Output → ASSERT log.jsonl) + TC ID naming + TDD RED-GREEN-REFACTOR 진입점 (v1 = `#[ignore]`, v1.5+ LLM mock 성숙 시 active)
- **§2**: code 5 sub-agent (code-reviewer/implementer/tester/refactorer/searcher) × 2 (happy + edge) = **10 TC** (TC-CODE-001~010)
- **§3**: server 4 sub-agent (status/log_analyzer/deployer/config_manager) × 2 = **8 TC** (TC-SERVER-001~008) — **TASK-002 ⏸ graceful degrade** 모든 sub-agent 가 placeholder 입력 시 정상 처리
- **§4**: env 4 sub-agent (setup/installer/shell/diagnose) × 2 = **8 TC** (TC-ENV-001~008) — **TASK-002 ⏸ graceful degrade** 동일
- **§5**: utility 2 sub-agent (git_operator/file_searcher) × 2 = **4 TC** (TC-UTILITY-001~004)
- **§6**: 3 mode dispatch (orchestrator/single/loop) × 1 = **3 TC** (TC-DISPATCH-001~003) — fan-out 검증 (3 sub-agent concurrent) / sub-agent spawn ❌ 검증 / ralph-wiggum iteration + exit
- **§7**: handoff (D-26 4-필드) + cross-ref 무결 + risks 5건 + suggested follow-up 7건

**Cross-reference 무결성** (5 SSOT + D-NNN):
- DD-3 §1 (trait SubAgent 5-필드) + §1.2 (sealed Output) + §1.5 (SubAgentContext) + §2 (master table) + §3-§6 (15 sub-agent × 5 sections) + §7 (3 mode dispatch) + §8 (permission matrix) → 본 §1-§6 정합
- DD-1 §2 (trait Tool) + §3 (6 builtin) + §4 (4 permission mode) + §5 (ToolRegistry) → 본 §1.4 (mock tool registry) + §1.5 (mock PermissionContext) + §2-§5 (sub-agent allowed_tools)
- DD-2 §2 (BudgetTracker) + §4 (Layer 1) + §5 (Layer 2) → 본 §1.5 (mock BudgetTracker + headroom)
- DD-5 §1 (RetryPolicy) + §2 (CircuitBreaker) + §3 (ExitCode) + §4 (ErrorCategory) → 본 §1.5 (mock LlmClient) + §2-§5 (sub-agent LLM call retry) + §6.3 (loop mode exit)
- REVIEW §6.3 (L3 권고) + §6.4 (TDD RED-GREEN-REFACTOR) + §5.2 (TASK-002 ⏸) → 본 §1 (L3 정의) + §1.6 (TDD 진입점) + §3 + §4 (graceful degrade)
- CONCEPT §5.11 (15 sub-agent) + §5.5.3 D-15 (LLM error) + §11.1 (TASK-002 ⏸) → 본 §2-§5 (sub-agent id 일치) + §1.5 (LlmError mock) + §3 + §4
- D-15 + D-23 + D-26 + D-29 + D-35 + D-36 → 본 §0.2 (SSOT cross-ref) + §6 (ralph-wiggum loop) + §7 (handoff)

---

## 10 verifier check PASS

| # | verifier check | status | evidence |
| - | --- | --- | --- |
| 1 | 15 sub-agent 모두 component TC (happy + edge) | ✅ PASS | §2-§5 (10+8+8+4 = 30 sub-agent TC) + §6 (3 mode dispatch TC) = 33 TC 합계 |
| 2 | 33 TC entries (15 × 2 + 3 dispatch × 1) | ✅ PASS | §2-§5 summary table 4개 + §6 summary table 1개 = 5 table, 33 row |
| 3 | 각 TC 가 mock LLM 스크립트 + tempdir + assertion 기반 | ✅ PASS | §1.2 LLM mock 3-전략 (rig-core + script replay + mock file system) + §1.4 mock file system (tempdir + fixture) + §1.5 TC common 5-step pattern (ARRANGE → ASSERT Output → ASSERT log.jsonl) |
| 4 | 3 mode dispatch TC (orchestrator/single/loop) 명확 | ✅ PASS | §6 TC-DISPATCH-001 (orchestrator fan-out 검증, 3 sub-agent concurrent) + TC-DISPATCH-002 (single sub-agent spawn ❌, main agent LLM 직접) + TC-DISPATCH-003 (loop ralph-wiggum iteration + exit) |
| 5 | graceful degrade TC (TASK-002 ⏸) 명시 | ✅ PASS | §3 (server 8 TC 모두 TASK-002 ⏸ graceful degrade) + §4 (env 8 TC 모두 TASK-002 ⏸ graceful degrade) + §3.9/§4.9 summary table 의 "TASK-002 ⏸" col |
| 6 | cross-ref 무결 (DD-3 §3-§6 + DD-1 §2 + DD-2 §4 + DD-5 §3) | ✅ PASS | §0.2 SSOT cross-ref (5 docs, 20+ entry) + §7.5 cross-ref 요약 (5 SSOT + D-NNN) |
| 7 | VERDICT marker top-level heading | ✅ PASS | line 3 (DD-1 lesson 적용) + closing VERDICT (line 991+) |
| 8 | 표준 6 원칙 (D-26) | ✅ PASS | §0.3 + §7.1 (한국어 / 결론 / 상태값 / 이벤트 소싱 / 비참조 / handoff 4-필드) |
| 9 | D-06 / 안티 6 미반영 | ✅ PASS | §0.3 (안티 6 매트릭스 6건) + §7.1 (API key / token 값 ❌, env var 이름만) |
| 10 | 분량 800~1,200 lines | ✅ PASS (within target) | 1,012 lines (target +0~26%, INITIAL_DESIGN +58% / DD-5 +29% over-shoot precedent 미적용) |

**VERDICT: PASS** — 10/10 PASS + 분량 1,012 lines (within target 800~1,200, no over-shoot). DD-1 (927) / DD-2 (1,278) / DD-3 (1,990) / DD-5 (776) / TC_UNIT (~1,800~2,200 TC-1) / TC_INTEGRATION (~800~1,200 TC-2) / 본 TC_COMPONENT (1,012) 7-체인 정합. v1 = L1+L2 active (TC-1 + TC-2), L3 = v1.5+ LLM mock 성숙 시점의 optional TC.

---

## Risks

- **분량 precision (within target)**: 1,012 lines vs target 800~1,200 = 정확히 범위 내 (+0~26% over-shoot 미해당). DD-1 (927) / DD-2 (1,278) / DD-3 (1,990) / DD-5 (776) / 본 TC_COMPONENT (1,012) 비교 시 L3 Component TC 가 mid-size spec (15 sub-agent × 2 TC + 3 mode dispatch × 1 + 1 LLM mock 3-전략 섹션 + handoff).
- **LLM mock 진실성 (R-1)**: mock LLM 이 real LLM 과 결과 다를 수 있음. TC 가 mock LLM 의 output struct 형식만 검증하지, LLM 추론 품질 자체는 검증 ❌. §1.7 명시. LLM 추론 품질 = L4 E2E TC (v1.5+, real local Ollama) 에서 별도 검증.
- **TASK-002 ⏸ placeholder (R-2)**: server 4 + env 4 sub-agent = 8 sub-agent 의 host alias / ssh / k8s context / docker host / stack manifest = yklee 인프라 정보 필요 (PROJECT_PROFILE.md §3.1 TODO). v1 = sub-agent module 구조 + dispatch + allowed_tools scope 만 구현, host/stack manifest = placeholder. §3 + §4 의 모든 TC 가 placeholder 입력 (`host: "local"`, `stack: "macos-dev"`, `env: "dev"`, `path: mock tempdir`) 사용. v1 sub-agent module 정상 동작 검증. v1.5+ 에서 yklee 인프라 정보 입력 시 real manifest 교체.
- **3 mode dispatch mock 비용 (R-3)**: TC-DISPATCH-001/003 의 orchestrator/loop mode 가 multi sub-agent LLM call 시 mock LLM 의 fixture load + replay. mock 이므로 cost 0 이지만, test runtime ↑. v1.5+ 에서 LLM mock 시 CI 부담 시 `#[ignore]` 가능. v1 = L1+L2 TC 만 active.
- **cross-OS fixture 차이 (R-4)**: TC-ENV-001 (macOS brew) / TC-ENV-004 (Linux apt) 등 platform 별 TC 가 cross-OS 에서 일관성 필요. mock Bash 가 platform 무관 fixture 반환. v1.5+ 의 real LLM TC = CI matrix (ubuntu/macos/windows, DD-1 §3.6 + DD-7 dual-remote) 에서 검증.
- **loop mode iteration count 비결정성 (R-5)**: TC-DISPATCH-003 의 LLM judge 가 `evaluate_success()` 시 1-라인 "yes/no" 응답. fixture 가 "yes" 일 때 success, "no" 일 때 fail. deterministic 검증 가능. §6.3 의사코드 + mock LlmClient 의 prompt→scenario mapping 으로 iteration 1/2/3 별 canned "yes/no" 응답. iteration count == 3 deterministic.

---

## Suggested Follow-up

1. **TASK-005-1 (Rust 1안 v1 MVP 구현)** — 본 TC_COMPONENT.md + TC_UNIT.md (L1) + TC_INTEGRATION.md (L2) 의 3-계층 TC plan + DD-1 (Tool) + DD-2 (Budget) + DD-3 (SubAgent) + DD-4 (Security) + DD-5 (Retry) 5-체인 입력으로 `myharness-agents::{subagent::{code,server,env,utility,output,pool},orchestrator,permission_scope}` 8 module + 15 sub-agent impl + L3 TC 33개 `#[ignore]` placeholder 작성. TDD 순서: L1 Unit TC 60+ (RED) → sub-agent 1-2 의 L3 TC 만 우선 활성화 (TC-CODE-001~002 등) → orchestrator 구현 → 3 mode dispatch (TC-DISPATCH-001~003).
2. **TASK-005-2 (v1.5+ LLM mock 성숙)** — rig-core mock provider 도입 + L3 TC 33개 모두 `#[ignore]` 해제 + 통합 GREEN 검증. 우선순위: code 5 (DD-3 §3) → server 4 (§4) → env 4 (§5) → utility 2 (§6) → 3 mode (§6/본 §6). 각 sub-agent 별 2 TC (happy + edge).
3. **DD-1 / DD-2 / DD-3 / DD-5 와 통합 검증**: TASK-005-1 구현 시 본 L3 TC 의 `MockLlmClient` = DD-5 §1 RetryPolicy + §2 CircuitBreaker 1:1 적용. `MockFileSystem` = DD-1 §3 6 builtin tool 의 fixture. `MockPermissionContext` = DD-1 §4 4 mode + DD-4 §5.4 9 hook pattern. `MockBudgetTracker` = DD-2 §2 BudgetTracker 80% threshold. 4-crate boundary 의 cross-check.
4. **align 룰 확립**: DD-1 + DD-2 + DD-3 + DD-5 + 본 TC_COMPONENT 5 문서 동시 align (D-23, D-35 룰). 향후 sub-agent 추가 / 변경 시 5 문서 동시 갱신 필수.
5. **TASK-002 ⏸ 해소 시점**: yklee 인프라 정보 (PROJECT_PROFILE.md §3.1 TODO) 입력 시 §3 + §4 의 mock placeholder → real manifest. v1.5+ TASK (TASK-002 follow-up).
6. **L4 E2E TC (TC_E2E.md) 와 정합**: L3 Component TC = sub-agent e2e (mock LLM), L4 = CLI invocation (real LLM via docker + local Ollama). 4-계층 TC plan (TC-1/2/3/4) 동시 align.
7. **verifier 검증**: 10 self-check (위 표) 모두 PASS. 분량 within target (no over-shoot, INITIAL_DESIGN precedent 미적용). 본 handoff + parent session 보고.

---

## Produced Artifacts

- `docs/specs/TC_COMPONENT.md` (메인 산출물, **1,012 lines / 8 sections + VERDICT top-level (line 3) + VERDICT closing (line 991+)**, within target 800~1,200)
- `docs/team/deliverable_tc3.md` (본 파일 — early signal (in_progress, line 4 status) → final (done, line 4 status), D-16 패턴)
- `/Users/yklee/.mavis/plans/plan_ddcdd2a3/outputs/tc-3/deliverable.md` (plan engine verifier 입력)
- `/Users/yklee/.mavis/plans/plan_ddcdd2a3/board.md` (start + end entry append, D-16 minimal board noise)

## cross-references

- **입력 SSOT (5 docs)**:
  - `docs/architecture/DETAILED_DESIGN_SUBAGENTS.md` (1,990 lines, §1 trait + §1.2 sealed Output + §1.3 ToolId + §1.5 SubAgentContext 7-field + §2 master table + §3-§6 15 sub-agent × 5 sections + §7 3 mode dispatch + §8 permission matrix)
  - `docs/architecture/DETAILED_DESIGN_TOOL.md` (927 lines, §2 trait Tool 5-필드 + §3 6 builtin + §4 4 permission mode + §5 ToolRegistry)
  - `docs/architecture/DETAILED_DESIGN_BUDGET.md` (1,278 lines, §2 BudgetTracker 80% threshold + §4 Layer 1 + §5 Layer 2 4 algo)
  - `docs/architecture/DETAILED_DESIGN_RETRY.md` (776 lines, §1 RetryPolicy + §2 CircuitBreaker + §3 ExitCode 4-단계 + §4 ErrorCategory 3 분류)
  - `docs/team/REVIEW.md` (485 lines, §6.3 L2/L3/L4 권고 + §6.4 TDD RED-GREEN-REFACTOR + §5.2 TASK-002 ⏸)
- **plan**: `docs/team/PLAN_v1_design.md` (WP3 spec, §5.2 DD-3 task 정의)
- **후속 task**: **TASK-005-1** (v1 Rust MVP 구현) — 본 TC_COMPONENT.md + DD-1 + DD-2 + DD-3 + DD-4 + DD-5 6-체인 입력. L3 TC 33개 모두 `#[ignore]` placeholder, v1.5+ LLM mock 성숙 시 active.
- **본 plan outputs**: `/Users/yklee/.mavis/plans/plan_ddcdd2a3/outputs/tc-3/deliverable.md`
- **related plan outputs (sibling TC tasks)**: TC-1 (`docs/specs/TC_UNIT.md`) / TC-2 (`docs/specs/TC_INTEGRATION.md`) / TC-4 (`docs/specs/TC_E2E.md`)
