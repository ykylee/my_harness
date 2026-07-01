# Operation Log — 2026-07-01 ENXIO Investigation

- **목적**: yklee 가 `./target/release/myharness` 실행 후 프롬프트 입력 직전 앱이 종료되는 현상 추적
- **환경**: cwd = `/home/yklee/repos/my_harness`, sandbox `workspace-write`, 비-TTY exec (PTY 미할당)
- **바이너리**: `myharness/target/release/myharness` (v0.1.0, W13)
- **main**: `5e39f5e` (D-105 Edit v2 line_anchored mode)
- **조사자**: opencode orchestrator
- **상태**: ✅ 원인 확정 + 회피 경로 확인. **버그 아님, 의도된 가드**.

---

## 1. 증상 (yklee 보고)

```text
$ ./target/release/myharness
…  (정상 startup log)
Error: No such device or address (os error 6)
$    ← 즉시 종료, 프롬프트 입력 불가
```

`Error: No such device or address (os error 6)` 는 `errno = ENXIO` (Linux: "No such device or address").
`os error 6` = ENXIO. raw backtrace 없음 (`<unknown>` 9 frame, `__libc_start_main` 직전 종료).

## 2. 1차 가설

`crates/llm/src/auth_keyring.rs` 의 keyring probe 또는 `myharness-auth` crate 의 `keyring` 호출이
libsecret 부재 / D-Bus session bus 부재 환경에서 ENXIO 로 panic 추측.

→ 1차 조사 결과: **기각**.

- `crates/auth/Cargo.toml:10` `keyring = { workspace = true }` dep 만 있고
  `crates/auth/src/**/*.rs` 어디에도 `use keyring::*` / `Entry::new` 호출이 **하나도 없음**.
  - store.rs (TokenStore) = file-only (`~/.myharness/oauth/{provider}.toml`, chmod 600)
  - `keyring` crate 은 dead dep (cleanup 후보 — D-75 batch follow-up 수준, 본 이슈와 무관)
- `crates/llm/src/auth_keyring.rs` 의 `detect_backend()` 는 `DBUS_SESSION_BUS_ADDRESS` /
  `XDG_RUNTIME_DIR` 미설정 시 `KeyringBackend::None` 으로 떨어지며 in-memory cache + env hint
  fallback 으로 정상 처리. `Error::NoEntry` 등 keyring crate error variant 매칭 없음.
- `discover.rs:50` `KeyringAuthStore::probe().list().await.unwrap_or_default()` 만 호출
  — 이 경로도 안전.

## 3. 2차 가설 + 확정

`crates/tui/src/events.rs:23` `TtyGuard::enter()` 가 의심됨. 호출 사이트 추적:

```bash
$ grep -rn "TtyGuard::enter" crates/
crates/cli/src/main.rs:280:                let tty = TtyGuard::enter()?;
crates/cli/src/main.rs:295:                drop(tty);
```

`main.rs:279~280`:

```rust
"orchestrator" | "single" => {
    let tty = TtyGuard::enter()?;
    let mut app = App::new("myharness", mode);
    …
}
```

`TtyGuard::enter()` 본체 (`crates/tui/src/events.rs:18~32`):

```rust
pub fn enter() -> Result<Self> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;                  // ← 비-TTY 에서 ENXIO
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Self { stdout, active: true })
}
```

`crossterm::terminal::enable_raw_mode()` 는 stdout fd 가 TTY 가 아닐 때 Linux kernel 의
`ioctl(TCSETS, ...)` 가 `ENXIO` 로 실패. 백트레이스가 `<unknown>` 인 이유는
enable_raw_mode() 이 `crossterm` → `nix` → `libc::ioctl` 단일 호출에서 즉시 abort 되기 때문.

`state.json.current_baseline` (D-125, 2026-07-01) 이 이 경로를 **의도된 비-TTY 가드**로 명시:

> "비-TTY ✅ TtyGuard ENXIO 정상 실패 (`unknown mode` 사라짐)"

→ **버그가 아니라 의도된 fail-fast 가드**. TUI 모드는 진짜 TTY 가 있을 때만 동작.

## 4. 비대화형 경로 검증

TtyGuard 호출이 TUI 분기 (L279 `"orchestrator" | "single" =>`) 안에만 있고
`ask` / `auth` / `code` / `env` / `git` / `task` / `handoff` 분기에는 없음을
코드 + 실측으로 확인.

| 실행 | 결과 |
| --- | --- |
| `./target/release/myharness --version` | `myharness 0.1.0` ✅ |
| `./target/release/myharness --help` | subcommand 7개 정상 출력 ✅ |
| `./target/release/myharness ask "ping"` | `MiniMax OAuth token` log → `[Code Reviewer] Reviewing: ping` → `[LLM-error] error sending request for url` (sandbox network 차단, ENXIO 아님) ✅ TtyGuard 발화 안 함 확인 |
| `./target/release/myharness auth status` (인자 부족) | clap 사용법 에러 정상 출력 ✅ |
| `./target/release/myharness --mode=orchestrator` | `Error: No such device or address (os error 6)` ❌ TtyGuard 발화 |
| `RUST_BACKTRACE=1 ./target/release/myharness --mode=orchestrator` | 동일 에러 + stack backtrace `<unknown>` 9 frame ❌ |

## 5. 결론

1. **ENXIO 의 출처**: `TtyGuard::enter()` → `crossterm::enable_raw_mode()` → `ioctl(TCSETS)` 실패.
2. **성격**: 의도된 비-TTY 가드 (D-125 회귀복구에서 정상 동작으로 분류).
3. **영향 범위**: `--mode=orchestrator` / `--mode=single` (TUI 진입) 한정.
4. **비대화형 subcommand 무관**: `ask`, `auth`, `code`, `env`, `git`, `task`, `handoff` 는 TtyGuard 미호출 → 정상 동작.

## 6. 회피 / 해결

### 6.1 TTY 환경에서 실행 (권장)

```bash
# 일반 터미널 (macOS Terminal, iTerm, GNOME Terminal, Windows Terminal 등)
$ ./target/release/myharness

# TTY 강제 할당 (CI / script / ssh 비대화형)
$ script -qfc "./target/release/myharness" /dev/null

# tmux / screen 내부
$ tmux new -s harness
$ ./target/release/myharness
```

### 6.2 비대화형 subcommand 사용 (TTY 불필요)

```bash
$ myharness ask "질문"                       # W15.a OAuth resolve → MiniMax API
$ myharness auth status <provider>           # OAuth 토큰 상태
$ myharness auth login --provider minimax    # OAuth 로그인 (TTY 필요할 수 있음)
$ myharness code review <target>             # code review
$ myharness code implement <feature>         # 구현
$ myharness task start --id ...              # 태스크 시작
```

### 6.3 IDE/통합 터미널 / Docker / ssh non-tty 사용 시

해당 환경이 PTY 를 안 잡아주는 경우:
- VSCode 통합 터미널: `terminal.integrated.gpuAcceleration` / shell profile 확인
- `docker exec -it <container> myharness` 로 `-it` (TTY) 명시
- `ssh -t user@host myharness` 로 `-t` (force tty) 명시

## 7. 부가 발견 (cleanup 후보, 본 이슈와 무관)

- `crates/auth/Cargo.toml:10` `keyring = { workspace = true }` 가 dead dep.
  `crates/auth/src/**/*.rs` 어디에서도 `use keyring::*` 없음. `cargo machete` / `cargo udeps` 로 확인 가능.
  - 정리 가능 commit: dead dep 제거, Cargo.lock 정리.
  - 본 작업과 분리해서 처리 권장 (D-126 또는 다음 안정화 batch).

## 8. 후속 작업 제안 (yklee 결정)

1. **(A) 현상 유지** — TUI 가드는 의도된 동작이므로 그대로 두기. yklee 가 정상 TTY 에서
   실행 시 정상 동작. docs 보강만.
2. **(B) UX 개선** — ENXIO 시 친절한 메시지 출력 후 종료
   (예: "error: TTY required for interactive mode. Run from a real terminal, or use a
   non-interactive subcommand (`myharness ask ...`). Exit 2."). 본 작업은 단순
   `Result` 매핑으로 끝나며 회귀 위험 낮음.
3. **(C) 비대화형 fallback** — `--mode=orchestrator` 가 비-TTY 감지 시 자동으로 비대화형
   `loop` 모드 또는 비대화형 `ask` 로 라우팅. 정책 결정 필요 (TUI UX 가드가 부주의하게
   무력화될 위험).

기본 권장: **(A) + docs 1줄 보강** ("TUI modes require a real TTY. In CI/pipe, use
`myharness ask ...`.").

## 9. 결정 (D-126 후보, yklee 승인 대기)

- D-126 = ENXIO 가드 UX 개선 (B안) + auth crate keyring dead dep 제거.
  - (a) `crates/tui/src/events.rs` `TtyGuard::enter()` 반환 에러를 그대로 살리되
    `crates/cli/src/main.rs:280` 의 `?` 직전 친절한 context 추가:
    ```rust
    let tty = match TtyGuard::enter() {
        Ok(t) => t,
        Err(e) if !std::io::IsTerminal::is_terminal(&std::io::stdout()) => {
            anyhow::bail!(
                "TUI mode ({mode}) requires a real TTY. \
                 Run from a terminal, or use a non-interactive subcommand like `myharness ask ...`. ({e})"
            );
        }
        Err(e) => return Err(e.into()),
    };
    ```
  - (b) `crates/auth/Cargo.toml` `keyring` dep 제거 + `cargo update`.
  - (c) `state.json` / `session_handoff.md` / 본 로그 동기화.
  - 1 commit + 단일 push (코드 + 메모리).

## 10. 메모

- 본 로그는 `state.json.current_baseline` 의 D-125 baseline 위에서 작성.
- D-125 자체는 비대화형 subcommand + `ask "ping"` + TtyGuard ENXIO 의 **expected failure**
  를 함께 verify 한 회귀복구 커밋. 본 로그는 그 중 TtyGuard ENXIO 한 가지를 더 깊게
  추적한 operation log.

---

## 11. D-126 결과 (2026-07-01) — b안 적용

### 11.1 적용 결과
yklee 결정 = b안 (UX 개선). 적용된 패치:

- `crates/cli/src/main.rs:6` `use std::io::IsTerminal;` top-level import 추가
- `crates/cli/src/main.rs:280~293` `TtyGuard::enter()?` → `match` 로 비-TTY (stdin or stdout non-TTY) 시
  `anyhow::bail!` 분기
- 친절한 메시지: `"TUI mode ({mode}) requires a real TTY. Run from a terminal (Terminal, iTerm, tmux, ...), or use a non-interactive subcommand like \`myharness ask \"...\"\`. ({e})"`
- D-83 pedantic batch 의 `useless_conversion` clippy 회피: `Err(e) => return Err(e.into())` → `Err(e) => return Err(e)` (TtyGuard::enter() 가 이미 anyhow::Error 반환)

### 11.2 3-way verify (실측)

| verify | 결과 |
| --- | --- |
| cargo build -p myharness | ✅ |
| cargo build --release -p myharness | ✅ (16.13s) |
| cargo clippy -p myharness --all-targets -- -D warnings | ✅ 0 warning |
| cargo test -p myharness | 8 pass / 1 fail (sandbox TcpListener e2e, D-126 무관) |
| `--version` | ✅ `myharness 0.1.0` 회귀 0 |
| `ask "ping"` | ✅ TtyGuard 발화 안 함, MiniMax token load, code reviewer 시작 (network 단계에서 sandbox 차단) |
| `--mode=orchestrator` (비-TTY) | ✅ `Error: TUI mode (orchestrator) requires a real TTY. Run from a terminal (Terminal, iTerm, tmux, ...), or use a non-interactive subcommand like \`myharness ask "..."\`. (No such device or address (os error 6))` + exit 1 |
| `--mode=single` (비-TTY) | ✅ 동일 패턴, `single` 모드명 반영 |
| cargo test --workspace --lib | 11 fail = 모두 `myharness-auth` mock-server + xdg-open 부재. D-126 무관 (sandbox 환경 회귀) |

### 11.3 무관 회귀 (D-126 PR 범위 밖)

- `cargo test --workspace --lib` 11 fail = `myharness-auth` mock-server e2e (`TcpListener::bind` PermissionDenied) + xdg-open 부재 (`/usr/bin/xdg-open: no method available for opening 'about:blank'`). 본 PR (main.rs 1 file) 변경과 무관. D-125 baseline (516 pass) 대비 sandbox 환경 변화.
- 잔여: `crates/auth/Cargo.toml:10` `keyring` dead dep. 본 로그 §7. D-128 후보.

### 11.4 메모리 동기화 (단일 push)

- `ai-workflow/memory/state.json` — `current_baseline` / `current_focus` / `current_axis` / `in_progress_items` (D-126 항목) / `generated_at` 갱신
- `ai-workflow/memory/session_handoff.md` — Updated line 에 D-126 mention 추가
- `ai-workflow/memory/work_backlog.md` — 첫줄 metadata 갱신
- `ai-workflow/memory/backlog/2026-07-01.md` — §3 D-126 섹션 추가
- `ai-workflow/memory/logs/2026-07-01-ENXIO-investigation.md` — 본 §11 결과 추가
- 결정 ID: **D-126** (73 → 74)

### 11.5 다음 (yklee 결정)

- (D-127 후보) workspace mock-server e2e 의 sandbox 환경 격리 (`#[ignore]` 또는 CI 전용 marker)
- (D-128 후보) `crates/auth/Cargo.toml` keyring dead dep 정리
- (a-j) D-106+ tree-sitter / pure insert/delete / Anthropic / D-100 한계 / TASK-002 / OAuth real flow / TUI shell / cargo hygiene / Lark / block-aware

