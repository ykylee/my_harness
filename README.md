# my_harness

- **yklee의 3-도메인 하네스 래퍼** — `myharness code|server|env …` 가 사용자 표면
- **엔진**: 설치된 [`grok`](./docs/references/grok-build.md) (Grok Build ≥ 1.0.3). 5 components 재구현 안 함
- **확장**: grok plugin (`plugins/myharness/`, `--plugin-dir`)
- **LLM**: grok `[model.minimax]` / `[model.ollama]` (`chat_completions`)
- **기반 표준**: [`standard_ai_workflow`](https://github.com/ykylee/standard_ai_workflow) 6 원칙은 **래퍼**가 지킴
- **컨셉 SSOT**: [`docs/CONCEPT.md`](./docs/CONCEPT.md) · 설계: [`docs/architecture/DETAILED_DESIGN_OVERLAY.md`](./docs/architecture/DETAILED_DESIGN_OVERLAY.md)
- **결정**: D-135 (2026-08-14) overlay. 구 standalone Rust MVP 는 v0 참고 구현

## 이 저장소의 두 가지 역할

이 저장소는 **두 가지 역할**을 동시에 한다 (D-25 명확화):

### A. **my_harness 산출물** 의 source tree (D-135 overlay)
- thin CLI 래퍼 + `plugins/myharness/` (grok `plugin.json`)
- spec: [`docs/CONCEPT.md`](./docs/CONCEPT.md) §0 · §5.1 · [DETAILED_DESIGN_OVERLAY.md](./docs/architecture/DETAILED_DESIGN_OVERLAY.md)
- 엔진 5 components 는 grok. 우리는 3-도메인 동사 + plugin + task/handoff
- `myharness/` Rust crates = v0 참고 구현 (신규 기능 금지)

### B. **이 repo 의 개발 workflow** (D-25, yklee 가 my_harness 개발 시 사용)
- Mavis(MiniMax Code) + `minimax-code` 오버레이 + `standard_ai_workflow` kit
- Mavis 메인 orchestrator + `.MiniMax/agents/` 워커 분화
- **개발할 때만** 사용, my_harness 가 동작할 때는 무관

## v1 핵심 컨셉 (D-22~D-34)

| 결정 ID | 내용 | § |
| --- | --- | --- |
| **D-135** | **제품 경로 = grok overlay** (래퍼+plugin). 자체 런타임 폐기 | §0, §5.1 |
| D-25 | **Mavis zero coupling** — 개발 workflow 와 산출물 분리 (유지) | §5.8 |
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
│   ├── CONCEPT.md                   # ★ 컨셉 SSOT (D-135 overlay)
│   ├── PROJECT_PROFILE.md           # 이 하네스 운영 규칙
│   ├── development_log.md           # 결정 이력 (D-01~D-42)
│   ├── REFERENCES.md                # 5 reference 1차 비교표 (TASK-004 1차, superseded)
│   └── references/                  # 7 reference 심층분석 + cross-review
│       ├── README.md                # 8-doc 통합 인덱스 + 8축 비교 매트릭스
│       ├── grok-build.md            # 8번째. D-135 엔진 실측
│       ├── ANALYSIS_PLAN.md         # 14섹션 템플릿
│       ├── PROVIDERS.md             # LLM provider 추상화 비교 (v0)
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

# overlay 산출물 (D-135, 다음 PR)
#   plugins/myharness/                # grok plugin.json + skills/agents/hooks
#   bin/myharness                     # thin 래퍼 (1차 셸 → Rust)
# myharness/                          # v0 crates — 참고만, 신규 기능 금지
```

## 첫 세션 시작하기 (개발 workflow, B)

1. Mavis 세션을 열고 프로젝트 루트에서 시작.
2. 다음 프롬프트로 워크플로우 세션 활성화:
   > 프로젝트 루트의 `MiniMax.md` 를 읽고, `ai-workflow/memory/state.json` 을 기준으로 워크플로우 세션을 시작해줘.
3. Mavis 가 `MiniMax.md` → `state.json` → `session_handoff.md` → `work_backlog.md` → `docs/PROJECT_PROFILE.md` 순서로 읽고 현재 상태를 복원한다.
4. 첫 실제 작업은 `ai-workflow/memory/backlog/2026-06-05.md` 에 TASK 추가하고 `state.json` 을 재생성한다.

## 산출물 CLI 명령 (A, overlay — CONCEPT.md §5.2)

엔진은 PATH 의 `grok` ≥ 1.0.3. 래퍼는 `--plugin-dir plugins/myharness` 를 붙인다.

```bash
# 설치 (엔진)
curl -fsSL https://x.ai/cli/install.sh | bash

# 이 저장소
./bin/myharness --help
./bin/myharness --print-cmd env diagnose   # 번역만
./scripts/overlay_smoke.sh                 # M1 검증
# ./bin/myharness env diagnose             # 실제 grok -p (LLM)
```

계획: [`docs/architecture/OVERLAY_IMPLEMENTATION_PLAN.md`](./docs/architecture/OVERLAY_IMPLEMENTATION_PLAN.md) (M1 done, 다음 M2 MiniMax + skills + hooks).

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

| task_id | 결정 | 보류 이유 / 상태 |
| --- | --- | --- |
| **TASK-002** | 도메인별 명령 가이드 | yklee 인프라 정보 필요 (서버 호스트 / SSH / B / dotfiles / asdf-mise) — **별도 세션 trigger 후 진행**. 코드 개발 5 TODO 는 본 PR 에서 자동 채움 완료. |
| **TASK-005** | 스택 (Rust 1안 vs TypeScript 2안) | **결정 완료 — Rust 1안** (D-36, 2026-06-07, `docs/development_log.md` §5 정식 기록). PROJECT_PROFILE.md §1 + §3.1 의 적용 환경, Cargo workspace 기준 표준 명령 패턴. TASK-005-1 W2 (`myharness/` workspace init) 진행 중. 의존성: `rig-core = "0.38"`, `rmcp = "1.7"`, `ratatui`, `keyring`, `cargo-dist`. |
| **TASK-006** | TUI 라이브러리 (ratatui vs React/Ink) | **결정 완료 — ratatui** (D-36 의 TASK-005 Rust 1안 정합 자동 확정, 2026-06-07). v0.1.0 부터 `myharness/crates/tui/` 진행. |
| **TASK-007** | headroom built-in 알고리즘 우선순위 | yklee 우선순위 결정 |
| **TASK-008** | Provider fallback list (3 모델) | yklee 의 LLM 선호/비용 |
| **D-42** | config 포맷 = TOML | (yklee 결정, 2026-06-09, W1 prerequisite 검증 중) |

`MiniMax.md` / `AGENTS.md` 의 코드 개발 TODO 5종 (설치 / 로컬 실행 / 빠른 테스트 / 격리 테스트 / 실행 확인) — 본 PR 에서 `cargo build/test --manifest-path myharness/Cargo.toml --workspace` 패턴으로 자동 채움 완료. **서버 관리 / 환경 셋업** 의 TODO 는 yklee 인프라 정보 별도 세션 trigger 후 진행.
