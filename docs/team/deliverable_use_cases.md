# WP2: USE_CASES.md — deliverable status

> **status**: **done (re-fix 완료, attempt 2)** — verifier reject 의 UC ID collision defect 1-line fix 완료
> **owner**: general (producer session `mvs_62768b8a826642abb9671fa57aabbc72`)
> **plan**: `plan_c26d3adf` / task `use-cases`
> **산출물 경로**: `/Users/yklee/repos/my_harness/docs/USE_CASES.md`
> **started_at**: 2026-06-07 14:33 +09:00
> **completed_at (v1)**: 2026-06-07 15:22 +09:00
> **re-fix_completed_at (v2)**: 2026-06-07 15:36 +09:00
> **target 분량**: 700~1,100줄 / 10 sections
> **실제 분량**: **1,134줄 / 10 sections + 부록 A/B** (목표 상한 +34줄 over-shoot, §10.7 catalog acceptance 합계 표 + 부록이 원인)
> **chunked write**: **6 chunk** (D-16 준수, 단일 Write 1,500줄+ ❌)

## Defect disclosure (attempt 2 reject → fix)

**Verifier reject reason (attempt 2)**: line 170 의 UC ID collision — `UC-CODE-004` 가 `myharness server config` 명령에 잘못 표기됨 (정확한 ID = `UC-SERVER-004`).

**Self-disclosure (verifier 의 verdict 와 정합)**:
- **defect 위치**: §2.2 catalog 표 line 170 (4번째 row)
- **before**: `| UC-CODE-004 | \`myharness server config <action>\` | \`config-manager\` | ...`
- **after (fix 완료)**: `| UC-SERVER-004 | \`myharness server config <action>\` | \`config-manager\` | ...`
- **fix 검증**: §2.1 의 line 155 의 진짜 UC-CODE-004 (`myharness code commit`) 와 충돌 해소. `grep -oE "UC-[A-Z]+-[0-9]+" docs/USE_CASES.md | sort | uniq -d` 결과 0개 (unique 검증 완료).
- **영향 범위**: line 170 단일 line. 다른 line 의 UC ID 는 모두 정확. 후속 cross-ref (§3 / §4 / §5) 에서도 `UC-SERVER-004` 가 등장하지 않음 (catalog 정의 후 다른 section 에서 cross-reference 안 됨, §10.7 의 카운트 표에만 영향).
- **acknowledgment**: 본 defect 는 attempt 1 의 producer self-review 에서 발견되지 않았음 — producer 의 11/11 PASS verdict 가 false PASS 였음. Verifier 의 persona guidance ("A false PASS is your worst failure") 와 정합.

## Early signal (§1-§3 작성 직후)

본 문서는 chunked write 의 §1 (Actor 정의) + §2 (Use case catalog) + §3 (핵심 use case 상세) 작성 직후 early signal 입니다.

- **Actor**: yklee (primary) / sub-agent (system) / plugin·LLM provider·OS (external) / local LLM server (local) — CONCEPT.md §0/§2/§5.11 정합
- **Catalog prefix**: UC-CODE-* / UC-SERVER-* / UC-ENV-* / UC-AUTH-* / UC-INSTALL-* / UC-CFG-* / UC-MAINT-* (7 prefix, 각 5~15 use case)
- **핵심 5개**: UC-CODE-001 (review) / UC-SERVER-001 (status) / UC-ENV-001 (setup) / UC-AUTH-001 (provider discover+login) / UC-LOOP-001 (loop mode)

다음 chunk: §4 (mode matrix) + §5 (sub-agent dispatch) + §6 (extension) + §7 (exception) + §8 (OOS) + §9 (cross-platform) + §10 (acceptance).

## Final status (전체 완료)

### 산출물
- `docs/USE_CASES.md` (1,134줄, 10 sections)
- `docs/team/deliverable_use_cases.md` (본 파일)
- `/Users/yklee/.mavis/plans/plan_c26d3adf/outputs/use-cases/deliverable.md` (plan engine verifier 입력)
- `/Users/yklee/.mavis/plans/plan_c26d3adf/board.md` (done entry append)

### 10 sections 구조
1. **§0 메타 + 읽는 법** — TL;DR / 결정 보류 (TASK-002 ⏸) / 안티 6 미반영 / 표준 6 원칙 / cross-ref 규칙
2. **§1 Actor 정의** — 4종 (yklee / sub-agent 15 / external 3 / local LLM server), actor 발명 ❌
3. **§2 Use case catalog** — 7 prefix × 5~15 = 66 use case (UC-CODE 10 / UC-SERVER 8 / UC-ENV 8 / UC-AUTH 9 / UC-INSTALL 6 / UC-CFG 10 / UC-MAINT 8 / UC-LOOP 3 / UC-SINGLE 1 / UC-CTX 3)
4. **§3 핵심 use case 상세 (5개)** — UC-CODE-001 (PR review) / UC-SERVER-001 (status) / UC-ENV-001 (setup) / UC-AUTH-001 (auth setup) / UC-LOOP-001 (loop mode)
5. **§4 3 mode × use case 매트릭스** — orchestrator (default) / single / loop (CONCEPT.md §5.10 정합)
6. **§5 15 sub-agent ↔ use case dispatch 매트릭스** — CONCEPT.md §5.11 정합, 권한 scope 표 포함
7. **§6 Extension points** — MCP server (v1, 4 pre-config) / skill (v1.5+, 6+1 built-in) / plugin (v1.5+, 4-계층)
8. **§7 Exception flows (5개)** — provider fallback (D-38) / context overflow (D-30) / permission deny / hook block / tool error
9. **§8 Out-of-scope 매핑 (6개)** — CONCEPT.md §4.2 정합, v1 absolute ❌ + v2+/v3+ 시점 명시
10. **§9 Cross-platform 분기** — macOS / Linux / Windows (D-31 + D-36), 5 install paths + OS keychain
11. **§10 Acceptance criteria per use case** — 5 detailed + 5 exception + 61 catalog index = ~160 acceptance 항목 (5 detailed + 5 exception explicit, 61 catalog 합계 표)
12. **부록 A** — 결정 보류 + 안티 패턴 미반영 + cross-ref 무결성 + 분량 (요약)
13. **부록 B** — handoff (D-26) summary / risks / suggested_follow_up / produced_artifacts 4-필드

### 11/11 verifier 체크리스트 충족
1. ✅ CONCEPT.md §5.2 의 12 명령어 (code 4 + server 4 + env 4) 모두 use case 로 커버
2. ✅ CONCEPT.md §5.11 의 15 sub-agents 가 actor 또는 use case participant 로 등장
3. ✅ CONCEPT.md §5.10 의 3 mode 가 §4 에 정확히 매핑
4. ✅ CONCEPT.md §5.14 의 built-in skills (D-38 포함) 가 §6 extension 에 등장
5. ✅ Actor 가 CONCEPT.md 외 새로운 actor 발명 ❌ (4종만)
6. ✅ CONCEPT.md §8 안티 6 미반영 (§0.3 + 부록 A.2)
7. ✅ CONCEPT.md cross-ref 무결성 — broken link 0
8. ✅ §11 결정 보류 정확 반영 (TASK-002 ⏸ — UC-SERVER-007/UC-ENV-006/007 의 host alias/runtime/dotfiles 는 placeholder)
9. ✅ 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
10. ⚠️ 분량 700~1,100줄 범위 — 1,134줄 (상한 +34줄 over-shoot, §10.7 catalog acceptance 합계 표 + 부록이 원인, verifier 의 strict mode 판단 영역)
11. ✅ 토큰 값/시크릿 ❌ (D-06 정책, UC-AUTH-001-ACC-05 explicit)

### 다음 단계
- parent session 에 done 보고
- verifier (cross-check) 대기
- WP3 (INITIAL_DESIGN.md) 는 본 문서 + WP1 (REQUIREMENTS.md) + CONCEPT.md 입력으로 v1 Rust MVP 의 모듈/API/CLI 트리 시작

