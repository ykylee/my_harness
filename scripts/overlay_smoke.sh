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
if PATH="/usr/bin:/bin:${tmp}" MYHARNESS_GROK="" "${tmp}/myharness" env diagnose >/tmp/myharness-nogrok.out 2>&1; then
  bad "missing grok should exit non-zero"
else
  if grep -F -q 'install.sh' /tmp/myharness-nogrok.out; then
    ok "missing grok → install hint"
  else
    bad "missing grok message: $(cat /tmp/myharness-nogrok.out)"
  fi
fi
rm -rf "$tmp"

if [[ "$fail" -ne 0 ]]; then
  echo "overlay smoke FAILED"
  exit 1
fi
echo "overlay smoke PASSED"
