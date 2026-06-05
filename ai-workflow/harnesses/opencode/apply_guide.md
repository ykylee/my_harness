# OpenCode Workflow Apply Guide

- 문서 목적: 기존 또는 신규 프로젝트에 표준 AI 워크플로우를 OpenCode 하네스 기준으로 적용하는 실제 절차를 단계별로 안내한다.
- 범위: bootstrap 실행, OpenCode 설정 연결, project-local skill/agent 검토, 첫 세션 시작 방법
- 대상 독자: OpenCode 사용자, 저장소 관리자, AI workflow 설계자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./README.md`, `../../core/workflow_adoption_entrypoints.md`, `../../core/workflow_configuration_layers.md`, `../../core/workflow_global_injection_policy.md`, `../../scripts/bootstrap_workflow_kit.py`

## 1. 언제 이 가이드를 쓰는가

- 프로젝트에서 OpenCode 를 주 하네스로 사용하려고 할 때
- 표준 workflow 문서를 `opencode.json` 과 project-local `.opencode/` 구조에 연결하려고 할 때
- 신규 프로젝트 또는 기존 프로젝트에 OpenCode 기준 도입을 시작하려고 할 때

## 2. 적용 전 확인

- OpenCode 가 프로젝트 루트 `opencode.json` 과 `.opencode/` 디렉터리를 읽는 구조를 사용할 수 있어야 한다.
- OpenCode 적용에서도 `AGENTS.md` 를 공통 상위 진입 문서로 함께 사용한다.
- OpenCode 전역 정책과 프로젝트 로컬 문서의 우선순위를 분리할 수 있어야 한다.
- 팀이 project-local skill/agent 정의를 저장소에 포함하는 방식에 동의해야 한다.
- 기존 프로젝트라면 자동 추정된 권한 정책과 명령을 사람이 검토해야 한다.

## 2.1 권장 설정 계층

- 전역:
- 사용자 기본 OpenCode 정책에는 공통 안전 규칙과 기본 권한 철학만 둔다.
- 공유:
- `AGENTS.md`, `opencode.json`, `.opencode/...`, `ai-workflow/` 패키지를 둔다.
- 로컬:
- `ai-workflow/memory/` 문서에서 실제 명령, 문서 경로, backlog 상태를 관리한다.

프로젝트별 규칙은 항상 local 문서가 우선한다.

## 2.2 전역 기본값 보호 원칙

- `opencode.json` 은 merge 되는 project config 이므로, 기본 생성물에서는 model/provider 기본값을 넣지 않는다.
- top-level permission 기본값도 프로젝트 config 에서 통째로 덮어쓰지 않는다.
- 프로젝트 workflow 연결에 꼭 필요한 `instructions`, project-local agent/skill, additive MCP 정보만 우선 넣는다.
- 더 강한 override 가 필요하면 기본 생성물에 넣지 말고 별도 opt-in 설정으로 분리하는 편이 안전하다.

## 2.3 전역에 넣을 것과 넣지 않을 것

- 전역에 넣기 좋은 것:
- additive MCP 연결
- 공통 안전 정책
- 공유 가능한 최소 권한 철학
- 전역에 넣지 않는 것이 좋은 것:
- project-local `instructions`
- model/provider 기본값 강제
- 프로젝트별 permission 전체 덮어쓰기
- 특정 저장소 문서 경로와 backlog 상태

## 2.4 추천 운영 패턴

- 개인 전역 OpenCode 설정:
- `~/.config/opencode/opencode.json` 에 additive snippet 만 유지
- 프로젝트 공통 진입:
- `AGENTS.md`, `opencode.json`, `.opencode/...`
- 프로젝트 실제 운영값:
- `ai-workflow/memory/` 문서 세트

이렇게 두면 OpenCode 의 merge 특성을 활용하면서도 기존 사용자 기본값을 최대한 보존할 수 있다.

## 2.5 언어와 컨텍스트 운영 원칙

- OpenCode 에서 사용자에게 보이는 작업 보고, 상태 요약, 문서 초안은 한국어로 작성하도록 `AGENTS.md`, `opencode.json` 의 instruction 연결, project-local skill/agent 문서에 반영한다.
- 코드, 명령어, 파일 경로, 설정 key 는 필요한 경우 원문 그대로 유지한다.
- 내부 사고 과정과 중간 분류 방식은 모델이 효율적으로 선택하게 두고, 사용자에게는 필요한 결정과 다음 행동만 짧게 전달하도록 한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피하도록 project-local instructions 에 명시하는 것이 좋다.
- handoff 와 backlog 에는 후속 세션에 꼭 필요한 사실만 남겨 컨텍스트 증가를 줄인다.
- 메인 오케스트레이터는 task-only coordinator 로 두고, 직접 도구 호출 없이 대량 탐색/로그 수집/실제 수정은 서브 에이전트에 맡기도록 권한 정책을 나누는 편이 좋다.
- `main`/`small` 모델 구조를 쓴다면, 메인 오케스트레이터는 `main`, 문서/코드/검증 worker 는 기본적으로 `small` 쪽에 두는 편이 효율적이다.

## 3. 신규 프로젝트 적용 순서

1. 아래 명령으로 기본 문서 세트와 OpenCode overlay 를 생성한다.

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /path/to/project \
  --project-slug my_project \
  --project-name "My Project" \
  --adoption-mode new \
  --harness opencode \
  --copy-core-docs
```

2. `ai-workflow/memory/PROJECT_PROFILE.md` 에 실제 명령과 검증 규칙을 채운다.
3. `AGENTS.md` 가 생성됐는지 확인하고, 공통 상위 지침으로 사용할 내용을 검토한다.
4. `opencode.json` 의 `instructions` 가 `AGENTS.md` 와 `ai-workflow/memory/` 문서를 함께 가리키는지 확인한다.
   이때 한국어 보고 원칙과 중간 설명 최소화 원칙이 instruction 체인에서 누락되지 않았는지 함께 확인한다.
5. `.opencode/skills/standard-ai-workflow/SKILL.md` 와 `.opencode/agents/workflow-orchestrator.md` 의 권한 정책을 팀 운영 방식에 맞게 조정한다.
   전역 snippet 을 쓰려면 [../../global-snippets/opencode/opencode.global.jsonc](../../global-snippets/opencode/opencode.global.jsonc) 도 함께 검토한다.
   export bundle 을 쓰는 경우 `bundle/source-docs/schemas/read_only_transport_descriptors.json` 는 read-only MCP 연결 검토용 descriptor 이며, 정식 서버 루프가 연결되기 전에는 참고 산출물로 취급한다.
   가능하면 메인 오케스트레이터는 직접 `bash`/`edit`/`webfetch` 를 호출하지 않고 `task` 위임만 수행하게 둔다.
   `.opencode/agents/workflow-worker.md` 는 bounded scope 실행용으로 두고, 실제 구현과 빌드는 `workflow-code-worker`, 검증 증빙은 `workflow-validation-worker` 에 맡기는 운영 패턴을 함께 검토한다.
   worker 쪽은 범위만 충분히 좁혀 두고, low-risk 실행에서는 `ask` 를 과도하게 유발하지 않도록 설정하는 편이 실제 운영성이 좋다.
   필요하면 worker 를 `workflow-doc-worker`, `workflow-code-worker`, `workflow-validation-worker` 로 나눠 역할별로 호출한다.
6. 첫 세션에서 handoff 와 backlog 를 채운다.

## 4. 작업 중인 프로젝트 적용 순서

1. 아래 명령으로 기존 저장소 분석과 OpenCode overlay 를 함께 생성한다.

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /path/to/project \
  --project-slug my_project \
  --project-name "My Project" \
  --adoption-mode existing \
  --harness opencode \
  --copy-core-docs
```

2. `ai-workflow/memory/repository_assessment.md` 를 읽고 추정 명령과 문서 경로를 검토한다.
3. `PROJECT_PROFILE.md` 의 실행, 테스트, 검증 규칙을 실제 저장소 기준으로 수정한다.
4. `AGENTS.md` 와 `opencode.json` 의 instruction 경로가 같이 맞는지 확인한다.
   작업 보고는 한국어, 내부 처리는 간결하게 유지한다는 원칙도 이 단계에서 같이 검토한다.
   export bundle 을 쓰는 경우 read-only MCP descriptor 의 `transport_ready` 값이 `false` 임을 확인하고, 실제 MCP 연결은 별도 서버 루프가 준비된 뒤 진행한다.
5. `.opencode/agents/` 권한 정책을 팀 기준에 맞게 조정한다.
   이때 오케스트레이터가 직접 `bash`/`edit`/`webfetch` 를 수행하지 않도록 task-only 권한 프로필을 우선 고려한다.
   worker agent 는 실제 수정과 확인 작업을 맡되, 책임 파일과 종료 조건이 분명한 형태로만 호출하는 패턴을 권장한다.
   특히 구현, 설정 변경, 빌드/컴파일 확인은 `workflow-code-worker` 에 우선 배정하는 편이 자연스럽다.
   작업 중 추가 질의는 genuinely blocking case 로 좁히고, 나머지는 worker 가 최소 가정으로 계속 진행하도록 두는 편이 좋다.
   모델을 나눠 운영한다면 기본값은 `main orchestrator + small workers` 로 두고, 구조 판단이 어려운 경우에만 worker 를 일시적으로 `main` 으로 올리는 편이 좋다.
6. 첫 실제 작업을 backlog 에 반영하고 handoff 기준선을 갱신한다.

## 5. OpenCode 에서 첫 세션 시작하는 방법

- 먼저 `AGENTS.md` 와 `opencode.json` 의 instruction 목록이 현재 저장소 문서를 제대로 가리키는지 확인한다.
- 이어서 아래 문서를 순서대로 읽는다.
- `AGENTS.md`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- 기존 프로젝트 도입 직후라면 `ai-workflow/memory/repository_assessment.md` 도 함께 읽는다.
- 이후 `.opencode/skills/standard-ai-workflow/SKILL.md` 와 `.opencode/agents/workflow-orchestrator.md` 가 이 흐름을 따르는지 검토한다.

## 6. 적용 후 확인 체크리스트

- `opencode.json` 이 존재한다.
- `AGENTS.md` 가 존재한다.
- `.opencode/skills/standard-ai-workflow/SKILL.md` 가 존재한다.
- `.opencode/agents/workflow-orchestrator.md` 가 존재한다.
- `.opencode/agents/workflow-worker.md` 가 존재한다.
- `.opencode/agents/workflow-doc-worker.md` 가 존재한다.
- `.opencode/agents/workflow-code-worker.md` 가 존재한다.
- `.opencode/agents/workflow-validation-worker.md` 가 존재한다.
- `ai-workflow/memory/` 문서 세트가 존재한다.
- profile 문서의 명령과 검증 규칙이 실제 저장소 기준으로 채워져 있다.
- handoff 와 backlog 가 비어 있지 않다.

## 7. 자주 손보게 되는 부분

- `opencode.json` 의 instruction 경로
- project-local agent 권한 정책
- worker agent 의 역할 범위와 권한
- `workflow-code-worker` 의 구현/빌드 책임 범위
- `PROJECT_PROFILE.md` 의 검증 기준
- handoff 와 backlog 의 최신 상태

## 8. 피해야 할 구성

- 프로젝트 `opencode.json` 에 model/provider 기본값을 직접 넣는 것
- project config 에 top-level permission 기본값을 통째로 넣는 것
- 전역 설정에 프로젝트별 `instructions` 를 넣는 것
- `.opencode/agents/` 권한 정책으로 프로젝트 문서 규칙까지 대체하려는 것

## 다음에 읽을 문서

- OpenCode 패키지 안내: [./README.md](./README.md)
- 하네스 허브: [../README.md](../README.md)
- 도입 분기 가이드: [../../core/workflow_adoption_entrypoints.md](../../core/workflow_adoption_entrypoints.md)
- 설정 계층 가이드: [../../core/workflow_configuration_layers.md](../../core/workflow_configuration_layers.md)
- 비침투적 주입 정책: [../../core/workflow_global_injection_policy.md](../../core/workflow_global_injection_policy.md)
- 전역 snippet: [../../global-snippets/opencode/opencode.global.jsonc](../../global-snippets/opencode/opencode.global.jsonc)
