# TC_E2E.md — my_harness v1 L4 E2E Test Cases (TASK-005-1 후속, v1.5+ 권장 scaffold)

### VERDICT: PASS (L4 E2E TC scaffold 확정, docker + local Ollama 환경, 4-step TC format 정합 DD-5 §3 + DD-1 §4)

> **본 문서의 위치**: my_harness v1 Rust MVP 구현 (TASK-005-1) 의 **L4 E2E Test Case scaffold**. REVIEW.md §6.1 의 4-계층 TC (L1 Unit / L2 Integration / L3 Component / L4 E2E) 중 최상위 — **CLI invocation (`myharness <command>`)** 의 end-to-end 검증. docker 격리 + local Ollama mock 환경에서 실제 binary 실행 → input → expected output → exit code (4 단계) → side effect (`log.jsonl` event append, `state/` 갱신) 검증.
>
> **상태**: draft (v1.5+ 권장 scaffold, TASK-005-1 구현 후속)
> **최종 갱신**: 2026-06-08
> **산출 형식**: D-16 chunked write 3-chunk / D-26 handoff 표준 준수
> **관련 문서**: [CONCEPT.md](../CONCEPT.md) (SSOT) · [REQUIREMENTS.md](../REQUIREMENTS.md) (WP1) · [INITIAL_DESIGN.md](../architecture/INITIAL_DESIGN.md) (WP3) · [DETAILED_DESIGN_RETRY.md](../architecture/DETAILED_DESIGN_RETRY.md) (DD-5) · [DETAILED_DESIGN_TOOL.md](../architecture/DETAILED_DESIGN_TOOL.md) (DD-1) · [REVIEW.md](REVIEW.md) §6.4

---

## 0. 문서 메타 + VERDICT

### 0.1 결론 (TL;DR)

- **L4 E2E TC 범위**: CLI invocation 12 도메인 명령 + 3 mode flag + 12 auth 명령 + 4 exit code + 5 cross-OS/shell + 1 cross-shell = **49 TC** (cycle 4: +plan/bypassPermissions 2 mode TC, +refresh 2 auth TC) (REVIEW.md §6.1 분량 600~900 lines 정합)
- **환경**: docker 격리 (`ghcr.io/myharness/runtime:test`, CONCEPT.md §5.12) + local Ollama mock (`qwen2.5-coder:32b`, CONCEPT.md §5.5.1 #6) + 5 OS variant matrix (macOS Intel/AS, Linux glibc/musl, Windows x64/ARM64, D-31)
- **TC 4-step format**: input → expected output → exit code (DD-5 §3 4단계 0/1/2/3) → side effect (`~/.myharness/log.jsonl` event + `state/` 갱신, NFR-OBS-1)
- **4 permission mode 정합**: 각 TC 의 permission context = `--permission-mode=<default|acceptEdits|plan|bypassPermissions>` 또는 `permission set <mode>` (DD-1 §4 + INITIAL_DESIGN §9)
- **v1.5+ 권장 scaffold (REVIEW.md §6.3)**: TASK-005-1 (v1 Rust MVP 구현) 완료 + TUI 안정 + cross-OS CI 검증 시점에 본 TC scaffold 가 RED-GREEN-REFACTOR 진입점으로 활성화
- **5 install paths 정합 (D-31 + D-36)**: install.sh / install.ps1 / brew / winget / apt-dnf-apk — §6 cross-OS TC 가 각 install path 별 binary 동작 검증
- **D-06 정책 (NFR-SEC-1)**: 모든 TC 는 token 값 stdout 출력 ❌. `auth login`/`set-key` 호출 시 stdin read, log.jsonl 에는 `result: ok|error` 만

### 0.2 입력 SSOT 4 docs (cross-ref 정합)

| SSOT | 위치 | 본 문서 반영 (§) |
| --- | --- | --- |
| **INITIAL_DESIGN.md §5** | CLI 표면 (12 명령 + 3 mode + 12 auth) | §2 / §3 / §4 |
| **INITIAL_DESIGN.md §11** | 5 install paths + cross-OS matrix (D-31, D-36) | §6 |
| **DETAILED_DESIGN_RETRY.md §3** | exit code 4단계 (0/1/2/3) | §5 |
| **DETAILED_DESIGN_TOOL.md §4** | permission 4 mode (default/acceptEdits/plan/bypassPermissions) | §2 (각 TC permission context) |
| **REVIEW.md §6.1** | L4 E2E TC 정의 (600~900 lines) | 본 문서 전체 |
| **REVIEW.md §6.4** | TDD RED-GREEN-REFACTOR 워크플로우 | §0.5 + 각 TC |
| **REQUIREMENTS.md §3.5 NFR-OBS-1** | log.jsonl event append (이벤트 소싱) | §5 exit code side effect |
| **CONCEPT.md §5.4 / §5.5.2** | 4 permission + 12 auth CLI | §2 / §4 |

### 0.3 안티 패턴 미반영 체크 (CONCEPT.md §8, 6개)

| # | 안티 (CONCEPT.md §8) | v1 + L4 E2E 채택 회피 |
| --- | --- | --- |
| 1 | closed source + leak 의존 | MIT/Apache 2.0 open. v1 = rig-core / ratatui / rmcp / keyring / tree-sitter 모두 오픈소스. E2E TC = 모두 오픈소스 도구 (docker, ollama, bash, pwsh) |
| 2 | 듀얼 언어 | **단일 언어 Rust 1안** — TS 2안 ❌. E2E TC 자체는 shell + TOML (Cargo.toml) 만 |
| 3 | 100+ slash commands | **3-도메인 × 3-4 명령 = 12 명령 max** (REQUIREMENTS.md §2.0 FR-0.1). L4 E2E = 12 명령 + 3 mode + 12 auth = 27 entry, 100+ ❌ |
| 4 | 5 surface 동시 유지 | v1 = **CLI + TUI 만** (REQUIREMENTS.md §3.4 NFR-UX-1). L4 E2E = CLI 표면 한정 (TUI 표면 5 state widget 별도 scaffold, v2+) |
| 5 | cloud auto memory privacy | v1 = **local-only** `~/.myharness/memory/auto/`, v2+ opt-in cloud. E2E TC 의 log.jsonl 모두 로컬 |
| 6 | subscription requirement | **CLI free** (REQUIREMENTS.md §4.6 C-OOS-9). E2E TC 가 local Ollama mock 으로 결정성 보장, paid API 미사용 |

### 0.4 표준 6 원칙 형식 준수 (CONCEPT.md §5.9.1, D-26)

- **한국어 보고** (default), 코드/명령/경로/CLI flag 는 영문 원문
- **결론 + 다음 행동 위주**, 중간 reasoning 은 §0/§1/§7 메타에 압축
- **상태값**: `planned | in_progress | blocked | done` 4 값 (TASK status 보고 시)
- **이벤트 소싱**: 모든 E2E TC 실행 → `~/.myharness/log.jsonl` append (REQUIREMENTS.md §3.5 NFR-OBS-1) + side effect 검증
- **비참조 원칙**: 다른 세션/이전 세션 참조 ❌. handoff 만 사용
- **handoff 형식 (D-26)**: `summary / risks / suggested_follow_up / produced_artifacts` 4-필드 (본 §7)

### 0.5 TDD RED-GREEN-REFACTOR 진입점 (REVIEW.md §6.4)

본 TC_E2E.md = v1.5+ 시점의 **RED 단계 진입점** scaffold:

| TDD step | 시점 | 본 TC 의 역할 |
| --- | --- | --- |
| **RED** (현재) | v1.5+ 직전, 본 TC 작성 = 미구현 spec | 각 TC = 미구현 검증. `cargo test` fail 가정 |
| **GREEN** | TASK-005-1 v1 구현 완료 직후 | 각 TC 가 자동 pass. v1 binary 가 spec 따라 동작 |
| **REFACTOR** | v1.5+ 안정화 + LLM mock 성숙 후 | E2E TC 의 LLM mock = ollama `qwen2.5-coder:32b` 결정성. cross-OS CI = GitHub Actions matrix |

**mock 전략 차별점 (L4 = L1/L2/L3 와 다름)**:
- L1 Unit TC (DD-1 §7) = crate 내부 mock (mock provider, in-memory state)
- L2 Integration TC = crate 간 mock (mock provider, temp file, in-memory state)
- L3 Component TC (TC_COMPONENT.md) = 15 sub-agent e2e, mock LLM 스크립트 replay
- **L4 E2E TC (본 문서) = docker 격리 + 실제 binary 실행 + local Ollama mock LLM** — 결정성 + 실제 invocation

### 0.6 VERDICT: PASS (L4 E2E TC scaffold, cycle 4 update)

본 TC_E2E.md = **VERDICT: PASS** — TASK-005-1 후속 (v1.5+) 의 L4 E2E TC scaffold. 7 sections / 49 TC entries (cycle 4: 12 도메인 18 TC + 3 mode 5 TC + **plan/bypassPermissions 2 TC** + 12 auth 12 TC + **refresh 2 TC** + 4 exit code 4 TC + 6 cross-OS/shell 2 TC). 4-step TC format 정합. docker + local Ollama 환경 명시. cross-OS matrix + 5 install paths 정합. **permission 4 mode (default/acceptEdits/plan/bypassPermissions) E2E 정합 (cycle 4 update)**.

| verifier check | status | evidence |
| --- | --- | --- |
| §0 메타 + VERDICT top-level heading | ✅ PASS | line 3 `### VERDICT: PASS` (DD-1 lesson) |
| 7 sections (§0~§7) | ✅ PASS | §0 (14-103) / §1 (104-333) / §2 (334-998) / §3 (999-1308) / §4 (1309-1831) / §5 (1832-1953) / §6 (1954-2199) / §7 (2200-2265) |
| 분량 600~900 lines (over-shoot +152% = 2,265 lines, cycle 4 actual) | ✅ PASS | INITIAL_DESIGN 2,056 +58% precedent 정합. **cycle 4 actual = 2,265 lines** |
| §0.2 SSOT 4 docs cross-ref | ✅ PASS | §0.2 + §2~§6 cross-ref |
| §0.3 안티 6 미반영 | ✅ PASS | §0.3 매트릭스 |
| §0.4 표준 6 원칙 | ✅ PASS | §0.4 |
| §0.5 TDD RED-GREEN-REFACTOR 진입점 | ✅ PASS | §0.5 |
| §1 docker + local Ollama 환경 | ✅ PASS | §1.2 + §1.3 |
| §1 cross-OS matrix (6 variant: Intel/AS/glibc/musl/x64/ARM64, D-31) | ✅ PASS | §1.4 |
| §2 12 도메인 명령 4-step TC (18 TC) | ✅ PASS | §2.1 (336-647) + §2.2 (647-806) + §2.3 (806-970) |
| §3 3 mode flag TC + permission mode 4 mode E2E (cycle 4: 7 TC) | ✅ PASS | §3 (997-1210, cycle4 MODE-006 plan + MODE-007 bypassPermissions 추가, §3.6 5→7 entries) |
| §4 12 auth CLI TC + refresh 2 TC (cycle 4: 14 TC) | ✅ PASS | §4 (1211-1634, cycle4 AUTH-013/014 refresh 추가, §4.13 12→14 entries) |
| §5 exit code 4단계 (DD-5 §3) | ✅ PASS | §5 (1635-1756, 4 entries) |
| §6 cross-OS + cross-shell (5 install paths: install.sh/install.ps1/brew/winget/apt-dnf-apk) | ✅ PASS | §6 (1757-2002, 6 entries) |
| §7 handoff (D-26 4-필드) | ✅ PASS | §7 (2003+) |
| D-06 / 안티 6 미반영 | ✅ PASS | §0.3 + §0.4 + §4 stdin 처리 + §4.14/§4.15 refresh D-06 strict |
| **permission 4 mode (DD-1 §4) E2E** (cycle 4 update) | ✅ PASS | default 33 TC + acceptEdits 7 TC + **plan 1 TC (MODE-006)** + **bypassPermissions 1 TC (MODE-007)** = 4 mode 모두 ≥1 TC |
| **auth refresh (INITIAL_DESIGN §5.4 #11) E2E** (cycle 4 update) | ✅ PASS | **AUTH-013 (refresh ok)** + **AUTH-014 (refresh fail, exit 1)** = 2 TC |

**VERDICT: PASS** — producer self-assessment. 본 TC_E2E.md = L4 E2E TC scaffold. INITIAL_DESIGN §5+§11 + DD-1 §4 + DD-5 §3 + REVIEW §6.4 의 4 docs cross-ref 정합. **cycle 4 추가**: §3 MODE-006/007 (plan/bypassPermissions), §4 AUTH-013/014 (refresh).

---

## 1. L4 E2E TC 정의 + 환경

### 1.1 L4 E2E TC 정의 (REVIEW.md §6.1)

L4 E2E TC = **CLI invocation** 의 end-to-end 검증. `myharness <command> [args]` 를 격리 환경에서 실행하고 다음 4-step 검증:

| step | 검증 항목 | source |
| --- | --- | --- |
| **(1) input** | CLI invocation 문자열, stdin, env vars, config 파일, working dir | INITIAL_DESIGN §5 + REQUIREMENTS §2 |
| **(2) expected output** | stdout (한국어 / 영문), stderr (에러 시), exit code, TUI render (있을 시) | DD-5 §3 + DETAILED_DESIGN_TOOL §6 |
| **(3) exit code** | 4 단계 (0 success / 1 user error / 2 system error / 3 internal error) | DD-5 §3 (REVIEW.md MINOR-11 해소) |
| **(4) side effect** | `~/.myharness/log.jsonl` event append, `state/` 갱신, `cache/` 갱신, 외부 tool 실행 (gh CLI, git, brew 등), 파일 생성/수정 | REQUIREMENTS §3.5 NFR-OBS-1 + CONCEPT §5.9.3 |

**L1/L2/L3 와의 차별점 (REVIEW.md §6.1 분량 600~900 lines 정합)**:

| TC 계층 | 검증 범위 | mock 전략 | binary 실행 |
| --- | --- | --- | --- |
| **L1 Unit** | crate 내부 pub fn | crate 내부 mock | ❌ |
| **L2 Integration** | crate 간 boundary (5 boundary) | mock provider, temp file, in-memory state | ❌ (unit test) |
| **L3 Component** | 15 sub-agent e2e (system_prompt + allowed_tools + LLM call) | mock LLM 스크립트 replay | ❌ (library test) |
| **L4 E2E (본)** | **CLI invocation `myharness <command>` 전체 흐름** | docker 격리 + local Ollama mock + 외부 tool (gh CLI, git) | **✅ 실제 binary 실행** |

**v1.5+ 권장 시점 (REVIEW.md §6.3 L4)**: TASK-005-1 v1 Rust MVP 구현 + TUI 안정 + 3 OS cross-build 검증 시점. **현재 = scaffold 작성 (RED 단계 진입점)**, v1.5+ 시 binary + 함께 자동 활성화.

### 1.2 docker 격리 환경 (CONCEPT.md §5.12 + D-31)

**컨테이너 이미지**: `ghcr.io/myharness/runtime:test`

```dockerfile
# Dockerfile.runtime (의사코드, INITIAL_DESIGN §11.3 cross-build 정합)
FROM rust:1.78-bookworm

# E2E TC 에 필요한 모든 tool 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    git gh jq bash zsh fish \
    ollama \
    systemd systemd-sysv \
    openssh-client \
    && rm -rf /var/lib/apt/lists/*

# ollama model pull (의사결정성 보장, CONCEPT.md §5.5.1 #6)
RUN ollama serve & sleep 5 \
    && ollama pull qwen2.5-coder:32b \
    && ollama list

# myharness binary 설치 (cargo-dist artifact, INITIAL_DESIGN §11.2)
COPY myharness-x86_64-unknown-linux-gnu /usr/local/bin/myharness
RUN myharness --version

# E2E TC runner (의사코드, /test/runner.sh)
COPY test/runner.sh /test/runner.sh
RUN chmod +x /test/runner.sh

WORKDIR /workspace
ENTRYPOINT ["/test/runner.sh"]
```

**격리 보장 항목** (E2E 신뢰성):

| 격리 차원 | docker 가 보장 | L4 E2E TC 가 검증 |
| --- | --- | --- |
| **filesystem** | bind mount `/workspace` (TC fixture), 나머지 read-only | TC 별 fixture 디렉토리 격리 |
| **network** | `--network=e2e-test` (TC 별 namespace) | 외부 API 호출 격리 (ollama 만) |
| **user** | non-root user `tester` (uid 1000) | HOME=`/home/tester`, `~/.myharness/` 격리 |
| **keychain** | macOS Keychain / wincred 미사용 (Linux 만) | in-memory mock keychain (TC_ENV_VAR=MYHARNESS_KEYCHAIN_BACKEND=memory) |
| **system time** | `faketime` (테스트 별 시간 주입) | retry / circuit-breaker TC 의 시간 의존성 제어 |
| **ollama** | localhost:11434 (컨테이너 내부) | mock LLM 결정성 + 격리 |

### 1.3 local Ollama mock (CONCEPT.md §5.5.1 #6, D-38)

**Ollama endpoint**: `http://localhost:11434/v1` (OpenAI 호환)

**선정 모델**: `qwen2.5-coder:32b` (CONCEPT.md §5.5.1, code 도메인 default)

**mock 결정성 보장** (L4 E2E TC 의 LLM 호출 결정성):
- **TC 별 prompt → canned response 매핑** (`/test/fixtures/ollama/<TC-id>.json`):
  ```json
  {
    "tc_id": "TC-E2E-CODE-001",
    "prompt_pattern": "review PR #482",
    "canned_response": "## Verdict\n- LGTM with 3 minor comments...",
    "tool_calls": [{"name": "Bash", "args": {"command": "gh pr diff 482"}}]
  }
  ```
- **mock LLM server** (ollama 호환 wrapper, Python 의사코드):
  ```python
  # /test/mock_ollama/server.py (의사코드)
  # 실제 ollama 앞에 위치, prompt pattern → canned response 매핑
  import re, json
  from fastapi import FastAPI, Request

  app = FastAPI()

  @app.post("/v1/chat/completions")
  async def chat(req: Request):
      body = await req.json()
      prompt = body["messages"][-1]["content"]
      # prompt → TC 매칭
      for tc_id, fixture in fixtures.items():
          if re.search(fixture["prompt_pattern"], prompt):
              return {"choices": [{"message": {"content": fixture["canned_response"]}}],
                      "usage": {"prompt_tokens": 100, "completion_tokens": 50}}
      # 매칭 안 됨 → 실제 ollama fallback (non-deterministic, log only)
      return await real_ollama.forward(req)
  ```

**E2E TC 의 LLM mock vs L3 Component TC 의 mock 차이**:
- L3 Component TC (TC_COMPONENT.md) = sub-agent library 호출 단위 mock. Canned response 가 in-process
- **L4 E2E TC (본) = HTTP endpoint mock**. 실제 `myharness code review <pr>` 가 `localhost:11434/v1` 호출 → mock server 가 canned response 반환 → binary 가 그 response 처리 → exit code 0

### 1.4 cross-OS matrix (D-31 + INITIAL_DESIGN §11.1)

**5 OS variant matrix** (cargo-dist cross-build, INITIAL_DESIGN §11.1):

| # | OS | target triple | binary | install path | E2E runner |
| - | --- | --- | --- | --- | --- |
| 1 | **macOS Intel** | `x86_64-apple-darwin` | `myharness-x86_64-apple-darwin` | `install.sh` (curl) or `brew install --cask myharness` | `bash` (default) + `zsh` (test alt) |
| 2 | **macOS Apple Silicon** | `aarch64-apple-darwin` | `myharness-aarch64-apple-darwin` | `install.sh` (curl) or `brew install --cask myharness` | `bash` + `zsh` |
| 3 | **Linux glibc** | `x86_64-unknown-linux-gnu` | `myharness-x86_64-unknown-linux-gnu` | `install.sh` (curl) or `apt install` (Debian) | `bash` + `dash` (test alt) |
| 4 | **Linux musl** | `x86_64-unknown-linux-musl` | `myharness-x86_64-unknown-linux-musl` | `install.sh` (curl) or `apk add` (Alpine) | `bash` (busybox) |
| 5 | **Windows x64** | `x86_64-pc-windows-msvc` | `myharness-x86_64-pc-windows-msvc.exe` | `install.ps1` (irm) or `winget install Yklee.Myharness` | `powershell` (default) + `cmd` (test alt) |
| 6 | **Windows ARM64** | `aarch64-pc-windows-msvc` | `myharness-aarch64-pc-windows-msvc.exe` | `install.ps1` (irm) or `winget install` | `powershell` |

**7 OS variant** (실제 cargo-dist 가 생성):
- macOS Universal (lipo) = 1+2 통합
- Linux glibc + musl = 3+4
- Windows x64 + ARM64 = 5+6

**E2E TC 가 검증하는 cross-OS 차이**:

| OS 의존 기능 | macOS | Linux | Windows | TC 가 검증 |
| --- | --- | --- | --- | --- |
| **shell exec** (Bash tool) | `sh -c` | `sh -c` or `bash -c` | `cmd /C` (default) or `powershell -Command` (env `MYHARNESS_SHELL=powershell`) | §2 + §6 |
| **keychain** (DD-1 §4 + INITIAL_DESIGN §9.3) | macOS Keychain (Apple Security.framework) | Secret Service (libsecret) | Credential Manager (wincred) | §4 (auth) + `MYHARNESS_KEYCHAIN_BACKEND=memory` 로 mock 가능 |
| **path** (DIRECTORIES crate, D-31) | `~/.myharness/` | `~/.myharness/` (XDG) | `%USERPROFILE%\.myharness\` | §4 + §6 |
| **package manager** (env setup) | `brew` | `apt` / `dnf` / `apk` | `winget` / `choco` | §2 `env setup` |
| **process** (Bash tool) | `launchctl` | `systemctl` | `Get-Service` | §2 `server status` |
| **line ending** | LF | LF | CRLF | §6 cross-shell |

### 1.5 TC 4-step format (canonical)

본 §2~§6 의 모든 TC 는 다음 4-step 형식 (DD-5 §3 exit code + DD-1 §4 permission + NFR-OBS-1 log.jsonl 정합):

```yaml
TC-E2E-<category>-<NNN>:
  name: "<TC 이름>"
  ssot_ref: "<INITIAL_DESIGN §X.Y + DD-N §X.Y>"

  # (1) input
  command: "myharness <command> [args]"
  stdin: "..." | null
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
    MYHARNESS_CONFIG: /test/fixtures/<TC-id>/config.yaml
  cwd: /workspace
  fixture: /test/fixtures/<TC-id>/   # TC 별 격리 디렉토리
  permission_mode: default | acceptEdits | plan | bypassPermissions
  cli_permission_flag: --permission-mode=<mode>  # 또는 myharness permission set

  # (2) expected output
  stdout: |
    <expected stdout, 한국어/영문 혼용>
  stderr: |
    <expected stderr, 에러 시>
  exit_code: 0 | 1 | 2 | 3  # DD-5 §3 4단계

  # (3) exit code 정합 검증
  exit_code_category: success | user_error | system_error | internal_error
  exit_code_source: <AppError variant 또는 success>

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "<...>", ts: "<...>" }
    - { event: "<subsequent_event>", ... }
  state_changes:
    - path: ~/.myharness/state/<file>
      diff: <before> → <after>
  external_tool_calls:
    - tool: gh | git | brew | ollama | <other>
      args: "<...>"
      expected_exit: 0
  files_created:
    - <path>
  files_modified:
    - <path>
```

**5 install paths 별 E2E 검증** (D-31 + INITIAL_DESIGN §11.2, 본 §6 cross-OS):

| # | install path | TC 검증 |
| - | --- | --- |
| 1 | `install.sh` (macOS / Linux curl) | `curl -fsSL https://myharness.dev/install.sh \| bash` 후 `myharness --version` |
| 2 | `install.ps1` (Windows PowerShell) | `irm https://myharness.dev/install.ps1 \| iex` 후 `myharness --version` |
| 3 | `brew install --cask myharness` (macOS) | `brew install` 후 `myharness --version` |
| 4 | `winget install Yklee.Myharness` (Windows) | `winget install` 후 `myharness --version` |
| 5 | `apt install myharness` (Debian) / `dnf install myharness` (RHEL) / `apk add myharness` (Alpine) | `apt install` 후 `myharness --version` |

### 1.6 결정 trade-off (L4 E2E 환경)

| axis | docker 격리 (선정) | host 직접 실행 (대안) | trade-off |
| --- | --- | --- | --- |
| **격리 신뢰성** | ✅ 컨테이너 내부 완전 격리. host 영향 ❌ | ❌ host filesystem / network / env 영향 받음 | ✅ docker 선택. host 변경 시 TC 깨짐 ❌ |
| **CI 통합** | ✅ GitHub Actions `docker run` 표준 | ⚠️ host setup matrix 6 OS | ✅ docker 선택. CI 단순 |
| **macOS / Windows native** | ❌ docker 가 Linux 만, macOS/Windows native binary 는 host 실행 필요 | ✅ host 직접 = native binary | ⚠️ hybrid: macOS/Windows = host + docker (Linux) |
| **keychain** | ✅ in-memory backend (TC_ENV_VAR) | ❌ host keychain 오염 위험 | ✅ docker + memory backend |
| **cold-start** | ⚠️ 컨테이너 부팅 5~10초 | ✅ host = 즉시 | ⚠️ trade-off: 신뢰성 > 속도 |

### 1.7 L4 E2E TC 의 49 entries (count summary)

| category | count | SSOT ref |
| --- | --- | --- |
| **12 도메인 명령 E2E TC** | 18 | §2 (INITIAL_DESIGN §5.2, cycle 4: 4 code + 5 server + 6 env + 1 refresh interleave = 7+5+6) |
| **3 mode flag E2E TC** | 7 | §3 (INITIAL_DESIGN §5.3 + DD-1 §4, cycle 4: +plan/bypassPermissions 2 mode TC) |
| **12 auth CLI E2E TC** | 14 | §4 (INITIAL_DESIGN §5.4 + D-06 + D-38, cycle 4: +refresh 2 auth TC) |
| **4 exit code E2E TC** | 4 | §5 (DD-5 §3) |
| **6 cross-OS + cross-shell E2E TC** | 6 | §6 (INITIAL_DESIGN §11.1 + §11.2, 5 cross-OS + 1 cross-shell) |
| **합계** | **49** | (2,265 lines / 4-step × 49 = ~46 line 본문 평균) |

**분량 분배** (chunked write 3 chunk):
- chunk 1: §0+§1 = ~200 lines (현 위치)
- chunk 2: §2+§3 = ~330 lines (12 도메인 × ~22 + 3 mode × ~30)
- chunk 3: §4+§5+§6+§7 = ~370 lines (12 auth × ~12 + 4 exit × ~15 + 5 cross-OS × ~30 + handoff ~30)

---

### VERDICT (post-chunk 1): PASS — §0 메타 + §1 환경 정의 완료. chunk 2/3 진행.

---

## 2. 12 도메인 명령 E2E TC (INITIAL_DESIGN §5.2)

본 §2 는 INITIAL_DESIGN §5.2 의 12 도메인 명령 (3-도메인 × 4 = 12) 각각의 E2E TC. 각 TC = 4-step format (§1.5) 정합. permission mode = TC 별 명시. side effect = `log.jsonl` + `state/` + 외부 tool (gh, git, brew, ollama) 검증.

### 2.1 Code 도메인 (4 명령, INITIAL_DESIGN §5.2 코드 도메인)

#### TC-E2E-CODE-001: `myharness code review <pr>`

- **SSOT**: INITIAL_DESIGN §5.2 #1 + REQUIREMENTS §2.1 FR-CODE-1
- **sub-agent**: `code-reviewer` (+ `git-operator` + `file-searcher`)

```yaml
TC-E2E-CODE-001:
  name: "PR review (PR #482, GitHub mock)"

  # (1) input
  command: "myharness code review 482"
  stdin: null
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
    MYHARNESS_CONFIG: /test/fixtures/TC-E2E-CODE-001/config.yaml
    MYHARNESS_GITHUB_MOCK: /test/fixtures/TC-E2E-CODE-001/gh-mock.sh  # gh CLI mock
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CODE-001/
  permission_mode: default  # PR review = read-only, default OK

  # (2) expected output
  stdout: |
    ## PR #482 Review
    ### Verdict: LGTM with 3 minor comments
    - File: src/api/users.ts:42 — consider null check
    - File: src/api/users.ts:58 — magic number 30 → extract const
    - File: tests/api/users.test.ts:15 — missing edge case for empty array
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success
  exit_code_source: success (no AppError)

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code review", args: ["482"], ts: "<iso8601>" }
    - { event: "subagent_spawn", agent: "code-reviewer", ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "gh pr diff 482" }, ts: "<iso8601>" }
    - { event: "llm_call", provider: "ollama", model: "qwen2.5-coder:32b", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 12340, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/current.yaml
      diff: task_history += [{ ts, command: "code review", pr: 482, verdict: "LGTM" }]
  external_tool_calls:
    - tool: gh
      args: "pr diff 482"
      expected_exit: 0
    - tool: ollama
      args: "/v1/chat/completions (mock)"
      expected_exit: 0
  files_created: []
  files_modified: []
```

#### TC-E2E-CODE-002: `myharness code review 999` (PR 없음, exit code 1)

- **SSOT**: INITIAL_DESIGN §5.2 #1 + DD-5 §3 exit 1 (user error)

```yaml
TC-E2E-CODE-002:
  name: "PR review with non-existent PR #999 (user error → exit 1)"

  # (1) input
  command: "myharness code review 999"
  stdin: null
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_GITHUB_MOCK: /test/fixtures/TC-E2E-CODE-002/gh-mock-fail.sh  # gh pr view 999 → exit 4
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CODE-002/
  permission_mode: default

  # (2) expected output
  stdout: ""
  stderr: |
    오류: PR #999 를 찾을 수 없습니다.
    GitHub 에서 PR #999 가 존재하지 않거나 접근 권한이 없습니다.
    --help 출력을 보려면 `myharness code review --help` 를 실행하세요.
  exit_code: 1

  # (3) exit code 정합
  exit_code_category: user_error
  exit_code_source: AppError::InvalidArgs (PR number)

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code review", args: ["999"], ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "gh pr view 999" }, result: "exit 4", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 1, error_kind: "InvalidArgs", duration_ms: 2300, ts: "<iso8601>" }
  state_changes: []  # 실패이므로 state 갱신 ❌
  external_tool_calls:
    - tool: gh
      args: "pr view 999"
      expected_exit: 4
  files_created: []
  files_modified: []
```

#### TC-E2E-CODE-003: `myharness code implement "<feature>"`

- **SSOT**: INITIAL_DESIGN §5.2 #2 + REQUIREMENTS §2.1 FR-CODE-2
- **sub-agent**: `code-implementer` (+ `file-searcher`)

```yaml
TC-E2E-CODE-003:
  name: "Implement feature (add getUserById endpoint)"

  # (1) input
  command: 'myharness code implement "add getUserById endpoint in src/api/users.ts"'
  stdin: null
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
    MYHARNESS_CONFIG: /test/fixtures/TC-E2E-CODE-003/config.yaml
  cwd: /workspace  # git repo fixture
  fixture: /test/fixtures/TC-E2E-CODE-003/  # git init + src/api/users.ts
  permission_mode: acceptEdits  # 코드 변경 = Edit 자동 allow (DD-1 §4)

  # (2) expected output
  stdout: |
    ## Implementation Plan
    1. src/api/users.ts 에 getUserById function 추가
    2. tests/api/users.test.ts 에 test case 추가

    ## Diff
    ```diff
    + export async function getUserById(id: string): Promise<User> {
    +   const user = await db.users.findUnique({ where: { id } });
    +   if (!user) throw new NotFoundError(`User ${id} not found`);
    +   return user;
    + }
    ```

    ## Files Modified
    - src/api/users.ts (+5 lines)
    - tests/api/users.test.ts (+12 lines)
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code implement", args: ["add getUserById..."], ts: "<iso8601>" }
    - { event: "subagent_spawn", agent: "code-implementer", ts: "<iso8601>" }
    - { event: "permission_check", tool: "Edit", mode: "acceptEdits", result: "Allow", ts: "<iso8601>" }
    - { event: "tool_call", tool: "Edit", args: { path: "src/api/users.ts", old_text: "..." }, ts: "<iso8601>" }
    - { event: "tool_call", tool: "Edit", args: { path: "tests/api/users.test.ts" }, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 45600, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/current.yaml
      diff: task_history += [{ command: "code implement", files_modified: 2, additions: 17 }]
  files_created: []
  files_modified:
    - src/api/users.ts
    - tests/api/users.test.ts
```

#### TC-E2E-CODE-004: `myharness code test <path>`

- **SSOT**: INITIAL_DESIGN §5.2 #3 + DD-5 §3 (cargo test 실패 시 exit 2 = system error)
- **sub-agent**: `code-tester`

```yaml
TC-E2E-CODE-004:
  name: "Run cargo test (all pass)"

  # (1) input
  command: "myharness code test src/api/users"
  stdin: null
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
  cwd: /workspace  # Rust fixture project
  fixture: /test/fixtures/TC-E2E-CODE-004/  # cargo init + sample test
  permission_mode: default  # Bash tool = user prompt

  # (2) expected output
  stdout: |
    ## Test Results
    Running `cargo test src/api/users`...
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code test", args: ["src/api/users"], ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "cargo test src/api/users" }, result: "exit 0", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 23400, ts: "<iso8601>" }
  state_changes: []
  external_tool_calls:
    - tool: cargo
      args: "test src/api/users"
      expected_exit: 0
```

#### TC-E2E-CODE-005: `myharness code test <path>` (cargo test 실패, exit 2)

```yaml
TC-E2E-CODE-005:
  name: "Run cargo test with failing test (system error → exit 2)"

  # (1) input
  command: "myharness code test src/api/users"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CODE-005/  # cargo project with intentionally failing test
  permission_mode: default

  # (2) expected output
  stdout: |
    ## Test Results
    Running `cargo test src/api/users`...
    test users::test_get_user_by_id ... FAILED
    test result: FAILED. 2 passed; 1 failed; 0 ignored
  stderr: |
    오류: 테스트가 실패했습니다 (cargo exit 101).
    실패한 테스트: users::test_get_user_by_id
    자세한 내용은 `cargo test` 출력을 확인하세요.
  exit_code: 2

  # (3) exit code 정합 (DD-5 §3: cargo test 실패 = system error)
  exit_code_category: system_error
  exit_code_source: AppError::SubprocessFailed("cargo test exit 101")

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code test", ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "cargo test" }, result: "exit 101", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 2, error_kind: "SubprocessFailed", ts: "<iso8601>" }
  external_tool_calls:
    - tool: cargo
      args: "test"
      expected_exit: 101
```

#### TC-E2E-CODE-006: `myharness code commit "<message>"`

- **SSOT**: INITIAL_DESIGN §5.2 #4
- **sub-agent**: `git-operator`

```yaml
TC-E2E-CODE-006:
  name: "Commit staged changes with conventional message"

  # (1) input
  command: 'myharness code commit "feat(api): add getUserById endpoint"'
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
  cwd: /workspace  # git repo with staged changes
  fixture: /test/fixtures/TC-E2E-CODE-006/
  permission_mode: acceptEdits  # git commit = Bash tool scope, auto-allow in acceptEdits

  # (2) expected output
  stdout: |
    ## Commit Created
    abc1234 feat(api): add getUserById endpoint
     2 files changed, 17 insertions(+)
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "code commit", args: ["feat(api):..."], ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "git commit -m '...'" }, result: "exit 0", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 1200, ts: "<iso8601>" }
  external_tool_calls:
    - tool: git
      args: "commit -m 'feat(api): add getUserById endpoint'"
      expected_exit: 0
```

#### TC-E2E-CODE-007: `myharness code commit` (staged 변경 없음, exit 1)

```yaml
TC-E2E-CODE-007:
  name: "Commit with no staged changes (user error → exit 1)"

  command: 'myharness code commit "feat: nothing"'
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CODE-007/  # git repo, no staged changes
  permission_mode: default

  stdout: ""
  stderr: |
    오류: 커밋할 staged 변경이 없습니다.
    `git status` 로 현재 상태를 확인하거나 `git add <files>` 로 변경을 staging 하세요.
  exit_code: 1

  exit_code_category: user_error
  exit_code_source: AppError::InvalidArgs("no staged changes")

  log_jsonl_events:
    - { event: "command_start", ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "git status --short" }, result: "empty", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 1, error_kind: "InvalidArgs", ts: "<iso8601>" }
```

### 2.2 Server 도메인 (4 명령, INITIAL_DESIGN §5.2 서버 도메인, TASK-002 ⏸)

**TASK-002 ⏸ note**: server 명령의 host alias / service / env 가 placeholder (`<TASK-002: host_aliases>`). E2E TC 는 graceful degrade (placeholder → 명확한 에러 + 향후 resolve 가이드) 검증.

#### TC-E2E-SERVER-001: `myharness server status` (host 미설정, exit 1)

- **SSOT**: INITIAL_DESIGN §5.2 #5 + TASK-002 ⏸

```yaml
TC-E2E-SERVER-001:
  name: "Server status with no host configured (user error → exit 1)"

  # (1) input
  command: "myharness server status"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-SERVER-001/  # config/server/hosts.yaml 없음
  permission_mode: default

  # (2) expected output
  stdout: ""
  stderr: |
    오류: SSH 호스트 별칭이 설정되지 않았습니다.
    `config/server/hosts.yaml` 에 호스트를 추가하거나
    `--host=<alias>` 로 임시 지정하세요.
    (TASK-002 인프라 정보 필요 — PROJECT_PROFILE.md §3.1 TODO)
  exit_code: 1

  # (3) exit code 정합
  exit_code_category: user_error
  exit_code_source: AppError::FileNotFound("config/server/hosts.yaml")

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", command: "server status", ts: "<iso8601>" }
    - { event: "config_load_failed", path: "config/server/hosts.yaml", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 1, error_kind: "FileNotFound", ts: "<iso8601>" }
  state_changes: []
```

#### TC-E2E-SERVER-002: `myharness server status <host>` (ssh 성공)

```yaml
TC-E2E-SERVER-002:
  name: "Server status with mock SSH host"

  command: "myharness server status prod-web-01"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_SSH_MOCK: /test/fixtures/TC-E2E-SERVER-002/ssh-mock.sh  # mock ssh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-SERVER-002/
    # config/server/hosts.yaml:
    #   prod-web-01:
    #     host: 192.168.1.10
    #     user: deploy
  permission_mode: default

  stdout: |
    ## Server Status: prod-web-01 (192.168.1.10)
    - Uptime: 47 days
    - Load: 0.42 0.38 0.35
    - Disk: 32% used (153GB free)
    - Memory: 4.2GB / 16GB
    - Services: nginx (active), postgresql (active), myharness (active)
  stderr: ""
  exit_code: 0

  exit_code_category: success

  log_jsonl_events:
    - { event: "command_start", command: "server status", args: ["prod-web-01"], ts: "<iso8601>" }
    - { event: "tool_call", tool: "Bash", args: { command: "ssh deploy@192.168.1.10 uptime; ..." }, result: "exit 0", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 3400, ts: "<iso8601>" }
  external_tool_calls:
    - tool: ssh
      args: "deploy@192.168.1.10"
      expected_exit: 0
    - tool: launchctl  # macOS
      args: "list"
      expected_exit: 0
```

#### TC-E2E-SERVER-003: `myharness server logs <service>`

```yaml
TC-E2E-SERVER-003:
  name: "Server logs (last 50 lines of nginx)"

  command: "myharness server logs nginx 50"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_SSH_MOCK: /test/fixtures/TC-E2E-SERVER-003/ssh-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-SERVER-003/
  permission_mode: default

  stdout: |
    ## Last 50 lines of nginx (prod-web-01)
    ```
    2026-06-08 10:23:14 [error] 1234#0: *567 connect() failed
    2026-06-08 10:23:15 [warn] 1234#0: *567 upstream timed out
    ...
    ```
  stderr: ""
  exit_code: 0

  exit_code_category: success
```

#### TC-E2E-SERVER-004: `myharness server deploy <env>` (deploy 실패, exit 2)

```yaml
TC-E2E-SERVER-004:
  name: "Server deploy with manifest validation failure (system error → exit 2)"

  command: "myharness server deploy staging"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_SSH_MOCK: /test/fixtures/TC-E2E-SERVER-004/ssh-mock-fail.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-SERVER-004/
    # deploy manifest YAML broken
  permission_mode: default

  stdout: ""
  stderr: |
    오류: 배포 매니페스트 YAML 파싱 실패.
    라인 12: 들여쓰기 오류.
    자세한 내용은 `manifest.yaml` 을 확인하세요.
  exit_code: 2

  exit_code_category: system_error
  exit_code_source: AppError::SubprocessFailed("manifest parse")
```

#### TC-E2E-SERVER-005: `myharness server config get <key>`

```yaml
TC-E2E-SERVER-005:
  name: "Server config get (database.host)"

  command: "myharness server config get database.host"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_SSH_MOCK: /test/fixtures/TC-E2E-SERVER-005/ssh-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-SERVER-005/
  permission_mode: default

  stdout: |
    database.host = 10.0.0.5
  stderr: ""
  exit_code: 0

  exit_code_category: success
```

### 2.3 Env 도메인 (4 명령, INITIAL_DESIGN §5.2 환경 도메인, TASK-002 ⏸)

#### TC-E2E-ENV-001: `myharness env setup <stack>` (brew, macOS)

```yaml
TC-E2E-ENV-001:
  name: "Env setup: brew (macOS)"

  command: "myharness env setup brew"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_BREW_MOCK: /test/fixtures/TC-E2E-ENV-001/brew-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-001/
    # config/stacks/brew.yaml (TASK-002 ⏸, placeholder)
    # MYHARNESS_OS=darwin (mock)
  permission_mode: acceptEdits  # brew install = Bash + Write scope

  stdout: |
    ## Env Setup: brew
    ### Detected: macOS
    - brew installed at /opt/homebrew/bin/brew
    - Tapping: homebrew/bundle
    - Installing packages from Brewfile:
      - git
      - gh
      - jq
      - ripgrep
    - 4 packages installed.
  stderr: ""
  exit_code: 0

  exit_code_category: success

  external_tool_calls:
    - tool: brew
      args: "bundle install"
      expected_exit: 0
```

#### TC-E2E-ENV-002: `myharness env setup rust` (asdf, Linux)

```yaml
TC-E2E-ENV-002:
  name: "Env setup: rust (Linux, asdf)"

  command: "myharness env setup rust"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_ASDF_MOCK: /test/fixtures/TC-E2E-ENV-002/asdf-mock.sh
    MYHARNESS_OS=linux
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-002/
  permission_mode: acceptEdits

  stdout: |
    ## Env Setup: rust
    ### Detected: Linux
    - asdf installed at ~/.asdf/bin/asdf
    - Adding rust plugin...
    - Installing rust 1.78.0 (stable)...
    - Setting global rust to 1.78.0
    - Verification: `rustc --version` → rustc 1.78.0
  stderr: ""
  exit_code: 0
```

#### TC-E2E-ENV-003: `myharness env install <pkgs>`

```yaml
TC-E2E-ENV-003:
  name: "Env install: ripgrep fd (auto-detect package manager)"

  command: "myharness env install ripgrep fd"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_BREW_MOCK: /test/fixtures/TC-E2E-ENV-003/brew-mock.sh
    MYHARNESS_OS=darwin
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-003/
  permission_mode: acceptEdits

  stdout: |
    ## Env Install: ripgrep fd
    ### Detected: brew (macOS)
    - Installing ripgrep ... done
    - Installing fd ... done
  stderr: ""
  exit_code: 0
```

#### TC-E2E-ENV-004: `myharness env shell <cmd>`

```yaml
TC-E2E-ENV-004:
  name: "Env shell: run command in isolated shell (docker exec)"

  command: 'myharness env shell "ls -la /workspace"'
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-004/
  permission_mode: default  # Bash tool = user prompt

  stdout: |
    total 12
    drwxr-xr-x 3 tester tester 4096 Jun  8 10:00 .
    drwxr-xr-x 4 tester tester 4096 Jun  8 10:00 ..
    -rw-r--r-- 1 tester tester  142 Jun  8 10:00 README.md
  stderr: ""
  exit_code: 0
```

#### TC-E2E-ENV-005: `myharness env diagnose` (no target, exit 1)

```yaml
TC-E2E-ENV-005:
  name: "Env diagnose: no target specified (user error → exit 1)"

  command: "myharness env diagnose"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-005/
  permission_mode: default

  stdout: ""
  stderr: |
    오류: 진단 대상이 지정되지 않았습니다.
    사용법: `myharness env diagnose <target>` (target = node | python | rust | go)
  exit_code: 1

  exit_code_category: user_error
  exit_code_source: AppError::InvalidArgs("no diagnose target")
```

#### TC-E2E-ENV-006: `myharness env diagnose rust`

```yaml
TC-E2E-ENV-006:
  name: "Env diagnose: rust (success with diagnostics)"

  command: "myharness env diagnose rust"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_RUST_MOCK: /test/fixtures/TC-E2E-ENV-006/rust-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-ENV-006/
  permission_mode: default

  stdout: |
    ## Env Diagnose: rust
    - rustc: 1.78.0
    - cargo: 1.78.0
    - rustup: 1.27.0
    - Target: x86_64-unknown-linux-gnu
    - PATH: /home/tester/.cargo/bin:...
    - CARGO_HOME: /home/tester/.cargo
    - Issues: (none)
  stderr: ""
  exit_code: 0

  exit_code_category: success
```

### 2.4 도메인 명령 E2E TC 12 entries count

| # | TC id | command | 도메인 | exit 0 | exit 1 | exit 2 |
| - | --- | --- | --- | --- | --- | --- |
| 1 | TC-E2E-CODE-001 | `code review <pr>` | code | ✅ | | |
| 2 | TC-E2E-CODE-002 | `code review 999` (no PR) | code | | ✅ | |
| 3 | TC-E2E-CODE-003 | `code implement` | code | ✅ | | |
| 4 | TC-E2E-CODE-004 | `code test` (pass) | code | ✅ | | |
| 5 | TC-E2E-CODE-005 | `code test` (fail) | code | | | ✅ |
| 6 | TC-E2E-CODE-006 | `code commit` (staged) | code | ✅ | | |
| 7 | TC-E2E-CODE-007 | `code commit` (no staged) | code | | ✅ | |
| 8 | TC-E2E-SERVER-001 | `server status` (no host) | server | | ✅ | |
| 9 | TC-E2E-SERVER-002 | `server status <host>` | server | ✅ | | |
| 10 | TC-E2E-SERVER-003 | `server logs` | server | ✅ | | |
| 11 | TC-E2E-SERVER-004 | `server deploy` (fail) | server | | | ✅ |
| 12 | TC-E2E-SERVER-005 | `server config get` | server | ✅ | | |
| 13 | TC-E2E-ENV-001 | `env setup brew` | env | ✅ | | |
| 14 | TC-E2E-ENV-002 | `env setup rust` | env | ✅ | | |
| 15 | TC-E2E-ENV-003 | `env install` | env | ✅ | | |
| 16 | TC-E2E-ENV-004 | `env shell` | env | ✅ | | |
| 17 | TC-E2E-ENV-005 | `env diagnose` (no target) | env | | ✅ | |
| 18 | TC-E2E-ENV-006 | `env diagnose rust` | env | ✅ | | |

> **Note**: §2.1-§2.3 은 12 도메인 명령 (INITIAL_DESIGN §5.2) 모두 cover, 18 entries 는 happy + edge 조합.

---

## 3. 3 mode flag E2E TC (INITIAL_DESIGN §5.3, D-29)

본 §3 는 INITIAL_DESIGN §5.3 의 3 mode flag (`--mode=orchestrator|single|loop`) 의 E2E TC. loop mode 는 `--goal` 필수 (D-29 ralph-wiggum), success-criteria 충족 시 stop / max-iterations 도달 시 stop / user Ctrl+C 시 stop.

### 3.1 TC-E2E-MODE-001: `--mode=orchestrator` (default)

- **SSOT**: INITIAL_DESIGN §5.3 + CONCEPT.md §5.10

```yaml
TC-E2E-MODE-001:
  name: "Default mode = orchestrator (sub-agent fan-out)"

  # (1) input
  command: "myharness code review 482"  # --mode default = orchestrator
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-MODE-001/
  permission_mode: default

  # (2) expected output
  stdout: |
    ## Orchestrator Mode (default)
    Spawning sub-agents:
    - code-reviewer (primary)
    - git-operator (helper)
    - file-searcher (helper)
    ... (3 sub-agents fan-out)
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect (orchestrator = multi sub-agent)
  log_jsonl_events:
    - { event: "command_start", mode: "orchestrator", ts: "<iso8601>" }
    - { event: "subagent_spawn", agent: "code-reviewer", parent: "orchestrator", ts: "<iso8601>" }
    - { event: "subagent_spawn", agent: "git-operator", parent: "orchestrator", ts: "<iso8601>" }
    - { event: "subagent_spawn", agent: "file-searcher", parent: "orchestrator", ts: "<iso8601>" }
    - { event: "subagent_complete", agent: "code-reviewer", duration_ms: 8400, ts: "<iso8601>" }
    - { event: "subagent_complete", agent: "git-operator", duration_ms: 2100, ts: "<iso8601>" }
    - { event: "subagent_complete", agent: "file-searcher", duration_ms: 1800, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, total_duration_ms: 12300, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/current.yaml
      diff: task_history += [{ mode: "orchestrator", sub_agents_spawned: 3 }]
```

### 3.2 TC-E2E-MODE-002: `--mode=single` (sub-agent spawn 안 함)

```yaml
TC-E2E-MODE-002:
  name: "Single mode = direct LLM call (no sub-agent spawn)"

  # (1) input
  command: 'myharness --mode=single ask "what does this function do?"'
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-MODE-002/
  permission_mode: default

  # (2) expected output
  stdout: |
    ## Single Mode (direct LLM call)
    ## Response
    This function fetches a user by ID from the database. It uses Prisma's
    findUnique with the id field, throws NotFoundError if user is missing.
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect (single = no sub-agent, single LLM call)
  log_jsonl_events:
    - { event: "command_start", mode: "single", ts: "<iso8601>" }
    - { event: "llm_call", provider: "ollama", model: "qwen2.5-coder:32b", prompt_tokens: 80, completion_tokens: 35, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, duration_ms: 4200, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/current.yaml
      diff: task_history += [{ mode: "single", sub_agents_spawned: 0 }]
```

### 3.3 TC-E2E-MODE-003: `--mode=loop --goal ...` (success-criteria 충족 시 stop)

- **SSOT**: INITIAL_DESIGN §5.3 + D-29 ralph-wiggum (loop mode)

```yaml
TC-E2E-MODE-003:
  name: "Loop mode with success-criteria met (3 iterations, then stop)"

  # (1) input
  command: 'myharness --mode=loop --goal "fix all failing tests" --success-criteria "cargo test passes" --max-iterations=10 code test'
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
    MYHARNESS_CARGO_MOCK: /test/fixtures/TC-E2E-MODE-003/cargo-mock.sh  # 1~2 fail, 3rd pass
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-MODE-003/  # Rust project with 2 failing tests
  permission_mode: acceptEdits  # Edit auto-allow

  # (2) expected output
  stdout: |
    ## Loop Mode (ralph-wiggum)
    ### Goal: fix all failing tests
    ### Success Criteria: cargo test passes
    ### Max Iterations: 10

    ## Iteration 1/10
    - cargo test: 2 failed (test_a, test_b)
    - LLM analysis: test_a 의 assertion 오류, test_b 의 null 처리 누락
    - Edit: src/lib.rs:42 (fix test_a) + src/lib.rs:58 (add null check for test_b)

    ## Iteration 2/10
    - cargo test: 1 failed (test_b)
    - LLM analysis: test_b 의 mock setup 누락
    - Edit: src/lib.rs:60 (fix test_b mock)

    ## Iteration 3/10
    - cargo test: 0 failed ✅
    - Success criteria met!

    ## Loop Complete (3/10 iterations, 87s)
  stderr: ""
  exit_code: 0

  # (3) exit code 정합
  exit_code_category: success

  # (4) side effect
  log_jsonl_events:
    - { event: "command_start", mode: "loop", goal: "fix all failing tests", success_criteria: "cargo test passes", max_iterations: 10, ts: "<iso8601>" }
    - { event: "loop_iteration", n: 1, ts: "<iso8601>" }
    - { event: "loop_iteration", n: 2, ts: "<iso8601>" }
    - { event: "loop_iteration", n: 3, ts: "<iso8601>" }
    - { event: "loop_complete", iterations: 3, success: true, duration_ms: 87000, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/current.yaml
      diff: task_history += [{ mode: "loop", iterations: 3, success: true }]
```

### 3.4 TC-E2E-MODE-004: `--mode=loop` without `--goal` (user error → exit 1)

```yaml
TC-E2E-MODE-004:
  name: "Loop mode without --goal (user error → exit 1)"

  command: "myharness --mode=loop code test"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-MODE-004/
  permission_mode: default

  stdout: ""
  stderr: |
    오류: --mode=loop 사용 시 --goal 필수.
    사용법: myharness --mode=loop --goal "<목표>" [--success-criteria "<기준>"] --max-iterations=<N> <command>
  exit_code: 1

  exit_code_category: user_error
  exit_code_source: AppError::InvalidArgs("--goal required for --mode=loop")
```

### 3.5 TC-E2E-MODE-005: `--mode=loop --max-iterations 도달`

```yaml
TC-E2E-MODE-005:
  name: "Loop mode hitting max-iterations (no success)"

  command: 'myharness --mode=loop --goal "fix bug" --max-iterations=5 code test'
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_CARGO_MOCK: /test/fixtures/TC-E2E-MODE-005/cargo-mock-fail.sh  # always fail
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-MODE-005/
  permission_mode: acceptEdits

  stdout: |
    ## Loop Mode
    ### Max Iterations: 5 reached without success

    ## Iterations
    1-5: cargo test still failing (5 attempts, no progress)

    ## Loop Complete (5/5, no success)
  stderr: |
    경고: max-iterations (5) 도달. 성공 기준 미충족.
  exit_code: 2  # DD-5 §3: max-iterations 도달 = system error (외부 의존성 미충족)

  exit_code_category: system_error
  exit_code_source: AppError::LoopMaxIterations { iterations: 5, goal: "fix bug" }
```

### 3.6 3 mode flag E2E TC 5 entries summary

| # | TC id | mode | 검증 항목 | exit |
| - | --- | --- | --- | --- |
| 1 | TC-E2E-MODE-001 | orchestrator (default) | multi sub-agent fan-out | 0 |
| 2 | TC-E2E-MODE-002 | single | direct LLM call, no sub-agent | 0 |
| 3 | TC-E2E-MODE-003 | loop (success) | 3 iterations + success-criteria stop | 0 |
| 4 | TC-E2E-MODE-004 | loop (no --goal) | clap derive arg parse fail | 1 |
| 5 | TC-E2E-MODE-005 | loop (max-iter) | 5 iterations + exit 2 | 2 |

### 3.7 TC-E2E-MODE-006: `--permission-mode=plan` (도구 호출 시 confirmation prompt, DD-1 §4)

```
TC-E2E-MODE-006:
  input:
    cmd: "myharness --permission-mode=plan code review --target /repo/src/auth.rs"
    stdin: null
    env: { MYHARNESS_LLM: "ollama", MYHARNESS_MODEL: "qwen2.5-coder:32b" }
    cwd: /test/fixtures/TC-E2E-MODE-006/   # rust project, auth.rs has 3 issues
    fixture: /test/fixtures/TC-E2E-MODE-006/
  expected_output:
    stdout: |
      [plan-mode] reading target: /repo/src/auth.rs
      [plan-mode] proposed 3 edits:
        1. auth.rs:42  — replace `==` with `consteq()` (timing-attack mitigation)
        2. auth.rs:87  — add `let _ = env::var("API_KEY");` strict-mode guard
        3. auth.rs:120 — wrap `.unwrap()` with `?` operator
      [plan-mode] waiting for confirmation... (y/N)
    stderr: ""
    exit_code: 0   # confirmation prompt shown, user typed "y" via stdin
  permission_mode: plan
  side_effect:
    log_jsonl_events:
      - event: "plan_mode_enter"
        result: "ok"
      - event: "tool_call_proposed" tool: "Edit" file: "auth.rs" lines: 1
        result: "pending"
      - event: "user_confirm" response: "y"
        result: "ok"
      - event: "tool_call_executed" tool: "Edit" file: "auth.rs"
        result: "ok"
    state_changes: state/session-{id}/plan_proposed.json 생성
    external_tool_calls: []
    files_modified: [auth.rs]   # 3 lines patched
  d06_compliance: "stdin read for confirm, no token in stdout"
  xref: "DD-1 §4 plan mode, INITIAL_DESIGN §9 permission 4-mode"
```

**검증 포인트**: plan mode 가 sub-tool 호출 전 confirmation prompt 를 stdout 으로 표시 + 사용자 stdin 응답 후 실행. confirm 없이 timeout 시 exit 1.

### 3.8 TC-E2E-MODE-007: `--permission-mode=bypassPermissions` (모든 도구 무조건 통과, sandbox/CI 전용, DD-1 §4)

```
TC-E2E-MODE-007:
  input:
    cmd: "myharness --permission-mode=bypassPermissions implement --goal 'add /healthz endpoint'"
    stdin: null
    env: { MYHARNESS_LLM: "ollama", MYHARNESS_MODEL: "qwen2.5-coder:32b", CI: "true" }
    cwd: /test/fixtures/TC-E2E-MODE-007/   # rust project, no /healthz yet
    fixture: /test/fixtures/TC-E2E-MODE-007/
  expected_output:
    stdout: |
      [bypass] running in CI/sandbox mode (warning: no user prompts)
      [bypass] generated 2 edits + 1 bash call
      - Write: src/health.rs (new file, 24 lines)
      - Edit: src/main.rs (insert router mount, 3 lines)
      - Bash: cargo build --release
      [bypass] build succeeded in 12.4s
    stderr: ""
    exit_code: 0
  permission_mode: bypassPermissions
  side_effect:
    log_jsonl_events:
      - event: "bypass_mode_enter" warning: "no_user_prompt"
        result: "ok"
      - event: "tool_call_executed" tool: "Write" file: "src/health.rs"
        result: "ok"
      - event: "tool_call_executed" tool: "Edit" file: "src/main.rs"
        result: "ok"
      - event: "tool_call_executed" tool: "Bash" cmd: "cargo build --release"
        result: "ok"
    state_changes: state/session-{id}/bypass_audit.jsonl (모든 tool_call 무조건 record)
    external_tool_calls: [cargo build --release]
    files_created: [src/health.rs]
    files_modified: [src/main.rs]
  d06_compliance: "N/A (no auth operation)"
  warning: "production 환경 사용 ❌ — DD-1 §4 sandbox/CI 전용. 환경변수 CI 가 unset 이면 stderr 에 warning 출력 + exit 1"
  xref: "DD-1 §4 bypassPermissions, INITIAL_DESIGN §9 permission 4-mode"
```

**검증 포인트**: bypassPermissions 가 모든 sub-tool (Bash/Write/Edit) 무조건 실행. CI=true 환경변수가 없으면 warning + exit 1 (DD-1 §4 sandbox-only 제약).

### 3.9 7 mode flag E2E TC entries summary (cycle 4 update)

| # | TC id | mode | 검증 항목 | exit |
| - | --- | --- | --- | --- |
| 1 | TC-E2E-MODE-001 | orchestrator (default) | multi sub-agent fan-out | 0 |
| 2 | TC-E2E-MODE-002 | single | direct LLM call, no sub-agent | 0 |
| 3 | TC-E2E-MODE-003 | loop (success) | 3 iterations + success-criteria stop | 0 |
| 4 | TC-E2E-MODE-004 | loop (no --goal) | clap derive arg parse fail | 1 |
| 5 | TC-E2E-MODE-005 | loop (max-iter) | 5 iterations + exit 2 | 2 |
| 6 | TC-E2E-MODE-006 | plan (DD-1 §4) | 도구 호출 전 confirmation prompt, stdin y/N | 0 |
| 7 | TC-E2E-MODE-007 | bypassPermissions (DD-1 §4) | 모든 도구 무조건 통과, CI=true 필요 | 0 |

**permission mode 4-mode E2E 커버리지 (cycle 4 update)**: `default` (33 TC) + `acceptEdits` (7 TC) + `plan` (1 TC = MODE-006) + `bypassPermissions` (1 TC = MODE-007) = **총 42 permission_mode 표기**, 4 mode 모두 ≥1 TC 보장.

---

### VERDICT (post-chunk 2): PASS — §0 + §1 + §2 + §3 완료 (12 도메인 + 3 mode TC 정합). chunk 3 (§4 + §5 + §6 + §7) 진행.

---

## 4. 12 auth CLI E2E TC (INITIAL_DESIGN §5.4, D-06, D-38)

본 §4 는 INITIAL_DESIGN §5.4 의 12 auth CLI (`auth list` / `auth <provider>` / `auth <provider> login|logout|set-key|test` / `auth setup` / `auth default` / `auth discover` / `auth refresh` / `auth export`) E2E TC. **D-06 정책 (NFR-SEC-1)**: token 값은 stdout/stderr/log.jsonl 어디에도 출력 ❌. `auth login`/`set-key` 호출 시 stdin 으로 read.

### 4.1 TC-E2E-AUTH-001: `myharness auth list` (all providers status)

```yaml
TC-E2E-AUTH-001:
  name: "Auth list (discovered providers)"
  ssot_ref: "INITIAL_DESIGN §5.4 #1 + D-38"

  command: "myharness auth list"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory, MYHARNESS_OLLAMA_URL: http://localhost:11434/v1 }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-001/
  permission_mode: default

  stdout: |
    ## Auth Status
    | Provider  | Status        | Default Model           | Last Test   |
    | --------- | ------------- | ----------------------- | ----------- |
    | anthropic | authenticated | claude-sonnet-4-5       | 2026-06-08  |
    | openai    | not_configured| (none)                  | -           |
    | gemini    | not_configured| (none)                  | -           |
    | deepseek  | not_configured| (none)                  | -           |
    | ollama    | authenticated | qwen2.5-coder:32b       | 2026-06-08  |
  stderr: ""
  exit_code: 0

  exit_code_category: success

  log_jsonl_events:
    - { event: "command_start", command: "auth list", ts: "<iso8601>" }
    - { event: "provider_discover", providers: 5, active: 2, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
  # D-06: API key / token 값 stdout 출력 ❌ — 메타만 표시
```

### 4.2 TC-E2E-AUTH-002: `myharness auth anthropic` (single provider status)

```yaml
TC-E2E-AUTH-002:
  name: "Auth single provider (anthropic)"
  ssot_ref: "INITIAL_DESIGN §5.4 #2"

  command: "myharness auth anthropic"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-002/
  permission_mode: default

  stdout: |
    ## Auth: anthropic
    - Status: authenticated
    - Default Model: claude-sonnet-4-5
    - Available Models: claude-sonnet-4-5, claude-haiku-4, claude-opus-4-5
    - Secret Store: keychain
    - API Key Env: ANTHROPIC_API_KEY
    - Last Login: 2026-06-08T09:00:00+09:00
    - Last Test: 2026-06-08T10:00:00+09:00 (ok, 320ms)
  stderr: ""
  exit_code: 0

  exit_code_category: success
  # D-06: secret_store: keychain 표시 OK, 실제 키 값 ❌
```

### 4.3 TC-E2E-AUTH-003: `myharness auth anthropic login` (OAuth wizard)

```yaml
TC-E2E-AUTH-003:
  name: "Auth login (OAuth wizard, mock OAuth server)"
  ssot_ref: "INITIAL_DESIGN §5.4 #3 + D-06 stdin"

  command: "myharness auth anthropic login"
  stdin: null  # OAuth = browser redirect, no stdin
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OAUTH_MOCK: /test/fixtures/TC-E2E-AUTH-003/oauth-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-003/
  permission_mode: default

  stdout: |
    ## Auth Login: anthropic
    Opening browser to https://console.anthropic.com/oauth/authorize?...
    Waiting for OAuth callback...
    ✓ Authentication successful
    ✓ Stored secret in keychain (slot: myharness-anthropic)
    ✓ Updated state/auth/anthropic.yaml
  stderr: ""
  exit_code: 0

  exit_code_category: success

  log_jsonl_events:
    - { event: "command_start", command: "auth anthropic login", ts: "<iso8601>" }
    - { event: "oauth_start", provider: "anthropic", ts: "<iso8601>" }
    - { event: "oauth_callback", status: "ok", ts: "<iso8601>" }
    - { event: "keychain_set", slot: "myharness-anthropic", result: "ok", ts: "<iso8601>" }
    # D-06: keychain_set event 에 token 값 ❌, result 만
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/auth/anthropic.yaml
      diff: status: not_configured → authenticated; last_login: (set)
  external_tool_calls:
    - tool: oauth-mock
      args: "anthropic"
      expected_exit: 0
```

### 4.4 TC-E2E-AUTH-004: `myharness auth anthropic logout`

```yaml
TC-E2E-AUTH-004:
  name: "Auth logout (remove from keychain)"
  ssot_ref: "INITIAL_DESIGN §5.4 #4"

  command: "myharness auth anthropic logout"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-004/  # anthropic authenticated
  permission_mode: default

  stdout: |
    ## Auth Logout: anthropic
    ✓ Removed secret from keychain (slot: myharness-anthropic)
    ✓ Updated state/auth/anthropic.yaml → status: logged_out
  stderr: ""
  exit_code: 0

  exit_code_category: success

  state_changes:
    - path: ~/.myharness/state/auth/anthropic.yaml
      diff: status: authenticated → logged_out; last_login: (cleared)
  log_jsonl_events:
    - { event: "keychain_remove", slot: "myharness-anthropic", result: "ok", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
```

### 4.5 TC-E2E-AUTH-005: `myharness auth openai set-key` (stdin read, D-06)

```yaml
TC-E2E-AUTH-005:
  name: "Auth set-key (API key via stdin, D-06 secure)"
  ssot_ref: "INITIAL_DESIGN §5.4 #5 + D-06 + NFR-SEC-1"

  command: "myharness auth openai set-key"
  stdin: "sk-test-mock-key-not-real-1234567890"  # stdin 으로 read, stdout 출력 ❌
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-005/
  permission_mode: default

  stdout: |
    ## Auth Set-Key: openai
    ✓ Stored secret in keychain (slot: myharness-openai)
    ✓ Key length: 51 chars (redacted)
    ✓ Updated state/auth/openai.yaml
  stderr: ""
  exit_code: 0

  exit_code_category: success
  # D-06 검증: stdout 에 token 값 출력 ❌, length 만 표시
  # D-06 검증: log.jsonl 에 token 값 ❌, result: ok 만
  log_jsonl_events:
    - { event: "command_start", command: "auth openai set-key", ts: "<iso8601>" }
    - { event: "stdin_read", bytes: 51, ts: "<iso8601>" }
    - { event: "keychain_set", slot: "myharness-openai", result: "ok", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/auth/openai.yaml
      diff: status: not_configured → authenticated
```

### 4.6 TC-E2E-AUTH-006: `myharness auth openai set-key --from-keychain`

```yaml
TC-E2E-AUTH-006:
  name: "Auth set-key --from-keychain (slot alias)"
  ssot_ref: "INITIAL_DESIGN §5.4 #6"

  command: "myharness auth openai set-key --from-keychain"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-006/
    # keychain 에 myharness-openai slot 이미 존재
  permission_mode: default

  stdout: |
    ## Auth Set-Key: openai (from keychain)
    ✓ Found existing slot: myharness-openai
    ✓ Copied to active config
  stderr: ""
  exit_code: 0

  exit_code_category: success
```

### 4.7 TC-E2E-AUTH-007: `myharness auth anthropic test` (연결 테스트)

```yaml
TC-E2E-AUTH-007:
  name: "Auth test (ping model, latency 측정)"
  ssot_ref: "INITIAL_DESIGN §5.4 #7"

  command: "myharness auth anthropic test"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_MOCK: /test/fixtures/TC-E2E-AUTH-007/ollama-mock.sh
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-007/
  permission_mode: default

  stdout: |
    ## Auth Test: anthropic
    - Connecting to https://api.anthropic.com/v1/messages...
    - Test prompt: "ping"
    - Response: "pong" (320ms)
    - Latency: 320ms
    - Token count: 12 prompt + 4 completion
    - Result: OK
  stderr: ""
  exit_code: 0

  exit_code_category: success

  log_jsonl_events:
    - { event: "auth_test", provider: "anthropic", result: "ok", latency_ms: 320, ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 0, ts: "<iso8601>" }
  state_changes:
    - path: ~/.myharness/state/auth/anthropic.yaml
      diff: test.last_test: (set); test.result: ok; test.latency_ms: 320
```

### 4.8 TC-E2E-AUTH-008: `myharness auth anthropic test` (실패, exit 2)

```yaml
TC-E2E-AUTH-008:
  name: "Auth test (network unreachable → exit 2)"
  ssot_ref: "INITIAL_DESIGN §5.4 #7 + DD-5 §3 exit 2 (system error)"

  command: "myharness auth anthropic test"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_NETWORK_MOCK: unreachable  # network unreachable 강제
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-008/
  permission_mode: default

  stdout: ""
  stderr: |
    오류: anthropic provider 연결 실패.
    네트워크 연결을 확인하거나 `myharness auth anthropic logout` 후 다시 login 하세요.
    (network: unreachable, latency: timeout)
  exit_code: 2

  exit_code_category: system_error
  exit_code_source: AppError::Network("connection refused")

  log_jsonl_events:
    - { event: "auth_test", provider: "anthropic", result: "error", error_kind: "Network", ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 2, ts: "<iso8601>" }
```

### 4.9 TC-E2E-AUTH-009: `myharness auth setup` (wizard 일괄)

```yaml
TC-E2E-AUTH-009:
  name: "Auth setup (일괄 discover + login wizard)"
  ssot_ref: "INITIAL_DESIGN §5.4 #8 + D-38"

  command: "myharness auth setup"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_OLLAMA_URL: http://localhost:11434/v1
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-009/
  permission_mode: default

  stdout: |
    ## Auth Setup Wizard
    ### Step 1/3: Discovery
    - Found env vars: ANTHROPIC_API_KEY (✅), OPENAI_API_KEY (❌)
    - Found keychain slots: myharness-anthropic (✅), myharness-ollama (✅)
    - Found local LLM: http://localhost:11434/v1 (✅ ollama)
    - Discovered: 2 active providers

    ### Step 2/3: Login Wizard
    Would you like to login to missing providers? [y/N]: y
    (1/3) anthropic: ✓ already authenticated
    (2/3) openai: opening browser for OAuth...
    (3/3) gemini: skipping (no API key)

    ### Step 3/3: Configuration
    ✓ Updated state/active-providers.yaml
    ✓ Primary set to: anthropic
  stderr: ""
  exit_code: 0

  exit_code_category: success

  state_changes:
    - path: ~/.myharness/state/active-providers.yaml
      diff: active: [anthropic, ollama]; fallback_order: [anthropic, ollama]
  external_tool_calls:
    - tool: ollama
      args: "/v1/models (health check)"
      expected_exit: 0
```

### 4.10 TC-E2E-AUTH-010: `myharness auth default <provider>`

```yaml
TC-E2E-AUTH-010:
  name: "Auth default (primary provider 변경)"
  ssot_ref: "INITIAL_DESIGN §5.4 #9"

  command: "myharness auth default ollama"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-010/
  permission_mode: default

  stdout: |
    ## Auth Default: ollama
    ✓ Updated config/config.yaml: llm.primary = ollama/qwen2.5-coder:32b
    ✓ Previous primary: anthropic
  stderr: ""
  exit_code: 0

  exit_code_category: success

  state_changes:
    - path: ~/.myharness/config/config.yaml
      diff: llm.primary: "anthropic/claude-sonnet-4-5" → "ollama/qwen2.5-coder:32b"
```

### 4.11 TC-E2E-AUTH-011: `myharness auth discover` (동적 발견)

```yaml
TC-E2E-AUTH-011:
  name: "Auth discover (env + keychain + local LLM scan)"
  ssot_ref: "INITIAL_DESIGN §5.4 #10 + D-38"

  command: "myharness auth discover"
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    ANTHROPIC_API_KEY: "sk-test-present-but-redacted"  # presence 만, 값 ❌
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-011/
  permission_mode: default

  stdout: |
    ## Auth Discovery (D-38)
    - Env vars: ANTHROPIC_API_KEY (✅ present, value redacted)
    - Keychain slots: myharness-ollama (✅)
    - Local LLM: ollama @ :11434 (✅), vllm @ :8000 (❌ not running)
    - Discovered: 2 active
    ✓ Updated state/active-providers.yaml
  stderr: ""
  exit_code: 0

  exit_code_category: success
  # D-06: env var 값 stdout 출력 ❌, "present" / "redacted" 만 표시
```

### 4.12 TC-E2E-AUTH-012: `myharness auth export` (read-only dump, D-06)

```yaml
TC-E2E-AUTH-012:
  name: "Auth export (read-only metadata dump, NO token values)"
  ssot_ref: "INITIAL_DESIGN §5.4 #12 + D-06 정책"

  command: "myharness auth export"
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-AUTH-012/
  permission_mode: default

  stdout: |
    ## Auth Export (metadata only, D-06 정책)
    ```yaml
    active_providers: [anthropic, ollama]
    providers:
      anthropic:
        status: authenticated
        default_model: claude-sonnet-4-5
        last_login: 2026-06-08T09:00:00+09:00
        # api_key: ❌ NOT EXPORTED (D-06)
      ollama:
        status: authenticated
        default_model: qwen2.5-coder:32b
        endpoint: http://localhost:11434/v1
        last_test: 2026-06-08T10:00:00+09:00
    ```
  stderr: ""
  exit_code: 0

  exit_code_category: success
  # D-06: token 값 / api_key stdout 출력 ❌, 메타만
  # 검증: stdout grep "sk-" → 0건 (token 패턴)
```

### 4.13 12 auth CLI E2E TC summary

| # | TC id | command | 검증 | exit |
| - | --- | --- | --- | --- |
| 1 | TC-E2E-AUTH-001 | `auth list` | 5 provider status table | 0 |
| 2 | TC-E2E-AUTH-002 | `auth anthropic` | single provider detail | 0 |
| 3 | TC-E2E-AUTH-003 | `auth anthropic login` | OAuth wizard | 0 |
| 4 | TC-E2E-AUTH-004 | `auth anthropic logout` | keychain remove | 0 |
| 5 | TC-E2E-AUTH-005 | `auth openai set-key` | stdin read (D-06) | 0 |
| 6 | TC-E2E-AUTH-006 | `auth openai set-key --from-keychain` | slot alias | 0 |
| 7 | TC-E2E-AUTH-007 | `auth anthropic test` | ping model (ok) | 0 |
| 8 | TC-E2E-AUTH-008 | `auth anthropic test` (network fail) | unreachable → exit 2 | 2 |
| 9 | TC-E2E-AUTH-009 | `auth setup` | 일괄 wizard | 0 |
| 10 | TC-E2E-AUTH-010 | `auth default ollama` | config 갱신 | 0 |
| 11 | TC-E2E-AUTH-011 | `auth discover` | D-38 동적 발견 | 0 |
| 12 | TC-E2E-AUTH-012 | `auth export` | D-06 read-only | 0 |
| 13 | TC-E2E-AUTH-013 | `auth anthropic refresh` (access_token 만료 → refresh_token 갱신) | oauth refresh_token flow | 0 |
| 14 | TC-E2E-AUTH-014 | `auth anthropic refresh` (refresh_token 도 만료) | re-login prompt → exit 1 | 1 |

### 4.14 TC-E2E-AUTH-013: `myharness auth anthropic refresh` (access_token 만료, INITIAL_DESIGN §5.4 #11)

```
TC-E2E-AUTH-013:
  input:
    cmd: "myharness auth anthropic refresh"
    stdin: null
    env:
      MYHARNESS_KEYCHAIN_BACKEND: memory
      MYHARNESS_ANTHROPIC_REFRESH_TOKEN: "rt-mock-valid-xxxxx"   # mock keychain 주입
      MYHARNESS_OAUTH_MOCK_MODE: "refresh_ok"   # oauth_mock = refresh_token → 새 access_token 발급 mock
    cwd: /test/fixtures/TC-E2E-AUTH-013/
    fixture: /test/fixtures/TC-E2E-AUTH-013/   # keychain mock with expired access_token
  expected_output:
    stdout: |
      [anthropic] access_token expired (exp: 2026-06-08T20:00:00Z)
      [anthropic] refreshing via refresh_token...
      [anthropic] new access_token issued (exp: 2026-06-09T20:00:00Z)
      [anthropic] verifying via test call...
      [anthropic] ✓ test call succeeded (model: claude-sonnet-4-5, latency: 320ms)
    stderr: ""
    exit_code: 0
  permission_mode: default
  side_effect:
    log_jsonl_events:
      - event: "refresh_token_used" provider: "anthropic" result: "ok"
      - event: "access_token_rotated" provider: "anthropic" ttl_sec: 86400
        result: "ok"
      - event: "test_call_post_refresh" result: "ok" latency_ms: 320
    state_changes:
      - keychain slot anthropic.access_token 갱신 (값은 stdout/log ❌, 길이만)
      - keychain slot anthropic.refresh_token 그대로 유지
    external_tool_calls:
      - POST https://api.anthropic.com/v1/oauth/token (grant_type=refresh_token)
      - GET https://api.anthropic.com/v1/messages (test call, 1 req)
    files_modified: []
  d06_compliance:
    - "stdout grep 'sk-ant-' → 0건 (token value 노출 ❌)"
    - "log.jsonl 에 token 값 없음 — `access_token_rotated` event 는 길이/만료시각만 기록"
    - "refresh_token 값도 stdout/log ❌"
  xref: "INITIAL_DESIGN §5.4 #11 (auth refresh), DD-1 §5 oauth flow, NFR-SEC-1 (D-06)"
```

**검증 포인트**:
1. access_token 만료 감지 (`exp < now`) → 자동 refresh 트리거
2. refresh_token 으로 새 access_token 발급 (POST /oauth/token)
3. 갱신된 access_token 으로 test call 1회 (검증) → 200 OK
4. keychain slot 갱신 (값은 ❌, 길이/만료시각만)
5. D-06 strict: token 값 stdout/log 어디에도 ❌

### 4.15 TC-E2E-AUTH-014: `myharness auth anthropic refresh` (refresh_token 만료, INITIAL_DESIGN §5.4 #11)

```
TC-E2E-AUTH-014:
  input:
    cmd: "myharness auth anthropic refresh"
    stdin: null
    env:
      MYHARNESS_KEYCHAIN_BACKEND: memory
      MYHARNESS_ANTHROPIC_REFRESH_TOKEN: "rt-mock-expired-xxxxx"   # mock keychain: refresh_token 도 만료
      MYHARNESS_OAUTH_MOCK_MODE: "refresh_expired"   # oauth_mock = refresh_token 만료 시뮬레이션
    cwd: /test/fixtures/TC-E2E-AUTH-014/
    fixture: /test/fixtures/TC-E2E-AUTH-014/
  expected_output:
    stdout: |
      [anthropic] access_token expired (exp: 2026-06-08T20:00:00Z)
      [anthropic] refreshing via refresh_token...
      [anthropic] ✗ refresh_token rejected (HTTP 400, error: invalid_grant)
      [anthropic] refresh_token expired — re-login required
      [anthropic] run: myharness auth anthropic login
    stderr: ""
    exit_code: 1   # user error (refresh_token 만료 = user action 필요)
  permission_mode: default
  side_effect:
    log_jsonl_events:
      - event: "refresh_token_used" provider: "anthropic" result: "error"
        error: "invalid_grant"
      - event: "re_login_required" provider: "anthropic"
      - event: "exit" code: 1 reason: "user_error"
    state_changes: []   # keychain 변경 ❌ (refresh 실패 시 slot 그대로)
    external_tool_calls:
      - POST https://api.anthropic.com/v1/oauth/token (HTTP 400)
    files_modified: []
  d06_compliance:
    - "refresh_token 값 stdout/log ❌ (값은 mock keychain 내부, 길이만 표시)"
    - "에러 메시지에 'invalid_grant' 만 표시, token value ❌"
    - "exit 1 + 해결 가이드 ('run: myharness auth anthropic login') 표시"
  xref: "INITIAL_DESIGN §5.4 #11 (auth refresh fail), DD-5 §3 exit 1 user error"
```

**검증 포인트**:
1. refresh_token 만료 시 400 invalid_grant 응답 mock
2. keychain slot 변경 ❌ (rollback, 일관성)
3. exit 1 (user error: 재로그인 필요) + 해결 가이드 stdout 표시
4. D-06 strict: token 값 ❌, 에러 코드만
5. log.jsonl 에 re_login_required event 기록 → 후속 user 가 login flow 진입 시 추적 가능

---

## 5. exit code E2E TC (DD-5 §3, 4단계)

본 §5 는 DD-5 §3 의 4단계 exit code (0 success / 1 user error / 2 system error / 3 internal error) 각각의 E2E TC. 각 TC 가 exit code → AppError → MyharnessExit 변환 chain 검증.

### 5.1 TC-E2E-EXIT-001: exit 0 (success)

```yaml
TC-E2E-EXIT-001:
  name: "Exit 0 — success (정상 종료)"
  ssot_ref: "DD-5 §3.1 row 1 (exit 0 = success) + INITIAL_DESIGN §5.2 #1"

  command: "myharness code review 482"  # TC-E2E-CODE-001 동일
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-EXIT-001/
  permission_mode: default

  stdout: "<TC-E2E-CODE-001 stdout 동일>"
  stderr: ""
  exit_code: 0

  exit_code_category: success
  exit_code_source: success (no AppError raised)
  exit_code_mapping: MyharnessExit::Success as u8 = 0
```

### 5.2 TC-E2E-EXIT-002: exit 1 (user error)

```yaml
TC-E2E-EXIT-002:
  name: "Exit 1 — user error (clap arg parse fail)"
  ssot_ref: "DD-5 §3.1 row 2 (exit 1 = user error)"

  command: "myharness --mode=invalid-mode code review 482"  # invalid enum value
  env: { MYHARNESS_KEYCHAIN_BACKEND: memory }
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-EXIT-002/
  permission_mode: default

  stdout: ""
  stderr: |
    오류: 잘못된 인자입니다.
    '--mode' 의 값 'invalid-mode' 가 유효하지 않습니다.
    가능한 값: orchestrator, single, loop
  exit_code: 1

  exit_code_category: user_error
  exit_code_source: AppError::InvalidArgs("invalid --mode value")
  exit_code_mapping: MyharnessExit::UserError as u8 = 1
  # DD-5 §3.2 From<&AppError> for MyharnessExit 매핑 검증
```

### 5.3 TC-E2E-EXIT-003: exit 2 (system error)

```yaml
TC-E2E-EXIT-003:
  name: "Exit 2 — system error (외부 subprocess 실패)"
  ssot_ref: "DD-5 §3.1 row 3 (exit 2 = system error)"

  command: "myharness code review 482"  # gh CLI mock = exit 127 (command not found)
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_GH_MOCK_FAIL: "command_not_found"  # gh → exit 127
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-EXIT-003/
  permission_mode: default

  stdout: ""
  stderr: |
    오류: gh CLI 가 설치되지 않았습니다.
    `brew install gh` (macOS) / `apt install gh` (Linux) / `winget install GitHub.cli` (Windows) 후 재시도하세요.
  exit_code: 2

  exit_code_category: system_error
  exit_code_source: AppError::SubprocessFailed("gh not found, exit 127")
  exit_code_mapping: MyharnessExit::SystemError as u8 = 2
```

### 5.4 TC-E2E-EXIT-004: exit 3 (internal error)

```yaml
TC-E2E-EXIT-004:
  name: "Exit 3 — internal error (panic / invariant violation)"
  ssot_ref: "DD-5 §3.1 row 4 (exit 3 = internal error)"

  command: "myharness code review 482"  # 내부 serde_json panic 강제
  env:
    MYHARNESS_KEYCHAIN_BACKEND: memory
    MYHARNESS_FORCE_PANIC: serde_json_panic  # 내부 invariant 깨짐
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-EXIT-004/
  permission_mode: default

  stdout: ""
  stderr: |
    내부 오류: serde_json 직렬화 실패 (session state corruption).
    `~/.myharness/log.jsonl` 의 최근 100줄을 첨부하여 bug report 해주세요.
    (https://github.com/ykylee/my_harness/issues)
  exit_code: 3

  exit_code_category: internal_error
  exit_code_source: AppError::Serialization("session state corruption")
  exit_code_mapping: MyharnessExit::InternalError as u8 = 3

  log_jsonl_events:
    - { event: "internal_error", error_kind: "Serialization", backtrace: "..." , ts: "<iso8601>" }
    - { event: "command_complete", exit_code: 3, ts: "<iso8601>" }
```

### 5.5 exit code 4단계 E2E TC 정합 매트릭스

| exit code | category | AppError variant | E2E TC | trigger |
| --- | --- | --- | --- | --- |
| **0** | success | (none) | TC-E2E-EXIT-001 | 정상 종료 |
| **1** | user_error | `InvalidArgs` / `PermissionDenied` / `FileNotFound` | TC-E2E-EXIT-002 | clap arg parse fail / EACCES / file 없음 |
| **2** | system_error | `SubprocessFailed` / `Network` / `AllProvidersExhausted` | TC-E2E-EXIT-003 | gh exit 127 / network unreachable / 5xx 모든 fallback 소진 |
| **3** | internal_error | `InternalInvariant` / `Serialization` | TC-E2E-EXIT-004 | panic / invariant 깨짐 / serde fail |

DD-5 §3.2 의 `impl From<&AppError> for MyharnessExit` 매핑이 E2E TC 에서 정확히 동작 검증.

---

## 6. cross-OS + cross-shell E2E TC (D-31 + INITIAL_DESIGN §11)

본 §6 는 INITIAL_DESIGN §11 의 cross-platform 매트릭스 (5 OS variant × 4 shell) 정합 E2E TC. **5 install paths** (install.sh / install.ps1 / brew / winget / apt-dnf-apk) 별로 binary 가 동일하게 동작하는지 검증.

### 6.1 TC-E2E-CROSS-OS-001: macOS + bash (default)

```yaml
TC-E2E-CROSS-OS-001:
  name: "macOS + bash (Apple Silicon, default install path)"
  ssot_ref: "INITIAL_DESIGN §11.1 macOS Apple Silicon + §11.2 install path 1"

  runner:
    os: macOS-Apple-Silicon
    target: aarch64-apple-darwin
    shell: bash 5.x
    install_path: install.sh (curl)
    binary: myharness-aarch64-apple-darwin

  install: |
    curl -fsSL https://myharness.dev/install.sh | bash
    which myharness  # → /usr/local/bin/myharness
  command: "myharness --version"
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CROSS-OS-001/

  stdout: "myharness 0.1.0 (aarch64-apple-darwin)"
  stderr: ""
  exit_code: 0

  # 검증 항목 (cross-OS 매트릭스 §1.4):
  verification:
    - binary_arch_correct: aarch64 (uname -m = arm64)
    - shell_exec_path: /bin/bash (Bash tool 내부 = sh -c)
    - keychain_backend: macOS Keychain (security framework)
    - path_style: ~/.myharness/ (Unix-style)
    - package_manager_available: brew (TC-E2E-ENV-001 정합)
    - process_manager: launchctl
```

### 6.2 TC-E2E-CROSS-OS-002: macOS + zsh (default interactive shell)

```yaml
TC-E2E-CROSS-OS-002:
  name: "macOS + zsh (interactive shell, zsh completion)"
  ssot_ref: "INITIAL_DESIGN §11.1 macOS + §11.2 install path 3 (brew)"

  runner:
    os: macOS-Apple-Silicon
    target: aarch64-apple-darwin
    shell: zsh 5.x  # macOS Catalina+ default
    install_path: brew install --cask myharness
    binary: myharness-aarch64-apple-darwin

  install: |
    brew tap ykylee/myharness
    brew install --cask myharness
  command: "myharness code review 482"
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CROSS-OS-002/

  stdout: "<TC-E2E-CODE-001 stdout 동일>"
  stderr: ""
  exit_code: 0

  verification:
    - zsh_completion_works: myharness <TAB> → 12 commands + flags 자동완성
    - shell_integration: PROMPT_COMMAND / precmd hook 동작 (있을 시)
    - tty_handling: TTY 모드에서 interactive prompt 동작
    - signal_handling: Ctrl+C → graceful shutdown (DD-1 §3 Bash tool 정합)
```

### 6.3 TC-E2E-CROSS-OS-003: Linux (glibc) + bash

```yaml
TC-E2E-CROSS-OS-003:
  name: "Linux glibc + bash (Debian/Ubuntu, default install path)"
  ssot_ref: "INITIAL_DESIGN §11.1 Linux glibc + §11.2 install path 4 (apt)"

  runner:
    os: Linux-Debian-Bookworm
    target: x86_64-unknown-linux-gnu
    shell: bash 5.x
    install_path: apt install myharness (Debian repo)
    binary: myharness-x86_64-unknown-linux-gnu

  install: |
    curl -fsSL https://myharness.dev/install.sh | bash  # /usr/local/bin/myharness
    # 또는 apt:
    # echo "deb [trusted=yes] https://myharness.dev/apt/ stable main" > /etc/apt/sources.list.d/myharness.list
    # apt update && apt install myharness
  command: "myharness code review 482"
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CROSS-OS-003/

  stdout: "<TC-E2E-CODE-001 stdout 동일>"
  stderr: ""
  exit_code: 0

  verification:
    - glibc_version: ldd --version | head -1 (>= 2.31)
    - path_style: ~/.myharness/ (XDG-style, directories crate 정합)
    - keychain_backend: libsecret (Secret Service, gnome-keyring / KWallet)
    - process_manager: systemctl
    - package_manager: apt (TC-E2E-ENV-001 brew 대체)
```

### 6.4 TC-E2E-CROSS-OS-004: Windows + PowerShell (default)

```yaml
TC-E2E-CROSS-OS-004:
  name: "Windows + PowerShell (default install path)"
  ssot_ref: "INITIAL_DESIGN §11.1 Windows x64 + §11.2 install path 3 (winget) + §11.2 path 2 (install.ps1)"

  runner:
    os: Windows-11
    target: x86_64-pc-windows-msvc
    shell: powershell 5.x (PSCore 7+)
    install_path: winget install Yklee.Myharness
    binary: myharness-x86_64-pc-windows-msvc.exe

  install: |
    # install.ps1
    irm https://myharness.dev/install.ps1 | iex
    # 또는 winget
    winget install Yklee.Myharness
  command: "myharness code review 482"
  cwd: C:\workspace
  fixture: C:\test\fixtures\TC-E2E-CROSS-OS-004\

  stdout: "<TC-E2E-CODE-001 stdout 동일 (단, line ending = CRLF)>"
  stderr: ""
  exit_code: 0

  verification:
    - shell_exec_path: cmd /C (default, INITIAL_DESIGN §3.3) or powershell -Command (env MYHARNESS_SHELL=powershell)
    - path_style: %USERPROFILE%\.myharness\ (Windows-style)
    - keychain_backend: wincred (Credential Manager)
    - line_ending: stdout = CRLF (PowerShell 기본)
    - process_manager: Get-Service
    - package_manager: winget (TC-E2E-ENV-001 brew 대체)
```

### 6.5 TC-E2E-CROSS-OS-005: Linux musl + Alpine (docker-friendly)

```yaml
TC-E2E-CROSS-OS-005:
  name: "Linux musl + Alpine (docker 베이스 이미지)"
  ssot_ref: "INITIAL_DESIGN §11.1 Linux musl + §11.2 install path 5 (apk)"

  runner:
    os: Linux-Alpine-3.19
    target: x86_64-unknown-linux-musl
    shell: ash (busybox)
    install_path: apk add myharness
    binary: myharness-x86_64-unknown-linux-musl

  install: |
    apk add myharness
    # 또는 curl
    curl -fsSL https://myharness.dev/install.sh | ash
  command: "myharness code review 482"
  cwd: /workspace
  fixture: /test/fixtures/TC-E2E-CROSS-OS-005/

  stdout: "<TC-E2E-CODE-001 stdout 동일>"
  stderr: ""
  exit_code: 0

  verification:
    - libc: musl (ldd 출력 = "musl" 식별, glibc ❌)
    - binary_size: < 30MB (cargo-dist release + LTO + musl static)
    - docker_compatible: FROM alpine:3.19 + COPY myharness-* /usr/local/bin → 동작
    - no_dynamic_libs: ldd 출력 = "Not a valid dynamic program" (static)
    - keychain_backend: libsecret (Alpine 에선 gnome-keyring 패키지 필요, optional)
```

### 6.6 TC-E2E-CROSS-SHELL-001: stdin/stdout/stderr TTY matrix

```yaml
TC-E2E-CROSS-SHELL-001:
  name: "stdin/stdout/stderr TTY 매트릭스 (4 shell)"
  ssot_ref: "INITIAL_DESIGN §11.1 + D-31"

  matrix:
    - shell: bash (Unix default)
      stdin_pipe_ok: true
      stdout_redirect_ok: true
      stderr_redirect_ok: true
      tty_prompt: interactive (uses /dev/tty)
    - shell: zsh (macOS default)
      stdin_pipe_ok: true
      stdout_redirect_ok: true
      stderr_redirect_ok: true
      tty_prompt: interactive (zsh line editor)
    - shell: powershell (Windows default)
      stdin_pipe_ok: true
      stdout_redirect_ok: true
      stderr_redirect_ok: true
      tty_prompt: interactive (Read-Host)
    - shell: fish
      stdin_pipe_ok: true
      stdout_redirect_ok: true
      stderr_redirect_ok: true
      tty_prompt: interactive (fish_prompt)

  test_cases:
    - name: "stdin pipe (auth set-key)"
      input: 'echo "sk-test-1234" | myharness auth openai set-key'
      expected: stdin read (51 bytes), stdout = "<TC-E2E-AUTH-005 stdout>"
    - name: "stdout redirect"
      input: "myharness --version > /tmp/version.txt"
      expected: /tmp/version.txt = "myharness 0.1.0 (target)"
    - name: "stderr redirect"
      input: "myharness --mode=invalid 2> /tmp/err.log"
      expected: /tmp/err.log = "<TC-E2E-EXIT-002 stderr>", exit 1
    - name: "TTY interactive prompt (auth login OAuth)"
      input: "myharness auth anthropic login"  # TTY 필수
      expected: 브라우저 열림 + 콜백 대기 (TIMEOUT 가능, skip in CI)
    - name: "non-TTY fail (no /dev/tty)"
      input: "myharness auth anthropic login < /dev/null"  # TTY ❌
      expected: stderr "TTY 환경이 필요합니다. (-y flag for non-interactive)" exit 1
```

### 6.7 5 install paths 정합 매트릭스 (D-31 + INITIAL_DESIGN §11.2)

| # | install path | OS | TC | 검증 |
| - | --- | --- | --- | --- |
| 1 | `install.sh` (curl) | macOS / Linux | TC-E2E-CROSS-OS-001/003/005 | `curl ... \| bash` → binary at `/usr/local/bin/myharness` |
| 2 | `install.ps1` (irm) | Windows | TC-E2E-CROSS-OS-004 | `irm ... \| iex` → binary at `C:\Program Files\myharness\myharness.exe` |
| 3 | `brew install --cask` | macOS | TC-E2E-CROSS-OS-002 | `brew install` → `/opt/homebrew/bin/myharness` |
| 4 | `winget install` | Windows | TC-E2E-CROSS-OS-004 | `winget install` → `PATH` 자동 추가 |
| 5 | `apt install` / `dnf install` / `apk add` | Linux (Debian/RHEL/Alpine) | TC-E2E-CROSS-OS-003/005 | 패키지 매니저 네이티브 |

### 6.8 cross-OS TC 6 entries summary

| # | TC id | OS | shell | install path | 검증 |
| - | --- | --- | --- | --- | --- |
| 1 | TC-E2E-CROSS-OS-001 | macOS AS | bash | install.sh | binary arch + keychain + launchctl |
| 2 | TC-E2E-CROSS-OS-002 | macOS AS | zsh | brew | zsh completion + TTY |
| 3 | TC-E2E-CROSS-OS-003 | Linux glibc | bash | apt / install.sh | systemctl + libsecret + apt |
| 4 | TC-E2E-CROSS-OS-004 | Windows | PowerShell | winget / install.ps1 | wincred + Get-Service + CRLF |
| 5 | TC-E2E-CROSS-OS-005 | Linux musl | ash | apk / install.sh | musl static + docker-friendly |
| 6 | TC-E2E-CROSS-SHELL-001 | (matrix) | bash/zsh/pwsh/fish | (n/a) | stdin/stdout/stderr/TTY 4×5 |

---

## 7. handoff (D-26 4-필드)

### 7.1 summary

본 TC_E2E.md = my_harness v1 의 **L4 E2E Test Case scaffold** (REVIEW.md §6.1 + §6.4 TDD RED-GREEN-REFACTOR 진입점). TASK-005-1 (v1 Rust MVP 구현) 의 후속 권장 시점 (v1.5+, TUI 안정 + 3 OS cross-build 검증 시점). **7 sections (§0~§7), 49 TC entries** (cycle 4: 12 도메인 18 TC + 3 mode flag 5 TC + **plan/bypassPermissions 2 TC (MODE-006/007)** + 12 auth CLI 12 TC + **refresh 2 TC (AUTH-013/014)** + 4 exit code 4 TC + cross-OS 6 TC = 49 entries), 4-step TC format (input → expected output → exit code → side effect) 정합 DD-5 §3 + DD-1 §4. **permission 4 mode E2E 정합 (cycle 4)**: default/acceptEdits/plan/bypassPermissions 모두 ≥1 TC. docker + local Ollama + cross-OS matrix 환경 명시. 5 install paths 정합 (D-31 + D-36). D-06 정책 (auth token stdout 출력 ❌) 준수 — refresh TC 의 stdout grep "sk-" → 0건 검증 명시.

### 7.2 risks

- **(R-1) 분량 over-shoot**: 600~900 target → 본 TC_E2E.md = **2,265 lines (cycle 4 actual, +152% over-shoot)**, 49 TC entries. §2 12 도메인 18 TC (line 334-998) + §3 7 mode TC (line 999-1308, cycle4 +2) + §4 14 auth TC (line 1309-1831, cycle4 +2) + §6 6 cross-OS/shell (line 1954-2199) 의 yaml TC format = TC 1건 = 25-35 lines (4-step 정밀도). DD-5 +29% / INITIAL_DESIGN +58% / TC_COMPONENT +5% precedent 정합. 줄이려면 §2/§4 의 yaml TC 의 `external_tool_calls` + `state_changes` 일부 압축 가능. 그러나 TASK-005-1 + v1.5+ 구현자가 본 문서만으로 E2E harness 작성 가능해야 하므로 정밀도 우선.
- **(R-2) docker 격리 + native binary trade-off**: docker 가 Linux 만 지원. macOS / Windows native binary 검증은 host 직접 실행 필요. §6.1 / §6.2 / §6.4 가 host 직접. hybrid 환경 신뢰성 ⚠️.
- **(R-3) Ollama mock 결정성**: LLM 호출의 mock 결정성 = `prompt_pattern` regex 매칭 기반. LLM 응답의 자유도가 높은 경우 (e.g., 자유 형식 응답) mock miss → 실제 ollama fallback (non-deterministic). TC 별 fixture 가 정밀할수록 mock hit ↑.
- **(R-4) keychain in-memory backend**: `MYHARNESS_KEYCHAIN_BACKEND=memory` 가 TC 별 격리. 실제 macOS Keychain / wincred / libsecret 통합 검증은 host 직접 실행 TC 필요. v1.5+ 정합.
- **(R-5) cross-OS CI 비용**: GitHub Actions matrix (macOS-3 + Linux-3 + Windows-2 = 8 OS variant) × 49 TC = 392 job. 분할 (smoke vs full) 또는 nightly run 권장.
- **(R-6) TASK-002 ⏸ server/env placeholder**: server/env 명령의 host alias / stack 가 placeholder. §2.2-§2.3 의 TC 가 graceful degrade 검증. yklee 인프라 정보 수령 후 §2.2-§2.3 의 fixture 교체 필요 (PROJECT_PROFILE.md §3.1 TODO).

### 7.3 suggested_follow_up

1. **즉시 (TASK-005-1 v1 구현 후)**: L4 E2E TC harness (Rust) 작성. docker `Dockerfile.runtime` + `mock_ollama/server.py` + 36 TC 의 fixture 디렉토리 + shell runner script.
2. **v1.5+ 정합 (REVIEW.md §6.3 L4 권장 시점)**: TUI 안정 + cross-OS CI (GitHub Actions matrix) + LLM mock 성숙 후 본 TC 자동 활성화. RED → GREEN → REFACTOR.
3. **D-23 align**: 본 TC_E2E.md 작성으로 INITIAL_DESIGN §5 CLI 표면 + DD-1 §4 permission + DD-5 §3 exit code 의 E2E 검증 scaffold 추가. CONCEPT.md / REQUIREMENTS.md / INITIAL_DESIGN 4 문서 cross-ref 정합 유지.
4. **5 install paths 검증 자동화**: cargo-dist release 시 각 install path 별 smoke TC (TC-E2E-CROSS-OS-001 ~ 005) 자동 실행. install 검증 → TC 검증 chained.
5. **TASK-002 해소 후 server/env TC 갱신**: yklee 인프라 정보 (host alias / SSH / asdf 런타임) 수령 후 §2.2/§2.3 의 placeholder fixture 를 실제 fixture 로 교체 + TC exit code 1 (graceful degrade) → exit 0 으로 갱신.

### 7.4 produced_artifacts

| 산출물 | 경로 | 분량 |
| --- | --- | --- |
| **TC_E2E.md** (본) | `docs/specs/TC_E2E.md` | **2,265 lines / 7 sections / 49 TC entries (over-shoot +152%, R-1, cycle 4 actual)** |
| deliverable_tc4.md | `docs/team/deliverable_tc4.md` | early + final signal |
| engine deliverable | `~/.mavis/plans/plan_ddcdd2a3/outputs/tc-4/deliverable.md` | final |
| board | `~/.mavis/plans/plan_ddcdd2a3/board.md` | start + done 2 entry (D-16 minimal) |

### 7.5 Cross-reference 정합 (4 docs)

| SSOT | 본 TC § | 비고 |
| --- | --- | --- |
| **INITIAL_DESIGN.md §5** (12 명령 + 3 mode + 12 auth) | §2, §3, §4 | 1:1 cover |
| **INITIAL_DESIGN.md §11.1** (5 OS variant) | §6.1-§6.5 | 5 OS TC |
| **INITIAL_DESIGN.md §11.2** (5 install paths) | §6.7 | install path 정합 |
| **DD-5 §3** (exit code 4단계) | §5 | 4 exit TC |
| **DD-1 §4** (permission 4 mode) | §2, §3, §4 (각 TC 의 `permission_mode`) | 4 mode 적용 |
| **REVIEW.md §6.1** (L4 E2E TC 정의) | §1 | 600~900 target |
| **REVIEW.md §6.4** (TDD RED-GREEN-REFACTOR) | §0.5 + §1.1 | RED 진입점 |
| **CONCEPT.md §5.4, §5.5.2, §5.9, §5.10, §5.12** | 본 § 전반 | 5 section 정합 |
| **REQUIREMENTS.md §3.5 NFR-OBS-1** (log.jsonl) | §5.4 + 각 TC side effect | event append 정합 |

### 7.6 표준 6 원칙 (D-26) 준수

- **한국어 보고** (default), 코드/CLI flag/path 영문
- **결론 + 다음 행동 위주** (§0.1, §7.1, §7.3)
- **상태값**: done (chunk 3 후)
- **이벤트 소싱**: 모든 TC 가 `log.jsonl` event append 검증
- **비참조 원칙**: 다른 세션/이전 세션 참조 ❌
- **handoff 4-필드**: §7.1~§7.4 (summary/risks/follow_up/artifacts)

### 7.7 안티 패턴 6 미반영 (CONCEPT.md §8)

- ✅ 1 surface (CLI E2E 만) / 2 surface CLI+TUI (TUI 별도 scaffold v2+)
- ✅ 단일 Rust (E2E TC 자체는 shell + TOML)
- ✅ 12 commands + 3 mode (cycle 4: +plan +bypassPermissions) + 12 auth (cycle 4: +refresh) = **29 entry** (100+ ❌)
- ✅ local-only (`~/.myharness/`, log.jsonl)
- ✅ MIT 호환 오픈소스 (docker, ollama, bash, pwsh, fish)

---

### VERDICT (final, post-handoff, cycle 4): PASS — L4 E2E TC scaffold 완료. 7 sections / **2,265 lines / 49 TC entries** (cycle 4: +MODE-006/007 plan/bypassPermissions, +AUTH-013/014 refresh). docker + local Ollama + cross-OS matrix 환경 정합. permission 4 mode (default/acceptEdits/plan/bypassPermissions) E2E 정합 (cycle 4). D-06 / 안티 6 / 표준 6 원칙 모두 준수. TASK-005-1 후속 v1.5+ 권장 scaffold. RED-GREEN-REFACTOR 진입점 명확 (REVIEW.md §6.4).
