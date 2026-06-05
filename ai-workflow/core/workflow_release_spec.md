# Workflow Release Spec

- 문서 목적: 하네스별 워크플로우 패키지를 배포 가능한 산출물로 묶을 때 필요한 release 규격과 산출물 구조를 정의한다.
- 범위: dist 구조, 하네스 패키지 manifest, export 기준, 검증 포인트
- 대상 독자: 저장소 관리자, 배포 담당자, AI workflow 설계자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `./workflow_harness_distribution.md`, `./prototype_promotion_scope.md`, `../scripts/export_harness_package.py`, `../harnesses/README.md`

## 1. release 목표

- 공통 workflow 문서와 하네스 오버레이를 재사용 가능한 배포 단위로 묶는다.
- 이번 릴리즈는 workflow/skill 온보딩과 파일럿 적용 준비를 바로 시작할 수 있는 문서 세트를 우선 배포한다.
- 배포 패키지는 AI agent 가 실제로 읽고 사용하는 런타임 파일만 우선 포함하고, 개발 참고용 문서는 기본적으로 제외한다.
- 하네스별 산출물이 어떤 파일을 포함하는지 manifest 로 추적 가능해야 한다.
- 로컬에서 빠르게 export 해 검토하거나, 이후 CI에서 그대로 재사용할 수 있어야 한다.
- MCP 관련 산출물은 포함하되, 이번 릴리즈에서는 활성화 기본 경로가 아니라 차기 승격 검토용 참고 자료로 취급한다.

## 2. dist 구조

권장 dist 구조는 아래와 같다.

- `dist/harnesses/<target>/`
- `dist/harnesses/<target>/<version>/`
- `dist/harnesses/<target>/<version>/manifest.json`
- `dist/harnesses/<target>/<version>/PACKAGE_CONTENTS.md`
- `dist/harnesses/<target>/<version>/APPLY_GUIDE.md`
- `dist/harnesses/<target>/<version>/bundle/`
- `dist/harnesses/<target>/<version>/bundle/<runtime files>`
- `dist/harnesses/<target>/<version>/standard-ai-workflow-<target>-<version>.zip`
- opt-in 시 `dist/harnesses/<target>/<version>/bundle/source-docs/...`
- opt-in 시 `dist/harnesses/<target>/<version>/bundle/global-snippets/...`

## 3. 하네스 패키지 구성

기본 배포 패키지는 아래 두 레이어만 포함한다.

1. 공통 workflow 레이어
2. 하네스 오버레이 레이어

공통 workflow 레이어 기본 포함 항목:

- `ai-workflow/README.md`
- `ai-workflow/core/workflow_adoption_entrypoints.md`
- `ai-workflow/core/workflow_skill_catalog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/backlog/YYYY-MM-DD.md`

하네스 오버레이 레이어 포함 항목:

- Codex: `AGENTS.md`, `.codex/config.toml.example`
- OpenCode: `AGENTS.md`, `opencode.json`, `.opencode/skills/...`, `.opencode/agents/...`

기본 제외 항목:

- `source-docs/` 아래 개발 참고 문서 사본
- `global-snippets/` 아래 전역 설정 예시
- draft MCP descriptor, fixture, 예시 문서 사본

필요하면 export 시 opt-in 플래그로만 포함한다.

## 4. 릴리스 문서 규격 (Release Documentation)

일관성 있는 릴리스 관리를 위해 아래 템플릿을 필수 사용한다.

- **릴리스 템플릿**: \`templates/release_note_template.md\`
- **릴리스 명칭**: \`Beta vX.Y.Z\` (정식 릴리스 전에는 반드시 Beta 접두어 사용)
- **태그 규칙**: \`vX.Y.Z-beta\`
- **기록 위치**: \`releases/Beta-vX.Y.Z.md\`

릴리스 노트 작성 시 필수 포함 항목:
- 🚀 기능 추가 (Features)
- 🛠 버그 수정 및 최적화 (Fixes & Refactoring)
- 📄 문서 및 가이드 (Docs)
- 📦 배포 패키지 목록 (Assets)

## 5. manifest 최소 필드

- `harness`
- `package_name`
- `package_version`
- `release_focus`
- `optimization_profile`
- `exported_at`
- `source_root`
- `bundle_root`
- `included_files`
- `recommended_entrypoints`
- `deferred_release_items`
- `excluded_by_default`
- `notes`

패키지 루트 문서:

- `PACKAGE_CONTENTS.md`: 배포 패키지 구성과 기본 제외 항목 안내
- `APPLY_GUIDE.md`: 다른 환경이나 하네스 운영자가 바로 따라 할 적용 절차 안내

## 5. export 검증 포인트

- 선택한 하네스 오버레이 파일이 모두 bundle 에 포함됐는지 확인
- 공통 workflow 문서가 bundle 에 포함됐는지 확인
- workflow/skill 온보딩 시작에 필요한 핵심 runtime 문서만 bundle 에 포함됐는지 확인
- 기본 배포 프로필에서 `source-docs/`, `global-snippets/`, draft MCP 참고 자료가 빠졌는지 확인
- manifest 의 포함 파일 목록이 실제 bundle 과 일치하는지 확인
- zip 산출물 이름에 하네스와 버전이 함께 반영됐는지 확인
- zip 산출물이 생성됐는지 확인

## 6. 이번 릴리즈 권장 진입점

manifest 의 `recommended_entrypoints` 는 이번 릴리즈 소비자가 가장 먼저 읽어야 하는 파일 묶음을 가리킨다.

- Codex: `bundle/AGENTS.md` 이후 `bundle/ai-workflow/...`
- OpenCode: `bundle/AGENTS.md`, `bundle/opencode.json`, `bundle/.opencode/...` 이후 `bundle/ai-workflow/...`

manifest 의 `deferred_release_items` 는 이번 패키지에 참고 자료로는 포함되지만 기본 적용 경로로는 승격하지 않은 항목을 기록한다.
manifest 의 `excluded_by_default` 는 컨텍스트 절약을 위해 기본 배포 프로필에서 제외한 항목을 기록한다.

## 7. 운영 원칙

- 배포 산출물은 source-of-truth 가 아니라 export 결과물이다.
- 정책 변경은 항상 `core/`, `harnesses/`, `scripts/` 원본에서 수행한다.
- dist 는 재생성 가능해야 하며, export 스크립트가 같은 구조를 반복 생성할 수 있어야 한다.

## 8. 다음 단계와의 관계

- 현재 release spec 은 문서 패키지와 하네스 overlay export 를 기준으로 한다.
- reusable package 또는 MCP server 승격 범위는 [./prototype_promotion_scope.md](./prototype_promotion_scope.md) 에서 별도로 정의한다.
- 즉, 현재의 `dist/` export 와 향후 library/server 배포는 같은 문제가 아니라 두 단계의 별도 배포 축으로 본다.

## 다음에 읽을 문서

- 하네스 배포 전략: [./workflow_harness_distribution.md](./workflow_harness_distribution.md)
- 승격 범위 문서: [./prototype_promotion_scope.md](./prototype_promotion_scope.md)
- 하네스 허브: [../harnesses/README.md](../harnesses/README.md)
