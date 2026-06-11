# Wiki-Query Skill Spec

- 문서 목적: `wiki-query` skill 의 입력/출력 계약, 동작 순서, 권한 경계, 실패 규칙을 정의한다.
- 범위: LLM Wiki vault (`~/wiki/`) 의 `wiki/projects/<project>/` 페이지에 대한 read-only 검색(read 모드) + (선택) `query/` 페이지 신규 작성·`log.md` append·`index.md` 갱신(--file 모드). vault 의 §2.2 Query 오퍼레이션 6 step 자동화.
- 대상 독자: AI agent 설계자, skill 구현자, vault 운영자, DevHub / my_harness 프로젝트 멤버
- 상태: **draft** (D-79, 2026-06-11)
- 최종 수정일: 2026-06-11 (D-79 Phase 1 — 본 저장소 thin wrapper DONE, my_harness SSOT 작성)
- 관련 문서: `~/wiki/AGENTS.md` (vault 운영 규약, §2.2 Query), `~/wiki/schema/lint_rules.md` (L01~L10), [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md), [`./session_start_skill_spec.md`](./session_start_skill_spec.md), `../skills/wiki-query/SKILL.md`

## 1. 목적

`wiki-query` skill 은 LLM Wiki vault 의 Query flow (AGENTS.md §2.2) 를 자동화한다. **사용자가 "vault 에서 X 에 대해 아는 거 정리해줘"** 라고 말할 때, 본 skill 은 다음 6 단계를 query 1건에 대해 실행한다:

1. `index.md` 읽고 후보 페이지 식별
2. 관련 `wiki/projects/<project>/{concepts,entities,topics,sources,...}` 페이지 read + 종합
3. 답변 끝에 `Filed as [[query/<date>-<topic>]]` 한 줄 추가
4. `query/` 페이지 본문 4섹션 (질문 / 사용 컨텍스트 / 답변 / 후속 액션) 작성
5. `log.md` 에 `## [<date>] query | <topic>` 한 줄 append
6. 답변은 Obsidian 내부 file → 향후 ingest source 가산 가능

핵심 정공법: **vault = 공유 자원 (DevHub/my_harness 2 프로젝트 동거)**, **raw/ = 읽기 전용 (mirror), wiki/ = LLM 편집**. 본 skill 의 read 모드는 100% read-only, write 모드(--file)도 `query/`, `log.md`, `index.md` 3종에만 한정. raw/ 절대 수정 금지 (AGENTS.md §6).

본 skill 은 D-72 §11.1 thin-wrapper 정공법의 SSOT 이며, DevHub 본 저장소 wrapper (`scripts/wiki-query.sh`) 는 본 skill 이 부재 시 exit 1 + SSOT 경로 안내.

## 2. 선행 원칙

- vault 운영 규칙은 `~/wiki/AGENTS.md` (v1.5, D-71) 의 정공법 우선. 본 skill 은 그 §2.2 Query 6 step 자동화.
- 본 skill 의 SSOT = [`./wiki_query_skill_spec.md`](./wiki_query_skill_spec.md). DevHub wrapper 의 `wiki-query.sh` 와 명확한 책임 분리:
  - `wiki-query.sh` (DevHub wrapper) = 통합 entry point + option parsing + my_harness skill dispatch
  - `wiki-query` (my_harness skill) = 실제 query 4 primitive + 6 step + side effect
- 본 skill 은 기본 `--no-file` mode. 실제 query 페이지 file + log.md append + index.md 갱신은 `--file` 명시 시점.
- frontmatter 누락 상태로 wiki page 작성 금지 (L01 위반, AGENTS.md §6). 본 skill 의 --file 모드 결과물(query/ 페이지)도 8 key frontmatter 필수.
- `index.md` / `log.md` 갱신 시 idempotent 보장 (같은 date+topic line 중복 X).
- 자동 lint 결과 자동 머지 금지 (AGENTS.md §6, wiki-lint 권고 정공법).
- cross-project link 는 `wiki/cross/` 에서 종합. `wiki/projects/<project>/` 내부의 cross-project link 자제 (AGENTS.md §11.6).
- **자동 vault Gitea push 금지** (사용자 수동, AGENTS.md §6.5 정책).

## 3. 입력 계약

### 3.1 필수 입력

- `--query` (필수) — 검색어. full-text / wikilink(`[[<text>]]`) / frontmatter key 모두 매칭.

### 3.2 선택 입력

- `--vault-path` (기본: `~/wiki`) — vault 루트 경로
- `--project` (기본: `devhub`) — 대상 project (`devhub` | `my-harness`). `cross` 미지원.
- `--tag` — frontmatter `tags:` 필터 (AND, 단일)
- `--type` (enum) — `concept` | `entity` | `topic` | `source` | `comparison` | `query`
- `--limit` (int, 기본 20) — 최대 결과 수. 0 이하 = 무제한
- `--format` (enum, 기본 `md`) — `md` | `json` | `plain`. `json` 은 다른 tool 입력용
- `--file` (flag, 기본 off) — `--no-file` 의 반대. query/ 페이지 자동 file + log.md 1 line append + index.md 1 line 갱신 (AGENTS.md §2.2 step 3-5)
- `--quiet` (flag) — stderr 메시지 최소화
- `--output` (enum, 기본 `json`) — `json` | `markdown` | `both`. lint report 용

### 3.3 입력 해석 규칙

- `--query` 부재 시 exit 2 (invalid option)
- `--project` 가 `devhub` | `my-harness` 외이면 exit 2
- `--format` 가 `md` | `json` | `plain` 외이면 exit 2
- `--type` 가 6개 enum 외이면 exit 2
- `--file` 와 `--no-file` 동시 지정 시 error (둘 중 하나만)
- `--limit` 0 이하는 무제한
- `--output` 가 `json` | `markdown` | `both` 외이면 exit 2

## 4. 출력 계약

### 4.1 JSON (stdout, `--output json` 또는 `--format json`)

```json
{
  "ok": true,
  "query": "Keycloak RBAC",
  "project": "devhub",
  "mode": "no-file",
  "tool_version": "0.1.0",
  "examined_at": "2026-06-11T05:20:00Z",
  "hit_count": 7,
  "results": [
    {
      "title": "rbac",
      "type": "concept",
      "tags": ["rbac", "auth"],
      "path": "wiki/projects/devhub/concepts/rbac.md",
      "sources": ["raw/projects/devhub/docs/governance/code-taxonomy.md"],
      "last_touched": "2026-06-11",
      "excerpt": "RBAC (Keycloak + cache + row scoping) — ...",
      "links": [],
      "backlinks": ["keycloak", "devhub-auth-session"]
    }
  ],
  "warnings": [],
  "errors": []
}
```

**9 key**: `ok` / `query` / `project` / `mode` / `tool_version` / `examined_at` / `hit_count` / `results` / (`warnings`, `errors`).

**results 내 9 sub-key**: `title` / `type` / `tags` / `path` / `sources` / `last_touched` / `excerpt` / `links` / `backlinks`.

### 4.2 Markdown (stdout, `--format md` 기본)

```markdown
# Query: "Keycloak RBAC"
- vault: /home/yklee/wiki
- project: devhub
- filters: tag=N/A, type=N/A, limit=20
- mode: no-file
- results: 7

## Hits

### [[rbac]] (type: concept, tags: [rbac, auth])
- path: wiki/projects/devhub/concepts/rbac.md
- sources: [raw/projects/devhub/docs/governance/code-taxonomy.md]
- last_touched: 2026-06-11
- excerpt: RBAC (Keycloak + cache + row scoping) — DevHub 의 권한 모델은 ...

### [[keycloak]] (type: entity, tags: [keycloak, sso])
- path: wiki/projects/devhub/entities/keycloak.md
- sources: [...]
- last_touched: 2026-06-11
- excerpt: DevHub SSO/IdP (25.0 → 26.0) — single source of truth (ADR-0019).
```

plain format (`--format plain`) 은 한 줄 per hit: `[<type>] <path> — <excerpt>`. 다른 tool 입력용으로 가공.

--file 모드 시 vault side effect 3종 보고 (`created`: query/ 페이지, `appended`: log.md line, `updated`: index.md). JSON 에 `side_effects` key 추가.

## 5. 동작 절차

### 5.1 사전 검증 (validate)

1. `--vault-path` 가 실제 디렉터리인지 확인
2. `~/wiki/AGENTS.md` 존재 확인 (vault 정합 마커)
3. `~/wiki/index.md` 존재 확인 (LLM query 의 첫 reading)
4. `~/wiki/wiki/projects/<project>/` 디렉터리 존재 확인
5. `--project` 가 whitelist (`devhub` | `my-harness`) 인지 확인
6. `--query` 가 비어있지 않은지 확인 (공백만인 경우 error)

### 5.2 source 식별 (collect)

vault 내 4 query primitive 으로 후보 페이지 식별 (per handoff §2.4):

1. **Tag list** — `rg '\#[a-zA-Z0-9_-]+' --only-matching` (rg 부재 시 Python regex fallback)
   - vault 의 모든 wiki 페이지에서 tag 추출 → `--tag` 필터와 AND 매칭
2. **Full-text** — `rg -w '<query>' --line-number --context 1 --json`
   - body 에서 단어 경계 매칭. 대소문자 무시
3. **Wikilink** — `rg '\[\[([^\]|]+)(?:\|[^\]]+)?\]\]' --only-matching`
   - body 의 wikilink 추출 → link target 에 `--query` 포함 시 매칭
4. **Frontmatter** — Python regex 로 8 key 파싱
   - `title` / `type` / `tags` / `sources` / `last_touched` / `related` / `status` / `contradictions`
   - `--type` 필터는 frontmatter `type` 키와 exact match

rg 부재 시 pure Python regex (`re.compile` + `re.findall` on file content) 으로 fallback. 4 primitive 모두 동일 결과 보장. (rg + Python 결과 차이 = max 1, permalink: `wiki-lint` 의 TOML skip 패턴 매칭은 lint 의 권한이지 query 의 권한 아님).

각 후보의 9 sub-key 추출: frontmatter → (title, type, tags, sources, last_touched) / body excerpt (max 2000 chars, 첫 H1 + 첫 paragraph) / wikilinks (body + frontmatter related) / backlinks (index scan: 다른 페이지의 wikilink 가 본 page 의 stem 을 가리키는지).

### 5.3 page 작성 (render, --file 모드 한정)

`--file` 모드에서만 신규 query/ 페이지 작성:

1. `topic` 결정: `--query` 의 kebab-case normalize (소문자, 비영숫자 → `-`, 연속 `-` 1개로 압축)
2. `<date>` 결정: today (UTC, `%Y-%m-%d`)
3. 파일명: `wiki/projects/<project>/query/<date>-<topic>.md`
4. frontmatter 8 key (AGENTS.md §3 정합):
   ```yaml
   ---
   title: "<query> (<date>)"
   type: query
   tags: [<query 토큰>, project-<project>]
   sources: [none]
   last_touched: <date>
   related: [none]
   status: draft
   contradictions: [none]
   ---
   ```
5. body 4섹션 (질문 / 사용 컨텍스트 / 답변 / 후속 액션):
   - **질문**: `--query` 원문
   - **사용 컨텍스트**: 실행 시각 + mode + hit_count
   - **답변**: 후보 페이지 종합 결과 (excerpt 5건 + cross-ref 5건). 본문 끝에 `Filed as [[query/<date>-<topic>]]` 한 줄 (AGENTS.md §2.2 step 3)
   - **후속 액션**: ingest 대상 후보 (sources/ 페이지 부족 시) / 다음 query 권장 / "no hits" 인 경우 후속 액션 = "후속 query 필요"

**idempotency**: 같은 `<date>-<topic>` 파일이 이미 존재하면 skip (warnings 에 "already filed" 추가, side effect 0).

### 5.4 cross-ref 갱신 (cross-link, --file 모드 한정)

1. 신규 query/ 페이지의 `related:` 필드에 hit_count 상위 5건 자동 추가
2. 기존 page 의 related 갱신은 하지 않음 (Query 의 책임 외, Ingest 의 cross-ref 책임 영역)
3. cross-project link 는 자제 (AGENTS.md §11.6, L02 false positive 회피)

### 5.5 index/log 갱신 (manifest, --file 모드 한정)

1. `wiki/projects/<project>/index.md` 의 "Query" 섹션에 한 줄 append:
   ```
   - [<date>-<topic>](query/<date>-<topic>.md) — <query 원문> (<hit_count> hits, 2026-06-11)
   ```
2. `~/wiki/log.md` 에 `## [<date>] query | <topic>` 한 줄 append

**idempotency**:
- `log.md` 의 같은 line 이 이미 있으면 skip (idempotent append)
- `index.md` 의 같은 `[[<date>-<topic>]]` link 가 이미 있으면 skip (idempotent update)

### 5.6 lint (post-query, 선택)

1. `--apply` 와 동등한 lint 호출은 본 skill 의 범위 외 (wiki-lint 의 권한)
2. --file 모드 후 wiki-lint 호출 권장이나, 본 skill 은 lint trigger X (사용자 명시 또는 다음 session-start 의 wiki-lint 통합에 위임)
3. lint L01~L10 의 query/ 페이지 검증은 자동 수행 X (D-74 / D-75 의 lint 통합에 위임)

### 5.7 최종 출력

1. JSON (stdout) — `--output json|both` 일 때
2. Markdown (stdout) — `--output markdown|both` 일 때
3. 종료 코드: errors > 0 이면 1, 0 results + read-only 정상 종료면 0, invalid option 2

## 6. 권한 경계

- **읽기** (read + --file 모두):
  - `<vault>/AGENTS.md`, `<vault>/index.md`, `<vault>/log.md` (vault 운영 마커)
  - `<vault>/raw/projects/<project>/**` (cross-reference 검증)
  - `<vault>/wiki/projects/<project>/{concepts,entities,topics,sources,comparisons,query,meta}/**` (vault read)
  - `<vault>/schema/**` (frontmatter 형식 참조)
  - `<vault>/wiki/cross/**` (cross-project 종합, readonly)
- **쓰기** (--file 모드 한정):
  - `<vault>/wiki/projects/<project>/query/<date>-<topic>.md` (신규)
  - `<vault>/log.md` (append, idempotent)
  - `<vault>/wiki/projects/<project>/index.md` (append query 섹션, idempotent)
- **금칙**:
  - `<vault>/raw/**` 절대 수정 금지 (AGENTS.md §6, mirror 결과)
  - `<vault>/schema/**` 절대 수정 금지
  - 기존 `<vault>/wiki/projects/<project>/{concepts,entities,topics,sources,comparisons,meta}/` 페이지 수정 금지 (Query 의 책임 외, Ingest 의 cross-ref 책임)
  - `<vault>/AGENTS.md` 수정 금지
  - 다른 project 의 `<vault>/wiki/projects/<other>/` 수정 금지 (cross-project는 별도 lint/operation SOP)
  - **자동 vault Gitea push 금지** (사용자 수동, AGENTS.md §6.5 정책)
  - 자동 lint 결과 자동 머지 금지
- 위반 시도 시 `error_code=PERMISSION_DENIED` 로 실패

## 7. 판단 규칙

- `--no-file` 기본 — 실제 query 페이지 file + log.md append + index.md 갱신은 사용자 명시 confirm (`--file`)
- 0 results 도 정상 (exit 0, "0 hits" 보고) — Query 가 vault 의 빈 영역을 드러내는 것도 가치가 있음
- query 가 너무 광범위 (hit_count > limit) 면 limit 적용 + warnings 에 "truncated, N more matches" 명시
- 기존 query/ 페이지와 같은 date+topic 면 idempotent skip (no side effect, exit 0)
- type/tag filter 는 AND (full-text 매칭 + type 매칭 + tag 매칭 모두 통과)
- cross-project match (devhub query 가 my-harness page 매칭) 도 vault 내 검색이므로 허용. 결과 path 에 project prefix 명시

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `--query` 부재
- `--project` whitelist 미준수
- `--format` / `--output` / `--type` enum 미준수
- `~/wiki/AGENTS.md` 부재 (vault 정합 마커)
- `~/wiki/index.md` 부재 (LLM query 의 첫 reading)
- `<vault>/wiki/projects/<project>/` 부재
- `--file` 모드에서 query/ 페이지 write 실패 (permission, encoding)
- raw/ 또는 schema/ 또는 AGENTS.md 수정 시도 (PERMISSION_DENIED)
- 종료 코드 1

### 8.2 경고로 처리할 조건

- hit_count 0 (no matches)
- hit_count > limit (truncated)
- 기존 query/ 페이지와 같은 date+topic (idempotent skip, warnings 에 사유)
- 0-byte wiki page (read-only 모드)
- frontmatter 누락 page (L01 위반 가능성, L08 미등록 가능성)
- cross-project match (다른 project 의 page 도 hit)
- 종료 코드 0

### 8.3 실패 시 최소 출력

- 검사한 wiki/ 디렉터리 목록
- 식별한 후보 page 목록 (preview)
- 실패한 reason (error_code + message)
- 사용자가 수동으로 먼저 확인해야 할 경로 (예: AGENTS.md 부재 → wiki-init SOP)

## 9. 권한과 수정 제한

- 기본 권한 = 읽기 전용. `--file` 시점에만 query/ + log.md + index.md 쓰기.
- raw/ 절대 수정 금지 (SSOT 보호, AGENTS.md §6)
- schema/ 절대 수정 금지
- 기존 wiki/ 페이지(concepts/entities/topics/sources/comparisons/meta/) 절대 수정 금지
- AGENTS.md 절대 수정 금지
- 다른 project 의 wiki/ 절대 수정 금지
- 자동 vault Gitea push 절대 금지 (사용자 수동)
- 자동 lint 결과 자동 머지 절대 금지 (wiki-lint 권고)
- `done` 상태 확정이나 lint 머지 자동화 없음
- 사용자 명시 approve 없이는 `wiki/` 의 기존 page 삭제/덮어쓰기 없음

## 10. 수동 대체 절차

본 skill 이 없거나 미실행 시 수동으로 AGENTS.md §2.2 정공법 6 step:

1. `cat ~/wiki/index.md` — 후보 페이지 식별 (step 1)
2. `cat ~/wiki/wiki/projects/<project>/{concepts,entities,topics,sources,...}/<candidate>.md` — 종합 (step 2)
3. 답변 끝에 `Filed as [[query/<date>-<topic>]]` 한 줄 (step 3)
4. `wiki/projects/<project>/query/<date>-<topic>.md` 신규 작성 (frontmatter 8 key + 4섹션, step 4)
5. `log.md` 에 `## [<date>] query | <topic>` 1 line append (step 5)
6. Obsidian graph view 에서 backlink 확인 (step 6 의 ingest source 가산 가능)
7. dry-run 검증: `python3 skills/wiki-lint/scripts/run_wiki_lint.py --vault-path ~/wiki --project <project>`
8. (선택) `git -C ~/wiki add ... && git -C ~/wiki commit ... && git -C ~/wiki push` (사용자 수동, Gitea remote)

## 11. 구현 체크리스트

- vault 정합 마커 (`AGENTS.md`, `index.md` 존재) 확인하는가
- project whitelist 강제하는가
- 4 query primitive (tag, full-text, wikilink, frontmatter) 을 안정적으로 처리하는가 (rg + pure Python fallback)
- 9 key JSON output schema (ok/query/project/mode/tool_version/examined_at/hit_count/results/warnings/errors) + results 9 sub-key (title/type/tags/path/sources/last_touched/excerpt/links/backlinks) 정확히 채우는가
- `index.md` / `log.md` / query/ 페이지 side effect 시 idempotent 한가 (재실행 시 중복 X)
- raw/ / schema/ / AGENTS.md / 다른 project wiki/ 수정 시도 시 error_code=PERMISSION_DENIED 반환하는가
- `--no-file` mode 가 파일 변경 없는가
- `--file` mode 가 query/ 페이지 + log.md 1 line + index.md 1 line 갱신하는가
- 0 results + read-only 정상 종료 시 exit 0 인가
- 종료 코드 0/1/2 가 errors / invalid option / 정상 종료 와 정합하는가
- **vault Gitea push 자동 호출 절대 금지** (사용자 수동 confirm)
- Python 3.10+ stdlib only (third-party library import X, ripgrep 은 subprocess 로 호출 — 부재 시 pure Python regex fallback)
- 본 저장소 wrapper (`scripts/wiki-query.sh`) 가 skill 부재 시 exit 1 + SSOT 경로 안내 (정공법)

## 다음에 읽을 문서

- skill 카탈로그: [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md)
- skill SKILL: [`../skills/wiki-query/SKILL.md`](../skills/wiki-query/SKILL.md)
- vault 운영 규약: `~/wiki/AGENTS.md` (외부, 이 vault 의 root, §2.2 Query)
- lint 규칙 SSOT: `~/wiki/schema/lint_rules.md` (외부)
- 가장 가까운 precedent (ingest): [`./wiki_ingest_skill_spec.md`](./wiki_ingest_skill_spec.md) (D-72, §1-§11 정합)
- 본 저장소 wrapper: `~/repos/Devhub_example/scripts/wiki-query.sh` (DevHub PR #552, 192 lines, thin wrapper)
