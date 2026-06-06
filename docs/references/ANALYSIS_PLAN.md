# Reference Code Deep Analysis — Plan

- 문서 목적: TASK-004 1차(`docs/REFERENCES.md` 8축 비교표) 의 후속. 5개 레퍼런스의 **실제 코드** 를 심층 분석해, `my_harness` 의 아키텍처 결정(언어 / TUI / 토폴로지 / 빌드 / 보안) 에 직접 활용 가능한 인사이트를 만든다.
- 범위: 5개 레퍼런스 × 14 섹션 표준 템플릿. 각 레퍼런스당 1개 마크다운 (대략 1,500~3,000줄).
- 대상 독자: yklee, Mavis, TASK-005 디자인 리뷰 참여자
- 상태: plan (실행 전 승인 대기)
- 최종 수정일: 2026-06-06
- 관련 문서: [REFERENCES.md (1차 비교표)](../REFERENCES.md), [PROJECT_PROFILE.md](../../docs/PROJECT_PROFILE.md), [TASK-005 (CLI/TUI 전환)](../../ai-workflow/memory/backlog/2026-06-05.md)

## 1. 목표

1차 분석은 "각각 무엇인가" 를 답했다. **2차 심층분석은 "어떻게 만들었나" 를 답한다.** 그 결과로 다음이 가능해진다:
- TASK-005 의 스택 결정(§5.1 Rust 1안 vs §5.2 TS 2안) 을 **추측이 아니라 코드 인용으로** 뒷받침
- 우리 CLI/TUI 의 **구체적 설계 패턴** 차용 (예: codex 의 10K 토큰/item 캡, gemini-cli 의 hook 시스템)
- 우리 `MiniMax.md` / `AGENTS.md` / `docs/` 의 **운영 표준** 강화
- 5개 다 보면 우리 **"없는 것을 발명"** 하지 않게 됨

## 2. 범위 — 5개 × 14 섹션

각 레퍼런스당 아래 14 섹션을 **전부 채움** (해당 없으면 "N/A" + 1줄 사유). 각 섹션은 **3~10줄** + 필요 시 코드 인용 + 파일 경로.

### 2.1 표준 섹션 목록

| # | 섹션 | 핵심 질문 | 필요 산출물 |
| --- | --- | --- | --- |
| 1 | **개요 (Overview)** | 이게 뭔가? 누구를 위한 도구? | 1문단 요약 + 라이선스 + LOC + 메인 binary |
| 2 | **아키텍처 (Architecture)** | 프로세스 모델, 모듈 경계, 핵심 추상화, 데이터 흐름 | ASCII 다이어그램 1개 + 핵심 디렉토리 트리 |
| 3 | **진입점 & CLI** | 바이너리 시작, 인자 파싱, 명령 dispatch, 서브커맨드 트리 | entry 파일 경로 + 명령 트리 (1단계) |
| 4 | **TUI/UI 구현** | 라이브러리, render loop, 상태 관리, 키 바인딩, 테마, Windows 처리 | 의존성 + render 루프 코드 발췌 |
| 5 | **LLM 통합** | provider 추상화, streaming, tool calling 프로토콜, 토큰 추적, 에러 처리 | provider trait / interface 코드 |
| 6 | **도구/스킬 시스템** | 도구 등록 메커니즘, 내장 도구 목록, 커스텀 도구 로딩, 권한 모델, 샌드박싱 | 도구 목록 표 + 등록 코드 |
| 7 | **컨텍스트 관리** | 파일 읽기 전략, repo 인덱싱(RAG/grep/AST), 토큰 예산, 요약, 잘라내기 | 핵심 알고리즘 의사코드 |
| 8 | **세션 영속화** | 저장 위치, 포맷(JSON/SQLite/custom), resume/replay, checkpoint | 스키마 / 저장 경로 |
| 9 | **확장 시스템** | plugin 포맷, MCP 통합, hooks, skill 정의, 설정 로딩 순서 | hooks 시퀀스 다이어그램 |
| 10 | **빌드 & 배포** | 빌드 시스템(cargo/npm/poetry), 단일 바이너리 전략, cross-platform 패키징, install/update 메커니즘 | 빌드 명령 + 산출물 |
| 11 | **테스트 & 품질** | 테스트 구조, unit vs integration, E2E, smoke, CI | 테스트 디렉토리 + CI 워크플로 |
| 12 | **보안** | 샌드박스(seatbelt/bwrap/Windows Job), 권한, 시크릿 관리(keychain?), 네트워크 정책, audit log | 샌드박스 호출 코드 |
| 13 | **주목할 패턴 (Notable Patterns)** | **우리 가 차야 할 것** (✅), 놀라운 것 (💡), 피해야 할 것 (❌) | 코드 인용 + 우리 적용 방안 |
| 14 | **미해결 질문 (Open Questions)** | 코드만으로 답 못 한 것. 이슈/문서/메인테이너 확인 필요 | 질문 리스트 |

### 2.2 레퍼런스별 특화 추가 (있을 때만)

- **opencode**: `packages/*` 워크스페이스, Effect 라이브러리 사용 패턴, worker/server 분리
- **aider**: `repo.py` 의 git 인덱싱, `repomap.py` 의 그래프 토큰 최적화
- **codex**: `codex-core` 비대화 방지 규율, `codex-message-history` 10K 토큰 캡
- **goose**: TUI 부재의 트레이드오프, Electron desktop + server (`goosed`) + CLI 멀티 인터페이스
- **gemini-cli**: hook 시스템 (`hookRegistry → Planner → Handler → Runner → Aggregator → Translator`), MCP OAuth, A2A-server

## 3. 산출물 형식 & 위치

### 3.1 파일 구조

```
docs/
├── REFERENCES.md                  # 1차 비교표 (이미 작성)
├── references/                   # 2차 심층분석 디렉토리 (신규)
│   ├── ANALYSIS_PLAN.md           # 본 문서
│   ├── README.md                  # 인덱스 — 5개 분석 요약 + 핵심 인사이트
│   ├── opencode.md                # opencode 심층분석
│   ├── aider.md
│   ├── codex.md
│   ├── goose.md
│   └── gemini-cli.md
```

### 3.2 마크다운 표준

- **헤더 메타**: 표준 6필드 (문서 목적 / 범위 / 대상 독자 / 상태 / 최종 수정일 / 관련 문서) — `ai-workflow/core/global_workflow_standard.md` 준수
- **코드 인용**: ```` ```언어 `path/to/file.rs:line` ```` 형태로 위치 명시
- **표**: 비교가 가능한 데이터는 표로
- **다이어그램**: ASCII art 우선, 복잡하면 mermaid

### 3.3 분량 가이드

- 각 문서 **1,500 ~ 3,000줄** (대략 30~80KB)
- 너무 짧으면 = 분석 부족, 너무 길면 = 요약 실패
- **§13 Notable Patterns** 가 우리한테 가장 중요 — 분량 충분히

## 4. 실행 방식

### 4.1 옵션 A: 워커 5명 병렬 위임 (mavis-team)

| 항목 | 설명 |
| --- | --- |
| **구조** | 메인 orchestrator (나) + 5개 worker 세션 (`general` 또는 `code-reviewer` 같은 적합한 role) |
| **분배** | 각 worker = 1개 레퍼런스, 14 섹션 표준 템플릿 따라 분석 |
| **산출물** | 각 worker 가 `docs/references/<name>.md` 작성 + PR 또는 직접 commit |
| **시간** | 병렬 = ~60~90분 (sequential = 5~7시간) |
| **검증** | 메인 orchestrator 가 5개 다 받고 README.md (인덱스) 작성 + 핵심 인사이트 추출 + yklee 리뷰 |
| **장점** | 빠름, 각 레퍼런스 깊은 분석 가능, 컨텍스트 절약 |
| **단점** | worker 가 표준 템플릿을 정확히 따를지 확신 필요. 메인이 QA 봐야 함 |

### 4.2 옵션 B: 메인이 직접 sequential 분석

| 항목 | 설명 |
| --- | --- |
| **구조** | 내가 5개 다 직접 읽고 분석 |
| **시간** | ~5~7시간 (한 세션 안에 다 못 끝낼 수 있음) |
| **장점** | 템플릿 일관성 100% 보장 |
| **단점** | 컨텍스트 폭주, 한 세션 끝나면 handoff 부담 |

### 4.3 추천: **옵션 A (워커 5명 병렬)**

근거:
- 5개 분석은 본질적으로 **독립적** (각자 다른 코드베이스)
- 시간 절약이 critical (TASK-005 가 대기 중)
- 메인이 컨텍스트 보존 → 인사이트 집계/리뷰에 집중
- 14 섹션 템플릿이 명확 → worker 가 일관성 유지 가능

worker 가 보낼 프롬프트 (요약):
- 작업: `/Users/yklee/repos/harness-refs/<name>` 의 **실제 코드** 를 14 섹션 표준 템플릿으로 분석
- 산출물: `/Users/yklee/repos/my_harness/docs/references/<name>.md`
- 형식: 표준 6필드 헤더 + 14 섹션 + §13 Notable Patterns 에 코드 인용
- 보고: 작업 끝나면 Mavis 에게 요약 + 1~3개 핵심 인사이트

## 5. 다음 단계

1. **본 계획 승인 (yklee)** — `ANALYSIS_PLAN.md` 리뷰 후 OK 사인
2. (승인 시) **5개 worker 세션 spawn** — `mavis session new <role> --title "deep-analysis-<name>"`
3. **5개 산출물 receive** — 각 worker 가 `docs/references/<name>.md` 작성 완료 시 `mavis communication` 으로 보고
4. **메인 집계** — 5개 통합 + `docs/references/README.md` 인덱스 + 핵심 인사이트 10개 추출
5. **yklee 리뷰** — §13 Notable Patterns 중심
6. **TASK-005 디자인 리뷰** — 본 분석 결과를 가지고 스택/이름/MVP 결정

## 6. 리스크

- **worker 의 깊이 편차**: 어떤 worker 는 §7 컨텍스트 관리를 5줄로 끝내고, 다른 worker 는 100줄로 풀 수 있음 → 메인이 QA 단계에서 분량/깊이 균질화
- **코드베이스 크기**: opencode / codex / gemini-cli 는 monorepo 라 분석 시간 오래 걸림. worker 에게 "핵심 부분만 깊게, 나머지는 1~2줄" 가이드 줘서 균형
- **비밀/시크릿 코드**: 일부 레퍼런스는 OAuth flow 등 시크릿 처리 코드가 있음. **시크릿 값은 분석 문서에 기록하지 않음** (구조만)
- **upstream 변동**: TASK-005 시작 전에 분석 완료되어야 의미 있음. 6/6~6/7 안에 끝내는 게 이상적

## 7. 결정 요청

1. **A (워커 병렬) vs B (직접)** — 추천: A
2. **14 섹션 템플릿** — 빠진 거 / 더 필요한 거 있나?
3. **분량 가이드 1,500~3,000줄** — OK? 더 짧게/길게?
4. **worker role** — `general` 으로 충분한지, 아니면 `code-reviewer` 같은 specialized role 선호?
5. **이름** — `docs/references/` 폴더명 OK? 다른 이름 선호?
6. **우선순위** — 5개 다 동등? 아니면 TASK-005 스택 결정에 더 필요한 거 먼저?

승인 또는 수정사항 알려주면 바로 5개 worker 세션 spawn 함.
