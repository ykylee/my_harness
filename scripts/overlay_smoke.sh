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

if "$WRAP" --help | grep -q '엔진 TUI'; then
  ok "--help hides engine TUI behind engine subcommand"
else
  bad "--help should mention engine TUI as opt-in"
fi

if ! "$WRAP" --help | grep -q 'grok TUI + this plugin'; then
  ok "--help no longer defaults to grok TUI"
else
  bad "--help still advertises grok TUI as default"
fi

cmd="$("$WRAP" --print-cmd env diagnose)"
if [[ "$cmd" == *"-p"* && "$cmd" == *"-m"* && "$cmd" == *"--always-approve"* ]]; then
  ok "env diagnose prints grok -p -m --always-approve"
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

# --- PR-S2: same CLI contract on in-tree Rust binary (install stays bash) ---
SURFACE_MANIFEST="${ROOT}/surface/Cargo.toml"
if cargo build --manifest-path "$SURFACE_MANIFEST" --offline --quiet; then
  SURFACE_BIN="${ROOT}/surface/target/debug/myharness"
  if [[ -x "$SURFACE_BIN" ]]; then
    if "$SURFACE_BIN" --help | grep -q 'code review'; then
      ok "surface --help lists code review"
    else
      bad "surface --help missing code review"
    fi
    if "$SURFACE_BIN" --help | grep -q 'env diagnose'; then
      ok "surface --help lists env diagnose"
    else
      bad "surface --help missing env diagnose"
    fi
    if "$SURFACE_BIN" --help | grep -q '엔진 TUI'; then
      ok "surface --help mentions 엔진 TUI"
    else
      bad "surface --help missing 엔진 TUI"
    fi
    scmd="$("$SURFACE_BIN" --print-cmd env diagnose)"
    if [[ "$scmd" == *"# no TTY, stderr piped"* && "$scmd" == *"-p"* && "$scmd" != *"--plugin-dir"* ]]; then
      ok "surface --print-cmd env diagnose"
    else
      bad "surface print-cmd: $scmd"
    fi
    scmd="$("$SURFACE_BIN" --print-cmd --model MiniMax-M3 code review src/main.rs)"
    if [[ "$scmd" == *"MiniMax-M3"* && "$scmd" == *"src/main.rs"* ]]; then
      ok "surface code review carries model + target"
    else
      bad "surface code review cmd: $scmd"
    fi
    stmp="$(mktemp -d)"
    ln -s "$SURFACE_BIN" "${stmp}/myharness"
    if PATH="/usr/bin:/bin:${stmp}" MYHARNESS_GROK="" "${stmp}/myharness" env diagnose >/tmp/myharness-surface-nogrok.out 2>&1; then
      bad "surface missing grok should exit non-zero"
    else
      if grep -F -q 'install.sh' /tmp/myharness-surface-nogrok.out; then
        ok "surface missing grok → install hint"
      else
        bad "surface missing grok: $(cat /tmp/myharness-surface-nogrok.out)"
      fi
    fi
    rm -rf "$stmp"
    if "$SURFACE_BIN" setup-model --print-snippet | grep -F -q '[model.minimax]'; then
      ok "surface setup-model --print-snippet"
    else
      bad "surface setup-model snippet"
    fi
    scfg="$(mktemp -d)"
    if "$SURFACE_BIN" setup-model --dest "${scfg}/config.toml" && grep -F -q 'MINIMAX_API_KEY' "${scfg}/config.toml"; then
      ok "surface setup-model --dest"
    else
      bad "surface setup-model --dest"
    fi
    rm -rf "$scfg"
    shome="$(mktemp -d)"
    if HOME="$shome" "$SURFACE_BIN" task start --id TASK-SMOKE --title "smoke" \
      && grep -F -q 'in_progress' "${shome}/.myharness/handoff/tasks/TASK-SMOKE.md"; then
      ok "surface task start"
    else
      bad "surface task start"
    fi
    if HOME="$shome" "$SURFACE_BIN" task end --id TASK-SMOKE --status done --summary "ok" \
      && grep -F -q 'done' "${shome}/.myharness/handoff/tasks/TASK-SMOKE.md"; then
      ok "surface task end"
    else
      bad "surface task end"
    fi
    rm -rf "$shome"
    if "$SURFACE_BIN" --yes --print-cmd server deploy staging | grep -F -q -- '-p'; then
      ok "surface server deploy --yes --print-cmd"
    else
      bad "surface server deploy print-cmd"
    fi
    if "$SURFACE_BIN" --yes --print-cmd server deploy staging | grep -F -q -- '--always-approve'; then
      ok "surface oneshot keeps --always-approve"
    else
      bad "surface oneshot missing --always-approve"
    fi
  else
    bad "surface binary missing after cargo build"
  fi
else
  bad "cargo build surface (offline)"
fi

INSTALL="${ROOT}/scripts/install.sh"
dry="$("$INSTALL" --prefix /tmp/x --home /tmp/y --dry-run)"
if [[ "$dry" == *dry-run:* ]]; then
  ok "install.sh --dry-run"
else
  bad "install.sh dry-run: $dry"
fi

stage="$(mktemp -d)"
if "$INSTALL" --prefix "${stage}/local" --home "${stage}/home/.myharness"; then
  if [[ -x "${stage}/local/bin/myharness" && -f "${stage}/home/.myharness/plugins/myharness/plugin.json" ]]; then
    ok "install.sh copies wrapper + plugin"
  else
    bad "install.sh missing dest files"
  fi
  grok_dir="$(dirname "$(command -v grok)")"
  if HOME="${stage}/home" PATH="${stage}/local/bin:${grok_dir}:/usr/bin:/bin" \
    "${stage}/local/bin/myharness" --print-cmd env diagnose | grep -F -q -- '-p'; then
    ok "installed wrapper finds plugin via --home"
  else
    bad "installed wrapper --print-cmd"
  fi
  if "$INSTALL" --prefix "${stage}/local" --home "${stage}/home/.myharness" --uninstall \
    && [[ ! -e "${stage}/local/bin/myharness" ]] \
    && [[ ! -e "${stage}/home/.myharness/plugins/myharness" ]]; then
    ok "install.sh --uninstall"
  else
    bad "install.sh uninstall"
  fi
else
  bad "install.sh failed"
fi
rm -rf "$stage"

if [[ "$fail" -ne 0 ]]; then
  echo "overlay smoke FAILED"
  exit 1
fi
echo "overlay smoke PASSED"
