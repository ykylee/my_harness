# my_harness

- **yklee의 개인 코딩 에이전트 하네스** — 모든 에이전트 작업의 단일 진입점
- 적용 도메인: **코드 개발 전반** (구현/리팩토링/리뷰/PR) · **기본 서버 관리** (프로세스/로그/설정/배포 헬퍼) · **환경 셋업** (로컬/원격 부트스트랩, 의존성, dotfiles)
- 기반 프레임워크: [`standard_ai_workflow`](https://github.com/ykylee/standard_ai_workflow) v0.5.0-beta (`ai-workflow/core/global_workflow_standard.md` 준수)
- 적용 하네스: `minimax-code` (Mavis / MiniMax Code 오버레이)
- 마지막 워크플로우 적용: 2026-06-05

## 이 저장소의 역할

이 저장소는 **워크플로우 컨슈머 + 운영 정책 단일 진입점** 이다. `standard_ai_workflow` 가 만든 kit 번들을 받아 `bootstrap_workflow_kit.py` 로 적용한 결과를 담고 있다. yklee 가 수행하는 모든 에이전트 작업 (코드 개발 / 서버 관리 / 환경 셋업) 은 본 하네스의 `MiniMax.md` 와 `docs/PROJECT_PROFILE.md` 규칙을 따른다. Mavis(MiniMax Code) 메인 orchestrator 가 `MiniMax.md` 를 진입점으로 워크플로우 세션을 시작하면, `.MiniMax/agents/` 의 워커들이 bounded scope 작업을 수행한다.

## 디렉토리 구조

```
.
├── MiniMax.md                       # Mavis 진입점 (메인 orchestrator 가 먼저 읽음)
├── MiniMax_config.example.json      # Mavis 설정 예시 (시크릿 제외)
├── .MiniMax/
│   └── agents/                      # 워커 정의 (orchestrator / worker / doc / code / validation)
├── docs/
│   └── PROJECT_PROFILE.md           # 이 하네스의 운영 규칙 / 명령 / 검증 기준
├── ai-workflow/
│   ├── README.md                    # Kit 사용 가이드
│   ├── core/                        # 워크플로우 코어 표준 (global_workflow_standard 등 7종)
│   ├── memory/
│   │   ├── state.json               # 자동 생성된 상태 캐시
│   │   ├── session_handoff.md       # 세션 간 인계 문서
│   │   ├── work_backlog.md          # 백로그 인덱스
│   │   └── backlog/                 # 일별 백로그
│   ├── skills/                      # 워크플로우 스킬 (session-start, backlog-update 등)
│   ├── mcp_servers/                 # MCP 도구 서버
│   ├── workflow_kit/                # 공통 엔진 (parser, state builder 등)
│   ├── scripts/                     # bootstrap, state 생성, export 등
│   ├── tests/                       # 스모크 테스트
│   ├── templates/                   # 문서 템플릿
│   ├── schemas/                     # 출력 JSON 스키마
│   ├── harnesses/                   # 하네스 카탈로그 (읽기용)
│   ├── examples/                    # 적용 예시 (읽기용)
│   └── global-snippets/             # 글로벌 설정 snippet 예시
└── README.md                        # 이 파일
```

## 첫 세션 시작하기

1. Mavis 세션을 열고 프로젝트 루트에서 시작.
2. 다음 프롬프트로 워크플로우 세션 활성화:
   > 프로젝트 루트의 `MiniMax.md` 를 읽고, `ai-workflow/memory/state.json` 을 기준으로 워크플로우 세션을 시작해줘.
3. Mavis 가 `MiniMax.md` → `state.json` → `session_handoff.md` → `work_backlog.md` → `docs/PROJECT_PROFILE.md` 순서로 읽고 현재 상태를 복원한다.
4. 첫 실제 작업은 `ai-workflow/memory/backlog/2026-06-05.md` 에 TASK 추가하고 `state.json` 을 재생성한다.

## 주요 명령

```bash
# 워크플로우 상태 캐시 재생성
PYTHONPATH=./ai-workflow python3 ./ai-workflow/scripts/generate_workflow_state.py \
  --project-profile-path docs/PROJECT_PROFILE.md \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --work-backlog-index-path ai-workflow/memory/work_backlog.md \
  --output-path ai-workflow/memory/state.json

# 워크플로우 재적용/업그레이드
python3 ./ai-workflow/scripts/bootstrap_workflow_kit.py \
  --target-root . \
  --project-slug my-harness \
  --project-name "My Harness" \
  --harness minimax-code \
  --adoption-mode new \
  --copy-core-docs \
  --force
```

## 문서 / 가이드 링크

- Mavis 진입 규칙: [MiniMax.md](./MiniMax.md)
- 하네스 운영 규칙: [docs/PROJECT_PROFILE.md](./docs/PROJECT_PROFILE.md)
- 워크플로우 코어 표준: [ai-workflow/core/global_workflow_standard.md](./ai-workflow/core/global_workflow_standard.md)
- Kit 사용 가이드: [ai-workflow/README.md](./ai-workflow/README.md)
- 세션 인계: [ai-workflow/memory/session_handoff.md](./ai-workflow/memory/session_handoff.md)
- 백로그 인덱스: [ai-workflow/memory/work_backlog.md](./ai-workflow/memory/work_backlog.md)
- 원본 프레임워크: https://github.com/ykylee/standard_ai_workflow

## 다음에 정해야 할 것

- `MiniMax.md` 의 TODO 명령 5종 (설치 / 로컬 실행 / 빠른 테스트 / 격리 테스트 / 실행 확인) — 실제 하네스 운영 명령으로 채우기
- `.MiniMax/config.json` 을 `MiniMax_config.example.json` 으로 초기화 (서버 토큰 등 시크릿은 환경변수 주입)
- 첫 실제 작업 시작 시 일별 백로그에 TASK 등록 → `state.json` 재생성
- (선택) `ai-workflow/tests/check_*.py` 를 컨슈머 레이아웃에 맞게 보정 — 현재는 소스 프레임워크(`workflow-source/` at root) 가정
