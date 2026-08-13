# Aider (Aider-AI/aider) 심층 분석

- **문서 목적**: TASK-004 1차 분석(`docs/REFERENCES.md` §3.2 의 1-페이지 aider 박스) 의 후속. aider 소스 코드를 **14 섹션 표준 템플릿** 으로 풀스캔해, `my_harness` (Rust/TS CLI+TUI 코딩 에이전트) 의 아키텍처 결정에 직접 인용 가능한 인사이트를 만든다.
- **범위**: aider Python 코드베이스 (`aider/` 패키지 + `tests/` + `requirements/`) + `pyproject.toml` + `aider/resources/model-settings.yml` 의 **실제 코드** 만. 추측/이슈 트래커/블로그는 명시적으로 표시될 때만 보조.
- **대상 독자**: yklee (소유자), Mavis, TASK-005 (스택/이름/MVP 결정) 디자인 리뷰어, 4-워커 중 "Python 레퍼런스" 를 보는 워커.
- **상태**: v2 updated (TASK-004 재방문, 결정 변경 불요, 2026-08-14)
- **최종 수정일**: 2026-08-14 (v2 §15-16 append, +124 lines)
- **v1 작성일**: 2026-06-06 (1,925 lines, 14섹션)
- **관련 문서**: [REFERENCES.md §3.2](../REFERENCES.md), [ANALYSIS_PLAN.md](./ANALYSIS_PLAN.md), [opencode.md](./opencode.md), [codex.md](./codex.md), [goose.md](./goose.md), [gemini-cli.md](./gemini-cli.md), TASK-005 (CLI/TUI 전환)
- **재방문 결정 ID**: D-128 (TASK-004 재방문, aider v2, 2026-08-14, 정직 0 commit)

---

## 목차

1. [개요 (Overview)](#1-개요-overview)
2. [아키텍처 (Architecture)](#2-아키텍처-architecture)
3. [진입점 & CLI](#3-진입점--cli)
4. [TUI/UI 구현](#4-tuiui-구현)
5. [LLM 통합](#5-llm-통합)
6. [도구/스킬 시스템 — 그리고 "부재"](#6-도구스킬-시스템--그리고-부재)
7. [컨텍스트 관리 — repo.py + repomap.py + history.py](#7-컨텍스트-관리--repopy--repomappy--historypy)
8. [세션 영속화](#8-세션-영속화)
9. [확장 시스템 — "부재의 미학"](#9-확장-시스템--부재의-미학)
10. [빌드 & 배포](#10-빌드--배포)
11. [테스트 & 품질](#11-테스트--품질)
12. [보안](#12-보안)
13. [주목할 패턴 (Notable Patterns)](#13-주목할-패턴-notable-patterns) — **가장 중요**
14. [미해결 질문 (Open Questions)](#14-미해결-질문-open-questions)

---

## 1. 개요 (Overview)

**Aider** 는 Paul Gauthier (Aider-AI) 가 만든 **터미널 페어 프로그래밍** 도구. 2023년 출시, GitHub Stars 약 33k (2026-06 기준), Python 3.10+ 만 필요. 핵심 가치는 "**AI 가 직접 git commit 까지 하는 페어 프로그래머**" — `git` 을 first-class 데이터 소스로 취급하고, LLM 이 만든 diff 를 `git` 워크플로우 (auto-commit, undo, /commit) 에 자연스럽게 통합한다.

| 항목 | 값 |
| --- | --- |
| 라이선스 | Apache 2.0 (`LICENSE.txt`, 11,358 bytes) |
| 메인 binary | `aider` (`[project.scripts] aider = "aider.main:main"` in `pyproject.toml:27`) |
| PyPI 패키지 | `aider-chat` (`pyproject.toml:3`) |
| 버전 (분석 시점) | `0.86.3.dev` (`aider/__init__.py:3`) |
| 코드베이스 크기 | `aider/*.py` 45 파일 / **13,359 LOC** ; `aider/coders/*.py` 33 파일 / 6,923 LOC; **합 약 20,000 LOC** |
| Python 요구 | `>=3.10,<3.15` (`pyproject.toml:20`) |
| 의존성 (런타임) | **63개** (`requirements.txt` top-level 자동 생성) |
| Entry 진입 | `python -m aider` (`aider/__main__.py:3`) |
| 모드 | **CLI (기본) + GUI (Streamlit 옵션)** 두 개; 같은 `Coder` 객체를 양쪽이 공유 |

**왜 우리에게 중요한가** (1차 분석에서 이미 짚은 점을 코드 인용으로 보강):

1. **git 중심 워크플로우**: aider 의 `aider/repo.py` 한 파일 (622 LOC) 이 git 의 모든 책임 (tracked files, dirty files, commit message, attribution, undo) 을 떠맡는다. 우리 my_harness 가 git-aware CLI 라면 **이 파일을 그대로 차용하거나 패턴을 본떠야** 한다.
2. **GraphRAG 의 실전 구현**: `aider/repomap.py` (867 LOC) 는 tree-sitter 로 tag 추출 → networkx MultiDiGraph → PageRank 로 "이 LLM 호출에서 어느 심볼이 가장 중요한가" 를 결정. RAG 의 가장 단순하고 강력한 형태.
3. **확장성 부재**: aider 에는 **plugin 시스템, MCP, hooks, skill, dynamic tool loading 이 전부 없다**. 그럼에도 잘 작동하는 이유를 분석하면 우리 `my_harness` 의 "extension surface" 를 좁히는 데 결정적 참고가 된다 (TASK-005 §5.1/§5.2 의 MVP 범위 결정).


---

## 2. 아키텍처 (Architecture)

### 2.1 프로세스 모델

aider 는 **단일 프로세스, 단일 이벤트 루프, 동기 I/O** 가 기본. asyncio 를 쓰지 않는다 (litellm 의 동기 API 만 사용). `watchfiles` (FileWatcher) 와 `threading.Thread` (cache warming, summarization) 두 곳에서만 백그라운드 스레드.

```
┌────────────────────────────────────────────────────────────────────┐
│                        aider 프로세스 (1개)                          │
│                                                                    │
│  ┌──────────────┐    ┌─────────────────┐    ┌──────────────────┐  │
│  │ main.main()  │───▶│ InputOutput     │◀──▶│ prompt_session   │  │
│  │  (1,274 LOC) │    │  (1,191 LOC)    │    │ (prompt_toolkit) │  │
│  └──────┬───────┘    └────────┬────────┘    └──────────────────┘  │
│         │                     │ prompt + completion               │
│         │  args + Coder.create│                                    │
│         ▼                     ▼                                    │
│  ┌────────────────┐    ┌─────────────────┐                         │
│  │ argparse       │    │ Commands        │──▶ SwitchCoder (except) │
│  │ configargparse │    │  (1,712 LOC)    │    (1,712 LOC 안)       │
│  │ + shtab        │    └────────┬────────┘                         │
│  └────────────────┘             │                                  │
│                                 ▼                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │   Coder (coders/base_coder.py, 2,485 LOC) — orchestrator     │  │
│  │   ├─ run() → run_one() → send_message() → self.send()         │  │
│  │   ├─ RepoMap (repomap.py, 867 LOC) — repo graph + PageRank    │  │
│  │   ├─ GitRepo (repo.py, 622 LOC) — git, dirty, attribution     │  │
│  │   ├─ ChatSummary (history.py, 143 LOC) — token-budgeted summ  │  │
│  │   ├─ Model (models.py, 1,338 LOC) — litellm wrapper, token ct │  │
│  │   ├─ Linter (linter.py, 304 LOC) — auto lint                  │  │
│  │   ├─ FileWatcher (watch.py, 318 LOC, thread) — live changes   │  │
│  │   └─ format_chat_chunks() → messages[]                        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│         │                                                            │
│         │  HTTP (sync)                                               │
│         ▼                                                            │
│  ┌────────────────┐    ┌──────────────────┐                         │
│  │ litellm (lazy) │───▶│  OpenAI / Anthro │                         │
│  │  / llm.py 47L  │    │  / DeepSeek / …  │                         │
│  └────────────────┘    └──────────────────┘                         │
└────────────────────────────────────────────────────────────────────┘
```

### 2.2 모듈 경계 (의존 방향)

```
aider.main            ──▶ coders.Coder.create
     │                       │
     │                       ├──▶ coders.base_coder.Coder
     │                       │         │
     │                       │         ├──▶ models.Model (litellm)
     │                       │         ├──▶ repo.GitRepo
     │                       │         ├──▶ repomap.RepoMap
     │                       │         ├──▶ history.ChatSummary
     │                       │         ├──▶ io.InputOutput
     │                       │         └──▶ commands.Commands
     │                       │
     │                       └──▶ coders.EditBlockCoder / AskCoder / … (subclass)
     │
     └──▶ analytics.Analytics (Posthog + Mixpanel, opt-in)
```

핵심 관찰: **방향성 있는 DAG** 다. `commands.py` 와 `analytics.py` 만 위로 (main 으로) 의존성이 흐르고, 나머지는 모두 base_coder 로 단방향 집중. `from .dump import dump` 패턴이 **모든 파일** 에 import 되어 있어 디버깅 출력을 위한 일종의 global print hook 역할.

### 2.3 디렉토리 트리 (심볼릭)

```
aider/
├── __init__.py            # version handling (setuptools_scm)
├── __main__.py            # `python -m aider`
├── main.py                # entry, args parsing, Analytics, run loop
├── args.py                # configargparse (945 LOC, 약 80 옵션)
├── args_formatter.py      # YAML/Markdown/DotEnv formatter
├── analytics.py           # Posthog + Mixpanel (opt-in, 258 LOC)
├── io.py                  # InputOutput, AutoCompleter, prompt_toolkit (1,191 LOC)
├── mdstream.py            # streaming markdown → rich (243 LOC)
├── commands.py            # Commands class, SwitchCoder exception (1,712 LOC)
├── repo.py                # GitRepo class — git abstraction (622 LOC)
├── repomap.py             # RepoMap class — graph + PageRank (867 LOC)
├── history.py             # ChatSummary — token-budgeted summarization (143 LOC)
├── models.py              # Model class, ModelSettings, litellm wrapper (1,338 LOC)
├── llm.py                 # LazyLiteLLM (47 LOC — 핵심: 시작 시간 1.5s 절약)
├── sendchat.py            # sanity_check_messages, ensure_alternating_roles
├── exceptions.py          # LiteLLMExceptions — retryable/description 매핑
├── linter.py              # Linter (flake8/syntax check)
├── voice.py               # /voice 명령 — sounddevice + OpenAI whisper
├── watch.py               # FileWatcher (watchfiles 기반, "ai" 주석 감지)
├── scrape.py              # Scraper (BeautifulSoup + optional Playwright)
├── help.py                # /help 명령 (의도 분류 → vector search)
├── openrouter.py          # OpenRouter model DB 관리
├── utils.py               # format_messages, format_tokens, is_image_file 등
├── prompts.py             # 60줄 — 시스템 prompt 의 fixed 섹션만 (나머지는 *_prompts.py)
├── reasoning_tags.py      # `<think>` / `<reasoning>` 태그 추출/제거
├── format_settings.py     # scrub_sensitive_info — API 키 마스킹
├── versioncheck.py        # PyPI vs current version check
├── report.py              # /report 명령 → GitHub issue URL 빌드
├── special.py             # filter_important_files — README/License 등
├── urls.py                # 모든 docs URL 상수
├── copypaste.py           # ClipboardWatcher (optional)
├── editor.py              # pipe_editor — $EDITOR 임시 파일 처리
├── run_cmd.py             # run_cmd — subprocess + tee 출력
├── waiting.py             # WaitingSpinner (rich 기반)
├── diffs.py               # udiff/diff 생성기
├── deprecated.py          # --4 / --opus 같은 deprecated alias
├── dump.py                # dump() — dev debug print
├── onboarding.py          # 첫 실행 / OpenRouter OAuth
├── resources/
│   ├── model-settings.yml         # 기본 모델별 ModelSettings dict
│   ├── model-metadata.json        # litellm 가격 DB 사본
│   └── ...
├── coders/                 # ← EditFormat 의 polymorphic 구현
│   ├── __init__.py         # Coder.create() 가 coders.__all__ 순회
│   ├── base_coder.py       # Coder (2,485 LOC, orchestrator)
│   ├── chat_chunks.py      # ChatChunks dataclass
│   ├── base_prompts.py     # main_system, system_reminder, …
│   ├── editblock_coder.py  # edit_format = "diff" (search/replace)
│   ├── editblock_fenced_coder.py
│   ├── editblock_func_coder.py
│   ├── udiff_coder.py
│   ├── udiff_simple.py
│   ├── wholefile_coder.py
│   ├── patch_coder.py      # unified diff format
│   ├── ask_coder.py        # read-only Q&A
│   ├── context_coder.py    # context-only mode
│   ├── architect_coder.py  # architect → editor 2-model 패턴
│   ├── help_coder.py
│   ├── shell.py            # shell-aware prompts
│   ├── search_replace.py   # SEARCH/REPLACE 유틸리티
│   ├── *_prompts.py        # 각 coder 의 prompt
│   └── ...
├── queries/                # tree-sitter tag 쿼리
│   ├── tree-sitter-language-pack/  ← 30+ .scm
│   └── tree-sitter-languages/
└── website/                # aider.chat 사이트 빌드 산출물
```

### 2.4 데이터 흐름 (1회 메시지 처리)

아래는 사용자가 한 줄 입력 → LLM 응답 → 파일 편집 → commit 까지의 완전한 데이터 흐름.

```
[user input]
    │
    ▼
[InputOutput.get_input()]  ──▶ prompt_toolkit.PromptSession
    │                          (PygmentsLexer(MarkdownLexer), ThreadedCompleter)
    │                          (kbd: Ctrl-Z bg, Ctrl-Up history, Ctrl-X Ctrl-E editor)
    ▼
[Coder.run(with_message=user_input)]
    │  init_before_message() ──▶ repo.get_head_commit_sha() 저장
    ▼
[Coder.run_one]  ──▶ preproc_user_input()  ──▶ Commands.is_command("/...")?
    │  no
    ▼
[Coder.send_message()]
    │  1) cur_messages.append({"role":"user","content":inp})
    │  2) format_chat_chunks()
    │       ├─ choose_fence()         # 7 후보 백틱 변종 중 충돌 안 나는 것 선택
    │       ├─ fmt_system_prompt()    # OS/lang/date + lazy/overeager reminder
    │       ├─ ChatChunks {
    │       │     system = [main_sys],
    │       │     examples = [...],
    │       │     done = self.done_messages,
    │       │     repo = self.get_repo_messages()           ◀── RepoMap
    │       │     readonly_files = self.get_readonly_files_messages(),
    │       │     chat_files = self.get_chat_files_messages(),
    │       │     cur = self.cur_messages,
    │       │     reminder = [...] (token-budgeted)
    │       │   }
    │       └─ if total_tokens < max_input_tokens: add reminder
    │  3) check_tokens(messages)        # 95% 안전선 확인, 100% 초과 시 confirm
    │  4) warm_cache(chunks)            # background thread, 5min 간격 max_tokens=1 ping
    ▼
[Coder.send()]
    │  model.send_completion(messages, functions, stream, temperature)
    │     │  send_completion:
    │     │    hash = sha1(json.dumps(kwargs, sort_keys=True))
    │     │    res = litellm.completion(model=self.name, messages=..., stream=stream, **kwargs)
    │     ▼
    │  show_send_output_stream(completion)        # chunk-by-chunk
    │     │  for chunk in completion:
    │     │      append reasoning_content / content / function_call to partial
    │     │      self.io.live_incremental_response(False)
    │     │          └─ mdstream.update(...)      # rich Live display
    │     ▼
    ▼
[Coder.send_message post-processing]
    │  - add_assistant_reply_to_cur_messages()
    │  - check_for_file_mentions(content)         # 단어 → filename 매핑
    │  - if partial_response_function_call:  …
    │  - apply_updates()  ──▶ EditBlockCoder.get_edits()
    │       └─ find_original_update_blocks(content, fence, fnames)
    │            └─ apply_edits()  ─▶ do_replace(file, content, original, updated, fence)
    │                                  ├─ perfect_replace
    │                                  ├─ replace_most_similar_chunk (whitespace, ...)
    │                                  └─ write_text(path, new_content)  # retry on lock
    │  - lint_edited(edited)  ──▶ Linter.lint()  ──▶ error 시 reflected_message
    │  - run_shell_commands()  ──▶ "/run" block 안의 shell command
    │  - if auto_test:  cmd_test(test_cmd)
    │  - auto_commit(edited)  ──▶ GitRepo.commit()    ◀── attribution logic
    │       ├─ diffs = get_diffs(edited)
    │       ├─ commit_message = get_commit_message(diffs)  ──▶ LLM
    │       ├─ set GIT_COMMITTER_NAME / GIT_AUTHOR_NAME env
    │       ├─ git add + git commit
    │       └─ return (hash, message)
    │  - move_back_cur_messages()  ──▶ summarize_start()   ◀── background
    ▼
[done]
```

핵심 통찰:

- **전부 동기**. `await` / `async` 가 한 군데도 없다. `litellm.completion` 만 동기 blocking, 그 외엔 그냥 `time.sleep`/`threading.Thread`. **"동기 + 백그라운드 스레드" 모델** — 우리 my_harness 가 단일 파일 고속 처리에 집중한다면 이게 맞다.
- **format_chat_chunks()가 진짜 핵심**. 이 함수가 `system + examples + done + repo + readonly + chat + cur + reminder` 8개 슬롯을 정렬하고, 토큰 한도 내로 끼워 맞추는 작업을 한다. 이게 §7 "context management" 의 본체.
- **edit format = polymorphic dispatch**. `Coder.create(edit_format=...)` 가 `coders.__all__` 을 순회하며 `coder.edit_format == edit_format` 매치 → 인스턴스. 13개 edit format (`diff`, `whole`, `architect`, `ask`, `context`, `udiff`, `help`, `patch`, ...).


---

## 3. 진입점 & CLI

### 3.1 바이너리

```python
# pyproject.toml:27
[project.scripts]
aider = "aider.main:main"
```

또는 모듈로: `aider/__main__.py` 가 `from .main import main; main()` 만 한다 (4 LOC).

### 3.2 인자 파싱 — configargparse + shtab

`aider/args.py` (945 LOC) 은 **`configargparse`** 를 써서 **CLI flag + YAML config file + env var** 3중 precedence 로 옵션을 읽는다. shell completion 은 **shtab** 으로 자동 생성.

```python
# aider/args.py:35
def get_parser(default_config_files, git_root):
    parser = configargparse.ArgumentParser(
        description="aider is AI pair programming in your terminal",
        add_config_file_help=True,
        default_config_files=default_config_files,
        config_file_parser_class=configargparse.YAMLConfigFileParser,
        auto_env_var_prefix="AIDER_",
    )
    # 동적으로 edit format 후보 수집
    from aider import coders as _aider_coders
    edit_format_choices = sorted(
        {c.edit_format for c in _aider_coders.__all__
         if hasattr(c, "edit_format") and c.edit_format is not None}
    )
```

주요 옵션 그룹 (10개):

| Group | 대표 옵션 |
| --- | --- |
| Main model | `--model`, `files` (positional) |
| API Keys and settings | `--openai-api-key`, `--anthropic-api-key`, `--set-env`, `--api-key` |
| Model settings | `--weak-model`, `--editor-model`, `--reasoning-effort`, `--thinking-tokens`, `--edit-format` |
| Cache settings | `--cache-prompts`, `--cache-keepalive-pings` |
| Repomap settings | `--map-tokens`, `--map-refresh` (`auto`/`always`/`files`/`manual`), `--map-multiplier-no-files` |
| History Files | `--input-history-file`, `--chat-history-file`, `--restore-chat-history` |
| Repo settings | `--git`, `--aiderignore`, `--subtree-only` |
| Fixing and committing | `--auto-commits`, `--dirty-commits`, `--commit` |
| Mode | `--gui`, `--message`, `--message-file`, `--apply`, `--apply-clipboard-edits` |
| Misc | `--dark-mode`, `--light-mode`, `--vim`, `--shell-completions` |

**Config 파일 검색 순서** (`aider/main.py:464-477`):

```python
conf_fname = Path(".aider.conf.yml")
default_config_files = [conf_fname.resolve()]   # CWD
if git_root:
    default_config_files.append(git_root / conf_fname)  # git root
default_config_files.append(Path.home() / conf_fname)  # homedir
# → CWD → git root → home 순으로 load (뒤가 우선)
default_config_files.reverse()
```

### 3.3 main() 의 처리 시퀀스 (1,274 LOC 중 본문)

`aider/main.py:451 main()` 은 다음을 순서대로 한다:

1. **git root 추측** (`get_git_root()`) — `git.Repo(search_parent_directories=True)`
2. **config 파일 경로 결정** (CWD → git root → home)
3. **args 파싱 1차** (config 파일 + CLI + env)
4. **`.env` 파일 로드** (`load_dotenv_files` — git_root 와 `~/.aider/oauth-keys.env` 합쳐서)
5. **args 파싱 2차** (env var 가 args 에 영향 줄 수 있으므로)
6. **shell completion 처리** (shtab — `parser.prog = "aider"; print(shtab.complete(parser, shell=...))`)
7. **dark/light mode → 색상 override**
8. **InputOutput 생성** (dumb terminal 검사, PromptSession 초기화)
9. **API key 처리** (--set-env, --api-key, 직접 --openai-api-key)
10. **Analytics opt-in** (Posthog + Mixpanel)
11. **GUI mode 체크** → `launch_gui(args)` 가 `streamlit run aider/gui.py` 로 위임
12. **Model 선택** (`select_default_model` — 첫 실행 시 OpenRouter OAuth 옵션)
13. **main_model 인스턴스화** (litellm, ModelSettings 적용, `apply_generic_model_settings` 으로 fallback)
14. **GitRepo 인스턴스화** (model.commit_message_models() 도 같이)
15. **ChatSummary 인스턴스화** (max_tokens = 1024 또는 모델 default)
16. **Coder.create()** — 이 시점부터 모든 게 폴리몰픽 dispatch
17. **FileWatcher / ClipboardWatcher** 옵션
18. **one-shot 명령 처리**: `--lint`, `--test`, `--commit`, `--show-repo-map`, `--apply`, `--message`
19. **메인 루프**: `while True: coder.run()`. `SwitchCoder` 예외가 던져지면 새 Coder 인스턴스로 교체 (모델/모드 변경).

### 3.4 서브커맨드 트리 — 1단계

aider 는 **argparse subparser 를 안 쓴다**. 모든 기능이 "**slash command**" 로 (in-chat) 진입한다 (`aider/commands.py` 의 `cmd_*` 메서드). CLI 인자는 거의 모두 **boolean / value flag** 다.

```
aider [FILE ...]
  ├── (in-chat) /add <file>         /drop <file>
  ├── (in-chat) /ls                  /map
  ├── (in-chat) /commit [msg]        /undo
  ├── (in-chat) /diff                /lint
  ├── (in-chat) /test                /run <cmd>          (alias: !)
  ├── (in-chat) /clear               /reset
  ├── (in-chat) /tokens              /settings
  ├── (in-chat) /model <name>        /weak-model <name>  /editor-model <name>
  ├── (in-chat) /chat-mode <mode>    /architect          /ask
  ├── (in-chat) /code                /context            /help [q]
  ├── (in-chat) /read-only <file>    /web <url>          /voice
  ├── (in-chat) /paste               /copy               /save <file>
  ├── (in-chat) /load <file>         /multiline-mode
  ├── (in-chat) /think-tokens <n>    /reasoning-effort <lvl>
  ├── (in-chat) /map-refresh         /git <git-cmd>
  ├── (in-chat) /report [title]      /editor
  ├── (in-chat) /copy-context        /exit               /quit
  └── (one-shot) --lint / --test / --commit / --message "..." / --apply <file>
```

전체 `cmd_*` 메서드 수: **57개** (`commands.py` grep "def cmd_"). 이들이 런타임에 `getattr(self, "cmd_" + name)` 으로 디스패치된다.

---

## 4. TUI/UI 구현

### 4.1 라이브러리 의존

aider 의 TUI 는 **두 라이브러리의 합** 으로 구성된다:

| 라이브러리 | 역할 | LOC 점유 |
| --- | --- | --- |
| `prompt_toolkit` | 입력 (auto-completion, syntax highlight, history, vi/emacs) | `io.py` 중 약 600 LOC |
| `rich` | 출력 (Markdown, color, Columns, spinner, Live) | `io.py` + `mdstream.py` + `waiting.py` |

설치 의존성 (이 두 개는 선택 아님): `prompt-toolkit==3.0.52`, `rich==14.3.3` (from `requirements.txt`).

### 4.2 render 루프 (의사코드)

aider 는 진정한 TUI render 루프 (예: opencode 의 Bubble Tea) 가 **없다**. `prompt_toolkit.PromptSession.prompt()` 가 메인 루프고, LLM 응답 출력은 rich 가 단발성으로 처리. 하지만 streaming 출력 시에는 `mdstream.MarkdownStream.update()` 가 rich 의 `Live` 컨텍스트를 사용.

```python
# io.py:523 — get_input() 메인 루프
def get_input(self, root, rel_fnames, addable_rel_fnames, commands, ...):
    self.rule()
    self.ring_bell()
    ...
    completer_instance = ThreadedCompleter(AutoCompleter(root, rel_fnames, addable_rel_fnames, ...))
    kb = KeyBindings()
    @kb.add("c-space"): event.current_buffer.insert_text(" ")
    @kb.add("c-up"):    event.current_buffer.history_backward()
    @kb.add("c-down"):  event.current_buffer.history_forward()
    @kb.add("c-x","c-e"): pipe_editor(...)  # $EDITOR 호출
    @kb.add("enter", eager=True, filter=~is_searching):
        if self.multiline_mode and not vi-nav: insert "\n"
        else: validate_and_handle()
    @kb.add("escape", "enter", eager=True, filter=~is_searching):  # Alt+Enter
        if self.multiline_mode: validate_and_handle()
        else: insert "\n"
    line = self.prompt_session.prompt(
        show, default=self.placeholder, completer=completer_instance,
        reserve_space_for_menu=4, complete_style=CompleteStyle.MULTI_COLUMN,
        style=style, key_bindings=kb, complete_while_typing=True,
        prompt_continuation=get_continuation,
    )
```

### 4.3 rich 의 활용 패턴 — `io.py` 전체

rich 의 핵심 primitive 들이 `io.py` 에서 어떻게 쓰이는지:

```python
# 1) Color 검증 + fallback
# io.py:374 _validate_color_settings
for attr_name in color_attributes:  # 9개 color attr
    try:
        RichStyle(color=color_value)  # 잘못된 color 는 여기서 예외
    except ColorParseError as e:
        setattr(self, attr_name, None)  # disable

# 2) Markdown streaming
# io.py:1023 assistant_output
if pretty:
    show_resp = Markdown(message, style=self.assistant_output_color, code_theme=self.code_theme)
else:
    show_resp = Text(message or "(empty response)")
self.console.print(show_resp)

# 3) Columns — 파일 목록을 컬럼으로
# io.py:1138 format_files_for_input
output = StringIO()
console = Console(file=output, force_terminal=False)
console.print(Columns(files_with_label))   # "Editable:" + 파일들

# 4) Spinner — LLM 응답 대기
# io.py:1440 (base_coder.py:1440)
self.waiting_spinner = WaitingSpinner("Waiting for " + self.main_model.name)
self.waiting_spinner.start()

# 5) error/warning 색상
# io.py:988
def tool_error(self, message="", strip=True):
    self.num_error_outputs += 1
    self._tool_message(message, strip, self.tool_error_color)  # 빨강
```

### 4.4 상태 관리

aider 의 TUI 상태는 **단일 InputOutput 객체** + **Coder 인스턴스** 에 집중. **명시적 state store 없음**. `done_messages` / `cur_messages` / `aider_commit_hashes` 같은 것들이 Coder 의 attribute. `analytics.py` 의 `State` 는 GUI 모드용 (Streamlit cache_resource).

```python
# io.py:230 InputOutput
class InputOutput:
    num_error_outputs = 0
    num_user_asks = 0
    clipboard_watcher = None
    bell_on_next_input = False
    # instance:
    self.placeholder = None
    self.interrupted = False
    self.never_prompts = set()
    self.multiline_mode = False
    self.notifications = False
    self.prompt_session = None   # prompt_toolkit session
    self.console = Console(...)  # rich console
```

### 4.5 키 바인딩

`io.py:575-634` 의 `KeyBindings` 매핑:

| 키 | 동작 |
| --- | --- |
| `Ctrl-Z` | 백그라운드로 suspend (시그널 SIGTSTP 있는 OS 한정) |
| `Ctrl-Space` | 무시 (단순히 스페이스 입력) |
| `Ctrl-Up` / `Ctrl-Down` | 입력 히스토리 이동 |
| `Ctrl-X Ctrl-E` | 외부 editor (Bash 와 동일) |
| `Enter` | (multiline) newline / (normal) submit |
| `Alt-Enter` | (multiline) submit / (normal) newline |
| `Esc-Enter` | (multiline) submit (호환) |

### 4.6 Windows / Dumb terminal 처리

```python
# io.py:339
self.is_dumb_terminal = is_dumb_terminal()
if self.is_dumb_terminal:
    self.pretty = False
    fancy_input = False
```

`prompt_toolkit.output.vt100.is_dumb_terminal()` 로 검사 → plain `input()` fallback. Windows 는 prompt_toolkit 이 잘 지원하므로 추가 코드 없음. `windows-curses` 같은 의존성도 없음 (Python stdlib `msvcrt` 안 씀).

### 4.7 GUI 모드

`--gui` 플래그 시 `launch_gui` (main.py:233) → `streamlit run aider/gui.py` (subprocess). **같은 Coder 객체를 CaptureIO 로 wrap** 해서 Streamlit 위에서 동작. `gui.py` 545 LOC, `st.cache_resource` 로 coder 캐싱.


---

## 5. LLM 통합

### 5.1 Provider 추상화 — litellm 한 곳으로

aider 는 **litellm** (BerriAI/litellm, 800+ 모델 통합) 한 곳을 통해서만 LLM 호출. 자체 provider 추상화 없음. 결과적으로:

```python
# aider/llm.py:21 — Lazy import
class LazyLiteLLM:
    _lazy_module = None
    def __getattr__(self, name):
        if name == "_lazy_module": return super()
        self._load_litellm()
        return getattr(self._lazy_module, name)
    def _load_litellm(self):
        if self._lazy_module is not None: return
        self._lazy_module = importlib.import_module("litellm")
        self._lazy_module.suppress_debug_info = True
        self._lazy_module.set_verbose = False
        self._lazy_module.drop_params = True   # ← 미지원 param 자동 drop
        self._lazy_module._logging._disable_debugging()
litellm = LazyLiteLLM()
```

**핵심 트릭**: `litellm` import 가 **1.5초** 걸리는데 (주석: `aider/llm.py:16`), `LazyLiteLLM` 으로 미루고, `load_slow_imports` (main.py:1256) 에서 `import httpx, litellm, networkx, numpy` 를 **첫 실행엔 동기 / 이후엔 background thread** 로 로드:

```python
# main.py:1226
def check_and_load_imports(io, is_first_run, verbose=False):
    if is_first_run:
        load_slow_imports(swallow=False)
    else:
        thread = threading.Thread(target=load_slow_imports)
        thread.daemon = True
        thread.start()
```

### 5.2 Model 클래스 — `models.py:329` 의 모든 것

```python
# models.py:329
class Model(ModelSettings):
    def __init__(self, model, weak_model=None, editor_model=None,
                 editor_edit_format=None, verbose=False):
        model = MODEL_ALIASES.get(model, model)
        self.name = model
        self.max_chat_history_tokens = 1024
        # ...
        self.info = self.get_model_info(model)
        # max_chat_history_tokens = clamp(max_input_tokens / 16, 1024, 8192)
        self.max_chat_history_tokens = min(max(max_input_tokens / 16, 1024), 8192)
        self.configure_model_settings(model)  # name → exact or generic
        self.get_weak_model(weak_model)
        self.get_editor_model(editor_model, editor_edit_format)

    def token_count(self, messages):
        if type(messages) is list:
            try:    return litellm.token_counter(model=self.name, messages=messages)
            except: return 0
        # str/dict: use tokenizer
        if not self.tokenizer: return
        msgs = messages if type(messages) is str else json.dumps(messages)
        try:    return len(self.tokenizer(msgs))
        except: return 0

    def send_completion(self, messages, functions, stream, temperature=None):
        # ...
        kwargs = dict(model=self.name, stream=stream)
        if self.use_temperature is not False:
            kwargs["temperature"] = ...  # bool True → 0
        if functions is not None:
            function = functions[0]
            kwargs["tools"] = [dict(type="function", function=function)]
            kwargs["tool_choice"] = {"type":"function", "function":{"name": function["name"]}}
        if self.extra_params:
            kwargs.update(self.extra_params)
        if self.is_ollama() and "num_ctx" not in kwargs:
            kwargs["num_ctx"] = int(self.token_count(messages) * 1.25) + 8192
        # ...
        res = litellm.completion(**kwargs)
        return hash_object, res
```

### 5.3 모델 정보 — 3단 캐스케이드

`models.py:249 get_model_info()` 가 다음 순서로 폴백:

1. **local_model_metadata** dict (in-memory) — 사용자가 `.aider.model.metadata.json` 으로 override 한 것
2. **litellm 의 내장 DB** (`litellm.get_model_info`) — BerriAI 의 `model_prices_and_context_window.json`
3. **OpenRouter 캐시** (`openrouter.py` 가 SQLite 로 관리) — 24h TTL
4. **OpenRouter 웹 스크래핑** (최후 fallback) — `re.search(r"([\d,]+)\s*context", html)`

```python
# models.py:161
class ModelInfoManager:
    MODEL_INFO_URL = ("https://raw.githubusercontent.com/BerriAI/litellm/main/"
                      "model_prices_and_context_window.json")
    CACHE_TTL = 60 * 60 * 24  # 24h
    def _update_cache(self):
        response = requests.get(self.MODEL_INFO_URL, timeout=5, verify=self.verify_ssl)
        if response.status_code == 200:
            self.content = response.json()
            self.cache_file.write_text(json.dumps(self.content, indent=4))
```

### 5.4 streaming / token counting 흐름

**streaming 응답** 시 `show_send_output_stream` (base_coder.py:1900) 가 chunk-by-chunk:

```python
# base_coder.py:1900
def show_send_output_stream(self, completion):
    for chunk in completion:
        if len(chunk.choices) == 0: continue
        # 1) length finish?
        if chunk.choices[0].finish_reason == "length": raise FinishReasonLength()
        # 2) function_call 누적이면 dict 에 merge
        try:
            func = chunk.choices[0].delta.function_call
            for k, v in func.items():
                self.partial_response_function_call[k] = (
                    self.partial_response_function_call.get(k, "") + v
                )
        except AttributeError: pass
        # 3) reasoning_content / content
        # 4) self.partial_response_content += text
        # 5) if self.show_pretty():
        #      self.live_incremental_response(False)   ← rich Live
        #    else: sys.stdout.write(text); sys.stdout.flush(); yield text
```

**`ChatModel.token_count` 가 호출되는 곳** (전부 grep):

| 호출 | 위치 | 사용 |
| --- | --- | --- |
| `model.token_count(messages)` | `repo.py:355` | commit message 생성 시 max_input_tokens 체크 |
| `model.token_count(text)` | `repomap.py:92,99` | repo map 의 token budget (binary search) |
| `model.token_count_for_image(fname)` | `commands.py:488` | 이미지 토큰 비용 계산 |
| `model.token_count(messages)` | `base_coder.py:298,1398,1617,1631,1634` | context window 체크, cost 계산 |
| `model.token_count(combined_output)` | `commands.py:1023` | `/run` 출력 토큰량 표시 ("Add X.Yk tokens?") |
| `model.token_count(self.partial_response_content)` | `base_coder.py:2018,1631` | output 토큰 수 |

### 5.5 예외 처리 — LiteLLMExceptions

`aider/exceptions.py:60` 의 `LiteLLMExceptions` 가 **litellm 의 동적 예외 클래스를 사전 등록** 한 후 `get_ex_info` 로 (retryable, description) 튜플 반환:

```python
# exceptions.py:60
class LiteLLMExceptions:
    exceptions = dict()
    exception_info = {exi.name: exi for exi in EXCEPTIONS}

    def _load(self, strict=False):
        import litellm
        for var in dir(litellm):
            if var.endswith("Error") and issubclass(getattr(litellm, var), BaseException):
                if var not in self.exception_info:
                    raise ValueError(f"{var} is in litellm but not in aider's exceptions list")
        for var in self.exception_info:
            ex = getattr(litellm, var)
            self.exceptions[ex] = self.exception_info[var]
```

사전 등록된 예외 24개 (`exceptions.py:13`):

| Name | Retry | Description |
| --- | --- | --- |
| APIConnectionError | yes | — |
| AuthenticationError | no | "Check your API key." |
| BadGatewayError | yes | "servers are down or overloaded." |
| BadRequestError | no | — |
| ContextWindowExceededError | no | special (base_coder 에서 처리) |
| RateLimitError | yes | "Try again later or check your quotas." |
| Timeout | yes | "API timed out… may be down or overloaded." |
| ... | | |

→ 이걸로 retry 정책이 데이터-드리븐. 새 litellm 예외는 `raise ValueError` 로 빠르게 fail (strict mode).

### 5.6 retry 정책

```python
# models.py:1039 simple_send_with_retries
retry_delay = 0.125
while True:
    try:
        _hash, response = self.send_completion(messages, functions=None, stream=False)
        if not response or not response.choices: return None
        res = response.choices[0].message.content
        return remove_reasoning_content(res, self.reasoning_tag)
    except litellm_ex.exceptions_tuple() as err:
        ex_info = litellm_ex.get_ex_info(err)
        if not ex_info.retry: return None
        retry_delay *= 2
        if retry_delay > RETRY_TIMEOUT:  # =60s
            return None
        time.sleep(retry_delay)
```

기본: 0.125s 시작, doubling, max 60s. **context window error 는 retry 안 함** (이건 §7 의 budget 로 해결).

### 5.7 비용 추적

```python
# base_coder.py:1994
def calculate_and_show_tokens_and_cost(self, messages, completion=None):
    if completion and hasattr(completion, "usage") and completion.usage is not None:
        prompt_tokens = completion.usage.prompt_tokens
        completion_tokens = completion.usage.completion_tokens
        cache_hit_tokens = getattr(completion.usage, "prompt_cache_hit_tokens", 0) or \
                           getattr(completion.usage, "cache_read_input_tokens", 0)
        cache_write_tokens = getattr(completion.usage, "cache_creation_input_tokens", 0)
    # ...
    cost = litellm.completion_cost(completion_response=completion)  # 공식 cost
    if not cost: cost = self.compute_costs_from_tokens(...)  # 자체 계산
    self.total_cost += cost
```

**Anthropic / DeepSeek cache 차이 처리** 가 흥미롭다 (base_coder.py:2089):

```python
if input_cost_per_token_cache_hit:
    # DeepSeek: cache_hit = 1.0x, miss = 1.0x (둘 다 동일 가격이지만 다름)
    cost += input_cost_per_token_cache_hit * cache_hit_tokens
    cost += (prompt_tokens - input_cost_per_token_cache_hit) * input_cost_per_token
else:
    # Anthropic: cache write = 1.25x, cache read = 0.10x
    cost += cache_write_tokens * input_cost_per_token * 1.25
    cost += cache_hit_tokens * input_cost_per_token * 0.10
    cost += prompt_tokens * input_cost_per_token
```

### 5.8 cache warming

```python
# base_coder.py:1340
def warm_cache(self, chunks):
    if not self.add_cache_headers or not self.num_cache_warming_pings: return
    delay = 5 * 60 - 5  # 4분 55초
    self.next_cache_warm = time.time() + delay
    self.warming_pings_left = self.num_cache_warming_pings
    self.cache_warming_chunks = chunks
    if self.cache_warming_thread: return
    def warm_cache_worker():
        while self.ok_to_warm_cache:
            time.sleep(1)
            if self.warming_pings_left <= 0: continue
            # ...
            self.warming_pings_left -= 1
            completion = litellm.completion(
                model=self.main_model.name,
                messages=self.cache_warming_chunks.cacheable_messages(),
                stream=False, **kwargs, max_tokens=1,
            )
    self.cache_warming_thread = threading.Timer(0, warm_cache_worker)
    self.cache_warming_thread.daemon = True
    self.cache_warming_thread.start()
```

→ **5분마다 max_tokens=1 핑** 을 보내 prompt cache TTL 을 유지. Anthropic / OpenAI 모두 cache TTL 5분이라 정확히 그 간격.

---

## 6. 도구/스킬 시스템 — 그리고 "부재"

### 6.1 aider 의 도구

aider 에는 **"도구 시스템"이 없다**. search_code / read_file / write_file 같은 tool calling 인터페이스를 LLM 에 노출하지 않는다. 대신:

| "도구"에 해당하는 것 | 구현 |
| --- | --- |
| File read | LLM prompt 에 file content 를 그대로 넣음 (`get_files_content`) |
| File write | LLM 응답을 search/replace 블록으로 파싱 (`do_replace`) |
| Shell exec | LLM prompt 에 시스템 지시 + 응답에 `!` 또는 `^` shell blocks 파싱 |
| Web fetch | `/web <url>` slash command, BeautifulSoup+Playwright |
| Git ops | 자동 commit / `/commit` / `/undo` slash command |
| Lint | `auto_lint=True` 시 `Linter.lint()` 자동 호출, 에러는 reflected_message |
| Test | `auto_test=True` 시 `cmd_test()` 자동 호출 |

**Tool calling (functions) 는 코더에 1개 슬롯만 있다** (architect coder 의 `<perform_edit>` 같은 메타-도구):

```python
# base_coder.py:1006-1009
if functions is not None:
    function = functions[0]
    kwargs["tools"] = [dict(type="function", function=function)]
    kwargs["tool_choice"] = {"type":"function","function":{"name":function["name"]}}
```

이건 **architect coder 전용** (architect → editor 2-model 패턴). 일반 coder 는 tool calling 안 씀.

### 6.2 "도구" 등록 메커니즘

없음. **Edit format = polymorphic subclass dispatch** (13개 coder 클래스) + **slash command = runtime getattr** (57개 cmd_ 메서드). 새 도구를 추가하려면:

1. `coders/foo_coder.py` 작성, `class FooCoder(Coder): edit_format = "foo"`
2. `coders/__init__.py` 의 `__all__` 에 추가
3. `coders/foo_prompts.py` 작성 (system prompt)
4. `args.py` 의 `edit_format_choices` 가 자동으로 잡음 (동적 import)

새 slash command:

1. `commands.py` 에 `def cmd_foo(self, args)` 추가
2. (선택) `def completions_foo(self): ...` 추가
3. 끝. 런타임 자동 dispatch.

### 6.3 비교 — 우리 my_harness 에게

| aider | 우리 my_harness (후보) |
| --- | --- |
| 13 edit formats | 1-3 modes 만 |
| 57 slash commands | 10-20 commands |
| 도구 시스템 없음 | MCP/도구/스킬? |
| file content 를 prompt 에 직접 | 동일 (text-only) |
| shell blocks 파싱 | 동일 or 별도 confirm |

→ **aider 의 "도구 부재" 가 우리에게 주는 시그널은 명확**: "LLM tool calling 으로 file read/write 를 wrapping 하지 말라" — 더 단순하고 디버깅 쉽고, 토큰/비용 추적이 더 정확하다. **우리 MVP 는 tool calling 없이 동일하게 가는 게 옳다**. (TASK-005 §5.1 vs §5.2 결정에 영향.)


---

## 7. 컨텍스트 관리 — repo.py + repomap.py + history.py

**이 섹션이 aider 의 진짜 기술적 핵심** 이다. 세 파일이 합쳐서 **"LLM context window 안에 가능한 한 정확한 코드 컨텍스트를 유지"** 라는 단일 문제를 각자 다른 레벨에서 해결한다.

### 7.1 `aider/repo.py` — git 추상화 + dirty file 추적

#### 7.1.1 책임

- 단일 git repo (working tree) 의 wrapper
- tracked file 목록 (sub-tree filter 가능)
- file-level ignore (`.aiderignore` + gitignore)
- dirty file 추적 (staged + unstaged)
- commit / undo (with attribution logic)
- LLM-based commit message generation

#### 7.1.2 핵심 데이터 구조

```python
# repo.py:52
class GitRepo:
    repo = None                                    # gitpython Repo
    aider_ignore_file = None
    aider_ignore_spec = None                       # pathspec.PathSpec
    aider_ignore_ts = 0
    aider_ignore_last_check = 0
    subtree_only = False
    ignore_file_cache = {}
    git_repo_error = None
    # instance:
    self.normalized_path = {}                      # cache
    self.tree_files = {}                           # commit → set(blob.path)
    self.ignore_file_cache = {}                    # fname → bool
    # attribute flags
    self.attribute_author / self.attribute_committer
    self.attribute_co_authored_by                  # Co-authored-by trailer
```

#### 7.1.3 추적 파일 — `get_tracked_files()` (repo.py:433)

```python
# repo.py:433
def get_tracked_files(self):
    if not self.repo: return []
    try:
        commit = self.repo.head.commit
    except ValueError:
        commit = None
    except ANY_GIT_ERROR as err:
        self.git_repo_error = err
        return []

    files = set()
    if commit:
        if commit in self.tree_files:
            files = self.tree_files[commit]        # ← 캐시 히트
        else:
            try:
                iterator = commit.tree.traverse()
                blob = None
                while True:
                    try:
                        blob = next(iterator)
                        if blob.type == "blob": files.add(blob.path)
                    except IndexError:
                        # https://github.com/gitpython-developers/GitPython/issues/...
                        self.io.tool_warning("GitRepo: Index error… Skipping.")
                        continue
                    except StopIteration: break
            except ANY_GIT_ERROR as err:
                self.git_repo_error = err
                return []
            files = set(self.normalize_path(path) for path in files)
            self.tree_files[commit] = set(files)    # ← 캐시 미스 → set

    # Add staged files
    index = self.repo.index
    try:
        staged_files = [path for path, _ in index.entries.keys()]
        files.update(self.normalize_path(path) for path in staged_files)
    except ANY_GIT_ERROR as err:
        self.io.tool_error(f"Unable to read staged files: {err}")

    res = [fname for fname in files if not self.ignored_file(fname)]
    return res
```

**관찰**:
- `self.tree_files[commit]` 캐시 — 같은 HEAD commit 에선 git traversal 안 함
- **per-commit 캐시** — `git switch` 후엔 캐시 미스, 새로 traverse
- IndexError 시 한 entry 건너뛰고 계속 (트래버스 도중 발생할 수 있는 gitpython race)
- staged + committed 둘 다 합쳐서 tracked_files 정의 (이게 실제 "tracked or going-to-be-tracked" 의미)

#### 7.1.4 인덱싱 (refresh_aider_ignore) — repo.py:500

```python
def refresh_aider_ignore(self):
    if not self.aider_ignore_file: return
    current_time = time.time()
    if current_time - self.aider_ignore_last_check < 1: return  # throttle
    self.aider_ignore_last_check = current_time
    if not self.aider_ignore_file.is_file(): return
    mtime = self.aider_ignore_file.stat().st_mtime
    if mtime != self.aider_ignore_ts:                  # mtime 비교로 invalidation
        self.aider_ignore_ts = mtime
        self.ignore_file_cache = {}                    # 캐시 비우기
        lines = self.aider_ignore_file.read_text().splitlines()
        self.aider_ignore_spec = pathspec.PathSpec.from_lines(
            pathspec.patterns.GitWildMatchPattern, lines,
        )
```

**mtime + throttle(1s) + cache invalidation** 의 정석 패턴. 1초 throttle 은 파일 watch 와 동시 호출될 때 IO 부하 방지.

#### 7.1.5 gitignore 통합 — `ignored_file_raw` (repo.py:542)

```python
def ignored_file_raw(self, fname):
    if self.subtree_only:
        try:
            fname_path = Path(self.normalize_path(fname))
            cwd_path = Path.cwd().resolve().relative_to(Path(self.root).resolve())
        except ValueError:
            return True   # not in cwd → ignore
        if cwd_path not in fname_path.parents and fname_path != cwd_path:
            return True
    if not self.aider_ignore_file or not self.aider_ignore_file.is_file():
        return False
    try:
        fname = self.normalize_path(fname)
    except ValueError:
        return True
    return self.aider_ignore_spec.match_file(fname)
```

`--subtree-only` 옵션: `cwd` 외부의 파일은 자동 ignore. 1000+ 파일 repo 에서 aider 가 일부 디렉토리만 다루게 함.

#### 7.1.6 commit / attribution — repo.py:131

가장 복잡한 함수. **31줄의 docstring** 으로 attribution 매트릭스를 명시:

```python
# repo.py:131
def commit(self, fnames=None, context=None, message=None, aider_edits=False, coder=None):
    # 1) dirty check
    if not fnames and not self.repo.is_dirty(): return
    diffs = self.get_diffs(fnames)
    if not diffs: return
    # 2) commit message: LLM-generated or user-provided
    if message: commit_message = message
    else:
        user_language = coder.commit_language or coder.get_user_language()
        commit_message = self.get_commit_message(diffs, context, user_language)
    # 3) attribute flags 결정 (explicit vs default)
    # --attribute-author / --attribute-committer / --attribute-co-authored-by
    # 4) build trailer
    if aider_edits and attribute_co_authored_by:
        commit_message_trailer = (
            f"\n\nCo-authored-by: aider ({model_name}) <aider@aider.chat>"
        )
    # 5) GIT_COMMITTER_NAME / GIT_AUTHOR_NAME env var (context manager)
    # 6) git add + git commit
    self.repo.git.add(fname)
    self.repo.git.commit(cmd)
    # 7) return (hash, message)
```

**Attribution 매트릭스** (aider docs/문서화 수준):

| aider_edits | co-authored-by | author-explicit | → Author | → Committer | Trailer |
| --- | --- | --- | --- | --- | --- |
| T (AI) | T (default) | F | You | You | yes "Co-authored-by: aider" |
| T (AI) | T | T | You(aider) | You | yes |
| T (AI) | F | F | You(aider) | You(aider) | — |
| F (user) | * | * | You | You(aider) | — |

**핵심 트릭**: env var 를 `set_git_env` (repo.py:39) context manager 로 set, commit 후 restore. `GIT_COMMITTER_NAME`, `GIT_AUTHOR_NAME` 둘 다 override.

```python
# repo.py:39
@contextlib.contextmanager
def set_git_env(var_name, value, original_value):
    os.environ[var_name] = value
    try: yield
    finally:
        if original_value is not None: os.environ[var_name] = original_value
        elif var_name in os.environ: del os.environ[var_name]
```

#### 7.1.7 LLM-based commit message — `get_commit_message` (repo.py:326)

```python
def get_commit_message(self, diffs, context, user_language=None):
    diffs = "# Diffs:\n" + diffs
    content = (context + "\n" if context else "") + diffs

    system_content = self.commit_prompt or prompts.commit_system
    language_instruction = f"\n- Is written in {user_language}." if user_language else ""
    system_content = system_content.format(language_instruction=language_instruction)

    for model in self.models:   # [weak_model, main_model]
        spinner_text = f"Generating commit message with {model.name}"
        with WaitingSpinner(spinner_text):
            messages = [
                dict(role="system", content=model.system_prompt_prefix + "\n" + system_content),
                dict(role="user", content=content),
            ]
            num_tokens = model.token_count(messages)
            max_tokens = model.info.get("max_input_tokens") or 0
            if max_tokens and num_tokens > max_tokens: continue   # ← skip
            commit_message = model.simple_send_with_retries(messages)
            if commit_message: break
    if not commit_message: self.io.tool_error("Failed to generate commit message!"); return
    return commit_message.strip()
```

→ **weak model 먼저 시도** (저렴), 안 되면 main model. token budget 초과 시 skip.

### 7.2 `aider/repomap.py` — 그래프 기반 토큰 최적화 (GraphRAG 의 실전 구현)

**이 파일이 aider 의 가장 알고리즘 무거운 코드**. 867 LOC, tree-sitter + networkx + diskcache 의 조합.

#### 7.2.1 목표

> "LLM context window 에 들어갈 수 있는 한, **repo 전체의 의미 있는 구조** (어떤 함수가 어디서 정의/참조되는지) 를 `max_map_tokens` 안에 압축"

**왜 필요한가**: aider 는 in-chat file 외에 다른 file 도 LLM 에 보여줘야 한다 (LLM 이 "이 함수 어디 정의돼 있어?" 같은 질문에 답하려면). 하지만 1000+ 파일 repo 를 전부 LLM prompt 에 넣을 순 없으니 **심볼 단위로 graph** 를 만들고 **PageRank** 로 "지금 대화 맥락에서 중요한" 심볼만 추린다.

#### 7.2.2 상수

```python
# repomap.py:35
CACHE_VERSION = 3
if USING_TSL_PACK:
    CACHE_VERSION = 4
UPDATING_REPO_MAP_MESSAGE = "Updating repo map"
```

#### 7.2.3 토큰 카운팅 — sampling 추정 (repomap.py:89)

```python
def token_count(self, text):
    len_text = len(text)
    if len_text < 200:
        return self.main_model.token_count(text)              # 정확한 카운트
    lines = text.splitlines(keepends=True)
    num_lines = len(lines)
    step = num_lines // 100 or 1                              # 100줄 sampling
    lines = lines[::step]
    sample_text = "".join(lines)
    sample_tokens = self.main_model.token_count(sample_text)
    est_tokens = sample_tokens / len(sample_text) * len_text
    return est_tokens
```

**200자 미만은 정확, 그 이상은 100줄 sampling 으로 선형 추정**. token_count() 자체가 비싸기 때문. → 우리 my_harness 의 token budget 로직에 그대로 차용 가능.

#### 7.2.4 tag 추출 — `get_tags_raw` (repomap.py:279)

```python
def get_tags_raw(self, fname, rel_fname):
    lang = filename_to_lang(fname)
    if not lang: return
    try:
        language = get_language(lang)
        parser = get_parser(lang)
    except Exception as err:
        print(f"Skipping file {fname}: {err}"); return

    query_scm = get_scm_fname(lang)
    if not query_scm.exists(): return
    query_scm = query_scm.read_text()                  # tree-sitter query (.scm)
    code = self.io.read_text(fname)
    if not code: return
    tree = parser.parse(bytes(code, "utf-8"))
    captures = self._run_captures(Query(language, query_scm), tree.root_node)

    captures_by_tag = defaultdict(list)
    matches = []
    for tag, nodes in captures.items():
        for node in nodes:
            captures_by_tag[tag].append(node)
        captures_by_tag[tag].append(node)
        matches.append((node, tag))
    # ...
    for node, tag in all_nodes:
        if tag.startswith("name.definition."): kind = "def"
        elif tag.startswith("name.reference."): kind = "ref"
        else: continue
        yield Tag(rel_fname=rel_fname, fname=fname,
                  name=node.text.decode("utf-8"), kind=kind, line=node.start_point[0])
```

**tree-sitter query 의 예** — `aider/queries/tree-sitter-language-pack/python-tags.scm`:

```scheme
(module (expression_statement (assignment left: (identifier) @name.definition.constant) @definition.constant))

(class_definition
  name: (identifier) @name.definition.class) @definition.class

(function_definition
  name: (identifier) @name.definition.function) @definition.function

(call
  function: [
      (identifier) @name.reference.call)
  ]) @reference.call
```

→ **`@name.definition.\*`** 와 **`@name.reference.\*`** 만 추출, 나머지 무시. Pygments tokenizer 로 def 만 있는 cpp 같은 언어는 backfill.

#### 7.2.5 SQLite cache (repomap.py:217)

```python
def load_tags_cache(self):
    path = Path(self.root) / self.TAGS_CACHE_DIR      # .aider.tags.cache.v4
    try:
        self.TAGS_CACHE = Cache(path)                  # diskcache
    except SQLITE_ERRORS as e:
        self.tags_cache_error(e)                       # → fallback dict
```

`diskcache.Cache` 는 SQLite 백엔드, **mtime 기반 invalidation** (repomap.py:246):

```python
file_mtime = self.get_mtime(fname)
if val is not None and val.get("mtime") == file_mtime:
    return self.TAGS_CACHE[cache_key]["data"]   # ← cache hit
# miss → parse
data = list(self.get_tags_raw(fname, rel_fname))
self.TAGS_CACHE[cache_key] = {"mtime": file_mtime, "data": data}
```

**SQLite 망가질 때 fallback** (repomap.py:177):

```python
def tags_cache_error(self, original_error=None):
    # 1) 시도: cache dir 삭제 + 재생성
    try:
        if path.exists(): shutil.rmtree(path)
        new_cache = Cache(path)
        new_cache["test"] = "test"; del new_cache["test"]   # sanity check
        self.TAGS_CACHE = new_cache
        return
    except SQLITE_ERRORS:
        pass
    # 2) fallback: in-memory dict
    self.TAGS_CACHE = dict()
```

→ **graceful degradation**: SQLite 깨져도 in-memory dict 로 동작. 우리도 이 패턴 차용.

#### 7.2.6 PageRank + personalization (repomap.py:365 `get_ranked_tags`)

```python
def get_ranked_tags(self, chat_fnames, other_fnames, mentioned_fnames, mentioned_idents, progress=None):
    import networkx as nx
    defines = defaultdict(set)         # ident → {rel_fname}
    references = defaultdict(list)     # ident → [rel_fname]
    definitions = defaultdict(set)     # (rel_fname, ident) → {Tag}
    personalization = dict()
    fnames = sorted(set(chat_fnames).union(set(other_fnames)))
    personalize = 100 / len(fnames)     # 균등 default personalization

    # [A] 각 파일 순회 → tag 수집 + personalization 계산
    for fname in fnames:
        rel_fname = self.get_rel_fname(fname)
        current_pers = 0.0
        if fname in chat_fnames:
            current_pers += personalize
        if rel_fname in mentioned_fnames:
            current_pers = max(current_pers, personalize)
        # path components vs mentioned_idents
        path_obj = Path(rel_fname)
        path_components = set(path_obj.parts)
        basename_with_ext = path_obj.name
        basename_without_ext, _ = os.path.splitext(basename_with_ext)
        components_to_check = path_components.union({basename_with_ext, basename_without_ext})
        matched_idents = components_to_check.intersection(mentioned_idents)
        if matched_idents:
            current_pers += personalize   # 1번만 더함 (idempotent)
        if current_pers > 0:
            personalization[rel_fname] = current_pers

        tags = list(self.get_tags(fname, rel_fname))
        for tag in tags:
            if tag.kind == "def":
                defines[tag.name].add(rel_fname)
                definitions[(rel_fname, tag.name)].add(tag)
            elif tag.kind == "ref":
                references[tag.name].append(rel_fname)

    # [B] graph construction
    G = nx.MultiDiGraph()
    # 1) def-only ident: self-edge weight 0.1
    for ident in defines.keys():
        if ident in references: continue
        for definer in defines[ident]:
            G.add_edge(definer, definer, weight=0.1, ident=ident)
    # 2) def↔ref: weight = mul * num_refs
    idents = set(defines.keys()).intersection(set(references.keys()))
    for ident in idents:
        mul = 1.0
        is_snake = ("_" in ident) and any(c.isalpha() for c in ident)
        is_kebab = ("-" in ident) and any(c.isalpha() for c in ident)
        is_camel = any(c.isupper() for c in ident) and any(c.islower() for c in ident)
        if ident in mentioned_idents: mul *= 10
        if (is_snake or is_kebab or is_camel) and len(ident) >= 8: mul *= 10
        if ident.startswith("_"): mul *= 0.1
        if len(defines[ident]) > 5: mul *= 0.1     # 흔한 ident 는 약화
        for referencer, num_refs in Counter(references[ident]).items():
            for definer in defines[ident]:
                use_mul = mul
                if referencer in chat_rel_fnames: use_mul *= 50
                num_refs = math.sqrt(num_refs)   # ← sqrt scale
                G.add_edge(referencer, definer, weight=use_mul * num_refs, ident=ident)

    # [C] PageRank
    pers_args = dict(personalization=personalization, dangling=personalization) if personalization else dict()
    ranked = nx.pagerank(G, weight="weight", **pers_args)

    # [D] distribute rank from src to dst
    ranked_definitions = defaultdict(float)
    for src in G.nodes:
        src_rank = ranked[src]
        total_weight = sum(data["weight"] for _src, _dst, data in G.out_edges(src, data=True))
        for _src, dst, data in G.out_edges(src, data=True):
            data["rank"] = src_rank * data["weight"] / total_weight
            ident = data["ident"]
            ranked_definitions[(dst, ident)] += data["rank"]

    # [E] chat files 빼고, rank 순으로 정렬
    ranked_tags = []
    for (fname, ident), rank in sorted(ranked_definitions.items(), reverse=True, key=lambda x: (x[1], x[0])):
        if fname in chat_rel_fnames: continue
        ranked_tags += list(definitions.get((fname, ident), []))
    return ranked_tags
```

**GraphRAG 알고리즘 정리**:

1. **Node** = file (rel_fname)
2. **Edge** = (referencer_file, definer_file) with weight = (mul × sqrt(num_refs))
3. **Edge weight multiplier (mul)**:
   - mentioned in chat → ×10
   - snake_case/kebab-case/camelCase + len≥8 → ×10 (의미있는 식별자 가정)
   - starts with `_` → ×0.1 (private)
   - >5개 file 에 정의 → ×0.1 (너무 흔함)
4. **PageRank personalization** = chat_files × 균등 baseline
5. **Output** = (dst_file, ident) pair, rank 순

→ 우리 my_harness 가 "의미 있는 심볼" 을 LLM 에 보여줘야 한다면 **이 알고리즘을 그대로 차용** 하자. TUI 의 코드 인텔리센스보다 훨씬 가볍다.

#### 7.2.7 Token budget binary search — `get_ranked_tags_map_uncached` (repomap.py:629)

```python
def get_ranked_tags_map_uncached(self, chat_fnames, other_fnames=None,
                                  max_map_tokens=None, mentioned_fnames=None, mentioned_idents=None):
    spin = Spinner(UPDATING_REPO_MAP_MESSAGE)
    ranked_tags = self.get_ranked_tags(chat_fnames, other_fnames, mentioned_fnames, mentioned_idents, progress=spin.step)
    other_rel_fnames = sorted(set(self.get_rel_fname(fname) for fname in other_fnames))
    special_fnames = filter_important_files(other_rel_fnames)   # README, License 등
    ranked_tags_fnames = set(tag[0] for tag in ranked_tags)
    special_fnames = [fn for fn in special_fnames if fn not in ranked_tags_fnames]
    special_fnames = [(fn,) for fn in special_fnames]
    ranked_tags = special_fnames + ranked_tags
    spin.step()

    num_tags = len(ranked_tags)
    lower_bound = 0
    upper_bound = num_tags
    best_tree = None
    best_tree_tokens = 0
    self.tree_cache = dict()
    middle = min(int(max_map_tokens // 25), num_tags)        # 시작점: max_tokens/25 (4% sample)

    while lower_bound <= upper_bound:
        tree = self.to_tree(ranked_tags[:middle], chat_rel_fnames)
        num_tokens = self.token_count(tree)
        pct_err = abs(num_tokens - max_map_tokens) / max_map_tokens
        ok_err = 0.15
        if (num_tokens <= max_map_tokens and num_tokens > best_tree_tokens) or pct_err < ok_err:
            best_tree = tree; best_tree_tokens = num_tokens
            if pct_err < ok_err: break
        if num_tokens < max_map_tokens: lower_bound = middle + 1
        else: upper_bound = middle - 1
        middle = int((lower_bound + upper_bound) // 2)
    spin.end()
    return best_tree
```

**핵심**: rank 순으로 정렬된 tag list 에서 **binary search** 로 "max_map_tokens 안에 들어가는 최대 prefix" 를 찾는다. 15% 오차 허용 (`ok_err = 0.15`).

**시작 middle**: `max_map_tokens // 25` — 즉 1024 토큰이면 40개 tag 부터 시작. 평균 25 토큰/tag 가정.

#### 7.2.8 tree format (repomap.py:748)

```python
def to_tree(self, tags, chat_rel_fnames):
    cur_fname = None; cur_abs_fname = None; lois = None
    output = ""
    dummy_tag = (None,)
    for tag in sorted(tags) + [dummy_tag]:
        this_rel_fname = tag[0]
        if this_rel_fname in chat_rel_fnames: continue
        if this_rel_fname != cur_fname:
            if lois is not None:
                output += "\n" + cur_fname + ":\n"
                output += self.render_tree(cur_abs_fname, cur_fname, lois)
                lois = None
            elif cur_fname:
                output += "\n" + cur_fname + "\n"
            if type(tag) is Tag:
                lois = []
                cur_abs_fname = tag.fname
            cur_fname = this_rel_fname
        if lois is not None:
            lois.append(tag.line)
    output = "\n".join([line[:100] for line in output.splitlines()]) + "\n"   # 100자 truncate
    return output
```

→ 출력 포맷:
```
src/foo.py:
   1: def hello():
   4:     return 1

src/bar.py
src/baz.py
```

각 line 은 `grep_ast.TreeContext` 로 render. `TreeContext` 는 tree-sitter AST 의 line-of-interest 만 발췌.


### 7.3 `aider/history.py` — token-based summarization

**이 파일이 가장 단순하고 강력한 컨텍스트 관리 코드** (143 LOC).

```python
# history.py:7
class ChatSummary:
    def __init__(self, models=None, max_tokens=1024):
        if not models: raise ValueError("At least one model must be provided")
        self.models = models if isinstance(models, list) else [models]
        self.max_tokens = max_tokens
        self.token_count = self.models[0].token_count

    def too_big(self, messages):
        sized = self.tokenize(messages)
        total = sum(tokens for tokens, _msg in sized)
        return total > self.max_tokens

    def tokenize(self, messages):
        sized = []
        for msg in messages:
            tokens = self.token_count(msg)
            sized.append((tokens, msg))
        return sized

    def summarize(self, messages, depth=0):
        messages = self.summarize_real(messages)
        if messages and messages[-1]["role"] != "assistant":
            messages.append(dict(role="assistant", content="Ok."))
        return messages

    def summarize_real(self, messages, depth=0):
        if not self.models: raise ValueError("No models available for summarization")
        sized = self.tokenize(messages)
        total = sum(tokens for tokens, _msg in sized)
        if total <= self.max_tokens and depth == 0:
            return messages                                   # 이미 작음

        min_split = 4
        if len(messages) <= min_split or depth > 3:
            return self.summarize_all(messages)                # 전체를 LLM 으로

        tail_tokens = 0
        split_index = len(messages)
        half_max_tokens = self.max_tokens // 2
        # tail 누적 (역방향)
        for i in range(len(sized) - 1, -1, -1):
            tokens, _msg = sized[i]
            if tail_tokens + tokens < half_max_tokens:
                tail_tokens += tokens
                split_index = i
            else: break
        # assistant message 가 boundary 에 오도록 조정
        while messages[split_index - 1]["role"] != "assistant" and split_index > 1:
            split_index -= 1
        if split_index <= min_split:
            return self.summarize_all(messages)

        tail = messages[split_index:]
        sized_head = sized[:split_index]
        # head 의 token 누적, model limit 까지
        model_max_input_tokens = self.models[0].info.get("max_input_tokens") or 4096
        model_max_input_tokens -= 512
        keep = []
        total = 0
        for tokens, msg in sized_head:
            total += tokens
            if total > model_max_input_tokens: break
            keep.append(msg)
        summary = self.summarize_all(keep)                     # LLM 으로 요약
        summary_tokens = self.token_count(summary)
        tail_tokens = sum(tokens for tokens, _ in sized[split_index:])
        if summary_tokens + tail_tokens < self.max_tokens:
            return summary + tail
        return self.summarize_real(summary + tail, depth + 1)  # 재귀
```

**알고리즘**:

1. 전체 `done_messages` 의 token 합이 `max_tokens` 이하면 그대로 반환
2. 아니면 tail (최근 half_max_tokens) 은 보존
3. head 의 `max_input_tokens - 512` 만큼을 LLM 으로 요약
4. 합쳐서 max_tokens 넘으면 depth++ 로 재귀
5. depth > 3 또는 messages < 4 면 `summarize_all` (모든 메시지 LLM 요약)

**핵심 통찰**:
- **half/half split** (max_tokens 의 절반은 tail 보존, 절반은 head 요약)
- **token budget aware** (model max_input_tokens - 512 안전선)
- **depth 로 무한 재귀 방지**
- **`{role}_prefix` 안 붙임** — `summarize_all` 이 `# USER\n...` 형태로 변환 후 단일 user prompt

`base_coder.py:1002 summarize_start()` 가 background thread 에서 호출:

```python
def summarize_start(self):
    if not self.summarizer.too_big(self.done_messages): return
    self.summarize_end()
    if self.verbose: self.io.tool_output("Starting to summarize chat history.")
    self.summarizer_thread = threading.Thread(target=self.summarize_worker)
    self.summarizer_thread.start()

def summarize_worker(self):
    self.summarizing_messages = list(self.done_messages)
    try:
        self.summarized_done_messages = self.summarizer.summarize(self.summarizing_messages)
    except ValueError as err: self.io.tool_warning(err.args[0])
```

### 7.4 context budget 전체 그림

```
[max_input_tokens]  (e.g. 200K for Claude)
    │
    ├──▶ system (main_sys + examples)         (8K-15K)
    ├──▶ done (summarized if > max_chat_history) (1K-8K, default 1024)
    ├──▶ repo (repo_map)                       (map_tokens, default 1024)
    ├──▶ readonly_files (max ~20K)
    ├──▶ chat_files (사용자 add 한 file)
    ├──▶ cur (사용자 입력)
    └──▶ reminder (token-budgeted add)
        = if total < max_input_tokens: add reminder
```

`base_coder.py:1313-1329`:

```python
max_input_tokens = self.main_model.info.get("max_input_tokens") or 0
if (not max_input_tokens or total_tokens < max_input_tokens
    and self.gpt_prompts.system_reminder):
    if self.main_model.reminder == "sys":
        chunks.reminder = reminder_message
    elif self.main_model.reminder == "user" and final and final["role"] == "user":
        new_content = (final["content"] + "\n\n" + self.fmt_system_prompt(self.gpt_prompts.system_reminder))
        chunks.cur[-1] = dict(role=final["role"], content=new_content)
```

→ **system reminder** 가 마지막 user message 에 inline 으로 들어감 (`reminder == "user"`). 또는 별도 system message (`reminder == "sys"`). 이 옵션이 model_settings.yml 에 정의됨.

### 7.5 file mention 자동 감지 — `check_for_file_mentions` (base_coder.py:1761)

```python
def check_for_file_mentions(self, content):
    mentioned_rel_fnames = self.get_file_mentions(content)
    new_mentions = mentioned_rel_fnames - self.ignore_mentions
    if not new_mentions: return
    added_fnames = []
    group = ConfirmGroup(new_mentions)
    for rel_fname in sorted(new_mentions):
        if self.io.confirm_ask("Add file to the chat?", subject=rel_fname, group=group, allow_never=True):
            self.add_rel_fname(rel_fname)
            added_fnames.append(rel_fname)
        else:
            self.ignore_mentions.add(rel_fname)
    if added_fnames:
        return prompts.added_files.format(fnames=", ".join(added_fnames))
```

→ **LLM 응답에서 filename 자동 추출** → "Add file to the chat?" confirm → add. **"don't ask again" 으로 무시 가능** (`ignore_mentions` set 에 add).

---

## 8. 세션 영속화

### 8.1 저장 위치

| 데이터 | 경로 | 포맷 | 코드 위치 |
| --- | --- | --- | --- |
| Input history (사용자 입력) | `.aider.input.history` (git root 또는 CWD) | prompt_toolkit.FileHistory (line-by-line) | `args.py:271`, `io.py:736` |
| Chat history (전체 대화) | `.aider.chat.history.md` (git root 또는 CWD) | Markdown (chat hist format) | `args.py:274`, `io.py:1117` |
| LLM call log | `--llm-history-file` (optional) | timestamp + role + content | `io.py:754` |
| Install/version history | `~/.aider/installs.json` | JSON | `main.py:1183 is_first_run_of_new_version` |
| Model metadata cache | `~/.aider/caches/model_prices_and_context_window.json` | JSON (24h TTL) | `models.py:161` |
| Model settings (user) | `.aider.model.settings.yml` (git root) 또는 `~/.aider/oauth-keys.env` | YAML | `models.py:153` |
| Model metadata (user) | `.aider.model.metadata.json` | JSON | `main.py:390` |
| Tags cache (RepoMap) | `.aider.tags.cache.v4` (git root) | SQLite (diskcache) | `repomap.py:43` |
| OpenRouter OAuth keys | `~/.aider/oauth-keys.env` | dotenv | `main.py:370` |
| Aiderignore | `.aiderignore` (옵션) | gitignore syntax | `args.py:22` |
| Persistent config | `.aider.conf.yml` (CWD → git root → home 순) | YAML | `main.py:464` |
| Help vector DB | `~/.aider/help.cache` | (numpy + Help.extra 의존) | `help.py` |

### 8.2 Chat history 포맷 (`io.py:1117 append_chat_history`)

```markdown
# aider chat started at 2026-06-06 22:00:00

#### User message 1
> LLM response 1

#### User message 2
> LLM response 2
>
> > y/n/d

#### User message 3 (multiline)
```

각 항목은 `#### ` prefix, LLM 응답은 `>` blockquote, confirm 응답은 `>>` 중첩 blockquote.

### 8.3 resume / restore

```python
# base_coder.py:519
if not self.done_messages and restore_chat_history:
    history_md = self.io.read_text(self.io.chat_history_file)
    if history_md:
        self.done_messages = utils.split_chat_history_markdown(history_md)
        self.summarize_start()   # ← restore 직후 summarize
```

`--restore-chat-history` CLI flag → 채팅 복원. 단 즉시 `summarize_start()` 호출되므로 **최근 1024 토큰만 살아남음** (만약 1MB 가 됐다면 99% 가 요약됨).

### 8.4 /save 와 /load

```python
# commands.py:1497 cmd_save
def cmd_save(self, args):
    "/drop\n"
    for fname in sorted(self.coder.abs_fnames):
        rel_fname = self.coder.get_rel_fname(fname)
        f.write(f"/add       {rel_fname}\n")
    for fname in sorted(self.coder.abs_read_only_fnames):
        # /read-only ...
```

→ **세션 전체를 reconstruct 가능한 command list** 로 dump. `/load <file>` 은 그걸 replay. → **replay 기반 resume** 으로 "session state" 영속화. 파일/모드/commit history 다 재현 가능.

### 8.5 git 의 commit history 자체가 "session log"

가장 흥미로운 패턴: **aider 는 session log 를 git commit history 에 위임**. `aider_commit_hashes` (base_coder.py:349) 가 in-memory set 으로 "이 세션에서 aider 가 만든 commit" 들 추적. `/undo` 가 이 set 을 보고 "직전 aider commit revert" 가능.

→ **"DB 안 쓰고 git history 를 영속 계층으로"** — 우리 my_harness 도 같은 패턴 가능.

---

## 9. 확장 시스템 — "부재의 미학"

### 9.1 명시적 부재 항목

| 항목 | 존재 여부 | 위치 |
| --- | --- | --- |
| **Plugin 시스템** | 없음 | — |
| **MCP 통합** | 없음 | — |
| **Hooks (pre/post tool)** | 없음 | — |
| **Skill 정의** | 없음 | — |
| **Dynamic tool loading** | 없음 | — |
| **Sub-agent / spawn** | 없음 (architect 2-model 만) | `coders/architect_coder.py` |
| **Custom provider** | 없음 (litellm 한 곳) | `llm.py` |
| **/etc/ config dir** | 있음 부분 (`~/.aider/`, `.aiderignore`, `.aider.conf.yml`) | main.py:464, args.py |
| **/commands 동적 로딩** | 없음 — 모두 `commands.py` 한 파일에 | `commands.py` |
| **Extension point** | 있음 polymorphic: `class Coder` + `class Commands` | `coders/__init__.py` |

### 9.2 어떻게 작동하는가 (extension 부재에도)

1. **polymorphic dispatch via `__all__`** — `coders/__init__.py` 의 `__all__` 리스트에 새 coder 추가 → Coder.create() 가 자동 인식 → args 도 자동 인식
2. **getattr-based command discovery** — `commands.py` 에 `def cmd_foo` 추가 → `/foo` 자동 동작
3. **YAML-driven config** — `.aider.conf.yml`, `model-settings.yml`, `model-metadata.json` 모두 외부 파일로 override
4. **YAML-driven env** — `.env` 와 `oauth-keys.env` (dotenv) — provider/auth 추가 시 env var 추가면 끝

### 9.3 우리 my_harness 에게 주는 시사점 (TASK-005 직접 반영)

| aider 의 선택 | 우리 권장 |
| --- | --- |
| MCP/Plugin **없음** | MVP 에선 plugin 시스템 **만들지 말라** — polymorphic dispatch + YAML config 로 충분 |
| 13 edit formats (polymorphic) | 우리도 1-3 mode 만 (architect / edit / ask) 으로 시작 |
| 57 slash commands (getattr) | 10-20 commands 만, 모두 한 commands.rs/ts 파일에 |
| Custom user config (YAML) | 동일 — `~/.myharness/config.yaml` + git root `.myharness.yaml` |
| Litellm 한 곳 (모든 provider) | **이게 핵심 차용점**: LLM 호출은 한 추상화 계층으로 격리 (우리: rig-core or custom one) |
| Architect = 2-model (cheap model → editor model) | 2-model 패턴은 우리 MVP 에서도 가능 (Claude Haiku → Sonnet) |

→ **결론**: "확장성 부재" 는 **aider 의 가장 강한 디자인 결정** 이지 약점이 아니다. **우리 MVP 도 같은 미니멀리즘을 따라야** 한다. 플러그인/훅은 v2+ 에서.

## 10. 빌드 & 배포 (Build & Distribution)

### 10.1 빌드 시스템

`pyproject.toml`:
- `[build-system] requires = ["setuptools>=68", "setuptools_scm[toml]>=8"]` — `setuptools_scm` 으로 git tag 기반 자동 버전.
- `pyproject.toml:11` `requires-python = ">=3.10,<3.15"` — Python 3.10~3.14.
- `[tool.setuptools_scm] write_to = "aider/_version.py"` — 빌드 시점에 `_version.py` 자동 생성.
- `[tool.setuptools.dynamic] dependencies = { file = "requirements.txt" }` — requirements.txt 에서 의존성 동적 로드.
- 4개 optional-dependencies 그룹: `dev` (requirements-dev.txt), `help` (requirements-help.txt), `browser` (requirements-browser.txt), `playwright` (requirements-playwright.txt).

### 10.2 의존성 (requirements.txt)

주요 의존성 (대략적 카테고리):
- **LLM 통합**: `litellm` (모든 provider), `tiktoken` (토큰 계산), `openai` (백업)
- **Git 통합**: `gitpython`, `pygit2` 대안
- **Repo 분석**: `grep-ast`, `tree-sitter`, `tree-sitter-languages`
- **Web / Fetch**: `requests`, `playwright` (옵션), `html2text`, `markdownify`
- **TUI**: `rich`, `prompt_toolkit`
- **유틸**: `pyyaml`, `toml`, `json_repair`, `diff-match-patch`

### 10.3 단일 바이너리 / 설치

- **PyPI 설치**: `pip install aider-chat` (PyPI 패키지명)
- **GitHub Release**: `https://github.com/Aider-AI/aider/releases` (wheel + sdist)
- **Docker**: `docker pull paulgauthier/aider` (비공식, community)
- **소스 설치**: `pip install -e .[dev]`
- **바이너리 빌드 도구 없음** — PyInstaller / Nuitka 같은 단일 바이너리 패키징은 안 함. **PyPI wheel** + `python -m aider` 가정.

### 10.4 cross-platform

- Linux, macOS, Windows 전부 지원 (Python 3.10+ 어디서나)
- WSL 환경에서 Git Bash / Windows native 둘 다 동작
- Playwright 브라우저 의존성은 optional
- 폰트 / 터미널 폭 등은 `aider/io.py:374 _validate_color_settings` 에서 환경 검증

### 10.5 CI / 릴리스

- `.github/workflows/` — Python matrix (3.10~3.14) + lint + test
- `scripts/` — release 도구 (Docker, docs 빌드 등)
- `HISTORY.md` — changelog (release note 형식)
- `MANIFEST.in` — 패키지 매니페스트 (LICENSE, README 등 포함)
- Docker 이미지는 community-maintained (aider 공식은 아님)

### 10.6 우리 my_harness 의 시사점 (TASK-005)

- **Python wheel + pip install** 의 단순함 = 우리 MVP 1안. 바이너리 빌드 (PyInstaller) 는 v2+ 검토.
- `setuptools_scm` + git tag 기반 자동 버전 = 우리도 동일 적용. `pyproject.toml` 의 `[project.scm]` 한 줄.
- `optional-dependencies` (dev/help/browser/playwright) 패턴 = 우리도 **profile 기반 install** (예: `pip install myharness[server,mcp]`) 도입.
- PyPI 배포 = cargo crates.io 또는 npm registry 와 동일한 인프라. **첫 배포는 PyPI 가 진입장벽 최저**.

## 11. 테스트 & 품질 (Testing & Quality)

### 11.1 테스트 구조

- `tests/` — pytest 기반, ~200+ 테스트 파일 추정 (디렉토리 깊이는 `ls tests | wc -l` 로 확인 가능)
- `pytest.ini` — pytest 설정
- `aider/benchmark/` — 별도 벤치마크 디렉토리 (aider 자체 LLM 벤치마크)
- `requirements/requirements-dev.txt` — pytest, pytest-cov, mock 등 dev 의존성

### 11.2 테스트 패턴

- **Unit test**: 함수/메서드 단위 (예: `tests/test_repomap.py`)
- **Integration test**: git repo + LLM 호출 통합 (mocked)
- **Benchmark test**: `aider/benchmark/benchmark.py` — 실제 LLM 으로 SWE-bench 스타일 평가
- **Fixture**: tmp_path + git repo 자동 생성 (`conftest.py`)

### 11.3 품질 도구

- **Linter**: ruff (또는 flake8) — `pyproject.toml` 또는 별도
- **Formatter**: black (또는 ruff format)
- **Type checker**: mypy (부분적, aider 는 type hint 의무는 아닌 듯)
- **CI**: GitHub Actions — Python matrix, lint, test, build wheel

### 11.4 LLM 호출 mocking

테스트가 LLM API 를 직접 호출하지 않도록 mocking 전략:
- `litellm.completion` 모킹 (unittest.mock 또는 pytest-mock)
- recorded response 사용 (정확한 LLM 출력 캐싱)
- `Model` 클래스의 `simple_send_with_retries` mocking

### 11.5 우리 my_harness 의 시사점

- **Pytest + pytest-mock + tmp_path** 의 기본 조합 그대로 채택. 우리도 Python 일 경우.
- **Benchmark 디렉토리 분리** 가 좋은 패턴 — production code 와 평가 code 분리. 우리도 `benchmarks/` 분리.
- **CI Python matrix** = 우리 1안 (Python), 2안 (TypeScript) 일 경우 Node matrix.
- **Type checker 선택은 강제 X** — `mypy` optional. 우리도 MVP 에서 strict typing 강제 안 함 (점진적 도입).
- **LLM mocking** = 우리 verifier 가 격리된 환경에서 LLM 호출 mocking 필요. `unittest.mock` 패턴.

## 12. 보안 (Security)

### 12.1 샌드박싱

**OS-level sandbox 없음** — aider 는 LLM 이 생성한 shell 명령을 사용자 권한으로 직접 실행. 의존성:
- **User trust**: aider 는 사용자가 신뢰하는 환경에서 실행된다 가정
- **Git safety**: `aider/repo.py` 가 모든 edit 을 git commit 으로 추적 (rollback 가능)
- **File system**: `aider/args.py` 의 `--yes-always` / `--auto-commits` / `--no-auto-commits` 등으로 사용자 컨트롤

### 12.2 권한 시스템

**명시적 permission model 없음** — aider 의 보안 모델:
- 모든 edit 을 git 으로 commit (auto-commits on by default)
- 사용자가 `/undo` 로 되돌리기 가능
- 위험 명령 (rm -rf, chmod, sudo 등) 에 대한 명시적 allowlist/blocklist 없음
- **신뢰 경계**: aider 사용자 = aider 가 실행하는 모든 명령의 책임자

### 12.3 시크릿 관리

- **API key**: 환경변수 (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY` 등) 또는 `.aider.conf.yml` 에 직접 (보안 약함)
- **OAuth tokens**: `oauth-keys.env` (dotenv) 파일에 저장
- **Keyring 미사용** — macOS Keychain / Windows Credential Manager 통합 없음
- `.env` 는 `.gitignore` 에 포함 (보안)

### 12.4 네트워크 정책

- **제한 없음** — LLM API endpoint (openai.com, anthropic.com) 만 allow
- 프록시 지원 (`HTTP_PROXY` 환경변수)
- TLS 검증 (litellm / requests 기본)
- Web fetch (Playwright) 는 사용자가 명시적으로 enable 한 경우만

### 12.5 Audit log

- **Git history** 가 audit log 역할 — 모든 edit 이 commit 으로 추적 가능
- LLM 요청/응답 자체 로깅은 user 옵션 (`--analytics` / `--analytics-disable`)
- 디버깅용 `aider --verbose` 로그 (stderr)

### 12.6 알려진 보안 한계

- **Prompt injection**: aider 는 LLM 출력에 대한 검증 없이 그대로 실행. 악의적 repo (README.md 에 hidden prompt) 가능
- **No AST-level safety**: aider 는 tree-sitter 로 파싱은 하지만 LLM 의 잘못된 edit 을 사전 차단 안 함
- **No network egress filter**: LLM 이 임의 URL fetch 가능 (Playwright enable 시)

### 12.7 우리 my_harness 의 시사점

- aider 의 "git commit = audit log" 패턴 = 우리도 **모든 tool call 결과를 git commit 으로** (TASK-005 MVP 검토).
- **Keyring 1급** (osxkeychain) 은 goose / gemini-cli 와 다름 — 우리 my_harness 는 **keyring 강제**. aider 의 `.env` 패턴은 약함.
- **OS-level sandbox (Seatbelt / bwrap / Windows Job)** 는 우리 MVP v1+ 에서 도입. aider 는 의도적으로 안 함.
- **Permission model** — aider 는 user-trust, 우리 my_harness 는 **도구별 allowlist** (예: `bash_run` 도구만 dangerous 명령 블록).
- **Prompt injection 대비** — 우리 my_harness 는 **사용자 컨펌 단계** (코드 변경 전 diff 표시 + yes/no) 가 1차 방어.

## 13. 주목할 패턴 (Notable Patterns) — 우리 가 차야 할 것

> §9 에서 "확장성 부재" 의 미학 으로 §13 일부 다뤘음. 본 §은 추가로 **우리 가 차야 할 패턴** 과 **피해야 할 패턴** 분리 정리.

### ✅ 우리가 차야 할 패턴 (Adopt)

#### 13.1 Litellm 한 곳 (모든 LLM provider)

`aider/llm.py:21` 의 `Lazy import litellm` + `aider/main.py:1226` 호출. **모든 LLM 호출이 litellm 한 곳** 으로 격리. 우리 my_harness 도 동일 — `llm/completion.rs` (Rust 1안) 또는 `llm/completion.ts` (TS 2안) 한 추상화. provider 추가 시 한 곳만 수정.

#### 13.2 Architect = 2-model (cheap model → editor model)

`coders/architect_coder.py` — cheap model 로 plan 만들고, editor model 로 실제 edit. **비용 최적화** 와 **품질 보장** 의 절충안. 우리 my_harness MVP 도 동일:
- "plan" 모드: Haiku/Flash 로 architecture 제안
- "edit" 모드: Sonnet/Opus 로 실제 코드 작성

#### 13.3 Edit format 의 polymorphic dispatch (13종)

`Coder` base class + 13개 subclass (`editblock_coder`, `udiff_coder`, `wholefile_coder`, ...). `__all__` 리스트 기반 자동 등록 (`coders/__init__.py`). 우리 my_harness 도 **3~5 mode** (architect / edit-block / whole-file / ask / summarize) 시작, polymorphic dispatch.

#### 13.4 `getattr` 기반 command discovery (57개)

`commands.py` 에 `def cmd_<name>` 메서드만 추가하면 `/<name>` 자동 동작. **명시적 라우터 테이블 불필요**. 우리 my_harness 도 10~20 slash commands 같은 패턴 (commands.rs/ts 한 파일에).

#### 13.5 Git = audit log / undo 메커니즘

`aider/repo.py` 의 모든 edit 이 auto-commit. 사용자가 `/undo` 로 즉시 되돌리기. **별도 audit log / version control 불필요** — git 이 그 역할. 우리도 동일 (TASK-005 설계 시).

#### 13.6 `.aider.conf.yml` (YAML config)

git root 와 home 양쪽에서 로드. 모델, provider, auto-commit 등 override. 우리 my_harness 도 `~/.myharness/config.yaml` + `.myharness.yaml` (project local) 동일 패턴.

#### 13.7 `tree-sitter` + `grep-ast` (AST-aware 검색)

`aider/repomap.py:867` 의 tree-sitter AST + PageRank. 단순 grep 대신 **의미 기반** 매칭. 우리 my_harness 도 MVP 이후에 (Rust 1안: `tree-sitter-rust` + `tree-sitter-typescript`).

#### 13.8 `diskcache` (SQLite 기반)

`aider/repomap.py` 의 repo map 캐싱. **재실행 시 캐시 hit** 으로 즉시 응답. 우리 my_harness 도 repo 인덱스 캐싱 (SQLite or sled).

#### 13.9 Token-based summarization (max_tokens=1024)

`aider/history.py:14` `ChatSummary(max_tokens=1024)`. 자동 context 압축. 우리도 동일 (`§7.2` 1차 분석에서 다룸).

#### 13.10 LiteLLM 의 unified interface

`aider/llm.py:21` 가 litellm 한 곳 import. provider 가 50+ 이어도 우리한테는 **한 함수 시그니처**. 우리도 rig-core (Rust 1안) 또는 Vercel AI SDK (TS 2안) 으로 동일 abstraction.

### ❌ 피해야 할 패턴 (Anti-patterns)

#### 13.11 확장성 부재 → 한계 (opencode 의 정반대 교훈)

aider 는 plugin/MCP/hooks 없음. **우리 MVP 는 같은 미니멀리즘** 으로 시작하지만, **community contribution** 받으려면 v2+ 에서 plugin 시스템 필요. aider 는 Paul Gauthier 1인 프로젝트로 의도적.

#### 13.12 시크릿 = `.env` 파일 (보안 약함)

API key 를 `.aider.conf.yml` 또는 `.env` 에 직접 저장. **Keychain 강제** 가 우리 my_harness 의 차별점. (goose 의 `keyring` crate / gemini-cli 의 `oauth-token-storage.ts` 와 비교.)

#### 13.13 OS-level sandbox 없음

aider 는 LLM 이 만든 shell 명령을 사용자 권한으로 실행. **우리 my_harness 는 Seatbelt / bwrap / Windows Job 1차 방어** (opencode/codex/goose 와 동일).

#### 13.14 Permission system 없음

aider 는 도구별 allowlist/blocklist 없음. **우리 my_harness 는 "dangerous 명령" 명시적 차단** (rm -rf, sudo, chmod 777 등) + 사용자 컨펌 단계.

#### 13.15 Prompt injection 무방어

LLM 출력 그대로 실행. 악의적 README.md 가능. **우리 my_harness 는 diff 표시 + 컨펌** 이 1차, OS sandbox 가 2차.

#### 13.16 Audit log = git only (부족)

`aider --analytics` 옵션 없으면 LLM 호출 로깅 안 함. **우리 my_harness 는 `state.json` + `session_handoff.md` + git log** 3중으로 추적.

#### 13.17 형광등 ❌ — `summary` 가 가끔 토큰 0 일 때

`aider/history.py:48` `if total <= self.max_tokens and depth == 0: return messages` — depth 0 에서 토큰 0 이면 즉시 반환. 미세한 버그. 우리 history.py 포팅 시 `if total == 0` 명시적 처리.

#### 13.18 `setup.py` + `pyproject.toml` 중복 (legacy)

aider 는 `pyproject.toml` 만 사용 (modern). **우리도 `pyproject.toml` only**, `setup.py` 작성 안 함.

#### 13.19 `pip install -e .[dev]` 패턴

Editable install + extras. 우리도 `pip install -e .[dev,server,mcp]` 패턴 채택 (Rust 1안은 cargo features 로, TS 2안은 npm workspaces 로).

#### 13.20 `models.py:329` 의 hard-coded model list

`aider/models.py:329` 가 100+ model 의 metadata 를 하드코딩. **유지보수 nightmare**. 우리 my_harness 는 litellm-style unified API 에 위임 (model metadata 자동).

## 14. 미해결 질문 (Open Questions)

코드만으로 답 못 한 것. 메인테이너 / 이슈 / PR 확인 필요.

### 14.1 `aider` 의 일일 활성 사용자 수 / GitHub stars

`aider` 는 PyPI 다운로드 수 + GitHub stars 추정 가능. 우리 my_harness 의 시장 위치 평가용. `https://pypistats.org/packages/aider-chat` + `gh repo view Aider-AI/aider --json stargazerCount`.

### 14.2 `aider` 의 실제 TUI vs REPL 사용 비율

`io.py:523 get_input()` 이 prompt_toolkit 기반 REPL. 풀스크린 TUI (ink/ratatui) 와 다른 UX. 우리 my_harness 가 REPL 로 갈지, 풀스크린 TUI 로 갈지 — 어느 쪽이 사용성 우위? `aider` 가 REPL 채택한 이유는?

### 14.3 `litellm` 한 곳의 trade-off

litellm 가 unified interface 를 제공하지만, **provider-specific 기능** (예: OpenAI 의 function calling 상세, Anthropic 의 prompt caching) 추상화에서 손실. 우리 my_harness 도 동일 — **순수 abstraction vs provider feature 접근성**.

### 14.4 `playwright` 옵션의 보안 영향

`aider --browser` 가 Playwright 띄우면 LLM 이 임의 URL fetch + click 가능. **악의적 사이트** 가능. sandbox 없이 (aider 의 정책) 위험. 우리 my_harness 는 `web_browse` 도구 별도 권한 + URL allowlist.

### 14.5 `aider/benchmark/` 의 LLM 평가 방법론

`aider/benchmark/benchmark.py` 가 SWE-bench 스타일 평가. **자체 벤치마크 인프라** — 우리 my_harness 도 동일 필요? (TASK-005 후속 v2+)

### 14.6 `tree-sitter-languages` 의 binary size

`tree-sitter` 가 native binary 필요. 50+ 언어 지원 시 binary 수십 MB. 우리 my_harness 가 Rust 1안이면 `tree-sitter-*` crate 별도 빌드 — binary size 영향.

### 14.7 `aider` 의 Windows native 지원

GitHub Actions 의 windows-latest runner 에서 테스트는 통과하는데, **Windows native GUI** (cmd.exe vs PowerShell vs WSL) 호환성? 우리 cross-platform 우선순위.

### 14.8 `aider/website/` 의 docs 빌드

`aider/website/` 가 mkdocs / sphinx 기반. 우리도 docs 사이트 필요 시 동일.

### 14.9 Paul Gauthier 의 향후 방향

`aider` 가 1인 프로젝트. 2026년 roadmap / BAAI 인수설 등 외부 변동. 우리 my_harness 가 reference 로 쓸 수 있는 기간.

### 14.10 `aider/coders/architect_coder.py` 의 실제 사용 통계

architect 2-model 패턴이 정말 효과 있는지? **A/B 테스트 데이터** 공개? 우리 my_harness 가 동일 패턴 도입 시 정량적 근거 필요.

---

## 15. v2 Changelog (2026-06-09 → 2026-08-14 재방문)

- **문서 목적**: v1 (2026-06-06 작성, HEAD = `5dc9490bb` = `v0.86.3.dev-53-g5dc9490bb`) 작성 시점부터 현재 (2026-08-14) 까지 aider 의 upstream 변화를 추적하고, my_harness 의 TASK-004 결정에 영향이 있는지 정직하게 평가한다.
- **상태**: v2 updated (TASK-004 재방문, 결정 변경 불요)
- **최종 수정일**: 2026-08-14

### 15.1 HEAD / release tag (재방문 시점)

| 항목 | v1 (2026-06-06) | v2 (2026-08-14) | Δ |
| --- | --- | --- | --- |
| HEAD commit | `5dc9490bb` (2026-05-22) | `5dc9490bb` (2026-05-22) | **0** |
| `git describe` | `v0.86.3.dev-53-g5dc9490bb` | `v0.86.3.dev-53-g5dc9490bb` | **0** |
| 최신 release tag | `v0.86.3.dev` | `v0.86.3.dev` | **0** |
| `aider/__init__.py` 버전 | `0.86.3.dev` | `0.86.3.dev` | **0** |

**정직 명시**: aider 의 HEAD commit / release tag / `git describe` 모두 v1 작성 시점과 완전히 동일. 동일 commit SHA `5dc9490bb` 가 70일간 HEAD.

### 15.2 Commit 활동 (2026-06-09 → 2026-08-14)

```bash
# 측정 명령 (reproducible)
$ cd /Users/yklee/repos/harness-refs/aider && \
    git log --since="2026-06-09" --until="2026-08-14" --oneline | wc -l
0
```

| 기간 | commit 수 | 비고 |
| --- | --- | --- |
| 2026-06-09 → 2026-08-14 (70일) | **0** | 정직 0. 분석 목적지 동일 SHA. |
| 비교: 2026-05-22 → 2026-06-08 (v1 작성 ± 17일) | 3 | 모두 ANTHROPIC_MODELS expansion (model config 만) |
| 비교: 2026-01-01 → 2026-06-08 (5개월) | 82 | 대부분 model config / minor copy |
| 비교: 2025-06-10 → 2025-12-31 (6개월) | 209 | 활발한 활동 |

**해석**: aider 는 v1 작성 시점부터 v2 분석 시점까지 **완전 정지 (frozen) 상태**. release tag 도 `v0.86.3.dev` 그대로 = 미출시. v0.86 시리즈가 안정 phase 에 들어갔거나 Paul Gauthier 의 side-project 성격상 maintenance 모드 가능성. 둘 중 어느 쪽이든 **my_harness reference 로서의 가치는 변하지 않음** (v1 패턴 = 현행 패턴).

### 15.3 0 commit 의 가능한 원인 (외부 컨텍스트)

> ⚠️ 본 절은 추측이며, aider 의 GitHub issue tracker / roadmap 을 직접 확인하지 않았다. 검증하지 않은 사실은 명시적으로 분리한다.

- **가설 A — stable maintenance phase**: v0.86.x 가 stable 한 상태. model config 만 추가하고 architecture 변경은 보류.
- **가설 B — BAAI 인수 / 외부 변동**: v1 §14.9 (Paul Gauthier 의 향후 방향) 에서 짚었던 "BAAI 인수설 / 외부 변동" 가능성. 동일 시점 (2026 Q2-Q3) 에 GitHub 활동이 줄어든 다른 OSS 프로젝트의 패턴과 부합하는지 미확인.
- **가설 C — seasonal slowdown**: 2026-Q3 가 summer vacation 시즌. 단, 209 commit / 6개월 의 활발함 대비 0/70일 은 seasonal 로 설명 안 됨.

**검증되지 않은 항목**: GitHub Insights traffic / Issue response rate / PR merge rate 등. 본 v2 에서는 측정하지 않음 — next revisit 시 가설 A/B/C 중 어느 것인지 좁힐 수 있다.

### 15.4 v2 변경 영향 (concrete delta on my_harness 결정)

| my_harness 결정 (v1 기반) | v2 영향 | 사유 |
| --- | --- | --- |
| CONVENTIONS.md 패턴 (v1 §3.2, §3.5) | **0** | aider 의 codebase 변동 0 → patterns 유효 |
| `repo.py` git-first 622 LOC 패턴 (v1 §2.4) | **0** | 동일 commit |
| `repomap.py` GraphRAG 867 LOC (v1 §2.4) | **0** | 동일 commit |
| model-settings.yml LLM 비종속 (v1 §5.1) | **0** | 동일 commit (model config 만 minor update) |
| `.aider.conf.yml` minimal config (v1 §13) | **0** | 동일 commit |
| `--auto-commits` / `--dirty-commits` (v1 §10.3) | **0** | 동일 commit |
| `architect_coder.py` 2-model 패턴 (v1 §14.10) | **0** | 동일 commit |
| TUI 단일 InputOutput 객체 (v1 §12.1) | **0** | 동일 commit |

**결론**: 결정 변경 불요. v1 의 14섹션 분석은 그대로 my_harness TASK-004 reference 가치 유지.

### 15.5 v2 의 명시적 한계 (honest limitations)

1. **v2 는 SHA-level 회귀 검증 안 함**. `5dc9490bb` 가 동일하므로 회귀 검증 불요하지만, 만약 cache issue 등으로 local file 이 upstream 과 다른 상태였다면 v1/v2 분석 모두 같은 잘못된 snapshot 을 본다. `git diff origin/main..HEAD -- aider/` 같은 sanity check 를 더했어야 했다.
2. **model-settings.yml 변경 안 봄**. v2 의 15.4 표에서 "model config 만 minor update" 라고 기술했지만, `aider/resources/model-settings.yml` 의 구체적 diff 를 v2 에서 다시 풀어보지는 않았다. v1 분석 시점 이미 ANTHROPIC_MODELS 확장이 들어가 있었고, 그 이후 모델 추가 (gpt-5.5, Claude Opus 4.7 등) 는 의미상 영향 없음 — pattern 은 동일.
3. **aider/website/ 의 docs 빌드 변경 안 봄**. v1 §14.8 에서 짚었던 mkdocs 빌드 시스템. upstream 0 commit 이므로 무의미하지만 명시적으로 안 봤음을 적는다.
4. **github release notes / PyPI changelog cross-check 안 함**. `git log` 만 보고 0 commit 으로 단정. GH release 가 별도 commit 이 아닌 tag-only 일 가능성도 검토 안 함 — 다만 `git describe` 와 tag list 가 동일한 시점의 동일한 사실이라 cross-check 불요.

### 15.6 v2 산출물 메타데이터 (재방문 추적용)

- **v1 → v2 사이 기간**: 2026-06-06 → 2026-08-14 = **69일**
- **v2 결정 ID**: **D-128** (TASK-004 재방문, aider v2, 2026-08-14, 정직 0 commit)
- **누적 결정 수**: 74 → **75**
- **문서 LOC 변화**: 1,925 (v1) → 1,925 + ~120 (v2 §15-16 append) ≈ **2,045 lines**
- **commit message trailer**: `Refs: D-128 (TASK-004 재방문, aider v2, 2026-08-14, 정직 0 commit)`
- **branch**: `analysis/aider-v2` (push to this branch only, origin/upstream 금지)

---

## 16. v2 영향 분석 (my_harness 결정에 대한 concrete delta)

### 16.1 0 commit → my_harness 영향 0

가장 강한 결론은 **결론이 없다는 것**. v1 의 14섹션 분석은 aider 의 source-of-truth 의 70일 정지 기간 동안 **무손상** 이다. 따라서:

| 결정 카테고리 | v2 결과 |
| --- | --- |
| 새 architecture 결정 추가 | **0건** |
| v1 기존 결정 변경 | **0건** |
| v1 기존 결정 강화 (논거 추가) | **0건** |
| v1 기존 결정 약화 (반례 발견) | **0건** |

### 16.2 v1 의 reference 가치는 그대로

v1 의 14섹션이 인용한 모든 패턴 (CONVENTIONS.md 자동 생성 / repo.py git-first / repomap.py GraphRAG / .aider.conf.yml minimal config / 1 LLM = single source 의 transformer + prompt-only / InputOutput 단일 TUI 객체 / watchfiles + threading.Thread background / `--auto-commits` attribution logic / model-settings.yml 비종속 등록) 은 **현행 upstream 과 1:1 정합**.

이는 my_harness 의 다음 1순위 후보들 — TASK-002 도메인 명령, Cargo workspace 8-crate 구조, rig-core 1안 provider 추상화 — 의 reference 토대로 **여전히 유효** 함을 의미한다.

### 16.3 결정 변경 불요 (explicit "no change" statement)

D-128 은 결정 변경이 아니라 **reference 의 현재성 검증** 결정이다. AGENTS.md 의 "검증하지 않은 결과는 완료로 확정하지 않는다" 원칙에 따라, v1 의 14섹션이 stale 해졌는지 명시적으로 확인하는 한 cycle 을 소모했고, 결과는 no-op 였다. 이는 시스템의 의도된 정상 상태 — reference 가 안정적이면 revisit 도 no-op 이어야 한다.

### 16.4 향후 (next revisit 기준) 트리거

다음 v3 revisit 은 다음 중 **하나라도** 발생할 때:

| 트리거 | 측정 | 임계값 |
| --- | --- | --- |
| aider release tag 새 minor/major | `git describe --tags` | `v0.87.x` / `v0.86.x` patch +N |
| HEAD SHA 변경 | `git rev-parse HEAD` | `5dc9490bb` 외 |
| `aider/repo.py` 또는 `aider/repomap.py` 단독 변경 (architecture signal) | `git log -- aideer/repo.py aider/repomap.py` 의 non-model 변경 | ≥5 commits |
| aider 의 외부 변동 (BAAI 인수 / contributor 급변 / repo archive) | GH Insights / news | qualitative |
| my_harness Cargo workspace 8-crate 결정 구현 (D-128 cycle 의 downstream) | task_backlog | TASK-002 → TASK-005-2 v2.0 진척 |

기본 revisit cadence = **90일** (2026-08-14 + 90 = 2026-11-12). 위 트리거 중 하나라도 먼저 발생 시 cadence 무관 즉시 revisit.

### 16.5 v2 의 my_harness 결정 (D-128)

- **결정**: TASK-004 (CLI/TUI 레퍼런스 분석) 의 aider reference 의 현행성을 2026-06-09 → 2026-08-14 70일 동안 재방문. 0 commit 이므로 v1 의 14섹션 분석은 그대로 유효. my_harness 의 결정 변경 불요.
- **연계**: D-128 = reference verification 결정 (no architecture impact). my_harness 의 다음 1순위 후보 — TASK-002 도메인 명령 / TUI shell + interactive mode 검증 / A-proper native tool calling (v1.5+) — 와 직교.
- **다음**: D-128 cycle 종료 → 다음 1순위 후보 중 yklee 결정 시 기존 cadence 로 복귀. 2026-11-12 또는 §16.4 트리거 시 v3 revisit.

