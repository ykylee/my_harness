# my_harness

- **yklee의 개인 코딩 에이전트 CLI/TUI** — `myharness <command>` 로 terminal 에서 직접 실행
- **산출물**: standalone CLI/TUI coding agent (3-도메인: 코드 개발 / 서버 관리 / 환경 셋업)
- **기반 표준**: [`standard_ai_workflow`](https://github.com/ykylee/standard_ai_workflow) v0.5.0-beta — **6 원칙 native 준수** (한국어 보고 / 컨텍스트 절약 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
- **런타임 의존**: LLM provider API 와 **직접 통신** (Anthropic / OpenAI / Google / DeepSeek / local Ollama 등)
- **v1 컨셉 SSOT**: [`docs/CONCEPT.md`](./docs/CONCEPT.md) (Mavis zero coupling — my_harness 자체는 Mavis 와 무관)
- **컨셉 확립일**: 2026-06-07 (D-22~D-34)
- **다음 결정**: TASK-005 (스택: Rust vs TS) → v1 MVP 빌드

## 이 저장소의 두 가지 역할

이 저장소는 **두 가지 역할**을 동시에 한다 (D-25 명확화):

### A. **my_harness 산출물** 의 source tree (v1+)
- `myharness <command>` CLI 의 코드가 들어갈 영역
- v1 spec: [`docs/CONCEPT.md`](./docs/CONCEPT.md) §5 (12 subsections)
- 핵심 컴포넌트: Harness 5 (Tools/Context/Session/Plugins/Sub-agents) + Agent 3 모드 + 15개 built-in sub-agents

### B. **이 repo 의 개발 workflow** (D-25, yklee 가 my_harness 개발 시 사용)
- Mavis(MiniMax Code) + `minimax-code` 오버레이 + `standard_ai_workflow` kit
- Mavis 메인 orchestrator + `.MiniMax/agents/` 워커 분화
- **개발할 때만** 사용, my_harness 가 동작할 때는 무관

## v1 핵심 컨셉 (D-22~D-34)

| 결정 ID | 내용 | § |
| --- | --- | --- |
| D-25 | **Mavis zero coupling** — my_harness 는 Mavis 와 100% 독립 | §5.8 |
| D-26 | **standard_ai_workflow 6 원칙 native** + 옵션 Mavis 디렉토리 sync | §5.9 |
| D-27 | **headroom = built-in 압축** (외부 proxy 의존 X) | §3.3, §5.6 |
| D-28 | **Provider 6개** (claude/codex/gemini native + deepseek/minimax/local OpenAI 호환) | §5.5 |
| D-29 | **Agent 3 모드** (orchestrator/single/loop) + 15개 built-in sub-agents | §5.10, §5.11 |
| D-30 | **2-계층 Context 압축** (Layer 1 필수 + Layer 2 opt-in) | §5.6 |
| D-31 | **`~/.myharness/` 디렉토리 구조** (sibling tool 컨벤션) | §5.12 |
| D-32 | **LLM Wiki memory** (Karpathy 패턴, v2+) | §5.13 |
| D-33 | **Skill/MCP first-class** (claude-code/goose 동급) | §5.14 |
| D-34 | **TASK-NNN 형식 통일** + 2.1.169 pending 표 | §6, §11 |

## 디렉토리 구조 (D-31)

```
.
├── MiniMax.md                       # Mavis 진입점 (개발 workflow 한정, B)
├── MiniMax_config.example.json      # Mavis 설정 예시
├── .MiniMax/
│   └── agents/                      # Mavis 워커 정의 (orchestrator/code/doc/validation)
├── docs/
│   ├── CONCEPT.md                   # ★ v1 컨셉 SSOT (A 의 v1 spec)
│   ├── PROJECT_PROFILE.md           # 이 하네스 운영 규칙
│   ├── development_log.md           # 결정 이력 (D-01~D-42)
│   ├── REFERENCES.md                # 5 reference 1차 비교표 (TASK-004 1차, superseded)
│   └── references/                  # 7 reference 심층분석 + cross-review
│       ├── README.md                # 7-doc 통합 인덱스 + 8축 비교 매트릭스
│       ├── ANALYSIS_PLAN.md         # 14섹션 템플릿
│       ├── PROVIDERS.md             # LLM provider 추상화 비교
│       ├── codex.md / aider.md / goose.md / opencode.md / gemini-cli.md
│       ├── claude-code.md           # 7번째 (closed source 분석)
│       └── headroom.md              # 6번째 (context compression)
├── ai-workflow/                     # standard_ai_workflow kit (B 의 workflow 표준)
│   ├── core/                        # 워크플로우 코어 (global_workflow_standard 등)
│   ├── memory/                      # state.json / handoff / backlog
│   ├── skills/                      # 워크플로우 스킬
│   ├── scripts/                     # bootstrap / state 생성
│   └── tests/                       # 스모크 테스트
└── README.md                        # 이 파일

# my_harness v1 출시 시 (TASK-005-1)
# 신규 추가 영역 (v1 산출물 source):
#   myharness/                        # Python 또는 Rust 패키지
#   ├── cli/                          # CLI entry + argparse
#   ├── llm/                          # provider 비종속 LLM client
#   ├── harness/                      # 5 components
│   ├── agents/                       # 15개 built-in sub-agents
#   ├── skills/                       # built-in skills
#   ├── hooks/                        # markdown rules
#   ├── compression/                  # Layer 1+2 압축
#   └── llm-wiki/                     # LLM Wiki memory
#   tests/
#   pyproject.toml / Cargo.toml
```

## 첫 세션 시작하기 (개발 workflow, B)

1. Mavis 세션을 열고 프로젝트 루트에서 시작.
2. 다음 프롬프트로 워크플로우 세션 활성화:
   > 프로젝트 루트의 `MiniMax.md` 를 읽고, `ai-workflow/memory/state.json` 을 기준으로 워크플로우 세션을 시작해줘.
3. Mavis 가 `MiniMax.md` → `state.json` → `session_handoff.md` → `work_backlog.md` → `docs/PROJECT_PROFILE.md` 순서로 읽고 현재 상태를 복원한다.
4. 첫 실제 작업은 `ai-workflow/memory/backlog/2026-06-05.md` 에 TASK 추가하고 `state.json` 을 재생성한다.

## v1 산출물 CLI 명령 (A, TASK-005-1 출시 후)

```bash
# 3-도메인 명령 (CONCEPT.md §5.2)
myharness code review <pr-url>          # code-reviewer sub-agent
myharness code implement "<feature>"    # code-implementer sub-agent
myharness code test <path>              # code-tester sub-agent
myharness code commit "<message>"       # git-operator sub-agent

myharness server status [host]          # server-status sub-agent
myharness server logs <service> [N]     # log-analyzer sub-agent
myharness server deploy <env>           # deployer sub-agent
myharness server config <action>        # config-manager sub-agent

myharness env setup <stack>             # env-setup sub-agent
myharness env install <pkgs>            # env-installer sub-agent
myharness env shell <cmd>               # env-shell sub-agent
myharness env diagnose                  # env-diagnose sub-agent

# Agent 모드 (CONCEPT.md §5.10)
myharness --mode=orchestrator ...       # default
myharness --mode=single ...             # 단일 에이전트
myharness --mode=loop --goal "fix all TODOs" --max-iterations=20 .

# 워크플로우 명령 (CONCEPT.md §5.9, standard_ai_workflow 호환)
myharness task start --id TASK-005 --title "스택 결정"
myharness task end --id TASK-005 --status done --summary "..." --risks "..." --follow-up "..."
```

## 개발 워크플로우 명령 (B)

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

**v1 컨셉 (산출물 spec)**:
- ★ [`docs/CONCEPT.md`](./docs/CONCEPT.md) — my_harness v1 SSOT (12 sections)
- [`docs/development_log.md`](./docs/development_log.md) — 결정 이력 (D-01~D-42)
- [`docs/references/README.md`](./docs/references/README.md) — 7 reference 통합 인덱스

**레퍼런스 분석**:
- [`docs/REFERENCES.md`](./docs/REFERENCES.md) — 5 reference 1차 (TASK-004 1차, superseded)
- [`docs/references/codex.md`](./docs/references/codex.md), `aider.md`, `goose.md`, `opencode.md`, `gemini-cli.md`
- [`docs/references/claude-code.md`](./docs/references/claude-code.md) — closed source 분석 (D-21)
- [`docs/references/headroom.md`](./docs/references/headroom.md) — context compression (D-15)
- [`docs/references/PROVIDERS.md`](./docs/references/PROVIDERS.md) — LLM provider 비교

**Mavis / 워크플로우 (개발 시)**:
- [MiniMax.md](./MiniMax.md) — Mavis 진입점
- [docs/PROJECT_PROFILE.md](./docs/PROJECT_PROFILE.md) — 하네스 운영 규칙
- [ai-workflow/core/global_workflow_standard.md](./ai-workflow/core/global_workflow_standard.md) — 워크플로우 코어 표준
- [ai-workflow/README.md](./ai-workflow/README.md) — Kit 사용 가이드

**외부**:
- 원본 프레임워크: https://github.com/ykylee/standard_ai_workflow

## 다음에 정해야 할 것 (TASK-NNN)

| task_id | 결정 | 보류 이유 |
| --- | --- | --- |
| **TASK-002** | 도메인별 명령 가이드 | yklee 인프라 정보 필요 |
| **TASK-005** | 스택 (Rust 1안 vs TypeScript 2안) | yklee 의 desktop 우선순위 |
| **TASK-006** | TUI 라이브러리 (ratatui vs React/Ink) | TASK-005 결정 의존 |
| **TASK-007** | headroom built-in 알고리즘 우선순위 | yklee 우선순위 결정 |
| **TASK-008** | Provider fallback list (3 모델) | yklee 의 LLM 선호/비용 |
| **D-42** | config 포맷 = TOML | (yklee 결정, 2026-06-09, W1 prerequisite 검증 중) |

`MiniMax.md` 의 TODO 명령 5종 (설치 / 로컬 실행 / 빠른 테스트 / 격리 테스트 / 실행 확인) — 실제 my_harness 운영 명령으로 채우기 (TASK-002 후)
