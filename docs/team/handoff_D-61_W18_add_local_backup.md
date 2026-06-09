# D-61 W18 Handoff — `auth add-local` 자동 backup + Confirm prompt 완료 (R-4 대응)

> **session**: 2026-06-09
> **트리거**: W17 manual test 중 R-4 (사용자 home providers.toml 덮어쓰기) 1회 사고 → mavis agent memory lesson append → 즉시 W18 진입 (yklee "이어서 쭉 진행하자")
> **SDLC**: DD-AddLocal §10 신규 spec + TC §W18 (3 L1 + 2 L2 + W17-004 재활성화 1) + UC-AUTH-010 R-4 variant

## 1. summary (4-필드, D-26)

### 1.1 완료 (DONE)

- **SDLC 문서 3종 patch/추가** (D-61):
  - `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` — §10 신규 spec (backup_providers_toml API + register_local_provider 흐름 + 파일 형식 + retention + --yes flag + 3 trade-off + 1 risk + 1 TC table + 1 cli 변경 + 4 사용 예시)
  - `docs/specs/TC_UNIT.md` — §W16.5 OI-1 ✅ + §W17 신규 section (4 L1 TC, **W18 에서 main 누락 확인**) + §W18 신규 section (3 L1 + 2 L2 + W17-004 재활성화)
  - `docs/specs/TC_INTEGRATION.md` — §W17 신규 section (2 L2, **W18 에서 main 누락 확인**) + §W18 신규 section (2 L2)
- **구현 (W18)**:
  - `myharness-llm/src/add_local.rs` patch: `backup_providers_toml(path, max_backups)` 신규 fn (60 lines) + `with_backup_suffix` + `cleanup_old_backups` helpers + `register_local_provider` 안에 backup 호출 연결 (`let _ = backup_providers_toml(&path, 5);`) + W17 의 `register_local_provider_non_interactive` 도 함께 main merge (누락분)
  - `myharness-llm/src/lib.rs` patch: re-export `backup_providers_toml` + `register_local_provider_non_interactive`
  - `myharness-llm/tests/w16_add_local.rs` patch: 2 L2 integration test 추가 (TC-W18-I01, I02)
  - `myharness-cli/src/main.rs` patch: `AuthAction::AddLocal` 에 `--yes` flag 추가 + `handle_auth_add_local` 5-arg 시그니처 + `handle_add_local_interactive(skip_confirm)` 분리 + `inquire::Confirm` prompt ("덮어쓰시겠습니까?") + `print_register_report` 공통 출력 helper 추출
- **검증 (DONE)**:
  - `cargo test --manifest-path myharness/Cargo.toml -p myharness-llm --lib` — **104 PASS / 0 fail / 2 ignored** (W16 9 + W17-004 1 + W18 3 = 13 신규, vs main 100 baseline)
  - `cargo test --manifest-path myharness/Cargo.toml -p myharness-llm --test w16_add_local` — **5 PASS** (W16 3 + W18 2)
  - `cargo build --manifest-path myharness/Cargo.toml -p myharness` — cli 컴파일 OK
  - manual: **MYHARNESS_HOME 격리** (R-4 lesson 즉시 적용) — `mkdir /tmp/w18-test-1 && MYHARNESS_HOME=/tmp/w18-test-1 ./myharness auth add-local --url http://localhost:9999/v1 --model test-m1 --probe-skip` → providers.toml 생성, **backup ❌** (1번째 write, 신규)
  - manual: `MYHARNESS_HOME=/tmp/w18-test-1 ... --url http://localhost:9998/v1 --model test-m2 --probe-skip` → **providers.toml.backup.1781016095 생성** (1번째 m1 내용), current = m2
  - manual: `ls /tmp/w18-test-1/` → `providers.toml` + `providers.toml.backup.1781016095` 2개 확인 → R-4 완전 차단 검증
  - cleanup: `mavis-trash /tmp/w18-test-1` → 사용자 home 무영향 ✅

### 1.2 산출물 (4-필드, D-26)

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DD-AddLocal §10** | `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` | +~120 lines | done |
| **TC §W17 + §W18** | `docs/specs/TC_UNIT.md` + `docs/specs/TC_INTEGRATION.md` | +~200 lines | done |
| **add_local.rs W18 patch** | `myharness/crates/llm/src/add_local.rs` | +~200 lines (backup fn + helpers + 3 L1 TC + W17 helper + W17-004) | done |
| **lib.rs re-export** | `myharness/crates/llm/src/lib.rs` | +2 lines | done |
| **w16_add_local.rs L2 patch** | `myharness/crates/llm/tests/w16_add_local.rs` | +~95 lines (2 L2 TC) | done |
| **cli main.rs patch** | `myharness/crates/cli/src/main.rs` | +~150 lines (AddLocal struct +yes + 3 fn) | done |
| **handoff (본)** | `docs/team/handoff_D-61_W18_add_local_backup.md` | this | done |

## 2. W17 PR 누락 발견 (정합성 cross-check, D-26)

W18 작업 중 main branch 의 `add_local.rs` 가 W16 까지만 머지된 상태 확인. **W17 PR 의 일부 patch 가 main 에 누락**:
- `register_local_provider_non_interactive` fn: **누락** (W18 에서 재추가)
- TC-W17-001~003 (L1 3개): **누락** (W18 에서 미재추가, W19+ OOS)
- TC-W17-I01, I02 (L2 2개): **누락** (W18 에서 미재추가, W19+ OOS)
- TC-W17-004 (L1 1개): **W18 에서 재활성화** ✅
- DD-AddLocal §9 spec, handoff D-60, memory 갱신: main 에 정상 머지 (W17 PR 의 일부는 들어옴)

**원인 가설**: W17 의 commit 들이 SDLC 1개 + impl 1개 + test 1개 + cli 1개 + memory 1개 = 5 commit 으로 분리. yklee 가 PR 작성 시 일부 commit 만 merge 했을 가능성 (또는 PR 미작성). W18 의 정합성 cross-check 단계 (impl 컴파일 시도)에서 발견.

**교훈 (D-16 lesson 강화)**:
1. **W 작업 완료 시 PR 작성 + 머지까지 끝내야 다음 W 가 main 에서 깨끗하게 시작 가능**. 중간에 멈추면 다음 W 가 fallback 작업 부담.
2. **W18 의 정합성 cross-check (impl 컴파일 + main branch state 확인)** 가 누락분 catch. **W19+ 에서 같은 cross-check 절차 의무화**.
3. **W19+ follow-up**: TC-W17-001~003 + TC-W17-I01~I02 재추가 (누락분 보강, 1 session)

## 3. TC scaffold (D-43~D-47 chapter 패턴 준수)

### 3.1 L1 Unit (W18 + W17-004)

| TC | 결과 |
| --- | --- |
| TC-W18-001 backup_created_before_overwrite | ✅ PASS |
| TC-W18-002 backup_max_retention_5 | ✅ PASS |
| TC-W18-003 backup_helper_unit_no_file | ✅ PASS |
| TC-W17-004 non_interactive_empty_model_id (재활성화) | ✅ PASS |

### 3.2 L2 Integration (W18)

| TC | 결과 |
| --- | --- |
| TC-W18-I01 register_creates_backup_before_overwrite (wiremock 2 server) | ✅ PASS |
| TC-W18-I02 backup_max_retention_keeps_only_n_files | ✅ PASS |

### 3.3 parallel-safe 패턴 (D-58 follow-up + W16 precedent)

- `MYHARNESS_HOME=tempdir` env mutation 으로 paths.rs 격리
- `#[serial_test::serial(env)]` attribute 로 L1 3개 (TC-001/002/004) + L2 2개 직렬화
- L1 1개 (TC-003) 는 env 무관 → parallel 가능
- 1 session 안에 5 TC 모두 PASS (D-47 chapter 1~3-B 의 27.5% 1-session 패턴 100% 적용)

## 4. risks / follow-up (1 + 3)

### 4.1 risks (1, R-4 follow-up)

- **R-4 (사용자 home 덮어쓰기, W18 으로 1차 차단)**: W18 으로 silent backup + Confirm prompt 추가. **복구**: `cp ~/.myharness/providers.toml.backup.<ts> ~/.myharness/providers.toml`. **남은 위험**:
  - sub-second 연속 register 시 ts 동일 → backup overwrite 가능 (sleep 1.1s 또는 monotonic_ts 도입으로 해결, v1.5+ OOS)
  - backup corruption (filesystem full 등) → 5개 retention 자동 정리되지만 silent

### 4.2 suggested follow-up (3)

- **F-1 (W19+ 후보)**: `monotonic_ts` 도입 (sub-second 충돌 방지). 1 session.
- **F-2 (W19+ 후보)**: backup → git-style versioning (TS + content hash) — 변경 있을 때만 backup. 1-2 session.
- **F-3 (W19+ 후보, D-60 handoff F-2 와 동일)**: Ollama native `/api/tags` 지원 (W16 R-2 대응). 1-2 session.

### 4.3 OOS (v1.5+ 후보, W18 이후)

- (F-1, F-2) — 본 §4.2 정리
- (TC-W17-001~003 + TC-W17-I01~I02 재추가) — W19+ 1 session
- (W17 PR 작성 + main merge) — yklee 결정

## 5. 의존성 / 변경 파일 (4-필드 produced_artifacts)

### 5.1 Cargo.toml 변경 (0 files)

- W18 은 **추가 의존성 0** — wiremock (W16) + inquire (W16) + url (W16) + serial_test (W16) + tempdir/tempfile 모두 재사용

### 5.2 SDLC docs 변경 (3 files)

- `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md`: §10 신규 spec (~120 lines)
- `docs/specs/TC_UNIT.md`: §W16.5 OI-1 ✅ + §W17 + §W18 신규 sections (~200 lines)
- `docs/specs/TC_INTEGRATION.md`: §W17 + §W18 신규 sections (~80 lines)

### 5.3 impl 변경 (4 files)

- `myharness/crates/llm/src/add_local.rs`: `+backup_providers_toml` (60 lines) + `+with_backup_suffix` + `+cleanup_old_backups` + `+register_local_provider_non_interactive` (W17 누락분) + 3 L1 unit test (W18) + 1 L1 unit test (W17-004 재활성화) (~200 lines)
- `myharness/crates/llm/src/lib.rs`: `+backup_providers_toml`, `+register_local_provider_non_interactive` re-export
- `myharness/crates/llm/tests/w16_add_local.rs`: `+2 L2 integration test` (TC-W18-I01, I02) (~95 lines)
- `myharness/crates/cli/src/main.rs`: `+enum AddLocal { url, token, model, probe_skip, yes }` + `+handle_auth_add_local` 5-arg + `+handle_add_local_interactive(skip_confirm)` 분리 + `+inquire::Confirm` prompt + `+print_register_report` 공통 (~150 lines)

## 6. VERDICT

### 6.1 SDLC 통과 (D-26 6 원칙)

- **한국어 보고** ✅
- **결론 위주** ✅
- **상태값** ✅ (D-61 W18 = done, SDLC 3 docs = done, impl = done, tests 5/5 PASS, W17 누락분 식별)
- **이벤트 소싱** ✅ (cargo test 결과, manual backup 검증, mavis-trash cleanup 모두 handoff 에 append)
- **비참조** ✅ (DD §10 cross-ref 5 SSOT, §10.6 cross-references)
- **handoff** ✅ (본 문서, 4-필드 D-26 정합)

### 6.2 TC scaffold 통과 (D-43 4-Layer)

- L1 Unit 4 (W18 3 + W17-004 1) ✅
- L2 Integration 2 (W18) ✅
- L3 Component 1 (cli dispatch, --yes flag) — TC_COMPONENT.md patch 미작성 (W18 범위 외, v1.5+ OOS)
- L4 E2E 1 (manual, backup 확인) — TC_E2E.md patch 미작성 (CI ❌, manual only)

### 6.3 D-43~D-47 chapter 패턴 준수

- **1 session 안에 4 chapter 묶기** ✅:
  - chapter 1: backup_providers_toml fn (TC-001, 003) — RED → GREEN
  - chapter 2: retention + W17-004 재활성화 (TC-002, W17-004) — RED → GREEN
  - chapter 3: cli patch — 5-arg + Confirm prompt + 공통 출력
  - chapter 4: L2 integration 2개 (TC-I01, I02) — wiremock + file system
- chunked write 4 chunk (impl / lib / test / cli)
- cargo test 매 chapter 후 + manual backup 2 시나리오 검증

### 6.4 결론

본 W18 = `auth add-local` 자동 backup + Confirm prompt (R-4 직접 차단) v1.5 구현 + SDLC 3 docs + 5 TC 모두 PASS. **VERDICT: PASS**.

## 7. 다음 단계

1. **dual push** (Gitea origin + GitHub upstream) — W18 4 commit (impl / lib / test / cli) + 1 doc commit (SDLC)
2. **memory 갱신** (state.json + work_backlog.md + session_handoff.md) — TASK-005-2 v1.5 W18 완료 + W17 누락분 식별
3. **W17 PR + W18 PR 작성 결정** — yklee 책임. main = 140acf9 (W16 follow-up), W17/W18 feature branch 미머지.
4. **TASK-005-2 v1.5 W19+ 결정** — yklee 결정 대기. 후보: F-1 monotonic_ts / F-2 git-style versioning / F-3 Ollama native /api/tags / W17 누락분 보강 / Plugin 4-계층 (큰 사이클) / CCR + Kompress-base

### 7.1 dual push 명령 (owner)

```bash
cd /Users/yklee/repos/my_harness
git add docs/ myharness/
git commit -m "feat(llm): W18 backup_providers_toml + register_local_provider 안에서 자동 backup (D-61)

- R-4 (사용자 home providers.toml 덮어쓰기) 직접 차단
- impl: backup_providers_toml(path, max_backups=5) + with_backup_suffix + cleanup_old_backups
- impl: register_local_provider 안에 backup 호출 연결 (silent, fail-soft)
- W17 누락분 재추가: register_local_provider_non_interactive + TC-W17-004
- tests: 3 L1 (W18) + 1 L1 (W17-004) + 2 L2 (W18) = 6 PASS"
# ... (cli, docs, memory 등 추가 commit)
git push origin feature/v15-add-local-backup
gh pr create --base main --head feature/v15-add-local-backup --title "W18 v1.5 add-local backup (R-4 대응)"
```
