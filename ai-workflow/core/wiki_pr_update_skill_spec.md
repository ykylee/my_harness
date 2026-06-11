# Wiki-PR-Update Skill Spec

- 문서 목적: `wiki-pr-update` skill 의 입력/출력 계약, 동작 순서, 권한 경계, 실패 규칙을 정의한다.
- 범위: GitHub PR 의 메타데이터 + touched file 을 LLM Wiki vault 의 `wiki/projects/<project>/prs/<num>.md` 페이지로 자동 갱신 + `log.md` append + `index.md` 갱신. (--reingest 분기 시 mirror-list 매칭 source 를 `wiki-ingest-from-raw` 로 재투입.)
- 대상 독자: AI agent 설계자, skill 구현자, vault 운영자, DevHub / my_harness 프로젝트 멤버
- 상태: **draft** (D-80, 2026-06-11)
- 최종 수정일: 2026-06-11
- 관련 문서: `~/wiki/AGENTS.md` (vault 운영 규약, §11.1 D-72 cross-project), `~/wiki/schema/lint_rules.md` (L01~L10), [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md), [`./session_start_skill_spec.md`](./session_start_skill_spec.md), [`../skills/wiki-pr-update/SKILL.md`](../skills/wiki-pr-update/SKILL.md)

## 1. 목적

`wiki-pr-update` skill 은 GitHub PR 단위의 메타정보 + touched file 을 LLM Wiki vault 의 `prs/<num>.md` 페이지로 자동 반영한다. **사용자가 "PR 이 머지됐는데 vault 의 PR 페이지가 없다"** 라고 말할 때, 본 skill 은 다음 7 단계를 PR 1건에 대해 일괄 실행한다:

1. `gh pr view <num> --json ...` 의 메타데이터 로드 (title, author, state, mergedAt, headRefOid, files)
2. idempotency key `pr-<num>-<head.sha>` 계산 — 기존 `prs/<num>.md` 의 `last_touched` 와 비교해 skip / re-write 결정
3. `wiki/projects/<project>/prs/<num>.md` 신규 작성 (frontmatter 11+ key + body)
4. cross-ref 갱신 — touched file 중 wiki 에 이미 존재하는 source page 에 `## Related prs` 1줄 append (idempotent)
5. `wiki/projects/<project>/index.md` 갱신 — `prs` 섹션에 한 줄 append
6. `wiki/log.md` 에 `## [YYYY-MM-DD] pr-update | PR #<num>` 1줄 append
7. (선택, `--reingest`) mirror-list 7 patterns 매칭 시 `wiki-ingest-from-raw --source <file> --apply` dispatch

핵심 정공법: **PR = 1 page-per-PR, head.sha = idempotency key, gh CLI = metadata source of truth, vault = 공유 자원 (DevHub/my_harness 2 프로젝트 동거)**, **raw/ = 읽기 전용 (mirror), wiki/ 의 prs/ = LLM 편집**. 본 skill 은 wiki/prs/ 의 LLM 편집 자동화. raw/ 와 schema/ 와 AGENTS.md 는 절대 수정하지 않는다 (AGENTS.md §6). **자동 Gitea remote push 금지** — vault Gitea push 는 사용자 수동 (AGENTS.md §6.5).

## 2. 선행 원칙

- vault 운영 규칙은 `~/wiki/AGENTS.md` (v1.5, D-71) 의 정공법 우선. 본 skill 은 그 §11.1 D-72 cross-project 자동화의 PR 측면.
- 본 skill 의 SSOT = [`./wiki_pr_update_skill_spec.md`](./wiki_pr_update_skill_spec.md). `wiki-ingest-from-raw` (D-72) 와 명확한 책임 분리:
  - `wiki-ingest-from-raw` = raw source 1건 또는 N건 → wiki page 자동화 (raw/ → wiki/)
  - `wiki-pr-update` = PR 1건 → vault prs/ 자동화 (gh API → wiki/) + mirror-list 분기로 `wiki-ingest-from-raw` 재호출 가능
- 본 skill 은 기본적으로 `--dry-run` mode. 실제 적용은 `--apply` 명시 시점.
- idempotency 가 1순위 — 이미 갱신된 PR 은 skip (no side effect, exit 0)
- `gh` CLI 가 PATH 에 있어야 함 (`gh --version` 통과). wrapper 가 metadata JSON 을 file 로 전달할 때 본 impl 은 그 file 만 읽음
- `gh pr view --json` 의 field 는 wrapper 가 명시: `number,title,author,state,mergedAt,headRefOid,files` (`.changedFiles` 미사용)
- mirror-list 7 patterns 매칭은 **wrapper 측이 분기 결정** (handoff §3.3 명시). 본 impl 은 --reingest + source path list 를 받으면 wiki-ingest-from-raw dispatch 만 수행
- frontmatter 누락 상태로 prs/ page 작성 금지 (L01 위반, AGENTS.md §6)
- `index.md` / `log.md` 갱신 누락 금지 (AGENTS.md §6)
- 자동 lint 결과 자동 머지 금지 (AGENTS.md §6, wiki-lint 권고 정공법)
- cross-project link 는 `wiki/cross/` 에서 종합. `wiki/projects/<project>/prs/` 내부의 cross-project link 자제 (AGENTS.md §11.6)

## 3. 입력 계약

### 3.1 필수 입력

- `--pr` (int) — GitHub PR number. 예: `552`
- `--vault-path` (str, default `~/wiki`) — vault 루트 경로
- `--project` (str, default `devhub`) — 대상 project (`devhub` | `my-harness`). AGENTS.md §11.4 정합
- `--pr-metadata` (file) — `gh pr view <num> --json number,title,author,state,mergedAt,headRefOid,files` 출력 JSON file. wrapper 가 전달

### 3.2 선택 입력

- `--touched-files` (file) — `gh pr diff <num> --name-only` output file. wrapper 가 전달. 본 impl 은 metadata JSON 의 `files[].path` 와 교차 검증
- `--apply` (flag, default off = dry-run) — 실제 vault 갱신
- `--reingest` (flag) — touched file 중 mirror-list 7 patterns 매칭 시 `wiki-ingest-from-raw --source <file> --apply` dispatch
- `--quiet` (flag) — stderr 메시지 최소화
- `--output` (enum, default `json`) — `json` | `markdown` | `both`

### 3.3 입력 해석 규칙

- `--pr` 0 이하 또는 non-int → error (`error_code=INVALID_PR`)
- `--project` whitelist 미준수 (`devhub` / `my-harness` 외) → error (`error_code=INVALID_PROJECT`)
- `--pr-metadata` file 부재 또는 JSON 파싱 실패 → error (`error_code=METADATA_READ_FAIL`)
- `--apply` 없으면 실제 파일 변경 없음 — 모든 동작은 preview (idempotency skip 도 preview level)
- `--reingest` 단독 사용 불가 — `--apply` 와 함께 또는 dry-run 의 preview 분기. 단, 분기 결정은 wrapper 측 (본 impl 은 dispatch 만)
- `--touched-files` 와 `files[]` 가 동시 있을 때 동등성 검증 (mismatch 시 warn, 본 impl 은 `files[]` 우선)
- `--vault-path` 가 실제 디렉터리가 아니면 error

## 4. 출력 계약

### 4.1 JSON (stdout)

```json
{
  "ok": true,
  "pr_number": 552,
  "pr_title": "feat(wiki): wiki-ingest-from-raw skill (D-72 Phase 3)",
  "head_sha": "43eb18f0...",
  "vault_path": "/home/yklee/wiki",
  "project": "devhub",
  "mode": "apply",
  "tool_version": "0.1.0",
  "examined_at": "2026-06-11T...",
  "summary": {
    "touched_files": 6,
    "vault_source_files": 0,
    "pages_created": 1,
    "index_md_updates": 1,
    "log_md_appends": 1,
    "idempotent_skip": false
  },
  "created": ["wiki/projects/devhub/prs/552.md"],
  "appended": ["log.md (1 line)"],
  "warnings": [],
  "errors": []
}
```

- `ok` (bool) — `errors` 비고 `ok=true`. `errors` ≥1 이면 `ok=false`
- `pr_number` (int) — 입력 그대로
- `pr_title` (str) — metadata `title`
- `head_sha` (str) — metadata `headRefOid`
- `vault_path` (str) — resolve 된 절대 경로
- `project` (str) — whitelist 통과 후의 project
- `mode` (`dry-run` | `apply`) — `--apply` 유무
- `tool_version` (str) — 본 impl 의 `0.1.0`
- `examined_at` (ISO 8601) — UTC now
- `summary.touched_files` (int) — metadata `files[].path` 의 unique count
- `summary.vault_source_files` (int) — touched files 중 이미 `wiki/projects/<project>/sources/` 또는 `prs/` 에 존재하는 page 수
- `summary.pages_created` (int) — 본 실행에서 신규 작성한 prs/ page 수
- `summary.index_md_updates` (int) — 본 실행에서 갱신한 `index.md` 수 (idempotent skip 면 0)
- `summary.log_md_appends` (int) — 본 실행에서 append 한 `log.md` line 수 (idempotent skip 면 0)
- `summary.idempotent_skip` (bool) — frontmatter `last_touched >= head.sha` 으로 skip 했으면 `true`
- `created` (list[str]) — 본 실행에서 신규 작성한 vault 경로
- `appended` (list[str]) — 본 실행에서 append 한 파일/line 표기
- `warnings` (list[str]) — L01~L10 위반 의심 / cross-ref 매칭 0건 / title 충돌 / gh CLI 일부 field 부재
- `errors` (list[str]) — 실패 원인. error_code 와 함께 권장

### 4.2 Markdown (vault `_lint/<project>/pr_update_YYYY-MM-DD.md`)

```markdown
# PR Update Report — 2026-06-11

- vault: ~/wiki
- project: devhub
- PR: #552 — feat(wiki): wiki-ingest-from-raw skill (D-72 Phase 3)
- head.sha: 43eb18f0...
- mode: apply
- 검사 시각: 2026-06-11 ...
- 검사자: wiki-pr-update 0.1.0
- 결과: 1 page created, 0 errors

## Preview
| pr_number | title | head_sha | action | target_page |
| --- | --- | --- | --- | --- |
| 552 | feat(wiki): wiki-ingest-from-raw skill (D-72 Phase 3) | 43eb18f0... | create | wiki/projects/devhub/prs/552.md |

## Touched files
- scripts/wiki-pr-update.sh
- docs/llm-wiki/pr-update-skill.md
- ...
```

## 5. 동작 절차

### 5.1 사전 검증 (validate)

1. `--vault-path` 가 실제 디렉터리인지 확인
2. `~/wiki/AGENTS.md` 존재 확인 (vault 정합 마커)
3. `~/wiki/wiki/projects/<project>/prs/` 디렉터리 없으면 생성 시도 (--apply 시점, 권한 부재 시 error)
4. `~/wiki/wiki/projects/<project>/index.md` 존재 확인 (없으면 error)
5. `~/wiki/log.md` 존재 확인 (없으면 error)
6. `--project` whitelist (`devhub` | `my-harness`) 강제
7. `--pr-metadata` file JSON 파싱 + 필수 field (`number`, `title`, `state`, `headRefOid`) 존재 확인
8. `gh --version` PATH 확인 (warn 만, --reingest 분기 시점에 gh 호출은 wiki-ingest-from-raw 가 담당)

### 5.2 source 식별 (collect)

1. `metadata.files[]` 의 path list 추출 (중복 제거, sort)
2. 각 path 의 분류:
   - `vault_source_existing`: 이미 `wiki/projects/<project>/sources/<title>.md` 존재
   - `vault_pr_existing`: 이미 `wiki/projects/<project>/prs/<title>.md` 존재 (다른 PR 의 page)
   - `raw_unmapped`: `raw/projects/<project>/<path>` 의 mirror list 매칭 (--reingest 분기에서 wiki-ingest-from-raw 후보)
   - `other`: 그 외 (변환/스크립트 등 — 본 PR 의 prs/ body 에 "touched files" 로 기재만)
3. `--touched-files` file 이 있으면 위 list 와 동등성 비교 (mismatch 시 warn, 본 impl 은 `metadata.files[]` 우선)

### 5.3 page 작성 (render)

신규 PR page 작성 시:

1. `title`: `PR #<num>: <metadata.title>`
2. `type`: `pr` (AGENTS.md §3 정합)
3. `tags`: `[pr, project-<project>, <metadata.state>]` (예: `[pr, project-devhub, merged]`)
4. `pr_number`: `<metadata.number>`
5. `author`: `<metadata.author.login>` (없으면 `unknown`)
6. `state`: `<metadata.state>` (`OPEN` | `MERGED` | `CLOSED`)
7. `merged_at`: `<metadata.mergedAt>` (없으면 `null`)
8. `head_sha`: `<metadata.headRefOid>` (idempotency key 의 head 부분)
9. `sources`: `[<각 touched file 의 repo-relative path>]` (예: `scripts/wiki-pr-update.sh`)
10. `last_touched`: 오늘 (UTC, ISO date)
11. `related`: `[<vault_source_existing 의 title 들>]` (cross-ref 후보)
12. `status`: `draft` (사용자 검토 후 `reviewed` 승격)
13. `contradictions`: `[none]` (placeholder)
14. `body`:
    - 첫 H1 → page title (frontmatter 와 동일)
    - 한 줄 summary: PR 의 의도 (touched file 0건이면 metadata 의 title 만)
    - `## Touched files` 섹션: touched file list
    - `## State / Author / Head SHA` 메타 block
    - `## Related sources` 섹션: vault_source_existing 의 wikilink

### 5.4 cross-ref 갱신 (cross-link)

1. 각 `vault_source_existing` 의 page 읽기
2. body 끝에 `## Related prs` 섹션이 있으면 그 안에 `[[pr-<num>]]` 1줄 append (중복 방지)
3. 없으면 `## Related prs` 섹션 새로 생성 + 1줄 append
4. 매칭 0건이면 warn (`L08` 의심), 본 impl 은 자동 생성 안 함

### 5.5 index/log 갱신 (manifest)

1. `wiki/projects/<project>/index.md` 의 "PRs" 섹션에 한 줄 append:
   ```
   - [PR #552](prs/552.md) — feat(wiki): wiki-ingest-from-raw skill (D-72 Phase 3) (2026-06-11)
   ```
2. `wiki/log.md` 에 `## [YYYY-MM-DD] pr-update | PR #<num> | <title>` 한 줄 append (project 명시)
3. 두 append 모두 idempotent — 같은 PR number 의 같은 line 이 이미 있으면 skip

### 5.6 lint (post-ingest)

1. `--apply` 시 본 skill 완료 후 자동으로 `wiki-lint --project=<project>` 호출 권장 (wrapper 측 옵션)
2. lint exit 0 (clean) → success
3. lint exit 1 (findings) → warnings 에 lint 결과 추가, page 는 유지 (자동 머지 X, AGENTS.md §6)
4. 본 impl 은 lint 호출하지 않음 — wrapper / 호출자 측 결정

### 5.7 최종 출력

1. JSON (stdout)
2. Markdown report (`_lint/<project>/pr_update_YYYY-MM-DD.md`) — `--output markdown|both` 일 때
3. 종료 코드: `errors > 0` 이면 1, 아니면 0 (idempotent skip 은 exit 0)

## 6. 권한 경계

- **읽기**: `<vault>/wiki/projects/<project>/**` (기존 page 읽기, cross-ref 후보), `<vault>/wiki/index.md`, `<vault>/wiki/log.md`, `<vault>/_lint/<project>/pr_update_*.md`
- **쓰기 (--apply mode)**: `<vault>/wiki/projects/<project>/prs/<num>.md` (신규), `<vault>/wiki/projects/<project>/sources/<*.md>` 의 `## Related prs` 섹션 (cross-ref, idempotent append), `<vault>/wiki/projects/<project>/index.md` (PRs 섹션 append), `<vault>/wiki/log.md` (append, idempotent), `<vault>/_lint/<project>/pr_update_*.md`
- **금칙**: `<vault>/raw/**` 절대 수정 금지 (mirror 결과, AGENTS.md §6), `<vault>/schema/**` 절대 수정 금지, `<vault>/wiki/AGENTS.md` 절대 수정 금지 (AGENTS.md §6), 다른 project 의 `wiki/projects/<other>/` 절대 수정 금지 (cross-project 금지, AGENTS.md §11.6), **자동 Gitea remote push 금지** (사용자 수동, AGENTS.md §6.5)
- **gh CLI 호출**: 본 impl 은 직접 호출하지 않음. **wrapper 가 `gh pr view --json ...` 결과를 file 로 전달**하는 정공법 (my_harness impl 은 gh CLI 호출 의존도만 표기, 실제 dispatch 는 wrapper). `--reingest` 분기에서 mirror-list 매칭 시 wrapper 가 `wiki-ingest-from-raw --source <file> --apply` 를 dispatch (본 impl 은 `--reingest` flag 와 source path list 만 출력)
- 위반 시도 시 `error_code=PERMISSION_DENIED` 로 실패

## 7. 판단 규칙

- `--dry-run` 기본 — 실제 적용은 사용자 명시 confirm
- idempotency 가 1순위 — `prs/<num>.md` 가 존재하고 `last_touched >= head.sha` 이면 skip (no side effect, exit 0)
- force-push / rebase 로 head.sha 가 바뀌면 **idempotent re-write** (frontmatter `last_touched` + `head_sha` 갱신, body 는 touched file 차이분만 patch)
- `log.md` / `index.md` 의 같은 line 이 있으면 skip (idempotent append)
- cross-ref 의 `## Related prs` append 가 중복되면 skip
- 1 PR 의 touched file 이 0건이면 (e.g. draft PR 의 metadata-only update) page 는 작성하되 touched_files=0, vault_source_files=0, body 의 `## Touched files` 섹션 생략
- `--limit` 옵션은 v1 에 미지원 (PR 단위 작업이므로 1 PR 1 page)
- cross-project link 는 자제 — 본 PR 의 project 가 `devhub` 면 wiki/devhub/prs/ 에 작성, my-harness link 필요 시 `wiki/cross/` 에서 종합

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `--vault-path` 부재 또는 디렉터리 아님
- `--project` whitelist 미준수
- `--pr` 0 이하 또는 non-int
- `--pr-metadata` file 부재 또는 JSON 파싱 실패
- metadata 의 `headRefOid` 부재
- `~/wiki/AGENTS.md` 부재 (vault 정합 마커)
- `wiki/projects/<project>/index.md` 또는 `log.md` 부재
- 출력 디렉터리 생성 실패 (permission, --apply 시점)
- cross-ref 의 source page 쓰기 실패 (--apply 시점, permission)

### 8.2 경고로 처리할 조건

- `touched-files` file 과 `metadata.files[]` 의 mismatch
- metadata `author.login` 부재 (anonymous 으로 fallback)
- metadata `mergedAt` 부재 + state=`MERGED` (이상치)
- cross-ref 매칭 0건 (L08 의심)
- title 충돌 (이미 `prs/<num>.md` 존재하지만 `last_touched < head.sha` 인 re-write 케이스 외는 거의 없음)
- gh CLI PATH 부재 (본 impl 은 직접 호출 X, warn 만)
- `--reingest` + mirror-list 매칭 0건 (no-op)
- lint L01~L10 위반 (post-ingest, wrapper 가 호출 시)

### 8.3 실패 시 최소 출력

- 검사한 PR metadata 의 key 목록
- 시도한 page 경로 (preview)
- 실패한 단계 (validate / render / cross-link / manifest) 와 원인
- 사람이 수동으로 먼저 확인해야 할 경로 (raw/_manifest.md, prs/<num>.md 등)

## 9. 권한과 수정 제한

- 기본 권한 = 읽기 전용 + 미리 보기. `--apply` 시점에만 쓰기.
- raw/ 절대 수정 금지 (SSOT 보호, AGENTS.md §6)
- schema/ 절대 수정 금지
- `AGENTS.md` 절대 수정 금지 (vault 운영 규약 자체)
- 다른 project 의 wiki 절대 수정 금지 (cross-project 금지, AGENTS.md §11.6)
- `done` 상태 확정이나 lint 머지 자동화 없음
- 사용자 명시 approve 없이는 `prs/<num>.md` 삭제/덮어쓰기 없음
- **자동 Gitea remote push 금지** — vault Gitea push 는 사용자 수동 (AGENTS.md §6.5)
- **wrapper 의 gh pr view --json dispatch 정공법** — my_harness impl 은 gh CLI 호출 의존을 명시만, 실제 호출은 wrapper 가 담당 (D-72 §11.1 thin-wrapper 원칙)

## 10. 수동 대체 절차

본 skill 이 없거나 미실행 시 수동으로 PR 페이지 작성 정공법 (AGENTS.md §11.1 D-72 cross-project 정합):

1. `gh pr view <num> --json number,title,author,state,mergedAt,headRefOid,files` — metadata 확보
2. `gh pr diff <num> --name-only` — touched file list
3. `ls wiki/projects/<project>/prs/` — 기존 PR page 확인 (idempotency check)
4. 기존 page 의 `last_touched` 와 head.sha 비교 — 같으면 skip, 다르면 re-write
5. `wiki/projects/<project>/prs/<num>.md` 신규 작성 (frontmatter 11+ key + body)
6. touched file 중 vault source page 가 있으면 `## Related prs` 1줄 append
7. `wiki/projects/<project>/index.md` 의 "PRs" 섹션에 한 줄 append
8. `wiki/log.md` 에 `## [YYYY-MM-DD] pr-update | PR #<num> | <title>` 1줄 append
9. dry-run 검증: `python3 skills/wiki-lint/scripts/run_wiki_lint.py --vault-path ~/wiki --project <project>`
10. (선택) `--reingest`: touched file 중 mirror-list 7 patterns 매칭 시 `wiki-ingest-from-raw --source <file> --apply` 재실행
11. (선택) Obsidian 에서 graph view 확인
12. (선택) vault Gitea remote push (사용자 수동, AGENTS.md §6.5)

## 11. 구현 체크리스트

- vault 정합 마커 (`AGENTS.md` 존재) 확인하는가
- project whitelist 강제하는가
- `--pr` 0 이하 / non-int 거부하는가
- `--pr-metadata` JSON 파싱 실패 시 `error_code=METADATA_READ_FAIL` 반환하는가
- idempotency key `pr-<num>-<head.sha>` 계산 후 skip / re-write 정확히 분기하는가
- frontmatter 11+ key (title/type/tags/pr_number/author/state/merged_at/head_sha/sources/last_touched/related/status/contradictions) 모두 채우는가
- `prs/<num>.md` 의 body 가 H1 + summary + touched files + state block + related sources 인가
- `## Related prs` cross-ref append 가 idempotent 한가 (재실행 시 중복 X)
- `index.md` / `log.md` 갱신 시 중복 append 방지하는가
- raw/, schema/, AGENTS.md, 다른 project 의 wiki/ 수정 시도 시 `error_code=PERMISSION_DENIED` 반환하는가
- `--dry-run` mode 가 파일 변경 없는가
- `--reingest` flag 출력은 dispatch 후보 source path list 만 (실제 호출은 wrapper / wiki-ingest-from-raw 담당)
- 종료 코드 0/1 이 errors 와 정합하는가 (idempotent skip 도 exit 0)
- gh CLI 직접 호출 안 하는가 (wrapper 정공법)
- 자동 Gitea push 시도 안 하는가 (사용자 수동, AGENTS.md §6.5)
- 출력 JSON 이 13 key (ok / pr_number / pr_title / head_sha / vault_path / project / mode / tool_version / examined_at / summary / created / appended / warnings / errors) 모두 포함하는가
- summary 의 6 sub-key (touched_files / vault_source_files / pages_created / index_md_updates / log_md_appends / idempotent_skip) 모두 포함하는가
- `_lint/<project>/pr_update_YYYY-MM-DD.md` Markdown report 가 `--output markdown|both` 일 때만 생성되는가

## 다음에 읽을 문서

- D-72 가장 가까운 precedent: [`./wiki_ingest_skill_spec.md`](./wiki_ingest_skill_spec.md)
- D-79 counterpart: [`./wiki_query_skill_spec.md`](./wiki_query_skill_spec.md) (별도 agent 진행 중, cross-ref)
- skill 카탈로그: [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md)
- vault 운영 규약: `~/wiki/AGENTS.md` (외부, 이 vault 의 root, §11.1 D-72 cross-project)
- lint 규칙 SSOT: `~/wiki/schema/lint_rules.md` (외부, L01~L10)
- 본 skill SKILL: [`../skills/wiki-pr-update/SKILL.md`](../skills/wiki-pr-update/SKILL.md)
- 본 skill impl: [`../skills/wiki-pr-update/scripts/run_wiki_pr_update.py`](../skills/wiki-pr-update/scripts/run_wiki_pr_update.py)
- D-79 (wiki-query) wrapper 정공법 — DevHub thin-wrapper, my_harness SSOT (handoff §2)
