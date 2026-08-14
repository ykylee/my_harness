# OVERLAY_IMPLEMENTATION_PLAN — D-135 구현 로드맵 / 마일스톤 / WBS

- 문서 목적: overlay 재구성의 **실행 계획**. 설계([DETAILED_DESIGN_OVERLAY.md](./DETAILED_DESIGN_OVERLAY.md))를 마일스톤과 WBS 로 내린다.
- 범위: PR-0 이후 구현. v0 crate 신규 기능 금지. grok 포크 금지.
- 상태: active (D-136, 2026-08-14)
- 관련: [CONCEPT.md](../CONCEPT.md) §6 · [DETAILED_DESIGN_OVERLAY.md](./DETAILED_DESIGN_OVERLAY.md) §12

---

## 0. 한 줄

엔진은 이미 있다. 우리가 만드는 것은 **plugin 한 개 + 얇은 래퍼 + MiniMax 연결 + 3-도메인 가드** 뿐이다.

의존 방향 (왼쪽이 먼저):

```
M0 문서 ✅ → M1 실행 가능 → M2 도메인 가치 → M3 경화 → M4 v0 정리
```

---

## 1. 로드맵

| 단계 | 이름 | 목표 | 완료 기준 | 예상 |
| --- | --- | --- | --- | --- |
| **M0** | 문서 잠금 | 경로·설계·본 계획 | CONCEPT §0 + 설계 + 본 문서. 코드 0 | **done** (D-135) |
| **M1** | 실행 가능 | `myharness` 가 grok 를 켠다 | `grok plugin validate` PASS. grok 없으면 exit 2. 12 동사 번역. `-p` 한 턴 | **done** (D-136) |
| **M2** | 도메인 가치 | MiniMax + 3-도메인 skill/hook + task | snippet + setup-model + hook deny + task 파일. live MiniMax 호출은 키 있을 때 | **done** (D-137, live LLM 은 opt-in) |
| **M3** | 경화 | 설치 경로 + 사용 문서 | `scripts/install.sh` + smoke. Rust clap 은 보류 | **done** (D-138, M3.2 deferred) |
| **M4** | v0 정리 | crates archive | yklee 승인 후 `archive/v0-runtime/` 또는 태그 | 보류 |

OOS: 자체 Plugin loader, grok 소스 포크, 엔진 5 components 재구현, v0 crates 신규 기능.
**표면 TUI + ACP 는 D-140 in-scope** — [DETAILED_DESIGN_SURFACE.md](./DETAILED_DESIGN_SURFACE.md) PR-S0–S8.

---

## 2. 마일스톤 상세

### M0 — 문서 잠금 (done)

| ID | 산출 | 상태 |
| --- | --- | --- |
| M0.1 | CONCEPT §0 / §5.1 overlay | done D-135 |
| M0.2 | DETAILED_DESIGN_OVERLAY.md | done D-135 |
| M0.3 | 본 계획 (로드맵 + WBS) | **본 문서** |

### M1 — 실행 가능 (이번 사이클)

| ID | WBS | 산출 | 완료 기준 |
| --- | --- | --- | --- |
| M1.1 | 1.1 | `plugins/myharness/plugin.json` + 빈 skills/commands/agents + hooks stub | `grok plugin validate plugins/myharness` exit 0 — **done** |
| M1.2 | 1.2 | `bin/myharness` 셸 래퍼 | `--help`, grok 가드, 12 동사 → `grok -p` / TUI — **done** |
| M1.3 | 1.3 | `scripts/overlay_smoke.sh` | 가드 + validate + usage 3 assert — **done** |

### M2 — 도메인 가치

| ID | WBS | 산출 | 완료 기준 |
| --- | --- | --- | --- |
| M2.1 | 2.1 | MiniMax `[model.*]` snippet + `myharness setup-model` | **done** (`examples/minimax.toml`, `--print-snippet` / `--dest`) |
| M2.2 | 2.2 | 3-도메인 skills (code-review, server-health, env-bootstrap) + 최소 agents | **done** |
| M2.3 | 2.3 | PreToolUse: `rm -rf /`, `server deploy` deny/confirm | **done** (훅 + `--yes`) |
| M2.4 | 2.4 | `myharness task start\|end` | **done** (`~/.myharness/handoff/tasks/<id>.md`) |

### M3 — 경화

| ID | WBS | 산출 |
| --- | --- | --- |
| M3.1 | 3.1 | `install.sh` → `~/.local/bin/myharness` + plugin 사본 | **done** (`--prefix` / `--home` / `--uninstall`) |
| M3.2 | 3.2 | 래퍼를 Rust clap 으로 이전 (셸 동작 고정 후) | **deferred** — 셸 래퍼가 M1–M3 계약. clap 은 동작이 더 굳은 뒤 |
| M3.3 | 3.3 | README 사용법 (grok 설치 URL, MiniMax 키) | **done** |

### M4 — v0 정리 (yklee 승인)

| ID | WBS | 산출 |
| --- | --- | --- |
| M4.1 | 4.1 | `v0-standalone` 태그 |
| M4.2 | 4.2 | `myharness/` crates → `archive/v0-runtime/` |

---

## 3. WBS (작업 분해)

```
1. M1 실행 가능
   1.1 Plugin 스캐폴드
       1.1.1 plugin.json (name=myharness, camelCase)
       1.1.2 convention dirs: skills/ commands/ agents/ hooks/
       1.1.3 hooks.json stub (빈 PreToolUse matcher)
       1.1.4 grok plugin validate
   1.2 Thin CLI
       1.2.1 grok which + semver ≥ 1.0.3 (exit 2)
       1.2.2 plugin-dir 해석 (env → repo → ~/.myharness/plugins)
       1.2.3 인자 없음 → exec grok --plugin-dir
       1.2.4 code|server|env <verb> → grok -p 번역
       1.2.5 --mode=single → -p / --mode=loop → goal 프롬프트
       1.2.6 알 수 없는 인자는 grok 로 통과
   1.3 Smoke
       1.3.1 validate
       1.3.2 PATH 에서 grok 제거 시 exit 2
       1.3.3 --help 에 12 동사

2. M2 도메인
   2.1 model snippet + setup-model
   2.2 skills 3 + agents 3
   2.3 PreToolUse 가드 스크립트
   2.4 task start/end

3. M3 경화
   3.1 install.sh
   3.2 (선택) Rust clap
   3.3 README

4. M4 v0
   4.1 태그
   4.2 archive
```

의존: `1.1 → 1.2 → 1.3`. `2.*` 는 `1.3` 이후. `2.1` 과 `2.2` 는 병렬 가능. `3.*` 는 `2.1`+`2.3` 이후. `4.*` 는 yklee 게이트.

---

## 4. 이번 사이클 범위 (M1)

포함: 1.1 + 1.2 + 1.3  
제외: MiniMax 실호출 (M2.1), 본격 skill 본문 (M2.2), deploy 가드 본문 (M2.3), task 파일 포맷 확정 (M2.4)

성공 기준은 설계 §13 중 1·2·부분 3:

1. grok 없으면 `myharness` → exit 2 + 설치 URL
2. `myharness` (인자 없음) 은 grok 를 `--plugin-dir` 과 함께 exec (본 세션은 TUI 를 띄우지 않고 명령줄만 검증)
3. `myharness env diagnose --print-cmd` 가 번역된 `grok -p` 커맨드를 보여 줌

---

## 5. 추적

| 마일스톤 | 결정 | backlog |
| --- | --- | --- |
| M0 | D-135 | 2026-08-14.md §8 |
| M1 | D-136 | 본 사이클 |
| M2+ | 후속 D-NN | M1 종료 후 |

상태값: `planned` / `in_progress` / `blocked` / `done`.
