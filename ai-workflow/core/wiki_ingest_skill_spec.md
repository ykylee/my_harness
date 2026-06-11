# Wiki-Ingest-from-Raw Skill Spec

- 문서 목적: `wiki-ingest-from-raw` skill 의 입력/출력 계약, 동작 순서, 권한 경계, 실패 규칙을 정의한다.
- 범위: LLM Wiki vault (`~/wiki/`) 의 `raw/projects/<project>/` 의 source file 을 읽어 `wiki/projects/<project>/sources/<title>.md` 페이지 자동 작성 + cross-ref 갱신 + `index.md`/`log.md` 정합.
- 대상 독자: AI agent 설계자, skill 구현자, vault 운영자, DevHub / my_harness 프로젝트 멤버
- 상태: **draft** (D-72, 2026-06-11)
- 최종 수정일: 2026-06-11 (D-72 Phase 1 wiki-sync 정공법 정합)
- 관련 문서: `~/wiki/AGENTS.md` (vault 운영 규약, §2.1 Ingest), `~/wiki/schema/lint_rules.md` (L01~L10), [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md), [`./session_start_skill_spec.md`](./session_start_skill_spec.md), `../skills/wiki-lint/SKILL.md`

## 1. 목적

`wiki-ingest-from-raw` skill 은 LLM Wiki vault 의 Ingest flow (AGENTS.md §2.1) 를 자동화한다. **사용자가 "raw 만 갱신되고 wiki 는 업데이트가 안됐다"** 라고 말할 때, 본 skill 은 다음 6 단계를 raw file 1건 또는 N건에 대해 일괄 실행한다:

1. `raw/projects/<project>/<rel_path>` 의 source file 읽기
2. `wiki/projects/<project>/sources/<title>.md` page-per-source 1건 작성 (frontmatter + body)
3. 관련 `wiki/projects/<project>/{concepts,entities,topics}/` 페이지 갱신 (cross-ref)
4. `wiki/projects/<project>/index.md` 갱신 (해당 페이지 한 줄 요약 + 카테고리 분류)
5. `wiki/log.md` 에 `## [YYYY-MM-DD] ingest | <title>` 한 줄 append
6. (선택) `wiki/projects/<project>/comparisons/` cross-source 분석 placeholder

핵심 정공법: **vault = 공유 자원 (DevHub/my_harness 2 프로젝트 동거)**, **raw/ = 읽기 전용 (mirror), wiki/ = LLM 편집**. 본 skill 은 wiki/ 의 LLM 편집 자동화. raw/ 는 절대 수정하지 않는다 (AGENTS.md §6).

## 2. 선행 원칙

- vault 운영 규칙은 `~/wiki/AGENTS.md` (v1.5, D-71) 의 정공법 우선. 본 skill 은 그 §2.1 Ingest 자동화.
- 본 skill 의 SSOT = [`./wiki_ingest_skill_spec.md`](./wiki_ingest_skill_spec.md). raw 의 `wiki-sync-devhub.sh` / `wiki-sync-ai-workflow.sh` 와 명확한 책임 분리:
  - `wiki-sync-*` = raw mirror 자동화 (out-of-repo → raw/)
  - `wiki-ingest-from-raw` = wiki page 자동화 (raw/ → wiki/)
- 본 skill 은 기본적으로 `--dry-run` mode. 실제 ingest 는 `--apply` 명시 시점.
- frontmatter 누락 상태로 wiki page 작성 금지 (L01 위반, AGENTS.md §6).
- `index.md` / `log.md` 갱신 누락 금지 (AGENTS.md §6).
- 자동 lint 결과 자동 머지 금지 (AGENTS.md §6, wiki-lint 권고 정공법).
- cross-project link 는 `wiki/cross/` 에서 종합. `wiki/projects/<project>/` 내부의 cross-project link 자제 (AGENTS.md §11.6).

## 3. 입력 계약

### 3.1 필수 입력

- `--vault-path` (기본: `~/wiki`) — vault 루트 경로
- `--project` (필수) — 대상 project (`devhub` | `my-harness` | `cross`). AGENTS.md §11.4 정합

### 3.2 선택 입력

- `--source` — 1건 ingest 시 raw/ 상대 경로 (예: `docs/adr/0001-idp-selection.md`)
- `--all` — `raw/projects/<project>/` 의 모든 source 일괄 ingest (Phase 3 mass ingest)
- `--limit N` — `--all` 사용 시 최대 N건만 ingest (default = 무제한)
- `--apply` — 실제 ingest (default = `--dry-run` 미적용, 미리 보기만)
- `--output` — `json|markdown|both` (default = `both`)
- `--quiet` — stderr 메시지 최소화
- `--skip-lint` — 적용 후 wiki-lint skip (CI / 빠른 ingest 용)

### 3.3 입력 해석 규칙

- `--source` 와 `--all` 동시 지정 시 error (둘 중 하나만)
- `--project` 가 `cross` 면 raw 의 `raw/_cross/` 또는 `raw/cross/*` 만 대상 (v1.5 범위 외, planned)
- `--apply` 없으면 실제 파일 변경 없음 — 모든 동작은 preview
- `--limit` 는 `--all` 없이도 사용 가능 (preview 의 부분 적용)

## 4. 출력 계약

### 4.1 JSON (stdout)

```json
{
  "status": "ok",
  "tool_version": "0.1.0",
  "vault_path": "/Users/yklee/wiki",
  "project": "devhub",
  "mode": "dry-run",
  "examined_at": "2026-06-11T01:50:00",
  "summary": {
    "sources_total": 82,
    "sources_already_ingested": 23,
    "sources_to_ingest": 59,
    "pages_to_create": 59,
    "pages_to_update": 12,
    "index_md_updates": 1,
    "log_md_appends": 59
  },
  "findings": [
    {
      "rule": "INGEST-01",
      "severity": "info",
      "source_path": "raw/projects/devhub/docs/adr/0001-idp-selection.md",
      "target_page": "wiki/projects/devhub/sources/adr-0001.md",
      "action": "create",
      "preview": "---\ntitle: ADR-0001 idp-selection\n..."
    }
  ],
  "warnings": [
    "L07 모순 가능: 'ADR-0001 idp-selection' 의 기존 wiki page 가 status=reviewed 상태입니다. --apply 시 lint 후 머지."
  ],
  "errors": []
}
```

### 4.2 Markdown (vault `_lint/<project>/ingest_YYYY-MM-DD.md`)

```markdown
# Ingest Report — 2026-06-11

- vault: ~/wiki
- project: devhub
- mode: dry-run
- 검사 시각: 2026-06-11 01:50
- 검사자: wiki-ingest-from-raw 0.1.0
- 결과: 59 sources to ingest, 0 errors

## Preview
| source_path | target_page | action | title |
| --- | --- | --- | --- |
| raw/projects/devhub/docs/adr/0001-idp-selection.md | wiki/projects/devhub/sources/adr-0001.md | create | ADR-0001 idp-selection |
| ... |
```

## 5. 동작 절차

### 5.1 사전 검증 (validate)

1. `--vault-path` 가 실제 디렉터리인지 확인
2. `~/wiki/AGENTS.md` 존재 확인 (vault 정합 마커)
3. `~/wiki/raw/projects/<project>/` 디렉터리 존재 확인
4. `~/wiki/raw/projects/<project>/_manifest.md` 의 가장 최근 timestamp 확인
5. `--project` 가 whitelist (`devhub` | `my-harness` | `cross`) 인지 확인

### 5.2 source 식별 (collect)

1. `--source` 지정 시: 1 file 만 대상
2. `--all` 지정 시: `raw/projects/<project>/` 의 모든 file (mirror list 의 7 패턴에 한정하지 않음, 모든 file)
3. `--limit` 적용
4. 각 source 의 status 분류:
   - `already_ingested`: `wiki/projects/<project>/sources/<title>.md` 이미 존재
   - `to_ingest`: 위 page 미존재
   - `skipped`: 0-byte file 또는 frontmatter 미준수 raw

### 5.3 page 작성 (render)

각 `to_ingest` source 에 대해:

1. `title` 결정: file 의 frontmatter `title` (있으면) 또는 파일명 stem (kebab-case)
2. `type` 결정: 기본 `source`. source 가 ADR/Governance/Planning/Setup/Requirements/OpenAPI 등 도메인별 자동 분류:
   - `docs/adr/0[0-9][0-9][0-9]-*.md` → `source` (단, ADR 정합 마커)
   - `docs/governance/*.md` → `topic` (운영 정책)
   - `docs/planning/*.md` → `topic` (계획/전략)
   - `docs/setup/*.md` → `topic` (운영 SOP)
   - `docs/requirements.md` → `concept` (요구사항 SSOT)
   - `docs/openapi.yaml` → `source` (API contract)
   - `ai-workflow/memory/{state.json,session_handoff.md,work_backlog.md}` → `topic` (워크플로우 운영)
3. `tags` 결정: file 의 첫 H1 (있으면) 또는 directory 기반 자동 태그
4. `sources` 결정: raw 의 상대 경로 (frontmatter `sources:` 필드, AGENTS.md §3)
5. `related` 결정: cross-ref 검색 — 같은 project 의 다른 source page 중 keyword 매칭 상위 5건
6. `body` 결정:
   - 첫 H1 → page title
   - 첫 paragraph → 1-line summary
   - 1-3 H2 → outline (목차)
   - 원본 raw 의 본문 일부 발췌 (max 2000 chars) — 전체 복사 X, 요약 + 발췌
7. `last_touched`: 오늘 (UTC)
8. `status`: `draft` (사용자 검토 후 `reviewed` 승격)

### 5.4 cross-ref 갱신 (cross-link)

1. 각 `to_ingest` page 의 tags/keywords 추출
2. 같은 project 의 `wiki/projects/<project>/{concepts,entities,topics}/` page 의 body 에서 동일 keyword 검색
3. 매칭 page 의 body 에 `## Related sources` 섹션 자동 append (또는 생성):
   ```markdown
   ## Related sources
   - [[adr-0001-idp-selection]] (ADR-0001)
   - [[adr-0019-keycloak-only-idp]] (ADR-0019)
   ```
4. 매칭 0건이면 warn

### 5.5 index/log 갱신 (manifest)

1. `wiki/projects/<project>/index.md` 의 "Sources" 섹션에 한 줄 append:
   ```
   - [adr-0001-idp-selection](sources/adr-0001-idp-selection.md) — idp selection 결정 (2026-06-11)
   ```
2. `wiki/log.md` 에 `## [YYYY-MM-DD] ingest | <title>` 한 줄 append (project 명시)

### 5.6 lint (post-ingest)

1. `--apply` 시 ingest 완료 후 자동으로 `wiki-lint --project=<project>` 호출
2. lint exit 0 (clean) → success
3. lint exit 1 (findings) → warnings 에 lint 결과 추가, page 는 유지 (자동 머지 X, AGENTS.md §6)
4. `--skip-lint` 면 lint skip

### 5.7 최종 출력

1. JSON (stdout)
2. Markdown report (`_lint/<project>/ingest_YYYY-MM-DD.md`) — `--output markdown|both` 일 때
3. 종료 코드: errors > 0 이면 1, 아니면 0

## 6. 권한 경계

- **읽기**: `<vault>/raw/projects/<project>/**` (source 읽기), `<vault>/wiki/projects/<project>/**` (기존 page 읽기), `<vault>/index.md`, `<vault>/log.md`
- **쓰기**: `<vault>/wiki/projects/<project>/sources/*.md`, `<vault>/wiki/projects/<project>/concepts|entities|topics/*.md` (cross-ref), `<vault>/wiki/projects/<project>/index.md`, `<vault>/wiki/log.md`, `<vault>/_lint/<project>/ingest_*.md`
- **금칙**: `<vault>/raw/**` 절대 수정 금지 (mirror 결과), `<vault>/schema/**` 절대 수정 금지, 자동 lint 결과 자동 머지 금지
- 위반 시도 시 `error_code=PERMISSION_DENIED` 로 실패

## 7. 판단 규칙

- `--dry-run` 기본 — 실제 적용은 사용자 명시 confirm
- 이미 ingest 된 source 는 skip (idempotent)
- file 변경 없이 cross-ref 만 갱신 가능 (`--cross-ref-only` v1.1 planned)
- `wiki/projects/<project>/sources/` 의 기존 page 와 title 충돌 시 warn (덮어쓰기 X)
- `--limit` 은 preview 의 부분 적용 + 실제 적용 모두 지원

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `--vault-path` 부재
- `--project` whitelist 미준수
- `--source` + `--all` 동시 지정
- raw/ 또는 wiki/ 의 필수 디렉터리 부재
- `~/wiki/AGENTS.md` 부재 (vault 정합 마커)
- raw file 읽기 실패 (permission, encoding)
- 출력 디렉터리 생성 실패 (permission)

### 8.2 경고로 처리할 조건

- 이미 ingest 된 source
- 0-byte raw file
- cross-ref 매칭 0건
- title 충돌 (기존 page 존재)
- lint L01~L10 위반 (post-ingest)
- 1 project 의 source 가 0건 (`--all` + project 부재)

### 8.3 실패 시 최소 출력

- 검사한 raw file 목록
- ingest 시도한 page 목록 (preview)
- 실패한 source 목록과 원인
- 사람이 수동으로 먼저 확인해야 할 경로

## 9. 권한과 수정 제한

- 기본 권한 = 읽기 전용 + 미리 보기. `--apply` 시점에만 쓰기.
- raw/ 절대 수정 금지 (SSOT 보호, AGENTS.md §6)
- schema/ 절대 수정 금지
- `done` 상태 확정이나 lint 머지 자동화 없음
- 사용자 명시 approve 없이는 `wiki/` 의 기존 page 삭제/덮어쓰기 없음
- 자동 lint 결과 자동 머지 금지 (AGENTS.md §6, wiki-lint 권고)

## 10. 수동 대체 절차

본 skill 이 없거나 미실행 시 수동으로 AGENTS.md §2.1 정공법:

1. `cat raw/_manifest.md` — 가장 최근 mirror 시점 확인
2. `ls raw/projects/<project>/` — source list
3. `cat wiki/projects/<project>/index.md` — 기존 page 목록
4. 1 file 당:
   - `wiki/projects/<project>/sources/<title>.md` 신규 작성 (frontmatter + body)
   - cross-ref 갱신
   - `index.md` 1줄 append
   - `log.md` 1줄 append
5. dry-run 검증: `python3 skills/wiki-lint/scripts/run_wiki_lint.py --vault-path ~/wiki --project <project>`
6. (선택) Obsidian 에서 graph view 확인

## 11. 구현 체크리스트

- vault 정합 마커 (`AGENTS.md` 존재) 확인하는가
- project whitelist 강제하는가
- source 식별 (--source / --all / --limit) 을 안정적으로 처리하는가
- page 작성 시 frontmatter 필수 필드 (title/type/tags/sources/last_touched/related/status) 모두 채우는가
- cross-ref 의 `## Related sources` 섹션이 idempotent 한가 (재실행 시 중복 X)
- `index.md` / `log.md` 갱신 시 중복 append 방지하는가
- `raw/` 수정 시도 시 error_code=PERMISSION_DENIED 반환하는가
- `--dry-run` mode 가 파일 변경 없는가
- `--apply` mode 가 lint 호출 후 findings 를 warnings 에 추가하는가
- 종료 코드 0/1 이 findings/errors 와 정합하는가

## 다음에 읽을 문서
- skill 카탈로그: [`./workflow_skill_catalog.md`](./workflow_skill_catalog.md)
- vault 운영 규약: `~/wiki/AGENTS.md` (외부, 이 vault 의 root)
- lint 규칙 SSOT: `~/wiki/schema/lint_rules.md` (외부)
- wiki-lint skill: [`../skills/wiki-lint/SKILL.md`](../skills/wiki-lint/SKILL.md)
- session-start skill: [`./session_start_skill_spec.md`](./session_start_skill_spec.md)
