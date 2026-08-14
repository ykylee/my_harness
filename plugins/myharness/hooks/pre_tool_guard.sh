#!/bin/bash
# PreToolUse gate. stdout = {"decision":"allow"|"deny","reason":"..."}.
set -euo pipefail

payload="$(cat || true)"

deny() {
  printf '{"decision":"deny","reason":"%s"}\n' "$1"
  exit 0
}

allow() {
  printf '{"decision":"allow"}\n'
  exit 0
}

if command -v python3 >/dev/null 2>&1; then
  result="$(
    printf '%s' "$payload" | python3 -c '
import json, os, re, sys
raw = sys.stdin.read()
try:
    data = json.loads(raw) if raw.strip() else {}
except json.JSONDecodeError:
    data = {}
tool = str(data.get("toolName") or data.get("tool_name") or "")
inp = data.get("toolInput") or data.get("tool_input") or {}
if isinstance(inp, str):
    cmd = inp
elif isinstance(inp, dict):
    cmd = str(inp.get("command") or inp.get("cmd") or inp.get("command_line") or "")
else:
    cmd = ""
blob = " ".join([tool, cmd, raw])
patterns = [
    (r"rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+)?--no-preserve-root", "rm --no-preserve-root"),
    (r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+/\s*$", "rm -rf /"),
    (r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+/\s", "rm -rf /"),
    (r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+/\*", "rm -rf /*"),
    (r"mkfs\.", "mkfs"),
    (r"dd\s+.*\bof=/dev/", "dd to /dev"),
    (r":\(\)\s*\{\s*:\|:&\s*\};:", "fork bomb"),
]
for pat, label in patterns:
    if re.search(pat, blob):
        print("deny:" + label)
        sys.exit(0)
if os.environ.get("MYHARNESS_ALLOW_DEPLOY") != "1":
    if re.search(r"\b(terraform\s+apply|kubectl\s+apply|helm\s+upgrade|docker\s+stack\s+deploy|ansible-playbook)\b", cmd):
        print("deny:deploy-without-confirm")
        sys.exit(0)
print("allow")
'
  )"
  case "$result" in
    deny:*) deny "blocked: ${result#deny:}" ;;
    *) allow ;;
  esac
fi

if printf '%s' "$payload" | grep -E -q 'rm[[:space:]]+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*[[:space:]]+/($|[[:space:]]|\*)'; then
  deny "blocked: rm -rf /"
fi
allow
