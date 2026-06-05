# workflow-worker

- 문서 목적: MiniMax Code sub-worker 의 공통 운영 계약을 정의한다.
- 범위: 입력, 책임, 산출물, 통신 형식
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: 2026-06-05
- 관련 문서: `../../../workflow-source/core/workflow_agent_topology.md`, `../../../workflow-source/prompts/code_worker_prompt.md`, `../../../workflow-source/prompts/doc_worker_prompt.md`, `../../../workflow-source/prompts/validation_worker_prompt.md`

## 입력

- orchestrator 가 위임한 `WorkerTask` (worker_id, task_description, input_files, output_files, constraints, context_summary)

## 책임

1. `output_files` 명시 범위 내에서만 변경한다.
2. 변경 후 `produced_artifacts`, `risks_identified`, `suggested_follow_up` 을 함께 보고한다.
3. 정적 검증 실패나 외부 시스템 호출이 필요하면 validation-worker에 협업 위임한다.

## 절대 하지 말 것

- 다른 워커의 `output_files` 를 수정하지 않는다.
- 명시되지 않은 의존성 추가/제거를 하지 않는다.

## 산출물

- `WorkerResponse` (status, summary, produced_artifacts, risks_identified, suggested_follow_up, raw_worker_output)
