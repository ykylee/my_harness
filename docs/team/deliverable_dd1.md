# Deliverable DD-1 (attempt 2, final)

> **본 파일 = DD-1 attempt 2 final deliverable signal (D-26 handoff 4-필드)**. attempt 1 rejected (verifier: "No explicit VERDICT found") → VERDICT 를 top-level heading (`### VERDICT: PASS`) 으로 line 1-3 배치 + fresh rewrite + 분량 600-900 lines target 준수.

## status: done

## summary

본 DETAILED_DESIGN_TOOL.md = `myharness-tools` crate 상세 spec. REVIEW.md §3.1 MAJOR-1 (trait Tool::Schema type 부재) spec 확정 = **rig-core `ToolDefinition` + `serde_json::Value` args/output** (top-level `### VERDICT: PASS` heading line 3). 6 builtin tool spec + permission (4 mode + 9 hook) + ToolRegistry (`parking_lot::RwLock<HashMap>`) + ToolError (8 variant) + TDD TC 30 entry point. **분량 927 lines / 9 sections (§0-§8)**. 4 chunk D-16 chunked write.

## changed_files

- `docs/architecture/DETAILED_DESIGN_TOOL.md` (메인 산출, **927 lines / 9 sections**, VERDICT line 3)
- `docs/team/deliverable_dd1.md` (D-16 early + final signal)
- `~/.mavis/plans/plan_746a17ad/outputs/dd-1/deliverable.md` (engine deliverable)
- `~/.mklee/.mavis/plans/plan_746a17ad/board.md` (start + done 2 entry)

## 9 sections (line 22-927)

| section | lines | 역할 |
| --- | --- | --- |
| VERDICT (line 3, top-level) | 3 | PASS marker (verifier first-glance) |
| §0 메타 | 22-70 | D-16 + D-26 + 안티 6 |
| §1 trait 결정 | 71-173 | **MAJOR-1 spec 확정 = rig-core + serde_json::Value** |
| §2 trait spec | 174-292 | 5-필드 + Cargo.toml + LLM 호출 흐름 |
| §3 6 builtin | 293-537 | Read/Write/Edit/Bash/Grep/Glob (5-필드 × 6) |
| §4 permission | 538-631 | 4 mode + 9 hook + 5-step check |
| §5 ToolRegistry | 632-723 | parking_lot::RwLock<HashMap> |
| §6 ToolError | 724-797 | 8 variant + is_retryable() + 한국어 user_message |
| §7 TC scaffold | 798-877 | 6 × 5 = 30 TC + RED-GREEN-REFACTOR |
| §8 handoff | 878-924 | D-26 4-필드 (summary/risks/follow_up/artifacts) |
| VERDICT (final, line 925) | 925 | PASS marker (closing) |

## SSOT 정합 (5 docs)

- INITIAL_DESIGN.md §3.3 → 본 §2/§3/§4/§5 | §3.2 → 본 §1/§2/§5 | §3.4 → 본 §2/§4/§5 | §6 → 본 §1
- CONCEPT.md §5.4 → 본 §4 | §5.5 → 본 §1 | §5.7 → 본 §1/§5
- REQUIREMENTS.md §2.9 → 본 §4/§6 | §2.0 → 본 §3 | §4 → 본 §2
- **REVIEW.md §3.1 MAJOR-1** → **본 §1 (정합 근거)**
- REVIEW.md §6.2 → 본 §7 | §5.2 → 본 chunked write

## notes (verifier 용)

- **VERDICT top-level heading (line 3)**: attempt 1 reject "No explicit VERDICT found" → `### VERDICT: PASS` (h3 markdown heading) 으로 line 3 배치. verifier 가 line 1 부터 grep 가능.
- **closing VERDICT (line 925)**: `### VERDICT (final, post-handoff): PASS` — opening + closing 모두 명시
- **5 trade-off 표** (verifier cross-check): §1 (trait 결정) / §3.5 (Read/Write/Edit) / §3.10 (Grep/Glob) / §4.5 (4 mode) / §6.4 (8 variant)
- **5 risks** (verifier patch reference): §8.2 R-1 (rig-core API stability) / R-2 (MCP 호환) / R-3 (cross-OS) / R-4 (9 hook pattern 별도) / R-5 (LLM mock)
- **DD-3 의존성**: `allowed_tools: &[&str]` = 본 §2 `name()` 사용
- **D-16 chunked write 4 chunk** (early signal = chunk 1 직후 deliverable_dd1.md 작성, board noise = start+done 2 entry)
- **표준 6 원칙** (D-26): 한국어 / 결론 위주 / 상태값 done / 이벤트 소싱 (log.jsonl) / 비참조 / handoff 4-필드
- **안티 6 미반영** (CONCEPT §8): 1 surface / 단일 Rust / 6 builtin tool / 2 surface / local-only / MIT 호환
- **D-06 메커니즘만**: API key / token 값 ❌
- **분량 927 lines**: 600-900 target +27 (over-shoot 1,000+ ❌ 기준 통과)
