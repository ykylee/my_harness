# myharness plugin

Grok Build overlay plugin (D-135 / D-136).

로드 경로: `myharness` 래퍼가 `--plugin-dir` 로 이 디렉터리를 넘긴다 (자동 trust).

```bash
grok plugin validate plugins/myharness
../bin/myharness setup-model --print-snippet
```

- skills: env-bootstrap, code-review-best-practices, server-health-check
- hooks: PreToolUse 가드 (`rm -rf /`, deploy 명령은 `MYHARNESS_ALLOW_DEPLOY=1` 필요)
- examples/minimax.toml: `[model.minimax]` / `[model.ollama]`
