# Wiki-Lint Skill

- 문서 목적: `wiki-lint` skill 의 역할, 입력/출력, 실행 예시를 정리한다.
- 범위: LLM Wiki vault (`~/wiki/`) 의 frontmatter·링크·운영 무결성 검사 (L01~L10)
- 대상 독자: AI agent 설계자, 개발자, vault 운영자
- 상태: beta (D-71, 2026-06-10)
- 관련 문서:
  - [~/wiki/AGENTS.md](../../../wiki/AGENTS.md) (운영 규약)
  - [~/wiki/schema/lint_rules.md](../../../wiki/schema/lint_rules.md) (검사 매트릭스 SSOT)
  - [docs/architecture/DETAILED_DESIGN_LLM_WIKI.md](../../../docs/architecture/DETAILED_DESIGN_LLM_WIKI.md) (디자인)

## 1. 목적

LLM Wiki vault 의 무결성을 검사한다. 검사 항목은 `~/wiki/schema/lint_rules.md` 의 L01~L10.
검사 결과는 사람이 검토할 수 있도록 JSON + markdown 리포트 두 가지로 출력한다.

## 2. 기대 입력

- `--vault-path` (필수): vault 루트 경로 (예: `~/wiki/`)
- `--rules` (선택): 검사할 규칙 ID 콤마 구분 (기본: `L01,L02,L03,L04,L05,L06,L07,L08,L09,L10`)
- `--output` (선택): 출력 형식 `json|markdown|both` (기본: `both`)
- `--report-dir` (선택): markdown 리포트 저장 디렉터리 (기본: `<vault>/_lint/`)
- `--quiet` (선택): stderr 메시지 최소화

## 3. 기대 출력

### 3.1 JSON (stdout)

```json
{
  "status": "ok",
  "tool_version": "0.1.0",
  "vault_path": "/Users/yklee/wiki",
  "examined_at": "2026-06-10T21:50:00",
  "summary": {
    "errors": 0,
    "warns": 0,
    "infos": 0,
    "pages_scanned": 0,
    "rules_executed": ["L01", ..., "L10"]
  },
  "findings": [
    {
      "rule": "L01",
      "severity": "error",
      "path": "wiki/concepts/llm-wiki-pattern.md",
      "message": "frontmatter 누락 또는 필수 필드 부재",
      "missing_fields": ["tags", "status"]
    }
  ]
}
```

### 3.2 Markdown (vault `_lint/report_YYYY-MM-DD.md`)

```markdown
# Lint Report — 2026-06-10

- vault: ~/wiki
- 검사 시각: 2026-06-10 21:50
- 검사자: wiki-lint 0.1.0
- 결과: 0 error / 0 warn / 0 info

(검사 위반 있을 때만 ## Error / ## Warn / ## Info 섹션)
```

## 4. 권한 경계

- **읽기**: `<vault>/**` (raw/, wiki/, schema/, index.md, log.md, _lint/)
- **쓰기**: `<vault>/_lint/report_YYYY-MM-DD.md` 만
- **금칙**: wiki/, raw/, schema/, index.md, log.md 자동 수정
- 위반 시도 시 `error_code=PERMISSION_DENIED` 로 실패

## 5. 구현 메모

- **stdlib only** — `pathlib`, `re`, `argparse`, `json` 만 사용. 외부 의존 없음.
- 검사 항목 SSOT 는 `~/wiki/schema/lint_rules.md` 이지만, 본 스킬은 self-contained — L01~L10 의 핵심을 내장 (lint_rules.md 와 1:1 대응, 표 변경 시 동기화 필요)
- v1.5 (D-71) 의 lint L07 (모순) 은 **구조적 모순만** 감지 (예: 같은 title 의 두 페이지). 의미적 모순은 v2.5+ LLM 호출 필요
- v1.5 의 lint L04 (duplicate) 는 파일명 + frontmatter `title` 의 exact/normalized 일치로 한정

## 6. 검사 항목 빠른 참조

| ID | 검사 | 심각도 |
|---|---|---|
| L01 | frontmatter 누락 / 필수 필드 부재 | error |
| L02 | broken wiki link | error |
| L03 | 고아 페이지 (inbound link 0) | warn |
| L04 | 중복 페이지 (title 일치) | warn |
| L05 | stale (90일+ 미갱신) | info |
| L06 | sources: 경로 부재 | error |
| L07 | 모순 (같은 title 두 페이지) | error |
| L08 | index.md 미등록 wiki 페이지 | warn |
| L09 | log.md 1주일+ 미갱신 | info |
| L10 | source/comparison 타입인데 raw/ source 0 | error |

자세한 정의: `~/wiki/schema/lint_rules.md`

## 7. 스킬 실행

```bash
# 기본 (둘 다 출력)
python3 ai-workflow/skills/wiki-lint/scripts/run_wiki_lint.py --vault-path ~/wiki

# JSON 만
python3 ai-workflow/skills/wiki-lint/scripts/run_wiki_lint.py --vault-path ~/wiki --output json

# 특정 규칙만 (예: error 만)
python3 ai-workflow/skills/wiki_lint.py --vault-path ~/wiki --rules L01,L02,L06,L07,L10

# 종료 코드: findings 가 있으면 1, 없으면 0
```

## 8. 현재 상태

- Beta (D-71) — L01~L10 10개 규칙 구현, stdlib only
- v1.5: 검사 + 리포트 출력까지 (자동 머지 금지, 사람이 _lint/ 검토 후 반영)

## 다음에 읽을 문서
- skills 허브: [../README.md](../README.md)
- lint 규칙 SSOT: `~/wiki/schema/lint_rules.md`
- 디자인: [docs/architecture/DETAILED_DESIGN_LLM_WIKI.md](../../../docs/architecture/DETAILED_DESIGN_LLM_WIKI.md)
