---
status: in_progress
plan: plan_ddcdd2a3
task: tc-1
producer: coder (mvs_d6891b1edecd4e9496d25ddb0f855a3c)
started_at: 2026-06-08 21:18 +09:00 (attempt 4 RESUME)
status_done_at: 2026-06-08 21:32 +09:00 (chunk 5 완료)
cycle4_fixed_at: 2026-06-08 21:55 +09:00 (4 critical + 1 significant + 3 minor fix)
target_output: docs/specs/TC_UNIT.md (1,800~2,200 lines)
actual_output: docs/specs/TC_UNIT.md (2,769 lines / 11 sections / 160 TC)
output_dir: /Users/yklee/.mavis/plans/plan_ddcdd2a3/outputs/tc-1/
---

# TC-1: L1 Unit TC scaffold (60+ entries) — early signal + final

> **NOTE**: 본 파일은 D-16 패턴의 **early signal** 파일. cycle 3 에서 누락되어 verifier FAIL (CRITICAL 4) 발생. cycle 4 에서 write tool 로 실제 생성. engine 측 `outputs/tc-1/deliverable.md` 와 별개로 repo 측 `docs/team/` 디렉토리에 존재.

## 목적
`docs/specs/TC_UNIT.md` 작성 — TASK-005-1 (v1 Rust MVP 구현) 의 L1 Unit TC scaffold. RED-GREEN-REFACTOR 의 RED 단계 진입점. 8 categories × 다수 TC.

## 입력 SSOT (5 docs)
- `docs/architecture/DETAILED_DESIGN_TOOL.md` §7 (TC 30)
- `docs/architecture/DETAILED_DESIGN_BUDGET.md` §6 (TC 8)
- `docs/architecture/DETAILED_DESIGN_SUBAGENTS.md` §3-§6 (TC 54)
- `docs/specs/security-patterns.md` §5.1+§5.5 (TC 40)
- `docs/architecture/DETAILED_DESIGN_RETRY.md` §5 (TC 6)
- `docs/team/REVIEW.md` §6.2 (L1 Unit TC 우선순위)

## TC distribution (160 TC, 8 categories)
| category | TC | SSOT |
| --- | --- | --- |
| myharness-tools (6 builtin × 5 시나리오) | **30** | DD-1 §7 |
| myharness-context (BudgetTracker + CompressionPipeline) | **8** | DD-2 §6 |
| myharness-session (Status/Event/handoff) | **6** | REVIEW §6.2 |
| myharness-plugins (markdown hook / MCP / auto_expose) | **6** | REVIEW §6.2 |
| myharness-llm (AuthManager / FallbackChain / Provider retry) | **10** | REVIEW §6.2 |
| myharness-agents (15 sub-agent × 3-5) | **54** | DD-3 §3-§6 |
| security patterns (9 × 3-7, SP-02 = 16) | **40** | DD-4 §5.1+§5.5 |
| myharness-llm retry (backoff/breaker/exit-code/categorization) | **6** | DD-5 §5 |
| **합계** | **160** | 8 categories |

## status timeline
- [x] **2026-06-08 21:18+09:00** Attempt 4 RESUME 시작. PRE-FLIGHT: TC_UNIT.md 1,097 lines (chunk 1~3 prior attempt). engine deliverable.md status=in_progress.
- [x] **2026-06-08 21:32+09:00** Chunk 5 완료. TC_UNIT.md = 2,768 lines. status=done.
- [x] **2026-06-08 21:55+09:00** Cycle 4 verifier FAIL fix. 4 critical + 1 significant + 3 minor in-place edit. 본 파일 write (cycle 3 누락 회복).
- [x] **DONE**

## chunked write D-16 패턴 (5 chunk)
- **chunk 1** (prior attempt 1, 1,097 lines): VERDICT + §0 + §1 + §2 myharness-tools TC (30)
- **chunk 1 (resume append)**: §3 myharness-context TC 8 (1,335)
- **chunk 2 (resume append)**: §4 myharness-session TC 6 + §5 myharness-plugins TC 6 (1,736)
- **chunk 3 (resume append)**: §6 myharness-llm TC 10 (2,033) — 1,800 target hit
- **chunk 4 (resume append)**: §7 myharness-agents TC 54 (2,371)
- **chunk 5 (resume append)**: §8 security TC 40 + §9 retry TC 6 + §10 handoff (2,768)
- **cycle 4 fix (in-place Edit)**: 4 critical + 1 significant + 3 minor (2,769 lines)

## Cycle 4 fix 상세 (2026-06-08 21:55+09:00)
- **CRITICAL 1**: TC-T-010 line 501 quote syntax — `json!({ "path: "/nonexistent..." })` → `json!({ "path": "/nonexistent...", "content": "x" })` (unclosed quote 제거)
- **CRITICAL 2**: TC-L-004 return type align — `assert_eq!(result, "primary response")` (String) → `assert_eq!(result.content, "primary response")` (struct field). TC-L-005/TC-R-006 와 동일 `ChainResult { content, fallback_used }` struct 통일
- **CRITICAL 3**: TC-C-001 BudgetTracker::new_for_test signature align — 3-arg async Result → DD-2 §6.2 4-arg sync `(model_length, threshold, accumulated, system_prompt_tokens)`. TC-C-002/003/004/005/007 의 5건 call site 동일 align
- **CRITICAL 4**: 본 파일 (docs/team/deliverable_tc1.md) write tool 로 실제 생성. cycle 3 에서 producer 8/8 self-check 가 file 존재 미확인 + claim-only. **D-16 정합 = engine 측 outputs/tc-1/deliverable.md 와 별개로 repo 측 docs/team/early signal 도 반드시 존재**
- **SIGNIFICANT**: headline (line 5) + §0.0 + §1.5 self-check #2 의 ✅ PASS → ⚠ (sig+placeholder 64/160 disclosed, §10.2 정직 disclosure 정합)
- **MINOR 1**: TC-A-002~005 name field 추가 (7-field schema 정합)
- **MINOR 2**: TC-T-014 table name `tc_edit_04_timeout_rare_covered_by_dry_run` → code `tc_edit_04_timeout_rare` 통일
- **MINOR 3**: TC-L-007 elapsed range 1000ms → DD-5 §5 spec 500-750ms 정정

## 준수
- **D-16 chunked write** 5 chunk + 200 lines / 7-8KB limit
- **D-26 handoff 4-필드** (§10)
- **D-06** strict — test corpus = EXAMPLEPLACEHOLDER only
- **표준 6 원칙** — 한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff
- **안티 6 미반영** — 1 surface md, 단일 Rust, 6 builtin, 2 surface, local memory, MIT
- **모든 TC** — actual Rust test code (10-30 lines) + mock strategy + SSOT §X.Y cross-ref
- **TDD RED 진입점** — `cargo test --workspace` 160 fail → GREEN → REFACTOR
