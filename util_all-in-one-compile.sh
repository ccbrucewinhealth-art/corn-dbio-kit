#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "[corn-dbio-kit] all-in-one compile start (standalone in current project)"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
cp "$SCRIPT_DIR/target/release/corn-dbio-kit" "$SCRIPT_DIR/corn-dbio-kit"
chmod +x "$SCRIPT_DIR/corn-dbio-kit"
echo "[corn-dbio-kit] all-in-one compile done => $SCRIPT_DIR/corn-dbio-kit"

