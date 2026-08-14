#!/bin/bash
# M1.3 overlay smoke. 네트워크/TUI 없음.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WRAP="${ROOT}/bin/myharness"
PLUGIN="${ROOT}/plugins/myharness"
fail=0

ok() { echo "PASS  $*"; }
bad() { echo "FAIL  $*"; fail=1; }

if grok plugin validate "$PLUGIN" >/tmp/myharness-validate.out 2>&1; then
  ok "grok plugin validate"
else
  bad "grok plugin validate"
  cat /tmp/myharness-validate.out >&2
fi

if "$WRAP" --help | grep -q 'code review'; then
  ok "--help lists code review"
else
  bad "--help missing code review"
fi

if "$WRAP" --help | grep -q 'env diagnose'; then
  ok "--help lists env diagnose"
else
  bad "--help missing env diagnose"
fi

cmd="$("$WRAP" --print-cmd env diagnose)"
if [[ "$cmd" == *"--plugin-dir"* && "$cmd" == *"-p"* ]]; then
  ok "env diagnose prints grok -p --plugin-dir"
else
  bad "env diagnose cmd: $cmd"
fi

cmd="$("$WRAP" --print-cmd --model MiniMax-M3 code review src/main.rs)"
if [[ "$cmd" == *"MiniMax-M3"* && "$cmd" == *"src/main.rs"* ]]; then
  ok "code review carries model + target"
else
  bad "code review cmd: $cmd"
fi

# grok 부재 — bash 는 남기고 grok 만 숨김
tmp="$(mktemp -d)"
ln -s "$WRAP" "${tmp}/myharness"
if PATH="/usr/bin:/bin:${tmp}" MYHARNESS_GROK="" MYHARNESS_PLUGIN_DIR="$PLUGIN" "${tmp}/myharness" env diagnose >/tmp/myharness-nogrok.out 2>&1; then
  bad "missing grok should exit non-zero"
else
  if grep -F -q 'install.sh' /tmp/myharness-nogrok.out; then
    ok "missing grok → install hint"
  else
    bad "missing grok message: $(cat /tmp/myharness-nogrok.out)"
  fi
fi
rm -rf "$tmp"

if "$WRAP" setup-model --print-snippet | grep -F -q '[model.minimax]'; then
  ok "setup-model --print-snippet"
else
  bad "setup-model snippet missing [model.minimax]"
fi

cfgdir="$(mktemp -d)"
if "$WRAP" setup-model --dest "${cfgdir}/config.toml" && grep -F -q 'MINIMAX_API_KEY' "${cfgdir}/config.toml"; then
  ok "setup-model writes dest without home"
else
  bad "setup-model --dest"
fi
rm -rf "$cfgdir"

home="$(mktemp -d)"
if HOME="$home" "$WRAP" task start --id TASK-SMOKE --title "smoke" \
  && grep -F -q 'in_progress' "${home}/.myharness/handoff/tasks/TASK-SMOKE.md"; then
  ok "task start"
else
  bad "task start"
fi
if HOME="$home" "$WRAP" task end --id TASK-SMOKE --status done --summary "ok"; then
  if grep -F -q 'done' "${home}/.myharness/handoff/tasks/TASK-SMOKE.md"; then
    ok "task end"
  else
    bad "task end status not done"
  fi
else
  bad "task end"
fi
rm -rf "$home"

GUARD="${PLUGIN}/hooks/pre_tool_guard.sh"
out="$(printf '%s' '{"toolName":"run_terminal_command","toolInput":{"command":"rm -rf /"}}' | "$GUARD")"
if [[ "$out" == *'"deny"'* ]]; then
  ok "hook denies rm -rf /"
else
  bad "hook rm: $out"
fi
out="$(printf '%s' '{"toolName":"run_terminal_command","toolInput":{"command":"ls"}}' | "$GUARD")"
if [[ "$out" == *'"allow"'* ]]; then
  ok "hook allows ls"
else
  bad "hook ls: $out"
fi
out="$(printf '%s' '{"toolName":"run_terminal_command","toolInput":{"command":"terraform apply"}}' | "$GUARD")"
if [[ "$out" == *'"deny"'* ]]; then
  ok "hook denies terraform apply without confirm"
else
  bad "hook terraform: $out"
fi
out="$(printf '%s' '{"toolName":"run_terminal_command","toolInput":{"command":"terraform apply"}}' | MYHARNESS_ALLOW_DEPLOY=1 "$GUARD")"
if [[ "$out" == *'"allow"'* ]]; then
  ok "hook allows deploy when MYHARNESS_ALLOW_DEPLOY=1"
else
  bad "hook deploy allow: $out"
fi

if "$WRAP" --yes --print-cmd server deploy staging | grep -F -q -- '-p'; then
  ok "server deploy --yes --print-cmd"
else
  bad "server deploy print-cmd"
fi

if [[ "$fail" -ne 0 ]]; then
  echo "overlay smoke FAILED"
  exit 1
fi
echo "overlay smoke PASSED"
