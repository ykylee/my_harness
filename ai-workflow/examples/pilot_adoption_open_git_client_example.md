# open_git_client 파일럿 적용 기록 예시

- 문서 목적: `open_git_client` 저장소에 표준 AI 워크플로우를 실제로 도입한 첫 파일럿 결과를 기록한다.
- 범위: 적용 전 상태, 실제 생성물, runner 재실행 결과, 최소 실행 검증, 남은 마찰 지점
- 대상 독자: 저장소 관리자, AI workflow 설계자, 프로젝트 온보딩 담당자
- 상태: sample
- 최종 수정일: 2026-04-22
- 관련 문서: `../templates/pilot_adoption_record_template.md`, `./pilot_candidate_open_git_client_assessment.md`, `./pilot_execution_plan_existing_repo.md`

## 1. 적용 대상

- 적용일:
- `2026-04-22`
- 대상 저장소:
- `/home/yklee/repos/open_git_client`
- 저장소 성격:
- `C++20`, `wxWidgets`, `libgit2`, `CMake + Ninja` 기반 데스크톱 Git 클라이언트 프로젝트
- 기본 스택:
- `C++20 desktop app`
- 하네스:
- `codex`
- 적용 모드:
- `existing`
- 적용 담당:
- codex

## 2. 적용 전 상태

- 기존 운영 문서 위치:
- `README.md`, `docs/requirements/*`, `docs/design/*`, `docs/thread-operating-guidelines.md`
- 기존 세션/handoff 관행:
- 요구사항 수집 스레드 규칙은 있었지만, 저장소 전반의 세션 handoff/backlog 문서는 없었다.
- 기존 backlog 관행:
- 날짜별 backlog 나 session handoff 문서가 없고, 문서 변경 이력과 브랜치 규칙 중심으로 운영되고 있었다.
- 확인한 기본 명령:
- 설치:
- `sudo apt-get install -y build-essential cmake ninja-build pkg-config libwxgtk3.2-dev libgit2-dev`
- 실행:
- `build/bin/OpenGitClient`
- 빠른 테스트:
- `ctest --test-dir build --output-on-failure`
- 격리 테스트:
- `ctest --test-dir build --output-on-failure -R task_coordinator_tests`
- smoke 확인:
- `cmake -S . -B build -G Ninja && cmake --build build && ctest --test-dir build --output-on-failure`
- 적용 전에 이미 있던 제약:
- 주 타깃 플랫폼은 Windows 이지만 현재 즉시 검증 가능한 기준은 Linux build/ctest 였다.

## 3. 적용 범위

- 생성/복사한 문서:
- `ai-workflow/README.md`
- `ai-workflow/memory/README.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/backlog/2026-04-22.md`
- `ai-workflow/memory/repository_assessment.md`
- 적용한 하네스 오버레이:
- `AGENTS.md`
- `.codex/config.toml.example`
- 실행한 스크립트:
- `python3 scripts/bootstrap_workflow_kit.py --target-root /home/yklee/repos/open_git_client --adoption-mode existing --harness codex ...`
- `python3 scripts/run_existing_project_onboarding.py ...`
- 참고한 core 문서:
- `core/existing_project_onboarding_contract.md`
- `core/workflow_adoption_entrypoints.md`
- `core/output_schema_guide.md`

## 4. 적용 중 관찰

- 잘 맞았던 점:
- bootstrap 이 실제 저장소 구조를 빠르게 읽어 `repository_assessment.md` 와 운영 문서 초안을 생성했다.
- 수정이 필요했던 경로/명칭:
- 생성 직후 기본 운영 문서 경로 설명은 `docs/operations` 계열 표현이 섞여 있었고, 실제 생성 위치인 `ai-workflow/memory/` 기준으로 다시 맞춰야 했다.
- 헷갈렸던 입력:
- `repository_assessment.summary` 는 충분히 유용했지만 Windows 주 개발 환경과 Linux 검증 기준의 차이는 사람이 profile 에 직접 적어야 했다.
- 과도하거나 불필요했던 규칙:
- 첫 파일럿 단계에서는 orchestration worker 분배 메타데이터가 실제 문서 정렬 작업보다 약간 과했다.
- 부족했던 가이드:
- 실제 대상 저장소 기준 파일럿 적용 기록 예시가 없어서 어떤 마찰을 남겨야 하는지 템플릿만으로는 덜 선명했다.
- 수동 확인이 많이 필요했던 지점:
- 운영 문서 장기 위치를 `ai-workflow/memory/` 로 유지할지, 기존 `README.md` 와 `docs/` 체계로 더 녹여 넣을지는 사람이 판단해야 했다.

## 5. 적용 후 상태

- 최종 project profile 경로:
- `ai-workflow/memory/PROJECT_PROFILE.md`
- 최종 session handoff 경로:
- `ai-workflow/memory/session_handoff.md`
- 최종 work backlog 경로:
- `ai-workflow/memory/work_backlog.md`
- 최신 backlog 경로:
- `ai-workflow/memory/backlog/2026-04-22.md`
- 실제 첫 세션에서 사용한 runner 또는 skill:
- `run_existing_project_onboarding.py`
- 하네스에서 우선 읽은 출력:
- `status`
- `onboarding_summary.recommended_next_steps`
- `warnings`
- `orchestration_plan`
- `repository_assessment.summary`
- 남은 미정 항목:
- 전체 configure/build/smoke 를 파일럿 기본 검증으로 강제할지
- Windows 포터블 패키징 검증을 언제 필수로 요구할지
- `ai-workflow/memory/` 위치를 장기 운영 위치로 유지할지

## 6. 검증 결과

- 실행한 smoke/test:
- `python3 tests/check_docs.py` in `standard_ai_workflow`
- `python3 scripts/run_existing_project_onboarding.py ...`
- `ctest --test-dir build --output-on-failure` in `open_git_client`
- 실행하지 못한 항목과 사유:
- `cmake -S . -B build -G Ninja && cmake --build build && ctest --test-dir build --output-on-failure` 전체 smoke 는 아직 실행하지 않았다.
- Windows 포터블 패키징은 현재 세션 환경에서 직접 검증하지 않았다.
- 출력 계약 문제 여부:
- onboarding runner 상위 구조와 `repository_assessment.summary`, `onboarding_summary.inferred_commands` 채움 방식은 의도대로 동작했다.
- 문서 링크/메타데이터 문제 여부:
- 운영 문서 경로를 `ai-workflow/memory/` 기준으로 맞추고 허브 README 를 추가한 뒤, code-index 경고는 사라졌다.
- runner 또는 하네스 연결 문제 여부:
- 초기에는 handoff/backlog 상태 불일치 경고와 index 후보 경고가 있었지만, 파일럿 문서 보정 후 대부분 해소됐다.
- 남은 runner 경고:
- docs-only 입력일 때 `validation-plan` 이 실행 명령을 추천하지 않아 경고 1개가 남는다.

## 7. 적용 전후 비교

- 도입 전 대비 좋아진 점:
- 첫 세션에서 읽을 기준 문서와 다음 작업 순서가 `AGENTS.md` 와 onboarding JSON 으로 정리돼 진입 비용이 줄었다.
- 여전히 수동인 부분:
- 플랫폼 제약, 승인 규칙, 장기 문서 위치 결정은 사람이 마지막 판단을 해야 한다.
- 예상보다 비용이 컸던 부분:
- 초안 생성 직후 경로 표현을 실제 생성 위치에 맞춰 정리하는 문서 보정이 생각보다 필요했다.
- 다음 파일럿 전에 꼭 고칠 점:
- docs-only 파일럿에서도 `validation-plan` 이 기본 quick test 를 제안할지 여부를 정책으로 결정한다.

## 8. 최종 판단

- 재적용 의향:
- `yes`
- 권장 도입 범위:
- 코드, README, 기본 테스트 명령이 모두 있는 기존 저장소의 첫 도입
- 권장 제외 범위:
- 기본 실행/테스트 명령을 전혀 추정할 수 없고 운영 문서 기준선도 거의 없는 저장소의 즉시 적용
- 다음 후속 작업:
- `validation-plan` 의 docs-only 경고 정책을 정리하고, 2번째 실제 파일럿 후보에도 같은 기록 형식을 적용한다.
