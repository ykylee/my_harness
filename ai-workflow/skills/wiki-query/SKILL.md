---
name: wiki-query
description: "LLM Wiki vault (`~/wiki/`) 의 read-only query + (--file 시) query/ 페이지 신규 + log.md append + index.md 갱신. DevHub/my_harness 2 project 동거 vault 의 §2.2 Query 6 step 자동화."
status: draft
created: 2026-06-11
decision_id: D-79
---

# Wiki-Query Skill

## 1. 목적

`wiki-query` skill 은 `~/wiki/` Obsidian vault 에서 사용자 query 1건을 처리한다. Read-only 가 기본이고, `--file` 시 vault 에 3종 side effect (query/ 페이지 + log.md 1 line + index.md 1 line) 만 발생. 자동 Gitea push 금지.

## 2. 연결 스펙

- 상세 스펙: [../../core/wiki_query_skill_spec.md](../../core/wiki_query_skill_spec.md) (D-79, §1-§11 verbatim)
- 가장 가까운 precedent: [../../core/wiki_ingest_skill_spec.md](../../core/wiki_ingest_skill_spec.md) (D-72, §1-§11 정합)
- vault 운영 규약: `~/wiki/AGENTS.md` v1.5 (D-71) §2.2 Query 6 step + §3 frontmatter 8 key + §6 금지
- 본 저장소 wrapper (DevHub): `~/repos/Devhub_example/scripts/wiki-query.sh` (192 lines, thin wrapper)

## 3. 예상 입력

- `--query <text>` (필수)
- `--vault-path` (기본: `~/wiki`)
- `--project` (기본: `devhub`) — `devhub` | `my-harness`
- `--tag` (선택, frontmatter tags: AND 필터)
- `--type` (선택, `concept` | `entity` | `topic` | `source` | `comparison` | `query`)
- `--limit` (int, 기본 20, 0 이하 = 무제한)
- `--format` (선택, `md` | `json` | `plain`, 기본 `md`)
- `--file` (flag, 기본 off) — query/ 페이지 신규 + log.md append + index.md 갱신
- `--quiet` (flag, stderr 최소화)
- `--output` (선택, `json` | `markdown` | `both`, 기본 `json`)

## 4. 예상 출력

**JSON (9 key)**: `ok` / `query` / `project` / `mode` / `tool_version` / `examined_at` / `hit_count` / `results` / `warnings` / `errors`.

**results[] (9 sub-key)**: `title` / `type` / `tags` / `path` / `sources` / `last_touched` / `excerpt` / `links` / `backlinks`.

**Markdown**: `# Query: "<text>"` 헤더 + `## Hits` + 각 hit 의 `### [[<title>]]` + frontmatter + excerpt.

**종료 코드**: 0 (정상, 0 results 포함) / 1 (error) / 2 (invalid option).

## 5. 권한 경계

- **읽기**: `<vault>/AGENTS.md`, `<vault>/index.md`, `<vault>/log.md`, `<vault>/raw/projects/<project>/**`, `<vault>/wiki/projects/<project>/{concepts,entities,topics,sources,comparisons,query,meta}/**`, `<vault>/schema/**`, `<vault>/wiki/cross/**`
- **쓰기** (--file 한정): `<vault>/wiki/projects/<project>/query/<date>-<topic>.md` (신규), `<vault>/log.md` (idempotent append), `<vault>/wiki/projects/<project>/index.md` (idempotent update)
- **금칙**: `<vault>/raw/**` / `<vault>/schema/**` / `<vault>/AGENTS.md` / 다른 project `<vault>/wiki/projects/<other>/` / 기존 read-only 영역 (concepts/entities/topics/sources/comparisons/meta) / **자동 Gitea push**
- 위반 시도 시 `error_code=PERMISSION_DENIED` 로 실패

## 6. 구현 메모

- **Python 3.10+ stdlib only** (`argparse`, `json`, `re`, `subprocess`, `pathlib`, `dataclasses`, `datetime`). third-party library import X
- **4 query primitive** (handoff §2.4): ripgrep via subprocess (preferred) + pure Python regex fallback (rg 부재 시)
  1. Tag list — `rg '\#[a-zA-Z0-9_-]+' --only-matching` / Python `re.findall`
  2. Full-text — `rg -w '<query>' --line-number --context 1 --json` / Python `re.search` (단어 경계)
  3. Wikilink — `rg '\[\[([^\]|]+)(?:\|[^\]]+)?\]\]' --only-matching` / Python `re.findall`
  4. Frontmatter — Python regex `^---\r?\n([\s\S]*?)\r?\n---\r?\n?` → 8 key 파싱
- **idempotency** (--file): 같은 `<date>-<topic>` 파일 존재 시 skip, log.md 같은 line 존재 시 skip, index.md 같은 link 존재 시 skip
- **vault Gitea push 자동 호출 절대 금지** — 사용자가 수동으로 `git -C ~/wiki push` 실행

## 7. 스킬 실행

```bash
# read-only (default)
python3 ai-workflow/skills/wiki-query/scripts/run_wiki_query.py \
  --vault-path ~/wiki --project devhub --query "Keycloak RBAC"

# tag + type + limit filter
python3 ai-workflow/skills/wiki-query/scripts/run_wiki_query.py \
  --vault-path ~/wiki --project devhub --query "ADR-0020" --tag rbac --type concept --limit 5

# JSON output (다른 tool 입력용)
python3 ai-workflow/skills/wiki-query/scripts/run_wiki_query.py \
  --vault-path ~/wiki --project devhub --query "keycloak" --format json --output json

# --file mode (AGENTS.md §2.2 6 step 자동)
python3 ai-workflow/skills/wiki-query/scripts/run_wiki_query.py \
  --vault-path ~/wiki --project devhub --query "ADR-0020 결정 사항" --file

# DevHub wrapper (thin wrapper, 이 skill dispatch)
bash ~/repos/Devhub_example/scripts/wiki-query.sh --query "Keycloak RBAC"
bash ~/repos/Devhub_example/scripts/wiki-query.sh --query "ADR-0020" --file
```

종료 코드: 0 (정상, 0 results 포함) / 1 (vault 부재, my_harness skill 미설치, side effect 실패, PERMISSION_DENIED) / 2 (--query 부재, invalid --project/--format/--type/--output).

## 8. 현재 상태

- **draft** (D-79, 2026-06-11) — 본 저장소 thin wrapper DONE (DevHub PR #552), my_harness SSOT 작성 (D-79 Phase 1)
- 4 query primitive 구현 (rg primary, Python fallback) — rg 부재 시 pure Python regex 로 동일 결과 보장
- --file mode 의 idempotency 보장 (재실행 시 중복 X)
- 검증 절차: 5 query sample (read-only 4 + --file 1) — T-d-79-3/4 (사용자 confirm 후)

## 다음에 읽을 문서

- skill 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/wiki_query_skill_spec.md](../../core/wiki_query_skill_spec.md) (D-79, §1-§11)
- 가장 가까운 precedent (ingest): [../../core/wiki_ingest_skill_spec.md](../../core/wiki_ingest_skill_spec.md) (D-72, §1-§11)
- vault 운영 규약: `~/wiki/AGENTS.md` v1.5 §2.2 Query 6 step
- vault lint: [../wiki-lint/SKILL.md](../wiki-lint/SKILL.md) (L01~L10)
