# myharness plugin

Grok Build overlay plugin (D-135 / D-136).

로드 경로: `grok plugin install --trust` (설치 스크립트). `--plugin-dir` 은 `grok agent stdio` 전용이지 `grok -p` 가 아니다.

```bash
grok plugin validate plugins/myharness
../bin/myharness setup-model --print-snippet
```

- skills: env-bootstrap, code-review-best-practices, server-health-check
- hooks: PreToolUse 가드 (`rm -rf /`, deploy 명령은 `MYHARNESS_ALLOW_DEPLOY=1` 필요)
- examples/minimax.toml: `[model.minimax]` / `[model.ollama]`
