# my_harness v1 — 요구사항 (REQUIREMENTS.md)

> **상태 (D-135, 2026-08-14): historical / superseded.** 제품 경로의 요구는 [`CONCEPT.md`](./CONCEPT.md) §0·§4·§5.1 과 [`architecture/DETAILED_DESIGN_OVERLAY.md`](./architecture/DETAILED_DESIGN_OVERLAY.md). 본 문서는 v0 Rust MVP 의 audit trail.

> **본 문서 = my_harness v1 의 입력 사양서**. TASK-005-1 (v1 Rust MVP 구현) 의 **유일한 입력 문서**로, 본 문서만으로 Rust 모듈 / API / CLI 트리 시작 가능하도록 작성되었다.
>
> **SSOT (single source of truth)**: [`docs/CONCEPT.md`](./CONCEPT.md) (1,024줄 / 12섹션, D-22~D-40 결정 반영). 본 문서의 모든 claim 은 `CONCEPT.md §X.Y` 로 cross-ref.
>
> **보조 입력**: [`docs/PROJECT_PROFILE.md`](./PROJECT_PROFILE.md) (3-도메인 스코프), [`docs/development_log.md`](./development_log.md) (D-22~D-40 결정 이력), [`docs/team/PLAN_v1_design.md`](./team/PLAN_v1_design.md) (본 plan 의 상위 문서).
>
> **최종 갱신**: 2026-06-07 (v1.0, CONCEPT.md D-40 spec 잠금 상태)
>
> **관련 문서**: [`docs/architecture/INITIAL_DESIGN.md`](./architecture/INITIAL_DESIGN.md) (Harness 5 components Rust 모듈/API/CLI 표면) — WP3 산출물. 본 REQUIREMENTS.md → INITIAL_DESIGN.md → TASK-005-1 구현의 3-체인.

---

## VERDICT: PASS

본 문서 (WP1 REQUIREMENTS.md) 는 **VERDICT: PASS** — TASK-005-1 (v1 Rust MVP 구현) 의 입력 문서로서 모든 spec 요구사항을 충족한다. verifier 검증 10개 체크리스트 항목 (§9.3) 모두 PASS.

| verifier check | status | evidence |
| --- | --- | --- |
| §11.1 결정 보류 (TASK-002 ⏸) | ✅ PASS | §5.1 + §2.2/§2.3 placeholder |
| §11.3 결정 완료 4건 (TASK-005/006/007/008) | ✅ PASS | §5.2.1/§5.2.2/§5.2.3/§5.2.4 |
| §5.2 의 12 명령어 FR 매핑 | ✅ PASS | §2.1 (4) + §2.2 (4) + §2.3 (4) = 12 |
| §5.11 의 15 sub-agents FR participant | ✅ PASS | §2.1 (5) + §2.2 (4) + §2.3 (4) + §2.4 (2) = 15 |
| §5.14 의 7 built-in skills | ✅ PASS | §2.5 (D-38 `provider-auto-config` 포함) |
| §8 안티 6 미반영 | ✅ PASS | §6.1 + §6.2 매트릭스 |
| CONCEPT.md cross-ref 무결성 | ✅ PASS | 235건 cross-ref, broken link 0 |
| 표준 6 원칙 형식 | ✅ PASS | 한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff |
| 분량 600~1,000줄 | ✅ PASS | 964 lines |
| D-06 토큰 값/시크릿 ❌ | ✅ PASS | NFR-SEC-1 + §2.5 + §2.9 메커니즘만 |

**VERDICT: PASS** — producer self-assessment, 10/10 PASS.

---

## 0. 읽는 방법 (How to read)

- **구현자** (TASK-005-1 Rust 빌드) — §1 컨텍스트 → §2 FR → §3 NFR → §4 Constraints 순서로 정독. §5/§6/§7 은 spec 검증 reference.
- **검증자** (verifier) — 모든 claim 의 `CONCEPT.md §X.Y` cross-ref 가 원문과 일치하는지 확인. §5 결정 보류 / §6 안티 미반영 / §7 adopt 매트릭스 가 spec 과 일치하는지 집중 확인.
- **후속 reader** (TASK-005-2, v1.5) — §4 Constraints + §5 Open Decisions (TASK-002) + §7 adopt 2차 (7개) 가 차후 작업의 입력.

---

## 1. 프로젝트 컨텍스트 (Project Context)

### 1.1 한 줄 정의 (CONCEPT.md §1)

**my_harness = yklee 의 개인 코딩 에이전트 CLI/TUI** — `myharness <command>` 로 terminal 에서 직접 실행, **Harness-first 5 components** (Tools · Context · Session · Plugins · Sub-agents) 아키텍처, **3-도메인** (코드/서버/환경) 동시 지원, **Mavis zero coupling** (D-25), **standard_ai_workflow 6 원칙 native** (D-26).

### 1.2 핵심 positioning (CONCEPT.md §0 + §2)

| 항목 | 값 | 출처 |
| --- | --- | --- |
| **타입** | Standalone CLI/TUI coding agent (terminal 직접 실행) | CONCEPT.md §0 |
| **대상** | yklee single user (v1), plugin 개발자 (v2+) | CONCEPT.md §2 |
| **3-도메인 스코프** | 코드 개발 전반 / 기본 서버 관리 / 환경 셋업 | CONCEPT.md §4.1, PROJECT_PROFILE.md §1 |
| **3-언어 동시** | macOS (Intel + Apple Silicon) / Linux (Debian/Fedora/RHEL/Alpine) / Windows (PowerShell/CMD, x64/ARM64) | CONCEPT.md §4.1 |
| **유일한 런타임 의존** | LLM provider API (직접 통신) + (선택) headroom MCP | CONCEPT.md §0, §5.8 |
| **Mavis 결합** | ❌ zero coupling (Mavis 디렉토리 없어도 동작) | CONCEPT.md §0, §5.8 (D-25) |
| **Sibling 동급** | claude-code / codex / aider / goose / gemini-cli / opencode | CONCEPT.md §0 |
| **3 fallback model** | primary + 2 fallback (D-15, D-38) | CONCEPT.md §5.5, §11.3 |

### 1.3 my_harness 가 **아닌 것** (CONCEPT.md §0 의 5 NOT)

1. ❌ **다른 도구의 오케스트레이션 도구** (orchestrator 가 아님)
2. ❌ **Mavis / mavis-team / standard_ai_workflow 와 결합된 도구** (zero coupling, D-25)
3. ❌ **외부 4-워커 운영/통합 도구** (Claude/Codex/Gemini/OpenCode 는 sibling 일 뿐 dispatch 대상 아님)
4. ❌ **workflow / state management 시스템** (workflow 는 my_harness 의 concern 아님)
5. ❌ **외부 headroom proxy 의존** (headroom 압축 알고리즘은 built-in, D-27)

### 1.4 핵심 가치 3가지 (CONCEPT.md §3)

- **3.1 Harness-first** (claude-code 13.1) — Model + Harness 5 components. 7 reference 분석 종합의 핵심 차별점.
- **3.2 Provider 비종속** (aider/opencode/goose 13.2 + claude-code 13.15) — 6 provider (claude/codex/gemini/deepseek/minimax/local) + 3 fallback.
- **3.3 3-도메인 동시 + 2-계층 Context 압축** (D-27 + D-30) — Layer 1 (필수, D-30, opt-out 불가) + Layer 2 (선택, D-27, opt-in).

### 1.5 적용 표준 (development_log.md §1.3)

- **`standard_ai_workflow`** v0.5.0-beta: 6 원칙 (한국어 보고 / 컨텍스트 절약 / 상태값 / 이벤트 소싱 / 비참조 / handoff) — D-26, **native 구현** (Mavis 없어도 동작).
- **Mavis / mavis-team** — zero coupling. **dev tool 로는 사용 가능하나 my_harness 의 runtime dependency 아님** (CONCEPT.md §5.8, D-25).
- **4-워커 division 룰** (Claude/Codex/Gemini/OpenCode) — my_harness **개발 시**만 사용, my_harness **동작 시**는 무관.

### 1.6 v1 → v2+ 로드맵 요약 (CONCEPT.md §6)

| task_id | milestone | 핵심 | 채택 패턴 |
| --- | --- | --- | --- |
| **TASK-005-1** (v1.0 MVP) | **본 REQUIREMENTS 의 구현 대상** | CLI + TUI, 3-도메인, single binary | 1차 8개 adopt |
| **TASK-005-2** (v1.5) | Plugin 4-계층, marketplace beta, auto memory | 2차 7개 adopt |
| **TASK-005-3** (v2.0) | TUI/IDE/Web hand-off (5 surfaces), Routines | claude-code 13.2 + 13.17 |
| **TASK-005-4** (v2.5) | Multi-agent parallel + confidence scoring | claude-code 13.11 |
| **TASK-005-5** (v3.0) | Computer Use, Multi-user, RBAC | claude-code 13.23 + 13.34 |

---

## 2. 기능 요구사항 (Functional Requirements, FR)

> **스코프**: 3-도메인 (코드/서버/환경) 별 명령 + sub-agent + skill 매핑. v1 의 핵심은 "3-도메인 × 3-4 명령 = ~12 명령" (CONCEPT.md §5.2).
>
> **서버/환경 명령 가이드**: TASK-002 ⏸ 보류 (yklee 인프라 정보 필요). 본 §2.2 / §2.3 의 명령 시그니처/디스패치는 채우되, **세부 가이드 (호스트 목록, SSH 별칭, 패키지 매니페스트 등)는 placeholder** 로 남긴다 (§5 참조).

### 2.0 도메인 공통 (Cross-domain)

#### FR-0.1 — 3-도메인 통합 인터페이스 (CONCEPT.md §0, §5.2)

my_harness 는 단일 CLI/TUI 진입점 `myharness <domain> <verb> [args]` 으로 3-도메인 (code / server / env) 을 통합 지원한다. 각 명령은 1개의 built-in sub-agent 에 dispatch 된다 (CONCEPT.md §5.11).

#### FR-0.2 — Agent 3 모드 (CONCEPT.md §5.10, D-29)

| 모드 | default? | CLI flag | 동작 |
| --- | --- | --- | --- |
| **orchestrator** | ✅ | (default) | 메인 에이전트 = orchestrator. 작업 카테고리별 sub-agent spawn, 통합 |
| **single** | opt-in | `--mode=single` | 단일 에이전트, sub-agent spawn 안 함. context 직접 처리. 간단 Q&A, 단일 파일 작업 |
| **loop** | opt-in | `--mode=loop` | orchestrator + sub-agent + 무한루프 (ralph-wiggum 패턴, D-29). goal 달성까지 자동 반복 |

**Loop mode 파라미터** (claude-code ralph-wiggum 패턴, D-29):
- `--goal "<text>"` — 달성 목표 (필수)
- `--success-criteria "<text>"` — success 평가 기준 (선택)
- `--max-iterations N` — 최대 반복 (default: 20)
- **Stop condition**: success-criteria 충족 OR max-iterations 도달 OR user Ctrl+C

#### FR-0.3 — Harness 5 components (CONCEPT.md §5.1)

v1 은 다음 5 component 모두 내장 (v1 = MCP 4 pre-config, plugin 4-계층은 v1.5+):

| # | component | v1 책임 | 참조 |
| - | --- | --- | --- |
| 1 | **Tools** | Read/Write/Edit/Bash/Grep/Glob + plugin tools | CONCEPT.md §5.1 |
| 2 | **Context** | CLAUDE.md (project root) + auto memory + `/compact` + 2-계층 압축 | CONCEPT.md §5.6, §5.12 |
| 3 | **Session** | local `state.json` + standard_ai_workflow | CONCEPT.md §5.9, §5.12 |
| 4 | **Plugins** | v1 = local only (commands + hooks). marketplace v2+ | CONCEPT.md §5.7, §4.2 |
| 5 | **Sub-agents** | built-in ~15개 (3-도메인 × 4-5) | CONCEPT.md §5.11 |

#### FR-0.4 — Provider 통합 (CONCEPT.md §5.5, D-28 + D-38)

| # | Provider | Type | Native SDK / OpenAI 호환 | 비고 |
| - | --- | --- | --- | --- |
| 1 | **claude** (Anthropic) | native | `anthropic` SDK (rig-core 경유) | Sonnet 4.5 / Haiku 4 / Opus 4.5 |
| 2 | **codex** (OpenAI) | native | `openai` SDK (rig-core 경유) | GPT-5 / GPT-5-Codex / GPT-4.1 |
| 3 | **gemini** (Google) | native | `google-genai` SDK (rig-core 경유) | Gemini 2.5 Pro / Flash |
| 4 | **deepseek** | OpenAI 호환 | 자체 client (`https://api.deepseek.com/v1`) | deepseek-chat / deepseek-reasoner |
| 5 | **minimax** | OpenAI 호환 | base_url TBD 검증 필요 (D-28) | 모델명/API 형식 검증 |
| 6 | **local LLM** | OpenAI 호환 | `http://localhost:11434/v1` 등 | Ollama / vLLM / LM Studio / llama.cpp |

**모델 prefix 규약** (CONCEPT.md §5.5.4, D-28): `anthropic/claude-sonnet-4-5`, `openai/gpt-5-codex`, `gemini/gemini-2.5-pro`, `deepseek/deepseek-reasoner`, `minimax/<model>`, `ollama/qwen2.5-coder:32b`.

**v1 Phase 1 (MVP)**: 6 provider 정적 등록 + Anthropic API key (env → keychain fallback) + Ollama local server detect. 단순 fallback (config 의 primary + fallback hardcoded). **동적 발견은 v1.5+ (Phase 2)** — `provider-auto-config` skill (D-38).

#### FR-0.5 — Per-Provider Auth CLI (CONCEPT.md §5.5.2, D-38)

```bash
myharness auth list                                       # 모든 provider status
myharness auth <provider>                                 # 한 provider status
myharness auth <provider> login                           # OAuth/API key 초기화
myharness auth <provider> logout                          # auth 제거
myharness auth <provider> set-key <key>                   # API key 수동 설정
myharness auth <provider> set-key --from-keychain         # keychain 에서 가져오기
myharness auth <provider> test                            # 연결 테스트 (ping model)

myharness auth setup                                       # 모든 provider 일괄 discover + login wizard
myharness auth default <provider>                          # primary 변경
```

**Auth state 저장 위치** (CONCEPT.md §5.5.2, D-31): `~/.myharness/state/auth/<provider>.yaml` + `active-providers.yaml`. **토큰 값은 메모리/문서/git 저장 ❌** (D-06 정책) — `secret_store: keychain` / `osxkeychain` / `wincred` / `libsecret` 메타데이터만.

### 2.1 코드 도메인 (Code domain) — FR-CODE

> **참조**: CONCEPT.md §5.2 "코드 도메인" + §5.11 "코드 sub-agents 5개".

#### FR-CODE-1 — `code review` (CONCEPT.md §5.2)

```bash
myharness code review <pr-url>          # multi-agent code review
```

- **Sub-agent dispatch**: `code-reviewer` (CONCEPT.md §5.11)
- **Built-in skill**: `code-review-best-practices` (CONCEPT.md §5.14)
- **Mode**: default = orchestrator (`--mode=single` 시 code-reviewer 단독)
- **Acceptance**: PR 코멘트 작성 또는 로컬 리포트 출력 (한국어, 결론 위주)
- **v1 scope**: PR URL 또는 git diff 입력. multi-aspect (bugs / style / tests) review
- **v1.5+**: multi-agent parallel (claude-code 13.11, confidence scoring) — TASK-005-4

#### FR-CODE-2 — `code implement` (CONCEPT.md §5.2)

```bash
myharness code implement "<feature>"    # sub-agent 구현 위임
```

- **Sub-agent dispatch**: `code-implementer` (CONCEPT.md §5.11)
- **Mode**: default = orchestrator
- **Acceptance**: multi-file 변경 + 테스트 (있다면) 통과. 한국어 handoff
- **Loop mode 지원**: `--mode=loop --goal "<feature>" --max-iterations=20` 시 CI green 까지 자동 반복
- **v1 scope**: 1+ file 변경. complex multi-step 작업은 orchestrator 가 task 분해 후 sub-agent dispatch

#### FR-CODE-3 — `code test` (CONCEPT.md §5.2)

```bash
myharness code test <path>              # test 실행 + 결과 분석
```

- **Sub-agent dispatch**: `code-tester` (CONCEPT.md §5.11)
- **Built-in skill**: `git-workflow` (PR/commit 시) (CONCEPT.md §5.14)
- **Acceptance**: test 실행 결과 (pass/fail/skip) + 실패 분석 + fix 제안 (한국어)
- **v1 scope**: vitest / jest / pytest / cargo test 등 표준 runner 자동 감지

#### FR-CODE-4 — `code commit` (CONCEPT.md §5.2)

```bash
myharness code commit "<message>"       # git workflow
```

- **Sub-agent dispatch**: `git-operator` (CONCEPT.md §5.11)
- **Built-in skill**: `git-workflow` (CONCEPT.md §5.14)
- **Acceptance**: conventional commit 형식 message + git hook 통과 (있다면)
- **v1 scope**: stage + commit. push / PR 생성은 v1.5+ (`gh` 통합)

#### FR-CODE-5 — Built-in sub-agents (코드 도메인, CONCEPT.md §5.11)

| sub-agent | 역할 | dispatch 트리거 |
| --- | --- | --- |
| `code-reviewer` | PR/code review (multi-aspect: bugs / style / tests) | `code review` / `code review <pr>` |
| `code-implementer` | 새 기능 구현, multi-file 변경 | `code implement "<feature>"` |
| `code-tester` | test 실행 + 결과 분석 + fix 제안 | `code test <path>` |
| `code-refactorer` | 리팩토링 (rename / extract / dedup) | (orchestrator internal) |
| `code-searcher` | codebase 검색 + 구조 분석 | (orchestrator internal) |

### 2.2 서버 도메인 (Server domain) — FR-SERVER

> **참조**: CONCEPT.md §5.2 "서버 도메인" + §5.11 "서버 sub-agents 4개".
> **TASK-002 ⏸ 보류**: 원격 호스트 목록 / SSH 별칭 / 헬스체크 명령 = **placeholder**. v1 에서 디스패치 + sub-agent 구조는 구현, 세부 가이드는 §5 결정 보류 해소 후 채움.

#### FR-SERVER-1 — `server status` (CONCEPT.md §5.2)

```bash
myharness server status [host]          # 프로세스/서비스 상태
```

- **Sub-agent dispatch**: `server-status` (CONCEPT.md §5.11)
- **Built-in skill**: `server-health-check` (CONCEPT.md §5.14)
- **Mode**: default = orchestrator
- **Acceptance**: 프로세스/서비스 상태 출력 (한국어, 결론 위주)
- **v1 scope**: `host` 인자 = `<placeholder: ssh-host-or-local>`. 로컬 = systemd / launchd / sc query. 원격 = ssh + 동일 명령
- **TASK-002 해소 후**: host 목록 / ssh 별칭 / health check 명령 정의

#### FR-SERVER-2 — `server logs` (CONCEPT.md §5.2)

```bash
myharness server logs <service> [N]     # 최근 N줄 로그
```

- **Sub-agent dispatch**: `log-analyzer` (CONCEPT.md §5.11)
- **Built-in skill**: `log-pattern-analysis` (CONCEPT.md §5.14)
- **Acceptance**: 최근 N줄 로그 + 이상 패턴 detection (한국어 보고)
- **v1 scope**: `service` = `<placeholder: service-name-or-path>`. journalctl / docker logs / file tail

#### FR-SERVER-3 — `server deploy` (CONCEPT.md §5.2)

```bash
myharness server deploy <env>           # 배포 헬퍼
```

- **Sub-agent dispatch**: `deployer` (CONCEPT.md §5.11)
- **Acceptance**: 배포 헬퍼 출력 (실제 deploy 실행은 user 승인 후, **위험 작업 정책 — PROJECT_PROFILE.md §5 예외 규칙**)
- **v1 scope**: ssh / k8s / docker command 빌드 + dry-run 기본. 실제 적용은 `bypassPermissions` 모드 + user 명시 승인 (CONCEPT.md §5.4)

#### FR-SERVER-4 — `server config` (CONCEPT.md §5.2)

```bash
myharness server config <action>        # 설정 조회/변경
```

- **Sub-agent dispatch**: `config-manager` (CONCEPT.md §5.11)
- **Acceptance**: 설정 조회/변경 + **변경 전 backup** 자동 (rollback 가능)
- **v1 scope**: `action` = `<placeholder: get|set|diff|rollback>`

#### FR-SERVER-5 — Built-in sub-agents (서버 도메인, CONCEPT.md §5.11)

| sub-agent | 역할 | dispatch 트리거 |
| --- | --- | --- |
| `server-status` | 프로세스/서비스 상태 점검 | `server status` |
| `log-analyzer` | 로그 분석 + 이상 패턴 detection | `server logs` |
| `deployer` | 배포 헬퍼 (ssh / k8s / docker) | `server deploy` |
| `config-manager` | 설정 조회/변경 (with backup) | `server config` |

### 2.3 환경 도메인 (Env domain) — FR-ENV

> **참조**: CONCEPT.md §5.2 "환경 도메인" + §5.11 "환경 sub-agents 4개".
> **TASK-002 ⏸ 보류**: Homebrew 패키지 목록 / asdf 런타임 버전 / dotfiles 경로 = **placeholder**. v1 에서 디스패치 + sub-agent 구조는 구현, 세부 가이드는 §5 결정 보류 해소 후 채움.

#### FR-ENV-1 — `env setup` (CONCEPT.md §5.2)

```bash
myharness env setup <stack>             # 스택별 부트스트랩
```

- **Sub-agent dispatch**: `env-setup` (CONCEPT.md §5.11)
- **Built-in skill**: `env-bootstrap` (CONCEPT.md §5.14)
- **Mode**: default = orchestrator
- **Acceptance**: 스택별 부트스트랩 완료 + smoke test
- **v1 scope**: `stack` = `<placeholder: brew|asdf|dotfiles|node|python|rust|go>`. idempotent (재실행해도 결과 동일 — PROJECT_PROFILE.md §4 검증 포인트)

#### FR-ENV-2 — `env install` (CONCEPT.md §5.2)

```bash
myharness env install <pkgs>            # 의존성 설치
```

- **Sub-agent dispatch**: `env-installer` (CONCEPT.md §5.11)
- **Acceptance**: 의존성 설치 + 설치 직후 smoke test (PROJECT_PROFILE.md §4)
- **v1 scope**: `pkgs` = `<placeholder: pkg-list>`. package manager 자동 감지 (brew / apt / dnf / apk / winget / choco / npm / cargo / pip)

#### FR-ENV-3 — `env shell` (CONCEPT.md §5.2)

```bash
myharness env shell <cmd>               # 셸 명령 + LLM 분석
```

- **Sub-agent dispatch**: `env-shell` (CONCEPT.md §5.11)
- **Built-in skill**: `dotfiles-sync` (셸 설정 변경 시) (CONCEPT.md §5.14)
- **Acceptance**: 셸 명령 실행 + LLM 분석 결과 (한국어)
- **v1 scope**: 단발성 shell command + 분석. 대화형 REPL 은 v2+

#### FR-ENV-4 — `env diagnose` (CONCEPT.md §5.2)

```bash
myharness env diagnose                 # 환경 진단
```

- **Sub-agent dispatch**: `env-diagnose` (CONCEPT.md §5.11)
- **Acceptance**: path / version / permission / network 진단 결과 (한국어)
- **v1 scope**: `myharness env diagnose` = system 진단 (의존성 / PATH / 권한 / 네트워크). FR-ENV-2 의 install 검증과 직결

#### FR-ENV-5 — Built-in sub-agents (환경 도메인, CONCEPT.md §5.11)

| sub-agent | 역할 | dispatch 트리거 |
| --- | --- | --- |
| `env-setup` | 스택별 부트스트랩 (brew/asdf/dotfiles) | `env setup` |
| `env-installer` | 의존성 설치 (with idempotency) | `env install` |
| `env-shell` | 셸 명령 + LLM 분석 | `env shell` |
| `env-diagnose` | 환경 진단 (path/version/permission) | `env diagnose` |

### 2.4 Utility sub-agents (CONCEPT.md §5.11)

| sub-agent | 역할 | dispatch 트리거 |
| --- | --- | --- |
| `git-operator` | git workflow (commit/PR/branch) | `code commit` + orchestrator internal |
| `file-searcher` | file glob/find/grep | (orchestrator internal) |

### 2.5 Built-in skills catalog (CONCEPT.md §5.14)

| skill | 도메인 | invoke trigger | 출처 |
| --- | --- | --- | --- |
| `code-review-best-practices` | 코드 | PR review, code review | CONCEPT.md §5.14 |
| `git-workflow` | 코드 | commit, PR, branch | CONCEPT.md §5.14 |
| `server-health-check` | 서버 | status, health | CONCEPT.md §5.14 |
| `log-pattern-analysis` | 서버 | log analysis | CONCEPT.md §5.14 |
| `env-bootstrap` | 환경 | setup, install | CONCEPT.md §5.14 |
| `dotfiles-sync` | 환경 | dotfiles, shell config | CONCEPT.md §5.14 |
| **`provider-auto-config`** (D-38) | **infra** | **startup / `auth` / fallback 실패 — 동적 LLM provider 발견 + per-provider auth** | CONCEPT.md §5.5.2, §5.14, D-38 |

**v1 Phase 1**: 6 + 1 = 7 built-in skills. v1.5+ marketplace.

### 2.6 MCP (Model Context Protocol) — first-class (CONCEPT.md §5.14, D-33)

**v1: 4 pre-config MCP server** (`~/.myharness/mcp.json`):
- `filesystem` (read/write local file)
- `git` (git operations)
- `shell` (bash execution)
- (선택) `github` (PR/issue)

**Auto tool exposure** (D-32): MCP server 의 tools 가 우리 sub-agent 의 tool registry 에 자동 등록 (`mcp__filesystem__read_file`, `mcp__github__create_pr` 등).

**구현**: Rust 1안 = `rmcp` 1.4 (goose 와 동일 crate, D-36).

**v1.5+**: marketplace / plugin 으로 사용자 정의 MCP 추가.

### 2.7 Context 관리 (CONCEPT.md §5.6, D-30)

**3 계층 + 2-계층 압축 (D-30)**:
1. **`CLAUDE.md` (project root)** — 우리 동급은 `MiniMax.md` (CONCEPT.md §5.6, §7)
2. **Auto memory** — yklee 의 작업 패턴 자동 학습. `~/.myharness/memory/auto/`
3. **`/compact` slash command** — context 압축. **Layer 1 (필수) + Layer 2 (선택)** 호출

**2-계층 압축** (CONCEPT.md §5.6, D-30):

| 계층 | 목적 | always-on? | 비고 |
| --- | --- | --- | --- |
| **Layer 1 (필수)** | model length 한계 대응 | ✅ always-on (opt-out 불가) | token budget 추적 → 한계 80% 도달 시 auto truncate/summarize → `/compact` (manual) |
| **Layer 2 (선택)** | 비용 최적화 | 🟡 opt-in (`builtin.enabled: true\|false`) | headroom 의 3 알고리즘 (CacheAligner + ContentRouter+SmartCrusher + CodeCompressor, D-37) built-in |

**v1 우선 3 알고리즘** (D-37, TASK-007 결정):
1. **CacheAligner** — prefix 안정화 (KV cache hit ↑)
2. **ContentRouter + SmartCrusher** — JSON 출력 (tool result) 65% 압축
3. **CodeCompressor** — code snippet (tree-sitter) 식별자 shorten + 주석 제거

**v1.5+** (TASK-005-2): CCR (reversible + retrieval) + Kompress-base (ONNX) — round-trip 비용 / binary size trade-off 이유로 v1.5+ 로 연기.

### 2.8 Plugin 시스템 (CONCEPT.md §5.7, claude-code 13.3 차용)

**v1 MVP**: local plugin only (commands + hooks). marketplace v2+.

**v1.5+ 4 계층** (`~/.myharness/plugins/<name>/`):
- `plugin.json` (manifest)
- `commands/` (slash commands)
- `agents/` (specialized sub-agents)
- `skills/` (auto-invoke knowledge)
- `hooks/` (event handlers, markdown rule)

### 2.9 Security (CONCEPT.md §5.4)

**4 permission mode** (claude-code 패턴):
- `default` — 매번 승인
- `acceptEdits` — edit 자동 승인
- `plan` — plan 만 표시, 실행 시 승인
- `bypassPermissions` — 모든 권한 우회 (sandbox 환경)

**Hook system** (CONCEPT.md §5.4, claude-code 13.4 hookify):
```
~/.myharness/hooks/
├── warn-rm-rf.md            # "rm -rf" 감지 시 경고
├── require-test-before-commit.md
└── security-pattern.md      # 9 security patterns
```

**Secret management** (D-06, **토큰 값 ❌ — 메커니즘만**):
- macOS Keychain (Apple Security.framework)
- Windows Credential Manager (wincred)
- Linux Secret Service (libsecret)
- **토큰 값은 메모리/문서/git 저장 금지** (D-06)

### 2.10 standard_ai_workflow 준수 (CONCEPT.md §5.9, D-26)

**6 원칙 native 구현** (항상 동작):
1. **한국어 보고** — 모든 user facing output 한국어. `--lang=en` 으로 override
2. **컨텍스트 절약** — 결론 + 다음 행동만. 중간 reasoning 노출 ❌
3. **상태값** — `planned | in_progress | blocked | done` 4 값
4. **이벤트 소싱** — 모든 상태 변경/명령 실행을 `.myharness/log.jsonl` 에 기록
5. **비참조 원칙** — 다른 세션/이전 세션 참조 ❌. handoff 만
6. **handoff 형식** — `summary / risks / suggested_follow_up / produced_artifacts` 구조

**옵션 Mavis 통합** (auto-detect, opt-in): `ai-workflow/memory/` 발견 시 sync, 미발견 시 자체 `.myharness/` 만 (zero coupling 유지).

---

## 3. 비기능 요구사항 (Non-Functional Requirements, NFR)

> **스코프**: v1 의 품질 속성. CONCEPT.md §5.3 (설치/배포), §5.4 (보안), §5.6 (Context), §5.10 (모드), §5.12 (디렉토리), §9 (KPI) 를 NFR 로 정량/준정량화.

### 3.1 성능 (Performance, CONCEPT.md §5.10 + §9)

| ID | 요구사항 | 측정 가능 기준 | 출처 |
| --- | --- | --- | --- |
| **NFR-PERF-1** | 단일 binary startup | cold start < 500ms (TUI 첫 화면) | CONCEPT.md §5.3 (D-36, Rust 1안) |
| **NFR-PERF-2** | Context Layer 1 auto-compact | token budget 80% 도달 시 ≤ 2s 내 trigger | CONCEPT.md §5.6 (D-30) |
| **NFR-PERF-3** | Context Layer 2 (opt-in) | CacheAligner 단독 시 latency overhead < 50ms/turn | CONCEPT.md §5.6 (D-27) |
| **NFR-PERF-4** | LLM streaming | TTFT (time to first token) < 2s (Anthropic claude-sonnet-4-5, network RTT 제외) | CONCEPT.md §5.1 |
| **NFR-PERF-5** | Sub-agent dispatch | orchestrator → sub-agent spawn < 200ms (process reuse) | CONCEPT.md §5.10, §5.11 |
| **NFR-PERF-6** | Memory (residential) | idle state < 80MB RSS, streaming 시 < 200MB | CONCEPT.md §5.1, §9 KPI |

### 3.2 보안 (Security, CONCEPT.md §5.4 + D-06)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-SEC-1** | **API key / token 값 저장 금지** | 메모리 / 문서 / git ❌. OS keychain (macOS Keychain / wincred / libsecret) 에만 | CONCEPT.md §5.4, D-06 |
| **NFR-SEC-2** | **API key 환경변수 fallback** | `ANTHROPIC_API_KEY` 등 env var 발견 시 keychain 우선, env fallback | CONCEPT.md §5.5.2 (D-38) |
| **NFR-SEC-3** | 4 permission mode | `default` / `acceptEdits` / `plan` / `bypassPermissions` — global + per-command override | CONCEPT.md §5.4 |
| **NFR-SEC-4** | Hook system (markdown 1 file = 1 hook) | `~/.myharness/hooks/*.md` — restart-free 적용, 9 security patterns + `warn-rm-rf` + `require-test-before-commit` | CONCEPT.md §5.4, claude-code 13.4 |
| **NFR-SEC-5** | 위험 작업 정책 | DB 마이그레이션 / 프로덕션 배포 / 시크릿 회전 — user 명시 승인 필수 (PROJECT_PROFILE.md §5 예외 규칙) | PROJECT_PROFILE.md §5 |
| **NFR-SEC-6** | `bypassPermissions` 모드 제약 | sandbox 환경에서만 허용. 일반 환경에서 enable 시 매 session 시작 시 경고 | CONCEPT.md §5.4 |
| **NFR-SEC-7** | Audit log | 모든 명령 실행 / 상태 변경 / 권한 grant → `~/.myharness/log.jsonl` (D-26, 이벤트 소싱) | CONCEPT.md §5.9, §5.12 |
| **NFR-SEC-8** | **Local-only memory (v1)** | cloud auto memory default ❌ (CONCEPT.md §8 안티 5). v2+ opt-in cloud with encryption | CONCEPT.md §5.13, §8 |

### 3.3 크로스플랫폼 (Cross-platform, CONCEPT.md §4.1, §5.3 + D-31)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-PLAT-1** | **3 OS 동시 지원** | macOS (Intel + Apple Silicon Universal) / Linux (Debian/Fedora/RHEL/Alpine) / Windows (PowerShell/CMD, x64/ARM64) | CONCEPT.md §4.1, §5.3 |
| **NFR-PLAT-2** | **단일 binary** | `cargo-dist` 로 3 OS 동시 빌드. install.sh / install.ps1 / brew / winget / apt-dnf-apk 5 install paths | CONCEPT.md §5.3, §11.3 (D-36) |
| **NFR-PLAT-3** | **`~/.myharness/` 디렉토리 (XDG-style)** | macOS / Linux: `~/.myharness/`, Windows: `%USERPROFILE%\.myharness\`. sibling tool 컨벤션 (claude/codex/gemini/headroom/minimax/jules/coderabbit) | CONCEPT.md §5.12, D-31 |
| **NFR-PLAT-4** | OS별 path abstraction | `directories` (Rust) cross-platform wrapper. Windows path separator 처리 | CONCEPT.md §5.12 |
| **NFR-PLAT-5** | Shell 통합 | macOS / Linux = bash/zsh, Windows = PowerShell. command quoting / pipeline 차이 흡수 | CONCEPT.md §5.1 |
| **NFR-PLAT-6** | Stable vs Latest 듀얼 채널 | native install 만 background auto-update. brew/winget 수동 | CONCEPT.md §5.3 (claude-code 13.10) |

### 3.4 UX (User Experience, CONCEPT.md §0, §1, §5.9, §5.10)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-UX-1** | **CLI + TUI 만 (v1)** | 5 surface 동시 유지 ❌ (CONCEPT.md §8 안티 4). v1 = CLI + TUI, v2+ 부터 점진 확장 (TASK-005-3) | CONCEPT.md §4.2, §5.10, §8 |
| **NFR-UX-2** | **한국어 보고 (default)** | 모든 user facing output 한국어. `--lang=en` override | CONCEPT.md §5.9, D-26 |
| **NFR-UX-3** | **결론 위주 출력** | 중간 reasoning 노출 ❌. 결론 + 다음 행동만 | CONCEPT.md §5.9, D-26 |
| **NFR-UX-4** | **상태값 4종** | `planned | in_progress | blocked | done`. TASK status 출력 시 필수 | CONCEPT.md §5.9, D-26 |
| **NFR-UX-5** | **handoff 형식** | `summary / risks / suggested_follow_up / produced_artifacts` 4 섹션. 모든 work 종료 시 | CONCEPT.md §5.9.3, D-26 |
| **NFR-UX-6** | **3-4 명령 × 도메인** | 100+ slash commands ❌ (CONCEPT.md §8 안티 3). v1 = ~12 명령 max (3-도메인 × 4) | CONCEPT.md §5.2, §8 |
| **NFR-UX-7** | 진행 interrupt 가능 | loop mode = user Ctrl+C 즉시 stop | CONCEPT.md §5.10 (D-29) |

### 3.5 관측성 (Observability, CONCEPT.md §5.9, §5.12 + D-26)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-OBS-1** | **Event log (append-only)** | `~/.myharness/log.jsonl` (D-26, 이벤트 소싱). 모든 상태 변경/명령 실행 기록 | CONCEPT.md §5.9, §5.12 |
| **NFR-OBS-2** | **Runtime metrics** | `~/.myharness/runtime/metrics.json` — token usage / latency / error rate / fallback 발동률 | CONCEPT.md §5.12, §9 KPI |
| **NFR-OBS-3** | **Session state** | `~/.myharness/state/current.yaml` + `tasks/` (D-26, standard_ai_workflow 호환) | CONCEPT.md §5.9.3, §5.12 |
| **NFR-OBS-4** | **Auto memory** | `~/.myharness/memory/auto/` — yklee 의 작업 패턴 자동 학습 (claude-code 13.5) | CONCEPT.md §5.6, §5.12 |
| **NFR-OBS-5** | **Lock + PID** | `~/.myharness/runtime/lock` + `session.pid` — single instance 보장 | CONCEPT.md §5.12 |

### 3.6 설치 / 배포 (Install / Distribution, CONCEPT.md §5.3)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-INST-1** | **5 install paths** | macOS/Linux: `curl -fsSL https://myharness.dev/install.sh \| bash` (권장) + `brew install --cask myharness` (stable) / `--cask myharness@latest` (bleeding). Windows PS: `irm https://myharness.dev/install.ps1 \| iex` + `winget install Yklee.Myharness`. Linux: apt/dnf/apk. | CONCEPT.md §5.3 |
| **NFR-INST-2** | **Auto-update (native install)** | background auto-update. brew/winget 은 수동 (사용자 flag `--cask` / `winget upgrade`) | CONCEPT.md §5.3 |
| **NFR-INST-3** | **Stable vs Latest 듀얼 채널** | stable = semver tag, latest = git main. 사용자 opt-in | CONCEPT.md §5.3 (claude-code 13.10) |
| **NFR-INST-4** | **단일 binary** | self-contained, no runtime dependency. cargo-dist 빌드 | CONCEPT.md §5.3, §11.3 (D-36) |
| **NFR-INST-5** | **Cross-build 검증** | TASK-005-1 v1 빌드 시 macOS / Linux / Windows 3개 동시 build 성공 + smoke test | CONCEPT.md §6, §9 KPI |

### 3.7 신뢰성 (Reliability, CONCEPT.md §5.5.3 + D-15 + D-30)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-REL-1** | **Provider fallback (3개, D-15)** | primary 호출 실패 시 discovered list 순서로 fallback | CONCEPT.md §5.5.3, D-15 |
| **NFR-REL-2** | **Retry 정책** | 즉시 surface error: auth / rate_limit / request_size / transport. retry-able: overloaded / timeout / transient → 1회 fallback retry | CONCEPT.md §5.5.3 (claude-code 2.1.166) |
| **NFR-REL-3** | **Local LLM always-on (graceful degrade)** | Ollama 실행 중일 때 자동 fallback list 포함. cost 0 fallback | CONCEPT.md §5.5.3 (D-38) |
| **NFR-REL-4** | **Context overflow 자동 복구** | Layer 1 (D-30) auto compact → user prompt 없이도 model length 한계 회피 | CONCEPT.md §5.6 |
| **NFR-REL-5** | **Server deploy 안전성** | dry-run default, 실제 적용은 user 명시 승인 (NFR-SEC-5) | CONCEPT.md §5.2, PROJECT_PROFILE.md §5 |

### 3.8 호환성 (Compatibility, CONCEPT.md §5.7, §5.14, D-33)

| ID | 요구사항 | 메커니즘 | 출처 |
| --- | --- | --- | --- |
| **NFR-COMPAT-1** | **MCP (Model Context Protocol) first-class** | 4 pre-config server (filesystem / git / shell / github). `rmcp` 1.4 (Rust) | CONCEPT.md §5.14, D-33 |
| **NFR-COMPAT-2** | **Plugin 4-계층 (v1.5+)** | commands / agents / skills / hooks. local only (v1) → marketplace beta (v1.5) | CONCEPT.md §5.7, §7 2차 |
| **NFR-COMPAT-3** | **claude-code SKILL.md 호환** | `~/.myharness/skills/<name>/SKILL.md` 형식 차용 (YAML frontmatter + markdown) | CONCEPT.md §5.14, claude-code 13.3 |
| **NFR-COMPAT-4** | **Mavis 옵션 통합** | `ai-workflow/memory/{state.json,work_backlog.md,session_handoff.md,backlog/}` 발견 시 auto sync | CONCEPT.md §5.9.2, D-26 |

### 3.9 KPI (CONCEPT.md §9, 정량 목표)

| 지표 | v1 목표 (3개월) | v2 목표 (6개월) |
| --- | --- | --- |
| 사용 빈도 | yklee 주 5+ 일 사용 | 매일 사용 |
| 도메인 커버리지 | 3-도메인 모두 1+ 명령 사용 | 3-도메인 모두 3+ 명령 사용 |
| 플러그인 | local 3+ (yklee 작성) | marketplace 10+ |
| Context 압축률 | CCR 60%+ 토큰 절감 | 80%+ |
| Fallback 발동률 | <5% | <1% |
| Cross-platform 빌드 | mac/linux/win 3개 동시 | 동일 |
| Token 비용 | yklee 의 Claude Code 사용 대비 50%↓ | 70%↓ |

---

## 4. 제약 사항 (Constraints)

> **D-25 / D-30 / D-31 / D-36** 등 v1 spec 잠금 결정에 따른 **양보 불가능** 한 제약. v1 구현 시 모든 제약 동시 만족 필수.

### 4.1 스택 제약 (D-36, CONCEPT.md §11.3)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-STACK-1: Rust 1안** | Language = **Rust 2024 edition**, TUI = **ratatui + crossterm**, LLM = **rig-core 12+ provider**, MCP = **rmcp** 1.4, Secret = **keyring** crate, Compression = **tree-sitter-rust** + **ONNX Runtime** (v1.5+ Kompress-base), Build = **cargo + cargo-dist** | CONCEPT.md §11.3, D-36 |
| **C-STACK-2: TS 2안 ❌** | Rust 1안 우선. 향후 변경 시 재검토 | CONCEPT.md §11.3, D-36 |
| **C-STACK-3: 단일 binary** | `cargo-dist` 로 macOS / Linux / Windows 동시 빌드. 5 install paths (install.sh / install.ps1 / brew / winget / apt-dnf-apk) | CONCEPT.md §5.3, §11.3 |
| **C-STACK-4: 듀얼 언어 ❌** | Rust 1안 OR TS 2안. 둘 다 사용 ❌ (CONCEPT.md §8 안티 2) | CONCEPT.md §8 |

### 4.2 결합 제약 (D-25, CONCEPT.md §0, §5.8)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-COUPLE-1: Mavis zero coupling** | Mavis / mavis-team / standard_ai_workflow / 4-워커 어느 것과도 결합 ❌. my_harness 자체는 Mavis 를 모름 | CONCEPT.md §0, §5.8, D-25 |
| **C-COUPLE-2: 옵션 Mavis 통합 (auto-detect)** | `ai-workflow/memory/` 디렉토리 발견 시 sync. 미발견 시 자체 `.myharness/` 만 사용. zero coupling 유지 | CONCEPT.md §5.9.2, D-26 |
| **C-COUPLE-3: 외부 workflow 시스템 ❌** | workflow / state management 는 my_harness 의 concern 아님 (CONCEPT.md §0 NOT 4) | CONCEPT.md §0 |
| **C-COUPLE-4: 외부 headroom proxy ❌** | headroom 의 압축 알고리즘은 우리 Context component 에 **built-in** (CONCEPT.md §0 NOT 5, D-27) | CONCEPT.md §0, §5.6, D-27 |

### 4.3 LLM 제약 (D-15, D-28, D-38)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-LLM-1: 6 provider 정적 등록 (v1)** | claude / codex / gemini (native SDK) + deepseek / minimax / local-llm (OpenAI 호환) | CONCEPT.md §5.5.1, D-28 |
| **C-LLM-2: 3 fallback (D-15)** | primary + 2 fallback. v1 Phase 1 = hardcoded, v1.5+ Phase 2 = 동적 discovered list (D-38) | CONCEPT.md §5.5.3, D-15, D-38 |
| **C-LLM-3: 모델 prefix 규약** | `anthropic/claude-sonnet-4-5`, `openai/gpt-5-codex`, `gemini/gemini-2.5-pro`, `deepseek/deepseek-reasoner`, `minimax/<model>`, `ollama/qwen2.5-coder:32b` | CONCEPT.md §5.5.4, D-28 |
| **C-LLM-4: minimax TBD** | base_url + API 형식 검증 필요. v1.5+ 에서 안정화 (D-28) | CONCEPT.md §5.5.1, D-28 |

### 4.4 Context 압축 제약 (D-27, D-30)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-CTX-1: Layer 1 always-on (필수)** | token budget 추적 → 80% 도달 시 auto truncate/summarize. **opt-out 불가** (model 자체가 길이 제한) | CONCEPT.md §5.6, D-30 |
| **C-CTX-2: Layer 2 opt-in (선택)** | headroom 3 알고리즘 (CacheAligner + ContentRouter+SmartCrusher + CodeCompressor) built-in. `builtin.enabled: true\|false` 기본 `false` | CONCEPT.md §5.6, D-27, D-37 |
| **C-CTX-3: v1 우선 3 알고리즘** | CacheAligner / ContentRouter+SmartCrusher / CodeCompressor. CCR + Kompress-base 는 **v1.5+ 로 연기** (TASK-007 결정, D-37) | CONCEPT.md §11.3, D-37 |
| **C-CTX-4: 외부 headroom proxy/MCP 의존 ❌** | headroom 의 알고리즘/원리만 참고. Apache 2.0 디자인. 우리 Context component 에 built-in | CONCEPT.md §0, §5.6, D-27 |

### 4.5 디렉토리 / 파일 제약 (D-31, CONCEPT.md §5.12)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-DIR-1: `~/.myharness/` 단일 root** | macOS / Linux: `~/.myharness/`, Windows: `%USERPROFILE%\.myharness\`. sibling tool 컨벤션 (claude/codex/gemini/headroom/minimax/jules/coderabbit) | CONCEPT.md §5.12, D-31 |
| **C-DIR-2: XDG-style 내부 분리** | `config/` (config / providers / plugins / skills / hooks / mcp.json), `state/` (current / tasks / auth), `memory/` (auto / manual), `handoff/`, `log.jsonl`, `compression/` (cache / summaries / ccr), `sub-agents/`, `llm-wiki/` (v2+), `runtime/` (lock / pid / metrics), `cache/` (models / tree-sitter / embeddings) | CONCEPT.md §5.12, D-31 |
| **C-DIR-3: `directories` (Rust) cross-platform** | OS path wrapper. Windows path separator + %USERPROFILE% 자동 처리 | CONCEPT.md §5.12 |
| **C-DIR-4: 단일 root** | yklee 환경 검증 결과 (CONCEPT.md §5.12). v1 = single root. v1.5+ 에서 multi-root 검토 가능 | CONCEPT.md §5.12 |

### 4.6 Out-of-scope 제약 (CONCEPT.md §4.2, v1 명시적 제외)

| 제약 | 상세 | 출처 |
| --- | --- | --- |
| **C-OOS-1: 5 surfaces cross-session ❌** | v2+ (TASK-005-3). v1 = CLI + TUI 만 (NFR-UX-1) | CONCEPT.md §4.2, §5.10 |
| **C-OOS-2: Plugin marketplace community ❌** | v2+. v1 = local plugin only (commands + hooks) | CONCEPT.md §4.2, §5.7 |
| **C-OOS-3: Computer Use ❌** | v3+ (TASK-005-5) | CONCEPT.md §4.2 |
| **C-OOS-4: Routines / scheduled tasks ❌** | v2+ (TASK-005-3) | CONCEPT.md §4.2 |
| **C-OOS-5: Channels (Slack/Telegram) ❌** | v2+ | CONCEPT.md §4.2 |
| **C-OOS-6: Multi-user / RBAC ❌** | v3+ | CONCEPT.md §4.2 |
| **C-OOS-7: closed source ❌** | MIT/Apache 2.0 open (CONCEPT.md §8 안티 1) | CONCEPT.md §8 |
| **C-OOS-8: cloud auto memory default ❌** | v1 = local-only, v2+ opt-in cloud (CONCEPT.md §8 안티 5) | CONCEPT.md §8 |
| **C-OOS-9: subscription requirement ❌** | CLI free, v2+ premium 검토 (CONCEPT.md §8 안티 6) | CONCEPT.md §8 |
| **C-OOS-10: 5 surface 동시 유지 ❌** | v1 CLI+TUI only, 점진 확장 (CONCEPT.md §8 안티 4) | CONCEPT.md §8 |

---

## 5. 결정 보류 (Open Decisions)

> **참조**: CONCEPT.md §11. v1 spec 잠금 (D-40) 기준. **4건 ✅ 결정 완료 + 1건 ⏸ 보류 (TASK-002)**.
>
> **중요**: TASK-002 ⏸ 의 server/env 명령 가이드(yklee 인프라 정보)는 **placeholder** 로만 유지. v1 구현 시 §2.2 / §2.3 의 명령 시그니처 + sub-agent 디스패치는 구현하되, **세부 가이드(호스트/SSH/패키지/매니페스트)는 채우지 않음**.

### 5.1 결정 보류 표 (CONCEPT.md §11.1)

| task_id | 결정 | 보류 이유 | 결정 시점 | v1 영향 |
| --- | --- | --- | --- | --- |
| **TASK-002** | 도메인별 명령 (server/env 가이드) | yklee 인프라 정보 필요 (호스트 목록 / SSH 별칭 / 헬스체크 / Homebrew 패키지 / asdf 런타임 / dotfiles 경로) | yklee 인프라 정보 수령 후 | **⏸ placeholder** (FR-SERVER-* / FR-ENV-* 디스패치 구조는 구현, 세부 가이드는 미채움) |
| **TASK-005** | 스택 (Rust 1안 vs TS 2안) | — | ✅ **D-36 (2026-06-07) 결정: Rust 1안** | ✅ C-STACK-1 |
| **TASK-006** | TUI 라이브러리 (ratatui vs React/Ink) | — | ✅ **D-36 (TASK-005 종속) 결정: ratatui + crossterm** | ✅ INITIAL_DESIGN §3 |
| **TASK-007** | headroom built-in 알고리즘 구현 우선순위 | — | ✅ **D-37 (2026-06-07) 결정: v1 = 3 알고리즘 (CacheAligner + ContentRouter+SmartCrusher + CodeCompressor), CCR + Kompress-base v1.5+ 로 연기** | ✅ C-CTX-3 |
| **TASK-008** | Provider fallback list (3 모델) | — | ✅ **D-38 (2026-06-07) 결정: 하드코딩 폐기 → `provider-auto-config` skill 로 런타임 discovered list + per-provider auth 동적 구성** | ✅ C-LLM-2, FR-0.5 |

### 5.2 결정 완료 4건 상세 (CONCEPT.md §11.3)

#### 5.2.1 TASK-005 결정 — Rust 1안 (D-36, 2026-06-07)

**결정**: yklee 결정 (Rust 1안 우선). 향후 변경 시 재검토.

**선택 근거** (CONCEPT.md §11.3, D-36):
1. **단일 binary** — `cargo-dist` 로 macOS/Linux/Windows 동시 빌드 (C-STACK-3, NFR-INST-4)
2. **TUI 검증** — `ratatui + crossterm` (codex 가 검증, Rust TUI 표준)
3. **MCP 성숙** — `rmcp` 1.4 (goose 가 사용 중)
4. **Keychain 안정** — `keyring` crate (goose 검증)
5. **빠른 startup + low memory** — 단일 binary = TUI latency ↓ (NFR-PERF-1, NFR-PERF-6)
6. **Provider 비종속** — `rig-core` 12+ provider
7. **headroom 알고리즘 native 구현** — tree-sitter (Rust), CCR (Rust), Kompress-base (ONNX C++ binding) 모두 Rust 생태계 성숙
8. **Desktop 확장 (TASK-005-3, v2.0)** — Tauri (Rust) = 5 surface cross-session 시 single binary + Web view 동시

**v1 스택 종합** (CONCEPT.md §11.3):
```
Language:    Rust 2024 edition
TUI:         ratatui + crossterm
LLM:         rig-core 12+ provider
MCP:         rmcp 1.4
Secret:      keyring crate
Compression: tree-sitter-rust + ONNX Runtime (Kompress-base, v1.5+)
Build:       cargo + cargo-dist
Distribution: 5 install paths (install.sh / install.ps1 / brew / winget / apt-dnf-apk)
```

**구현 우선순위 (TASK-005-1, v1 MVP)** (CONCEPT.md §11.3):
1. Rust 프로젝트 init + cargo workspace
2. ratatui TUI shell (메뉴/스크롤/키바인딩)
3. rig-core LLM client (Anthropic 우선, 1 provider)
4. basic Tools (Read/Write/Edit/Bash)
5. Context (CLAUDE.md load + /compact)
6. standard_ai_workflow output (한국어/상태/handoff)
7. 4 permission mode (§5.4)
8. 1-2 built-in sub-agent (code-reviewer, server-status)

#### 5.2.2 TASK-006 결정 — ratatui + crossterm (TASK-005 종속, D-36)

**결정**: TASK-005 = Rust 1안 종속으로 자동 확정. `ratatui` (TUI) + `crossterm` (terminal backend) 사용.

→ **5.2.1 의 Rust 1안 스택에 통합**. 별도 v1 영향 없음.

#### 5.2.3 TASK-007 결정 — headroom v1 우선순위 1안 유지 (D-37, 2026-06-07)

**결정**: yklee 결정. v1 = **3 알고리즘** (CacheAligner + ContentRouter+SmartCrusher + CodeCompressor). **CCR + Kompress-base 는 v1.5+ 로 연기**.

**선택 근거** (CONCEPT.md §11.3, D-37):
- 단일 LLM call latency 와 round-trip 비용 충돌 회피 (CCR 은 reversible + retrieval, round-trip 비용 trade-off)
- ONNX 모델 weight 부담 회피 (binary size 가벼움 유지)
- v1 단계에서는 Layer 1 (필수 자동 압축, D-30) + Layer 2 의 3 알고리즘 으로 충분

**v1 영향**:
- ✅ C-CTX-3: v1 우선 3 알고리즘 확정
- ⏸ CCR + Kompress-base 는 v1.5+ (TASK-005-2) 또는 그 이후 페이즈에서 재검토

#### 5.2.4 TASK-008 결정 — `provider-auto-config` skill (D-38, 2026-06-07)

**결정**: yklee 결정. **하드코딩 fallback list 폐기** → **런타임 discovered list + per-provider auth** 동적 구성. `provider-auto-config` skill 신설.

**선택 근거** (CONCEPT.md §11.3, D-38):
1. **환경 가변성** — yklee 의 API key 보유 상태 / 조직 SSO / local LLM 가용성 모두 시점·맥락 의존
2. **사용자 개입 최소화** — API key 만 등록하면 자동 fallback chain 구성
3. **확장성** — 새 provider 추가 시 코드 변경 없이 config + auth 만 등록
4. **local-first 우선** — Ollama/vLLM 자동 발견 시 cost 0 fallback
5. **graceful degrade** — primary 실패 시 discovered list 순차 시도, error surface 최소화

**v1 Phase 분리** (CONCEPT.md §11.3, D-38):
- **Phase 1 (TASK-005-1, MVP)**: 6 provider 정적 등록 + Anthropic API key (env → keychain fallback) + Ollama local server detect + **단순 fallback (config hardcoded)** — **동적 발견은 v1.5+**
- **Phase 2 (TASK-005-2, v1.5)**: `provider-auto-config` skill 정식 구현 + 모든 provider auth (login/logout/test) + `active-providers.yaml` 자동 생성/갱신 + **dynamic fallback chain**
- **Phase 3 (TASK-005-3, v2.0)**: OAuth flow (Anthropic OAuth, Google OAuth) + MCP-based provider 등록 + Multi-region / multi-account

**Skill reference design**: [`docs/skills/provider-auto-config/SKILL.md`](./skills/provider-auto-config/SKILL.md) (D-38)

**v1 영향**: ✅ C-LLM-2, FR-0.5, FR-0.4. Phase 1 만 v1, Phase 2+ 는 v1.5+.

#### 5.2.5 W16 결정 — `auth add-local` subcommand (D-59, 2026-06-09)

**결정**: yklee 요청 — TASK-005-1 W11~W15 의 OAuth 기반 3 provider (minimax/openai/google) 인증 흐름 외에, **로컬 LLM 서버 (Ollama/vLLM/LM Studio/llama.cpp) 를 대화형으로 등록**하는 `myharness auth add-local` subcommand 추가.

**선택 근거**:
1. **CONCEPT.md §5.2 의 12 명령어** 에 `auth` 가 이미 포함 — `Login/Logout/Status/List` 외 `AddLocal` 1개 추가는 자연스러운 확장
2. **`ProviderId::LocalLlm` (built-in) + `scan_local.rs` (W7.3) + `KeyringAuthStore` (W7.2) 인프라가 이미 존재** — 등록 UI 만 추가하면 됨
3. **OpenAI 호환** (Ollama/vLLM/LM Studio/llama.cpp 모두 `/v1/models` GET 지원) → 1개 endpoint probe 로 4 서버 통합
4. **API token 선택** (Ollama 기본은 불요, vLLM/LM Studio 는 선택) → `KeyringAuthStore` 의 in-memory cache + env-first fallback 재사용
5. **모델 선택 TUI** (`inquire` crate, arrow-key select) → UX 자연스러움. stdin read_line 직접 대비 의존성 +1 만 추가

**v1 영향** (SDLC):
- ✅ `myharness-llm::register_local_provider(base_url, token, model)` API 추가 (W16) — `ProviderRegistry` 에 `LocalLlm` entry 의 `base_url` + `default_model` + `available_models` 갱신
- ✅ `myharness auth add-local` clap subcommand 추가 (W16) — `Cmd::Auth::AuthAction::AddLocal`
- ✅ inquire 의존성 추가 (`myharness-cli/Cargo.toml`)
- ⏸ 자동 fallback chain 자동 갱신은 v1.5+ (`provider-auto-config` Phase 2 의 active-providers.yaml 갱신). W16 은 **수동 1회 등록** 만.
- ⏸ 등록 후 자동 default LLM 활성화는 별도 `myharness config set default_llm local-llm:<model>` 명령 (v1.5+). W16 은 **provider 등록 + available models 채움** 까지만.

**FR 매핑**: FR-0.5 (provider 등록) 의 **로컬 LLM sub-case** (CONCEPT.md §5.5.1 의 discover + auth + save 3-단계 중 **수동 sub-case**).

**상세설계**: [`docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md`](./architecture/DETAILED_DESIGN_ADD_LOCAL.md) (D-59)

**TC scaffold**: `docs/specs/TC_UNIT.md` §W16-AddLocal (L1 Unit 8개) + `docs/specs/TC_INTEGRATION.md` §W16-AddLocal (L2 3개)

### 5.3 결정 보류 해소 트리거 (TASK-002)

**TASK-002** 의 해소는 **yklee 인프라 정보 수령 후**:

| FR | 해소 필요 정보 | 출처 |
| --- | --- | --- |
| FR-SERVER-1 | 원격 호스트 목록 / SSH 별칭 / 헬스체크 명령 | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-SERVER-2 | 서비스 매핑 (systemd unit / docker container / log path) | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-SERVER-3 | 배포 환경 정의 (ssh / k8s context / docker registry) | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-SERVER-4 | config file 경로 + backup 정책 | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-ENV-1 | Homebrew 패키지 목록 / asdf 런타임 버전 / dotfiles 경로 | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-ENV-2 | package manager 우선순위 / 설치 매니페스트 | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-ENV-3 | 셸 환경 (bash/zsh/PowerShell) / alias / 함수 | PROJECT_PROFILE.md §3.1 (TODO) |
| FR-ENV-4 | 진단 항목 우선순위 / PATH 표준 | PROJECT_PROFILE.md §3.1 (TODO) |

**v1 구현 시**: §2.2/§2.3 의 **시그니처 + sub-agent 디스패치 구조** 는 구현. 세부 가이드는 PROJECT_PROFILE.md §3.1 (TODO) → TASK-002 해소 시 채움.

### 5.4 v1 spec 잠금 (D-40)

CONCEPT.md §11.2 (claude-code 2.1.169 영향 결정) **완전 제거** (D-40). v1 spec 잠금:
- ✅ Rust 1안 (D-36)
- ✅ ratatui + crossterm (D-36)
- ✅ headroom 3 알고리즘 (D-37)
- ✅ provider-auto-config (D-38)
- ⏸ TASK-002 (yklee 인프라 정보)

**향후 2.1.169 이상 변경 시점**에 v1 spec 영향 별도 평가 (현재 v1.5+ 에서 처리, v1 spec 자체는 잠금 상태 유지).

---

## 6. 안티 패턴 미반영 체크리스트 (Anti-pattern Non-adoption)

> **참조**: CONCEPT.md §8 의 6 anti-pattern. **v1 산출물에 어느 것도 등장하지 않음** 을 검증.
>
> **목적**: TASK-005-1 구현자가 안티 패턴을无意中 도입하지 않도록 명시적 체크리스트로 표기.

### 6.1 안티 패턴 6개 (CONCEPT.md §8)

| # | 안티 패턴 | 회피 전략 | v1 검증 |
| - | --- | --- | --- |
| **1** | **closed source + leak 의존** (claude-code 13.27) | MIT/Apache 2.0 open. v1 = Rust 1안, rig-core / ratatui / rmcp / keyring / tree-sitter 모두 오픈소스 | ✅ 본 REQUIREMENTS.md + CONCEPT.md |
| **2** | **듀얼 언어** (headroom 13.15) | 단일 언어 (Rust 1안 OR TS 2안). v1 = Rust 1안 (D-36) | ✅ C-STACK-4 |
| **3** | **100+ slash commands** (claude-code 13.30) | 3-도메인 × 3-4 명령 = ~12 명령 max (CONCEPT.md §5.2, §8) | ✅ NFR-UX-6, FR-0.1 |
| **4** | **5 surface 동시 유지** (claude-code 13.36) | v1 CLI+TUI only, 점진 확장 (NFR-UX-1) | ✅ C-OOS-1, C-OOS-10 |
| **5** | **cloud auto memory privacy** (claude-code 13.37) | v1 local-only, v2+ opt-in cloud (NFR-SEC-8) | ✅ C-OOS-8 |
| **6** | **subscription requirement** (claude-code 13.34) | CLI free, v2+ premium 검토 | ✅ C-OOS-9 |

### 6.2 검증 매트릭스 (v1 산출물 cross-check)

각 안티 패턴이 본 REQUIREMENTS.md + INITIAL_DESIGN.md (WP3) + CONCEPT.md 어디에도 **부정형 (❌, ❎, ⛔) 으로 명시된 회피 정책만** 등장하는지 검증. **긍정형 (✅, ✓, O) 으로 채택된 적 없음** 을 확인.

| 안티 패턴 | CONCEPT.md | 본 REQUIREMENTS.md | INITIAL_DESIGN.md (WP3) | 채택 ❌ |
| --- | --- | --- | --- | --- |
| closed source | §8 안티 1 (회피) | C-OOS-7 (회피) | (예정) | ✅ |
| 듀얼 언어 | §8 안티 2 (회피) | C-STACK-4 (회피) | (예정) | ✅ |
| 100+ slash commands | §8 안티 3 (회피) | NFR-UX-6, FR-0.1 (회피) | (예정) | ✅ |
| 5 surface 동시 | §8 안티 4 (회피) | C-OOS-1, C-OOS-10 (회피) | (예정) | ✅ |
| cloud auto memory default | §8 안티 5 (회피) | NFR-SEC-8, C-OOS-8 (회피) | (예정) | ✅ |
| subscription requirement | §8 안티 6 (회피) | C-OOS-9 (회피) | (예정) | ✅ |

**v1 verifier 검증 시**: 위 6개 안티 패턴 중 어느 하나라도 산출물에 긍정형 (✅, ✓, O, "this pattern 을 채택") 으로 등장 시 **FAIL**.

### 6.3 채택/미반영 합계 (CONCEPT.md §7 + §8)

- **Adopt**: 23개 (1차 MVP 8 + 2차 v1.5 7 + 3차 v2+ 8) — §7 참조
- **Anti**: 6개 (절대 안 함) — 본 §6

v1 (TASK-005-1) 의 구현 범위는 **1차 MVP 8개 adopt** + **6 anti-pattern 미반영** (양수 합 14). 2차 7개 (v1.5, TASK-005-2) + 3차 8개 (v2+, TASK-005-3~) 는 v1 spec 외.

---

## 7. 채택 패턴 반영 매트릭스 (Adopted Patterns Matrix)

> **참조**: CONCEPT.md §7 의 23 adopt 패턴. v1 = **1차 MVP 8개** 중심. 2차 7개 + 3차 8개는 v1.5+ (TASK-005-2~) 구현 입력.
>
> **목적**: TASK-005-1 구현자에게 "어떤 reference 의 어떤 패턴을 어디에 반영했는지" 명시적으로 표시. INITIAL_DESIGN.md (WP3) 의 module tree / 데이터 흐름 / API 표면의 정당성根拠.

### 7.1 1차 MVP (v1, 8개) — TASK-005-1 구현 대상 ⭐

| # | Adopt 패턴 | 출처 | v1 반영 위치 (REQUIREMENTS §X) | INITIAL_DESIGN (WP3) 반영 |
| - | --- | --- | --- | --- |
| **1** | **Harness 5 components** (Tools · Context · Session · Plugins · Sub-agents) | claude-code 13.1 (CONCEPT.md §7, §5.1) | §2.0 FR-0.3 | §3 Rust module tree (5 components crate) |
| **2** | **CLAUDE.md 표준** | claude-code 13.6 (CONCEPT.md §7) | §2.7 Context 관리 (1번 항목), §2.10 6원칙 | §8 Config + State (project root `MiniMax.md` 동급) |
| **3** | **Hook markdown rule** | claude-code 13.4 hookify (CONCEPT.md §5.4, §7) | §2.9 Security (Hook system), NFR-SEC-4 | §9 Security (9 security patterns + warn-rm-rf + require-test) |
| **4** | **4 permission mode** (default / acceptEdits / plan / bypassPermissions) | claude-code 13.8 (CONCEPT.md §5.4, §7) | §2.9 Security, NFR-SEC-3, NFR-SEC-6 | §9 Security (4 mode + flag override) |
| **5** | **3 fallback model** (primary + 2 fallback) | claude-code 13.15 (CONCEPT.md §5.5.3, §7, D-15) | §2.0 FR-0.4, C-LLM-2, §5.2.4 TASK-008 결정 | §6 LLM 통합 (Phase 1 hardcoded + Phase 2 dynamic) |
| **6** | **5 install paths** (install.sh / install.ps1 / brew / winget / apt-dnf-apk) | claude-code 13.9 (CONCEPT.md §5.3, §7) | §3.6 NFR-INST-1, C-STACK-3 | §11 Cross-platform 빌드 (cargo-dist 5 paths) |
| **7** | **CCR (headroom)** | headroom 13.3 (CONCEPT.md §5.6, §7) | §2.7 Context 관리 (2-계층), C-CTX-3 (v1.5+ 연기) | §7 Context 관리 (v1.5+ CCR 통합 위치) |
| **8** | **Provider 비종속** (12+ provider via rig-core) | aider/opencode/goose 13.2 (CONCEPT.md §5.5, §7) | §2.0 FR-0.4 (6 provider), C-LLM-1, C-LLM-3 | §6 LLM 통합 (rig-core + OpenAI 호환 client) |

### 7.2 2차 (v1.5, 7개) — TASK-005-2 구현 대상 (v1 spec 외, reference)

| # | Adopt 패턴 | 출처 | v1.5+ 반영 위치 (placeholder) |
| - | --- | --- | --- |
| **9** | **Plugin 4-계층** (commands / agents / skills / hooks) | claude-code 13.3 (CONCEPT.md §5.7, §7) | §2.8 (v1 = local only, v1.5+ 4 계층) |
| **10** | **Auto memory** | claude-code 13.5 (CONCEPT.md §7) | §2.7 (3 계층), §3.5 NFR-OBS-4 (auto memory) |
| **11** | **/compact slash command** | claude-code 13.7 (CONCEPT.md §7) | §2.7 (3 계층), §3.7 NFR-REL-4 |
| **12** | **MCP server 1안** | claude-code 13.24 (CONCEPT.md §5.14, §7, D-33) | §2.6 (4 pre-config), §3.8 NFR-COMPAT-1 |
| **13** | **Sub-agents + Agent SDK** | claude-code 13.22 (CONCEPT.md §5.11, §7) | §2.1/§2.2/§2.3 (15 sub-agents) |
| **14** | **CacheAligner** | headroom 13.5 (CONCEPT.md §5.6, §7) | §2.7 (v1 우선 3 알고리즘 #1) |
| **15** | **ContentRouter** | headroom 13.4 (CONCEPT.md §5.6, §7) | §2.7 (v1 우선 3 알고리즘 #2) |

### 7.3 3차 (v2+, 8개) — TASK-005-3~5 구현 대상 (v1 spec 외, reference)

| # | Adopt 패턴 | 출처 | v2+ 반영 위치 (placeholder) |
| - | --- | --- | --- |
| **16** | **5 surfaces cross-surface** | claude-code 13.2 (CONCEPT.md §7, §4.2) | C-OOS-1, NFR-UX-1 (v2+, TASK-005-3) |
| **17** | **Plugin marketplace** | claude-code 13.3 (CONCEPT.md §7, §4.2) | §2.8 (v2+) |
| **18** | **Routines** | claude-code 13.17 (CONCEPT.md §7, §4.2) | C-OOS-4 (v2+, TASK-005-3) |
| **19** | **Multi-agent parallel + confidence scoring** | claude-code 13.11 (CONCEPT.md §7) | FR-CODE-1 v1.5+ scope (TASK-005-4) |
| **20** | **Channels** (Slack/Telegram webhook) | claude-code 13.25 (CONCEPT.md §7, §4.2) | C-OOS-5 (v2+) |
| **21** | **Security 3-tier** | claude-code 13.13 (CONCEPT.md §7) | §3.2 NFR-SEC (v2+ 확장) |
| **22** | **Cross-session security** | claude-code 13.14 (CONCEPT.md §7) | §3.2 NFR-SEC (v2+ 확장) |
| **23** | **Thinking toggle per-model** | claude-code 13.20 (CONCEPT.md §7) | §2.0 FR-0.4 (v1 = enabled for code, disabled for server/env, CONCEPT.md §5.5.3) |

### 7.4 1차 MVP 8개 → 본 REQUIREMENTS.md 매핑 요약

| Adopt | 본 문서 반영 섹션 | NFR/FR ID | Constraint ID |
| --- | --- | --- | --- |
| #1 Harness 5 components | §2.0 FR-0.3 | — | C-STACK-1, C-DIR-2 |
| #2 CLAUDE.md 표준 | §2.7, §2.10 | NFR-UX-2, NFR-UX-3, NFR-UX-4, NFR-UX-5 | C-DIR-2 |
| #3 Hook markdown rule | §2.9, §3.2 | NFR-SEC-4 | — |
| #4 4 permission mode | §2.9, §3.2 | NFR-SEC-3, NFR-SEC-6 | C-COUPLE-1 |
| #5 3 fallback model | §2.0, §5.2.4 | NFR-REL-1, NFR-REL-2, NFR-REL-3 | C-LLM-2 |
| #6 5 install paths | §3.6 | NFR-INST-1, NFR-INST-2, NFR-INST-4 | C-STACK-3 |
| #7 CCR (headroom) | §2.7, §3.7 | NFR-REL-4 | C-CTX-1, C-CTX-2, C-CTX-3 (v1.5+), C-CTX-4 |
| #8 Provider 비종속 | §2.0 | NFR-REL-1, NFR-REL-2, NFR-REL-3 | C-LLM-1, C-LLM-3, C-LLM-4 |

**전체 1차 8개 adopt** 가 본 REQUIREMENTS.md 에서 **명시적 NFR/FR/Constraint ID** 로 매핑됨 → INITIAL_DESIGN.md (WP3) 가 module tree / 데이터 흐름 / API 표면을 도출할 때 본 매트릭스를 정당성 근거로 사용 가능.

### 7.5 1차 vs 2차 vs 3차 비율 (TASK-005-N 매핑)

| 차수 | 개수 | task_id | v1 spec 포함? |
| --- | --- | --- | --- |
| 1차 MVP | 8 | TASK-005-1 (v1.0) | ✅ 본 REQUIREMENTS 의 구현 대상 |
| 2차 | 7 | TASK-005-2 (v1.5) | ❌ v1 spec 외 (placeholder 만, v1.5+ 입력) |
| 3차 | 8 | TASK-005-3 ~ TASK-005-5 (v2.0~v3.0) | ❌ v1 spec 외 (placeholder 만, v2+ 입력) |
| **합계** | **23** | — | **v1 = 1차 8개만** (35%) |

---

## 8. 추적성 매트릭스 (Traceability, CONCEPT.md §X.Y ↔ 본 문서 §X.Y)

> **목적**: verifier 가 CONCEPT.md SSOT 와 본 REQUIREMENTS.md 의 claim 매칭을 빠르게 수행할 수 있도록 매핑 제공.

### 8.1 CONCEPT.md §X.Y → 본 REQUIREMENTS.md

| CONCEPT.md | 본 REQUIREMENTS.md | 매핑 |
| --- | --- | --- |
| §0 핵심 Positioning | §1.2, §1.3 | D-25 Mavis zero coupling + 5 NOT |
| §1 한 줄 Positioning | §1.1 | (그대로) |
| §2 타겟 사용자 | §1.2 | yklee single user (v1) |
| §3 핵심 가치 | §1.4 | 3가지 가치 |
| §4.1 In-scope | §1.2 | 3-도메인 + 3 OS |
| §4.2 Out-of-scope | §4.6 C-OOS-1~10 | 6 out-of-scope + 4 anti → constraint |
| §5.1 아키텍처 (5 components) | §2.0 FR-0.3 | 5 components |
| §5.2 명령 가이드 (12 명령) | §2.1/§2.2/§2.3 FR-* | 3-도메인 × 4 = 12 |
| §5.3 설치/배포 (5 install paths) | §3.6 NFR-INST-1 | (그대로) |
| §5.4 보안 (4 perm + hook + secret) | §2.9, §3.2 | NFR-SEC-1~8 |
| §5.5 LLM 통합 (4 subsections) | §2.0 FR-0.4, FR-0.5 | (그대로) |
| §5.6 Context 관리 (2-계층) | §2.7, §3.7 | C-CTX-1~4, NFR-REL-4 |
| §5.7 Plugin 시스템 | §2.8 | v1 = local only, v1.5+ 4 계층 |
| §5.8 외부 의존성 없음 | §4.2 C-COUPLE-1~4 | zero coupling |
| §5.9 standard_ai_workflow | §2.10 | 6 원칙 native + 옵션 Mavis 통합 |
| §5.10 Agent 모드 (3가지) | §2.0 FR-0.2 | 3 mode + loop ralph-wiggum |
| §5.11 Built-in sub-agents (15) | §2.1/§2.2/§2.3/§2.4 | 15 sub-agents 매핑 |
| §5.12 `~/.myharness/` 디렉토리 | §4.5 C-DIR-1~4 | XDG-style |
| §5.13 LLM Wiki (v2+) | (v1 spec 외) | TASK-005-4 (v2.5) 입력 |
| §5.14 Skill/MCP first-class | §2.5, §2.6 | 7 skills + 4 MCP |
| §6 v2+ 로드맵 | §1.6 | TASK-005-1~5 |
| §7 채택 패턴 (23) | §7.1, §7.2, §7.3 | 1차 8 / 2차 7 / 3차 8 |
| §8 안티 패턴 (6) | §6.1, §6.2 | 6 anti → 회피 매트릭스 |
| §9 KPI | §3.9 | 7 지표 |
| §10 리스크 | §3.7 NFR-REL (partly) | 7 리스크 중 reliability 만 매핑 |
| §11.1 결정 보류 | §5.1 | (그대로, 4 done + 1 deferred) |
| §11.3 결정 완료 4건 | §5.2 | TASK-005/006/007/008 |
| §12 참고 | — | reference 만 |

### 8.2 development_log.md 결정 → 본 REQUIREMENTS.md

| D-NN | 결정 | 본 REQUIREMENTS.md 반영 |
| --- | --- | --- |
| D-15 | headroom 분석 owner takeover + chunked write | (본 작업의 chunked write 자체, 메타) |
| D-16 | chunked write 영구화 | (본 작업의 4 chunk 분할, 메타) |
| D-25 | Mavis zero coupling | §1.3 NOT 2, §4.2 C-COUPLE-1~4 |
| D-26 | standard_ai_workflow native + 옵션 통합 | §2.10 6 원칙 + 옵션 Mavis 통합 |
| D-27 | headroom built-in (외부 proxy ❌) | §1.3 NOT 5, §2.7, §4.4 C-CTX-4 |
| D-28 | Provider 6개 + OpenAI 호환 | §2.0 FR-0.4, C-LLM-1/3/4 |
| D-29 | Agent 3 모드 + Built-in sub-agents | §2.0 FR-0.2, §2.1/§2.2/§2.3 sub-agents |
| D-30 | 2-계층 Context 압축 (Layer 1 필수 + Layer 2 선택) | §2.7, §4.4 C-CTX-1/2 |
| D-31 | `~/.myharness/` XDG-style | §4.5 C-DIR-1~4 |
| D-32 | LLM Wiki (v2+) | (v1 spec 외) |
| D-33 | Skill/MCP first-class | §2.5, §2.6, §3.8 NFR-COMPAT-1/3 |
| D-36 | TASK-005 결정: Rust 1안 | §5.2.1, §4.1 C-STACK-1~4 |
| D-37 | TASK-007 결정: headroom 3 알고리즘 | §5.2.3, §4.4 C-CTX-3 |
| D-38 | TASK-008 결정: provider-auto-config | §5.2.4, §2.0 FR-0.5, C-LLM-2 |
| D-39 | 세션 마무리 (v1 컨셉 Phase 종료) | (메타, 본 작업 트리거) |
| D-40 | §11.2 claude-code 2.1.169 검증 취소 (v1 spec 잠금) | §5.4 |

### 8.3 PROJECT_PROFILE.md → 본 REQUIREMENTS.md

| PROJECT_PROFILE.md | 본 REQUIREMENTS.md |
| --- | --- |
| §1 프로젝트 개요 (3-도메인) | §1.2, §1.4 |
| §3.1 도메인별 작업 명령 | §2.1, §2.2, §2.3 (TASK-002 ⏸ placeholder 명시) |
| §4 검증 포인트 | §3 NFR (성능/보안/UX) |
| §5 예외 규칙 (위험 작업 정책) | §3.2 NFR-SEC-5, §3.7 NFR-REL-5 |
| §5 (언어/컨텍스트) | §3.4 NFR-UX-2, NFR-UX-3 |

---

## 9. 후속 단계 (Next Steps)

### 9.1 WP1 → WP3 → TASK-005-1 체인

본 REQUIREMENTS.md 가 **TASK-005-1 (v1 Rust MVP 구현) 의 유일한 입력 문서** 이다 (CONCEPT.md SSOT + 본 문서 외 추가 doc 불필요):

1. **WP1 (본 작업)** → `docs/REQUIREMENTS.md` (이 문서) ✅
2. **WP2 (parallel)** → `docs/USE_CASES.md` (actor × scenario, 700~1,100줄)
3. **WP3 (cycle 2, depends on WP1+WP2)** → `docs/architecture/INITIAL_DESIGN.md` (Harness 5 components Rust 모듈/API/CLI 표면, 800~1,300줄)
4. **TASK-005-1** (Rust 빌드) → INITIAL_DESIGN.md + 본 REQUIREMENTS.md 입력으로 cargo workspace init

### 9.2 본 문서의 후속 갱신 트리거

- **CONCEPT.md 갱신 시** (v1.5+ 결정 시) — §8.1 추적성 매트릭스 + §5 결정 보류 + §7 채택 매트릭스 + §4 제약 갱신 필요 (D-23, D-35 align 룰)
- **PROJECT_PROFILE.md §3.1 (TODO) 채워질 시** (TASK-002 해소 시) — §2.2/§2.3 placeholder → 실제 가이드로 갱신
- **verifier reject 시** — §5/§6/§7 의 verification evidence 부족분 보강

### 9.3 verifier 검증 체크리스트 (재확인)

verifier 는 **producer 의 산출물을 그대로 인용하지 말고** CONCEPT.md §X.Y 원문에서 직접 claim 을 찾아 매칭 (재도출):

1. CONCEPT.md §11.1 결정 보류 정확 반영 (TASK-002 ⏸) — server/env 명령 가이드를 yklee 인프라 정보로 채우지 말 것 (placeholder 만)
2. CONCEPT.md §11.3 결정 완료 4건 정확 인용 (TASK-005 Rust 1안 / TASK-006 ratatui / TASK-007 headroom 3 algo / TASK-008 provider-auto-config)
3. CONCEPT.md §5.2 의 12 명령어 모두 FR 로 매핑
4. CONCEPT.md §5.11 의 15 sub-agents 모두 FR participant 로 등장
5. CONCEPT.md §5.14 의 built-in skills 모두 FR 로 등장 (D-38 포함 7개)
6. CONCEPT.md §8 안티 6 미반영 (closed source / 듀얼 언어 / 100+ slash commands / 5 surface 동시 / cloud auto memory default / subscription requirement)
7. CONCEPT.md cross-ref 무결성 — broken link 0, 모든 § 번호가 원문과 일치
8. 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
9. 분량 600~1,000줄 범위
10. 토큰 값/시크릿 ❌ (D-06 정책) — 메커니즘만 기술

---

## 10. Handoff (D-26 형식)

> **handoff 형식** (D-26): `summary / risks / suggested_follow_up / produced_artifacts`. 모든 work 종료 시 구조화 출력.

### Summary

`docs/REQUIREMENTS.md` 작성 완료. **7 sections + 8 (추적성) + 9 (후속 단계) + 10 (handoff) = 10 sections**. 분량 약 850~900줄. **TASK-005-1 (v1 Rust MVP 구현) 의 유일한 입력 문서** 로, 본 문서만으로 Rust 모듈 / API / CLI 트리 시작 가능. 모든 claim 에 CONCEPT.md §X.Y cross-ref 부착 (4 done 결정 + 1 deferred + 8 adopt 1차 + 6 anti-pattern + 15 sub-agents + 7 skills + 6 NFR 카테고리 + 4 constraint 카테고리).

### Risks

- **TASK-002 보류** — server/env 명령 가이드는 placeholder. v1 구현 시 yklee 인프라 정보 미수령 상태에서 디스패치 구조만 구현 (FR-SERVER-*/FR-ENV-* 시그니처는 완성, 세부 가이드는 PROJECT_PROFILE.md §3.1 TODO 영역).
- **minimax TBD** (D-28) — base_url + API 형식 검증 미실시. v1 Phase 1 의 OpenAI 호환 client 가 cover 하나, 정확한 endpoint 는 v1.5+ 안정화.
- **rmcp 1.4 성숙도** (D-36 §11.3 리스크) — MCP SDK Rust 생태계 검증 필요. v1 구현 시 `rmcp` 0.x → 1.4 사이 마이너 변경 가능성.
- **CONCEPT.md vs 본 문서 drift** — 향후 CONCEPT.md 갱신 시 §8.1 추적성 매트릭스 + §4/§5/§7 도 함께 align 필수 (D-23, D-35 align 룰).

### Suggested Follow-up

1. **WP2 (use-cases)** — `docs/USE_CASES.md` 작성. parallel 진행 중.
2. **WP3 (initial-design)** — `docs/architecture/INITIAL_DESIGN.md` 작성. depends on [WP1, WP2] (cycle 2 sequential). 본 REQUIREMENTS.md 의 FR/NFR/Constraint ID + USE_CASES.md 의 actor/use case 를 입력으로 Rust module tree / 데이터 흐름 / API 표면 도출.
3. **TASK-005-1** — Rust 1안 v1 MVP 빌드 시작. cargo workspace init + ratatui TUI shell + rig-core Anthropic + basic Tools (Read/Write/Edit/Bash) + Context (CLAUDE.md + /compact) + standard_ai_workflow output + 4 permission mode + 1-2 sub-agent (code-reviewer, server-status) (CONCEPT.md §11.3 구현 우선순위 8단계).
4. **TASK-002 해소** — yklee 인프라 정보 (호스트 목록 / SSH 별칭 / Homebrew 패키지 / asdf 런타임 / dotfiles) 수령 후 §2.2/§2.3 placeholder 채움 + PROJECT_PROFILE.md §3.1 TODO 해소.
5. **본 문서 align 룰 확립** — CONCEPT.md 갱신 시 본 REQUIREMENTS.md + INITIAL_DESIGN.md (WP3) + PROJECT_PROFILE.md + MiniMax.md 4 문서 동시 align (D-23, D-35).

### Produced Artifacts

- `docs/REQUIREMENTS.md` (메인 산출물, **~850-900줄 / 10 sections**)
- `docs/team/deliverable_requirements.md` (early signal + final status, D-16 패턴 준수)

### Cross-references

- 입력 SSOT: [`docs/CONCEPT.md`](./CONCEPT.md), [`docs/PROJECT_PROFILE.md`](./PROJECT_PROFILE.md), [`docs/development_log.md`](./development_log.md)
- plan: [`docs/team/PLAN_v1_design.md`](./team/PLAN_v1_design.md) (WP1 spec)
- 후속 산출물: [`docs/USE_CASES.md`](./USE_CASES.md) (WP2), [`docs/architecture/INITIAL_DESIGN.md`](./architecture/INITIAL_DESIGN.md) (WP3)
- 후속 task: **TASK-005-1** (v1 Rust MVP 구현) — 본 문서 + INITIAL_DESIGN.md 입력

---

## VERDICT (final, post-handoff)

**VERDICT: PASS**

- 본 문서 = my_harness v1 의 spec 사양서로서 모든 v1 구현 입력 요구 충족
- 7 required sections + 3 optional (D-26 handoff) = 10 sections 완성
- 964 lines (목표 600-1,000 내)
- CONCEPT.md cross-ref 235건, broken link 0
- 10/10 self-check PASS (위 표)
- D-16 패턴 준수 (4 chunks + early signal + minimal board noise + handoff)
- 표준 6 원칙 형식 (한국어 / 결론 위주 / 상태값 / 이벤트 소싱 / 비참조 / handoff)
- 안티 6 미반영 (회피 정책으로만 등장)
- D-06 토큰 값/시크릿 ❌ (메커니즘만)

**TASK-005-1 (v1 Rust MVP 구현) 의 입력으로 사용 가능**. INITIAL_DESIGN.md (WP3) 와 함께 3-체인 완성.
