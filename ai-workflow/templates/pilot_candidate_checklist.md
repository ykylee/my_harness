# 파일럿 후보 선정 체크리스트

- 목적: 시범 적용 대상 저장소 선정 기준
- 관련: [적용 기록 템플릿](./pilot_adoption_record_template.md)

## 1. 선정 기준 (Criteria)
- [ ] `existing` 모드 검증 가치가 큰가 (코드+문서 공존)
- [ ] 핵심 스킬(session-start 등) 실효성 검증이 가능한가
- [ ] 기본 명령(설치/테스트) 확인이 용이한가
- [ ] 운영 마찰 및 보안 제약이 감당 가능한 수준인가
- [ ] `code-index-update`를 위한 README/Docs 허브가 있는가

## 2. 제외 대상 (Exclusion)
- 기본 실행이 불가능할 정도의 엄격한 보안 환경
- 운영 문서가 거의 없는 극단적 코드 위주 저장소
- 대규모 리팩토링/조직 개편 진행 중인 프로젝트
- 테스트 코드가 전무하거나 모두 깨진 상태

## 3. 적용 전 확인
- 도입 모드: (new | existing)
- 하네스: (gemini-cli | opencode)
- 기본 명령 확보: (설치, 실행, 테스트, 격리, smoke)
- 문서 기준선: (README, docs 홈, runbook, 기존 backlog)

## 4. 기록 원칙
- 모든 파일럿은 [적용 기록](./pilot_adoption_record_template.md)을 남긴다.
- 최소 2개 이상 파일럿 후 공통 규칙 일반화 고려.
