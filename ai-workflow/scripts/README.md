# Scripts

- 문서 목적: 표준 AI 워크플로우 저장소에서 제공하는 실행 스크립트의 역할과 사용 시점을 안내한다.
- 범위: bootstrap 스크립트, 신규/기존 프로젝트 도입 모드, Codex/OpenCode 하네스 오버레이, end-to-end 데모 runner, 출력 형태
- 대상 독자: 개발자, 운영자, AI agent 설계자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `../README.md`, `../examples/end_to_end_skill_demo.md`, `../examples/end_to_end_mcp_demo.md`, `../core/existing_project_onboarding_contract.md`

## 현재 포함된 스크립트

- [bootstrap_workflow_kit.py](./bootstrap_workflow_kit.py)
- [apply_harness_update.py](./apply_harness_update.py)
- [export_harness_package.py](./export_harness_package.py)
- [generate_workflow_state.py](./generate_workflow_state.py)
- [scaffold_harness.py](./scaffold_harness.py)
- [run_demo_workflow.py](./run_demo_workflow.py)
- [run_existing_project_onboarding.py](./run_existing_project_onboarding.py)

## 로컬 MCP SDK 환경 메모

- 공식 MCP Python SDK 패키지는 `mcp` 이고 Python 3.10 이상이 필요하다.
- 이 저장소에서는 Homebrew Python 3.11 과 루트 `.venv` 가상환경 기준으로 설치/검증했다.
- 설치 예시:

```bash
brew install python@3.11
rm -rf .venv
/opt/homebrew/bin/python3.11 -m venv .venv
./.venv/bin/python -m pip install --upgrade pip
./.venv/bin/python -m pip install "mcp[cli]"
```

- 검증 예시:

```bash
./.venv/bin/python -m pip show mcp
./.venv/bin/python tests/check_read_only_mcp_sdk_candidate.py
./.venv/bin/python tests/check_read_only_mcp_sdk_stdio.py
```

## bootstrap_workflow_kit.py

- 목적:
- 새 저장소 또는 임시 디렉터리에 표준 AI 워크플로우 기본 문서 세트와 선택적 core 문서를 한 번에 생성한다.
- 도입 모드:
- `--adoption-mode new` 는 신규 프로젝트용 기본 세트를 만든다.
- `--adoption-mode existing` 는 기존 코드베이스를 분석해 profile, handoff, backlog, repository assessment 초안을 함께 만든다.
- 하네스 옵션:
- `--harness codex` 는 `AGENTS.md` 와 `.codex/config.toml.example` 를 생성한다.
- `--harness opencode` 는 `AGENTS.md`, `opencode.json`, `.opencode/skills/...`, `.opencode/agents/...` 를 생성한다.
- `--harness gemini-cli` 는 `GEMINI.md` 를 생성한다.
- `--harness antigravity` 는 `ANTIGRAVITY.md` 를 생성한다.
- 하네스 확장 방식:
- bootstrap 스크립트는 하네스 레지스트리 기반으로 동작하므로, 추후 다른 하네스도 같은 패턴으로 추가할 수 있다.
- 기본 생성 구조:
- `ai-workflow/README.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/backlog/YYYY-MM-DD.md`
- `ai-workflow/memory/repository_assessment.md` (`existing` 모드일 때만)
- 선택 옵션:
- `--copy-core-docs` 를 주면 핵심 core 문서를 `ai-workflow/core/` 아래에 함께 복사한다.
- 경로 해석 원칙:
- `ai-workflow/memory/*` 는 workflow state docs 로 취급한다.
- `PROJECT_PROFILE.md` 안의 `문서 위키 홈`, `운영 문서 위치`, `백로그 위치`, `세션 인계 문서 위치` 는 실제 프로젝트 루트 기준 `docs/...` 경로를 적는다.
- 출력 형태:
- 생성된 경로와 다음 작업을 JSON 으로 출력한다.
- 상태 캐시:
- bootstrap 은 `ai-workflow/memory/state.json` 을 함께 생성해, 에이전트가 handoff/backlog 전체를 읽기 전에 현재 기준선을 빠르게 복원할 수 있게 한다.
- manifest 추가 정보:
- `global_snippet_candidates` 필드로 하네스별 전역 snippet 후보와 적용 대상 전역 설정 경로를 함께 출력한다.
- 생성물 예시:
- bootstrap 으로 생성되는 문서/하네스 문구 샘플은 [../examples/bootstrap_output_samples.md](../examples/bootstrap_output_samples.md) 에 정리돼 있다.

실행 예시:

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /tmp/sample-repo \
  --project-slug sample_api \
  --project-name "Sample API" \
  --harness codex \
  --harness opencode \
  --harness antigravity \
  --copy-core-docs
```

기존 프로젝트 분석 예시:

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /tmp/existing-repo \
  --project-slug payments_api \
  --project-name "Payments API" \
  --adoption-mode existing \
  --harness codex \
  --copy-core-docs
```

## apply_harness_update.py

- 목적:
- export package 나 bundle-like 디렉터리를 기존 저장소에 안전하게 반영한다.
- 기본 동작:
- `AGENTS.md`, `ai-workflow/`, `.codex`, `opencode.json`, `.opencode` 중 source 에 실제로 들어 있는 경로만 관리 대상으로 본다.
- 동일한 내용은 건너뛰고, 바뀐 경로만 `<target-root>/.ai-workflow-backups/<timestamp>/` 아래에 먼저 백업한 뒤 교체한다.
- 파일/디렉터리 타입 충돌도 백업 후 교체한다.
- 사용 시점:
- 기존 저장소에 workflow 재배포나 버전 업데이트를 적용할 때
- 과거 flat `.codex` 파일을 새 `.codex/` 디렉터리 구조로 안전하게 바꿔야 할 때
- 입력 형태:
- `--source-root` 는 export package 루트 또는 바로 `bundle/` 내용이 들어 있는 디렉터리를 받을 수 있다.
- 출력 형태:
- 적용 결과, 백업 위치, 생성/업데이트/건너뜀 경로를 JSON 으로 출력한다.

실행 예시:

```bash
python3 scripts/apply_harness_update.py \
  --source-root /path/to/standard-ai-workflow-codex-prototype-v2 \
  --target-root /path/to/project
```

dry-run 예시:

```bash
python3 scripts/apply_harness_update.py \
  --source-root /path/to/package \
  --target-root /path/to/project \
  --dry-run
```

## generate_workflow_state.py

- 목적:
- `PROJECT_PROFILE.md`, `session_handoff.md`, `work_backlog.md`, 최신 날짜 backlog 를 읽어 `state.json` 캐시를 다시 생성한다.
- 사용 시점:
- handoff 나 backlog 를 갱신한 직후
- 다른 환경에서 export bundle 을 적용한 뒤 첫 세션 기준선을 빠르게 맞출 때
- 출력 형태:
- 생성된 `state.json` 경로를 JSON 으로 출력한다.
- 운영 원칙:
- `state.json` 은 source of truth 가 아니라 빠른 세션 복원용 캐시다.
- 원본 상태는 계속 markdown 문서 세트를 기준으로 관리한다.
- `backlog-update`, `merge-doc-reconcile` 는 source-of-truth 문서가 준비된 경우 이 스크립트와 같은 로직으로 `state.json` 을 자동 재생성한다.
- 별도 문서 수정이나 외부 적용 뒤 독립적으로 캐시만 다시 만들고 싶을 때 이 스크립트를 직접 실행한다.

## backlog-update apply mode

- `skills/backlog-update/scripts/run_backlog_update.py --apply` 는 JSON 초안만 내는 read/suggest 경로와 달리 실제 workflow 문서를 갱신한다.
- 현재 자동 반영 범위:
- 날짜별 backlog entry 생성 또는 갱신
- `work_backlog.md` 에 날짜별 backlog 링크 보장
- `session_handoff.md` 의 `in_progress` / `blocked` / `done` 목록 동기화
- 그 뒤 `state.json` 자동 재생성
- 보수적 제한:
- `merge-doc-reconcile` 처럼 병합 후 재확정이 필요한 단계는 아직 read-first 원칙을 유지한다.

## merge-doc-reconcile apply mode

- `skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py --apply` 는 상태값 자동 확정 대신 handoff 운영 메모에 재정리 노트만 남기는 제한적 write mode 다.
- 현재 자동 반영 범위:
- `session_handoff.md` 의 `현재 세션 운영 메모` 섹션에 중복 없는 reconcile note 추가
- 그 뒤 `state.json` 자동 재생성
- 보수적 제한:
- backlog 상태, 완료 여부, 차단 해제, 허브 문서 설명은 자동 확정하지 않는다.

실행 예시:

```bash
python3 scripts/generate_workflow_state.py \
  --project-profile-path /path/to/project/ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path /path/to/project/ai-workflow/memory/session_handoff.md \
  --work-backlog-index-path /path/to/project/ai-workflow/memory/work_backlog.md \
  --latest-backlog-path /path/to/project/ai-workflow/memory/backlog/2026-04-24.md \
  --output-path /path/to/project/ai-workflow/memory/state.json
```

## run_demo_workflow.py

- 목적:
- 예시 프로젝트 문서를 대상으로 session-start, backlog-update, doc-sync, validation-plan, code-index-update, merge-doc-reconcile 흐름을 한 번에 실행한다.
- 지원 예시:
- `acme_delivery_platform`
- `research_eval_hub`
- 출력 형태:
- 단계별 JSON 결과와 요약 정보를 합친 통합 JSON
- 포함 단계:
- 최신 backlog 식별, 세션 기준선 복원, backlog 초안 생성, 영향 문서 추천, 검증 계획 추천, 색인 문서 후보 추천, 병합 후 문서 재정리 포인트 산출
- 탐색 범위 원칙:
- `doc-sync`, `validation-plan`, `code-index-update`, `merge-doc-reconcile` 는 `ai-workflow/` 경로를 일반 프로젝트 문서 탐색 범위에서 제외한다.

실행 예시:

```bash
python3 scripts/run_demo_workflow.py
python3 scripts/run_demo_workflow.py --example-project research_eval_hub
```

## run_existing_project_onboarding.py

- 목적:
- `bootstrap_workflow_kit.py --adoption-mode existing` 직후에 repository assessment 와 초기 workflow 문서를 읽고 후속 skill 을 순서대로 실행한다.
- 입력:
- `PROJECT_PROFILE.md`, `session_handoff.md`, `work_backlog.md`, backlog 디렉터리, 선택적으로 `repository_assessment.md`
- 출력 형태:
- latest backlog 식별, session-start, validation-plan, code-index-update 결과와 onboarding 요약을 합친 통합 JSON
- 계약 문서:
- 단계별 입력/출력 연결 규칙은 [../core/existing_project_onboarding_contract.md](../core/existing_project_onboarding_contract.md) 를 따른다.
- 소비 우선순위:
- 하네스 또는 사람이 바로 읽을 때는 `status -> onboarding_summary.recommended_next_steps -> warnings -> orchestration_plan -> validation_plan -> code_index_update -> session_start -> repository_assessment.summary` 순서를 권장한다.
- latest backlog 미발견 처리:
- backlog index 와 backlog 디렉터리에서 최신 backlog 를 찾지 못해도 runner 는 가능한 범위의 결과를 계속 반환한다.
- 이 경우 `latest_backlog.latest_backlog_path` 와 top-level `source_context.latest_backlog_path` 는 `null` 이다.
- 실제 파일럿 참고:
- `open_git_client` 에 bootstrap 과 onboarding runner 를 실제 적용한 기록은 [../examples/pilot_adoption_open_git_client_example.md](../examples/pilot_adoption_open_git_client_example.md) 에 정리돼 있다.

실행 예시:

```bash
python3 scripts/run_existing_project_onboarding.py \
  --project-profile-path /path/to/project/ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path /path/to/project/ai-workflow/memory/session_handoff.md \
  --work-backlog-index-path /path/to/project/ai-workflow/memory/work_backlog.md \
  --backlog-dir-path /path/to/project/ai-workflow/memory/backlog \
  --repository-assessment-path /path/to/project/ai-workflow/memory/repository_assessment.md
```

## scaffold_harness.py

- 목적:
- 새 하네스를 추가할 때 `harnesses/<target>/` 아래 starter README 와 overlay spec 초안을 자동 생성한다.
- 출력 형태:
- 생성된 경로와 다음 작업을 JSON 으로 출력한다.

실행 예시:

```bash
python3 scripts/scaffold_harness.py \
  --harness-name claude-code \
  --display-name "Claude Code" \
  --root-entrypoint "TODO: CLAUDE.md" \
  --config-file "TODO: .claude/config.json"
```

## export_harness_package.py

- 목적:
- 선택한 하네스 패키지를 `dist/harnesses/<target>/` 아래 bundle, manifest, zip 파일로 export 한다.
- 이번 릴리즈 기준:
- export 묶음은 workflow/skill 온보딩과 파일럿 적용 준비를 바로 시작할 수 있는 최소 런타임 문서 세트를 우선 포함한다.
- 출력 형태:
- export 결과 경로와 포함 파일 수를 JSON 으로 출력한다.
- 버저닝:
- 기본 버전 값은 `workflow_kit.__version__` 이며, 하네스별 export 는 `dist/harnesses/<target>/<version>/` 아래에 생성된다.
- zip 파일 이름도 `standard-ai-workflow-<target>-<version>.zip` 형식을 따른다.
- 패키지 문서:
- 각 export 루트에는 `PACKAGE_CONTENTS.md` 와 `APPLY_GUIDE.md` 가 함께 생성돼, 다른 환경에서 패키지를 받은 사람이 구성과 적용 순서를 바로 확인할 수 있다.
- manifest 추가 정보:
- `package_name`, `package_version` 은 배포 단위 식별자다.
- `release_focus` 는 현재 배포 중심축을 기록한다.
- `optimization_profile` 은 기본 배포 프로필이 `agent_runtime_minimal` 임을 기록한다.
- `recommended_entrypoints` 는 패키지 소비자가 우선 읽어야 하는 workflow/skill 진입 파일 묶음을 기록한다.
- `deferred_release_items` 는 패키지에 참고 자료로 포함되지만 이번 릴리즈 기본 경로로는 승격하지 않은 항목을 기록한다.
- `excluded_by_default` 는 컨텍스트 절약을 위해 기본 export 에서 뺀 항목을 기록한다.
- 선택 옵션:
- `--include-source-docs` 는 개발 참고용 원본 문서 사본을 함께 포함한다.
- `--include-global-snippets` 는 하네스 전역 설정 예시를 함께 포함한다.

## generate_read_only_transport_descriptors.py

- read-only MCP bundle registry 에서 draft transport tool descriptor 묶음을 생성한다.
- 체크인 산출물은 `schemas/read_only_transport_descriptors.json` 이다.
- 정식 MCP SDK 서버 루프를 붙이기 전, 외부 package/export 단계에서 descriptor 를 재사용할 수 있게 한다.

예시:

```bash
python3 scripts/generate_read_only_transport_descriptors.py > schemas/read_only_transport_descriptors.json
```

## generate_read_only_harness_mcp_examples.py

- read-only MCP descriptor 를 Codex/OpenCode 하네스별 MCP 설정 예시 draft 로 변환한다.
- 체크인 산출물은 `schemas/read_only_harness_mcp_examples.json` 이다.
- 예시는 `workflow_kit.server.read_only_jsonrpc --stdio-lines` draft bridge 를 가리키지만 `manual_review_only` 이며, `transport_ready=false` 상태에서는 실제 설정으로 자동 적용하지 않는다.

예시:

```bash
python3 scripts/generate_read_only_harness_mcp_examples.py > schemas/read_only_harness_mcp_examples.json
```

## generate_read_only_jsonrpc_fixtures.py

- read-only JSON-RPC draft bridge 의 대표 request/response fixture 를 생성한다.
- 체크인 산출물은 `schemas/read_only_jsonrpc_fixtures.json` 이다.
- 정식 MCP SDK transport 로 승격할 때 envelope 차이를 비교하기 위한 기준선으로 사용한다.

예시:

```bash
python3 scripts/generate_read_only_jsonrpc_fixtures.py > schemas/read_only_jsonrpc_fixtures.json
```
- manifest 추가 정보:
- `global_snippet_files` 필드로 관련 전역 snippet 파일이 bundle 에 함께 포함됐는지 추적한다.

실행 예시:

```bash
python3 scripts/export_harness_package.py \
  --harness codex \
  --harness opencode
```

## 다음에 읽을 문서

- 저장소 개요: [../README.md](../README.md)
- 도입 분기 가이드: [../core/workflow_adoption_entrypoints.md](../core/workflow_adoption_entrypoints.md)
- 기존 프로젝트 온보딩 계약: [../core/existing_project_onboarding_contract.md](../core/existing_project_onboarding_contract.md)
- 하네스 허브: [../harnesses/README.md](../harnesses/README.md)
- 릴리스 규격: [../core/workflow_release_spec.md](../core/workflow_release_spec.md)
- examples 허브: [../examples/README.md](../examples/README.md)
- bootstrap 생성물 샘플: [../examples/bootstrap_output_samples.md](../examples/bootstrap_output_samples.md)
- tests 허브: [../tests/README.md](../tests/README.md)
