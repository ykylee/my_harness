# Global Snippets

- 문서 목적: 하네스 전역 설정에 넣어도 비교적 안전한 표준 workflow snippet 예시를 모아둔다.
- 범위: Codex, OpenCode 전역 주입 샘플, 사용 시 주의점, 프로젝트 로컬 레이어와의 관계
- 대상 독자: 저장소 관리자, 하네스 통합 담당자, 운영 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `../core/workflow_global_injection_policy.md`, `../core/workflow_configuration_layers.md`

## 목적

- 전역 설정에 넣어도 되는 additive 한 항목만 별도로 관리한다.
- provider, model, reasoning effort 같은 사용자 기본값을 건드리지 않는 샘플을 제공한다.
- 프로젝트별 workflow 패키지는 여전히 `ai-workflow/` 와 하네스 overlay 에서 관리한다.

## 현재 포함된 스니펫

- [codex/README.md](./codex/README.md)
- [codex/config.toml.snippet](./codex/config.toml.snippet)
- [opencode/README.md](./opencode/README.md)
- [opencode/opencode.global.jsonc](./opencode/opencode.global.jsonc)

## 사용 원칙

- 전역 snippet 은 기본값 참고용이며, 프로젝트 규칙을 직접 담지 않는다.
- 실제 프로젝트 명령, 문서 경로, backlog 상태는 `ai-workflow/memory/` 문서가 우선한다.
- snippet 적용 전에는 사용 중인 기존 전역 설정과 충돌하지 않는지 확인한다.

## 다음에 읽을 문서

- 비침투적 주입 정책: [../core/workflow_global_injection_policy.md](../core/workflow_global_injection_policy.md)
- 설정 계층 가이드: [../core/workflow_configuration_layers.md](../core/workflow_configuration_layers.md)
