# myharness (v1 산출물)

> **이 디렉토리는 my_harness 의 v1 산출물 source tree 입니다.** (CONCEPT.md §11.3, TASK-005-1)
> 저장소 root 의 README.md / docs/ / ai-workflow/ 는 my_harness 의 **개발 workflow** (D-25: Mavis zero coupling).

## 상태

- v0.1.0 — TASK-005-1 W2 (workspace init)
- v1.0 — TASK-005-1 W11 (target)

## 구조

```
myharness/                       # 이 디렉토리
├── Cargo.toml                   # workspace root
├── rust-toolchain.toml
├── crates/
│   ├── core/                    # Harness 5 components 공통
│   ├── llm/                     # rig-core + provider registry
│   ├── tui/                     # ratatui + crossterm
│   ├── tools/                   # Read/Write/Edit/Bash/Grep/Glob
│   ├── context/                 # CLAUDE.md + memory + /compact
│   ├── cli/                     # binary entry (myharness 명령)
│   ├── auth/                    # keyring + per-provider state
│   └── compression/             # built-in 압축 (Layer 1 + Layer 2 stub)
└── README.md
```

## 빌드 / 실행

```bash
cd myharness
cargo build --release
./target/release/myharness --version
./target/release/myharness --mode=orchestrator
```

## Cross-platform 빌드 (cargo-dist)

```bash
# 향후 W11 에서 설정. 현재는 native cargo 만.
cargo install cargo-dist  # or cargo-binstall cargo-dist
cargo dist init
cargo dist build --targets=aarch64-apple-darwin,x86_64-unknown-linux-gnu
```

## 결정 (CONCEPT.md cross-ref)

- D-36: Rust 1안
- D-37: headroom v1 = 3 알고리즘
- D-38: provider-auto-config skill
- D-41: 환경 검증 완료
- D-42: config 포맷 TOML
