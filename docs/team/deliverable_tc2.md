# TC-2 deliverable signal (final, done)

> status: **done** (4 chunk write complete, deliverable.md self-VERDICT PASS, 19/19 check ✅ + 1 over-shoot warning)
> task: TC-2 L2 Integration TC — 5-체인 crate 간 contract
> started_at: 2026-06-08 05:55 +09:00
> finalized_at: 2026-06-08 21:18 +09:00 (cycle 3 retry, integrity verification only)
> plan: `plan_ddcdd2a3` / task `tc-2`

## Plan

| chunk | content | target lines |
| --- | --- | --- |
| 1 | §0 메타 + §1 정의/범위 + §2 LLM↔Context (5 TC) | ~300 |
| 2 | §3 Context↔Session (5 TC) + §4 Session↔Plugins (5 TC) | ~330 |
| 3 | §5 Plugins↔Tools (5 TC) + §6 Agents↔LLM (5 TC) | ~330 |
| 4 | §7 handoff (D-26) + closing VERDICT | ~150 |
| total | target 800~1,200 lines / 8 sections + handoff | |

## Key inputs cross-referenced

- DD-1 DETAILED_DESIGN_TOOL.md (5-필드 trait Tool, 6 builtin, 4 mode, 9 hook, registry)
- DD-2 DETAILED_DESIGN_BUDGET.md (BudgetTracker AtomicU32, model_length dynamic, Layer 1, Layer 2)
- DD-3 DETAILED_DESIGN_SUBAGENTS.md (5-필드 SubAgent, sealed Output, ToolId, 15 sub-agent)
- DD-4 security-patterns.md (9 builtin patterns, BUILTIN_HOOKS 상수, hook eval engine)
- DD-5 DETAILED_DESIGN_RETRY.md (RetryPolicy, CircuitBreaker, LlmError, FallbackChain)
- INITIAL_DESIGN.md §4 (5 sequence diagrams — wire-up reference)
- REVIEW.md §6.3 (L2 Integration TC 권고 범위)

## Risk / over-shoot

- 분량 800~1,200 target → over-shoot +29~50% (1,000~1,500) 정합 (DD-1 ~+58%, DD-2 +60%, DD-5 +29% precedent)
- 5↔6 (myharness-tools ↔ myharness-agents) cross-check = §6 의 sub-agent allowed_tools 검증 TC 1개로 cover
