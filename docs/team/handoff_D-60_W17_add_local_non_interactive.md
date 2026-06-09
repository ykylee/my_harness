# D-60 W17 Handoff — `myharness auth add-local` 비대화형 모드 완료

> **session**: 2026-06-09
> **트리거**: TASK-005-2 v1.5 진입 (D-59 follow-up TASK-005-1 v1 MVP 종료 선언 직후) + yklee 요청 — "추천안으로 가보자"
> **SDLC**: DD-AddLocal §6.3 OI-1 해소 + §9 신규 spec + UC-AUTH-010 CI variant + TC §W17 (4 L1 + 2 L2)

## 1. summary (4-필드, D-26)

### 1.1 완료 (DONE)

- **SDLC 문서 3종 patch/추가** (D-60):
  - `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` — §6.3 OI-1 ✅ 해소 표시 + §9 신규 spec (1 flags table + 1 분기 로직 + 1 API + 3 trade-off + 1 risk + 1 TC table + 1 cli 변경 + 1 사용 예시)
  - `docs/specs/TC_UNIT.md` — §W16.5 OI-1 ✅ + §W17 신규 section (4 L1 TC + 4-chapter TDD 사이클)
  - `docs/specs/TC_INTEGRATION.md` — §W17 신규 section (2 L2 TC + probe skip 증명 전략)
- **구현 (W17)**:
  - `myharness-llm/src/add_local.rs` patch: `register_local_provider_non_interactive(url, token, model_id)` 신규 fn 35 lines + 4 L1 unit test (TC-W17-001~004)
  - `myharness-llm/src/lib.rs` patch: re-export `register_local_provider_non_interactive`
  - `myharness-llm/tests/w16_add_local.rs` patch: 2 L2 integration test 추가 (TC-W17-I01, I02) — 같은 파일에 W16 + W17 통합
  - `myharness-cli/src/main.rs` patch: `AuthAction::AddLocal` enum variant → 4-field struct `{ url, token, model, probe_skip }` + `handle_auth_add_local` 분기 + `handle_add_local_interactive` / `handle_add_local_non_interactive` 분리 + `print_register_report` 공통 출력
- **검증 (DONE)**:
  - `cargo test --manifest-path myharness/Cargo.toml -p myharness-llm --lib` — **108 PASS / 0 fail / 2 ignored** (W16 9 + W17 4 신규)
  - `cargo test --manifest-path myharness/Cargo.toml -p myharness-llm --test w16_add_local` — **5 PASS** (W16 3 + W17 2 신규)
  - `cargo build --manifest-path myharness/Cargo.toml -p myharness` — cli 컴파일 OK
  - manual: `myharness auth add-local --help` — help 출력 OK
  - manual: `myharness auth add-local --url ...` (partial) → ERROR "non-interactive 모드: --url 과 --model 모두 필요" (exit 1) ✅
  - manual: `myharness auth add-local --url "not a url" --model x` → ERROR "invalid URL" (exit 1) ✅
  - manual: `myharness auth add-local --url http://localhost:65530/v1 --model x` (unreachable) → ERROR "connection refused" (exit 1) ✅
  - manual: `myharness auth add-local --url http://localhost:65531/v1 --model ci-test-model --probe-skip` → "✓ 로컬 LLM 등록 완료" (exit 0) ✅

### 1.2 산출물 (4-필드, D-26)

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DD-AddLocal §6.3 OI-1 ✅ + §9** | `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` | +~140 lines (§9 spec) | done |
| **TC scaffold §W17** | `docs/specs/TC_UNIT.md` §W17 + `docs/specs/TC_INTEGRATION.md` §W17 | +4 L1 +2 L2 | done |
| **add_local.rs patch** | `myharness/crates/llm/src/add_local.rs` | +~120 lines (fn 35 + 4 TC) | done |
| **lib.rs re-export** | `myharness/crates/llm/src/lib.rs` | +1 line | done |
| **w16_add_local.rs L2 patch** | `myharness/crates/llm/tests/w16_add_local.rs` | +~80 lines (2 L2 TC) | done |
| **cli main.rs patch** | `myharness/crates/cli/src/main.rs` | +~150 lines (AddLocal struct + 3 fn) | done |
| **handoff (본)** | `docs/team/handoff_D-60_W17_add_local_non_interactive.md` | this | done |

## 2. TC scaffold (D-43~D-47 chapter 패턴 준수)

### 2.1 L1 Unit 4개 (TC-W17-001~004)

| TC | 결과 |
| --- | --- |
| TC-W17-001 register_local_provider_non_interactive no token (atomic write + TOML 검증) | ✅ PASS |
| TC-W17-002 register_local_provider_non_interactive with token (keyring in-memory cache) | ✅ PASS |
| TC-W17-003 invalid URL → RegisterError::InvalidUrl | ✅ PASS |
| TC-W17-004 empty model_id → register 성공 (user 책임, available_models = [""]) | ✅ PASS |

### 2.2 L2 Integration 2개 (TC-W17-I01, I02)

| TC | 결과 |
| --- | --- |
| TC-W17-I01 probe skip 증명 (wiremock 0 routes mount → ConnectionRefused 안 받음 = probe 미호출) | ✅ PASS |
| TC-W17-I02 with token → keyring set + providers.toml 갱신 | ✅ PASS |

### 2.3 parallel-safe 패턴 (D-58 follow-up + W16 precedent)

- `MYHARNESS_HOME=tempdir` env mutation 으로 paths.rs 격리
- `#[serial_test::serial(env)]` attribute 로 L1 4개 (TC-001/002/004) + L2 2개 직렬화
- L1 1개 (TC-003) 는 env 무관 → parallel 가능
- 1 session 안에 6 TC 모두 PASS (D-47 chapter 1~3-B 의 27.5% 1-session 패턴 100% 적용)

## 3. risks / follow-up (1 + 3)

### 3.1 risks (1)

- **R-4 (사용자 home providers.toml 덮어쓰기)**: `register_local_provider` 가 `paths::providers_toml()` → `~/.myharness/providers.toml` 에 직접 write. **MYHARNESS_HOME env override 안 쓰면 진짜 사용자 설정 손실 위험**. **대응**:
  - 수동 검증 중 yklee 의 LM Studio 192.168.0.101:1234 설정을 `mavis-trash` 로 cleanup (v17.5 manual test 시 진짜로 덮어버림, 1회성 사고, recover 완료)
  - 향후 W17+ 에서 `~/.myharness/providers.toml.backup.<timestamp>` 자동 백업 권고 (v1.5+ OOS)
  - v1.5+ 에서 `register_local_provider` 시작 시 "→ 덮어쓰기 알림" stderr 출력 (또는 `--yes` flag 요구) — 합의 후 진행

### 3.2 suggested follow-up (3)

- **F-1 (W18+ 후보)**: `~/.myharness/providers.toml.backup.<ts>` 자동 백업 (R-4 대응)
- **F-2 (W18+ 후보)**: `--yes` flag (덮어쓰기 confirm) — interactive 모드에서도 지원
- **F-3 (W19+ 후보, DD §6.3 OI-2)**: Ollama native `/api/tags` 지원 (OpenAI 호환 미활성 시)

### 3.3 OOS (v1.5+ 후보)

- (OI-2~4) — DD §6.3 정리. OI-1 ✅ 해소.

## 4. 의존성 / 변경 파일 (4-필드 produced_artifacts)

### 4.1 Cargo.toml 변경 (0 files)

- W17 은 **추가 의존성 0** — wiremock (W16) + inquire (W16) + url (W16) + serial_test (W16) 모두 재사용

### 4.2 SDLC docs 변경 (3 files)

- `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md`: §6.3 OI-1 ✅ 표시 + §9 신규 spec (~140 lines)
- `docs/specs/TC_UNIT.md`: §W16.5 OI-1 ✅ + §W17 신규 section (~100 lines)
- `docs/specs/TC_INTEGRATION.md`: §W17 신규 section (~50 lines)

### 4.3 impl 변경 (4 files)

- `myharness/crates/llm/src/add_local.rs`: `+register_local_provider_non_interactive` (35 lines) + 4 L1 unit test (~85 lines)
- `myharness/crates/llm/src/lib.rs`: `+register_local_provider_non_interactive` re-export
- `myharness/crates/llm/tests/w16_add_local.rs`: `+2 L2 integration test` (~80 lines)
- `myharness/crates/cli/src/main.rs`: `+enum AddLocal { url, token, model, probe_skip }` (기존 unit → 4-field struct) + `handle_auth_add_local` 3-mode 분기 + `handle_add_local_interactive` / `handle_add_local_non_interactive` 분리 + `print_register_report` 공통 (~150 lines)

## 5. VERDICT

### 5.1 SDLC 통과 (D-26 6 원칙)

- **한국어 보고** ✅
- **결론 위주** ✅ (본 handoff, DD §9.0, §9.4/§9.5 trade-off/risks 우선)
- **상태값** ✅ (D-60 W17 = done, SDLC 3 docs = done, impl = done, tests 6/6 PASS)
- **이벤트 소싱** ✅ (cargo test 결과, manual 검증 4 시나리오 결과 handoff 에 append)
- **비참조** ✅ (DD §9 cross-ref 5 SSOT, §9.5 cross-references)
- **handoff** ✅ (본 문서, 4-필드 D-26 정합)

### 5.2 TC scaffold 통과 (D-43 4-Layer)

- L1 Unit 4 (+ W16 의 8+1 = 13) ✅
- L2 Integration 2 (+ W16 의 3 = 5) ✅
- L3 Component 1 (cli dispatch, --url/--token/--model/--probe-skip flag) — TC_COMPONENT.md patch 미작성 (W17 범위 외, v1.5+ OOS)
- L4 E2E 1 (manual, CI 환경 비대화형) — TC_E2E.md patch 미작성 (CI ❌, manual only)

### 5.3 D-43~D-47 chapter 패턴 준수

- **1 session 안에 4 chapter 묶기** ✅ (D-47 precedent 의 1-session 27.5% 패턴):
  - chapter 1: `register_local_provider_non_interactive` fn (TC-001~002) — RED → GREEN
  - chapter 2: error path + edge cases (TC-003~004) — RED → GREEN
  - chapter 3: cli patch — 3-mode 분기 + 핸들러 분리 + 공통 출력
  - chapter 4: L2 integration 2개 (TC-I01, I02) — wiremock + end-to-end
- chunked write 4 chunk (impl / lib / test / cli)
- cargo test 매 chapter 후 + manual cli 4 시나리오 검증

### 5.4 결론

본 W17 = `myharness auth add-local` 비대화형 모드 (--url/--token/--model/--probe-skip) v1.5 구현 + SDLC 3 docs + 6 TC 모두 PASS. **VERDICT: PASS**.

## 6. 다음 단계

1. **dual push** (Gitea origin + GitHub upstream) — W17 4 commit (impl / lib / test / cli) + 1 doc commit (SDLC)
2. **memory 갱신** (state.json + work_backlog.md + session_handoff.md) — TASK-005-2 v1.5 첫 작업 완료 표시
3. **TASK-005-2 v1.5 다음 작업 결정** — yklee 결정 대기. 후보: OI-2 Ollama native / F-1 backup / Plugin 4-계층 (W19+ 큰 사이클)
4. **W17 follow-up**: R-4 (home providers.toml 덮어쓰기) 대응 — `--yes` flag or 자동 backup. yklee 결정 시.

### 6.1 dual push 명령 (owner)

```bash
cd /Users/yklee/repos/my_harness
git add docs/ myharness/
git commit -m "feat(auth): W17 myharness auth add-local 비대화형 모드 (D-60, v1.5 OI-1)

- SDLC: DD-AddLocal §6.3 OI-1 ✅ + §9 spec + TC §W17 (4 L1 + 2 L2)
- impl: myharness-llm::register_local_provider_non_interactive(url, token, model_id)
- impl: cli AuthAction::AddLocal { url, token, model, probe_skip } + 3-mode 분기
- tests: 6 PASS (L1 4 + L2 2)
- ci/스크립트 환경에서 stdin/stdout non-tty 라도 사용 가능 (CI 가능)"
git push origin feature/v15-add-local-non-interactive
gh pr create --base main --head feature/v15-add-local-non-interactive --title "W17 v1.5 add-local non-interactive"
```
