# End-to-End MCP Demo

- 문서 목적: 우선순위 1 MCP 프로토타입 5종이 예시 프로젝트 문서를 기준으로 어떻게 이어지는지 end-to-end 데모 흐름으로 설명한다.
- 범위: `latest_backlog`, `check_doc_metadata`, `check_doc_links`, `create_backlog_entry`, `suggest_impacted_docs` 실행 순서와 기대 결과
- 대상 독자: 개발자, 운영자, AI agent 설계자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./README.md`, `./acme_delivery_platform/work_backlog.md`, `./research_eval_hub/work_backlog.md`, `../mcp_servers/README.md`, `../mcp_servers/prototype_layout.md`

## 1. 목적

이 문서는 예시 프로젝트를 기준으로 우선순위 1 MCP 프로토타입 5종이 실제로 어떤 순서로 이어지는지 보여준다.

현재 프로토타입은 모두 읽기 전용 또는 JSON 초안 생성 단계이며, 실제 MCP 서버 transport 는 포함하지 않는다. 대신 각 도구가 어떤 입력을 받고 어떤 구조화 결과를 내는지 빠르게 검증할 수 있다.

## 2. 준비 문서

데모에서 기본으로 사용하는 예시 문서:

- [acme_delivery_platform/PROJECT_PROFILE.md](./acme_delivery_platform/PROJECT_PROFILE.md)
- [acme_delivery_platform/work_backlog.md](./acme_delivery_platform/work_backlog.md)
- [acme_delivery_platform/backlog/2026-04-18.md](./acme_delivery_platform/backlog/2026-04-18.md)
- [acme_delivery_platform/session_handoff.md](./acme_delivery_platform/session_handoff.md)

같은 MCP 흐름을 다른 샘플에도 적용할 수 있다:

- [research_eval_hub/PROJECT_PROFILE.md](./research_eval_hub/PROJECT_PROFILE.md)
- [research_eval_hub/work_backlog.md](./research_eval_hub/work_backlog.md)
- [research_eval_hub/backlog/2026-04-19.md](./research_eval_hub/backlog/2026-04-19.md)
- [research_eval_hub/session_handoff.md](./research_eval_hub/session_handoff.md)

데모에서 사용하는 프로토타입:

- [../mcp_servers/latest-backlog/scripts/run_latest_backlog.py](../mcp_servers/latest-backlog/scripts/run_latest_backlog.py)
- [../mcp_servers/check-doc-metadata/scripts/run_check_doc_metadata.py](../mcp_servers/check-doc-metadata/scripts/run_check_doc_metadata.py)
- [../mcp_servers/check-doc-links/scripts/run_check_doc_links.py](../mcp_servers/check-doc-links/scripts/run_check_doc_links.py)
- [../mcp_servers/create-backlog-entry/scripts/run_create_backlog_entry.py](../mcp_servers/create-backlog-entry/scripts/run_create_backlog_entry.py)
- [../mcp_servers/suggest-impacted-docs/scripts/run_suggest_impacted_docs.py](../mcp_servers/suggest-impacted-docs/scripts/run_suggest_impacted_docs.py)

## 3. Step 1: Latest Backlog

backlog index 에서 최신 날짜 backlog 를 찾는다.

```bash
python3 mcp_servers/latest-backlog/scripts/run_latest_backlog.py \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md
```

기대 결과:

- `latest_backlog_path` 출력
- 링크 기반 후보 목록 출력
- 최신 backlog 를 찾지 못하면 경고 출력

## 4. Step 2: Check Doc Metadata

문서 디렉터리 전체에서 필수 메타데이터 누락 여부를 검사한다.

```bash
python3 mcp_servers/check-doc-metadata/scripts/run_check_doc_metadata.py \
  --doc-dir-path .
```

기대 결과:

- 검사한 markdown 파일 목록 출력
- 메타데이터 누락 파일이 있으면 `missing_metadata` 에 구조화된 결과 출력

## 5. Step 3: Check Doc Links

문서 디렉터리 전체에서 상대 링크 무결성을 검사한다.

```bash
python3 mcp_servers/check-doc-links/scripts/run_check_doc_links.py \
  --doc-dir-path .
```

기대 결과:

- 검사한 markdown 파일 목록 출력
- 끊어진 링크가 있으면 `broken_links` 에 구조화된 결과 출력

## 6. Step 4: Create Backlog Entry

새 작업을 날짜별 backlog 항목 초안으로 생성한다.

```bash
python3 mcp_servers/create-backlog-entry/scripts/run_create_backlog_entry.py \
  --task-id TASK-099 \
  --task-name "MCP 초안 생성 확인" \
  --request-date 2026-04-18
```

기대 결과:

- backlog 항목 형식의 `draft_entry` JSON 초안 출력
- 기본 상태와 우선순위가 함께 채워진 구조 출력

## 7. Step 5: Suggest Impacted Docs

변경 파일을 기준으로 함께 봐야 할 문서 후보를 빠르게 추천한다.

```bash
python3 mcp_servers/suggest-impacted-docs/scripts/run_suggest_impacted_docs.py \
  --changed-file app/jobs/delivery_sync.py \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --latest-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md
```

기대 결과:

- `impacted_documents` 출력
- 변경 파일 해석 근거를 `reasoning_notes` 로 출력
- 상태 문서 후보가 부족하면 경고 출력

## 8. 추천 읽기 순서

이 저장소를 처음 보는 사람에게는 아래 순서를 권장한다.

1. [../README.md](../README.md)
2. [../mcp_servers/README.md](../mcp_servers/README.md)
3. [./README.md](./README.md)
4. 이 문서의 5개 프로토타입 명령 실행

## 9. 현재 한계

- 현재 프로토타입은 CLI 스크립트 수준이며 MCP transport 는 아직 없다.
- `suggest_impacted_docs` 는 보수적인 후보 추천에 집중한 단순 버전이다.
- `check_doc_metadata` 와 `check_doc_links` 는 저장소 전역 검사에 적합하지만 프로젝트별 예외 규칙은 아직 반영하지 않는다.

## 다음에 읽을 문서

- examples 허브: [./README.md](./README.md)
- mcp 허브: [../mcp_servers/README.md](../mcp_servers/README.md)
