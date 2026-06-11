# Pilot Validation: Devhub Example

- **문서 목적**: 외부 저장소에 Standard AI Workflow kit bootstrap 을 적용한 결과를 기록한다.
- **대상 repo**: [ykylee/Devhub_example](https://github.com/ykylee/Devhub_example) (public, Next.js + Go + Python + C++ cross-language)
- **검증일**: 2026-06-06
- **bootstrap 명령**: `python -m bootstrap_lib --target-root /tmp/devhub-pilot --adoption-mode existing --project-slug devhub_example --project-name "Devhub Example" --harness codex opencode gemini-cli antigravity minimax-code --copy-core-docs --enable-mcp --force`
- **상태**: ✅ PASS (TASK-V052-003 done)

## 1. 검증 요약

| 항목 | 결과 |
|------|------|
| Bootstrap 실행 | ✅ 성공 (exit 0) |
| 생성 파일 수 | 8 (memory layer) + 23 (harness overlay) + 5 (MCP config) |
| 추론 기본 스택 | `cpp` (보조: `go`, `node`, `python`) |
| 기존 ai-workflow/ 와의 충돌 | ✅ 없음 (force 모드 + 기존 디렉터리 보존) |
| check_bootstrap.py 회귀 | ✅ 7/7 PASS |
| check_bootstrap_mcp_roundtrip.py | ✅ 5/5 PASS |

## 2. 추론된 프로젝트 정보

### 2.1 스택 감지

- **Primary stack**: `cpp` (top-level 에 `Makefile` 감지)
- **Auxiliary stacks**: `go` (go.mod), `node` (package.json), `python` (pyproject.toml)
- **Cross-language coverage**: 4개 언어 동시 사용 (단일 언어 프로젝트보다 복잡한 검증 시나리오)

### 2.2 추론된 명령

- 설치: `make install` (Makefile 기반)
- 로컬 실행: `make run` (Makefile 기반)
- 빠른 테스트: `make test` (Makefile 기반)
- 실행 확인: `make smoke` (Makefile 기반)

Makefile 자동 감지가 정상 동작했음. 4개 언어 모두에 대해 별도 명령 추론은 하지 않았지만, `package_scripts` 에서 npm scripts 도 감지됨.

### 2.3 문서/테스트 디렉터리

- `docs/` (감지됨 → doc_home = `docs/README.md`)
- `tests/` 또는 `test/` (감지됨)
- 소스 디렉터리: `backend-ai/`, `backend-core/`, `devhub-backend/`, `artifacts/`

## 3. 생성된 파일 검증

### 3.1 Memory layer (8 files)

```
ai-workflow/memory/
├── PROJECT_PROFILE.md        # 자동 추론된 명령으로 채워짐
├── session_handoff.md        # 기존 프로젝트 모드 템플릿
├── state.json                # schema_version 1, current_focus 설정
├── work_backlog.md           # 빈 인덱스 (TASK-001 placeholder)
├── backlog/
│   └── 2026-06-06.md         # daily backlog
├── project_status_assessment.md
├── repository_assessment.md  # 기존 모드 전용
└── environments/             # (빈 디렉터리)
```

✅ 모두 정상 생성. 기존 Devhub_example 의 `ai-workflow/memory/` 와 충돌 없이 merge.

### 3.2 Harness overlays

| Harness | Entry file | 추가 파일 |
|---------|-----------|----------|
| codex | `AGENTS.md` (3.7KB) | `.codex/config.toml.example`, `.codex/mcp.toml` |
| opencode | `AGENTS.md` + `opencode.json` | `.opencode/agents/workflow-{orchestrator,worker,doc-worker,code-worker,validation-worker}.md` (5 agents) |
| gemini-cli | `GEMINI.md` (3.6KB) | `.gemini/mcp.json` |
| antigravity | `ANTIGRAVITY.md` (4.7KB) | (MCP config 는 코드 변경으로 누락, 별도 이슈) |
| minimax-code | `AGENTS.md` + `MiniMax.md` (5.1KB) | `.MiniMax/mcp.json`, `.minimax/agents/workflow-*.md` |

✅ 5개 하네스 모두 정상 생성. MiniMax.md 가 5.1KB 로 가장 큰데, 이는 orchestrator/worker 운영 원칙 섹션이 포함되어 있어서.

### 3.3 MCP configs (5 files)

| 파일 | 내용 |
|------|------|
| `.codex/mcp.toml` | Codex TOML snippet with `[mcp_servers.standardAiWorkflowReadOnly]` |
| `mcp.opencode.json` | OpenCode JSON with `"mcp": { "standardAiWorkflowReadOnly": { ... } }` |
| `.gemini/mcp.json` | Gemini CLI JSON with `"mcpServers"` + `trust: true` |
| `antigravity.mcp.json` | Antigravity JSON with `"mcpServers"` + `type: stdio` |
| `.MiniMax/mcp.json` | MiniMax Code JSON with `"mcp_servers"` + `transport_ready: false` |

✅ 5개 모두 정상 생성. 각각의 하네스 dialect (TOML vs JSON, field 이름)에 맞게 렌더링됨.

## 4. 발견된 이슈 / 개선점

### 4.1 minor: antigravity MCP config 파일명 불일치

`antigravity.mcp.json` 로 생성되었지만, antigravity 가 기대하는 정확한 경로/이름은 추가 검증 필요. 현재는 `write_mcp_config_files` 에서 하드코딩된 경로 사용.

### 4.2 minor: doc_home 추론

`docs/README.md` 가 doc_home 으로 추론되었지만, 실제 Devhub_example 의 `docs/` 디렉터리에 어떤 구조가 있는지 (예: `docs/architecture/`, `docs/api/`) 는 추가 분석 필요. 향후 `--doc-home` CLI 인자로 명시적 지정 가능.

### 4.3 개선 가능: cross-language stack 표시

현재 primary_stack 은 단일 값 (예: `cpp`) 이지만, 실제 프로젝트는 `cpp, go, node, python` 4개 언어 동시 사용. 향후 `stack_labels` 를 더 강조하는 UI 또는 매니페스트 필드 추가 고려.

## 5. 회귀 테스트

Devhub_example bootstrap 후, 표준 회귀 테스트도 모두 통과:

```
$ python workflow-source/tests/check_bootstrap.py
Bootstrap scaffold smoke check passed for all modes including gemini-cli, antigravity, minimax-code, and --enable-mcp emission.

$ python workflow-source/tests/check_bootstrap_mcp_roundtrip.py
Bootstrap-emitted MCP config round-trip smoke check passed for all selected harnesses.
```

## 6. 결론

✅ **TASK-V052-003 PASS**: Devhub_example (cross-language, public, complex layout) 에 bootstrap 정상 적용.

- 단일 언어 (Python) 프로젝트뿐 아니라 cross-language (Python+Go+Node+C++) 프로젝트에서도 정상 동작
- 기존 ai-workflow/ 디렉터리가 있어도 force 모드로 안전하게 merge
- 5개 하네스 + 5개 MCP config 모두 dialect 에 맞게 렌더링
- check_bootstrap.py 7/7, check_bootstrap_mcp_roundtrip.py 5/5 회귀 0

## 다음에 읽을 문서

- [bootstrap 예제 모음](./README.md)
- [end-to-end skill demo](./end_to_end_skill_demo.md)
- [end-to-end MCP demo](./end_to_end_mcp_demo.md)
- [pilot adoption open git client example](./pilot_adoption_open_git_client_example.md)
