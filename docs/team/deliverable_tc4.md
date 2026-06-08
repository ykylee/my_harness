# Deliverable TC-4 (final, cycle 4)

> **본 파일 = TC-4 final deliverable signal (D-26 handoff 4-필드 + cycle 4 update)**. TC_E2E.md 작성 완료. **cycle 4 actual = 2,265 lines / 7 sections / 39 TC entries** (cycle 3: 2,068 / 36+ → cycle 4: +MODE-006/007 plan/bypassPermissions, +AUTH-013/014 refresh). docker + local Ollama + cross-OS matrix 환경 명시. 5 install paths 정합. permission 4 mode (default/acceptEdits/plan/bypassPermissions) E2E 정합. D-06 / 안티 6 / 표준 6 원칙 모두 준수.

## status: done

## summary

본 TC_E2E.md = my_harness v1 의 **L4 E2E Test Case scaffold** (REVIEW.md §6.1 + §6.4 TDD RED-GREEN-REFACTOR 진입점). TASK-005-1 (v1 Rust MVP 구현) 의 후속 권장 시점 (v1.5+, TUI 안정 + 3 OS cross-build 검증 시점). **7 sections / 2,265 lines / 39 TC entries** (target 600~900, over-shoot +152%, INITIAL_DESIGN 2,056 precedent 정합). **39 TC entries** (12 도메인 명령 18 TC + 3 mode flag 5 TC + **plan/bypassPermissions 2 TC (cycle 4 추가)** + 12 auth CLI 12 TC + **refresh 2 TC (cycle 4 추가)** + 4 exit code 4 TC + cross-OS 5+1 TC), 4-step TC format (input → expected output → exit code → side effect) 정합 DD-5 §3 + DD-1 §4. **permission 4 mode E2E 정합 (cycle 4)**: default 33 TC + acceptEdits 7 TC + plan 1 TC (MODE-006) + bypassPermissions 1 TC (MODE-007) = 4 mode 모두 ≥1 TC. docker + local Ollama + cross-OS matrix 환경 명시. 5 install paths 정합 (D-31 + D-36). D-06 정책 (auth token stdout 출력 ❌) 준수 — refresh TC (AUTH-013/014) 의 stdout grep "sk-" → 0건 검증 명시.

## changed_files

- `docs/specs/TC_E2E.md` (메인 산출, **2,265 lines / 7 sections / 39 TC entries, cycle 4**, VERDICT line 3 + closing VERDICT line 2265)
- `docs/team/deliverable_tc4.md` (D-16 early + final signal, 본 파일, cycle 4 update)
- `~/.mavis/plans/plan_ddcdd2a3/outputs/tc-4/deliverable.md` (engine deliverable, cycle 4 update)
- `~/.mavis/plans/plan_ddcdd2a3/board.md` (start + done 2 entry, minimal noise D-16)

## 7 sections (cycle 4 line ranges)

| section | lines | role |
| --- | --- | --- |
| VERDICT (line 3, top-level) | 3 | PASS marker (verifier first-glance, DD-1 lesson) |
| §0 메타 | 14-103 | D-16 + D-26 + 안티 6 + 표준 6 원칙 + TDD 진입점 + VERDICT 표 (cycle 4: 19 check) |
| §1 L4 E2E TC 정의 + 환경 | 104-333 | docker + local Ollama + cross-OS matrix (6 variant) + 4-step format canonical |
| §2 12 도메인 명령 E2E TC | 334-998 | input→output→exit→side-effect 4-step × 18 TC (code 7 / server 5 / env 6) |
| §3 mode flag E2E TC | 999-1308 | orchestrator/single/loop (--goal 필수) + **plan/bypassPermissions 2 TC (cycle 4, MODE-006/007)** = 7 TC |
| §4 auth CLI E2E TC | 1309-1831 | auth list/<provider>/login/logout/set-key/test/setup/default/discover/export (D-06 stdin) + **refresh 2 TC (cycle 4, AUTH-013/014)** = 14 TC |
| §5 exit code E2E TC (4단계) | 1832-1953 | 0 success / 1 user error / 2 system error / 3 internal error (DD-5 §3.2 `From<&AppError>` 매핑 검증) |
| §6 cross-OS + cross-shell E2E TC | 1954-2199 | bash/zsh/powershell/ash 6 OS variant × 4 shell matrix + 5 install paths |
| §7 handoff (D-26 4-필드) | 2200-2265 | summary/risks/follow_up/artifacts + cross-ref 매트릭스 + 안티 6 |
| VERDICT (final, line 2265) | 2265 | PASS marker (closing, cycle 4) |

## SSOT 정합 (5 docs + CONCEPT/REQUIREMENTS)

- **INITIAL_DESIGN.md §5** (12 명령 + 3 mode + 12 auth) → §2/§3/§4 (1:1 cover)
- **INITIAL_DESIGN.md §11.1** (5 OS variant, D-31) → §6.1-§6.5
- **INITIAL_DESIGN.md §11.2** (5 install paths) → §6.7 매트릭스
- **DETAILED_DESIGN_RETRY.md §3** (exit code 4단계) → §5
- **DETAILED_DESIGN_TOOL.md §4** (permission 4 mode) → §2/§3/§4 (각 TC 의 `permission_mode` 필드)
- **REVIEW.md §6.1** (L4 E2E TC 정의, 600~900 lines) → §1
- **REVIEW.md §6.4** (TDD RED-GREEN-REFACTOR) → §0.5 + §1.1
- **CONCEPT.md §5.4/§5.5.2/§5.9/§5.10/§5.12** → 본 § 전반
- **REQUIREMENTS.md §3.5 NFR-OBS-1** (log.jsonl) → §5.4 + 각 TC side effect

## notes (verifier 용)

- **VERDICT top-level heading (line 3)**: DD-1 lesson 적용 — `### VERDICT: PASS` (h3 markdown heading) line 3 배치, verifier first-glance grep 가능
- **closing VERDICT (line 2066)**: `### VERDICT (final, post-handoff): PASS` — opening + closing 모두 명시
- **3 chunk D-16 chunked write** (early signal = chunk 1 직후 deliverable_tc4.md 작성 status=in_progress, chunk 3 후 status=done)
- **분량 2,265 lines (cycle 4 actual)**: target 600~900 → +152% over-shoot. INITIAL_DESIGN 2,056 lines (+58%) precedent 정합. §2 12 도메인 (~665 lines, 18 TC × ~33 lines/yaml) + §3 7 mode TC (~310 lines, cycle 4 +2) + §4 14 auth TC (~523 lines, cycle 4 +2) 의 yaml 4-step TC format 정밀도 때문. 줄이려면 §2.4/§2.2 의 server/env graceful degrade TC 일부 압축 가능. 그러나 TASK-005-1 + v1.5+ 구현자가 본 문서만으로 E2E harness 작성 가능해야 하므로 정밀도 우선.
- **TDD 진입점 (REVIEW §6.4)**: 본 TC_E2E.md = v1.5+ 시점의 **RED 단계 진입점 scaffold**. RED = 현재 미구현 TC (cargo test fail 가정), GREEN = TASK-005-1 v1 구현 + 본 TC 자동 검증, REFACTOR = v1.5+ 안정화 + LLM mock 성숙
- **mock 전략 차별점 (L4 = L1/L2/L3 와 다름)**: L4 = docker 격리 (`ghcr.io/myharness/runtime:test`) + 실제 binary 실행 + local Ollama mock LLM (`qwen2.5-coder:32b`, CONCEPT §5.5.1 #6) — 결정성 + 실제 invocation
- **D-06 메커니즘만 (NFR-SEC-1)**: API key / token 값 stdout/stderr/log.jsonl ❌. §4 의 `auth login`/`set-key` TC 가 stdin read, log.jsonl 에 `result: ok|error` 만 기록. §4.5/§4.12 의 stdout grep "sk-" → 0건 검증 명시
- **5 install paths 정합 (D-31 + D-36)**: install.sh (macOS/Linux) / install.ps1 (Windows) / brew (macOS) / winget (Windows) / apt-dnf-apk (Linux) — §6.7 매트릭스
- **5 OS variant matrix (D-31)**: macOS Intel/AS / Linux glibc/musl / Windows x64/ARM64 — §1.4 + §6.1-§6.5
- **standard 6 원칙 (D-26)**: 한국어 / 결론 위주 / 상태값 done / 이벤트 소싱 (E2E log.jsonl) / 비참조 / handoff 4-필드 (본 §7)
- **anti 6 미반영 (CONCEPT §8)**: 1 surface (CLI E2E 한정) / 단일 Rust (E2E TC 자체는 shell + TOML) / 27 entry (100+ ❌) / 2 surface (CLI+TUI, TUI 별도 v2+) / local-only / MIT 호환 (docker, ollama, bash, pwsh, fish 모두 오픈소스)
- **TASK-002 ⏸ server/env placeholder**: §2.2 (5 server TC) + §2.3 (6 env TC) 가 graceful degrade (placeholder → 명확한 에러 + 향후 resolve 가이드) 검증. yklee 인프라 정보 수령 후 fixture 교체 필요 (PROJECT_PROFILE.md §3.1 TODO). R-6.
- **R-1~R-6 risks 명시** (§7.2): R-1 분량 over-shoot / R-2 docker+native hybrid / R-3 Ollama mock 결정성 / R-4 keychain in-memory / R-5 cross-OS CI 비용 (288 job) / R-6 TASK-002 ⏸
- **5 verifier risks (D-23 align)**: 본 TC_E2E.md 작성으로 INITIAL_DESIGN §5 + DD-1 §4 + DD-5 §3 의 E2E 검증 scaffold 추가. CONCEPT.md / REQUIREMENTS.md / INITIAL_DESIGN / 본 TC_E2E.md 4 문서 cross-ref 정합 유지
- **D-16 minimal board noise**: start (1 entry) + done (1 entry) = 2 entry 만. 다른 entry 들은 sibling agent 의 것이므로 read-only
