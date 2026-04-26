#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOOP_INTERVAL_SECONDS="${INTERVAL_SECONDS:-30}"

while true; do
  echo "[corn-dbio-kit] loop run @ $(date '+%F %T')"
  bash "$SCRIPT_DIR/util_corn-dbio-kit.sh" "$@"
  sleep "$LOOP_INTERVAL_SECONDS"
done

