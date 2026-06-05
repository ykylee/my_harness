# Examples

- 문서 목적: 표준 워크플로우를 실제 프로젝트에 적용할 때 참고할 샘플 프로파일과 예시 구조를 배치할 위치를 안내한다.
- 범위: 향후 추가할 예시 문서와 샘플 프로젝트 프로파일
- 대상 독자: 개발자, 운영자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `../templates/project_workflow_profile_template.md`

## 현재 상태

- 샘플 프로젝트 프로파일과 운영 문서 세트를 `acme_delivery_platform/` 아래에 추가했다.
- 첫 예시는 특정 내부 프로젝트를 직접 노출하지 않고도 문서 구조와 운영 흐름을 이해할 수 있게 단순화된 가상 프로젝트 형태로 제공한다.
- 두 번째 예시는 문서와 평가 기준 정합성이 중요한 리서치 운영형 프로젝트를 가정해, 같은 표준 문서 세트가 다른 작업 성격에도 적용되는지 보여준다.

## 포함된 예시

- [acme_delivery_platform/PROJECT_PROFILE.md](./acme_delivery_platform/PROJECT_PROFILE.md)
- [acme_delivery_platform/session_handoff.md](./acme_delivery_platform/session_handoff.md)
- [acme_delivery_platform/work_backlog.md](./acme_delivery_platform/work_backlog.md)
- [acme_delivery_platform/backlog/2026-04-18.md](./acme_delivery_platform/backlog/2026-04-18.md)
- [research_eval_hub/PROJECT_PROFILE.md](./research_eval_hub/PROJECT_PROFILE.md)
- [research_eval_hub/session_handoff.md](./research_eval_hub/session_handoff.md)
- [research_eval_hub/work_backlog.md](./research_eval_hub/work_backlog.md)
- [research_eval_hub/backlog/2026-04-19.md](./research_eval_hub/backlog/2026-04-19.md)
- [end_to_end_skill_demo.md](./end_to_end_skill_demo.md)
- [end_to_end_mcp_demo.md](./end_to_end_mcp_demo.md)
- [bootstrap_output_samples.md](./bootstrap_output_samples.md)
- [pilot_adoption_existing_repo_example.md](./pilot_adoption_existing_repo_example.md)
- [pilot_adoption_open_git_client_example.md](./pilot_adoption_open_git_client_example.md)
- [pilot_execution_plan_existing_repo.md](./pilot_execution_plan_existing_repo.md)
- [pilot_candidate_shortlist_example.md](./pilot_candidate_shortlist_example.md)
- [pilot_candidate_existing_service_repo_example.md](./pilot_candidate_existing_service_repo_example.md)
- [pilot_candidate_open_git_client_assessment.md](./pilot_candidate_open_git_client_assessment.md)

## 예시 비교 포인트

- `acme_delivery_platform/` 는 운영 절차, 장애 대응, staging 검증처럼 서비스 운영 문서 흐름이 중심인 프로젝트 예시다.
- `research_eval_hub/` 는 평가 데이터셋, 실험 결과, 배포 가능한 문서 패키지 정합성처럼 문서와 연구 산출물 관리가 중심인 프로젝트 예시다.
- 두 예시는 같은 표준 handoff, backlog, profile 구성을 유지하면서도 검증 포인트와 예외 규칙만 프로젝트 성격에 맞게 바뀌는 방식을 보여준다.

## 예시 사용 방법

- 먼저 프로젝트 프로파일을 읽고 저장소별 규칙이 어떻게 채워지는지 본다.
- 그다음 handoff 와 backlog 를 읽어 세션 시작 흐름이 실제로 어떻게 이어지는지 확인한다.
- 프로토타입 skill 이 실제로 어떻게 이어지는지 보려면 `end_to_end_skill_demo.md` 를 읽고 명령을 순서대로 실행한다.
- 메인 오케스트레이터가 `doc-worker`, `code-worker`, `validation-worker` 를 어떻게 분배하면 좋은지 보려면 `end_to_end_skill_demo.md` 의 운영 예시 섹션을 먼저 읽는다.
- 프로토타입 MCP 가 실제로 어떻게 이어지는지 보려면 `end_to_end_mcp_demo.md` 를 읽고 명령을 순서대로 실행한다.
- bootstrap 생성물이 어떤 문구를 기본으로 포함하는지 보려면 `bootstrap_output_samples.md` 를 먼저 확인한다.
- 실제 파일럿 적용 기록을 어떤 형식으로 남길지 보려면 `pilot_adoption_existing_repo_example.md` 를 먼저 확인한다.
- 실제 로컬 저장소에 적용한 첫 파일럿 기록을 보려면 `pilot_adoption_open_git_client_example.md` 를 먼저 확인한다.
- 실제 파일럿을 어떤 순서로 진행할지 보려면 `pilot_execution_plan_existing_repo.md` 를 먼저 확인한다.
- 파일럿 후보를 어떤 기준으로 1~2개로 좁힐지 보려면 `pilot_candidate_shortlist_example.md` 를 먼저 확인한다.
- 1순위 후보 체크리스트를 실제로 어떻게 채우는지 보려면 `pilot_candidate_existing_service_repo_example.md` 를 먼저 확인한다.
- 실제 로컬 저장소 하나를 후보로 평가한 예시는 `pilot_candidate_open_git_client_assessment.md` 를 먼저 확인한다.
- 이후 자신의 프로젝트에 맞게 문서 경로, 명령, 검증 포인트만 바꿔서 복사 적용한다.
