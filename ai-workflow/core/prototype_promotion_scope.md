# Prototype Promotion Scope

- 문서 목적: 현재 skill/MCP 프로토타입을 reusable package 또는 MCP server 로 승격할 때 어떤 단위를 먼저 올릴지 범위와 기준을 정의한다.
- 범위: 승격 판단 기준, 우선 승격 후보, 보류 후보, package/server 분리 원칙, 단계별 권장 순서
- 대상 독자: 저장소 관리자, AI workflow 설계자, 구현자, 배포 담당자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `./workflow_kit_roadmap.md`, `./workflow_release_spec.md`, `./workflow_mcp_candidate_catalog.md`, `./read_only_mcp_transport_promotion.md`, `../skills/README.md`, `../mcp_servers/README.md`, `../workflow_kit/README.md`

## 1. 목적

현재 저장소에는 문서, bootstrap, skill 프로토타입, MCP 프로토타입이 함께 들어 있다. 다음 단계에서는 이 중 일부를 더 강한 재사용 단위로 승격해야 한다.

이 문서는 아래 질문에 답하기 위한 기준 문서다.

- 무엇을 먼저 reusable package 로 빼는가
- 무엇을 MCP server 로 묶는가
- 아직 문서/스크립트 수준에 남겨야 하는 것은 무엇인가

## 2. 승격 판단 기준

### 2.1 reusable package 에 적합한 경우

- 파일 시스템 읽기/쓰기와 JSON 생성 로직이 핵심인 경우
- CLI, bootstrap, CI, 로컬 스크립트에서 공통으로 재사용할 가능성이 큰 경우
- 하네스와 무관하게 Python 함수 호출로도 가치가 있는 경우

예:

- project profile 파서
- 문서 구조 분류기
- validation/code-index 추천 로직
- bootstrap 생성/manifest 조립 로직

### 2.2 MCP server 에 적합한 경우

- agent 가 도구 호출처럼 짧은 입력/출력으로 반복 사용하기 좋은 경우
- 읽기 전용 검사, 후보 추천, 초안 생성처럼 명확한 tool 인터페이스가 있는 경우
- 하네스나 UI 에서 “도구” 단위로 바로 연결하는 가치가 큰 경우

예:

- latest backlog 찾기
- metadata/link 검사
- quickstart stale 링크 점검
- impacted docs 추천

### 2.3 아직 문서/스크립트에 남겨야 하는 경우

- 입력 계약이 아직 유동적인 경우
- 여러 프로토타입을 묶는 orchestration 성격이 강한 경우
- 실제 운영 피드백이 더 필요한 경우

예:

- end-to-end demo runner
- 기존 프로젝트 onboarding runner
- 하네스별 apply guide

## 3. 현재 자산 분류

### 3.1 우선 reusable package 후보

- bootstrap 공통 로직
- project profile / handoff / backlog 파서
- 변경 파일 분류 로직
- validation-plan 추천 로직
- code-index-update 추천 로직

이 로직은 CLI 에서도 쓰이고, 나중에 MCP server 내부 구현에서도 재사용될 수 있으므로 먼저 라이브러리화할 가치가 크다.

현재 상태:

- `workflow_kit/common/paths.py`
- `workflow_kit/common/markdown.py`
- `workflow_kit/common/docs.py`
- `workflow_kit/common/project_docs.py`
- `workflow_kit/common/change_types.py`
- `workflow_kit/common/normalize.py`
- `workflow_kit/common/reconcile.py`
- `workflow_kit/common/planning.py`
- `workflow_kit/common/doc_sync.py`
- `workflow_kit/common/session_outputs.py`
- `workflow_kit/common/runner.py`

즉, 읽기 전용 MCP 일부에서 사용하는 공통 경로/Markdown/메타데이터 유틸뿐 아니라 skill 파서, 정규화, 요약 생성, orchestration runner helper 까지 package 루트로 옮겨지기 시작했다.
다만 다음 확장부터는 외부 리뷰 의견을 반영해 parser / classifier / recommendation 계열을 우선하고, orchestration helper 추가 추출은 실패 계약과 실제 적용 검증 이후로 미루는 편이 안전하다.

### 3.2 우선 MCP server 후보

- `latest_backlog`
- `check_doc_metadata`
- `check_doc_links`
- `suggest_impacted_docs`
- `check_quickstart_stale_links`

이 다섯 개는 읽기 전용이거나 경고/후보 출력 중심이라 MCP tool 인터페이스로 묶기 쉽다.

### 3.3 2차 MCP server 후보

- `create_backlog_entry`
- `create_session_handoff_draft`
- `create_environment_record_stub`

이 그룹은 초안 생성/쓰기 성격이 조금 더 강하므로, 읽기 전용 MCP 들보다 한 단계 뒤에 올리는 편이 안전하다.

### 3.4 문서/runner 로 유지할 후보

- `run_demo_workflow.py`
- `run_existing_project_onboarding.py`
- harness overlay / bootstrap output sample 문서

이들은 여러 프로토타입을 연결하고 사용자 온보딩을 돕는 역할이므로, 당장은 package 보다는 orchestration 레이어로 두는 것이 낫다.

## 4. 권장 승격 순서

### 1단계: 공통 라이브러리 추출

권장 결과:

- `workflow_kit/` 또는 유사한 Python package 루트 생성
- profile/handoff/backlog 파서 모듈
- 링크 검사, metadata 검사, changed-file 분류 유틸 분리
- 기존 스크립트가 라이브러리 함수를 호출하도록 재구성

완료 기준:

- 기존 CLI 스크립트가 로직 복사 없이 공통 모듈을 호출한다.
- 테스트가 스크립트 레벨과 함수 레벨로 분리된다.

### 2단계: 읽기 전용 MCP 묶음 server 화

권장 결과:

- 우선 MCP 5종을 하나의 MCP server 엔트리포인트로 묶는다.
- 각 tool 은 현재 JSON 출력 계약을 유지한다.
- 로컬 실행과 원격 MCP 연결 모두를 고려한 엔트리 구조를 만든다.

완료 기준:

- `latest_backlog`, `check_doc_metadata`, `check_doc_links`, `suggest_impacted_docs`, `check_quickstart_stale_links` 가 server tool 로 호출 가능하다.
- 기존 CLI 샘플과 MCP 응답 계약이 크게 어긋나지 않는다.

### 3단계: 초안 생성 계열 MCP 확장

권장 결과:

- `create_backlog_entry` 와 후속 draft MCP 들을 server 에 추가한다.
- 읽기 전용과 쓰기 가능 초안 생성을 permission 관점에서 분리한다.

완료 기준:

- 초안 생성 tool 이 읽기 전용 MCP 들과 섞여도 안전 경계가 명확하다.

### 4단계: orchestration 계층 재정렬

권장 결과:

- onboarding runner 와 demo runner 가 공통 package 와 MCP tool 을 조합하는 상위 레이어로 정리된다.
- 하네스별 agent/skill 은 package 와 MCP server 를 조합해 읽도록 단순화된다.

완료 기준:

- CLI/demo/onboarding/harness 가 같은 공통 구현을 공유한다.

## 5. 보류 기준

아래 조건 중 하나라도 강하면 즉시 승격하지 않는다.

- 출력 계약이 아직 자주 바뀌는 경우
- 실제 적용 예시가 없어 인터페이스 안정성이 낮은 경우
- 쓰기 권한 경계가 아직 문서로 충분히 정의되지 않은 경우
- 하네스별 요구 차이가 커서 공통 tool 로 묶기 이른 경우

## 6. 현재 시점 권장 결론

현재 시점에서는 아래 순서를 권장한다.

1. 공통 파서/분류/추천 로직을 reusable package 로 추출한다.
2. `check_quickstart_stale_links` 를 포함한 읽기 전용 MCP 5종을 MCP server 1호 범위로 묶는다.
3. 초안 생성 MCP 는 permission 설계를 더 명확히 한 뒤 2차로 올린다.
4. onboarding/demo runner 는 library + MCP 조합의 상위 orchestration 레이어로 남긴다.

즉, 지금은 “모든 것을 server 화”하기보다 “공통 라이브러리화 -> 읽기 전용 MCP server 화 -> 쓰기 성격 확장” 순서가 가장 안전하다. 추가로 library 확장도 한 번에 넓히기보다, 변경 파일 분류/문서 파서/추천 로직처럼 계약이 비교적 안정적인 영역부터 좁게 가져가는 것이 현재 단계에 더 맞다.

현재 착수 상태:

- 읽기 전용 MCP 1차 묶음의 registry 와 draft entrypoint 가 `workflow_kit/server/read_only_registry.py`, `workflow_kit/server/read_only_entrypoint.py` 로 시작됐다.
- 현재 단계는 direct-call adapter, JSON-RPC draft bridge, fixture export 수준의 draft 구조이며, 정식 MCP SDK transport 승격 기준은 [./read_only_mcp_transport_promotion.md](./read_only_mcp_transport_promotion.md) 에서 분리한다.

## 7. 다음에 읽을 문서

- 상위 로드맵: [./workflow_kit_roadmap.md](./workflow_kit_roadmap.md)
- 릴리스 규격: [./workflow_release_spec.md](./workflow_release_spec.md)
- MCP 후보 카탈로그: [./workflow_mcp_candidate_catalog.md](./workflow_mcp_candidate_catalog.md)
- read-only transport 승격 기준: [./read_only_mcp_transport_promotion.md](./read_only_mcp_transport_promotion.md)
- package 루트: [../workflow_kit/README.md](../workflow_kit/README.md)
- mcp 허브: [../mcp_servers/README.md](../mcp_servers/README.md)
