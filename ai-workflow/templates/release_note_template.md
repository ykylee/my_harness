# [Release Name: e.g. v0.4.0: Enhanced Packaging]

- **날짜**: YYYY-MM-DD
- **버전**: `vX.Y.Z[-suffix]`
- **상태**: **[Alpha / Beta / Release]**

## 1. 개요
- 이번 릴리스의 핵심 목표와 배경을 요약합니다.
- 예: "워크플로우 패키징 고도화 및 버전 기반 업그레이드 지원"

## 2. 주요 변경 사항 (Key Changes)

### 🚀 기능 추가 (Features)
- **[기능 명칭]**: 변경 사항에 대한 상세 설명 (예: 신규 스크립트 도입, 새 하네스 지원 등)
- **[기능 명칭]**: ...

### 🛠 버그 수정 및 최적화 (Fixes & Refactoring)
- **[수정 사항]**: 해결된 버그 또는 개선된 로직 설명 (예: 경로 판별 오류 수정, 설정 중앙화 등)
- **[최적화]**: 성능 개선 또는 코드 구조 리팩토링 내역

### 📄 문서 및 가이드 (Docs)
- **[문서 명칭]**: 새로 추가되거나 수정된 가이드 위치 (예: `core/workflow_release_spec.md`)
- **[가이드]**: 사용자/관리자를 위한 주요 안내 사항

## 3. 포함된 배포 패키지 (Assets)
- `standard-ai-workflow-antigravity-vX.Y.Z.zip`
- `standard-ai-workflow-codex-vX.Y.Z.zip`
- `standard-ai-workflow-gemini-cli-vX.Y.Z.zip`
- `standard-ai-workflow-opencode-vX.Y.Z.zip`
- `standard-ai-workflow-pi-dev-vX.Y.Z.zip`

## 4. 설치 및 업그레이드 가이드

### 신규 설치
1. `dist/` 내 하네스 패키지 압축을 해제합니다.
2. `bootstrap_workflow_kit.py`를 실행하여 초기화합니다.

### 업그레이드 (Recommended)
1. 최신 배포 패키지를 다운로드합니다.
2. `apply_workflow_upgrade.py`를 사용하여 안전하게 업데이트를 수행합니다.
   ```bash
   python3 workflow-source/scripts/apply_workflow_upgrade.py --source-root <BUNDLE_PATH> --target-root . --force-cleanup
   ```

---
*본 릴리스는 표준 AI 워크플로우 가버넌스를 준수하여 작성되었습니다.*
