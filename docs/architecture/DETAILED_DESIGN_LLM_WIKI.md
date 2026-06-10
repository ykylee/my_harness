# LLM Wiki + Obsidian — Second Brain Fusion Design (D-71)

- 문서 목적: Karpathy 의 LLM Wiki 패턴을 my_harness 의 `second brain` 으로 정식 채택하고, Obsidian 과 융합하여 ai-workflow 와 통합 운영하는 설계를 정의한다.
- 범위: 디렉터리 레이아웃, 스키마, 오퍼레이션, ai-workflow 연동, v1.5 마일스톤 범위, 위험과 결정 포인트
- 대상 독자: yklee (오너), my_harness 개발자, AI 에이전트
- 상태: proposed (D-71, 2026-06-10)
- 관련 문서:
  - [CONCEPT.md §5.12 `~/.myharness/` 구조](../CONCEPT.md) (D-31)
  - [CONCEPT.md §5.13 LLM Wiki memory](../CONCEPT.md) (D-32, 본 디자인의 원안)
  - [CONCEPT.md §5.14 Skill/MCP first-class](../CONCEPT.md) (D-33)
  - [PROJECT_PROFILE.md](../PROJECT_PROFILE.md)
  - [ai-workflow/core/global_workflow_standard.md](../../ai-workflow/core/global_workflow_standard.md)
  - [ai-workflow/skills/session-start/SKILL.md](../../ai-workflow/skills/session-start/SKILL.md) (consumer 패턴 참조)
  - [ai-workflow/skills/code-index-update/SKILL.md](../../ai-workflow/skills/code-index-update/SKILL.md) (lint 스킬 패턴 참조)
  - 원본 레퍼런스: <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>

---

## 0. 결정 요약 (yklee 승인, D-71)

| # | 결정 | 채택 |
|---|---|---|
| D-71.1 | 위키 vault 위치 | **`~/wiki/`** 별도 Obsidian vault (out-of-repo) |
| D-71.2 | v1.5 적용 범위 | **CONCEPT 원안 그대로** — schema + lint only (full compile 은 v2.5 TASK-005-4 로 연기) |
| D-71.3 | ai-workflow ↔ wiki 관계 | **consumer** — ai-workflow 산출물 (handoff/backlog/state.json) 이 wiki 의 raw/ source 가 됨. 단, SSOT 위배 금지 (CONCEPT.md = SSOT 유지, wiki 는 derived) |
| D-71.4 | Obsidian 가시화 | graph view + dataview plugin + Obsidian Git (text-only vault 이므로 mobile 가능, 단 권장은 desktop-first) |
| D-71.5 | 검색 | v1.5 = `index.md` 만 (~100 소스까지). v2.5 = `qmd` (BM25+벡터+LLM 리랭킹, 로컬, MCP) 도입 검토 |

근거:
- **out-of-repo 채택** — second brain 은 사적 노트·독서·회고까지 흡수해야 하므로, public repo에 두는 건 부적합. mobile git (Working Copy) 으로 어디서나 노트가 가능해지는 게 second brain 의 본질.
- **v1.5 = schema+lint** — CONCEPT 원안의 점진적 도입 원칙을 존중. full compile 은 알고리즘/품질 튜닝이 필요해 v1.5 scope 를 부풀리지 않음.
- **ai-workflow = consumer** — SSOT (CONCEPT.md) 원칙을 깨지 않으면서도, "내가 한 일" 이 second brain 에 자동 축적되는 효과가 있음. 거울(mirror) 안 함 — single source of truth 와 mirror 의 분리가 흐려지면 동기화 책임이 양쪽으로 분산됨.

---

## 1. 아키텍처 — 3층 + 운영 메커니즘

```
┌──────────────────────────────────────────────────────────────────┐
│ Layer 0: Raw sources (불변, LLM 읽기 전용)                       │
│   - ai-workflow/memory/{handoff,backlog,state.json}   ← 자동    │
│   - ~/wiki/raw/clippings/*.md                         ← 수동    │
│   - ~/wiki/raw/books/, articles/, podcasts/           ← 수동    │
│   - LOG.jsonl (이벤트 스트림)                          ← 자동    │
│   - 외부에서 drop 되는 모든 immutable 자료                            │
└──────────────────────────────────────────────────────────────────┘
                              ▲
                              │ ingest (LLM or 수동)
                              │
┌──────────────────────────────────────────────────────────────────┐
│ Layer 1: Wiki (LLM 작성, 인간 읽기)                              │
│   - ~/wiki/wiki/<topic>.md        (개념/엔티티/요약 페이지)        │
│   - ~/wiki/wiki/overview.md       (현재 종합, top-level)           │
│   - ~/wiki/wiki/comparisons/      (비교 분석, query 산출물)        │
│   - cross-references: [[PageName]] Obsidian wiki-link            │
│   - YAML frontmatter: tags, sources, last_touched, contradiction  │
└──────────────────────────────────────────────────────────────────┘
                              ▲
                              │ 규율
                              │
┌──────────────────────────────────────────────────────────────────┐
│ Layer 2: Schema (human+LLM 공진화)                                │
│   - ~/wiki/AGENTS.md              (Obsidian vault 안 — 에이전트용)  │
│   - ~/wiki/schema/page_template.md  (페이지 형식)                  │
│   - ~/wiki/schema/lint_rules.md     (제약 검증 규칙)              │
│   - Obsidian 설정 (.obsidian/)    (graph view, dataview, git)     │
└──────────────────────────────────────────────────────────────────┘

운영 메커니즘 (별도 디렉터리):
┌──────────────────────────────────────────────────────────────────┐
│ Layer 1.5: 운영 / 네비게이션                                      │
│   ~/wiki/                                                            │
│   ├── index.md           ← 콘텐츠 카탈로그 (페이지+한 줄 요약)     │
│   ├── log.md             ← 시계열 변경 이력 (## [YYYY-MM-DD] 접두) │
│   └── _lint/             ← lint 산출물 (제안, 자동 머지 금지)       │
│       └── report_YYYY-MM-DD.md                                            │
└──────────────────────────────────────────────────────────────────┘
```

핵심 비유 (Karpathy 그대로 + 우리 적용):
- **Obsidian = IDE** (인간이 wiki 를 **읽는** 곳 — graph view, dataview, search)
- **LLM = 프로그래머** (wiki 를 **쓰는** 것 — ingest, query, lint)
- **Wiki = 코드베이스** (영구·가산·버전관리)

---

## 2. 디렉터리 레이아웃 (v1.5 마일스톤, 결정 1)

### 2.1 사용자 vault (사설, Obsidian 열기)

```
~/wiki/                                 # Obsidian vault root
├── .obsidian/                          # Obsidian 설정
│   ├── app.json                        # "showLineNumber": true 등
│   ├── dataview.json                   # Dataview plugin enable
│   ├── graph.json                      # graph view 색/필터
│   └── ...
├── .git/                               # Obsidian Git plugin 또는 외부 git
├── AGENTS.md                           # ★ 스키마 — LLM이 매 세션 시작 시 읽음
├── README.md                           # vault 소개 (인간용)
│
├── raw/                                # Layer 0 — LLM 읽기 전용
│   ├── _manifest.md                    # 원본 인덱스 (날짜별 추가)
│   ├── ai-workflow/                    # ← symlink 또는 export (D-71.3)
│   │   ├── handoff.md                  # ← ~/repos/my_harness/ai-workflow/memory/session_handoff.md
│   │   ├── backlog-index.md            # ← work_backlog.md
│   │   ├── state.json                  # ← state.json (raw 보관, 편집 X)
│   │   └── by-date/                    # ← daily backlog (YYYY-MM-DD.md)
│   │       ├── 2026-06-05.md
│   │       └── 2026-06-10.md
│   ├── clippings/                      # Obsidian Web Clipper 출력
│   │   └── 2026-06-10_karpathy-llm-wiki.md
│   ├── books/                          # 챕터별 노트
│   │   └── tolkien/silmarillion/
│   │       ├── ch01.md
│   │       └── characters.md
│   ├── articles/                       # 논문/아티클 (PDF → md)
│   ├── podcasts/                       # 전사 노트
│   └── personal/                       # 저널/회고/목표
│
├── wiki/                               # Layer 1 — LLM 작성
│   ├── overview.md                     # vault 종합 (top-level)
│   ├── concepts/                       # 개념 페이지
│   │   ├── llm-wiki-pattern.md
│   │   ├── memex.md
│   │   └── context-engineering.md
│   ├── entities/                       # 엔티티 페이지 (사람/도구/시스템)
│   │   ├── my-harness.md
│   │   ├── obsidian.md
│   │   ├── karpathy.md
│   │   └── ...
│   ├── topics/                         # 주제별 (멀티 엔티티 종합)
│   │   ├── agent-memory-design.md
│   │   ├── context-compression-2026.md
│   │   └── ...
│   ├── sources/                        # 원본별 요약 (1:1, page-per-source)
│   │   └── karpathy-gist-llm-wiki.md
│   ├── comparisons/                    # query 산출물 (가산)
│   │   └── ai-workflow-vs-llm-wiki.md
│   └── meta/                           # vault 운영 메타
│       └── conventions.md
│
├── index.md                            # ★ 운영 — 콘텐츠 카탈로그
├── log.md                              # ★ 운영 — 시계열 로그
│
└── schema/                             # Layer 2 — 스키마/규칙
    ├── page_template.md                # 표준 wiki page 형식
    ├── lint_rules.md                   # lint 검사 규칙
    └── naming.md                       # 파일/태그 명명 규칙
```

### 2.2 my_harness 저장소 측 (변경 최소, D-71.3)

my_harness 저장소 자체엔 새 위키 디렉터리를 두지 **않는다** (out-of-repo). 단 다음만 추가:
- `myharness-wiki` crate 또는 `~/.myharness/wiki/` 설정 모듈 — wiki 경로·vault 검증·AGENTS.md 자동 작성
- `myharness` CLI 에 `wiki` subcommand 후보 (v2.0+, v1.5 는 스킬 우선)

### 2.3 ai-workflow 측 (consumer 연동, D-71.3)

`ai-workflow/memory/` 는 **그대로** 유지. `~/wiki/raw/ai-workflow/` 로의 동기화는 두 가지 방식:

| 방식 | 장점 | 단점 | 채택 |
|---|---|---|---|
| Symlink | 실시간, 0 cost | Obsidian이 따라오는지 확인 필요, Windows 호환 | macOS 한정 OK |
| Export script | 명시적, 부서지기 쉬운 결합 없음 | 자동화 안 됨 (또는 별도 훅) | 채택 — 초기엔 수동, v2.0 훅화 |

**v1.5 권장 초기 동작**: `myharness wiki sync` 명령 또는 수동 `cp`/`rsync` — vault 진입 시 `wiki sync` 가 ai-workflow 산출물을 `~/wiki/raw/ai-workflow/` 로 복사 (SSOT 아님, mirror 아님 — **복사**).

---

## 3. 스키마 (`~/wiki/AGENTS.md`, v1.5 초안)

LLM이 매 세션 시작 시 가장 먼저 읽는 파일. 사람도 같은 파일을 열어 LLM과 규약을 합의한다.

```markdown
# AGENTS.md — LLM Wiki 운영 규칙 (v1.5, D-71)

## 0. 역할
- 이 vault 는 LLM Wiki 패턴을 Obsidian 으로 구현한 second brain 이다.
- 당신(LLM) 은 wiki/ 디렉터리의 **유일한 편집자**다. raw/ 는 읽기 전용.
- 사용자는 wiki/ 를 **읽기만** 한다. Obsidian graph view 와 dataview 로 탐색한다.

## 1. 3층 구조
- raw/  = 불변 원본. 당신이 인덱싱·요약할 대상. 절대 수정 금지.
- wiki/ = 당신이 작성·유지하는 interlinked markdown. 유일한 진실(쓰기 진실).
- schema/ = 페이지 형식·lint 규칙. 당신과 사용자가 함께 진화시킨다.

## 2. 오퍼레이션 3종
- **Ingest** — raw/ 에 새 항목이 생기면 (또는 사용자가 명시 요청 시):
  1. raw/_manifest.md 에 한 줄 추가 (날짜 + 출처)
  2. sources/ 에 page-per-source 요약 1건 작성
  3. 관련 concepts/, entities/, topics/ 페이지 갱신 (cross-ref)
  4. index.md 갱신 (해당 페이지 한 줄 요약)
  5. log.md 에 `## [YYYY-MM-DD] ingest | <source>` 한 줄 append
  6. (선택) wiki/comparisons/ 에 cross-source 분석 작성

- **Query** — 사용자가 물으면:
  1. index.md 먼저 읽고 후보 페이지 식별
  2. 관련 wiki/ 페이지 읽고 종합
  3. **답변을 Obsidian 내부에 파일링** — query/YYYY-MM-DD-<topic>.md 에 저장
  4. 답변 끝에 "Filed as [[query/2026-06-10-X]]" 한 줄 추가
  5. 이 query 페이지가 향후 ingest 의 source 가 될 수 있음 (가산)

- **Lint** — 사용자 요청 또는 세션 종료 시:
  1. schema/lint_rules.md 의 검사 실행
  2. 결과를 _lint/report_YYYY-MM-DD.md 에 저장
  3. 자동 머지 금지. 사용자 검토 후 반영

## 3. 페이지 형식 (필수 frontmatter)
모든 wiki/ 페이지 상단에:
\`\`\`
---
title: <title>
type: concept | entity | topic | source | comparison | query
tags: [<tag1>, <tag2>]
sources: [<raw/relative/path>...]
last_touched: YYYY-MM-DD
related: [[other-page]]
status: draft | reviewed | stale
contradictions: [<page> | <description>]
---
\`\`\`
빈 frontmatter 항목은 항목 자체를 생략하지 말고 `none` 으로 둔다 (lint 가 잡음).

## 4. Cross-reference 규칙
- 모든 위키링크는 Obsidian 형식 `[[Page Name]]`
- 파일명은 kebab-case + 의미 단위 (예: `context-compression-2026.md`)
- 동일 엔티티/개념이 두 페이지에 등장하면 한 곳을 primary 로 정하고 다른 곳은 `related:` 로 참조

## 5. 명명 규칙
- 디렉터리: concepts/, entities/, topics/, sources/, comparisons/, query/
- 파일: kebab-case.md
- 태그: 단수형, 소문자, 하이픈 (예: `#agent-memory`)
- 로그 항목: `## [YYYY-MM-DD] <op> | <title>` (op ∈ {ingest, query, lint, edit, fix})

## 6. 금지
- raw/ 수정
- wiki/ 페이지 삭제 (사용자 명시 승인 없이)
- 자동 lint 결과 자동 머지
- frontmatter 누락 상태로 wiki/ 페이지 작성
- index.md / log.md 갱신 누락

## 7. 도구 의존 (선택)
- Obsidian (인간용 — graph view, dataview, search)
- Obsidian Git (commit/push, 모바일은 Working Copy 권장)
- (v2.5+) qmd — BM25+벡터+LLM 리랭킹, MCP 서버
- myharness CLI (v2.0+ `myharness wiki` subcommand)
```

---

## 4. lint 스킬 (D-71.2 핵심, v1.5 산출물)

### 4.1 검사 항목 (`schema/lint_rules.md` 초안)

| ID | 검사 | 심각도 | 처리 |
|---|---|---|---|
| L01 | `wiki/` 페이지에 frontmatter 가 없거나 필수 필드 누락 | error | 자동 보강 제안, 자동 적용 금지 |
| L02 | `links:`/`related:` 의 대상 페이지 부재 (broken link) | error | 보고 |
| L03 | 어떤 페이지에서도 inbound link 가 없는 고아 페이지 | warn | 보고 (의도적일 수 있음) |
| L04 | 동일 주제의 페이지가 둘 이상 (duplicate concept/entity) | warn | 통합 제안 |
| L05 | `last_touched` 가 90일 이상 경과 (stale) | info | 재검토 제안 |
| L06 | `sources:` 의 raw/ 경로 부재 (orphan source ref) | error | 보고 |
| L07 | 모순: 동일 entity 를 다루는 두 페이지의 fact 가 충돌 | error | 사용자에게 알리고 결정 대기 |
| L08 | `index.md` 에 등록되지 않은 wiki/ 페이지 | warn | 자동 추가 제안 |
| L09 | `log.md` 가 1주일 이상 갱신 안 됨 | info | ingest 알림 |
| L10 | `wiki/` 페이지에 raw/ source 가 아예 없음 (1차 출처 부재) | error | 보고 (합성 페이지 허용하나 명시 요구) |

### 4.2 스킬 위치

```
ai-workflow/skills/wiki-lint/
├── SKILL.md
└── scripts/
    └── wiki_lint.py     # vault 경로 인자로 받아 L01~L10 검사
```

`SKILL.md` 핵심:
- **읽기**: `~/wiki/wiki/**`, `~/wiki/raw/_manifest.md`, `~/wiki/index.md`, `~/wiki/schema/lint_rules.md`
- **쓰기**: `~/wiki/_lint/report_YYYY-MM-DD.md` 만. wiki/ 자동 수정 금지.
- **출력**: JSON + 사람이 읽을 수 있는 markdown 리포트
- **권한 경계**: vault 안 read + `_lint/` 안 write 만. 그 외 변경 시도 시 error_code=PERMISSION_DENIED.

### 4.3 ai-workflow 통합 — `code-index-update` 와 동급의 스킬로

기존 `code-index-update` (코드베이스 인덱싱) 와 `wiki-lint` (second brain 인덱싱/검증) 가 대칭 구조. 양쪽 다:
- 읽기 전용 + 보조 디렉터리(_lint/ 또는 _code-index/) 에 산출물
- `session-start` 의 `next_documents` 후보에 자기 자신을 포함 가능

→ v1.5 에서 둘 다 정식 beta 승격 검토.

---

## 5. ai-workflow ↔ wiki 연동 (D-71.3, consumer)

### 5.1 데이터 흐름

```
my_harness repo 작업
   │
   ├─ ai-workflow/memory/{handoff,backlog,state.json,daily/}
   │       │
   │       │  myharness wiki sync (v2.0+)
   │       │  or 수동 cp -r (v1.5)
   │       ▼
   │  ~/wiki/raw/ai-workflow/   ← "내가 한 일" 의 immutable copy
   │       │
   │       │  LLM ingest
   │       ▼
   │  ~/wiki/wiki/{concepts,entities,topics,sources}/
   │       │
   │       │  LLM query
   │       ▼
   └─ Obsidian graph view (인간이 읽음)
```

### 5.2 wiki-lint 가 ai-workflow 도 검사하는 이유

`wiki-lint` 스킬은 L01~L10 검사 중 일부를 ai-workflow 산출물에도 적용 가능:
- L02 broken link — wiki/ → ai-workflow/ cross-ref 검사 (e.g. `[[../ai-workflow/memory/2026-06-10]]` 가 살아있는지)
- L07 contradiction — wiki/concepts/agent-memory.md ↔ ai-workflow/state.json 의 `decisions.decided[*]` 가 모순되는지

→ 이는 v1.5 의 추가 가치. **second brain 이 단순 노트 공간을 넘어 "프로젝트 상태와 노트가 서로 검증하는 구조"** 가 됨.

### 5.3 session-start 의 변경

`session-start` 스킬은 현재 PROJECT_PROFILE + handoff + backlog 만 본다. v1.5 에서 **vault 가 발견되면 wiki/ 도 함께 훑고 `next_documents` 에 wiki 페이지 후보를 추가**한다. (D-71.3 consumer 의 자연스러운 귀결.)

→ 이는 `session-start` 의 input 에 `wiki_path` 를 추가하는 작은 변경. CONCEPT §5.10 의 orchestrator 흐름에 부합.

---

## 6. v1.5 마일스톤 범위 (D-71.2)

CONCEPT 원안 그대로:
- ✅ `~/wiki/` 디렉터리 레이아웃 + AGENTS.md + page_template.md + lint_rules.md
- ✅ `index.md` / `log.md` 자동 작성 규칙 (사람이 직접 또는 LLM ad-hoc)
- ✅ `wiki-lint` 스킬 (L01~L10 검사, `_lint/` 에 리포트)
- ✅ `session-start` 의 vault 발견 시 wiki/ 도 훑는 확장
- ✅ Obsidian 권장 설정 (.obsidian/{dataview,graph,git} .json 템플릿)
- ⏸ **v2.5 TASK-005-4 로 연기**:
  - 자동 ingest (raw → wiki 컴파일)
  - cross-reference 자동 갱신
  - contradiction 자동 resolution
  - qmd / 임베딩 검색

**v1.5 의 명시적 non-goal** (사용자 오해 방지):
- raw → wiki **자동** 변환 (v1.5 에서는 LLM 이 ingest 명령 받았을 때만)
- wiki 페이지 자동 삭제·병합
- LLM 끼리의 모순 자동 해결 (사람이 결정)

---

## 7. Obsidian 통합 (D-71.4, 권장 설정)

### 7.1 권장 plugin
- **Dataview** — YAML frontmatter 메타로 동적 테이블/리스트 (예: `TABLE last_touched FROM "wiki/concepts"`)
- **Obsidian Git** — vault 를 git 으로 commit/push. 데스크탑은 OK, 모바일은 **Working Copy (iOS) / GitSync (Android)** 권장 (per 2026 검색 결과, Obsidian Git 모바일은 experimental)
- **Graph view (built-in)** — 무료·강력. hub/orphan 시각화에 충분.
- **Marp** (선택) — query 산출물을 슬라이드로

### 7.2 권장 .obsidian/ 설정
```
.obsidian/app.json
  - "showLineNumber": true      # LLM이 diff 보기 편함
  - "readableLineLength": true  # LLM 토큰 절약
  - "useMarkdownLinks": false   # Obsidian wiki-link [[...]] 우선
  - "newLinkFormat": "short"

.obsidian/dataview.json
  - "enableDataview": true
  - "enableInlineDataview": true
  - "dataviewJsQuery": true

.obsidian/graph.json
  - "showOrphans": true
  - "hideFirstPartyPlugins": false
```

### 7.3 .gitignore (vault 내)
```
.obsidian/workspace
.obsidian/workspace-mobile
.trash/
```

→ 위 파일들은 Obsidian 개인 상태라 git 에 안 올라가는 게 맞음. **Obsidian Git 의 commit 메시지 prefix** 는 `## [YYYY-MM-DD] edit | <description>` 로 통일 (log.md 와 동일 prefix).

---

## 8. 마이그레이션 (초기 1회)

### 8.1 vault 초기화 (v1.5 결정 시 1회)
1. `mkdir -p ~/wiki/{raw,wiki,schema,_lint}` + 하위 디렉터리
2. `~/wiki/AGENTS.md` 작성 (본 문서 §3 그대로)
3. `~/wiki/schema/{page_template,lint_rules,naming}.md` 3종 작성
4. `~/wiki/index.md` 빈 템플릿 작성
5. `~/wiki/log.md` 헤더 + 첫 줄 (`## [2026-06-10] init | LLM Wiki vault bootstrapped (D-71)`)
6. Obsidian 에서 "Open folder as vault" → `~/wiki/`
7. Dataview/Obsidian Git plugin 설치·활성화
8. `~/wiki/.gitignore` 작성
9. `cd ~/wiki && git init && git add -A && git commit -m "init: LLM Wiki vault (D-71)"` — **vault 자체도 git 으로 버전관리**

### 8.2 ai-workflow 산출물 첫 sync
- `cp -r ~/repos/my_harness/ai-workflow/memory ~/wiki/raw/ai-workflow`
- `~/wiki/raw/_manifest.md` 첫 줄 작성
- LLM 한테 "raw/ai-workflow/ 를 ingest 해줘" 요청 → sources/karpathy-...-style 한 줄 + concepts/my-harness.md 등 5~10 페이지 초안

### 8.3 그 이후 운영
- **인간** = raw/ 에 노트 drop + Obsidian 으로 wiki/ 탐색
- **LLM** = 사용자 대화 중 "이거 wiki 에 정리해줘" / "lint 돌려줘" 명령에 반응
- **자동산출물**: lint 리포트, query 시 비교표 등 Obsidian 안에서

---

## 9. 위험과 결정 포인트 (v1.5 진행 중 다시 봐야 할 것)

| # | 위험 | 대응 |
|---|---|---|
| R-1 | vault 비대해지면 `index.md` 만으로 부족 | v2.5 에서 qmd 도입. L08 검사 (index 누락) 가 v1.5 부터 강제. |
| R-2 | LLM이 wiki/ 를 실수로 raw/ 에 작성하거나 그 반대 | AGENTS.md §1, §6 에서 명시. lint L01 (frontmatter) 가 잡아줌. |
| R-3 | ai-workflow/ ↔ wiki/ 동기화 누락 | v1.5 = 수동 cp, v2.0 = `wiki sync` 훅. lint L09 가 log.md 갱신 강제. |
| R-4 | 위키가 SSOT 처럼 여겨져서 CONCEPT.md 와 drift | v1.5 AGENTS.md §3 + PROJECT_PROFILE.md 에 "wiki = derived, CONCEPT = SSOT" 명시. lint 가 양방향 검사. |
| R-5 | 모바일 git 으로 vault 동기화 시 충돌 | 2026 권장: 데스크탑이 source of truth, 모바일은 read-only 또는 Working Copy + 수동 머지. |
| R-6 | Obsidian Graph 가 너무 커져서 noise | 100 페이지 단위로 vault 분할 검토 (`~/wiki/personal/`, `~/wiki/work/` 분리). |
| R-7 | yklee 만 사용하지만 multi-user 가능성 | v1.5 = single-user 가정, multi-user 는 v3.0 (CONCEPT §5.3 RBAC) |

---

## 10. 다음 행동 (v1.5 TASK-005-2 진입 시)

1. **D-71 결정 확정** (본 문서 §0)
2. **vault 초기화** (§8.1, 1~2시간)
3. **`wiki-lint` 스킬 작성** (v1.5 산출물, `ai-workflow/skills/wiki-lint/`)
4. **`session-start` 확장** (vault 발견 → wiki/ 후속)
5. **CONCEPT.md §5.13 갱신** — v1.5 범위를 본 디자인에 맞춰 명확화, §5.12 `~/.myharness/` 의 `llm-wiki/` 와의 관계 명시
6. **`PROJECT_PROFILE.md` §2 문서 구조에 vault 경로 추가** (D-71.1)
7. **`~/.myharness/wiki/` 설정 모듈** (vault 경로 + 검증) — myharness crate v0.2 정도에서
8. **TASK-005-2 v1.5 plan 에 본 디자인의 §6 v1.5 범위 항목 4개 추가**

### 11. v2.0+ hook
- `myharness wiki ingest <source-path>` — raw/ 등록 + LLM 호출
- `myharness wiki query <question>` — index + qmd 후속
- `myharness wiki lint` — wiki-lint 스킬 호출
- `myharness wiki sync` — ai-workflow → raw/ai-workflow/ mirror
- `myharness wiki vault init` — §8.1 자동화

---

## 변경 이력
- 2026-06-10 (D-71) — 초안 작성. yklee 와 3 결정 합의 (vault 위치=vault out-of-repo, v1.5 범위=schema+lint, ai-workflow 관계=consumer). CONCEPT §5.13 (D-32) 의 후속.
