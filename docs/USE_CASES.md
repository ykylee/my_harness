# USE_CASES.md — my_harness v1 유스케이스 (TASK-005-1 입력)

> **본 문서의 위치**: my_harness v1 구현의 **유스케이스 도출서**. CONCEPT.md (마스터 컨셉) + PROJECT_PROFILE.md (도메인 스코프) 를 입력으로, `docs/REQUIREMENTS.md` (WP1, 참고용) 와 함께 **별도 doc 재참조 없이 본 문서만으로 Rust 모듈 시작 가능** 한 수준의 actor × scenario 매트릭스를 제공한다.
>
> **상태**: draft (v1, WP2 산출물)
> **최종 갱신**: 2026-06-07
> **산출 형식**: D-16 chunked write 6-chunk / D-26 handoff 표준 준수
> **관련 문서**: [CONCEPT.md](./CONCEPT.md) (SSOT) · [PROJECT_PROFILE.md](./PROJECT_PROFILE.md) · [REQUIREMENTS.md](./REQUIREMENTS.md) (참고) · [development_log.md](./development_log.md)

---

## 0. 문서 메타 + 읽는 법

### 0.1 결론 (TL;DR)

- **Actor는 4 종류** (yklee / sub-agent / external / local LLM server) — CONCEPT.md §0/§2/§5.11 정합, 신규 발명 없음.
- **Use case는 7 prefix × 5~15개 = 50~70개** (UC-CODE / UC-SERVER / UC-ENV / UC-AUTH / UC-INSTALL / UC-CFG / UC-MAINT). 그 중 **5개는 detailed**, 나머지는 **catalog 인덱스** + acceptance criteria 로 정밀화.
- **3 agent mode** (orchestrator / single / loop) × 50+ use case 의 **dispatch matrix** 가 §4.
- **15 built-in sub-agents** (CONCEPT.md §5.11) × 50+ use case 의 **participant matrix** 가 §5.
- **3 extension points** (plugin v1.5+ / MCP server v1 4 pre-config / skill v1.5+) 가 §6.
- **5 exception flows** (provider fallback D-38 / context overflow D-30 / permission deny / hook block / tool error) 가 §7.
- **6 out-of-scope** (CONCEPT.md §4.2) 는 §8 에서 의도적 누락 use case 로 매핑 — v1 implementation 에서 absolute ❌.
- **3 platform** (macOS / Linux / Windows, D-31 + D-36) 의 분기 표가 §9.
- **각 use case 마다 acceptance criteria** (테스트 가능) 가 §10.

### 0.2 결정 보류 반영 (CONCEPT.md §11.1)

| task_id | 결정 | 상태 | 본 문서 반영 |
| --- | --- | --- | --- |
| **TASK-002** | 도메인별 명령 가이드 (server/env) | ⏸ yklee 인프라 정보 필요 | §2 catalog 의 server/env use case 는 **placeholder** — yklee 인프라 (원격 호스트 / SSH alias / dotfiles / asdf runtime) 정보는 `PROJECT_PROFILE.md §3.1` 의 "TASK-002 채움 예정" 영역을 그대로 비워둠. **본 문서는 인프라 정보를 발명하지 않음**. |
| **TASK-005** | 스택 = Rust 1안 | ✅ D-36 | 모든 use case 는 Rust 1안 가정 (ratatui + rig-core + rmcp + keyring + cargo-dist) |
| **TASK-006** | TUI = ratatui + crossterm | ✅ D-36 (TASK-005 종속) | UC-TUI-* use case 는 ratatui surface 로 한정 |
| **TASK-007** | headroom 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) | ✅ D-37, v1 우선 | UC-CTX-001/002/003 use case 가 §2 catalog + §3 detailed 에 등장. CCR + Kompress-base 는 v1.5+ 이므로 §8 OOS 에 명시 |
| **TASK-008** | Provider fallback = `provider-auto-config` skill (D-38) | ✅ D-38 | UC-AUTH-001 (discover+login) + UC-AUTH-006/007 (fallback chain) 가 §3 detailed + §7 exception 에 등장 |

### 0.3 안티 패턴 미반영 체크 (CONCEPT.md §8, 6개)

| # | 안티 | 본 문서 채택 |
| --- | --- | --- |
| 1 | closed source + leak 의존 | MIT/Apache 2.0 (open) — use case 자체는 라이선스 중립이나 **UC-PLUGIN-* 의 marketplace 는 v2+ OOS** (§8) |
| 2 | 듀얼 언어 | **단일 언어 (Rust 1안)** — 모든 use case 의 구현 표면은 Rust module |
| 3 | 100+ slash commands | **3-도메인 × 3~4 명령 = 12 명령 max** — UC-* catalog 도 12 CLI entry point 의 participant 만 정의, 100+ 발명 ❌ |
| 4 | 5 surface 동시 | **v1 = CLI + TUI only** — UC-SURFACE-* 는 v2+ OOS (§8) |
| 5 | cloud auto memory default | **v1 = local-only** — 모든 UC-MEMORY-* use case 는 `~/.myharness/memory/` (CONCEPT.md §5.12) 한정, cloud sync 는 opt-in v2+ |
| 6 | subscription requirement | **CLI free** — UC-AUTH-* 의 `auth setup` / `auth login` 은 free CLI 안에서 동작 |

### 0.4 표준 6 원칙 (CONCEPT.md §5.9.1) 형식 준수

- **한국어 보고** (default), 코드/명령/경로/CLI flag 는 영문 원문
- **결론 + 다음 행동 위주**, 중간 reasoning 은 §0/§1 메타 + §10 acceptance 에만 압축
- **상태값**: `planned | in_progress | blocked | done` 4 값 (TASK status 보고 시)
- **이벤트 소싱**: 모든 use case 실행은 `~/.myharness/log.jsonl` (CONCEPT.md §5.12) 에 append — **본 문서는 그 메커니즘을 사용한다고 선언**
- **비참조 원칙**: 다른 세션/이전 세션 참조 ❌. handoff 만 사용
- **handoff 형식 (D-26)**: `summary / risks / suggested_follow_up / produced_artifacts` 4-필드

### 0.5 §X.Y cross-ref 규칙

본 문서의 모든 claim 은 `CONCEPT.md §X.Y` / `PROJECT_PROFILE.md §X.Y` / `REQUIREMENTS.md §X.Y` (참고) 의 원문 § 번호로 추적 가능. **새로운 actor / 명령 / 결정 발명 ❌**.

---

## 1. Actor 정의

**4 종류의 actor** 가 my_harness v1 시스템에 등장한다. CONCEPT.md §0 (Positioning, Mavis zero coupling) + §2 (타겟 사용자) + §5.11 (15 sub-agents) 정합.

### 1.1 Actor 분류 표

| Actor ID | 이름 | 종류 | 정의 | 출처 |
| --- | --- | --- | --- | --- |
| **A1** | `yklee` | **primary** (오너, single user) | terminal 에서 `myharness <command>` 직접 실행. v1 의 모든 작업 개시자 + 최종 승인자. 3-도메인 (코드/서버/환경) 작업 수행 | CONCEPT.md §0, §2, §4.1 |
| **A2** | `sub-agent` | **system** (내장 15개) | my_harness 가 spawn 하는 specialized worker. 3-도메인 × 5 + utility 2 = **15개** (`code-reviewer`, `code-implementer`, `code-tester`, `code-refactorer`, `code-searcher`, `server-status`, `log-analyzer`, `deployer`, `config-manager`, `env-setup`, `env-installer`, `env-shell`, `env-diagnose`, `git-operator`, `file-searcher`) | CONCEPT.md §5.11 |
| **A3-EXT** | `external` (3 종류) | **external** | ① **plugin** (v1.5+, 사용자 정의) ② **LLM provider** (Anthropic / OpenAI / Google / DeepSeek / minimax) ③ **OS** (macOS / Linux / Windows 의 filesystem / process / network / keychain) | CONCEPT.md §5.4, §5.5, §5.7 |
| **A4-LOC** | `local LLM server` | **local** | Ollama / vLLM / LM Studio / llama.cpp (OpenAI 호환 endpoint, `http://localhost:11434/v1` 등) | CONCEPT.md §5.5.1 (#6 local LLM) |

### 1.2 Actor 별 책임 / 권한

#### A1. yklee (primary)

- **책임**: 3-도메인 작업 발주 + 결과 승인 + 시스템 설정 (config / plugin / hook / skill) 결정 + secret 등록 (keychain 위임)
- **권한**: 4 permission mode 의 `default` (CONCEPT.md §5.4) 에서 매번 승인 / `bypassPermissions` (sandbox) 시 자동 / `acceptEdits` 시 edit 자동 / `plan` 시 plan 만 보고 실행 시 승인
- **인터페이스**: CLI (12 명령) + TUI (ratatui) + stdin/stdout pipe
- **single user, multi-machine**: 동일 `~/.myharness/` 를 여러 머신에서 동시 사용 가능 (state 는 per-machine, config 는 sync 가능)

#### A2. sub-agent (system, 15 내장)

- **책임**: 1개 작업 = 1 sub-agent 위임 (CONCEPT.md §5.2 마지막 항목) — e.g., `myharness code review` → `code-reviewer` 1개 spawn
- **권한**: orchestrator 가 spawn 시 권한 scope 위임. sub-agent 자신의 권한 scope 외부 tool 호출 시도 시 거부
- **수명**: 1 작업 단위로 spawn → 결과 반환 → 종료. **stateful context 는 parent (orchestrator) 가 보관** (CONCEPT.md §5.10 — sub-agent 는 sub-set of Context)
- **위치**: v1 = 하드코딩 (Python module 내장 → Rust v1 = sub-agent module). v1.5+ = `~/.myharness/sub-agents/<name>/SYSTEM.md` (CONCEPT.md §5.11 두 번째 항목)

#### A3-EXT. external

- **A3a. plugin** (v1.5+ OOS) — CONCEPT.md §5.7 의 4-계층 (commands / agents / skills / hooks). v1 에서는 hook (markdown rule) 만 local 가능
- **A3b. LLM provider** — 6종 정적 등록 (CONCEPT.md §5.5.1). `rig-core` (Rust 1안) 가 Anthropic / OpenAI / Google / Ollama native, deepseek + minimax 는 자체 OpenAI 호환 client
- **A3c. OS** — filesystem (`directories` crate cross-platform), network (reqwest / hyper), process (tokio), keychain (keyring crate — macOS Keychain / Windows Credential Manager / Linux Secret Service, CONCEPT.md §5.4 Secret management)

#### A4-LOC. local LLM server

- **책임**: offline LLM inference 제공. cost 0 fallback 의 핵심 (CONCEPT.md §11.3 TASK-008 의 "local-first 우선")
- **검출**: `provider-auto-config` skill (D-38) 가 startup / `myharness auth` / fallback 실패 시 `http://localhost:11434/v1` 등 scan
- **상태**: `auth/state/ollama.yaml` 의 `status: available | unreachable` (CONCEPT.md §5.5.2)
- **모델**: `discovered_models: [qwen2.5-coder:32b, llama3:70b, codellama:34b]` 등 runtime 발견

### 1.3 Actor 간 관계 (시퀀스 다이어그램)

```
yklee (A1)
   │ myharness code review <pr>
   ▼
Orchestrator (A2 — main agent, mode=orchestrator default)
   │ spawn code-reviewer
   │ + git-operator (PR metadata)
   │ + file-searcher (changed files)
   ▼
code-reviewer (A2 sub-agent) + 동료 sub-agents
   │  multi-aspect: bugs / style / tests
   │  Tools: Read, Grep, Glob, mcp__github__pr_diff
   ▼
External (A3b LLM provider — Anthropic primary)
   │  prompt + tool use
   │  (D-38 fallback: OpenAI → Gemini → DeepSeek → Ollama A4)
   ▼
Result aggregation → Orchestrator → yklee (한국어 요약 + handoff)
```

**핵심**: actor 4 종류 외에 **5번째 actor 발명 ❌**. (e.g., "team lead", "scheduler" 같은 가상 actor 추가는 CONCEPT.md §0 의 "오케스트레이션 도구 아님" 위반.)

---

## 2. Use case catalog (인덱스)

**7 prefix** × **5~15 use case** = **약 50~70 use case**. 그 중 5개는 §3 detailed, 나머지는 catalog 인덱스 + §10 acceptance.

**Prefix 규칙**:
- `UC-CODE-*` — 코드 도메인 (CONCEPT.md §5.2, 명령 4)
- `UC-SERVER-*` — 서버 도메인 (CONCEPT.md §5.2, 명령 4)
- `UC-ENV-*` — 환경 도메인 (CONCEPT.md §5.2, 명령 4)
- `UC-AUTH-*` — LLM provider auth + discover (CONCEPT.md §5.5.2 + D-38)
- `UC-INSTALL-*` — 설치 / 배포 (CONCEPT.md §5.3, 5 install paths)
- `UC-CFG-*` — config / permission / hook (CONCEPT.md §5.4, §5.12)
- `UC-MAINT-*` — 유지보수 (log / state / handoff / memory / compression cache)

추가로 §3-§4 에서 등장하는 **mode-specific** prefix:
- `UC-LOOP-*` — loop mode 전용 (CONCEPT.md §5.10)
- `UC-SINGLE-*` — single mode 전용 (CONCEPT.md §5.10)
- `UC-CTX-*` — context 압축 (CONCEPT.md §5.6, D-30 + D-27)

### 2.1 UC-CODE-* (코드 도메인, 10개)

| UC ID | 명령 | sub-agent(s) | 핵심 시나리오 | 상세 |
| --- | --- | --- | --- | --- |
| **UC-CODE-001** | `myharness code review <pr-url>` | `code-reviewer` + `git-operator` + `file-searcher` | PR multi-aspect review (bugs / style / tests) | **§3 detailed** |
| UC-CODE-002 | `myharness code implement "<feature>"` | `code-implementer` + `file-searcher` | 새 기능 구현, multi-file 변경, plan 후 implement | catalog |
| UC-CODE-003 | `myharness code test <path>` | `code-tester` | test 실행 + 결과 분석 + fix 제안 | catalog |
| UC-CODE-004 | `myharness code commit "<message>"` | `git-operator` | git workflow (staged 변경 → commit → optional push) | catalog |
| UC-CODE-005 | `myharness code refactor <scope>` | `code-refactorer` + `code-searcher` | 리팩토링 (rename / extract / dedup) AST-aware | catalog |
| UC-CODE-006 | `myharness code search <query>` | `code-searcher` | ripgrep / tree-sitter 기반 구조 검색 | catalog |
| UC-CODE-007 | `myharness code analyze <file>` | `code-reviewer` (단일 파일 모드) | 단일 파일 static analysis | catalog |
| UC-CODE-008 | `myharness code format <path>` | `code-implementer` (format scope) | formatter 실행 + 변경 요약 | catalog |
| UC-CODE-009 | `myharness code deps <action>` | `code-implementer` + `env-installer` | 의존성 추가/제거/업그레이드 (Cargo.toml 등) | catalog |
| UC-CODE-010 | `myharness code diff <ref>` | `git-operator` + `code-reviewer` | working tree diff 분석 | catalog |

### 2.2 UC-SERVER-* (서버 도메인, 8개) — TASK-002 ⏸ placeholder 포함

| UC ID | 명령 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| **UC-SERVER-001** | `myharness server status [host]` | `server-status` | 프로세스/서비스 상태 점검 | **§3 detailed** |
| UC-SERVER-002 | `myharness server logs <service> [N]` | `log-analyzer` | 최근 N줄 로그 + 이상 패턴 detection | catalog |
| UC-SERVER-003 | `myharness server deploy <env>` | `deployer` | 배포 헬퍼 (ssh / k8s / docker) | catalog |
| UC-SERVER-004 | `myharness server config <action>` | `config-manager` | 설정 조회/변경 (with backup) | catalog |
| UC-SERVER-005 | `myharness server health [host]` | `server-status` + `log-analyzer` | 종합 헬스체크 (status + health endpoint + tail log) | catalog |
| UC-SERVER-006 | `myharness server restart <service>` | `deployer` + `config-manager` | service restart (with pre/post 상태 비교) | catalog |
| UC-SERVER-007 | `myharness server connect <host>` | (도구 위임, ssh subprocess) | SSH 연결 (config 의 host alias 사용) | catalog — **host alias 목록은 TASK-002 ⏸** |
| UC-SERVER-008 | `myharness server metrics [host]` | `server-status` | CPU / mem / disk / net 메트릭 | catalog |

> **TASK-002 ⏸ 노트**: UC-SERVER-* 의 host alias / SSH 별칭 / k8s context / docker host 정보는 **yklee 인프라 정보**가 필요. CONCEPT.md §11.1 / PROJECT_PROFILE.md §3.1 "초기값은 TODO" 와 정합. **본 문서는 placeholder 로 비워두고 발명 ❌**.

### 2.3 UC-ENV-* (환경 도메인, 8개) — TASK-002 ⏸ placeholder 포함

| UC ID | 명령 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| **UC-ENV-001** | `myharness env setup <stack>` | `env-setup` | 스택별 부트스트랩 (brew/asdf/dotfiles) | **§3 detailed** |
| UC-ENV-002 | `myharness env install <pkgs>` | `env-installer` | 의존성 설치 (with idempotency) | catalog |
| UC-ENV-003 | `myharness env shell "<cmd>"` | `env-shell` | 셸 명령 + LLM 분석 | catalog |
| UC-ENV-004 | `myharness env diagnose` | `env-diagnose` | 환경 진단 (path/version/permission) | catalog |
| UC-ENV-005 | `myharness env doctor` | `env-diagnose` (대화형) | 진단 + 자동 fix 제안 | catalog |
| UC-ENV-006 | `myharness env runtime <action>` | `env-installer` + `env-setup` | asdf/rtx runtime 관리 | catalog — **runtime 목록은 TASK-002 ⏸** |
| UC-ENV-007 | `myharness env dotfiles <action>` | `env-setup` (dotfiles scope) | dotfiles repo sync | catalog — **dotfiles 경로는 TASK-002 ⏸** |
| UC-ENV-008 | `myharness env upgrade` | `env-installer` | 설치된 tool/dependency upgrade | catalog |

> **TASK-002 ⏸ 노트**: UC-ENV-006/007 의 Homebrew 패키지 / asdf 런타임 / dotfiles 경로는 **yklee 의 macOS 셋업 정보**가 필요. PROJECT_PROFILE.md §3.1 의 "TASK-002 채움 예정" 영역을 그대로 비워둠.

### 2.4 UC-AUTH-* (LLM provider auth + discover, D-38, 9개)

| UC ID | 명령 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| **UC-AUTH-001** | `myharness auth setup` | orchestrator + `provider-auto-config` skill | **모든 provider 일괄 discover + login wizard** | **§3 detailed** |
| UC-AUTH-002 | `myharness auth list` | orchestrator (read-only) | 모든 provider status 조회 | catalog |
| UC-AUTH-003 | `myharness auth <provider>` | orchestrator (read-only) | 한 provider status | catalog |
| UC-AUTH-004 | `myharness auth <provider> login` | `provider-auto-config` skill | OAuth/API key 초기화 | catalog |
| UC-AUTH-005 | `myharness auth <provider> logout` | `provider-auto-config` skill | auth 제거 (keychain 에서 삭제) | catalog |
| UC-AUTH-006 | `myharness auth <provider> set-key <key>` | `provider-auto-config` skill | API key 수동 설정 (env 또는 keychain) | catalog |
| UC-AUTH-007 | `myharness auth <provider> set-key --from-keychain` | `provider-auto-config` skill | keychain 에서 가져오기 (장소별 alias) | catalog |
| UC-AUTH-008 | `myharness auth <provider> test` | `provider-auto-config` skill | 연결 테스트 (ping model, latency 측정) | catalog |
| UC-AUTH-009 | `myharness auth default <provider>` | orchestrator | primary 변경 (config.yaml 갱신) | catalog |
| **UC-AUTH-010** | **`myharness auth add-local`** | orchestrator (직접) | **로컬 LLM 서버 (Ollama/vLLM/LM Studio/llama.cpp) URL + 선택적 token 입력 → /v1/models probe → 모델 선택 TUI → ProviderRegistry 의 LocalLlm entry 의 base_url/default_model 갱신** | **§3 detailed, D-59 W16** |

> **D-38 노트**: UC-AUTH-001/008 의 `auth setup` / `auth <provider> test` 는 fallback chain 의 source — `~/.myharness/state/active-providers.yaml` 자동 갱신 (CONCEPT.md §5.5.3).
>
> **D-59 노트 (W16)**: UC-AUTH-010 (`auth add-local`) 은 OAuth flow 와 다른 **수동 endpoint 등록** sub-case. CONCEPT.md §5.5.1 의 discover + auth + save 3-단계 중 **수동 sub-case** (자동 discover 는 W7.3 `scan_local.rs` 가 담당). `provider-auto-config` skill (D-38) 영역 밖 — W16 은 v1 의 6 built-in provider 중 `LocalLlm` 의 1-shot 등록 UI.

### 2.5 UC-INSTALL-* (설치 / 배포, 6개)

| UC ID | 명령 / 동작 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| UC-INSTALL-001 | `curl ... \| bash` (macOS / Linux install.sh) | installer script | 단일 binary 설치 (cargo-dist 5 install paths 의 ①) | catalog |
| UC-INSTALL-002 | `irm ... \| iex` (Windows install.ps1) | installer script | Windows PowerShell 설치 (②) | catalog |
| UC-INSTALL-003 | `brew install --cask myharness` | installer (homebrew formula) | macOS Homebrew stable (③) | catalog |
| UC-INSTALL-004 | `brew install --cask myharness@latest` | installer (homebrew formula) | macOS Homebrew bleeding (④) | catalog |
| UC-INSTALL-005 | `winget install Yklee.Myharness` | installer (winget manifest) | Windows winget (⑤) | catalog |
| UC-INSTALL-006 | `apt/dnf/apk install myharness` | installer (linux pkg) | Linux 패키지 매니저 (대안) | catalog |

> **CONCEPT.md §5.3 노트**: 5 install paths 의 Auto-update 는 **native install 만 background**. brew/winget 은 수동.

### 2.6 UC-CFG-* (config / permission / hook, 10개)

| UC ID | 명령 / 동작 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| UC-CFG-001 | `myharness config show` | orchestrator (read-only) | `~/.myharness/config/config.yaml` 표시 | catalog |
| UC-CFG-002 | `myharness config edit` | orchestrator + `file-searcher` | editor (`$EDITOR`) 로 config 열기 | catalog |
| UC-CFG-003 | `myharness config set <key> <val>` | orchestrator | config key=value 갱신 | catalog |
| UC-CFG-004 | `myharness permission set <mode>` | orchestrator | 4 permission mode 변경 (default / acceptEdits / plan / bypassPermissions) | catalog |
| UC-CFG-005 | `myharness hook list` | orchestrator | `~/.myharness/hooks/*.md` 목록 + 활성 여부 | catalog |
| UC-CFG-006 | `myharness hook enable <name>` | orchestrator | hook 활성화 (markdown 1 file = 1 hook) | catalog |
| UC-CFG-007 | `myharness hook disable <name>` | orchestrator | hook 비활성화 | catalog |
| UC-CFG-008 | `myharness hook test <name>` | orchestrator + tool dry-run | hook dry-run (실제 tool 호출 없이 매칭만 확인) | catalog |
| UC-CFG-009 | `myharness secret set <provider>` | `provider-auto-config` skill (또는 OS keychain 위임) | keychain 에 secret 저장 (token 값 ❌ 표시) | catalog |
| UC-CFG-010 | `myharness dir` | orchestrator (read-only) | `~/.myharness/` 디렉토리 트리 표시 | catalog |

> **CONCEPT.md §5.4 노트**: secret token 값은 **메모리/문서/git 저장 ❌** (D-06 정책). UC-CFG-009 는 keychain slot 이름만 표시하고 값은 표시 ❌.

### 2.7 UC-MAINT-* (유지보수, 8개)

| UC ID | 명령 / 동작 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- |
| UC-MAINT-001 | `myharness log tail [N]` | orchestrator (read-only) | `~/.myharness/log.jsonl` 최근 N줄 표시 | catalog |
| UC-MAINT-002 | `myharness log query <filter>` | orchestrator + `file-searcher` | log.jsonl filter (jsonpath) | catalog |
| UC-MAINT-003 | `myharness state show` | orchestrator (read-only) | `~/.myharness/state/current.yaml` 표시 | catalog |
| UC-MAINT-004 | `myharness state reset` | orchestrator | state 초기화 (주의: task history 손실) | catalog |
| UC-MAINT-005 | `myharness handoff write` | orchestrator (handoff 형식) | `~/.myharness/handoff/<session>.md` 작성 (D-26) | catalog |
| UC-MAINT-006 | `myharness handoff read` | orchestrator (read-only) | 최근 handoff 표시 | catalog |
| UC-MAINT-007 | `myharness memory show [topic]` | orchestrator | auto memory dump (LLM Wiki v1.5+ 에서 확장) | catalog |
| UC-MAINT-008 | `myharness cache clear` | orchestrator | `~/.myharness/cache/` 비우기 (regenerable) | catalog |

### 2.8 UC-LOOP-* + UC-SINGLE-* (mode-specific, 4개)

| UC ID | mode | 명령 / 동작 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- | --- |
| **UC-LOOP-001** | loop | `myharness --mode=loop --goal "<text>" --max-iterations=N` | orchestrator + sub-agent + 무한루프 | **ralph-wiggum 패턴 (D-29)** — goal 달성까지 자동 반복 | **§3 detailed** |
| UC-LOOP-002 | loop | `--success-criteria "<text>"` | orchestrator (LLM judge) | success 평가 기준 (UC-LOOP-001 의 stop condition) | catalog |
| UC-LOOP-003 | loop | `--max-iterations N` | orchestrator (counter) | 최대 반복 (default 20) + user interrupt | catalog |
| UC-SINGLE-001 | single | `myharness --mode=single ask "<question>"` | main agent only | 단일 에이전트, sub-agent spawn ❌, context 직접 처리 | catalog |

### 2.9 UC-CTX-* (context 압축, D-30 + D-27, 3개 — v1 우선)

| UC ID | 계층 | 명령 / 동작 | sub-agent(s) | 핵심 시나리오 | 비고 |
| --- | --- | --- | --- | --- | --- |
| UC-CTX-001 | Layer 1 (필수, D-30) | auto (token budget 추적) | orchestrator (always-on) | 한계 80% 도달 시 truncate / summarize / hybrid | catalog |
| UC-CTX-002 | Layer 1 (필수, D-30) | `myharness /compact` (slash) | orchestrator | user-callable 수동 압축 | catalog |
| UC-CTX-003 | Layer 2 (선택, D-27) | `config.yaml` 의 `context.builtin.enabled: true` | orchestrator + 알고리즘 | headroom 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) | catalog |

> **TASK-007 ✅ (D-37) 노트**: CCR + Kompress-base 는 v1.5+ 이므로 §8 OOS 에 명시. 본 문서의 UC-CTX-* 는 v1 우선 3 알고리즘만.

### 2.10 Catalog 합계

| Prefix | 개수 | 비고 |
| --- | --- | --- |
| UC-CODE-* | 10 | §3 detailed 1개 (UC-CODE-001) |
| UC-SERVER-* | 8 | §3 detailed 1개 (UC-SERVER-001) / TASK-002 ⏸ 일부 |
| UC-ENV-* | 8 | §3 detailed 1개 (UC-ENV-001) / TASK-002 ⏸ 일부 |
| UC-AUTH-* | 9 | §3 detailed 1개 (UC-AUTH-001) |
| UC-INSTALL-* | 6 | catalog only |
| UC-CFG-* | 10 | catalog only |
| UC-MAINT-* | 8 | catalog only |
| UC-LOOP-* | 3 | §3 detailed 1개 (UC-LOOP-001) |
| UC-SINGLE-* | 1 | catalog only |
| UC-CTX-* | 3 | catalog only |
| **합계** | **66** | 5 detailed + 61 catalog index |

---

## 3. 핵심 use case 상세 (5개)

> **선정 기준**: (a) CONCEPT.md §5.2 의 12 명령 중 각 도메인 representative 1개씩 (code/server/env) + (b) §5.5.2 + D-38 의 auth discover+login (cross-cutting) + (c) §5.10 의 loop mode (D-29 ralph-wiggum).
>
> 각 detailed UC 는 **actor / 사전조건 / 흐름 (정상) / 사후조건 / 확장 (mode 별) / 예외 (§7 연결) / §10 acceptance** 구조.

### 3.1 UC-CODE-001: PR multi-aspect code review

**명령**: `myharness code review <pr-url>` (CONCEPT.md §5.2, claude-code 13.1 + 13.22)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator (A2, mode=orchestrator default)
- sub-agents: `code-reviewer` (lead) + `git-operator` (PR metadata) + `file-searcher` (changed files enumeration) (A2)
- external: LLM provider (A3b, primary + D-38 fallback chain) + GitHub MCP server (A3a/plugin via MCP, e.g. `mcp__github__pr_diff`)

**사전조건**:
- yklee 가 GitHub 인증됨 (`gh auth status` 또는 `GITHUB_TOKEN` env 또는 `mcp__github__*` MCP)
- `~/.myharness/config/config.yaml` 에 `llm.primary` 설정됨
- `~/.myharness/hooks/` 중 `security-pattern.md` 활성 (선택)

**흐름 (정상)**:
1. **CLI parse** — orchestrator 가 `<pr-url>` 파싱 → PR repo/owner/number 식별
2. **PR metadata fetch** — `git-operator` sub-agent 가 `gh pr view <url>` 또는 `mcp__github__get_pull_request` 호출 → head SHA, base ref, title, body
3. **Changed files enumeration** — `file-searcher` sub-agent 가 diff stats (added/modified/deleted files) 추출
4. **Diff fetch** — `mcp__github__get_pull_request_diff` 또는 `gh pr diff <url>` 로 patch 텍스트
5. **Multi-aspect review plan** — orchestrator 가 `code-reviewer` sub-agent 에 3-aspect prompt 위임:
   - **Bugs**: logic error, edge case, race condition
   - **Style**: naming, structure, idiomatic patterns
   - **Tests**: coverage gap, missing test cases
6. **LLM call (primary)** — D-15 fallback chain 으로 primary provider 호출. 실패 시 fallback.
7. **Hook check** — review 결과에 sensitive 패턴 (e.g., secret 의심 string) 감지 시 `security-pattern.md` hook 발동 → 경고 표시
8. **Result aggregation** — orchestrator 가 3-aspect 결과를 한국어 요약 + markdown 형식으로 통합
9. **Output** — stdout 출력 + `~/.myharness/handoff/<timestamp>_code_review_<pr>.md` 저장 (D-26)
10. **Event log** — `~/.myharness/log.jsonl` 에 `{event: "code_review", pr, aspects, latency, provider, fallback_used: bool}` append

**사후조건**:
- yklee 가 한국어 요약 + 발견된 이슈 목록 (severity 별) + 권장 action 확인
- `handoff/<timestamp>_code_review_<pr>.md` 작성됨
- log.jsonl 에 이벤트 기록됨

**Mode 별 확장**:
- `mode=orchestrator` (default): 위 흐름 그대로
- `mode=single`: sub-agent spawn ❌, main agent 가 직접 PR fetch + review. context 가 커서 큰 PR 에선 비추천
- `mode=loop`: `--goal "PR #<N> 의 모든 blocker 코멘트 해결"` 시 UC-LOOP-001 패턴과 결합 (review → fix → re-review)

**예외 (§7 연결)**:
- Provider fallback (D-38) → §7.1
- Context overflow (D-30) → §7.2
- Permission deny (edit 제안이 hook block) → §7.3
- Tool error (`mcp__github__*` 실패) → §7.5

**§10 acceptance**: UC-CODE-001-ACC-01 ~ ACC-08 (§10.1)

---

### 3.2 UC-SERVER-001: 프로세스/서비스 상태 점검

**명령**: `myharness server status [host]` (CONCEPT.md §5.2, goose 13.1 차용)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator (mode=orchestrator) + `server-status` sub-agent (A2)
- external: OS process list (A3c — `ps` / `systemctl` / `launchctl` / Windows `Get-Service`)

**사전조건**:
- **TASK-002 ⏸**: `[host]` 가 생략되면 local host. 원격 host 인 경우 SSH 별칭이 config 에 등록되어 있어야 함 (UC-SERVER-007). 본 문서는 host alias 목록을 발명 ❌.
- `server-status` sub-agent 권한 scope 에 `ps`, `systemctl` 등 read-only process query 명령 포함

**흐름 (정상)**:
1. **CLI parse** — `[host]` 옵션. 생략 시 local
2. **Host resolve** — `~/.myharness/config/config.yaml` 의 `server.hosts.<alias>` (TASK-002 ⏸) 또는 default = local
3. **Sub-agent dispatch** — orchestrator 가 `server-status` sub-agent spawn (1 sub-agent = 1 작업)
4. **Process enumeration** — sub-agent 가 platform 별 명령 실행:
   - macOS: `launchctl list` (user/root launchd)
   - Linux: `systemctl list-units --type=service --state=running` + `ps aux | head`
   - Windows: `Get-Service | Where-Object {$_.Status -eq 'Running'}` (PowerShell)
5. **Anomaly detection** — sub-agent 가 LLM 에 process list 전달 → "high CPU" / "zombie" / "unhealthy" 패턴 detection
6. **Output** — 표 형식 (`SERVICE | PID | STATUS | UPTIME | NOTE`) + 한국어 요약
7. **Event log** — `log.jsonl` 에 `{event: "server_status", host, services_count, anomalies_count, provider}`

**사후조건**:
- yklee 가 서비스 상태 표 + anomaly 목록 확인
- 4 permission mode 중 `default` 시 SSH password 입력 prompt 가능 (CONCEPT.md §5.4)

**Mode 별 확장**:
- `mode=orchestrator` (default): 위 흐름
- `mode=single`: sub-agent spawn ❌, main agent 가 직접 process list + LLM 분석
- `mode=loop`: `--goal "<host> 의 모든 unhealthy 서비스 재시작"` → UC-LOOP-001 + UC-SERVER-006

**예외**:
- SSH 연결 실패 → §7.5 (tool error)
- Permission deny (root-only process 조회) → §7.3 + sudo prompt

**§10 acceptance**: UC-SERVER-001-ACC-01 ~ ACC-06 (§10.2)

---

### 3.3 UC-ENV-001: 스택별 부트스트랩

**명령**: `myharness env setup <stack>` (CONCEPT.md §5.2, opencode 13.1 차용)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator + `env-setup` sub-agent (lead) + `env-installer` (의존성) + `env-diagnose` (사전 검증)
- external: OS package manager (A3c — Homebrew macOS / apt-dnf-apk Linux / winget-choco Windows) + asdf/rtx runtime (선택) + dotfiles repo (선택)

**사전조건**:
- **TASK-002 ⏸**: `<stack>` 의 구체적 정의 (e.g., "rust", "python-data", "node-fullstack") 와 yklee 의 dotfiles 경로 / asdf runtime 목록은 yklee 인프라 정보 필요. 본 문서는 **stack name 형식 + dispatch 로직**만 정의, stack 의 구체적 manifest 는 placeholder.
- `env-installer` sub-agent 의 idempotency 보장 (재실행해도 결과 동일) — PROJECT_PROFILE.md §4 검증 포인트 정합

**흐름 (정상)**:
1. **CLI parse** — `<stack>` 식별 (e.g., `rust`, `python-data`)
2. **Stack manifest resolve** — `~/.myharness/config/stacks/<stack>.yaml` (TASK-002 ⏸ — placeholder) 또는 built-in default (v1.5+ plugin 가능)
3. **Pre-diagnose** — `env-diagnose` sub-agent 가 현재 환경 (path, version, permission) 스냅샷
4. **Sub-agent dispatch** — orchestrator 가 `env-setup` sub-agent spawn
5. **Bootstrap 실행** — platform 별:
   - macOS: `brew bundle --file=<stack>.Brewfile` 또는 `brew install <pkgs>`
   - Linux Debian: `apt-get install -y <pkgs>`
   - Linux RHEL: `dnf install -y <pkgs>`
   - Linux Alpine: `apk add --no-cache <pkgs>`
   - Windows: `winget install <pkgs>` 또는 `choco install <pkgs>`
6. **Runtime install** (선택) — asdf plugin add + asdf install (e.g., rust 1.78, python 3.12)
7. **Dotfiles sync** (선택) — `git pull <dotfiles-repo>` + stow/symlink (TASK-002 ⏸)
8. **Post-diagnose** — `env-diagnose` 가 재실행 → 변화 확인
9. **Smoke test** — PROJECT_PROFILE.md §4 "설치 직후 smoke test" 정합 — 설치된 tool 의 `--version` 검증
10. **Output** — 한국어 요약 + `~/.myharness/handoff/<timestamp>_env_setup_<stack>.md`
11. **Event log** — `{event: "env_setup", stack, pkgs_installed, runtimes, smoke_test_result}`

**사후조건**:
- yklee 의 terminal 에 새 PATH 적용 (`.zshrc` / `.bashrc` / PowerShell profile reload 필요할 수 있음)
- `~/.myharness/memory/auto/<stack>-setup.md` 에 학습 노트 저장 (D-26 auto memory)

**Mode 별 확장**:
- `mode=orchestrator` (default): 위 흐름
- `mode=single`: sub-agent spawn ❌
- `mode=loop`: `--goal "<stack> 환경 100% green"` 시 UC-LOOP-001 + 반복 diagnose + fix

**예외**:
- `apt-get` 권한 부족 → §7.3 (sudo prompt)
- 이미 설치된 package version 충돌 → §7.5 (tool error, 사용자에게 confirm)
- 네트워크 불가 → §7.5

**§10 acceptance**: UC-ENV-001-ACC-01 ~ ACC-07 (§10.3)

---

### 3.4 UC-AUTH-001: provider discover + login wizard (D-38)

**명령**: `myharness auth setup` (CONCEPT.md §5.5.2, D-38)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator (main) + `provider-auto-config` skill (CONCEPT.md §5.5.2 의 NEW skill, D-38) + 6 provider 정적 등록 (CONCEPT.md §5.5.1)
- external: LLM provider 6종 (A3b) + OS keychain (A3c — keyring crate) + local LLM server (A4-LOC)

**사전조건**:
- yklee 가 최소 1개 provider 의 API key 보유 (또는 local LLM server 실행 중)
- `~/.myharness/state/auth/` 디렉토리 존재 (없으면 orchestrator 가 startup 시 생성)

**흐름 (정상)**:
1. **CLI parse** — `auth setup` (no-arg). 별도 arg 있으면 UC-AUTH-002/003/004 등 (catalog)
2. **Skill auto-invoke** — orchestrator 가 `provider-auto-config` skill 의 SKILL.md frontmatter trigger 매칭 (`auth`, `setup`, `startup`, `fallback failed`) → skill load
3. **Discover phase**:
   - **Env vars scan**: `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GOOGLE_API_KEY` / `DEEPSEEK_API_KEY` / `MINIMAX_API_KEY` (값 ❌ 표시, presence ✅ 표시)
   - **OS keychain scan**: `keyring` crate 가 `myharness-<provider>` slot 조회 (값 ❌, presence ✅)
   - **Local LLM server scan**: `http://localhost:11434/v1` (Ollama) / `:1234/v1` (LM Studio) / `:8000/v1` (vLLM) health check
   - **MCP provider scan**: `mcp__*` 동적 discover (D-38 v1.5+; v1 Phase 1 는 정적 6 provider 한정)
4. **Per-provider auth status**:
   - `anthropic.yaml` / `openai.yaml` / ... 6개 파일 생성/갱신
   - 각 파일: `status: authenticated | logged_out | error | not_configured | available` (CONCEPT.md §5.5.2 의 status enum)
5. **Active providers build** — `~/.myharness/state/active-providers.yaml` 자동 생성:
   ```yaml
   discovered_at: 2026-06-07T14:00:00+09:00
   active:
     - anthropic    # env 또는 keychain detected
     - ollama       # local server running
   inactive:
     - openai       # not_configured
     - minimax      # logged_out (또는 not detected)
   fallback_order: [anthropic, ollama, openai, deepseek, gemini]  # config 우선순위 + active filter
   ```
6. **Login wizard** (interactive):
   - yklee 에게 미인증 provider 목록 표시
   - 각 provider 별 login method (API key paste / OAuth flow / --from-keychain)
   - 4 permission mode 에 따라 자동/수동 (default 시 매번 confirm)
7. **Test** (선택) — `myharness auth <provider> test` 로 ping model, latency 측정, 결과 yaml 기록
8. **Persist + reload** — `active-providers.yaml` 저장 + 다음 LLM call 부터 적용
9. **Output** — 한국어 요약 + 권장 primary (`fallback_order[0]`)
10. **Event log** — `{event: "auth_setup", providers_discovered, providers_authenticated, fallback_order}`

**사후조건**:
- yklee 가 6 provider status 한눈에 확인
- `active-providers.yaml` 가 다음 LLM call 의 fallback chain source (CONCEPT.md §5.5.3)

**Mode 별 확장**:
- `mode=orchestrator` (default): 위 흐름
- `mode=single` / `mode=loop`: 동일 (sub-agent 미사용)

**예외**:
- 모든 provider logged_out → fallback chain empty → §7.1 (provider fallback exhausted)
- Keychain 접근 권한 거부 → §7.3

**§10 acceptance**: UC-AUTH-001-ACC-01 ~ ACC-09 (§10.4)

---

### 3.5 UC-AUTH-010: 로컬 LLM 서버 등록 (`auth add-local`, D-59 W16)

**명령**: `myharness auth add-local` (no-arg, 완전 interactive wizard)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator (mode 무관, 단일 sub-command) + `myharness-llm::register_local_provider` API + `inquire` UI crate

**사전조건**:
- 사용자 입력이 tty (stdin/stdout 모두 tty). 비대화형 (`auth add-local < /dev/null`) 실행 시 graceful error.
- 로컬 LLM 서버 (Ollama/vLLM/LM Studio/llama.cpp) 가 **이미 실행 중** (URL probe 시 available 해야 함). 미실행 시 §7.1 (connection refused) + exit code 1.

**흐름 (정상)**:
1. **CLI parse** — `auth add-local` (no-arg, AuthAction::AddLocal enum)
2. **URL 입력 (inquire Text prompt)**:
   - placeholder: `http://localhost:11434/v1` (Ollama default) / `http://localhost:8000/v1` (vLLM) / `http://localhost:1234/v1` (LM Studio) / `http://localhost:8080/v1` (llama.cpp)
   - 검증: `url::Url::parse` 성공해야 진행. 실패 시 재입력 (loop, ESC 로 cancel)
3. **Token 입력 (inquire Text prompt, `is_password = true` style — input masking)**:
   - placeholder hint: "press Enter to skip (Ollama 는 보통 불요)"
   - 빈 입력 = None (token 없이 등록)
   - 비어있지 않으면 `KeyringAuthStore::set(ProviderId::LocalLlm, &token).await` 호출 (W7.2 in-memory cache + macOS keyring / Win credential / Linux libsecret fallback)
4. **`GET {base_url}/models` probe (reqwest, 3s timeout)**:
   - 성공 (200) → JSON `{"data": [{"id": "..."}, ...]}` 파싱 → 모델 목록 확보
   - 실패 (connection refused / 4xx / 5xx) → 에러 출력 + `continue? (y/n)` prompt. n 선택 시 abort
5. **모델 선택 (inquire Select prompt, arrow-key)**:
   - 모델 0개 → "no models found at {url}" + abort
   - 1개+ → arrow-key select UI 표시
   - Enter → 선택 확정
6. **`ProviderRegistry` 갱신**:
   - `~/.myharness/providers.toml` 로드 (없으면 with_builtins() 시작)
   - `LocalLlm` entry 의 `base_url` (입력값) + `default_model` (선택한 모델 id) + `available_models` (전체 목록) 갱신
   - `save_to_path()` 로 atomic write
7. **결과 출력 (한국어)**:
   ```
   ✓ 로컬 LLM 등록 완료
     서버: <입력 URL>
     모델: <선택 모델 id> (전체 N개 사용 가능)
     저장: ~/.myharness/providers.toml
   ```
8. **§10 acceptance**: UC-AUTH-010-ACC-01 ~ ACC-05 (§10.4)

**예외**:
- URL parse 실패 / connection refused / 4xx 5xx / 모델 0개 / inquire 사용자 cancel (ESC) → §7.1 (graceful abort, exit code 1)
- Keychain backend None (Linux libsecret 미설치) → env var hint 메시지 출력 후 in-memory cache 로 진행 (W7.2 graceful fallback 재사용)
- 비-tty 환경 실행 → "auth add-local 은 interactive 만 지원 — stdin redirect ❌" 에러 + exit 1

**Mode 별 동작**:
- `mode=orchestrator` (default): 위 흐름 그대로
- `mode=single` / `mode=loop`: 동일 (auth subcommand 는 mode 무관, §5.3 dispatch matrix 의 sub-agent = 없음)

**Sub-agent dispatch**: **없음** — `auth add-local` 은 orchestrator 가 직접 처리. UC-AUTH-001 의 `provider-auto-config` skill 도 호출 ❌ (W16 은 v1.5 의 auto-fallback 갱신 영역 밖).

**TASK-002 의존성**: ❌ (TASK-002 는 server/env 인프라. local LLM 서버 자체는 yklee 인프라와 무관 — Ollama/vLLM 모두 cross-platform standalone binary)

**관련 use case**: UC-AUTH-001 (provider discover+login), UC-AUTH-008 (auth test) — W16 은 LocalLlm 1-shot 수동 등록. UC-AUTH-001 의 batch wizard 와 별도 경로.

---

### 3.6 UC-LOOP-001: loop mode (ralph-wiggum, D-29)

**명령**: `myharness --mode=loop --goal "<text>" --max-iterations=N [--success-criteria "<text>"]` (CONCEPT.md §5.10 의 loop row)

**Actor**:
- 주 actor: yklee (A1)
- 시스템: orchestrator (mode=loop) + sub-agent pool (작업별 dispatch) + LLM judge (success 평가 시)

**사전조건**:
- `--goal` 필수. yklee 가 달성 목표 명시 (e.g., "fix all failing tests", "PR #482 의 모든 blocker 코멘트 해결")
- `--max-iterations` 선택 (default 20) — run-away 방지
- `--success-criteria` 선택 — LLM 이 success 평가 기준 (없으면 orchestrator 가 goal text 그대로 사용)

**흐름 (정상)**:
1. **CLI parse** — `--mode=loop --goal "<text>" [--max-iterations N] [--success-criteria "<text>"]` + 추가 명령 인자 (e.g., `code review` sub-command)
2. **Loop init**:
   - `iterations: 0`
   - `current_state: in_progress`
   - `goal`, `success_criteria` 보관
   - 첫 LLM call 에 goal + 현재 state + 이전 iteration log 주입
3. **Iteration N**:
   - orchestrator 가 goal 분석 → sub-agent dispatch (e.g., goal="fix all TODO comments" → `code-searcher` (TODO 검색) + `code-implementer` (해결))
   - sub-agent 결과 → orchestrator 가 통합
   - LLM judge (또는 deterministic check) 가 success_criteria 평가
   - `success: bool`, `reason: <text>` 반환
4. **Stop condition** (다음 중 하나):
   - `success == true` → 종료, 성공 보고
   - `iterations == max_iterations` → 종료, partial 보고
   - yklee 가 Ctrl+C → 종료, interrupted 보고
5. **Output** — 각 iteration 의 한국어 요약 + 최종 iteration summary + handoff
6. **Event log** — `{event: "loop_iteration", iteration, sub_agents, success, reason}`

**사후조건**:
- `state/loop-<goal-slug>.yaml` 에 iteration log 저장 (resume 가능)
- `handoff/<timestamp>_loop_<goal-slug>.md` 에 최종 보고

**Mode 별 확장**:
- `mode=loop` 한정 (UC-SINGLE-001 / UC-ORCHESTRATOR-001 은 별도)
- `--success-criteria` 가 deterministic check 가능 시 (e.g., "all tests pass" → `cargo test` exit 0) LLM judge 생략하고 tool exit code 사용

**예외**:
- `--max-iterations` 초과 → 종료 + partial
- Provider fallback exhausted (§7.1) → 즉시 종료 + error
- User Ctrl+C → 즉시 종료 + interrupted

**§10 acceptance**: UC-LOOP-001-ACC-01 ~ ACC-06 (§10.5)

---

## 4. 3 agent mode × use case 매트릭스

**3 mode** (CONCEPT.md §5.10):
- `orchestrator` (default)
- `single` (opt-in, `--mode=single`)
- `loop` (opt-in, `--mode=loop`, D-29 ralph-wiggum)

### 4.1 Mode 정의 요약

| Mode | 기본? | sub-agent spawn | 적합 use case | 부적합 use case |
| --- | --- | --- | --- | --- |
| `orchestrator` | ✅ default | ✅ yes | UC-CODE-001, UC-SERVER-001, UC-ENV-001, UC-AUTH-001 | (없음 — default 는 모든 작업 가능) |
| `single` | 🟡 opt-in | ❌ no | UC-SINGLE-001 (`ask`), UC-CODE-007 (단일 파일 분석) | UC-CODE-001 (PR review, multi-aspect), UC-LOOP-* |
| `loop` | 🟡 opt-in | ✅ yes (iteration 마다) | UC-LOOP-001, UC-CODE-001 + loop, UC-ENV-001 + loop | UC-SINGLE-001 (single 과 loop 는 mutually exclusive) |

### 4.2 Mode × Use case prefix dispatch

| Prefix | orchestrator | single | loop | 비고 |
| --- | --- | --- | --- | --- |
| UC-CODE-* (10) | ✅ | 🟡 1-2개만 | 🟡 loop 결부 시 | single 은 UC-CODE-006/007 (단일 파일 검색/분석) |
| UC-SERVER-* (8) | ✅ | 🟡 1-2개만 | 🟡 loop 결부 시 | single 은 UC-SERVER-005 (단일 health) |
| UC-ENV-* (8) | ✅ | 🟡 1-2개만 | 🟡 loop 결부 시 | single 은 UC-ENV-003 (단일 shell) |
| UC-AUTH-* (9) | ✅ | 🟡 가능 | ❌ 권장 ❌ | auth 는 multi-step 이라 single 도 가능하나, fallback chain 동적 구성 = orchestrator 권장 |
| UC-INSTALL-* (6) | ✅ | ❌ | ❌ | installer script 라 sub-agent 무관, 그러나 orchestrator 의 parent log 통합 필요 |
| UC-CFG-* (10) | ✅ | 🟡 read-only 가능 | ❌ | UC-CFG-001/002/003 (read) 는 single 가능, write (set/permission) 는 orchestrator |
| UC-MAINT-* (8) | ✅ | 🟡 read-only 가능 | ❌ | log/state/handoff read 는 single 가능, write (reset/handoff write) 는 orchestrator |
| UC-LOOP-* (3) | ❌ (loop mode 한정) | ❌ | ✅ | mutually exclusive |
| UC-SINGLE-* (1) | ❌ (single mode 한정) | ✅ | ❌ | mutually exclusive |
| UC-CTX-* (3) | ✅ | ✅ (auto) | ✅ (auto) | context 압축은 모든 mode 에서 auto (UC-CTX-001), manual (/compact, UC-CTX-002) 도 모든 mode 가능 |

### 4.3 Mode flag CLI syntax

```bash
# default (orchestrator)
myharness code review <pr>

# explicit orchestrator
myharness --mode=orchestrator code review <pr>

# single
myharness --mode=single ask "what does this function do?"
myharness --mode=single code search "TODO"

# loop
myharness --mode=loop --goal "fix all failing tests" --max-iterations=20 code test
myharness --mode=loop --goal "PR #482 의 blocker 코멘트 해결" --success-criteria "all threads resolved" code review 482
```

### 4.4 Mode 호환성 규칙 (acceptance)

- `orchestrator` + `single` + `loop` 동시 지정 → CLI parse error
- `--mode=single` + sub-command 가 multi-step (e.g., `code review`) → CLI 경고 + single 로 강제 (sub-agent spawn ❌, 단일 agent 가 모든 aspect 처리)
- `--mode=loop` + `--goal` 누락 → CLI error
- `--mode=loop` + `--max-iterations` 가 `0` 또는 음수 → CLI error
- `--mode=loop` + `--max-iterations > 100` → CLI 경고 (사용자 confirm)

---

## 5. Built-in sub-agent ↔ use case dispatch 매트릭스

**15 built-in sub-agents** (CONCEPT.md §5.11) × **66 use case** (catalog) 의 participant matrix.

### 5.1 15 sub-agent 목록 + 소속 도메인

| # | sub-agent | 도메인 | CONCEPT.md §5.11 row |
| --- | --- | --- | --- |
| 1 | `code-reviewer` | 코드 | 코드 도메인 1 |
| 2 | `code-implementer` | 코드 | 코드 도메인 2 |
| 3 | `code-tester` | 코드 | 코드 도메인 3 |
| 4 | `code-refactorer` | 코드 | 코드 도메인 4 |
| 5 | `code-searcher` | 코드 | 코드 도메인 5 |
| 6 | `server-status` | 서버 | 서버 도메인 1 |
| 7 | `log-analyzer` | 서버 | 서버 도메인 2 |
| 8 | `deployer` | 서버 | 서버 도메인 3 |
| 9 | `config-manager` | 서버 | 서버 도메인 4 |
| 10 | `env-setup` | 환경 | 환경 도메인 1 |
| 11 | `env-installer` | 환경 | 환경 도메인 2 |
| 12 | `env-shell` | 환경 | 환경 도메인 3 |
| 13 | `env-diagnose` | 환경 | 환경 도메인 4 |
| 14 | `git-operator` | Utility | Utility 1 |
| 15 | `file-searcher` | Utility | Utility 2 |

### 5.2 Sub-agent ↔ use case 매트릭스 (full)

| sub-agent | primary use case | secondary use case | 비고 |
| --- | --- | --- | --- |
| **`code-reviewer`** | UC-CODE-001 (PR review), UC-CODE-007 (단일 파일 분석) | UC-CODE-010 (diff 분석) | multi-aspect prompt (bugs / style / tests) |
| **`code-implementer`** | UC-CODE-002 (implement), UC-CODE-008 (format) | UC-CODE-005 (refactor), UC-CODE-009 (deps) | multi-file 변경 권한 |
| **`code-tester`** | UC-CODE-003 (test) | — | test runner + LLM 분석 |
| **`code-refactorer`** | UC-CODE-005 (refactor) | — | AST-aware (tree-sitter, §5.6) |
| **`code-searcher`** | UC-CODE-006 (search) | UC-CODE-007 (단일 파일 분석) | ripgrep / tree-sitter |
| **`server-status`** | UC-SERVER-001 (status), UC-SERVER-005 (health), UC-SERVER-008 (metrics) | — | process list / health endpoint |
| **`log-analyzer`** | UC-SERVER-002 (logs), UC-SERVER-005 (health) | — | 이상 패턴 detection |
| **`deployer`** | UC-SERVER-003 (deploy), UC-SERVER-006 (restart) | — | ssh / k8s / docker |
| **`config-manager`** | UC-SERVER-004 (config), UC-SERVER-006 (restart) | — | with backup (PROJECT_PROFILE.md §4) |
| **`env-setup`** | UC-ENV-001 (setup), UC-ENV-007 (dotfiles) | UC-ENV-006 (runtime) | stack manifest 기반 |
| **`env-installer`** | UC-ENV-002 (install), UC-ENV-008 (upgrade) | UC-ENV-006 (runtime) | idempotency 보장 |
| **`env-shell`** | UC-ENV-003 (shell) | — | 셸 명령 + LLM 분석 |
| **`env-diagnose`** | UC-ENV-004 (diagnose), UC-ENV-005 (doctor) | — | path / version / permission |
| **`git-operator`** | UC-CODE-004 (commit), UC-CODE-001 (PR metadata), UC-CODE-010 (diff) | — | 모든 git workflow 의 foundation |
| **`file-searcher`** | 모든 use case 의 tool (Read/Grep/Glob dispatch) | UC-CODE-006, UC-MAINT-002 | utility, 거의 모든 UC 에 등장 |

### 5.3 Use case → sub-agent fan-out (대표 사례)

#### UC-CODE-001 (PR review) — multi-sub-agent fan-out

```
UC-CODE-001 (orchestrator)
  ├─ git-operator   → PR metadata + diff fetch
  ├─ file-searcher  → changed files enumeration
  └─ code-reviewer  → multi-aspect review (lead)
       └─ (optional) code-tester → test coverage gap 분석
       └─ (optional) code-searcher → 영향받는 다른 file 검색
```

#### UC-ENV-001 (env setup) — multi-sub-agent fan-out

```
UC-ENV-001 (orchestrator)
  ├─ env-diagnose  → pre-check (path/version/permission)
  ├─ env-setup     → stack manifest 실행 (lead)
  ├─ env-installer → 의존성 설치
  └─ env-diagnose  → post-check (smoke test)
```

#### UC-AUTH-001 (auth setup) — sub-agent 없음, skill 만

```
UC-AUTH-001 (orchestrator)
  └─ provider-auto-config skill  (CONCEPT.md §5.5.2, D-38)
       └─ 6 provider × env/keychain/local LLM scan
       └─ active-providers.yaml 생성
```

#### UC-LOOP-001 (loop) — iteration 마다 dynamic fan-out

```
UC-LOOP-001 (orchestrator, iteration N)
  └─ (goal 기반 dynamic dispatch) — N=1..max_iterations
       └─ e.g., goal="fix all TODO comments" →
            ├─ code-searcher (TODO 위치 열거)
            ├─ code-implementer (해결)
            └─ code-tester (regression)
```

### 5.4 Sub-agent 권한 scope (CONCEPT.md §5.11, "1 sub-agent = 1 작업")

| sub-agent | 허용 tool scope | 거부 tool scope |
| --- | --- | --- |
| `code-reviewer` | Read, Grep, Glob, mcp__github__* (read) | Write, Edit, Bash (eval) |
| `code-implementer` | Read, Grep, Glob, Write, Edit, Bash (build) | mcp__github__push (PR 생성은 user confirm) |
| `code-tester` | Bash (test runner), Read (결과) | Write, Edit |
| `code-refactorer` | Read, Grep, Glob, Write, Edit | Bash (eval), mcp__github__* |
| `code-searcher` | Read, Grep, Glob | Write, Edit, Bash |
| `server-status` | Bash (ps / systemctl / launchctl / Get-Service, read-only) | Write, Edit |
| `log-analyzer` | Read, Bash (tail / journalctl) | Write, Edit |
| `deployer` | Bash (ssh / kubectl / docker, write scope) | Read, Edit (config 는 config-manager 위임) |
| `config-manager` | Read, Write, Edit (config file scope) | Bash (deploy 는 deployer 위임) |
| `env-setup` | Bash (brew / apt / dnf / apk / winget / choco) | Read, Edit (manifest file read only) |
| `env-installer` | Bash (brew install / apt install / etc) | Read, Edit |
| `env-shell` | Bash (user-provided cmd, scope yklee confirm) | Read, Edit |
| `env-diagnose` | Bash (read-only version / path / which) | Write, Edit |
| `git-operator` | Bash (git), mcp__github__* (read + push scope) | Write (non-git files), Edit |
| `file-searcher` | Read, Grep, Glob | Write, Edit, Bash |

> **권한 scope 의 "Bash (build)" 의미**: e.g., `cargo build` / `npm test` / `pytest` 등 sub-agent 의 역할에 부합하는 read-ish or build 명령. yklee 가 4 permission mode 로 추가 제약 가능.

### 5.5 Sub-agent dispatch 로직 (orchestrator)

**orchestrator 의 dispatch 의사결정 (CONCEPT.md §5.11 마지막 항목)**:

1. **user 명령 분석** — CLI 인자 + sub-command
2. **도메인 매칭** — `code` / `server` / `env` / `auth` / `install` / `cfg` / `maint`
3. **카테고리 매칭** — UC-* 의 primary sub-agent 식별
4. **Fan-out** — secondary sub-agent 자동 spawn (e.g., UC-CODE-001 → git-operator + file-searcher + code-reviewer)
5. **통합** — sub-agent 결과를 한국어 요약 + structured markdown 으로 user 에게 보고
6. **Handoff** — `~/.myharness/handoff/<timestamp>_<uc>.md` 자동 저장

---

## 6. Extension points

**3 종류의 extension** (CONCEPT.md §5.7 + §5.14, v1 = MCP 4 pre-config, v1.5+ = plugin 4-계층 + skill 6 built-in + provider-auto-config).

### 6.1 MCP server (v1, 4 pre-config)

**CONCEPT.md §5.14 의 "v1: 기본 (3-4개 MCP server pre-config)"** 정합.

| MCP server | 노출 tool 예시 | Use case (primary) | Use case (secondary) |
| --- | --- | --- | --- |
| `filesystem` | `mcp__filesystem__read_file`, `mcp__filesystem__write_file`, `mcp__filesystem__list_directory` | 모든 UC (Read/Write 도구) | UC-MAINT-001 (log read) |
| `git` | `mcp__git__status`, `mcp__git__diff`, `mcp__git__commit`, `mcp__git__log` | UC-CODE-004 (commit), UC-CODE-001 (PR metadata), UC-CODE-010 (diff) | 모든 git workflow |
| `shell` | `mcp__shell__bash`, `mcp__shell__exec` | 모든 UC (Bash 도구), UC-ENV-001/002/003, UC-SERVER-001/002/003 | sub-agent 권한 scope |
| `github` (선택) | `mcp__github__get_pull_request`, `mcp__github__create_pr`, `mcp__github__list_issues` | UC-CODE-001 (PR review), UC-CODE-004 (commit + push) | UC-MAINT-002 (log filter) |

**구현 (Rust 1안)**: `rmcp` 1.4 (CONCEPT.md §5.5.4 / §11.3 D-36)

**Config 위치**: `~/.myharness/mcp.json` (CONCEPT.md §5.12)

**Use case (auto tool exposure, CONCEPT.md §5.14)**: MCP server 의 tool 이 우리 sub-agent 의 tool registry 에 자동 등록 → `mcp__filesystem__read_file` 등.

### 6.2 Skill (v1.5+, 6 built-in + 1 NEW)

**CONCEPT.md §5.14 의 "Built-in skills catalog (3-도메인)"** 정합.

| Skill | 도메인 | invoke trigger (CONCEPT.md §5.14) | Use case (primary) | 비고 |
| --- | --- | --- | --- | --- |
| `code-review-best-practices` | 코드 | PR review, code review | UC-CODE-001 | multi-aspect prompt template |
| `git-workflow` | 코드 | commit, PR, branch | UC-CODE-004 | commit message convention |
| `server-health-check` | 서버 | status, health | UC-SERVER-001, UC-SERVER-005 | check list (CPU/mem/disk) |
| `log-pattern-analysis` | 서버 | log analysis | UC-SERVER-002 | anomaly patterns |
| `env-bootstrap` | 환경 | setup, install | UC-ENV-001, UC-ENV-002 | stack manifest conventions |
| `dotfiles-sync` | 환경 | dotfiles, shell config | UC-ENV-007 | (TASK-002 ⏸ placeholder) |
| **`provider-auto-config`** (D-38, NEW) | infra | startup / `auth` / fallback 실패 | UC-AUTH-001, UC-AUTH-006/007/008 | CONCEPT.md §5.5.2 의 동적 발견 + per-provider auth |

**위치**: `~/.myharness/skills/<name>/SKILL.md` (CONCEPT.md §5.12)

**SKILL.md 형식**: markdown + YAML frontmatter (`auto_invoke.triggers` + `auto_invoke.priority`) — CONCEPT.md §5.14 정합.

**Use case (skill auto-invoke)**: orchestrator 가 user 명령 / context keyword 와 skill 의 frontmatter trigger 매칭 → 자동 load.

### 6.3 Plugin (v1.5+, 4-계층)

**CONCEPT.md §5.7 의 "4 계층"** 정합.

| 계층 | 디렉토리 | v1 사용? | v1.5+ 사용? | Use case (예시) |
| --- | --- | --- | --- | --- |
| `commands/` | `~/.myharness/plugins/<name>/commands/` | ❌ | ✅ | 사용자 정의 slash command |
| `agents/` | `~/.myharness/plugins/<name>/agents/` | ❌ | ✅ | 사용자 정의 sub-agent (SYSTEM.md) |
| `skills/` | `~/.myharness/plugins/<name>/skills/` | ❌ | ✅ | plugin-scoped skill |
| `hooks/` | `~/.myharness/plugins/<name>/hooks/` | 🟡 local 가능 | ✅ | user hook (markdown rule) |

**v1 MVP**: local plugin only (`commands` + `hooks` 만). marketplace 는 v2+ OOS (CONCEPT.md §4.2).

**Use case (plugin)**: yklee 가 자체 plugin 작성 → `~/.myharness/plugins/<name>/` 에 install → orchestrator 가 자동 discover.

**marketplace OOS (v1)**: §8 에서 의도적 누락 use case 로 매핑.

### 6.4 Extension point 비교

| Extension | v1 사용 | v1.5+ 사용 | 위치 | Use case (대표) |
| --- | --- | --- | --- | --- |
| **MCP server** | ✅ 4 pre-config | ✅ unlimited | `~/.myharness/mcp.json` | UC-CODE-001 (mcp__github__*) |
| **Skill** | 🟡 6 built-in 만 | ✅ 6 built-in + user | `~/.myharness/skills/<name>/SKILL.md` | UC-AUTH-001 (provider-auto-config) |
| **Plugin (4-계층)** | 🟡 hook 만 | ✅ full 4-계층 | `~/.myharness/plugins/<name>/` | (v1.5+ 확장 use case) |

### 6.5 Extension point ↔ use case 매트릭스

| Use case | MCP server | Skill | Plugin (v1.5+) |
| --- | --- | --- | --- |
| UC-CODE-001 (PR review) | ✅ github MCP | ✅ code-review-best-practices | (optional) review prompt plugin |
| UC-SERVER-001 (status) | (없음, Bash 직접) | ✅ server-health-check | (optional) custom service list |
| UC-ENV-001 (setup) | (없음, Bash 직접) | ✅ env-bootstrap | (optional) stack manifest plugin |
| UC-AUTH-001 (auth setup) | (없음) | ✅ provider-auto-config (D-38) | (optional) provider-specific auth plugin |
| UC-LOOP-001 (loop) | (MCP tools 모두) | (모든 skill) | (optional) goal template plugin |

---

## 7. Exception flows

**5 exception category** (UC-3 §3 detailed 의 각 "예외" 와 매핑).

### 7.1 Provider fallback (D-38)

**Trigger**: primary LLM provider 호출 실패
**출처**: CONCEPT.md §5.5.3, §11.3 TASK-008 (D-38 결정)

**flow**:
1. LLM call 시 primary provider 호출
2. 실패 감지:
   - **즉시 surface error**: auth fail, rate_limit, request_size, transport → fallback 안 하고 user 에게 surface
   - **retry-able error**: overloaded, timeout, transient → 1회 fallback retry
3. Fallback chain 적용 — `~/.myharness/state/active-providers.yaml` 의 `fallback_order` 순서 (CONCEPT.md §5.5.3)
4. Domain mapping 적용 (CONCEPT.md §5.5.3):
   - `code` → primary (성능 우선)
   - `server` → discovered-cheapest (cost 우선)
   - `env` → discovered-local-or-cheapest (local Ollama 우선)
5. 모든 fallback exhausted → `error: no_available_provider` (UC-LOOP-001 의 경우 즉시 종료)
6. Event log: `{event: "provider_fallback", primary, fallback_used, final_status, error_code}`

**관련 use case**: UC-CODE-001, UC-SERVER-001, UC-ENV-001, UC-AUTH-001, UC-LOOP-001 (모두)

**§10 acceptance**: UC-EXC-001-ACC-01 ~ ACC-04 (§10.6)

### 7.2 Context overflow (D-30, Layer 1)

**Trigger**: token 사용량이 model length 한계 80% 도달
**출처**: CONCEPT.md §5.6 의 Layer 1 (필수, always-on, opt-out 불가)

**flow**:
1. 매 message 마다 token budget 추적
2. 한계 80% 도달 시 auto trigger:
   - **truncate**: 최근 N=5 message 만 keep, 나머지 제거
   - **summarize**: 오래된 message 를 LLM 으로 요약
   - **hybrid**: 둘 다 (truncate 먼저, 그 다음 summarize)
3. user-callable: `/compact` (UC-CTX-002) — manual 압축
4. Layer 2 (선택) 가 enable 인 경우 추가 headroom 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) 적용 (UC-CTX-003)

**관련 use case**: UC-CODE-001 (PR review, 큰 diff), UC-LOOP-001 (iteration 누적)

**§10 acceptance**: UC-EXC-002-ACC-01 ~ ACC-03 (§10.6)

### 7.3 Permission deny

**Trigger**: sub-agent 가 권한 scope 외 tool 호출 시도
**출처**: CONCEPT.md §5.4 의 4 permission mode

**flow**:
1. sub-agent 권한 scope 검사 (CONCEPT.md §5.11 + 본 문서 §5.4)
2. 권한 scope 외 → 거부 + event log
3. 4 permission mode 별 동작:
   - `default`: user 에게 prompt → yklee 가 allow / deny
   - `acceptEdits`: edit 계열은 자동 allow
   - `plan`: plan 만 표시, 실행 시 user 승인
   - `bypassPermissions`: sandbox 환경, 자동 allow
4. deny 시:
   - sub-agent 에 error 반환
   - sub-agent 가 재시도 / 우회 시도 시 orchestrator 가 중단 결정
5. Event log: `{event: "permission_deny", sub_agent, tool, mode, decision}`

**관련 use case**: UC-ENV-001 (`sudo` 필요 시), UC-SERVER-003 (deploy), UC-SERVER-006 (restart)

**§10 acceptance**: UC-EXC-003-ACC-01 ~ ACC-04 (§10.6)

### 7.4 Hook block

**Trigger**: user hook (markdown rule) 가 매칭되어 block 결정
**출처**: CONCEPT.md §5.4 의 Hook system, §5.4 "markdown 1 file = 1 hook, restart-free 적용"

**flow**:
1. tool 호출 전 hook engine 이 `~/.myharness/hooks/*.md` 활성 rule 매칭
2. 매칭 rule 의 action:
   - `warn`: 경고 표시 + 실행 계속
   - `block`: 실행 거부 + user confirm
   - `transform`: tool input 수정 후 실행
3. 기본 hook (CONCEPT.md §5.4):
   - `warn-rm-rf.md`: `rm -rf` 감지 시 경고
   - `require-test-before-commit.md`: commit 전 test 실행 강제
   - `security-pattern.md`: 9 security pattern detection
4. block 시:
   - sub-agent 에 error 반환
   - yklee 가 `--bypass-hook <name>` flag 로 일시 우회 가능 (4 permission mode 의 `bypassPermissions` 와 별개)
5. Event log: `{event: "hook_block", hook_name, action, tool}`

**관련 use case**: UC-CODE-002 (implement, rm -rf 방지), UC-CODE-004 (commit, test 강제)

**§10 acceptance**: UC-EXC-004-ACC-01 ~ ACC-04 (§10.6)

### 7.5 Tool error

**Trigger**: tool (Bash / MCP / file I/O) 실행 실패
**출처**: 일반 runtime error

**flow**:
1. tool 실행 → 비-zero exit / exception / timeout
2. error 분류:
   - **transient** (네트워크 glitch, timeout) → 1회 retry
   - **permanent** (file not found, permission denied, syntax error) → 즉시 surface
3. transient retry 후에도 실패 → user 에게 error surface + 권장 action 제시
4. Event log: `{event: "tool_error", tool, error_type, retry_count, suggestion}`

**관련 use case**: 모든 use case (tool 사용)

**§10 acceptance**: UC-EXC-005-ACC-01 ~ ACC-03 (§10.6)

---

## 8. Out-of-scope 매핑

**6 out-of-scope** (CONCEPT.md §4.2) → 의도적 **누락 use case** 매핑. v1 implementation 에서 **absolute ❌** (CONCEPT.md §8 의 안티 6 + §4.2 정합).

| OOS # | CONCEPT.md §4.2 | 의도적 누락 use case | v1 absolute ❌ 이유 | v2+/v3+ 시점 |
| --- | --- | --- | --- | --- |
| **OOS-1** | 5 surfaces cross-session (claude-code 13.2) | UC-SURFACE-001 (TUI ↔ IDE hand-off), UC-SURFACE-002 (Web hand-off), UC-SURFACE-003 (5 surface 동시 state) | CONCEPT.md §8 안티 4 ("5 surface 동시 유지") | v2+ (TASK-005-3) |
| **OOS-2** | Plugin marketplace community (claude-code 13.3) | UC-MKT-001 (plugin publish), UC-MKT-002 (plugin install from marketplace) | CONCEPT.md §0 single user + marketplace 는 community scale | v2+ |
| **OOS-3** | Computer Use (claude-code 13.23) | UC-COMPUTER-001 (screenshot + click), UC-COMPUTER-002 (form input) | v1 의 desktop 자동화는 shell + filesystem 으로 충분 | v3+ (TASK-005-5) |
| **OOS-4** | Routines / scheduled tasks (claude-code 13.17) | UC-ROUTINE-001 (cron schedule), UC-ROUTINE-002 (event trigger) | v1 = user-callable only, schedule 은 OS cron 위임 | v2+ (TASK-005-3) |
| **OOS-5** | Channels (Slack/Telegram webhook) | UC-CHANNEL-001 (Slack incoming), UC-CHANNEL-002 (Telegram bot) | v1 = terminal only, channel 은 입구 다변화 | v2+ (TASK-005-3) |
| **OOS-6** | Multi-user / RBAC | UC-RBAC-001 (user 권한), UC-RBAC-002 (audit log per user) | v1 = yklee single user (CONCEPT.md §2), RBAC 무의미 | v3+ (TASK-005-5) |

**추가 OOS (v1.5+ 연기, v1 자체 OOS 는 아님)**:

| 항목 | v1 OOS? | v1.5+ 시점 | 비고 |
| --- | --- | --- | --- |
| Plugin 4-계층 (`commands/agents/skills`) | 🟡 부분 (hook 만 v1) | v1.5+ (TASK-005-2) | CONCEPT.md §5.7 |
| Auto memory cross-device sync | ✅ v1 OOS (local-only) | v2+ opt-in cloud with encryption (CONCEPT.md §8 안티 5) | CONCEPT.md §3.2 |
| LLM Wiki (Karpathy pattern, D-32) | ✅ v1 OOS (flat memory) | v2.5+ (TASK-005-4) | CONCEPT.md §5.13 |
| CCR + Kompress-base (D-37 v1.5+ 연기) | ✅ v1 OOS (3 알고리즘만) | v1.5+ (TASK-005-2) | CONCEPT.md §11.1 TASK-007 |
| Dynamic provider discover (D-38) | 🟡 부분 (Phase 1 hardcoded, Phase 2 dynamic) | v1.5+ full dynamic (CONCEPT.md §11.3 TASK-008) | |
| OAuth flow (Anthropic OAuth, Google OAuth) | ✅ v1 OOS (API key 만) | v2.0+ (TASK-005-3) | CONCEPT.md §11.3 TASK-008 |
| MCP-based provider 등록 | ✅ v1 OOS (정적 6 provider) | v2.0+ (TASK-005-3) | |

> **v1 implementation 시 절대 구현 ❌**: OOS-1 ~ OOS-6 의 6개. §11 결정 보류 (TASK-002 ⏸) 와 함께, 본 문서의 use case catalog 가 v1 scope boundary 의 SSOT.

---

## 9. Cross-platform 분기

**3 platform** (CONCEPT.md §4.1 의 "3-언어 동시"): **macOS (Intel + Apple Silicon Universal) / Linux (Debian/Fedora/RHEL/Alpine) / Windows (PowerShell/CMD, x64/ARM64)**.

**출처**: CONCEPT.md §5.3 (5 install paths) + §5.4 (Secret management, OS keychain) + §5.12 (`~/.myharness/` cross-platform) + D-31 + D-36.

### 9.1 Platform 별 분기 표

| 영역 | macOS | Linux (Debian/RHEL/Alpine) | Windows |
| --- | --- | --- | --- |
| **Install** | `brew --cask myharness` (CONCEPT.md §5.3 ③) | `apt / dnf / apk install myharness` (대안) | `winget install Yklee.Myharness` (⑤) |
| **Install 대안** | `curl -fsSL ... \| bash` (①) | `curl -fsSL ... \| bash` (①) | `irm ... \| iex` (②) |
| **Binary** | Universal (Intel + Apple Silicon) | x86_64 + ARM64 | x64 + ARM64 |
| **Auto-update** | native install 시 background | native install 시 background | native install 시 background |
| **Shell** | zsh (default) + bash | bash (default) + zsh | PowerShell 5.1+ + CMD |
| **Package manager** | Homebrew | apt / dnf / apk | winget / choco |
| **Keychain** | Keychain (Apple Security.framework) | Secret Service (libsecret) | Credential Manager (wincred) |
| **`~/.myharness/` 위치** | `$HOME/.myharness/` | `$HOME/.myharness/` | `%USERPROFILE%\.myharness\` |
| **Path wrapper** | `directories` crate (Rust) | `directories` crate | `directories` crate |
| **Process query (UC-SERVER-001)** | `launchctl list` + `ps aux` | `systemctl list-units --type=service` + `ps aux` | `Get-Service` (PowerShell) |
| **Log query (UC-SERVER-002)** | `log show --last <N>m` (unified log) | `journalctl -u <service> -n <N>` | `Get-EventLog` / `Get-WinEvent` |
| **Service mgmt (UC-SERVER-006)** | `brew services` / `launchctl` | `systemctl restart <svc>` | `Restart-Service <svc>` (PS) |
| **Env setup (UC-ENV-001)** | `brew bundle` + `softwareupdate` | `apt-get install` / `dnf install` / `apk add` | `winget install` / `choco install` |

### 9.2 Cross-platform 영향 use case

- **UC-INSTALL-001~006**: platform 별 install script 분기 (5 install paths)
- **UC-SERVER-001/002/006**: process/log/service query 명령 분기
- **UC-ENV-001/002**: package manager 분기
- **UC-CFG-009 (secret set)**: keychain API 분기 (CONCEPT.md §5.4)
- **UC-MAINT-001 (log tail)**: log.jsonl 자체는 platform 무관, 그러나 log rotate 정책만 OS 별

### 9.3 구현 시 보장 사항 (acceptance)

- 단일 binary (`cargo-dist`, CONCEPT.md §11.3 D-36) 가 3 platform 동시 빌드
- `directories` crate (Rust 1안) 가 cross-platform path resolve
- `keyring` crate (CONCEPT.md §5.4 + §11.3 D-36) 가 3 platform keychain 통합
- platform 별 분기는 `cfg(target_os = "macos" | "linux" | "windows")` Rust attribute 로 명시
- platform 별 test suite (matrix CI: 3 OS × {x86_64, ARM64})

---

## 10. Acceptance criteria per use case

**각 use case 의 완료 조건 (테스트 가능)**. §3 detailed 5개 + §7 exception 5개 + catalog index use case 61개 = **합 ~70+ acceptance 항목**.

### 10.1 UC-CODE-001 (PR review)

- **UC-CODE-001-ACC-01**: `myharness code review https://github.com/owner/repo/pull/123` 실행 시 30초 이내에 3-aspect review 결과 (bugs / style / tests) 가 stdout 에 markdown 으로 출력된다
- **UC-CODE-001-ACC-02**: review 결과에 severity 분류 (blocker / major / minor / nit) 가 포함된다
- **UC-CODE-001-ACC-03**: review 결과에 권장 action (request changes / approve / comment) 이 포함된다
- **UC-CODE-001-ACC-04**: `~/.myharness/handoff/<timestamp>_code_review_<pr>.md` 가 자동 생성된다
- **UC-CODE-001-ACC-05**: `~/.myharness/log.jsonl` 에 `event: "code_review"` 항목이 append 된다
- **UC-CODE-001-ACC-06**: primary provider 실패 시 fallback chain (anthropic → openai → ...) 으로 자동 전환되며, `fallback_used: true` 가 log 에 기록된다
- **UC-CODE-001-ACC-07**: large PR (e.g., 100+ changed files) 에서 context overflow 발생 시 Layer 1 (D-30) auto 압축이 동작한다
- **UC-CODE-001-ACC-08**: `security-pattern.md` hook 활성 시 review 결과에 secret 의심 string 이 감지되면 warn 한다

### 10.2 UC-SERVER-001 (status)

- **UC-SERVER-001-ACC-01**: `myharness server status` (local) 실행 시 모든 running service 목록이 표 형식으로 출력된다
- **UC-SERVER-001-ACC-02**: 각 service 별 PID / STATUS / UPTIME 컬럼이 포함된다
- **UC-SERVER-001-ACC-03**: anomaly (high CPU / zombie / unhealthy) 가 있으면 한국어 요약에 강조 표시된다
- **UC-SERVER-001-ACC-04**: 3 platform (macOS / Linux / Windows) 모두에서 platform 별 명령 (`launchctl` / `systemctl` / `Get-Service`) 이 자동 분기된다
- **UC-SERVER-001-ACC-05**: `--host <alias>` (TASK-002 ⏸) 사용 시 config 의 SSH 별칭으로 원격 host 에 SSH 접속 후 동일 결과 반환
- **UC-SERVER-001-ACC-06**: `log.jsonl` 에 `event: "server_status"` 항목이 append 된다

### 10.3 UC-ENV-001 (setup)

- **UC-ENV-001-ACC-01**: `myharness env setup <stack>` 실행 시 pre-diagnose 가 먼저 수행된다
- **UC-ENV-001-ACC-02**: stack manifest (TASK-002 ⏸) 또는 built-in default 로 package 설치가 진행된다
- **UC-ENV-001-ACC-03**: post-diagnose + smoke test (설치된 tool 의 `--version`) 가 자동 실행된다
- **UC-ENV-001-ACC-04**: 3 platform 모두에서 platform 별 package manager (brew / apt / dnf / apk / winget / choco) 가 자동 분기된다
- **UC-ENV-001-ACC-05**: idempotency — 동일 stack 재실행 시 추가 설치 없이 "이미 설치됨" 보고된다
- **UC-ENV-001-ACC-06**: `apt-get` 등 권한 부족 시 sudo prompt 가 뜨고, 4 permission mode 의 `bypassPermissions` 시 자동 처리된다
- **UC-ENV-001-ACC-07**: `handoff/<timestamp>_env_setup_<stack>.md` 와 `log.jsonl` 모두 갱신된다

### 10.4 UC-AUTH-001 (provider discover+login)

- **UC-AUTH-001-ACC-01**: `myharness auth setup` 실행 시 6 provider (anthropic / openai / gemini / deepseek / minimax / ollama) 모두 scan 된다
- **UC-AUTH-001-ACC-02**: env var / OS keychain / local LLM server 3가지 source 모두 검사된다
- **UC-AUTH-001-ACC-03**: `~/.myharness/state/auth/<provider>.yaml` 6개 파일이 생성/갱신된다
- **UC-AUTH-001-ACC-04**: `~/.myharness/state/active-providers.yaml` 가 자동 갱신된다
- **UC-AUTH-001-ACC-05**: token 값은 어디에도 ❌ (display 시 `***` mask)
- **UC-AUTH-001-ACC-06**: login wizard 가 interactive 로 yklee 에게 미인증 provider 별 login method 를 제시한다
- **UC-AUTH-001-ACC-07**: `auth <provider> test` 실행 시 ping model + latency 측정이 성공한다
- **UC-AUTH-001-ACC-08**: `fallback_order` 가 config 우선순위 + active filter 로 자동 구성된다
- **UC-AUTH-001-ACC-09**: `log.jsonl` 에 `event: "auth_setup"` 항목이 append 된다

### 10.4b UC-AUTH-010 (add-local, D-59 W16)

- **UC-AUTH-010-ACC-01**: `myharness auth add-local` 실행 시 interactive wizard 가 시작되어 (1) URL (2) 선택적 token (3) 모델 선택 3 단계를 차례로 제시한다
- **UC-AUTH-010-ACC-02**: URL 입력은 `url::Url::parse` 검증 후에만 다음 단계로 진행되며, 잘못된 URL 입력 시 재입력 prompt 가 뜬다
- **UC-AUTH-010-ACC-03**: `GET {base_url}/models` probe 가 3s timeout 내에 성공하면 JSON `data[*].id` 가 arrow-key 선택 UI 로 표시된다
- **UC-AUTH-010-ACC-04**: 모델 선택 확정 후 `~/.myharness/providers.toml` 의 `LocalLlm` entry 의 `base_url` + `default_model` + `available_models` 가 갱신되며, atomic write (tmp + rename) 로 손상 방지된다
- **UC-AUTH-010-ACC-05**: token 비어있음 → keychain set 생략, `requires_key: false` 인 LocalLlm 의 정의대로 진행. token 있음 → `KeyringAuthStore::set(LocalLlm, &token)` 호출 + in-memory cache + env hint 메시지 (Linux None backend 시)
- **UC-AUTH-010-ACC-06**: connection refused / 4xx 5xx / 모델 0개 / inquire cancel (ESC) 중 어느 하나라도 발생 시 graceful abort + exit code 1 + 한국어 에러 메시지
- **UC-AUTH-010-ACC-07**: 비-tty 환경에서 실행 시 (stdin 또는 stdout 이 tty 아님) "interactive only" 에러 + exit 1

### 10.5 UC-LOOP-001 (loop mode)

- **UC-LOOP-001-ACC-01**: `--mode=loop --goal "<text>"` 실행 시 iteration loop 가 시작된다
- **UC-LOOP-001-ACC-02**: `--success-criteria "<text>"` 가 있으면 LLM judge 가 매 iteration 마다 success 평가한다
- **UC-LOOP-001-ACC-03**: `--max-iterations` 초과 시 자동 종료 + partial 보고된다
- **UC-LOOP-001-ACC-04**: yklee 가 Ctrl+C 입력 시 즉시 종료 + interrupted 보고된다
- **UC-LOOP-001-ACC-05**: 각 iteration 마다 한국어 요약 + handoff 가 갱신된다
- **UC-LOOP-001-ACC-06**: success 시 조기 종료되며, `state/loop-<goal-slug>.yaml` 에 최종 iteration log 가 저장된다

### 10.6 Exception flows (UC-EXC-001 ~ UC-EXC-005)

- **UC-EXC-001-ACC-01** (provider fallback, §7.1): primary 호출 실패 시 1회 retry 후 fallback chain 으로 전환된다
- **UC-EXC-001-ACC-02**: 모든 fallback exhausted 시 `error: no_available_provider` 가 surface 된다
- **UC-EXC-001-ACC-03**: domain mapping (code → primary / server → cheapest / env → local) 이 자동 적용된다
- **UC-EXC-001-ACC-04**: `log.jsonl` 에 `event: "provider_fallback"` 가 기록된다
- **UC-EXC-002-ACC-01** (context overflow, §7.2): token 사용량 80% 도달 시 auto 압축 (truncate / summarize / hybrid) 이 발동된다
- **UC-EXC-002-ACC-02**: `/compact` (UC-CTX-002) manual 호출 시 즉시 압축된다
- **UC-EXC-002-ACC-03**: Layer 2 enable 시 3 알고리즘 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor) 이 추가 적용된다
- **UC-EXC-003-ACC-01** (permission deny, §7.3): sub-agent 권한 scope 외 tool 호출 거부된다
- **UC-EXC-003-ACC-02**: 4 permission mode (`default` / `acceptEdits` / `plan` / `bypassPermissions`) 별 동작 차이가 정확히 반영된다
- **UC-EXC-003-ACC-03**: deny 시 sub-agent 에 error 반환되며 우회 시도 시 orchestrator 가 중단한다
- **UC-EXC-003-ACC-04**: `log.jsonl` 에 `event: "permission_deny"` 가 기록된다
- **UC-EXC-004-ACC-01** (hook block, §7.4): `~/.myharness/hooks/*.md` 활성 rule 이 tool 호출 전 매칭된다
- **UC-EXC-004-ACC-02**: `warn-rm-rf.md` hook 이 `rm -rf` 패턴 감지 시 경고한다
- **UC-EXC-004-ACC-03**: `require-test-before-commit.md` hook 이 commit 전 test 실행을 강제한다
- **UC-EXC-004-ACC-04**: `--bypass-hook <name>` flag 로 일시 우회 가능하다
- **UC-EXC-005-ACC-01** (tool error, §7.5): transient error (timeout / network glitch) 시 1회 retry 후 surface 된다
- **UC-EXC-005-ACC-02**: permanent error (file not found / permission denied) 는 즉시 surface 된다
- **UC-EXC-005-ACC-03**: `log.jsonl` 에 `event: "tool_error"` + error_type + suggestion 이 기록된다

### 10.7 Catalog index use case acceptance (요약)

catalog index 의 61개 use case 도 각 2~3 acceptance 로 정밀화 가능 (본 문서의 분량 한계로 **요약 표** 만 제공).

| Prefix | acceptance count (대략) | 검증 방식 |
| --- | --- | --- |
| UC-CODE-* (9 catalog) | 2~3 / UC = ~25 | unit + integration (cargo test) |
| UC-SERVER-* (7 catalog) | 2~3 / UC = ~18 (TASK-002 ⏸ 일부 placeholder) | integration (CI matrix 3 OS) |
| UC-ENV-* (7 catalog) | 2~3 / UC = ~18 (TASK-002 ⏸ 일부 placeholder) | integration (CI matrix 3 OS) |
| UC-AUTH-* (8 catalog) | 2~3 / UC = ~20 | unit (mock provider) + manual (실제 provider) |
| UC-INSTALL-* (6) | 2~3 / UC = ~15 | install script dry-run + real install in sandbox VM |
| UC-CFG-* (10) | 2~3 / UC = ~25 | unit (config parser) + integration (실제 config edit) |
| UC-MAINT-* (8) | 2~3 / UC = ~20 | unit (jsonl parse / yaml read) + integration |
| UC-LOOP-* (2 catalog) | 2~3 / UC = ~5 | integration (loop 종료 조건별) |
| UC-SINGLE-* (1) | 2~3 | unit (mode flag parse) |
| UC-CTX-* (3) | 2~3 / UC = ~8 | unit (token budget mock) + integration (실제 LLM) |
| **합계** | **~160 acceptance 항목** | (TASK-005-1 v1 Rust MVP 의 test plan 입력) |

> **TASK-005-1 구현 시**: 본 §10 의 acceptance 가 test plan 의 SSOT. v1 MVP 의 ~160 acceptance 항목이 integration test 의 source.

---

## 부록 A. 본 문서의 결정 보류 + 안티 패턴 미반영 (요약)

### A.1 §11 결정 보류 (CONCEPT.md §11.1)

- ✅ TASK-005 (Rust 1안) → 본 문서 모든 use case Rust 1안 가정
- ✅ TASK-006 (ratatui + crossterm) → UC-TUI surface 한정
- ✅ TASK-007 (headroom 3 알고리즘 v1 우선) → UC-CTX-* 가 3 알고리즘만
- ✅ TASK-008 (D-38 provider-auto-config) → UC-AUTH-001 detailed + §7.1 exception
- ⏸ **TASK-002 (도메인별 명령 가이드, server/env)** → **UC-SERVER-007, UC-ENV-006/007 의 host alias / runtime / dotfiles 경로는 placeholder** (yklee 인프라 정보 수령 후 채움, 발명 ❌)

### A.2 §8 안티 6 미반영 (CONCEPT.md §8)

| # | 안티 | 본 문서 | 증거 |
| --- | --- | --- | --- |
| 1 | closed source | ✅ open | 라이선스 중립 (본 문서) |
| 2 | 듀얼 언어 | ✅ 단일 (Rust 1안) | §0.3 + 모든 use case |
| 3 | 100+ slash commands | ✅ 12 CLI + ~66 use case | §2 catalog 합계 |
| 4 | 5 surface 동시 | ✅ CLI + TUI only | §4 + §8 OOS-1 |
| 5 | cloud auto memory default | ✅ v1 local-only | §8 + UC-MAINT-007 |
| 6 | subscription requirement | ✅ free CLI | UC-AUTH-* 모두 free |

### A.3 cross-ref 무결성

본 문서의 모든 claim 은 CONCEPT.md / PROJECT_PROFILE.md / REQUIREMENTS.md 의 §X.Y 와 매핑. **broken link 0** (cross-ref 표 자동 생성 — TASK-005-1 v1 Rust MVP 의 `doc-test` 입력 가능).

### A.4 분량

- **목표**: 700~1,100줄
- **현재**: §10 까지 작성 완료 (chunked write 6-chunk 합)

---

## 부록 B. handoff 형식 (D-26)

> **summary**: 본 문서는 my_harness v1 의 3-도메인 (코드/서버/환경) + cross-cutting (auth / install / cfg / maint / loop / single / ctx) 의 use case 66개를 7 prefix 로 catalog 화. 5개 (UC-CODE-001, UC-SERVER-001, UC-ENV-001, UC-AUTH-001, UC-LOOP-001) 는 detailed, 3 mode (orchestrator / single / loop) 매트릭스 (§4) + 15 sub-agent dispatch 매트릭스 (§5) + 3 extension point (§6) + 5 exception flow (§7) + 6 OOS 매핑 (§8) + 3 platform 분기 (§9) + ~160 acceptance (§10) 으로 정밀화. Rust 1안 (D-36) + ratatui (D-36) + headroom 3 algo (D-37) + D-38 provider-auto-config 결정 모두 반영. **TASK-002 ⏸** (UC-SERVER-007, UC-ENV-006/007 의 yklee 인프라 정보) 는 placeholder 유지, 발명 ❌.
>
> **risks**:
> - §10 acceptance 가 5 detailed + 5 exception + 61 catalog = ~160 항목. v1 Rust MVP 의 test plan 으로 변환 시 분량 폭증. **권장**: TASK-005-1 의 `tests/integration/` 디렉토리에 acceptance → test 함수 1:1 매핑 자동화 (별도 tool 또는 본 문서 §10 → cargo test 자동 생성)
> - cross-platform 3 OS × {x86_64, ARM64} = 6 matrix CI 러닝타임. **권장**: cargo-dist 의 GitHub Actions matrix 로 자동화
> - MCP server 4 pre-config 의 일부 (e.g., `mcp__github__*`) 는 yklee 의 `GITHUB_TOKEN` / `gh auth` 의존. **권장**: UC-INSTALL-* 의 installer 가 GitHub token 미설정 시 graceful skip
>
> **suggested_follow_up**:
> - **TASK-005-1**: 본 문서 §10 acceptance → `tests/integration/use_cases/*.rs` test 자동 생성
> - **TASK-005-2 (v1.5)**: §6.3 plugin 4-계층 (commands/agents/skills) 구현 + UC-PLUGIN-* use case 정식 추가
> - **TASK-005-3 (v2.0)**: §8 OOS-1~5 의 5 surfaces / routines / channels 구현 + UC-SURFACE-* / UC-ROUTINE-* / UC-CHANNEL-* use case 추가
> - **TASK-005-4 (v2.5)**: §8 의 LLM Wiki (D-32) + UC-WIKI-* use case 추가
> - **cross-doc sync**: REQUIREMENTS.md (WP1) 와 본 문서 (WP2) 의 FR ↔ UC 매핑 검증 (양방향)
> - **REQUIREMENTS.md 가 cycle 1 종료 후 사용 가능** — 본 문서 작성 시점 (cycle 1 in_progress) 에서는 입력 미사용, 후속 verifier 가 양방향 cross-check 시 활용
>
> **produced_artifacts**:
> - `/Users/yklee/repos/my_harness/docs/USE_CASES.md` (메인, 본 문서)
> - `/Users/yklee/repos/my_harness/docs/team/deliverable_use_cases.md` (early signal + final status)
> - `/Users/yklee/.mavis/plans/plan_c26d3adf/outputs/use-cases/deliverable.md` (plan deliverable, 별도 작성)

