---
name: wiki-pr-update
description: |
  GitHub PR 의 메타데이터 + touched file 을 LLM Wiki vault 의
  `wiki/projects/<project>/prs/<num>.md` 페이지로 자동 갱신하는 skill.
  idempotency key `pr-<num>-<head.sha>` 로 중복 갱신을 막고,
  `--reingest` 분기에서 mirror-list 7 patterns 매칭 source 를
  `wiki-ingest-from-raw` 로 재투입한다.
version: 0.1.0
tags: [wiki, vault, github, pr, d-80]
requires:
  - python>=3.10
  - gh>=2.0 (PATH; --reingest 분기 시점)
entry_point: scripts/run_wiki_pr_update.py
---

# Wiki-PR-Update Skill

- 문서 목적: `wiki-pr-update` skill 의 실행 진입점, 입력/출력 사용법, 권한 경계를 짧게 안내한다.
- 범위: GitHub PR 1건 → vault `prs/<num>.md` + `log.md` + `index.md` 자동 갱신 (D-80)
- 대상 독자: AI agent 운영자, vault 운영자, DevHub / my_harness 프로젝트 멤버
- 상태: **draft** (D-80, 2026-06-11)
- 관련 문서:
  - 상세 스펙: [../../core/wiki_pr_update_skill_spec.md](../../core/wiki_pr_update_skill_spec.md)
  - 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)
  - D-72 counterpart (raw→wiki ingest): [../../core/wiki_ingest_skill_spec.md](../../core/wiki_ingest_skill_spec.md)
  - vault 운영 규약: `~/wiki/AGENTS.md` (§11.1 D-72 cross-project)
  - lint 규칙 SSOT: `~/wiki/schema/lint_rules.md`

## 1. 목적

GitHub PR 단위의 메타정보 + touched file 을 LLM Wiki vault 의 `prs/<num>.md` 페이지로 자동 반영한다. **사용자가 "PR 이 머지됐는데 vault 의 PR 페이지가 없다"** 라고 말할 때 사용. 상세 동작 / 입력 / 출력 / 권한 경계는 SSOT 스펙 참조.

## 2. 실행

### 2.1 기본 (dry-run preview)

```bash
python3 ai-workflow/skills/wiki-pr-update/scripts/run_wiki_pr_update.py \
  --pr 552 \
  --vault-path ~/wiki \
  --project devhub \
  --pr-metadata /tmp/pr-552.json
```

`pr-metadata` 는 wrapper 가 다음 gh CLI 출력으로 생성:

```bash
gh pr view 552 --json number,title,author,state,mergedAt,headRefOid,files > /tmp/pr-552.json
gh pr diff 552 --name-only > /tmp/pr-552-files.txt
# (--touched-files 옵션 사용 시 위 file 전달)
```

### 2.2 apply (실제 vault 갱신)

```bash
python3 ai-workflow/skills/wiki-pr-update/scripts/run_wiki_pr_update.py \
  --pr 552 \
  --vault-path ~/wiki \
  --project devhub \
  --pr-metadata /tmp/pr-552.json \
  --apply
```

→ `wiki/projects/devhub/prs/552.md` 신규 + `log.md` 1 line + `index.md` PRs 섹션 1줄.

### 2.3 reingest (mirror-list 분기)

```bash
python3 ... --pr 552 --pr-metadata /tmp/pr-552.json --reingest --apply
```

→ touched file 중 7 patterns (ADR / governance / planning / setup / requirements / openapi / workflow memory) 매칭 시 wrapper 가 `wiki-ingest-from-raw --source <file> --apply` dispatch.

### 2.4 idempotency 확인

```bash
python3 ... --pr 552 --pr-metadata /tmp/pr-552.json --apply
# → skip + "already updated" (no side effect, exit 0)
```

frontmatter `last_touched >= head.sha` 이면 자동 skip. force-push / rebase 로 head.sha 가 바뀌면 re-write.

## 3. 출력 예시

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
  "examined_at": "2026-06-11T01:50:00",
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

## 4. 권한 경계 (요약)

| 영역 | Read | Write |
|---|---|---|
| `wiki/projects/<project>/prs/` | ✓ | ✓ (--apply, 신규 prs/<num>.md) |
| `wiki/projects/<project>/sources/` | ✓ | ✓ (--apply, `## Related prs` idempotent append) |
| `wiki/projects/<project>/index.md` | ✓ | ✓ (--apply, PRs 섹션 1줄) |
| `wiki/log.md` | ✓ | ✓ (--apply, idempotent append) |
| `_lint/<project>/pr_update_*.md` | ✓ | ✓ (--output markdown\|both) |
| `raw/**`, `schema/**`, `AGENTS.md` | ✓ | ❌ (AGENTS.md §6) |
| 다른 project 의 `wiki/projects/<other>/` | n/a | ❌ (cross-project 금지) |
| **vault Gitea remote push** | n/a | ❌ (사용자 수동, AGENTS.md §6.5) |
| `gh pr view --json` dispatch | wrapper | wrapper (본 impl 직접 호출 X) |

## 5. 실패 시 가이드

| 증상 | 원인 / 대응 |
|---|---|
| `error_code=INVALID_PR` | `--pr` 0 이하 또는 non-int |
| `error_code=INVALID_PROJECT` | `--project` 가 `devhub` / `my-harness` 외 |
| `error_code=METADATA_READ_FAIL` | `--pr-metadata` file 부재 또는 JSON 파싱 실패 → wrapper 가 `gh pr view` 출력 확인 |
| `error_code=PERMISSION_DENIED` | raw/, schema/, AGENTS.md, 다른 project 의 wiki/ 수정 시도 → 권한 재확인 |
| `ok=false` + `errors` 비어있지 않음 | 출력 JSON 의 `errors` 첫 항목 확인, spec §8 참조 |
| idempotent skip 출력인데 갱신 필요 | head.sha 가 바뀌었거나 frontmatter `last_touched` 가 잘못됨 → vault 의 `prs/<num>.md` frontmatter 확인 |

## 6. 현재 상태

- D-80 draft (2026-06-11)
- 정공법: 본 저장소 (DevHub) = thin wrapper 4 file DONE (PR #552), my_harness = SSOT 3 file (본 스킬)
- wrapper 부재 시 my_harness 측 SSOT 3 file 작성으로 즉시 동작
- Python 3.10+ stdlib only (zero deps), gh CLI 호출은 wrapper 가 담당 (D-72 §11.1 thin-wrapper 원칙)

## 다음에 읽을 문서

- 상세 스펙: [../../core/wiki_pr_update_skill_spec.md](../../core/wiki_pr_update_skill_spec.md)
- D-72 counterpart: [../../core/wiki_ingest_skill_spec.md](../../core/wiki_ingest_skill_spec.md)
- 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)
- D-79 (wiki-query) SKILL: [../../skills/wiki-query/SKILL.md](../../skills/wiki-query/SKILL.md) (별도 agent 진행 중)
