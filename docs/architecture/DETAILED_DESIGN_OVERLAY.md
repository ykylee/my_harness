# DETAILED_DESIGN_OVERLAY.md — grok overlay 재구성 (D-135)

- 문서 목적: D-135 (A안 overlay) 의 구현 사양. CONCEPT.md 의 제품 포지셔닝을 모듈/CLI/plugin/디렉터리로 내린다.
- 범위: 래퍼 CLI, grok plugin 4계층, `[model.*]` 연결, 홈 디렉터리 분리, v0 crate 처분, 1차 PR 분해
- 대상 독자: yklee, 오케스트레이터, 구현 워커
- 상태: active (D-135, 2026-08-14). 제품 경로의 설계 SSOT. v0 Rust MVP 설계는 [INITIAL_DESIGN.md](./INITIAL_DESIGN.md) (historical)
- 최종 수정일: 2026-08-14
- 관련 문서: [CONCEPT.md](../CONCEPT.md) (제품 SSOT) · [grok-build.md](../references/grok-build.md) (엔진 실측) · [INITIAL_DESIGN.md](./INITIAL_DESIGN.md) (v0, superseded)

---

## 0. 결론 (TL;DR)

my_harness 는 더 이상 standalone 런타임이 아니다. **설치된 `grok`(Grok Build) 가 엔진**이고, **myharness 는 3-도메인 래퍼 + grok plugin** 이다.

```
yklee
  └─ myharness <domain> <verb> [...]
        ├─ grok 존재/버전 가드
        ├─ 3-도메인 동사 → grok -p / --agent / stdio
        ├─ grok plugin install --trust (hooks/skills)
        └─ grok -p / TUI ( --plugin-dir 는 agent 전용 )
              └─ [model.minimax] chat_completions + base_url
```

5 components (Tools / Context / Session / Plugins / Sub-agents) 는 **재구현하지 않는다.** grok 가 이미 1:1 로 가지고 있다.

---

## 1. Key Decisions

| ID | 결정 | 근거 |
| --- | --- | --- |
| **D-135** | 제품 경로 = **A overlay**. B grok 포크 비권장. C goose 포크는 독립 런타임이 필요할 때만 | grok-build 14섹션 실측. 5 components 이미 조립. 소스 포크 = 136만 줄 + generated workspace + 외부 PR 거부 |
| **D-135.1** | 런타임 의존 = **설치된 `grok` ≥ 1.0.3** | 공식 바이너리 사용. 소스 빌드/포크 안 함 |
| **D-135.2** | 사용자 표면 = `myharness code\|server\|env ...` 유지 | 3-도메인 UX 가 우리 차별점. grok 에 해당 서브커맨드 없음 |
| **D-135.3** | 확장 = grok `plugin.json` 4계층 (commands/skills/agents/hooks + MCP) | CONCEPT §5.7 자체 loader 폐기. TASK-005-2 Sub-task 2 (자체 Plugin 인프라) = **OOS** |
| **D-135.4** | LLM = grok `[model.<name>]`. MiniMax 기본, Ollama 는 chat_completions | sampler 3 backend. 우리 llm crate 는 엔진이 아님 |
| **D-135.5** | 홈 분리. `~/.myharness/` = 래퍼·workflow. `~/.grok/` = 엔진 세션·auth·memory | `GROK_HOME` 을 함부로 `~/.myharness` 로 바꾸지 않음 (이전 비용 + 공식 경로 충돌) |
| **D-135.6** | v0 `myharness/` crates = **참고 구현**. 즉시 삭제 금지, 신규 기능 금지 | 23k LOC. 패턴 이식·회귀 참고용. overlay 가 안정되면 archive |
| **D-135.7** | CONCEPT §0 standalone / Direct LLM / Zero external runtime 문장 **폐기** | overlay 와 모순 (grok-build.md §13.8) |
| **D-135.8** | TUI 브랜드는 grok 그대로. 로고/키맵/빌트인 slash 는 overlay 로 못 지움 | 실측. 제품 주인이 되는 길이 아님 |
| **D-135.9** | 개발 workflow (Mavis / `ai-workflow/` / MiniMax.md) 는 **그대로** | D-25 의 "개발 도구와 산출물 분리" 유지. 바뀌는 것은 산출물 런타임 |

---

## 2. 프로세스 토폴로지

### 2.1 런타임

```
myharness (thin CLI)
    │  which grok && grok --version
    │  resolve plugin dir + model + cwd
    ├─ interactive / no extra args
    │     exec grok [--plugin-dir ...] [-m minimax] [PROMPT]
    ├─ myharness <domain> <verb> [args]
    │     exec grok -p "<translated prompt>" --plugin-dir ... -m ...
    ├─ myharness agent stdio
    │     exec grok agent stdio --plugin-dir ...
    └─ myharness task start|end
          자체 구현 (ai-workflow 파일 write). grok 호출 없음
```

기본 TUI 는 grok pager 가 in-process `MvpAgent` 를 띄운다. 우리는 ACP 클라이언트를 다시 짜지 않는다. 래퍼는 `exec` 가 기본이다.

### 2.2 우리가 소유하는 프로세스

| 바이너리 | 역할 | 구현 |
| --- | --- | --- |
| `myharness` | 도메인 동사 번역, grok 가드, plugin/model 플래그 | 신규 thin CLI (Rust 또는 셸 부트스트랩 → Rust) |
| (없음) | TUI / tool dispatch / session | grok |

1차 슬라이스는 셸 래퍼로도 충분하다. 인자 파싱·exit code·버전 가드가 굳으면 Rust clap 으로 옮긴다.

---

## 3. CLI 표면

### 3.1 유지하는 사용자 명령

CONCEPT §5.2 의 12 동사는 **그대로**다. 구현만 grok 호출로 바뀐다.

| 사용자 명령 | 번역 |
| --- | --- |
| `myharness` (인자 없음) | `grok --plugin-dir $PLUGIN` |
| `myharness code review <target>` | `grok -p "Review <target> …" --agent plan --plugin-dir $PLUGIN` |
| `myharness code implement "<feature>"` | `grok -p "Implement <feature> …" --plugin-dir $PLUGIN` |
| `myharness code test <path>` | `grok -p "Run and analyze tests at <path>" --plugin-dir $PLUGIN` |
| `myharness code commit "<msg>"` | `grok -p "Create a git commit: <msg>" --plugin-dir $PLUGIN` |
| `myharness server status [host]` | `grok -p "Check server status …" --plugin-dir $PLUGIN` + PreToolUse |
| `myharness server logs <svc> [N]` | 동일 패턴 |
| `myharness server deploy <env>` | 동일 패턴. deploy 훅 deny-by-default |
| `myharness server config <action>` | 동일 패턴 |
| `myharness env setup\|install\|shell\|diagnose` | 동일 패턴 |
| `myharness --mode=single …` | `grok -p …` (서브에이전트 없이 one-shot) |
| `myharness --mode=loop --goal …` | `grok` + 래퍼가 goal/max-iterations 를 프롬프트에 실음. grok 에 ralph-wiggum 플래그 없음 |
| `myharness task start\|end` | 래퍼 자체. `ai-workflow/memory/` 또는 `~/.myharness/handoff/` |

`--mode=orchestrator` 는 grok 기본 TUI 와 같다. 별도 플래그로 노출할 필요 없다.

### 3.2 grok 로 넘기는 플래그

래퍼가 항상 붙이는 것:

- `--plugin-dir <abs-path>` — 자동 trust (grok-build.md §3.2)
- `-m <model>` — 기본 `minimax` (config override)
- 필요 시 `--permission-mode`, `--sandbox`, `-r` / `-c`

래퍼가 넘기지 않는 것:

- `--system-prompt-override` — 1차에서 안 씀. 암호화 프롬프트를 통째 교체하는 핵
- `--yolo` / `--always-approve` — 사용자 명시 플래그로만

### 3.3 exit code

| 상황 | code |
| --- | --- |
| grok 없음 / 버전 `< 1.0.3` | 2 |
| 도메인 동사 번역 실패 (인자 부족) | 2 |
| grok 가 반환한 code | 그대로 |
| task start/end 파일 IO 실패 | 1 |

---

## 4. Plugin 레이아웃

자체 `PluginLoader` / manifest crate 는 만들지 않는다. grok 가 읽는 트리를 저장소에 둔다.

```
plugins/myharness/                    # --plugin-dir 대상
├── plugin.json
├── commands/                         # slash 추가 (빌트인과 충돌 시 빌트인 승)
│   ├── code-review.md
│   ├── server-status.md
│   └── env-diagnose.md
├── skills/
│   ├── code-review-best-practices/SKILL.md
│   ├── git-workflow/SKILL.md
│   ├── server-health-check/SKILL.md
│   ├── log-pattern-analysis/SKILL.md
│   ├── env-bootstrap/SKILL.md
│   ├── dotfiles-sync/SKILL.md
│   └── provider-auto-config/SKILL.md
├── agents/
│   ├── code-reviewer.md
│   ├── code-implementer.md
│   ├── server-status.md
│   └── env-diagnose.md
└── hooks/
    └── hooks.json                    # PreToolUse: 서버/환경 가드
```

`plugin.json` (camelCase, grok-build.md §9.1):

```json
{
  "name": "myharness",
  "version": "0.1.0",
  "skills": "skills/",
  "commands": "commands/",
  "agents": "agents/",
  "hooks": "hooks/hooks.json"
}
```

discovery 우선: `--plugin-dir`(자동 trust) → 프로젝트 `.grok/plugins` → `~/.grok/plugins`. 래퍼는 **항상 `--plugin-dir`** 로 우리 트리를 싣는다.

설치 위치:

- 개발: 저장소 `plugins/myharness/`
- 사용자: `~/.myharness/plugins/myharness/` (래퍼가 복사하거나 심볼릭 링크)

빌트인 grok 에이전트 이름 (`general-purpose` / `explore` / `plan`) 은 섀도잉하지 않는다. 우리 에이전트는 `code-reviewer` 등 고유 이름.

---

## 5. LLM / Auth

### 5.1 모델

`~/.grok/config.toml` (또는 래퍼가 생성하는 snippet):

```toml
[model.minimax]
model = "MiniMax-M3"
base_url = "https://api.minimax.io/v1"
env_key = "MINIMAX_API_KEY"
api_backend = "chat_completions"

[model.ollama]
model = "qwen2.5-coder:32b"
base_url = "http://localhost:11434/v1"
api_backend = "chat_completions"
```

기본 `-m minimax`. xAI 로그인 UI·구독·image/video 는 건드리지 않는다. MiniMax 기본일 때 그 툴은 실패할 수 있다 (grok-build.md §14.4).

### 5.2 Auth

- MiniMax / OpenAI 호환: env + grok `auth.json` (평문, unix 0600)
- 우리 v0 `myharness-auth` (Device Grant, keyring) 는 **1차에서 연결하지 않는다**
- 후속: grok login 과 MiniMax device flow 를 래퍼 `myharness auth` 가 중계할지 별도 결정

D-51~D-58 의 OAuth 구현은 v0 참고 자산이다. overlay 1차의 blocker 가 아니다.

---

## 6. 홈 디렉터리

| 경로 | 소유 | 내용 |
| --- | --- | --- |
| `~/.grok/` | grok | sessions JSONL, auth.json, memory, trusted-plugins, config.toml |
| `~/.myharness/` | 래퍼 | config (래퍼 기본 모델/plugin path), state, handoff, log.jsonl, plugins 사본 |

`GROK_HOME=~/.myharness` 로 합치지 않는다. 세션·인증 이전 비용과 공식 경로 충돌 (grok-build.md §14.5).

래퍼 config 예시 `~/.myharness/config/config.toml`:

```toml
[engine]
binary = "grok"
min_version = "1.0.3"
plugin_dir = "~/.myharness/plugins/myharness"

[llm]
default_model = "minimax"

[workflow]
mode = "auto"   # D-26 유지. ai-workflow/ 발견 시 task/handoff sync
```

---

## 7. 보안 / Permission

엔진 permission 은 grok 파이프라인 (deny > YOLO > grant > auto > prompt). 우리는 정책을 다시 짜지 않는다.

우리 가드는 **PreToolUse 훅** 한 겹:

- `server deploy` / `rm -rf` / 프로덕션 호스트 → `{"decision":"deny"}` 또는 사용자 확인 문구
- hook 실패는 grok 가 **fail-open**. 보안 경계로 믿지 말 것 (grok-build.md §13.4)
- 진짜 차단이 필요하면 grok `--deny` 규칙 + folder-trust 를 같이 건다

4 permission mode 사용자 플래그:

| 우리 이름 (구) | grok 플래그 |
| --- | --- |
| default | `--permission-mode default` |
| acceptEdits | `--permission-mode acceptEdits` |
| plan | `--permission-mode plan` |
| bypassPermissions | `--permission-mode bypassPermissions` |

---

## 8. v0 crate 처분

`myharness/` workspace (core / llm / tui / tools / context / cli / auth / compression) 는 **즉시 삭제하지 않는다.**

| 단계 | 규칙 |
| --- | --- |
| 지금 | 신규 기능 금지. 문서에서 "엔진"으로 인용 금지. "v0 참고 구현" |
| overlay 1차 (래퍼+plugin smoke) | crates 그대로. 테스트는 돌리지 않아도 됨 |
| overlay 안정 (주 사용이 grok 경로) | `myharness/` → `archive/v0-runtime/` 또는 태그 `v0-standalone` 후 트리에서 제거 |

INITIAL_DESIGN.md / REQUIREMENTS.md / TC_*.md 는 **v0 historical**. 배너만 달고 본문은 보존 (audit trail).

---

## 9. CONCEPT 매핑 (무엇이 어디로 갔는지)

| CONCEPT (구) | overlay (신) |
| --- | --- |
| §0 standalone CLI/TUI | §0 grok overlay 래퍼 |
| §5.1 자체 5 components | grok 엔진 5 components |
| §5.2 12 명령 | 래퍼 동사 → grok -p / --agent |
| §5.3 5 install paths | `grok` 공식 install + 래퍼 설치 |
| §5.4 자체 hook md | grok hooks.json PreToolUse |
| §5.5 rig-core 직접 통신 | grok `[model.*]` |
| §5.6 Layer 2 built-in | grok compaction. headroom 재구현 OOS 유지 (D-130/D-66) |
| §5.7 자체 plugin 4계층 | grok plugin.json |
| §5.8 Zero external runtime | 런타임 의존 = grok |
| §5.10 3 mode | TUI 기본 / `-p` / 래퍼 loop 프롬프트 |
| §5.11 15 sub-agent 하드코딩 | grok `agents/*.md` + 빌트인 3 |
| §5.12 `~/.myharness/` only | 래퍼 홈 + `~/.grok/` 엔진 홈 |
| TASK-005-2 자체 Plugin 인프라 | **OOS** |

유지: 3-도메인, 한국어 보고, 6원칙, Mavis zero coupling(개발), MiniMax 우선, local-only memory 기본.

---

## 10. 리스크

| 리스크 | 대응 |
| --- | --- |
| grok 업데이트가 래퍼 플래그를 깨뜨림 | `min_version` 가드 + smoke (`myharness env diagnose`) |
| 공개 트리 lag (커밋 5 vs 설치 1.0.3) | 설치 바이너리 기준. 소스 클론은 분석만 |
| MiniMax 기본 + xAI-only 툴 실패 | plugin 에서 image/web_search 비활성 문서화 |
| hook fail-open | `--deny` 병행. 서버 deploy 는 래퍼가 확인 프롬프트 |
| 브랜드/키맵 못 지움 | 사용자에게 사전 고지. 제품 주인이 아님 |
| v0 crate 와 문서 drift | 배너 + D-135.6. 신규 기능 금지 |
| CONCEPT 잔여 standalone 문장 | 본 사이클에서 §0/§5.1/§5.8/§5.9.4 교체. 세부 §는 "엔진=grok" 주석 |

---

## 11. Open Questions

구현을 막지 않는 후속. 1차 슬라이스에서 기본값을 박고 간다.

1. 래퍼 언어 — **기본: 1차는 셸, 직후 Rust clap**. (확정: D-135 구현 순서)
2. `myharness auth` 를 grok login 과 중계할지 — **1차 skip**. env `MINIMAX_API_KEY` 만
3. loop mode 를 grok 세션으로 돌릴지 래퍼 재실행으로 돌릴지 — **1차 = 프롬프트에 goal 삽입**
4. Gitea 에 grok-build 미러를 둘지 — **안 둠** (1.3M LOC)
5. D-130 CCR/Memory fail-closed — overlay 경로와 무관. **별도 백로그**

---

## 12. PR Plan

| PR | 제목 | 파일 | 의존 | 내용 |
| --- | --- | --- | --- | --- |
| **PR-0** | docs: D-135 overlay 재구성 | CONCEPT, 본 문서, README, PROFILE, MiniMax, AGENTS, REFERENCES, grok-build §15, INITIAL_DESIGN/REQUIREMENTS 배너, development_log, memory | 없음 | **본 사이클**. 코드 0 |
| **PR-1** | feat: plugin 스캐폴드 | `plugins/myharness/plugin.json` + 최소 skill 1 + hooks.json stub | PR-0 | grok 가 로드하는 빈 plugin |
| **PR-2** | feat: thin CLI 래퍼 | `bin/myharness` 또는 `crates/wrapper` | PR-1 | grok 가드 + `--plugin-dir` + 12 동사 번역 |
| **PR-3** | feat: MiniMax `[model.*]` | 래퍼가 snippet 생성 또는 문서화된 config | PR-2 | `MINIMAX_API_KEY` smoke |
| **PR-4** | feat: 3-도메인 skills + PreToolUse | skills/* + hooks | PR-2 | 서버/환경 가드 |
| **PR-5** | feat: task start/end 래퍼 | workflow 파일 write | PR-2 | D-26 유지 |
| **PR-6** | chore: v0 crate archive 결정 | `myharness/` 이동 또는 태그 | PR-4 이후, yklee 승인 | 신규 기능 금지 확인 후 |

PR-0 이 끝나야 PR-1 을 연다. 코드는 다음 세션.

---

## 13. 성공 기준 (PR-2 smoke)

1. `grok` 없는 머신에서 `myharness` → exit 2 + 설치 URL
2. `myharness` (인자 없음) → grok TUI, plugin 로드
3. `myharness env diagnose` → grok -p 한 턴 + 한국어 보고
4. MiniMax 키 있으면 실제 응답. 없으면 친절한 실패
5. 서버 `deploy` 는 PreToolUse 또는 래퍼 확인 없이 실행되지 않음
