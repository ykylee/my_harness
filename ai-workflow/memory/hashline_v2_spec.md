# Hashline v2 spec — D-104 (2026-07-01)

> oh-my-pi (`@oh-my-pi/hashline` v15.11.0, can1357/oh-my-pi, MIT) 기반 점진 차용 1차 cycle.
> v1.5+ 차용 1순위. 출처: `/Users/yklee/.bun/install/cache/@oh-my-pi/hashline@15.11.0@@@1/`

## 1. 왜 Hashline 인가 (문제)

**현재 edit.rs (D-100~D-103, v1)**:
```json
{"old_string": "...", "new_string": "...", "replace_all": false}
```

**한계**:
- LLM 이 old_string 을 정확히 복사해야 함 (typo 1자 → fail)
- 1000+ 라인 파일을 여러 번 edit 하면 "어디가 line N 인지" 알 수 없음 (Read 가 raw text 반환)
- 동시 다발 edit 의 atomicity 보장은 단순 replace_all 로 부족
- stale anchor 감지 불가 — 누가 그 사이 파일을 바꿨는지 모름

**Hashline 의 답**:
- **LINE:TEXT prefix** — Read 가 `1:fn main()\n2:...\n` 으로 반환 → LLM 이 "line N" 으로 anchor 가능
- **Content hash tag** — full-file 4-hex fingerprint → stale anchor 자동 reject
- **Tight range** — replace/delete 는 "원본 line N..M 만 변경" 의미론 → keeper line 손실 위험 0
- **Insert at anchor** — pure addition 도 anchored (file 중간 임의 위치 추가 가능)

## 2. v1.5+ 차용 범위 결정 (현재 v1 → 점진)

| 기능 | oh-my-pi | my_harness v1 | 비고 |
|---|---|---|---|
| `LINE:TEXT` Read format | ✅ | ❌ | **D-104 채택** |
| Content hash tag (4-hex xxhash32) | ✅ | ❌ | **D-104 채택** |
| `replace N..M` (concrete lines) | ✅ | ❌ | D-105 예정 |
| `replace block N` (tree-sitter) | ✅ | ❌ | ❌ v1.5+ 별도 (tree-sitter rust crate 도입 검토) |
| `delete N..M` / `insert before/after` | ✅ | ❌ | D-105 (replace 만 먼저) |
| Multi-section patch parser | ✅ Lark | ❌ | ❌ v1.5+ |
| `InMemoryFilesystem` / `NodeFilesystem` abstraction | ✅ | ❌ | ❌ v1 은 tokio::fs 그대로 (overkill 회피) |
| `SnapshotStore` / 3-way merge recovery | ✅ | ❌ | ❌ v2 (session memory 와 결합 시점) |
| `replace block` 의 tree-sitter resolution | ✅ | ❌ | ❌ v2 (Pi 의 차별점 1) |

**D-104 scope (이번)**: Read v2 + content_hash 모듈.
**D-105 (차기)**: Edit v2 — `line_anchored` mode (replace N..M + hash check).
**D-106+**: pure insert / delete / tree-sitter 도입 검토.

## 3. Content hash 결정

### 3.1. Algorithm
**xxHash32** (oh-my-pi 와 1:1). 16-bit truncated = 4 hex chars uppercase.

### 3.2. Dep 추가
- **transitive 이미 있음**: `twox-hash 2.1.2` (Cargo.lock).
- **direct 추가**: `myharness/crates/tools/Cargo.toml` 에 `xxhash-rust = { version = "0.8", features = ["xxhash32"] }` 추가 검토.
- **fallback 결정**: simplest path = `sha2` (이미 workspace dep) SHA256 truncate 16 bit. **충돌 확률**: 16-bit = 65536 values, session scope (단일 파일 ≤ 1MB) 에서는 사실상 충돌 없음. Hashline spec 1:1 보다는 real-world 충분. → **D-104 = sha2 truncate**, xxhash 도입은 D-105+ 에서 (필요 시).

### 3.3. Normalization
oh-my-pi 와 동일:
- 각 line 의 trailing `[ \t\r]` trim (`/[ \t\r]+(?=\n|$)/g`)
- file 끝 line 의 trailing 도 trim
- BOM / line ending (CRLF) 정합은 Read 측에서 별도 처리 (D-104 는 LF 만, CRLF 는 v1.5+)

## 4. Read v2 spec (D-104)

### 4.1. Input schema (back-compat 100%)
```json
{
  "file_path": "src/main.rs",
  "offset": 0,
  "limit": 200
}
```
모든 필드 optional (이전과 동일).

### 4.2. Output format (default = `line_text`)
```
1:fn main() {
2:    println!("hello");
3:}
```
- `LINE:TEXT` (1-indexed, `HL_LINE_BODY_SEP = ":"`)
- offset 적용 시 line 번호는 original file 기준 (skip 한 줄도 count)
- 빈 줄 = `5:` (TEXT 없음)
- metadata JSON:
  ```json
  {
    "path": "/abs/path",
    "size": 1234,
    "line_count": 100,
    "format": "line_text",
    "content_hash": "A1B2",
    "start_line": 1,
    "end_line": 200
  }
  ```

### 4.3. Backward compat
- 기존 테스트 (full Read) 가 영향받지 않도록 **default = line_text**, opt-out `format: "raw"` 가능 (D-105 에서 추가, D-104 는 line_text 만 있어도 회귀 0).

### 4.4. LLM prompt 영향
- D-103 prompt 의 "Read description" 에 라인번호 anchor + content_hash 사용 가능 추가 (D-105 의 Edit v2 와 함께 — D-104 는 Read format 만 emit, prompt 는 미변경).

## 5. Edit v2 spec (D-105 예정 — 본 문서만 정의, 구현은 차기)

### 5.1. New input mode
```json
{
  "file_path": "src/main.rs",
  "line_anchored": {
    "start_line": 5,
    "end_line": 7,
    "expected_hash": "A1B2",
    "replacement": "fn main() {\n    println!(\"hello, {name}\");\n}"
  }
}
```

### 5.2. Validation 순서
1. Read file → current_content
2. Compute `current_hash` = `compute_file_hash(current_content)`
3. If `current_hash != expected_hash` → `Err("stale anchor: file modified; re-read with `Read` tool")`
4. Verify lines 5..=7 exist (content.lines().count() >= 7)
5. Replace: lines[5..=7] を `replacement` 으로 swap
6. Write file
7. Return `ToolResult { output, metadata: {replaced_lines: 3, new_hash: "<재계산>"} }`

### 5.3. Backward compat
- 기존 `old_string` / `new_string` / `replace_all` mode 보존 (default 가 line_anchored 으로 바뀌지 않음 — explicit opt-in).

### 5.4. Anti-patterns (D-105 prompt 에 포함)
- tight range: range 는 "실제 변경되는 line" 만 cover
- keeper line 을 body 에 재입력 ❌
- tree-sitter 도입 전이므로 `replace block` 미지원 (LLM 이 structure 모름 → 미안하지만 concrete range 만)

## 6. Future scope (D-106+)

### 6.1. Tree-sitter 도입 (v1.5+)
- 의존성: `tree-sitter 0.25` + `tree-sitter-rust 0.23` (+ language pack 필요 시)
- 장점: `replace block N` 으로 function/class/impl 전체 rewrite 가능
- 단점: dep weight ↑, build time ↑, 다른 언어 (JS/Python/Go) 도 pack 필요
- 결정: **v1.5+ 별도 cycle** — D-106 또는 그 이후

### 6.2. Pure insert/delete (D-105 의 replace 완료 후)
- `insert head/tail/before N/after N` — multi-section atomic 보장 (단일 patch parse)
- grammar.lark 직접 도입은 ❌ — 단순 manual parser (Rust 의 `nom` 또는 split-and-validate)

### 6.3. Snapshot store + 3-way merge recovery
- session 시작 시 read 한 file 들의 hash table 보관
- 동시 edit 시 stale hash → 3-way merge 시도 (common ancestor + current + new)
- **scope**: v2 (Plugin Sub-task 4 시점, hindsight memory 와 결합 검토)

### 6.4. Multi-section batch
- 단일 Edit 호출로 N개 file 동시 edit (LLM 이 "이 PR 의 모든 변경" 한번에 emit 가능)
- Patcher::apply 의 preflight (rollback on partial fail) 도 도입 검토

## 7. 위험 + 회피

| 위험 | 영향 | 회피 |
|---|---|---|
| Read format 변경 → 기존 test 회귀 | medium | back-compat `format: "raw"` (D-105), D-104 는 default 만 line_text 이고 새 test 만 line_text assert |
| xxhash vs sha2 다른 hash → Edit anchor 깨짐 | low | D-104 는 Read format 만, Edit v2 는 D-105 → D-105 에서 algorithm 단일화 결정 |
| LLM 이 LINE:TEXT 를 통째로 re-emit | low | Read 가 emit 한 메타 그대로 사용 (LLM 학습 패턴) — dedup D-102 가 자연 보호 |
| 큰 file 의 LINE:TEXT 도 여전히 큼 | low | D-103 의 chunked Read 와 합동 — chunk 내 line 번호 absolute, offset+limit 와 정합 |

## 8. 검증 기준 (D-104)

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warning
- [ ] `cargo test --workspace --lib` 전체 회귀 0 (D-103 baseline 68/68 tui + 기타 crate)
- [ ] 신규 tools test **≥ 6 PASS** (content_hash 4 + read_line_text 2-3, 추가는 lib.rs + schema 변경 시 더)
- [ ] Real LLM dry-run (이번 cycle 은 생략 가능 — Edit v2 가 D-105 이므로)

## 9. 다음 옵션 (D-104 완료 후)

1. **D-105 Edit v2 line_anchored mode** (이 spec 5. 절 구현) ← 추천
2. **TUI shell + interactive mode 검증** (binary `myharness` 실행, LoopRunner 통합)
3. **TASK-002 도메인 명령** (yklee 인프라 정보 의존 — homelab NAS / Gitea / GitHub)
4. **A-proper native tool calling** (OpenAI/Anthropic wire format, v1.5+)

---

**Refs**: D-100 (방향 전환 + 점진 차용 선언) / D-103 (large file chunked Read — hash anchor 의 전제) / D-105+ (Edit v2 구현 예정) / §5 CONCEPT.md v1 (3-모드 orchestrator/single/loop) / §5.10-§5.11 (Tool dispatch + prompt spec)

**Refs**: `@oh-my-pi/hashline@15.11.0` / `src/format.ts` (computeFileHash) / `src/prefixes.ts` (LINE:TEXT strip) / `src/prompt.md` (사용자 prompt spec)
