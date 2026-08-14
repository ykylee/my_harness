#!/bin/bash
# Overlay 래퍼 + plugin 을 사용자 홈에 설치한다 (D-138 / M3.1).
# 기본: ~/.local/bin/myharness + ~/.myharness/plugins/myharness
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${MYHARNESS_PREFIX:-${HOME}/.local}"
HARNESS_HOME="${MYHARNESS_HOME:-${HOME}/.myharness}"
DRY=0
UNINSTALL=0

usage() {
  cat <<EOF
Usage: scripts/install.sh [--prefix DIR] [--home DIR] [--dry-run] [--uninstall]

  --prefix DIR   래퍼 설치 prefix (default: ~/.local → DIR/bin/myharness)
  --home DIR     래퍼 홈 (default: ~/.myharness → DIR/plugins/myharness)
  --dry-run      복사하지 않고 경로만 출력
  --uninstall    설치한 래퍼와 plugin 사본 삭제 (홈의 handoff/는 남김)

엔진 grok 는 설치하지 않는다:
  curl -fsSL https://x.ai/cli/install.sh | bash
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    --prefix)
      PREFIX="$2"
      shift 2
      ;;
    --home)
      HARNESS_HOME="$2"
      shift 2
      ;;
    --dry-run)
      DRY=1
      shift
      ;;
    --uninstall)
      UNINSTALL=1
      shift
      ;;
    *)
      echo "install.sh: 알 수 없는 인자: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

BIN_DIR="${PREFIX}/bin"
WRAP_DEST="${BIN_DIR}/myharness"
PLUGIN_SRC="${ROOT}/plugins/myharness"
PLUGIN_DEST="${HARNESS_HOME}/plugins/myharness"
WRAP_SRC="${ROOT}/bin/myharness"

[[ -f "$WRAP_SRC" && -f "${PLUGIN_SRC}/plugin.json" ]] || {
  echo "install.sh: 저장소 루트에서 실행하세요 (bin/myharness, plugins/myharness 필요)" >&2
  exit 2
}

if [[ "$UNINSTALL" -eq 1 ]]; then
  if [[ "$DRY" -eq 1 ]]; then
    echo "dry-run: rm -f ${WRAP_DEST}"
    echo "dry-run: rm -rf ${PLUGIN_DEST}"
    exit 0
  fi
  rm -f "$WRAP_DEST"
  rm -rf "$PLUGIN_DEST"
  echo "install.sh: 제거됨 ${WRAP_DEST}"
  echo "install.sh: 제거됨 ${PLUGIN_DEST}"
  echo "install.sh: ${HARNESS_HOME}/handoff 는 보존"
  exit 0
fi

if [[ "$DRY" -eq 1 ]]; then
  echo "dry-run: ${WRAP_SRC} → ${WRAP_DEST}"
  echo "dry-run: ${PLUGIN_SRC} → ${PLUGIN_DEST}"
  exit 0
fi

mkdir -p "$BIN_DIR" "$PLUGIN_DEST"
cp "$WRAP_SRC" "$WRAP_DEST"
chmod 755 "$WRAP_DEST"

# plugin 사본 (repo 와 분리). 기존 사본은 통째 교체.
rm -rf "${PLUGIN_DEST}.tmp"
cp -R "$PLUGIN_SRC" "${PLUGIN_DEST}.tmp"
# hooks 실행 비트
if [[ -d "${PLUGIN_DEST}.tmp/hooks" ]]; then
  find "${PLUGIN_DEST}.tmp/hooks" -type f -name '*.sh' -exec chmod 755 {} \;
fi
rm -rf "$PLUGIN_DEST"
mv "${PLUGIN_DEST}.tmp" "$PLUGIN_DEST"

echo "install.sh: 래퍼 → ${WRAP_DEST}"
echo "install.sh: plugin → ${PLUGIN_DEST}"

if ! command -v grok >/dev/null 2>&1; then
  echo "install.sh: 경고 — grok 가 PATH 에 없습니다."
  echo "  curl -fsSL https://x.ai/cli/install.sh | bash"
fi

case ":${PATH}:" in
  *":${BIN_DIR}:"*) ;;
  *)
    echo "install.sh: 경고 — ${BIN_DIR} 이 PATH 에 없습니다. 예:"
    echo "  export PATH=\"${BIN_DIR}:\$PATH\""
    ;;
esac

echo "install.sh: 다음 → ${WRAP_DEST} setup-model   그리고  MINIMAX_API_KEY"
