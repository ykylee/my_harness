# Workflow Global Injection Policy

- 문서 목적: 표준 AI 워크플로우를 하네스 전역 설정과 프로젝트 설정에 주입할 때, 기존 사용자 기본값을 최대한 덮어쓰지 않도록 하는 비침투적 주입 원칙을 정의한다.
- 범위: 전역 주입 원칙, 프로젝트 설정 최소화 원칙, 충돌 가능 항목, 언어/보고 정책, 컨텍스트 절약 원칙, 하네스별 적용 메모
- 대상 독자: 저장소 관리자, 하네스 통합 담당자, AI workflow 설계자, 운영 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./workflow_configuration_layers.md`, `./workflow_harness_distribution.md`, `../global-snippets/README.md`, `../harnesses/codex/apply_guide.md`, `../harnesses/opencode/apply_guide.md`

## 1. 기본 원칙

- 전역 설정은 하네스 사용자의 기본 선호를 담는 공간으로 남겨둔다.
- 프로젝트 설정은 workflow 진입점과 프로젝트 로컬 문서를 연결하는 데 집중한다.
- 프로젝트 설정이 기존 provider, model, reasoning effort, theme, UI 같은 사용자 선호를 불필요하게 덮어쓰지 않도록 한다.
- 프로젝트별로 꼭 필요한 항목만 명시하고, 나머지는 하네스의 기존 전역 기본값을 그대로 따른다.

## 2. 프로젝트 설정에서 기본적으로 피할 항목

프로젝트별 workflow 주입 시 아래 항목은 기본적으로 쓰지 않는 것을 권장한다.

- 기본 provider 지정
- 기본 model 지정
- reasoning effort 기본값 지정
- 전역 UI/TUI 테마 지정
- 사용자 전역 permission 기본값 전체 덮어쓰기
- 사용자 전역 공유/telemetry/autoupdate 같은 개인 선호

이 항목은 프로젝트마다 달라질 필요가 적고, 사용자 개인 선호와 충돌하기 쉽다.

## 3. 프로젝트 설정에 넣기 좋은 항목

- 공통 상위 진입 문서 경로
- 프로젝트 로컬 workflow 문서 경로
- 프로젝트 전용 skill/agent 정의
- 프로젝트에 필요한 MCP 연결 정보
- 프로젝트 문서 구조와 backlog/handoff 경로
- 사용자에게 보이는 작업 보고와 문서 작성 언어 정책
- 중간 사고/출력 최소화 같은 컨텍스트 절약 원칙

## 3.1 언어/컨텍스트 정책 주입 원칙

- 하네스 전역 또는 프로젝트 로컬 instruction 에는 “사용자 노출 산출물은 한국어” 같은 명시적 출력 언어 정책을 넣는다.
- 반대로 내부 사고 과정의 언어까지 강제하지는 않는다.
- 모델 내부 처리 방식은 자유롭게 두되, 사용자에게 보이는 중간 설명은 짧고 필요한 수준으로 제한한다.
- 장문의 자기 설명, 중복 요약, 불필요한 사고 과정 노출을 기본 동작으로 만들지 않는다.
- 이 정책은 model/provider 기본값보다 프로젝트 workflow 품질과 더 직접적으로 연결되므로 additive instruction 으로 두기 좋다.

이 항목은 workflow 적용에 직접 필요하고, 프로젝트 맥락 의존성이 크다.

## 4. Codex 적용 원칙

- `AGENTS.md` 는 프로젝트 로컬 지침 진입점이므로 적극 사용한다.
- `.codex/config.toml.example` 는 자동 적용 파일이 아니라 참고용 예시로 둔다.
- Codex 전역 설정에 주입할 snippet 도 MCP 연결이나 기능 플래그처럼 additive 한 항목 중심으로 유지한다.
- model/provider/reasoning 기본값은 사용자가 이미 갖고 있을 수 있으므로 예시 snippet 에 기본 포함하지 않는다.
- 언어/보고 원칙과 컨텍스트 절약 원칙은 `AGENTS.md` 또는 연결된 workflow 문서에서 명시하는 편이 안전하다.

## 5. OpenCode 적용 원칙

- `opencode.json` 은 merge 되는 프로젝트 config 이므로 특히 더 비침투적으로 유지한다.
- `opencode.json` 에는 `instructions`, 프로젝트 로컬 agent/skill 연결, 프로젝트에 필요한 additive MCP 정보만 우선 넣는다.
- model/provider 관련 top-level key 는 기본 생성물에서 생략한다.
- 사용자 전역 permission 기본값을 프로젝트 config 에서 통째로 덮어쓰지 않는다.
- 프로젝트별 권한 정책이 필요하면 project-local agent 단위에서만 최소 범위로 정의한다.
- 특히 메인 오케스트레이터 agent 는 task-only coordinator 성격으로 두고, 직접 수정/광범위 탐색 권한은 원칙적으로 주지 않는 편이 안전하다.
- 실제 탐색, 수정, 검증은 project-local worker agent 로 분리하고, `ask` 는 genuinely blocking decision 이나 위험한 외부 작업에만 쓰도록 좁히는 편이 기본 정책과 잘 맞는다.
- 언어 정책과 응답 길이/중간 설명 최소화 정책은 `instructions` 또는 project-local skill/agent 문서에 두는 것이 적합하다.

## 6. 예외가 필요한 경우

프로젝트에서 특정 model/provider 를 강하게 요구하는 경우도 있을 수 있다. 그 경우에도 아래 순서를 권장한다.

1. 먼저 문서로 왜 override 가 필요한지 적는다.
2. 가능하면 프로젝트 로컬 보조 설정 파일이나 별도 opt-in snippet 으로 분리한다.
3. 자동 생성 기본값에는 넣지 않고, 명시적 선택 시에만 반영한다.

## 7. 체크리스트

- 프로젝트 설정이 기존 사용자 model/provider 기본값을 덮어쓰지 않는가
- additive 한 설정만 project config 에 넣었는가
- permission 은 global default 대신 local agent 범위로 최소화했는가
- 사용자 전역 설정 없이도 프로젝트 workflow 진입이 가능한가

## 8. 전역 snippet 운영

- 전역 snippet 은 `global-snippets/` 아래에 하네스별로 둔다.
- snippet 은 additive 한 예시만 포함하고, 프로젝트 경로나 backlog 상태는 넣지 않는다.
- 전역 snippet 적용은 자동이 아니라 수동 merge 또는 운영 도구를 통한 opt-in 방식이 적절하다.

## 다음에 읽을 문서

- 설정 계층 가이드: [./workflow_configuration_layers.md](./workflow_configuration_layers.md)
- 전역 snippet 허브: [../global-snippets/README.md](../global-snippets/README.md)
- Codex 적용 가이드: [../harnesses/codex/apply_guide.md](../harnesses/codex/apply_guide.md)
- OpenCode 적용 가이드: [../harnesses/opencode/apply_guide.md](../harnesses/opencode/apply_guide.md)
