# Codex Workflow Apply Guide

- 문서 목적: 기존 또는 신규 프로젝트에 표준 AI 워크플로우를 Codex 하네스 기준으로 적용하는 실제 절차를 단계별로 안내한다.
- 범위: bootstrap 실행, 생성 파일 검토, Codex 설정 연결, 첫 세션 시작 방법
- 대상 독자: Codex 사용자, 저장소 관리자, AI workflow 설계자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./README.md`, `../../core/workflow_adoption_entrypoints.md`, `../../core/workflow_configuration_layers.md`, `../../core/workflow_global_injection_policy.md`, `../../scripts/bootstrap_workflow_kit.py`

## 1. 언제 이 가이드를 쓰는가

- 프로젝트에서 Codex 를 주 하네스로 사용하려고 할 때
- 표준 workflow 문서를 Codex 의 `AGENTS.md` 진입점과 연결하려고 할 때
- 신규 프로젝트 또는 기존 프로젝트에 Codex 기준 도입을 시작하려고 할 때

## 2. 적용 전 확인

- Codex 가 프로젝트 루트의 `AGENTS.md` 를 읽는 흐름을 사용할 수 있어야 한다.
- Codex 전역 설정과 프로젝트 로컬 문서의 우선순위를 분리할 수 있어야 한다.
- 프로젝트에서 workflow 문서를 둘 위치를 `ai-workflow/` 로 유지할지 결정한다.
- 기존 프로젝트라면 기본 실행 명령과 테스트 명령을 자동 추정값과 대조할 사람이 필요하다.

## 2.1 권장 설정 계층

- 전역:
- `~/.codex/config.toml` 에 공통 MCP 와 기본 진입 원칙만 둔다.
- 공유:
- 프로젝트 루트 `AGENTS.md` 와 `ai-workflow/` 패키지를 둔다.
- 로컬:
- `ai-workflow/memory/` 문서에서 실제 명령, 경로, backlog 상태를 관리한다.

프로젝트별 규칙은 항상 local 문서가 우선한다.

## 2.2 전역 기본값 보호 원칙

- Codex 전역 `config.toml` 에 이미 있는 model/provider 기본값은 프로젝트 workflow 가 자동으로 덮어쓰지 않게 유지한다.
- `.codex/config.toml.example` 는 additive 한 MCP 예시 중심으로 유지한다.
- 프로젝트 특화 명령과 규칙은 `AGENTS.md` 와 `ai-workflow/memory/` 문서에서 읽게 한다.

## 2.3 전역에 넣을 것과 넣지 않을 것

- 전역에 넣기 좋은 것:
- 공통 MCP 연결
- 공통 기능 플래그
- 기본 안전 정책 철학
- 전역에 넣지 않는 것이 좋은 것:
- 프로젝트별 실행 명령
- 특정 저장소 문서 경로
- backlog 상태
- model/provider 기본값 강제

## 2.4 추천 운영 패턴

- 개인 전역 Codex 설정:
- `~/.codex/config.toml` 에 additive snippet 만 유지
- 프로젝트 공통 진입:
- 루트 `AGENTS.md`
- 프로젝트 실제 운영값:
- `ai-workflow/memory/` 문서 세트

이렇게 두면 전역 설정을 거의 건드리지 않으면서 프로젝트별 규칙만 교체할 수 있다.

## 2.5 언어와 컨텍스트 운영 원칙

- Codex 에서 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 초안은 한국어로 작성하도록 `AGENTS.md` 에 명시한다.
- 코드, 명령어, 파일 경로, 설정 key 는 필요한 경우 원문 그대로 유지한다.
- 내부 사고 과정, 임시 분류, 중간 reasoning 은 모델이 효율적으로 처리하게 두고, 사용자에게는 필요한 결론만 짧게 전달하도록 한다.
- 진행 업데이트는 짧고 목적 지향적으로 유지하고, 이미 확인한 사실을 반복해서 길게 설명하지 않는다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 컨텍스트 누적을 줄인다.
- Codex 에서도 가능한 경우 메인 에이전트는 조정/통합에 집중하고, bounded scope 읽기/쓰기/검증은 worker 성격의 서브 에이전트로 분리하는 운영 패턴을 권장한다.
- `main`/`small` 모델 구조를 운영한다면, 메인 에이전트는 `main`, 문서/코드/검증 성격의 bounded scope 서브 에이전트는 `small` 을 기본값으로 두는 편이 효율적이다.

## 3. 신규 프로젝트 적용 순서

1. 아래 명령으로 기본 문서 세트와 Codex overlay 를 생성한다.

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /path/to/project \
  --project-slug my_project \
  --project-name "My Project" \
  --adoption-mode new \
  --harness codex \
  --copy-core-docs
```

2. 생성된 `ai-workflow/memory/PROJECT_PROFILE.md` 에 실제 명령과 검증 규칙을 채운다.
3. 루트 `AGENTS.md` 가 `ai-workflow/memory/` 문서를 먼저 읽도록 연결됐는지 확인한다.
   이때 사용자 노출 산출물은 한국어, 내부 처리는 간결하게 유지한다는 원칙도 함께 넣는 것을 권장한다.
4. `.codex/config.toml.example` 를 참고해 필요한 Codex 전역 설정을 수동 반영할지 결정한다.
   전역 snippet 을 쓰려면 [../../global-snippets/codex/config.toml.snippet](../../global-snippets/codex/config.toml.snippet) 도 함께 검토한다.
   export bundle 을 쓰는 경우 `bundle/source-docs/schemas/read_only_transport_descriptors.json` 는 read-only MCP 연결 검토용 descriptor 이며, 정식 서버 루프가 연결되기 전에는 참고 산출물로 취급한다.
   Codex 는 project-local agent permission 분리를 직접 제공하지 않으므로, 메인/worker 역할 분리는 `AGENTS.md` 운영 원칙으로 명시하는 방식을 우선 사용한다.
   worker 는 문서 비교, bounded code edit, 검증 로그 수집처럼 역할을 나눠 호출하면 컨텍스트 절약 효과가 더 크다.
5. 첫 세션에서 `session_handoff.md` 와 오늘 날짜 backlog 를 채운다.

기존에 workflow 배포본이 이미 있다면 manual copy 대신 아래 updater 를 먼저 검토하는 편이 안전하다.

```bash
python3 scripts/apply_harness_update.py \
  --source-root /path/to/exported-package \
  --target-root /path/to/project
```

## 4. 작업 중인 프로젝트 적용 순서

1. 아래 명령으로 기존 저장소 분석과 Codex overlay 를 함께 생성한다.

```bash
python3 scripts/bootstrap_workflow_kit.py \
  --target-root /path/to/project \
  --project-slug my_project \
  --project-name "My Project" \
  --adoption-mode existing \
  --harness codex \
  --copy-core-docs
```

2. `ai-workflow/memory/repository_assessment.md` 를 읽고 추정 스택, 명령, 문서 경로가 실제 저장소와 맞는지 검토한다.
3. `PROJECT_PROFILE.md` 의 설치, 실행, 테스트 명령을 실제 운영 기준으로 수정한다.
4. 루트 `AGENTS.md` 의 기본 명령과 문서 경로가 맞는지 확인한다.
   작업 보고 언어와 컨텍스트 절약 원칙도 이 단계에서 함께 검토한다.
   export bundle 을 쓰는 경우 read-only MCP descriptor 의 `transport_ready` 값이 `false` 임을 확인하고, 실제 MCP 연결은 별도 서버 루프가 준비된 뒤 진행한다.
   가능하면 메인 에이전트가 직접 모든 읽기/쓰기를 떠안지 않도록, bounded scope worker 호출 원칙도 이 단계에서 같이 검토한다.
5. 첫 실제 작업을 오늘 날짜 backlog 에 등록하고, 세션 종료 전에 handoff 를 갱신한다.

기존 배포본을 갱신하는 상황이라면 먼저 dry-run 으로 변경 예정 경로와 backup 대상부터 확인한다.

```bash
python3 scripts/apply_harness_update.py \
  --source-root /path/to/exported-package \
  --target-root /path/to/project \
  --dry-run
```

## 5. Codex 에서 첫 세션 시작하는 방법

- 먼저 `AGENTS.md` 를 기준으로 현재 저장소 규칙을 읽는다.
- 이어서 아래 세 문서를 순서대로 읽는다.
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- 기존 프로젝트 도입 직후라면 `ai-workflow/memory/repository_assessment.md` 도 함께 읽는다.

## 6. 적용 후 확인 체크리스트

- `AGENTS.md` 가 존재한다.
- `ai-workflow/memory/` 문서 세트가 존재한다.
- profile 문서의 명령이 실제 저장소 기준으로 채워져 있다.
- 첫 backlog 항목과 handoff 가 비어 있지 않다.
- Codex 가 읽어야 할 시작 문서 경로가 팀 내에서 합의되어 있다.

## 7. 자주 손보게 되는 부분

- `AGENTS.md` 의 프로젝트 기본 명령
- `PROJECT_PROFILE.md` 의 검증 규칙
- `session_handoff.md` 의 현재 기준선
- 최신 날짜 backlog 의 상태값과 다음 세션 시작 포인트

## 8. 피해야 할 구성

- `~/.codex/config.toml` 에 프로젝트별 명령을 직접 넣는 것
- 여러 프로젝트가 공유하는 전역 설정에 특정 저장소 backlog 경로를 넣는 것
- `AGENTS.md` 대신 전역 설정만으로 프로젝트 운영 규칙을 모두 해결하려는 것

## 다음에 읽을 문서

- Codex 패키지 안내: [./README.md](./README.md)
- 하네스 허브: [../README.md](../README.md)
- 도입 분기 가이드: [../../core/workflow_adoption_entrypoints.md](../../core/workflow_adoption_entrypoints.md)
- 설정 계층 가이드: [../../core/workflow_configuration_layers.md](../../core/workflow_configuration_layers.md)
- 비침투적 주입 정책: [../../core/workflow_global_injection_policy.md](../../core/workflow_global_injection_policy.md)
- 전역 snippet: [../../global-snippets/codex/config.toml.snippet](../../global-snippets/codex/config.toml.snippet)
