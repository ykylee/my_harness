# Bootstrap Output Samples

- 문서 목적: `bootstrap_workflow_kit.py` 가 생성하는 핵심 산출물에 어떤 운영 문구가 들어가는지 샘플 형태로 보여준다.
- 범위: `ai-workflow/README.md`, `session_handoff.md`, 날짜별 backlog, `AGENTS.md`, OpenCode skill/agent 생성 문구 예시
- 대상 독자: 프로젝트 온보딩 담당자, AI workflow 설계자, 하네스 통합 담당자, 개발자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `../scripts/README.md`, `../harnesses/codex/README.md`, `../harnesses/opencode/README.md`, `../core/global_workflow_standard.md`

## 1. 왜 이 문서를 보는가

bootstrap 스크립트는 문서 세트와 하네스 오버레이를 자동 생성하지만, 실제로 어떤 운영 문구가 들어가는지는 한 번 눈으로 보는 편이 빠르다.

특히 아래 두 원칙이 생성물에 기본 포함되는지 확인하려는 경우 이 문서를 먼저 보면 된다.

- 사용자에게 직접 보이는 작업 보고와 문서 작성은 한국어를 기본으로 한다.
- 내부 사고 과정은 효율적으로 처리하고, 다음 세션에 필요한 핵심 사실만 남겨 컨텍스트 누적을 줄인다.

## 2. `ai-workflow/README.md` 예시

생성된 kit README 에는 도입 직후 해야 할 일과 함께 언어/컨텍스트 운영 원칙이 포함된다.

```md
## 6. 언어와 컨텍스트 운영 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, handoff/backlog 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 중간 분류는 모델이 가장 효율적인 형태로 처리하고, 사용자에게는 필요한 결론만 짧게 전달한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.
```

## 3. 초기 `session_handoff.md` 예시

초기 handoff 에는 다음 세션이 바로 이어받을 수 있도록 기록 원칙이 함께 들어간다.

```md
## 1.1 기록 원칙

- 이 문서는 다음 세션이 바로 이어받는 데 필요한 핵심 사실만 간결하게 남긴다.
- 사용자에게 직접 보여지는 요약과 작업 보고는 한국어를 기본으로 한다.
- 코드, 명령어, 파일 경로, 설정 key 는 필요한 경우 원문 그대로 유지한다.
- 내부 탐색 메모나 장문의 reasoning 기록은 남기지 않고, 결정과 검증 결과 중심으로 정리한다.
```

## 4. 날짜별 backlog 예시

초기 날짜별 backlog 에도 같은 방향의 기록 메모가 포함된다.

```md
## 기록 메모

- 작업 기록은 한국어를 기본으로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key 는 필요할 때 원문 그대로 유지한다.
- 다음 세션에 필요한 핵심 결정, 검증 결과, 미실행 사유만 남기고 장문의 중간 사고 기록은 생략한다.
```

## 5. `AGENTS.md` 예시

Codex 와 OpenCode 모두 공통 상위 진입점으로 쓰는 `AGENTS.md` 에도 같은 원칙이 들어간다.

```md
## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.
```

## 6. OpenCode 생성물 예시

OpenCode project-local skill 과 agent 에도 같은 정책이 영어 instruction 문장으로 포함된다. 여기서는 내부 사고의 언어를 강제하지 않고, 사용자 노출 산출물 언어와 설명 길이만 통제한다.

skill 예시:

```md
- Write user-facing status updates, work reports, and document drafts in Korean by default.
- Keep internal reasoning and intermediate classification compact, and avoid long repeated explanations to the user.
- Leave only essential facts in handoff/backlog so session context stays lean.
```

agent 예시:

```md
- Write visible work reports, summaries, and document drafts in Korean by default.
- Use concise progress updates and avoid long repeated reasoning in user-visible messages.
- Keep internal processing compact and preserve only the facts needed for the next step or next session.
- Do not call direct tools yourself. Use only task delegation for repository exploration, comparisons, implementation, checks, and draft generation.
- Treat this agent as a read-mostly coordinator with task-only execution: delegate edits, scans, log review, and validation to sub-agents instead of making exceptions for direct tool use.
```

worker 예시:

```md
- You are a code-focused workflow worker for this repository.
- Stay within the assigned write scope.
- Minimize asks during execution.
- If your harness supports per-agent model selection, use a smaller model for routine edits and reserve the main model for unusually risky or architectural code tasks.
```

문서/검증 worker 도 같은 방식으로 생성되며, 권장 운영 패턴은 `main orchestrator + small workers` 다.

## 7. 어떻게 검증하는가

이 문서의 내용은 아래 스모크 테스트와 연결된다.

- [../tests/check_bootstrap.py](../tests/check_bootstrap.py)
- [../tests/check_docs.py](../tests/check_docs.py)

즉, bootstrap 생성물에서 핵심 문구가 빠지면 테스트가 바로 실패하도록 구성돼 있다.

## 다음에 읽을 문서

- scripts 허브: [../scripts/README.md](../scripts/README.md)
- Codex 하네스 안내: [../harnesses/codex/README.md](../harnesses/codex/README.md)
- OpenCode 하네스 안내: [../harnesses/opencode/README.md](../harnesses/opencode/README.md)
- 전역 주입 정책: [../core/workflow_global_injection_policy.md](../core/workflow_global_injection_policy.md)
