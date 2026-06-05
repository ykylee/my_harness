# Output Samples

- 문서 목적: 실행형 skill 과 MCP 프로토타입의 대표 JSON 출력 예시를 한곳에서 참조할 수 있게 정리한다.
- 범위: 현재 추가된 skill/MCP 출력 샘플 파일과 사용 용도
- 대상 독자: AI workflow 설계자, skill 구현자, 테스트 작성자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-22
- 관련 문서: `../../core/output_schema_guide.md`, `../end_to_end_skill_demo.md`

## 현재 포함된 샘플

- [session_start.acme_delivery_platform.json](./session_start.acme_delivery_platform.json)
- [session_start.error.missing_document.json](./session_start.error.missing_document.json)
- [backlog_update.acme_delivery_platform.json](./backlog_update.acme_delivery_platform.json)
- [backlog_update.error.missing_document.json](./backlog_update.error.missing_document.json)
- [doc_sync.acme_delivery_platform.json](./doc_sync.acme_delivery_platform.json)
- [doc_sync.error.missing_change_input.json](./doc_sync.error.missing_change_input.json)
- [merge_doc_reconcile.acme_delivery_platform.json](./merge_doc_reconcile.acme_delivery_platform.json)
- [merge_doc_reconcile.error.missing_document.json](./merge_doc_reconcile.error.missing_document.json)
- [validation_plan.acme_delivery_platform.json](./validation_plan.acme_delivery_platform.json)
- [validation_plan.docs_primary.acme_delivery_platform.json](./validation_plan.docs_primary.acme_delivery_platform.json)
- [validation_plan.error.missing_change_input.json](./validation_plan.error.missing_change_input.json)
- [code_index_update.research_eval_hub.json](./code_index_update.research_eval_hub.json)
- [code_index_update.error.missing_change_input.json](./code_index_update.error.missing_change_input.json)
- [demo_workflow.acme_delivery_platform.json](./demo_workflow.acme_delivery_platform.json)
- [demo_workflow.error.missing_document.json](./demo_workflow.error.missing_document.json)
- [existing_project_onboarding.acme_delivery_platform.json](./existing_project_onboarding.acme_delivery_platform.json)
- [existing_project_onboarding.with_assessment.sample.json](./existing_project_onboarding.with_assessment.sample.json)
- [existing_project_onboarding.error.missing_document.json](./existing_project_onboarding.error.missing_document.json)
- [latest_backlog.acme_delivery_platform.json](./latest_backlog.acme_delivery_platform.json)
- [check_doc_metadata.examples.json](./check_doc_metadata.examples.json)
- [check_doc_links.examples.json](./check_doc_links.examples.json)
- [check_quickstart_stale_links.sample.json](./check_quickstart_stale_links.sample.json)
- [create_backlog_entry.sample.json](./create_backlog_entry.sample.json)
- [suggest_impacted_docs.sample.json](./suggest_impacted_docs.sample.json)

## 사용 목적

- 출력 스키마 문서에서 실제 필드 구성을 예시로 참조할 때 사용한다.
- 테스트 작성 시 기대 필드 구조를 빠르게 확인할 때 사용한다.
- 이후 공통 schema 파일 또는 MCP server 응답 규격으로 승격할 때 초안 샘플로 재사용한다.

## 주의 사항

- 이 디렉터리의 JSON 파일은 대표 예시이며, 모든 프로젝트에서 값이 동일하다는 뜻은 아니다.
- `demo_workflow.*`, `existing_project_onboarding.*` 샘플은 runner 전체 출력을 보여주기 위해 개별 step 요약이 아니라 중첩 payload 를 포함한 richer sample 형태를 사용한다.
- `existing_project_onboarding.with_assessment.sample.json` 은 `repository_assessment.summary` 와 `onboarding_summary.inferred_commands` 가 실제로 채워진 하네스 소비 예시를 보여준다.
- `validation_plan.docs_primary.acme_delivery_platform.json` 은 docs-primary 변경에서도 기본 quick test 를 제안하는 최신 정책 예시를 보여준다.
- 경고 문구나 후보 경로는 프로젝트 프로파일과 변경 파일 입력에 따라 달라질 수 있다.
- 실패 샘플은 대표적인 오류 분류 예시이며, `source_context` 세부 값은 실행 인자에 따라 달라질 수 있다.
- 시간 정보가 들어가는 샘플은 예시 생성 시점의 값이 포함될 수 있다.
- `tests/check_output_samples.py` 는 이 README 목록과 실제 JSON 파일 집합이 서로 일치하는지도 함께 검사한다.

## 다음에 읽을 문서

- 출력 스키마 가이드: [../../core/output_schema_guide.md](../../core/output_schema_guide.md)
- end-to-end skill demo: [../end_to_end_skill_demo.md](../end_to_end_skill_demo.md)
