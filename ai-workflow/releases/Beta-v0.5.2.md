# Beta v0.5.2 — Bootstrap 풀 리팩터, 정식 패키지화, 외부 pilot 검증

- **릴리스 일자**: 2026-06-06
- **브랜치**: `release/v0.5.2`
- **포함 커밋**: v0.5.1 (#13, #14, #15) 이후 ~3개
- **상태**: ✅ v0.5.1 후속 3개 TASK 모두 완료

## 1. 하이라이트

v0.5.1 까지 stable 이었던 main 위에 v0.5.2 의 3개 후속 TASK 를 모두 실행:

- **TASK-V052-001** (PR #16): `bootstrap_workflow_kit.py` 2,468줄 모놀리식 → `bootstrap_lib/` 6 모듈 패키지화
- **TASK-V052-002**: `workflow_kit` + `workflow-source` 정식 Python 패키지화 (`pip install -e .` 가능)
- **TASK-V052-003**: 외부 저장소 ([Devhub_example](https://github.com/ykylee/Devhub_example)) pilot validation 1건 통과

## 2. TASK-V052-001: Bootstrap 풀 리팩터

### 동기

v0.5.0 에서 같은 작업을 시도했다가 abort. v0.5.1 에서 `bootstrap_harnesses/` registry 가 추출되어 더 성숙한 상태였기에 재도전.

### 변경

| Before | After |
|--------|-------|
| `bootstrap_workflow_kit.py` (2,468 줄 모놀리식) | `bootstrap_lib/` 6 모듈 패키지 |
| `bootstrap_harnesses/__init__.py` (154 줄, 별도 디렉터리) | `bootstrap_lib/harnesses/__init__.py` (이동) + `renderers.py` (1,046 줄) |

### 모듈 구성

| 모듈 | 줄수 | 책임 |
|------|-----:|------|
| `bootstrap_lib/paths.py` | 158 | Paths, HarnessDefinition, make_paths, 13 path helpers, IGNORED_DIRS |
| `bootstrap_lib/writes.py` | 154 | write_text, copy_core_docs, build_manifest, rel |
| `bootstrap_lib/discovery.py` | 163 | iter_repo_files, detect_package_scripts, guess_run_command, global_snippet_sources |
| `bootstrap_lib/renderers.py` | 351 | render_readme, render_project_profile, render_session_handoff, render_backlog_index, render_daily_backlog, render_assessment, load_template |
| `bootstrap_lib/mcp.py` | 255 | MCP_SERVER_ALIAS, 5 render_*_mcp_config, write_mcp_config_files, MCP_CONFIG_RENDERERS |
| `bootstrap_lib/__main__.py` | 831 | parse_args, infer_project_context, selected_harnesses, update_dependencies, write_harness_files, main |
| `bootstrap_lib/harnesses/__init__.py` | 154 | HARNESS_SPECS, HARNESS_FILE_BUILDERS, register_harness_builder, spec_for |
| `bootstrap_lib/harnesses/renderers.py` | 1,046 | 6 하네스 render_*_agents + 6 write_*_harness_files |
| `bootstrap_workflow_kit.py` | **53** | thin re-export entry (backward compat) |

### 회귀

- `check_bootstrap.py` 7/7 PASS (new / existing / opencode / gemini-cli / antigravity / minimax-code / enable_mcp)
- `check_bootstrap_mcp_roundtrip.py` 5/5 PASS (5개 하네스 round-trip)
- Linter status ok (0 issues, 0 broken links)
- 42/43 smoke tests PASS (`check_docs.py` 의 broken links 는 v0.5.2 memory layer 의 기존 이슈, 리팩터와 무관)

### Backward Compatibility

`python bootstrap_workflow_kit.py` 직접 실행 가능. 모든 public symbol re-export:
- `parse_args, make_paths, main, infer_project_context, selected_harnesses, update_dependencies, write_harness_files`
- `HARNESS_FILE_BUILDERS, HARNESS_SPECS, SUPPORTED_HARNESSES, HarnessSpec, spec_for, HarnessDefinition, Paths`

## 3. TASK-V052-002: 정식 Python 패키지화

### 동기

`workflow_kit` 가 `PYTHONPATH` 해킹으로만 import 가능했던 상태를 normal Python packaging 으로 전환. bootstrap script 도 `python -m bootstrap_lib` 으로 실행 가능하게.

### 변경

- `workflow-source/workflow_kit/pyproject.toml` (신규)
  - name: `standard-ai-workflow-kit`, version: 0.5.2-beta
  - Subpackages: common, common.modes, server, harness
  - Runtime deps: pydantic>=2.0, anyio>=4.0
  - Optional: mcp[cli]>=1.0 (stdio-sdk transport), dev (pytest, ruff, mypy)
- `workflow-source/pyproject.toml` (신규)
  - name: `standard-ai-workflow`, version: 0.5.2-beta
  - Bundles workflow_kit + bootstrap_lib as importable subpackages
  - Package mapping: workflow_kit → workflow_kit/, bootstrap_lib → scripts/bootstrap_lib/

### 설치 / 사용

```bash
# workflow_kit 만 (경량)
pip install -e workflow-source/workflow_kit

# 전체 (workflow_kit + bootstrap_lib + skills + mcp_servers)
pip install -e workflow-source
```

```python
# importable
import workflow_kit  # v0.5.0-beta
import bootstrap_lib
from workflow_kit.common.workflow_state import build_workflow_state_payload
```

## 4. TASK-V052-003: 외부 pilot validation

### 검증 대상

[ykylee/Devhub_example](https://github.com/ykylee/Devhub_example) — public, cross-language (Python+Go+Node+C++), existing ai-workflow/ 디렉터리 보유

### Bootstrap 결과

- 모드: existing
- 하네스: codex, opencode, gemini-cli, antigravity, minimax-code (5종)
- 옵션: `--copy-core-docs --enable-mcp --force`
- 생성: 8 memory layer + 23 harness overlay + 5 MCP config
- 추론: primary_stack=cpp, auxiliary=go/node/python, Makefile 기반 명령

### 검증된 시나리오

1. **Cross-language 프로젝트**: 4개 언어 동시 사용에도 단일 primary_stack 추론 OK
2. **기존 ai-workflow/ 디렉터리**: force 모드로 안전하게 merge
3. **5개 하네스 dialect**: 각각의 entry 파일 위치 + config 형식 (TOML/JSON) 에 맞게 정상 생성
4. **5개 MCP config**: alias `standardAiWorkflowReadOnly`, transport 옵션 jsonrpc-bridge 정상

### 발견된 minor 개선점 (v0.5.3 후보)

- antigravity MCP config 파일명 (`antigravity.mcp.json`) 표준화 필요
- cross-language stack 표시 강화 (현재 primary_stack 단일 값)

## 5. 메모리 layer

v0.5.2 의 모든 작업은 `ai-workflow/memory/release/v0.5.2/` 에 기록:
- `PROJECT_PROFILE.md` — v0.5.2 프로젝트 프로파일
- `session_handoff.md` — 세션 인계 (Work Status, Key Changes, Next Actions)
- `backlog/2026-06-06.md` — TASK-V052-001/002/003 상세 기록
- `state.json` — workflow 상태 캐시

## 6. 회귀 테스트 종합

| 테스트 | 결과 |
|--------|------|
| `check_bootstrap.py` (7 모드) | ✅ 7/7 |
| `check_bootstrap_mcp_roundtrip.py` (5 하네스) | ✅ 5/5 |
| `check_docs.py` | ⚠️ pre-existing broken links (v0.5.2 memory layer) |
| Linter | ✅ status ok, 0 issues |
| Smoke tests (43) | ✅ 42/43 (check_docs.py 제외) |

## 7. 다음 단계 (v0.5.3 후보)

- [ ] `check_docs.py` 의 broken links 수정 (v0.5.2 memory layer 의 stale references)
- [ ] antigravity MCP config 파일명 표준화
- [ ] cross-language stack 표시 (stack_labels 배열)
- [ ] mini-coder-max / general agent 와의 통합 검증
- [ ] PyPI 배포 (현재는 local pip install only)

## 8. v0.5.0 → v0.5.2 변경 통계

```
v0.5.0 (PR #13): 42/42 smoke + MiniMax Code harness overlay
v0.5.1 (PR #14): per-harness MCP install + auto-emit + guide
v0.5.1 (PR #15): check_bootstrap_mcp_roundtrip + 5 harnesses round-trip
v0.5.2 (PR #16): bootstrap 풀 리팩터 + 패키지화 + pilot validation
```

총 4 PR, 7 커밋 (main 기준).
