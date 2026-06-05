# Project Workflow Profile

- 문서 목적: 프로젝트 특화 규칙과 실행/검증 기준을 정의한다.
- 범위: 프로젝트 개요, 문서 구조, 기본 명령, 검증 포인트, 예외 규칙
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: YYYY-MM-DD
- 관련 문서: [공통 표준](../ai-workflow/core/global_workflow_standard.md)

## 1. 프로젝트 개요
- 프로젝트명: <Project Name>
- 프로젝트 목적: <핵심 사용자 가치 및 목표>
- 주요 이해관계자: <협업 부서 및 담당자>

## 2. 문서 구조 (Path)
- 문서 위키 홈: <README.md>
- 운영 문서 홈: <docs/operations/>
- 백로그 위치: <ai-workflow/memory/backlog/>
- 세션 인계 문서: <ai-workflow/memory/session_handoff.md>
- 환경 기록 위치: <ai-workflow/memory/repository_assessment.md>

## 3. 기본 명령 (Commands)
- 설치: <설치 및 가상환경 구성 명령>
- 로컬 실행: <어플리케이션 실행 명령>
- 빠른 테스트: <단위 테스트 및 Lint 명령>
- 격리 테스트: <Docker 또는 독립 환경 테스트 명령>
- 실행 확인: <상태 체크 및 E2E 확인 명령>

## 4. 검증 포인트 (Validation)
- 코드 변경: <테스트/리뷰 필수 사항>
- 문서 변경: <링크/메타데이터 정합성 기준>
- UI 변경: <시각적 검증 및 브라우저 확인 기준>
- 배포/운영: <릴리즈 승인 및 롤백 절차>

## 5. 예외 규칙 (Policy)
- 병합: <상태 문서 충돌 시 해결 우선순위>
- 승인: <특정 변경 시 필수 승인권자>
- 제약: <환경적/보안적 제약 사항>
- 기타: <프로젝트 특유의 컨벤션>

## 다음에 읽을 문서
- [세션 인계 문서](../ai-workflow/memory/session_handoff.md)
- [작업 백로그](../ai-workflow/memory/work_backlog.md)
