# End-to-End Skill Demo

- 문서 목적: 핵심 skill 과 후속 검증 planning skill 이 예시 프로젝트 문서를 기준으로 어떻게 이어지는지 end-to-end 데모 흐름으로 설명한다.
- 범위: `session-start`, `backlog-update`, `doc-sync`, `validation-plan`, `code-index-update`, `merge-doc-reconcile` 실행 순서와 기대 결과
- 대상 독자: 개발자, 운영자, AI agent 설계자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `./README.md`, `./acme_delivery_platform/PROJECT_PROFILE.md`, `./research_eval_hub/PROJECT_PROFILE.md`, `../skills/session-start/SKILL.md`, `../skills/backlog-update/SKILL.md`, `../skills/doc-sync/SKILL.md`, `../skills/merge-doc-reconcile/SKILL.md`

## 1. 목적

이 문서는 예시 프로젝트를 기준으로 1차 핵심 skill 4종이 실제로 어떤 순서로 이어지는지 보여준다.

현재 프로토타입은 기본적으로 JSON 초안과 추천을 반환하며, `--apply` 또는 `--scaffold` 옵션을 주면 쓰기/생성 작업을 수행한다. 쓰기 기능은 Beta v1 이후에 정식 도입됐다.

통합 실행 스크립트도 제공한다:

- [../scripts/run_demo_workflow.py](../scripts/run_demo_workflow.py)

현재 권장 운영 패턴은 아래와 같다.

- 메인 오케스트레이터는 세션 기준선 복원, 우선순위 조정, 결과 통합, 사용자 보고를 담당한다.
- 메인 오케스트레이터는 직접 도구 호출보다 worker 위임과 결과 통합을 기본값으로 둔다.
- `doc-worker` 는 문서 읽기/비교/초안 정리에 집중한다.
- `code-worker` 는 범위가 명확한 코드/설정 수정에 집중한다.
- `validation-worker` 는 검증 실행, 로그 확인, 증빙 수집에 집중한다.
- worker 는 bounded scope 안에서 실제 실행을 맡고, low-risk 작업에서는 `ask` 를 최소화하는 편을 기본 운영값으로 본다.
- 모델을 `main` + `small` 로 운영한다면 기본값은 `main orchestrator + small workers` 를 권장한다.

## 2. 준비 문서

데모에서 기본으로 사용하는 예시 문서:

- [acme_delivery_platform/PROJECT_PROFILE.md](./acme_delivery_platform/PROJECT_PROFILE.md)
- [acme_delivery_platform/session_handoff.md](./acme_delivery_platform/session_handoff.md)
- [acme_delivery_platform/work_backlog.md](./acme_delivery_platform/work_backlog.md)
- [acme_delivery_platform/backlog/2026-04-18.md](./acme_delivery_platform/backlog/2026-04-18.md)

다른 성격의 샘플로도 같은 흐름을 검증할 수 있다:

- [research_eval_hub/PROJECT_PROFILE.md](./research_eval_hub/PROJECT_PROFILE.md)
- [research_eval_hub/session_handoff.md](./research_eval_hub/session_handoff.md)
- [research_eval_hub/work_backlog.md](./research_eval_hub/work_backlog.md)
- [research_eval_hub/backlog/2026-04-19.md](./research_eval_hub/backlog/2026-04-19.md)

데모에서 사용하는 프로토타입:

- [../skills/session-start/scripts/run_session_start.py](../skills/session-start/scripts/run_session_start.py)
- [../skills/backlog-update/scripts/run_backlog_update.py](../skills/backlog-update/scripts/run_backlog_update.py)
- [../skills/doc-sync/scripts/run_doc_sync.py](../skills/doc-sync/scripts/run_doc_sync.py)
- [../skills/validation-plan/scripts/run_validation_plan.py](../skills/validation-plan/scripts/run_validation_plan.py)
- [../skills/code-index-update/scripts/run_code_index_update.py](../skills/code-index-update/scripts/run_code_index_update.py)
- [../skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py](../skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py)

## 3. 권장 운영 예시: Orchestrator + Workers

실제 운영에서는 메인 오케스트레이터가 아래처럼 worker 를 호출하는 흐름을 권장한다.

예시 시나리오:

- 사용자 요청: "delivery sync 재시도 로직 수정 후 문서와 backlog 를 같이 정리해줘"

권장 분배:

1. 메인 오케스트레이터 (`main`)
2. `doc-worker` (`small`)
3. `code-worker` (`small`)
4. `validation-worker` (`small`)

권장 호출 흐름:

1. 메인 오케스트레이터가 `session-start` 로 현재 기준선을 복원한다.
2. `doc-worker` 에게 handoff, latest backlog, 관련 runbook 비교를 맡겨 영향 문서와 상태 불일치를 요약받는다.
3. `code-worker` 에게 실제 코드 수정이나 설정 수정이 필요한 범위만 넘긴다.
4. `validation-worker` 에게 quick test, smoke check, 로그 확인, 증빙 메모 수집을 맡긴다.
5. 메인 오케스트레이터가 `doc-sync`, `validation-plan`, `code-index-update` 결과와 worker 요약을 합쳐 backlog/handoff 반영 방향을 정한다.
6. 병합 직후라면 마지막에 `merge-doc-reconcile` 로 재확정 포인트를 정리한다.

운영 메모:

- 문서 대량 읽기와 코드 수정을 한 에이전트에 몰지 않으면 컨텍스트가 훨씬 덜 오염된다.
- worker 는 원문 dump 대신 핵심 사실, draft, 리스크, follow-up 만 반환하는 편이 좋다.
- 메인 오케스트레이터는 직접 도구 예외를 늘리기보다 worker 호출을 기본 경로로 유지하는 편이 재현성과 권한 경계 관리에 유리하다.
- 난도가 높아지면 특정 worker 만 일시적으로 `main` 모델로 승격하고, 기본값은 `small` 로 유지하는 편이 효율적이다.

## 4. Step 1: Session Start

세션 시작 시점에 현재 기준선을 복원한다.

```bash
python3 skills/session-start/scripts/run_session_start.py \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md
```

기대 결과:

- 현재 기준선 요약 출력
- 진행 중 작업과 차단 작업 목록 출력
- 최신 backlog 경로 식별
- 다음에 읽을 문서 경로 추천
- handoff 와 backlog 불일치 시 경고 출력

## 5. Step 2: Backlog Update

세션에서 수행할 작업을 날짜별 backlog 초안으로 만든다.

기존 항목 갱신 예시:

```bash
python3 skills/backlog-update/scripts/run_backlog_update.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --daily-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --mode update \
  --task-id TASK-021 \
  --task-name "배송 상태 동기화 실패 대응 절차 문서 정리" \
  --task-brief "runbook 및 handoff 반영 상태를 점검했다." \
  --status in_progress
```

신규 항목 생성 예시:

```bash
python3 skills/backlog-update/scripts/run_backlog_update.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --target-date 2026-04-19 \
  --mode create \
  --task-name "운영 허브 링크 무결성 재점검" \
  --task-brief "새 runbook 링크 반영 여부를 확인한다."
```

기대 결과:

- 신규/갱신 여부에 맞는 `operation_type` 출력
- 날짜별 backlog 대상 경로 계산
- 작업 항목 `draft_entry` JSON 초안 생성
- `done` 상태 오판 방지 경고 출력
- 사람이 직접 채워야 할 `fields_requiring_confirmation` 분리

## 6. Step 3: Doc Sync

변경 파일을 기준으로 어떤 문서를 함께 봐야 하는지 추천한다.

코드 변경 예시:

```bash
python3 skills/doc-sync/scripts/run_doc_sync.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --latest-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --changed-file app/jobs/delivery_sync.py \
  --change-summary "delivery sync 재시도 로직 변경"
```

(여기에 `--apply` 옵션을 추가하면, 영향 문서 중 백로그/상태 문서에 변경 감지 메모를 남긴다.)

문서 변경 예시:

```bash
python3 skills/doc-sync/scripts/run_doc_sync.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --latest-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --changed-file docs/operations/runbooks/delivery-sync.md
```

기대 결과:

- 영향 문서 후보 추천
- 허브 또는 인덱스 문서 재확인 후보 추천
- stale 가능성 경고 출력
- 후속 검토 순서와 행동 제안 출력

## 7. Step 4: Validation Plan

문서와 코드 변경을 기준으로 이번 세션의 검증 수준과 증빙 기대값을 구조화한다.

```bash
python3 skills/validation-plan/scripts/run_validation_plan.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --latest-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --changed-file app/jobs/delivery_sync.py \
  --changed-file docs/operations/runbooks/delivery-sync.md \
  --change-summary "delivery sync 재시도 로직과 운영 runbook 동시 수정"
```

(여기에 `--scaffold` 옵션을 추가하면, 테스트 코드 뼈대 파일(`tests/repro_*.py`)을 자동 생성하고 handoff에 기록한다.)

기대 결과:

- 감지된 변경 유형 목록 출력
- 권장 검증 수준과 명령 목록 출력
- 문서화 체크 항목과 증빙 기대값 출력
- 미실행 항목을 backlog 또는 handoff 에 남길지 판단할 힌트 출력
- 환경 제약이나 승인 규칙 기반 경고 출력

## 8. Step 5: Code Index Update

변경된 문서나 코드가 README, 운영 허브, backlog index 같은 색인 문서를 stale 하게 만들었는지 추천한다.

```bash
python3 skills/code-index-update/scripts/run_code_index_update.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --changed-file app/jobs/delivery_sync.py \
  --changed-file docs/operations/runbooks/delivery-sync.md \
  --change-summary "delivery sync 재시도 로직과 운영 runbook 동시 수정"
```

(여기에 `--scaffold` 옵션을 추가하면, 테스트 코드 뼈대 파일(`tests/repro_*.py`)을 자동 생성하고 handoff에 기록한다.)

기대 결과:

- 색인 문서 갱신 후보와 우선순위 후보 출력
- 허브 stale 가능성 경고 출력
- 문서 구조 변경 가능성 신호 출력
- 색인 문서 후속 액션 추천 출력

## 9. Step 6: Merge Doc Reconcile

병합 이후 상태 문서와 허브 문서의 재확정 포인트를 정리한다.

```bash
python3 skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --latest-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --merge-result-summary "runbook 링크와 상태 문서가 함께 수정된 브랜치 병합 후 재정리" \
  --changed-file docs/operations/runbooks/delivery-sync.md \
  --changed-file app/jobs/delivery_sync.py
```

기대 결과:

- 병합 후 재확인 대상 문서 목록 출력
- handoff 와 backlog 간 상태 충돌 목록 출력
- 재확정 포인트와 재정리 메모 초안 출력
- 병합 후 검증 미완료 상태 경고 출력

## 10. 추천 읽기 순서

이 저장소를 처음 보는 사람에게는 아래 순서를 권장한다.

1. [../README.md](../README.md)
2. [./README.md](./README.md)
3. [acme_delivery_platform/PROJECT_PROFILE.md](./acme_delivery_platform/PROJECT_PROFILE.md)
4. [acme_delivery_platform/session_handoff.md](./acme_delivery_platform/session_handoff.md)
5. 이 문서의 6개 프로토타입 명령 실행

한 번에 흐름을 실행해보려면 아래 명령도 사용할 수 있다.

```bash
python3 scripts/run_demo_workflow.py
```

`--apply` 옵션을 주면 쓰기/생성(`--apply`, `--scaffold`) 흐름까지 함께 검증할 수 있다.

```bash
python3 scripts/run_demo_workflow.py --apply
```

다른 샘플 프로젝트로 돌려보려면 예제 이름만 바꾸면 된다.

```bash
python3 scripts/run_demo_workflow.py --example-project research_eval_hub
```

## 11. 현재 한계

- 예시 프로젝트 문서 구조는 단순화된 가상 사례다.
- 프로토타입 쓰기 기능(`--apply`)은 현재 백로그 갱신, 문서 인덱스 추천 및 state 동기화 위주로 제한되어 있으며, 코드 본문을 직접 리팩토링하지는 않는다.
- 허브 문서 후보는 실제 프로젝트 문서 구조에 따라 더 정교한 규칙이 필요할 수 있다.

## 다음에 읽을 문서

- examples 허브: [./README.md](./README.md)
- skills 허브: [../skills/README.md](../skills/README.md)
