#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$SCRIPT_DIR/corn-dbio-kit"

DB_TYPE="${1:-mssql}"
HOST="${2:-192.168.0.25}"
PORT="${3:-1433}"
USER_NAME="${4:-TRC201}"
PASSWORD="${5:-syscom#1}"
DB_NAME="${6:-TRC_RData}"
TABLES="${7:-.*}"
CONDITION="${8:-}"
OUTPUT_DIRECTORY="${9:-./sql}"
RECSIZE="${10:-}"
PAGE_SIZE="${11:-10000}"
PAGE_NO="${12:-}"

if [[ ! -x "$BIN" ]]; then
  echo "[corn-dbio-kit] binary not found: $BIN"
  echo "[corn-dbio-kit] please run util_compile.sh first"
  exit 1
fi

ARGS=(
  --db-type "$DB_TYPE"
  --host "$HOST"
  --port "$PORT"
  --user "$USER_NAME"
  --password "$PASSWORD"
  --db-name "$DB_NAME"
  --tables "$TABLES"
  --output-directory "$OUTPUT_DIRECTORY"
  --page-size "$PAGE_SIZE"
)

if [[ -n "$CONDITION" ]]; then
  ARGS+=(--condition "$CONDITION")
fi
if [[ -n "$RECSIZE" ]]; then
  ARGS+=(--recsize "$RECSIZE")
fi
if [[ -n "$PAGE_NO" ]]; then
  ARGS+=(--page-no "$PAGE_NO")
fi

"$BIN" "${ARGS[@]}"

